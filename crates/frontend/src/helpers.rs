use std::time::Duration;

use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum TooltipDirection {
    AtRight,
    AtLeft,
    AtTop,
    AtBottom,
}

impl TooltipDirection {
    pub fn to_class_name(&self) -> String {
        String::from(match self {
            TooltipDirection::AtRight => "tooltip tooltip-at-right",
            TooltipDirection::AtLeft => "tooltip tooltip-at-left",
            TooltipDirection::AtTop => "tooltip tooltip-at-top",
            TooltipDirection::AtBottom => "tooltip tooltip-at-bottom",
        })
    }
}

#[component]
pub fn ToggleableNumericField(name: String, value: (Signal<bool>, Signal<i32>)) -> Element {
    rsx! {
        div {
            input {
                checked: "{value.0}",
                r#type: "checkbox",
                onchange: move |v| {
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
    disabled: Option<bool>,
    dropdown_tooltip: Option<String>,
) -> Element {
    rsx! {
        div {
            input {
                checked: "{value.0}",
                r#type: "checkbox",
                onchange: move |v| {
                    value.0.set(v.value() == "true");
                }
            }
            label { "{name}" }
        }
        div { class: "tooltip-parent",
            Dropdown {
                selected: value.1(),
                onchange: move |v: String| {
                    value.1.set(v);
                },
                disabled: !value.0() || disabled.unwrap_or_default(),
                items,
                tooltip: if let Some(ref dropdown_tooltip) = dropdown_tooltip {
                    Some((TooltipDirection::AtLeft, dropdown_tooltip.clone()))
                } else {
                    None
                }
            }
        }
    }
}

#[component]
pub fn ToggleableToggle(
    name: String,
    value: (Signal<bool>, Signal<bool>),
    disabled: Option<bool>,
    toggle_tooltip: Option<String>,
) -> Element {
    rsx! {
        div {
            input {
                checked: "{value.0}",
                r#type: "checkbox",
                onchange: move |v| {
                    value.0.set(v.value() == "true");
                }
            }
            label { "{name}" }
        }
        div { class: "tooltip-parent",
            if toggle_tooltip.is_some() {
                span { class: "tooltip tooltip-at-left", "{toggle_tooltip.unwrap()}" }
            }

            input {
                r#type: "checkbox",
                onchange: move |v| { value.1.set(v.value() == "true") },
                checked: "{value.1}",
                disabled: !value.0() || disabled.unwrap_or_default()
            }
        }
    }
}

#[component]
pub fn Dropdown(
    selected: String,
    items: Vec<String>,
    disabled: bool,
    tooltip: Option<(TooltipDirection, String)>,
    oninput: Option<EventHandler<String>>,
    onchange: Option<EventHandler<String>>,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div { class: "tooltip-parent",
            if tooltip.is_some() {
                span { class: "{tooltip.as_ref().unwrap().0.to_class_name()}",
                    "{tooltip.as_ref().unwrap().1.clone()}"
                }
            }
            select {
                oninput: move |v| {
                    if let Some(oninput) = oninput {
                        oninput.call(v.value());
                    }
                },
                onchange: move |v| {
                    if let Some(onchange) = onchange {
                        onchange.call(v.value());
                    }
                },
                onclick: move |e| {
                    if let Some(onclick) = onclick {
                        onclick.call(e);
                    }
                },
                disabled,
                for item in items {
                    option { selected: item == selected, "{item}" }
                }
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
