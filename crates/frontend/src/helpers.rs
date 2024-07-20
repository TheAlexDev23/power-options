use std::time::Duration;

use dioxus::prelude::*;

#[component]
pub fn Dropdown(
    name: String,
    items: Vec<String>,
    selected: String,
    disabled: Option<bool>,
    onchange: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        select {
            onchange: move |f| {
                if let Some(handler) = onchange {
                    handler.call(f.value());
                }
            },
            name: name,
            disabled: if disabled.is_some() {
                disabled.unwrap()
            } else {
                false
            },
            for value in items {
                option {
                    initial_selected: selected == value,
                    "{value}"
                }
            }
        }
    }
}

#[component]
pub fn Toggle(mut val: Signal<bool>, initial: bool) -> Element {
    val.set(initial);
    rsx! {
        input {
            onchange: move |e| {
                val.set(e.value() == "true");
            },
            initial_checked: initial,
            r#type: "checkbox"
        }
    }
}

pub async fn wait_for_msg<T>(rx: &mut UnboundedReceiver<T>) -> T {
    loop {
        if let Ok(Some(msg)) = rx.try_next() {
            return msg;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

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
