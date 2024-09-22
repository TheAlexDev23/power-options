use core::f64;
use std::time::Duration;

use adw::prelude::*;
use power_daemon::{AmdGpuInfo, GpuInfo, GpuSettings, Profile};
use relm4::{
    binding::{Binding, BindingGuard, U32Binding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    helpers::extra_bindings::AdjustmentBinding,
    AppInput, AppSyncUpdate, RootRequest,
};

lazy_static::lazy_static! {
    static ref AMD_PERF_LEVELS: Vec<&'static str> = vec![
        "low",
        "auto",
        "high",
    ];
    static ref AMD_STATES: Vec<&'static str> = vec![
        "battery",
        "balanced",
        "performance",
    ];
    static ref AMD_POWER_PROFILES: Vec<&'static str> = vec![
        "low",
        "mid",
        "high",
        "auto",
        "default",
    ];
}

#[derive(Debug, Clone)]
pub enum GpuInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for GpuInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug, Default)]
pub struct GpuGroup {
    settings_obtained: bool,

    intel_min: AdjustmentBinding,
    intel_max: AdjustmentBinding,
    intel_boost: AdjustmentBinding,

    amd_perf_level: U32Binding,
    amd_power_state: U32Binding,
    amd_power_profile: U32Binding,

    last_gpu_settings: Option<GpuSettings>,
    active_profile: Option<(usize, Profile)>,

    supports_intel_gpu: bool,
    supports_amd_gpu: bool,
    supports_amd_perf_level: bool,
    supports_amd_power_state: bool,
    supports_amd_power_profiles: bool,
}

