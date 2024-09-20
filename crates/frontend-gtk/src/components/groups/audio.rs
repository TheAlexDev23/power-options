use std::time::Duration;

use adw::prelude::*;
use power_daemon::{AudioModule, AudioSettings, Profile};
use relm4::{binding::Binding, prelude::*, RelmObjectExt};

use crate::{
    communications::{daemon_control, system_info},
    helpers::extra_bindings::AdjustmentBinding,
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum AudioInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for AudioInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug, Default)]
pub struct AudioGroup {
    settings_obtained: bool,

    idle_timeout: AdjustmentBinding,

    last_audio_settings: Option<AudioSettings>,
    active_profile: Option<(usize, Profile)>,

    supports_audio_drivers: bool,
}

impl AudioGroup {
    #[allow(clippy::wrong_self_convention)]
    fn from_audio_settings(&mut self, audio_settings: &AudioSettings) {
        let idle_timeout = self.idle_timeout.guard();

        idle_timeout.set_upper(3600.0);
        idle_timeout.set_lower(0.0);
        idle_timeout.set_step_increment(1.0);
        idle_timeout.set_value(audio_settings.idle_timeout.unwrap_or_default() as f64);
    }

    fn to_audio_settings(&self) -> AudioSettings {
        AudioSettings {
            idle_timeout: if self.supports_audio_drivers {
                Some(self.idle_timeout.value().value() as u32)
            } else {
                None
            },
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for AudioGroup {
    type Input = AudioInput;

    type Output = AppInput;

    type Init = ();

    view! {
        gtk::Box {
            set_homogeneous: true,
            set_expand: true,
            if !model.settings_obtained {
                gtk::Box {
                    set_align: gtk::Align::Center,
                    gtk::Label::new(Some("Connecting to the daemon...")),
                    gtk::Spinner {
                        set_spinning: true,
                        set_visible: true,
                    }
                }
            } else {
                adw::PreferencesPage {
                    set_expand: true,
                    set_title: "Audio settings",
                    adw::PreferencesGroup {
                        adw::SpinRow {
                            set_title: labels::AUDIO_IDLE_TIMEOUT_TITLE,
                            #[watch]
                            set_sensitive: model.supports_audio_drivers,
                            #[watch]
                            set_tooltip_text: if !model.supports_audio_drivers {
                                Some(labels::AUDIO_IDLE_TIMEOUT_MODULE_UNSPORTED_TT)
                            } else {
                                Some(labels::AUDIO_IDLE_TIMEOUT_TT)
                            },
                            add_binding: (&model.idle_timeout, "adjustment"),
                            connect_value_notify => AudioInput::Changed,
                        },
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AudioGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            AudioInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();
                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            self.from_audio_settings(&profile.audio_settings);
                            self.settings_obtained = true;
                            self.last_audio_settings = Some(self.to_audio_settings());
                        }
                    }
                    if let AppSyncUpdate::SystemInfo(ref system_info) = message {
                        if let Some(system_info) = system_info.as_ref() {
                            self.supports_audio_drivers =
                                system_info.opt_features_info.audio_module != AudioModule::Other;
                        }
                    }
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(15.0),
                    system_info::SystemInfoSyncType::Opt,
                ),
                RootRequest::Apply => {
                    if !(self.settings_obtained && self.active_profile.is_some()) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.audio_settings = self.to_audio_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::Audio,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            AudioInput::Changed => {
                if let Some(ref last_settings) = self.last_audio_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_audio_settings(),
                            crate::SettingsGroup::Audio,
                        ))
                        .unwrap()
                }
            }
        }
    }
}
