use core::f64;
use std::time::Duration;

use adw::prelude::*;
use power_daemon::{Profile, ProfilesInfo, SystemInfo, USBSettings};
use relm4::{
    binding::{Binding, BoolBinding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    helpers::extra_bindings::AdjustmentBinding,
    whiteblacklist::{
        WhiteBlackListRenderer, WhiteBlackListRendererInit, WhiteBlackListRendererInput,
    },
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum USBInput {
    RootRequest(RootRequest),
    WhiteBlackListChanged,
    Changed,
}

impl From<RootRequest> for USBInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug)]
pub struct USBGroup {
    initialized: bool,

    system_info: Option<SystemInfo>,
    profiles_info: Option<ProfilesInfo>,

    enable_usb_pm: BoolBinding,
    usb_autosuspend_delay_ms: AdjustmentBinding,

    usb_pm_whiteblacklist_renderer: Controller<WhiteBlackListRenderer>,

    awaiting_whiteblacklist_renderer_init: bool,

    last_usb_settings: Option<USBSettings>,

    active_profile: Option<(usize, Profile)>,
}

impl USBGroup {
    fn initialize_from_profile_and_system_info(&mut self) {
        assert!(self.profiles_info.is_some() && self.system_info.is_some());

        let profile = self.profiles_info.as_ref().unwrap().get_active_profile();
        let info = self.system_info.as_ref().unwrap();

        *self.enable_usb_pm.guard() = profile.usb_settings.enable_pm.unwrap();

        let adjustment = self.usb_autosuspend_delay_ms.guard();
        adjustment.set_lower(0.0);
        adjustment.set_upper(u32::MAX as f64);
        adjustment.set_step_increment(100.0);
        adjustment.set_value(profile.usb_settings.autosuspend_delay_ms.unwrap() as f64);

        self.awaiting_whiteblacklist_renderer_init = true;
        self.usb_pm_whiteblacklist_renderer
            .sender()
            .send(WhiteBlackListRendererInput::Init(
                WhiteBlackListRendererInit {
                    list: profile.usb_settings.whiteblacklist.clone().unwrap(),
                    rows: info
                        .usb_info
                        .usb_devices
                        .clone()
                        .into_iter()
                        .map(|d| [d.id, d.display_name.to_string()])
                        .collect(),
                },
            ))
            .unwrap();

        self.initialized = true;
    }

    fn to_usb_settings(&self) -> USBSettings {
        USBSettings {
            enable_pm: self.enable_usb_pm.value().into(),
            autosuspend_delay_ms: (self.usb_autosuspend_delay_ms.value().value() as u32).into(),
            whiteblacklist: self
                .usb_pm_whiteblacklist_renderer
                .model()
                .to_whiteblacklist()
                .into(),
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for USBGroup {
    type Input = USBInput;

    type Output = AppInput;

    type Init = ();

    view! {
        gtk::Box {
            set_homogeneous: true,
            set_expand: true,
            if !model.initialized {
                gtk::Box {
                    set_align: gtk::Align::Center,
                    gtk::Label::new(Some("Connecting to the daemon...")),
                    gtk::Spinner {
                        set_spinning: true,
                        set_visible: true,
                    }
                }
            } else {
                gtk::Box {
                    adw::PreferencesPage {
                        set_expand: true,
                        adw::PreferencesGroup {
                            set_title: "USB Power Management",
                            adw::SwitchRow {
                                set_title: "Enable USB Power Management",
                                add_binding: (&model.enable_usb_pm, "active"),
                                connect_active_notify => USBInput::Changed,
                            },
                            adw::SpinRow {
                                set_title: "USB Autosupsend delay in miliseconds",
                                add_binding: (&model.usb_autosuspend_delay_ms, "adjustment"),
                                connect_value_notify => USBInput::Changed,
                            },
                        },
                        model.usb_pm_whiteblacklist_renderer.widget(),
                    },
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let usb_pm_whiteblacklist_renderer = WhiteBlackListRenderer::builder()
            .launch(())
            .forward(sender.input_sender(), |_| USBInput::WhiteBlackListChanged);

        let model = USBGroup {
            initialized: Default::default(),
            profiles_info: Default::default(),
            system_info: Default::default(),
            enable_usb_pm: Default::default(),
            usb_autosuspend_delay_ms: Default::default(),
            usb_pm_whiteblacklist_renderer,
            awaiting_whiteblacklist_renderer_init: Default::default(),
            last_usb_settings: Default::default(),
            active_profile: Default::default(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            USBInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(info) = profiles_info.as_ref() {
                            self.active_profile =
                                Some((info.active_profile, info.get_active_profile().clone()));
                        }
                        self.profiles_info = (**profiles_info).clone();

                        self.initialized = false;
                    }

                    if let AppSyncUpdate::SystemInfo(ref info) = message {
                        self.system_info = (**info).clone();
                    }

                    if !self.initialized
                        && self.profiles_info.is_some()
                        && self.system_info.is_some()
                    {
                        self.initialize_from_profile_and_system_info();
                        self.last_usb_settings = Some(self.to_usb_settings());
                    }
                }
                RootRequest::ConfigureSystemInfoSync => {
                    system_info::set_system_info_sync(
                        Duration::from_secs_f32(10.0),
                        system_info::SystemInfoSyncType::USB,
                    );
                    system_info::set_system_info_sync(
                        Duration::from_secs_f32(10.0),
                        system_info::SystemInfoSyncType::None,
                    );
                }
                RootRequest::Apply => {
                    if !self.initialized {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.usb_settings = self.to_usb_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::USB,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            USBInput::WhiteBlackListChanged => {
                if self.awaiting_whiteblacklist_renderer_init {
                    self.last_usb_settings = Some(self.to_usb_settings());
                    self.awaiting_whiteblacklist_renderer_init = false;
                }
                sender.input(USBInput::Changed);
            }
            USBInput::Changed => {
                if let Some(ref last_usb_settings) = self.last_usb_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_usb_settings != self.to_usb_settings(),
                            crate::SettingsGroup::USB,
                        ))
                        .unwrap()
                }
            }
        }
    }
}