impl GpuGroup {
    #[allow(clippy::wrong_self_convention)]
    fn from_gpu_settings(&mut self, gpu_settings: &GpuSettings) {
        fn configure_initial_frequency_adjustment(
            adjustment: BindingGuard<AdjustmentBinding>,
            value: u32,
        ) {
            adjustment.set_lower(0.0);
            adjustment.set_upper(f64::MAX);
            adjustment.set_step_increment(50.0);
            adjustment.set_value(value as f64);
        }

        fn configure_dropdown(
            mut index: BindingGuard<U32Binding>,
            value: &str,
            items: &[&'static str],
        ) {
            *index = items.iter().position(|v| *v == value).unwrap() as u32;
        }

        configure_initial_frequency_adjustment(
            self.intel_min.guard(),
            gpu_settings.intel_min.unwrap_or_default(),
        );
        configure_initial_frequency_adjustment(
            self.intel_max.guard(),
            gpu_settings.intel_max.unwrap_or_default(),
        );
        configure_initial_frequency_adjustment(
            self.intel_boost.guard(),
            gpu_settings.intel_boost.unwrap_or_default(),
        );

        configure_dropdown(
            self.amd_perf_level.guard(),
            &gpu_settings
                .amd_dpm_perf_level
                .clone()
                .unwrap_or("auto".to_string()),
            &AMD_PERF_LEVELS,
        );
        configure_dropdown(
            self.amd_power_state.guard(),
            &gpu_settings
                .amd_dpm_power_state
                .clone()
                .unwrap_or("balanced".to_string()),
            &AMD_STATES,
        );
        configure_dropdown(
            self.amd_power_profile.guard(),
            &gpu_settings
                .amd_power_profile
                .clone()
                .unwrap_or("auto".to_string()),
            &AMD_POWER_PROFILES,
        );
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_gpu_info(&mut self, gpu_info: &GpuInfo) {
        if let Some(ref info) = gpu_info.intel_info {
            let set_proper_ranges = move |b: BindingGuard<AdjustmentBinding>| {
                b.set_lower(info.min_frequency as f64);
                b.set_upper(info.max_frequency as f64);
            };

            set_proper_ranges(self.intel_min.guard());
            set_proper_ranges(self.intel_max.guard());
            set_proper_ranges(self.intel_boost.guard());

            self.supports_intel_gpu = true;
        } else {
            self.supports_intel_gpu = false;
        }

        if let Some(ref info) = gpu_info.amd_info {
            if matches!(info, AmdGpuInfo::Legacy { power_profile: _ }) {
                self.supports_amd_perf_level = false;
                self.supports_amd_power_state = false;
                self.supports_amd_power_profiles = true;
            }
            if matches!(info, AmdGpuInfo::AmdGpu { dpm_perf: _ }) {
                self.supports_amd_perf_level = true;
                self.supports_amd_power_state = false;
                self.supports_amd_power_profiles = false;
            }
            if matches!(
                info,
                AmdGpuInfo::Radeon {
                    dpm_perf: _,
                    dpm_state: _
                }
            ) {
                self.supports_amd_perf_level = true;
                self.supports_amd_power_state = true;
                self.supports_amd_power_profiles = false;
            }

            self.supports_amd_gpu = true;
        } else {
            self.supports_amd_gpu = false;
        }
    }

    fn to_gpu_settings(&self) -> GpuSettings {
        GpuSettings {
            intel_min: if self.supports_intel_gpu {
                (self.intel_min.value().value() as u32).into()
            } else {
                None
            },
            intel_max: if self.supports_intel_gpu {
                (self.intel_max.value().value() as u32).into()
            } else {
                None
            },
            intel_boost: if self.supports_intel_gpu {
                (self.intel_boost.value().value() as u32).into()
            } else {
                None
            },
            amd_dpm_perf_level: if self.supports_amd_gpu && self.supports_amd_perf_level {
                AMD_PERF_LEVELS[self.amd_perf_level.value() as usize]
                    .to_string()
                    .into()
            } else {
                None
            },
            amd_dpm_power_state: if self.supports_amd_gpu && self.supports_amd_power_state {
                AMD_STATES[self.amd_power_state.value() as usize]
                    .to_string()
                    .into()
            } else {
                None
            },
            amd_power_profile: if self.supports_amd_gpu && self.supports_amd_power_profiles {
                AMD_POWER_PROFILES[self.amd_power_profile.value() as usize]
                    .to_string()
                    .into()
            } else {
                None
            },
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for GpuGroup {
    type Input = GpuInput;

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
                    set_title: "GPU settings",
                    adw::PreferencesGroup {
                        adw::SpinRow {
                            set_title: labels::INTEL_GPU_MIN,
                            #[watch]
                            set_sensitive: model.supports_intel_gpu,
                            #[watch]
                            set_tooltip_text: if !model.supports_intel_gpu {
                                Some(labels::INTEL_GPU_MISSING_TT)
                            } else {
                                None
                            },
                            add_binding: (&model.intel_min, "adjustment"),
                            connect_value_notify => GpuInput::Changed,
                        },
                        adw::SpinRow {
                            set_title: labels::INTEL_GPU_MAX,
                            #[watch]
                            set_sensitive: model.supports_intel_gpu,
                            #[watch]
                            set_tooltip_text: if !model.supports_intel_gpu {
                                Some(labels::INTEL_GPU_MISSING_TT)
                            } else {
                                None
                            },
                            add_binding: (&model.intel_max, "adjustment"),
                            connect_value_notify => GpuInput::Changed,
                        },
                        adw::SpinRow {
                            set_title: labels::INTEL_GPU_BOOST,
                            #[watch]
                            set_sensitive: model.supports_intel_gpu,
                            #[watch]
                            set_tooltip_text: if !model.supports_intel_gpu {
                                Some(labels::INTEL_GPU_MISSING_TT)
                            } else {
                                None
                            },
                            add_binding: (&model.intel_boost, "adjustment"),
                            connect_value_notify => GpuInput::Changed,
                        },
                    },
                    adw::PreferencesGroup {
                        adw::ComboRow {
                            set_title: labels::AMD_GPU_PERF_LEVEL,
                            #[watch]
                            set_sensitive: model.supports_amd_gpu && model.supports_amd_perf_level,
                            #[watch]
                            set_tooltip_text: if !model.supports_amd_gpu {
                                Some(labels::AMD_GPU_MISSING_TT)
                            } else if !model.supports_amd_perf_level {
                                Some(labels::AMD_GPU_PERF_LEVEL_UNAVAILABLE)
                            } else {
                                Some(labels::AMD_GPU_PERF_LEVEL_TT)
                            },
                            set_model: Some(&gtk::StringList::new(&AMD_PERF_LEVELS)),
                            add_binding: (&model.amd_perf_level, "selected"),
                            connect_selected_item_notify => GpuInput::Changed,
                        },
                        adw::ComboRow {
                            set_title: labels::AMD_GPU_STATE,
                            #[watch]
                            set_sensitive: model.supports_amd_gpu && model.supports_amd_power_state,
                            #[watch]
                            set_tooltip_text: if !model.supports_amd_gpu {
                                Some(labels::AMD_GPU_MISSING_TT)
                            } else if !model.supports_amd_power_state {
                                Some(labels::AMD_GPU_STATE_UNAVAILABLE)
                            } else {
                                Some(labels::AMD_GPU_STATE_TT)
                            },
                            set_model: Some(&gtk::StringList::new(&AMD_STATES)),
                            add_binding: (&model.amd_power_state, "selected"),
                            connect_selected_item_notify => GpuInput::Changed,
                        },
                        adw::ComboRow {
                            set_title: labels::AMD_GPU_POWER_PROFILE,
                            #[watch]
                            set_sensitive: model.supports_amd_gpu && model.supports_amd_power_profiles,
                            #[watch]
                            set_tooltip_text: if !model.supports_amd_gpu {
                                Some(labels::AMD_GPU_MISSING_TT)
                            } else if !model.supports_amd_power_state {
                                Some(labels::AMD_GPU_POWER_PROFILE_UNAVAILABLE)
                            } else {
                                Some(labels::AMD_GPU_POWER_PROFILE_TT)
                            },
                            set_model: Some(&gtk::StringList::new(&AMD_POWER_PROFILES)),
                            add_binding: (&model.amd_power_profile, "selected"),
                            connect_selected_item_notify => GpuInput::Changed,
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
        let model = GpuGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            GpuInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();
                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            self.from_gpu_settings(&profile.gpu_settings);
                            self.settings_obtained = true;
                            self.last_gpu_settings = Some(self.to_gpu_settings());
                        }
                    }
                    if let AppSyncUpdate::SystemInfo(ref system_info) = message {
                        if let Some(system_info) = system_info.as_ref() {
                            self.from_gpu_info(&system_info.gpu_info);
                        }
                    }
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(5.0),
                    system_info::SystemInfoSyncType::Gpu,
                ),
                RootRequest::Apply => {
                    if !(self.settings_obtained && self.active_profile.is_some()) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.gpu_settings = self.to_gpu_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::Gpu,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            GpuInput::Changed => {
                if let Some(ref last_settings) = self.last_gpu_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_gpu_settings(),
                            crate::SettingsGroup::Gpu,
                        ))
                        .unwrap()
                }
            }
        }
    }
}
