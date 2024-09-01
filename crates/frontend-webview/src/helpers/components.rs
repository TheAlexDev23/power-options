use dioxus::prelude::*;

use super::TooltipDirection;

#[component]
pub fn ValueBindDropdown(
    value: Signal<String>,
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
                    value.set(v.value());
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
                    option { selected: item == value(), "{item}" }
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
