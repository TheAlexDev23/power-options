use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{WhiteBlackList, WhiteBlackListType};

#[derive(PartialEq, Clone)]
#[allow(unused)]
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
pub fn ToggleableRadio(
    toggle_label: String,
    list_name: String,
    value: (Signal<bool>, Signal<String>),
    items: Vec<String>,
    onchange: Option<EventHandler<String>>,
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
            label { "{toggle_label}" }
        }

        fieldset {
            legend { "{list_name}" }
            for item in items.into_iter() {
                div {
                    input {
                        r#type: "radio",
                        checked: value.1() == item,
                        disabled: !value.0(),
                        oninput: move |_| {
                            value.1.set(item.clone());
                            if let Some(onchange) = onchange {
                                onchange.call(item.clone());
                            }
                        }
                    }
                    label { "{item}" }
                }
            }
        }
    }
}

#[component]
fn ToggleableStringWhiteBlackListTypeToggle(
    value: (Signal<bool>, Signal<WhiteBlackList<String>>),
    toggle_name: String,
    onchange: Option<EventHandler<bool>>,
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
            label { "{toggle_name}" }
        }

        div { display: "flex", align_items: "center",
            label { margin_right: "4px", "Whitelist" }
            label { class: "toggle-switch",
                input {
                    r#type: "checkbox",
                    checked: value.1().list_type == WhiteBlackListType::Blacklist,
                    disabled: !value.0(),
                    oninput: move |v| {
                        value
                            .1
                            .write()
                            .list_type = if v.value() == "true" {
                            WhiteBlackListType::Blacklist
                        } else {
                            WhiteBlackListType::Whitelist
                        };
                    }
                }
                span { class: "slider" }
            }

            label { margin_left: "4px", "Blacklist" }
        }
    }
}

#[component]
pub fn ToggleableSwitchToggle(
    value: (Signal<bool>, Signal<bool>),
    toggle_name: String,
    names: Option<(String, String)>,
    onchange: Option<EventHandler<bool>>,
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
            label { "{toggle_name}" }
        }

        div { display: "flex", align_items: "center",
            if names.is_some() {
                label { margin_right: "4px", "{names.as_ref().unwrap().0.clone()}" }
            }
            label { class: "toggle-switch",
                input {
                    r#type: "checkbox",
                    checked: value.1,
                    disabled: !value.0(),
                    oninput: move |v| { value.1.set(v.value() == "true") }
                }
                span { class: "slider" }
            }
            if names.is_some() {
                label { margin_left: "4px", "{names.as_ref().unwrap().1.clone()}" }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ToggleableStringWhiteBlackListProps<const C: usize> {
    pub value: (Signal<bool>, Signal<WhiteBlackList<String>>),
    pub columns: [String; C],
    pub rows: Vec<[String; C]>,
    /// The index within `columns` that will identify the element in the whiteblacklist.
    /// If multiple informational columns are available it should point to the one that will identify each element in the whiteblacklist
    pub value_index: usize,
}

#[component]
pub fn ToggleableStringWhiteBlackList<const C: usize>(
    mut props: ToggleableStringWhiteBlackListProps<C>,
) -> Element {
    rsx! {
        div { class: "option-group",
            div { class: "option",
                ToggleableStringWhiteBlackListTypeToggle { toggle_name: "Enable custom include/exclude list", value: props.value }
            }
        }

        if props.value.0() {
            div {
                h3 { "{props.value.1().list_type.to_display_string()}" }

                table {
                    tr {
                        th { "" }
                        for column in props.columns.iter() {
                            th { "{column}" }
                        }
                    }

                    for row in props.rows.into_iter() {
                        tr {
                            td {
                                input {
                                    r#type: "checkbox",
                                    checked: props.value.1().items.iter().any(|i| **i == row[props.value_index].clone()),
                                    oninput: {
                                        let value_identifier = row[props.value_index].clone();
                                        move |v| {
                                            if v.value() == "true" {
                                                props.value.1.write().items.push(value_identifier.clone());
                                            } else {
                                                let pos = props
                                                    .value
                                                    .1()
                                                    .items
                                                    .iter()
                                                    .position(|e| *e == *value_identifier)
                                                    .unwrap();
                                                props.value.1.write().items.remove(pos);
                                            }
                                        }
                                    }
                                }
                            }

                            for item in row.iter() {
                                td { "{item}" }
                            }
                        }
                    }
                }
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
