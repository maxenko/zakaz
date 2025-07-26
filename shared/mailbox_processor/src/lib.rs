use tokio::{ sync::mpsc::{self, Sender}, task };
use std::fmt::Display;
use futures::future::{Future};

pub enum BufferSize {
    Default,
    Size(usize),
}

impl BufferSize {
    fn unwrap_or(&self, default_value: usize) -> usize {
        match self {
            BufferSize::Default => default_value,
            BufferSize::Size(x) => *x,
        }
    }
}

#[derive(Debug)]
pub struct MailboxProcessorError {
    msg: String,
    //source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

#[derive(Debug)]
pub struct MailboxProcessor<Msg, ReplyMsg> {
    message_sender: Sender<(Msg, Option<Sender<ReplyMsg>>)>,
}

impl<Msg: 'static + Send, ReplyMsg: 'static + Send> MailboxProcessor<Msg, ReplyMsg> {
    pub async fn new<State: 'static + Send, F>(
        buffer_size: BufferSize,
        initial_state: State,
        message_processing_function: impl Fn(Msg, State, Option<Sender<ReplyMsg>>) -> F + Send + Sync + 'static,
    ) -> Self
    where
        F: Future<Output = State> + Send,

    {
        let (s, mut r) = mpsc::channel(buffer_size.unwrap_or(1_000));

        task::spawn(async move {
            let mut state = initial_state;
            // receive loop
            while let Some((msg, reply_channel)) = r.recv().await {
                state = message_processing_function(msg, state, reply_channel).await;
            }
        });

        MailboxProcessor { message_sender: s }
    }

    pub async fn send(&self, msg: Msg) -> Result<ReplyMsg, MailboxProcessorError> {
        let (s, mut r) = mpsc::channel(1);
        self.message_sender.send((msg, Some(s))).await.map_err(|_| MailboxProcessorError {
            msg: "the mailbox channel is closed send back nothing".to_owned(),
            //source: None,
        })?;

        let result = r.recv().await.ok_or(MailboxProcessorError {
            msg: "the response channel is closed (did you mean to call fire_and_forget() rather than send())".to_owned(),
            //source: None,
        });

        result
    }

    pub async fn fire_and_forget(&self, msg: Msg) -> Result<(), MailboxProcessorError> {
        self.message_sender.send((msg, None)).await.map_err(|_| MailboxProcessorError {
            msg: "the mailbox channel is closed send back nothing".to_owned(),
            //source: None,
        })
    }
}

impl Display for MailboxProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::{OptionFuture};

    #[tokio::test]
    async fn mailbox_processor_tests() {

        enum SendMessageTypes {
            Increment(i32),
            GetCurrentCount,
            Decrement(i32),
        }

        let mb = MailboxProcessor::<SendMessageTypes, i32>::new(
            BufferSize::Default,
            0,
            |msg, state, reply_channel| async move {
                match msg {
                    SendMessageTypes::Increment(x) => {
                        OptionFuture::from(reply_channel.map(|rc| async move {
                            rc.send(state + x).await.unwrap()
                        })).await;
                        state + x
                    },
                    SendMessageTypes::GetCurrentCount => {
                        OptionFuture::from(reply_channel.map(|rc| async move {
                            rc.send(state).await.unwrap()
                        })).await;
                        state
                    },
                    SendMessageTypes::Decrement(x) => {
                        OptionFuture::from(reply_channel.map(|rc| async move {
                            rc.send(state - x).await.unwrap()
                        })).await;
                        state - x
                    },
                }
            }
        ).await;

        assert_eq!(mb.send(SendMessageTypes::GetCurrentCount).await.unwrap(), 0);

        mb.fire_and_forget(SendMessageTypes::Increment(55)).await.unwrap();
        assert_eq!(mb.send(SendMessageTypes::GetCurrentCount).await.unwrap(), 55);

        mb.fire_and_forget(SendMessageTypes::Increment(55)).await.unwrap();
        assert_eq!(mb.send(SendMessageTypes::GetCurrentCount).await.unwrap(), 110);

        mb.fire_and_forget(SendMessageTypes::Decrement(10)).await.unwrap();
        assert_eq!(mb.send(SendMessageTypes::GetCurrentCount).await.unwrap(), 100);

        assert_eq!(mb.send(SendMessageTypes::Increment(55)).await.unwrap(), 155);
        assert_eq!(mb.send(SendMessageTypes::GetCurrentCount).await.unwrap(), 155);
    }
}