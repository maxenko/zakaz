use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

pub trait Subscriber<R>: Send + Sync {
    fn call(&self, arg: R);
}

impl<R, F> Subscriber<R> for F
where
    F: Fn(R) + Send + Sync + 'static,
{
    fn call(&self, arg: R) {
        self(arg)
    }
}

pub struct SendOnlyWrapper<R> {
    f: StdMutex<Box<dyn Fn(R) + Send>>,
}

impl<R> SendOnlyWrapper<R> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(R) + Send + 'static,
    {
        Self {
            f: StdMutex::new(Box::new(f)),
        }
    }
}

impl<R> Subscriber<R> for SendOnlyWrapper<R> {
    fn call(&self, arg: R) {
        if let Ok(f) = self.f.lock() {
            f(arg)
        }
    }
}

pub struct Event<R> {
    subscribers: Mutex<Vec<Arc<dyn Subscriber<R>>>>,
}

impl<R> std::fmt::Debug for Event<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("subscribers", &"<subscribers>")
            .finish()
    }
}

impl<R> Event<R>
where
    R: 'static + Send + Clone + std::fmt::Display,
{
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
        }
    }

    pub async fn subscribe<S>(&self, subscriber: S)
    where
        S: Subscriber<R> + 'static,
    {
        let mut subscribers = self.subscribers.lock().await;
        subscribers.push(Arc::new(subscriber));
    }

    #[allow(dead_code)]
    pub async fn subscribe_fn<F>(&self, f: F)
    where
        F: Fn(R) + Send + Sync + 'static,
    {
        self.subscribe(f).await;
    }

    pub async fn subscribe_send_only<F>(&self, f: F)
    where
        F: Fn(R) + Send + 'static,
    {
        self.subscribe(SendOnlyWrapper::new(f)).await;
    }

    pub async fn notify(&self, arg: R) {
        let subscribers_snapshot = {
            let subscribers_guard = self.subscribers.lock().await;
            subscribers_guard.clone()
        };

        for subscriber in subscribers_snapshot {
            subscriber.call(arg.clone());
        }
    }
}