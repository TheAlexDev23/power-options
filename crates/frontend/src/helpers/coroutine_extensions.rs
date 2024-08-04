use std::time::Duration;

use dioxus::hooks::UnboundedReceiver;

/// Awaitable task that will return when the dioxus unbounded receiver has received a message
pub async fn wait_for_msg<T>(rx: &mut UnboundedReceiver<T>) -> T {
    loop {
        if let Ok(Some(msg)) = rx.try_next() {
            return msg;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Awaitable task that will only return when the dioxus unbounded receiver has received a message that differs from `current`
pub async fn wait_for_diff_msg<T: PartialEq>(current: T, rx: &mut UnboundedReceiver<T>) -> T {
    loop {
        if let Ok(Some(msg)) = rx.try_next() {
            if msg != current {
                return msg;
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
