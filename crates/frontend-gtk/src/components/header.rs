use gtk::{glib::clone, prelude::*};
use power_daemon::ProfilesInfo;
use relm4::prelude::*;

use crate::communications::daemon_control;

use super::{AppInput, AppSyncUpdate};

#[derive(Debug, Clone)]
pub enum HeaderInput {
    Sync(AppSyncUpdate),
    ChangingTo(Option<usize>),
    ResettingFrom(usize),
    AllowApplyButton(bool),
}

impl From<AppSyncUpdate> for HeaderInput {
    fn from(value: AppSyncUpdate) -> Self {
        Self::Sync(value)
    }
}

#[derive(Debug, Default)]
pub struct Header {
    profiles_info: Option<ProfilesInfo>,
    changing_to: Option<usize>,
    enable_apply_button: bool,
}

#[derive(Debug)]
pub struct HeaderWidgets {
    header_bar: gtk::HeaderBar,
}

impl SimpleComponent for Header {
    type Input = HeaderInput;

    type Output = AppInput;

    type Init = ();

    type Root = gtk::HeaderBar;

    type Widgets = HeaderWidgets;

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Header::default();

        let widgets = HeaderWidgets { header_bar: root };

        ComponentParts { model, widgets }
    }

    fn init_root() -> Self::Root {
        gtk::HeaderBar::new()
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            HeaderInput::Sync(message) => {
                if let AppSyncUpdate::ProfilesInfo(profiles_info) = message {
                    self.profiles_info = (*profiles_info).clone();
                }
            }
            HeaderInput::ChangingTo(idx) => self.changing_to = idx,
            HeaderInput::ResettingFrom(_) => todo!(),
            HeaderInput::AllowApplyButton(v) => self.enable_apply_button = v,
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        widgets
            .header_bar
            .set_title_widget(Some(&if let Some(ref profiles_info) = self.profiles_info {
                let ret = gtk::CenterBox::new();
                ret.set_expand(true);

                let profiles_list = gtk::Box::new(gtk::Orientation::Horizontal, 0);

                profiles_list.add_css_class("linked");
                let active_profile_name = &self
                    .profiles_info
                    .as_ref()
                    .unwrap()
                    .get_active_profile()
                    .profile_name;

                for (idx, profile) in profiles_info.profiles.iter().enumerate() {
                    if let Some(changing_to) = self.changing_to {
                        if idx == changing_to {
                            profiles_list.append(
                                &gtk::Spinner::builder().spinning(true).visible(true).build(),
                            );
                            continue;
                        }
                    }

                    let toggle_button = gtk::ToggleButton::builder()
                        .label(&profile.profile_name)
                        .active(if self.changing_to.is_some() {
                            false
                        } else {
                            *active_profile_name == profile.profile_name
                        })
                        .sensitive(self.changing_to.is_none())
                        .build();

                    let profile_name = profile.profile_name.clone();
                    toggle_button.connect_clicked(clone!(
                        #[strong]
                        sender,
                        move |_| {
                            let sender = sender.clone();
                            let profile_name = profile_name.clone();
                            tokio::spawn(async move {
                                sender.input(HeaderInput::ChangingTo(Some(idx)));

                                daemon_control::set_profile_override(profile_name.clone()).await;
                                daemon_control::get_profiles_info().await;

                                sender.input(HeaderInput::ChangingTo(None));
                            });
                        }
                    ));

                    profiles_list.append(&toggle_button);
                }
                ret.set_center_widget(Some(&profiles_list));

                let button = gtk::Button::builder()
                    .label("Apply")
                    .sensitive(self.enable_apply_button)
                    .css_classes([relm4::css::SUGGESTED_ACTION])
                    .build();
                button.connect_clicked(move |_| sender.output(AppInput::ApplySettings).unwrap());

                ret.set_end_widget(Some(&button));

                ret
            } else {
                let ret = gtk::CenterBox::new();
                let center_widget = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                center_widget.append(&gtk::Label::new(Some(
                    "Obtaining information about profiles...",
                )));
                ret.set_center_widget(Some(&center_widget));
                ret
            }))
    }
}
