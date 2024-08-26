use std::time::Duration;

use adw::prelude::*;
use power_daemon::{KernelSettings, Profile};
use relm4::{
    binding::{Binding, BindingGuard, BoolBinding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    helpers::extra_bindings::AdjustmentBinding,
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum KernelInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for KernelInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug, Default)]
pub struct KernelGroup {
    settings_obtained: bool,

    disable_nmi_watchdog: BoolBinding,
    vm_writeback: AdjustmentBinding,
    laptop_mode: AdjustmentBinding,

    last_kernel_settings: Option<KernelSettings>,
    active_profile: Option<(usize, Profile)>,
}

impl KernelGroup {
    fn from_kernel_settings(&mut self, kernel_settings: &KernelSettings) {
        *self.disable_nmi_watchdog.guard() = kernel_settings.disable_nmi_watchdog.unwrap();

        let configure_adjustment = |adj: BindingGuard<AdjustmentBinding>, v: u32| {
            adj.set_lower(0.0);
            adj.set_upper(u32::MAX as f64);
            adj.set_value(v as f64);
            adj.set_step_increment(1.0);
        };

        configure_adjustment(
            self.vm_writeback.guard(),
            kernel_settings.vm_writeback.unwrap(),
        );
        configure_adjustment(
            self.laptop_mode.guard(),
            kernel_settings.laptop_mode.unwrap(),
        );
    }

    fn to_kernel_settings(&self) -> KernelSettings {
        KernelSettings {
            disable_nmi_watchdog: self.disable_nmi_watchdog.value().into(),
            vm_writeback: (self.vm_writeback.value().value() as u32).into(),
            laptop_mode: (self.laptop_mode.value().value() as u32).into(),
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for KernelGroup {
    type Input = KernelInput;

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
                    set_title: "Kernel settings",
                    adw::PreferencesGroup {
                        adw::SwitchRow {
                            set_title: "Disable NMI watchdog",
                            add_binding: (&model.disable_nmi_watchdog, "active"),
                            connect_active_notify => KernelInput::Changed,
                        },
                        adw::SpinRow {
                            set_title: "VM writeback in seconds",
                            add_binding: (&model.vm_writeback, "adjustment"),
                            connect_value_notify => KernelInput::Changed,
                        },
                        adw::SpinRow {
                            set_title: "Laptop mode",
                            add_binding: (&model.laptop_mode, "adjustment"),
                            connect_value_notify => KernelInput::Changed,
                        },
                    },
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = KernelGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            KernelInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();
                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            self.from_kernel_settings(&profile.kernel_settings);
                            self.settings_obtained = true;
                            self.last_kernel_settings = Some(self.to_kernel_settings());
                        }
                    }
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(10.0),
                    system_info::SystemInfoSyncType::None,
                ),
                RootRequest::Apply => {
                    if !(self.settings_obtained && self.active_profile.is_some()) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.kernel_settings = self.to_kernel_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::Kernel,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            KernelInput::Changed => {
                if let Some(ref last_settings) = self.last_kernel_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_kernel_settings(),
                            crate::SettingsGroup::Kernel,
                        ))
                        .unwrap()
                }
            }
        }
    }
}
