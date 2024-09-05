use adw::prelude::*;
use relm4::prelude::*;

pub struct Dialog {
    pub heading: String,
    pub body: String,

    pub accept_is_danger: bool,

    pub accept_label: String,
    pub deny_label: String,
}

impl Dialog {
    pub async fn show(self) -> bool {
        relm4::view! {
            dialog_widget = adw::AlertDialog {
                set_heading: Some(&self.heading),
                set_body: &self.body,

                add_response: ("accept", &self.accept_label),
                add_response: ("deny", &self.deny_label),

                #[watch]
                set_response_appearance[adw::ResponseAppearance::Destructive]: if self.accept_is_danger {
                    "accept"
                } else {
                    "deny"
                }
            }
        }

        dialog_widget.choose_future(&gtk::Window::default()).await == "accept"
    }
}
