use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{Mutex, MutexGuard};

pub struct SyncedValue<T: PartialEq> {
    value: Mutex<Option<T>>,

    listeners: Mutex<Vec<Box<dyn Fn(Option<&T>) + Send>>>,
}

impl<T: PartialEq> SyncedValue<T> {
    pub fn new() -> SyncedValue<T> {
        SyncedValue {
            value: Mutex::new(None),
            listeners: Mutex::new(Vec::new()),
        }
    }

    pub fn is_none_blocking(&self) -> bool {
        self.value.blocking_lock().is_none()
    }
    pub async fn is_none(&self) -> bool {
        self.value.lock().await.is_none()
    }

    pub fn set_blocking(&self, new_value: T) {
        let mut value = self.value.blocking_lock();
        let new_value = Some(new_value);

        if *value == new_value {
            return;
        }

        *value = new_value;
        for listener in self.listeners.blocking_lock().iter() {
            listener(value.as_ref());
        }
    }
    pub async fn set(&self, new_value: T) {
        let mut value = self.value.lock().await;
        *value = Some(new_value);
        for listener in self.listeners.lock().await.iter() {
            listener(value.as_ref());
        }
    }

    pub fn set_mut_blocking(&self, closure: impl Fn(&mut Option<T>)) {
        closure(&mut self.value.blocking_lock())
    }
    pub async fn set_mut(&self, closure: impl Fn(MutexGuard<Option<T>>)) {
        closure(self.value.lock().await)
    }

    pub async fn listen<F>(&self, callback: F)
    where
        F: 'static + Fn(Option<&T>) + Send,
    {
        let mut listeners = self.listeners.lock().await;
        callback(self.value.lock().await.as_ref());
        listeners.push(Box::from(callback));
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
