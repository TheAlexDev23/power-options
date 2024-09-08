use super::{
    components::Dropdown,
    toggleable_types::{ToggleableBool, ToggleableInt, ToggleableString, ToggleableWhiteBlackList},
    TooltipDirection,
};

use dioxus::prelude::*;
use power_daemon::WhiteBlackListType;

#[component]
pub fn ToggleableNumericField(
    name: String,
    tooltip: Option<String>,
    disabled: Option<bool>,
    value: ToggleableInt,
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
            if tooltip.is_some() {
                span {
                    class: "tooltip",
                    class: TooltipDirection::Left.to_class_name(),
                    "{tooltip.clone().unwrap()}"
                }
            }
            input {
                class: "numeric-input",
                r#type: "text",
                onchange: move |v| {
                    value.1.set(v.value().parse().unwrap_or_default());
                },
                value: "{value.1}",
                disabled: !value.0.cloned() || disabled.unwrap_or_default()
            }
        }
    }
}

#[component]
pub fn ToggleableTextField(
    name: String,
    value: ToggleableString,
    disabled: Option<bool>,
    tooltip: Option<String>,
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
            if tooltip.is_some() {
                span {
                    class: "tooltip",
                    class: TooltipDirection::Left.to_class_name(),
                    "{tooltip.clone().unwrap()}"
                }
            }
            input {
                r#type: "text",
                onchange: move |v| {
                    value.1.set(v.value());
                },
                value: "{value.1}",
                disabled: !value.0.cloned() || disabled.unwrap_or_default()
            }
        }
    }
}

#[component]
pub fn ToggleableDropdown(
    name: String,
    items: Vec<String>,
    value: ToggleableString,
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
        Dropdown {
            selected: value.1(),
            onchange: move |v: String| {
                value.1.set(v);
            },
            disabled: !value.0() || disabled.unwrap_or_default(),
            items,
            tooltip: dropdown_tooltip
                .as_ref()
                .map(|dropdown_tooltip| (TooltipDirection::Left, dropdown_tooltip.clone()))
        }
    }
}

#[component]
pub fn ToggleableToggle(
    name: String,
    value: ToggleableBool,
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
                span {
                    class: "tooltip",
                    class: TooltipDirection::Left.to_class_name(),
                    "{toggle_tooltip.unwrap()}"
                }
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
    value: ToggleableString,
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
pub(crate) fn ToggleableStringWhiteBlackListTypeToggle(
    value: ToggleableWhiteBlackList,
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

#[derive(Props, PartialEq, Clone)]
pub struct ToggleableWhiteBlackListProps<const C: usize> {
    pub value: ToggleableWhiteBlackList,
    pub columns: [String; C],
    pub rows: Vec<[String; C]>,
    /// The index within `columns` that will identify the element in the whiteblacklist.
    /// If multiple informational columns are available it should point to the one that will identify each element in the whiteblacklist
    pub identifying_column: usize,
}

#[component]
pub fn ToggleableWhiteBlackListDisplay<const C: usize>(
    mut props: ToggleableWhiteBlackListProps<C>,
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
                                    checked: props.value.1().items.iter().any(|i| **i == row[props.identifying_column].clone()),
                                    oninput: {
                                        let value_identifier = row[props.identifying_column].clone();
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
