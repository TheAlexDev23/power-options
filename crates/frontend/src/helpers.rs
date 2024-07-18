use std::time::Duration;

use dioxus::prelude::*;

#[component]
pub fn Dropdown(items: Vec<String>, selected: String) -> Element {
    rsx! {
        select {
            value: selected,
            for value in items {
                option {
                    value: value,
                    "{value}"
                }
            }
        }
    }
}

#[component]
pub fn OptInToggle(overwriting: bool, value: bool) -> Element {
    let mut overwriting = use_signal(|| overwriting);
    let mut value = use_signal(|| value);

    rsx! {
        div {
            class: "optin-toggle",
            select {
                onchange: move |e| {
                    overwriting.set(e.value() == "overwrite");
                },

                value: if overwriting.cloned() {
                    "overwrite"
                } else {
                    "no overwrite"
                },

                option {
                    value: "overwrite",
                    "Overwrite"
                }
                option {
                    value: "no overwrite",
                    "Don't overwrite"
                }
            }
            input {
                onchange: move |e| {
                    value.set(e.value() == "true")
                },

                class: if !overwriting.cloned() {
                    "hidden"
                } else {
                    ""
                },

                checked: value,
                r#type: "checkbox"
            }
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
