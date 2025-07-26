#[macro_export]
macro_rules! notify_channel {
    ($reply_channel:expr, $message:expr) => {
        futures::future::OptionFuture::from($reply_channel.map(|s| async move {
            if s.is_closed() {
                err!("The mailbox channel is closed. Cannot send out message.");
            } else {
                s.send($message).await.unwrap()
            }
        })).await;
    };
}