use std::sync::Arc;
use tokio::sync::Mutex;
use mailbox_processor::MailboxProcessor;

use crate::{
    inf, err,
    system::{
        mailbox::Mailbox,
        state::State,
        types::{RuntimeInMessage, RuntimeOutMessage, UIMessage},
        event::Event,
    },
};

#[derive(Debug)]
pub struct Runtime {
    /// Internal message processing queue
    mailbox: Arc<Mutex<MailboxProcessor<RuntimeInMessage<State>, RuntimeOutMessage<State>>>>,
    /// UI event notifier
    pub ui_events: Arc<Mutex<Event<UIMessage>>>,
}

impl Runtime {
    pub async fn new() -> Arc<Self> {
        let mailbox = Mailbox::make().await;

        let runtime = Arc::new(Self {
            mailbox,
            ui_events: Arc::new(Mutex::new(Event::new())),
        });

        let mut state = State::load_or_default().0;
        
        // Store a reference to the runtime in the state
        state.runtime = Some(runtime.clone());

        // Set new state to mailbox
        runtime.tell(RuntimeInMessage::NewState(state));
        runtime
    }

    pub fn start(self: &Arc<Self>) {
        let rt = self.clone();

        tokio::spawn(async move {
            rt.tell_cb(
                RuntimeInMessage::Start,
                Some(|msg: RuntimeOutMessage<State>| {
                    match msg {
                        RuntimeOutMessage::Started(time) => {
                            inf!("Runtime started at: {}", time);
                        }
                        _ => {
                            err!("Error starting runtime, unexpected start message");
                        }
                    }
                }),
            );
        });
    }

    /// Send message without waiting for result
    pub fn tell(self: &Arc<Self>, message: RuntimeInMessage<State>) {
        let rt = self.clone();
        tokio::spawn(async move {
            let msg = rt.ask(message).await;
            Self::log_out_msg(msg);
        });
    }

    pub fn tell_cb<F>(
        self: &Arc<Self>,
        message: RuntimeInMessage<State>,
        callback: Option<F>,
    )
    where
        F: FnOnce(RuntimeOutMessage<State>) + Send + 'static,
    {
        let rt = self.clone();
        tokio::spawn(async move {
            let msg = rt.ask(message).await;
            if let Some(cb) = callback {
                cb(msg);
            }
        });
    }

    /// Send message and wait for reply
    pub async fn ask(self: &Arc<Self>, message: RuntimeInMessage<State>) -> RuntimeOutMessage<State> {
        let _self = self.clone();
        let mb_lock = _self.mailbox.lock().await;
        let out_msg = mb_lock.send(message).await;
        match out_msg {
            Ok(msg) => msg,
            Err(e) => {
                err!("Error sending message to mailbox: {}", e);
                RuntimeOutMessage::Error(e.to_string())
            }
        }
    }

    fn log_out_msg(msg: RuntimeOutMessage<State>) {
        match msg {
            RuntimeOutMessage::Error(error) => {
                err!("{}", error);
            }
            RuntimeOutMessage::OkMsg(msg) => {
                inf!("{}", msg);
            }
            _ => {}
        }
    }
}