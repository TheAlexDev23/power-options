use gtk::{glib::clone, prelude::*};
use power_daemon::ProfilesInfo;
use relm4::prelude::*;

use crate::communications::daemon_control;

use super::{AppInput, AppSyncUpdate, RootRequest};

#[derive(Debug, Clone)]
pub enum HeaderInput {
    RootRequest(RootRequest),
    ChangingTo(Option<usize>),
    AllowApplyButton(bool),
    UpdateTempOverrideResetBtn(TempOverrideResetButtonStatus),
}

impl From<RootRequest> for HeaderInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TempOverrideResetButtonStatus {
    #[default]
    Disabled,
    Loading,
    Enabled(String),
}

#[derive(Debug, Default)]
pub struct Header {
    profiles_info: Option<ProfilesInfo>,
    changing_to: Option<usize>,
    enable_apply_button: bool,
    reset_temp_override_btn_status: TempOverrideResetButtonStatus,
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
            HeaderInput::RootRequest(RootRequest::ReactToUpdate(message)) => {
                if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                    self.profiles_info = (**profiles_info).clone();
                }
                if let AppSyncUpdate::TemporaryOverride(ref temporary_override) = message {
                    if let Some(profile_name) = temporary_override.as_ref() {
                        self.reset_temp_override_btn_status =
                            TempOverrideResetButtonStatus::Enabled(profile_name.clone())
                    } else {
                        self.reset_temp_override_btn_status =
                            TempOverrideResetButtonStatus::Disabled;
                    }
                }
            }
            HeaderInput::RootRequest(_) => {}
            HeaderInput::ChangingTo(idx) => self.changing_to = idx,
            HeaderInput::AllowApplyButton(v) => self.enable_apply_button = v,
            HeaderInput::UpdateTempOverrideResetBtn(v) => self.reset_temp_override_btn_status = v,
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
                                daemon_control::get_profile_override().await;

                                sender.output(AppInput::ResetAllChanged).unwrap();
                                sender.input(HeaderInput::ChangingTo(None));
                            });
                        }
                    ));

                    profiles_list.append(&toggle_button);
                }
                ret.set_center_widget(Some(&profiles_list));

                relm4::view! {
                    end_widget = gtk::Box::new(gtk::Orientation::Horizontal, 0) {
                        gtk::Button {
                            set_label: "Apply",
                            set_sensitive: self.enable_apply_button,
                            add_css_class: relm4::css::SUGGESTED_ACTION,
                            connect_clicked[sender] => move |_| {
                                sender
                                    .output(AppInput::SetActiveGroupChanged(false))
                                    .unwrap();
                                sender
                                    .output(AppInput::SendRootRequestToActiveGroup(RootRequest::Apply))
                                    .unwrap()
                            }
                        },
                        gtk::Button {
                            set_label: "Daemon settings",
                            connect_clicked[sender] => move |_| {
                                sender.output(AppInput::ToggleSettings(true)).unwrap()
                            }
                        }
                    }
                }

                ret.set_end_widget(Some(&end_widget));

                if let TempOverrideResetButtonStatus::Enabled(ref profile_name) =
                    self.reset_temp_override_btn_status
                {
                    let reset_temporary_override_button = gtk::Button::builder()
                        .label("Reset Override")
                        .tooltip_text(format!("The current profile is currently locked to \"{}\" until next restart. Note that this is not the same as the persistent override found in daemon settings.", profile_name))
                        .build();

                    reset_temporary_override_button.connect_clicked(clone!(
                        #[strong]
                        sender,
                        move |_| {
                            let sender = sender.clone();
                            tokio::spawn(async move {
                                sender.input(HeaderInput::UpdateTempOverrideResetBtn(
                                    TempOverrideResetButtonStatus::Loading,
                                ));

                                daemon_control::remove_profile_override().await;
                                daemon_control::get_profiles_info().await;
                                daemon_control::get_profile_override().await;

                                sender.output(AppInput::ResetAllChanged).unwrap();

                                sender.input(HeaderInput::UpdateTempOverrideResetBtn(
                                    TempOverrideResetButtonStatus::Disabled,
                                ));
                            });
                        }
                    ));

                    ret.set_start_widget(Some(&reset_temporary_override_button));
                } else if self.reset_temp_override_btn_status
                    == TempOverrideResetButtonStatus::Loading
                {
                    let spinner = gtk::Spinner::builder().spinning(true).build();
                    ret.set_start_widget(Some(&spinner));
                } else {
                    ret.set_start_widget(None as Option<&gtk::Button>);
                }

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
