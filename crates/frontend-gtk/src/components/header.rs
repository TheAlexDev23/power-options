use gtk::{glib::clone, prelude::*};
use power_daemon::ProfilesInfo;
use relm4::prelude::*;

use crate::communications::daemon_control;

use super::{AppInput, AppSyncOutput};

#[derive(Debug, Clone)]
pub enum HeaderInput {
    Sync(AppSyncOutput),
    ChangingTo(Option<usize>),
    ResettingFrom(usize),
}

impl From<AppSyncOutput> for HeaderInput {
    fn from(value: AppSyncOutput) -> Self {
        Self::Sync(value)
    }
}

#[derive(Debug)]
pub struct Header {
    profiles_info: Option<ProfilesInfo>,
    changing_to: Option<usize>,
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
        let model = Header {
            profiles_info: None,
            changing_to: None,
        };

        let widgets = HeaderWidgets { header_bar: root };

        ComponentParts { model, widgets }
    }

    fn init_root() -> Self::Root {
        gtk::HeaderBar::new()
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            HeaderInput::Sync(message) => {
                if let AppSyncOutput::ProfilesInfo(profiles_info) = message {
                    self.profiles_info = (*profiles_info).clone();
                }
            }
            HeaderInput::ChangingTo(idx) => self.changing_to = idx,
            HeaderInput::ResettingFrom(_) => todo!(),
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        widgets
            .header_bar
            .set_title_widget(Some(&if let Some(ref profiles_info) = self.profiles_info {
                let ret = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                let active_profile_name = &self
                    .profiles_info
                    .as_ref()
                    .unwrap()
                    .get_active_profile()
                    .profile_name;

                for (idx, profile) in profiles_info.profiles.iter().enumerate() {
                    if let Some(changing_to) = self.changing_to {
                        if idx == changing_to {
                            ret.append(
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

                    ret.append(&toggle_button);
                }
                ret
            } else {
                let ret = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                ret.append(&gtk::Label::new(Some(
                    "Obtaining information about profiles...",
                )));
                ret
            }))
    }
}
