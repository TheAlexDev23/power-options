use dioxus::prelude::*;

use crate::helpers::Dropdown;

#[component]
pub fn CPUGroup() -> Element {
    rsx! {
        form {
            class: "cpu-form",
            div {
                class: "option-group",
                div {
                    class: "option",
                    label {
                        "Set EPP for all"
                    }
                    Dropdown {
                        items: vec!["default", "peformance", "balance_performance", "balance_power", "power"]
                    }
                }
                div {
                    class: "option",
                    label {
                        "Set governor for all"
                    }
                    Dropdown {
                        items: vec!["performance", "powersave"]
                    }
                }
            }

            div {
                class: "option-group",
                div {
                    class: "option",
                    label {
                        "Minimum frequency (MHz)"
                    }
                    input {
                        class: "numeric-input",
                        r#type: "text",
                    }
                }
                div {
                    class: "option",
                    label {
                        "Maximum frequency (MHz)"
                    }
                    input {
                        class: "numeric-input",
                        r#type: "text",
                    }
                }
            }

            div {
                class: "option-group",
                div {
                    class: "option",
                    label {
                        "Allow turbo"
                    }
                    input {
                        r#type: "checkbox"
                    }
                }
            }

            div {
                class: "confirm-buttons",
                input {
                    r#type: "submit",
                    value: "Apply"
                }
                input {
                    r#type: "reset",
                    value: "Cancel"
                }
            }
        }
    }
}
