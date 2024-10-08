use std::fmt::Debug;

use log::trace;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

pub mod extensions;
pub mod extra_bindings;

type BoxedListener<T> = Box<dyn Fn(Option<&T>) + Send>;

#[derive(Default)]
pub struct SyncedValue<T: PartialEq + Clone + Debug> {
    value: Mutex<Option<T>>,

    listener: Mutex<Option<BoxedListener<T>>>,
}

impl<T: PartialEq + Clone + Debug> SyncedValue<T> {
    pub fn new() -> SyncedValue<T> {
        SyncedValue {
            value: Mutex::new(None),
            listener: Mutex::new(None),
        }
    }

    pub async fn is_none(&self) -> bool {
        self.value.lock().await.is_none()
    }

    pub async fn get(&self) -> tokio::sync::MutexGuard<Option<T>> {
        self.value.lock().await
    }

    pub async fn set(&self, new_value: T) {
        trace!("Setting value: {new_value:#?}",);

        let mut value = self.value.lock().await;
        let old = value.clone();

        *value = Some(new_value);
        drop(value);
        self.invoke_listeners(old).await;
    }

    pub async fn set_mut(&self, closure: impl Fn(&mut Option<T>)) {
        trace!("Setting value with predicate");

        let mut value = self.value.lock().await;
        let old = value.clone();
        closure(&mut value);
        drop(value);
        self.invoke_listeners(old).await;
    }

    async fn invoke_listeners(&self, old_value: Option<T>) {
        trace!("Notifying listeners of change");

        let value = self.value.lock().await;
        if *value != old_value {
            if let Some(ref listener) = *self.listener.lock().await {
                listener(value.as_ref());
            }
        }
    }

    pub async fn set_listener<F>(&self, callback: F)
    where
        F: 'static + Fn(Option<&T>) + Send,
    {
        trace!("Subscribing to changes");

        callback(self.value.lock().await.as_ref());
        *self.listener.lock().await = Some(Box::from(callback));
    }
}

pub async fn recv_different<T: PartialEq>(receiver: &mut UnboundedReceiver<T>, current: T) -> T {
    loop {
        let next = receiver.recv().await.unwrap();
        if current != next {
            return next;
        }
    }
}
