use std::time::Duration;

use dioxus::prelude::*;

#[component]
pub fn ToggleableNumericField(name: String, value: (Signal<bool>, Signal<i32>)) -> Element {
    rsx! {
        div {
            input {
                checked: "{value.0}",
                r#type: "checkbox",
                onchange: move |v| {
                    println!("{}", v.value());
                    value.0.set(v.value() == "true");
                }
            }
            label { "{name}" }
        }
        input {
            class: "numeric-input",
            r#type: "text",
            onchange: move |v| {
                value.1.set(v.value().parse().unwrap_or_default());
            },
            value: "{value.1}",
            disabled: !value.0.cloned()
        }
    }
}

#[component]
pub fn ToggleableTextField(name: String, value: (Signal<bool>, Signal<String>)) -> Element {
    rsx! {
        div {
            input {
                checked: "{value.0}",
                r#type: "checkbox",
                onchange: move |v| {
                    println!("{}", v.value());
                    value.0.set(v.value() == "true");
                }
            }
            label { "{name}" }
        }
        input {
            r#type: "text",
            onchange: move |v| {
                value.1.set(v.value());
            },
            value: "{value.1}",
            disabled: !value.0.cloned()
        }
    }
}

#[component]
pub fn ToggleableDropdown(
    name: String,
    items: Vec<String>,
    value: (Signal<bool>, Signal<String>),
) -> Element {
    rsx! {
        div {
            input {
                checked: "{value.0}",
                r#type: "checkbox",
                onchange: move |v| {
                    println!("{}", v.value());
                    value.0.set(v.value() == "true");
                }
            }
            label { "{name}" }
        }
        select {
            onchange: move |v| {
                value.1.set(v.value());
            },
            disabled: !value.0.cloned(),
            for item in items {
                option { selected: item == *value.1.read(), "{item}" }
            }
        }
    }
}

#[component]
pub fn ToggleableToggle(name: String, value: (Signal<bool>, Signal<bool>)) -> Element {
    rsx! {
        div {
            input {
                checked: "{value.0}",
                r#type: "checkbox",
                onchange: move |v| {
                    println!("{}", v.value());
                    value.0.set(v.value() == "true");
                }
            }
            label { "{name}" }
        }
        input {
            r#type: "checkbox",
            onchange: move |v| {
                println!("{}", v.value());
                value.1.set(v.value() == "true")
            },
            checked: "{value.1}",
            disabled: !value.0.cloned()
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
