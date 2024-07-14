use dioxus::prelude::*;

#[component]
pub fn Dropdown(items: Vec<&'static str>) -> Element {
    rsx! {
        select {
            for value in items {
                option {
                    "{value}"
                }
            }
        }
    }
}
