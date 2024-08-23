use std::time::Duration;

use power_daemon::{CPUInfo, CPUSettings, Profile};

use adw::prelude::*;
use relm4::{
    binding::{Binding, BindingGuard, BoolBinding, U32Binding},
    prelude::*,
    RelmObjectExt,
};

use super::{CPU_EPPS, CPU_GOVERNORS_ACTIVE, CPU_GOVERNORS_PASSIVE};
use crate::{
    communications::{daemon_control, system_info},
    helpers::extra_bindings::{AdjustmentBinding, StringListBinding},
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum CPUInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for CPUInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[tracker::track]
#[derive(Debug, Default)]
pub struct CPUGroup {
    info_obtained: bool,

    can_change_modes: bool,
    can_change_epps: bool,

    has_perf_pct: bool,
    has_boost: bool,
    has_hwp_dyn_boost: bool,

    #[do_not_track]
    min_freq: AdjustmentBinding,
    #[do_not_track]
    max_freq: AdjustmentBinding,

    #[do_not_track]
    min_perf: AdjustmentBinding,
    #[do_not_track]
    max_perf: AdjustmentBinding,

    #[do_not_track]
    available_epps: StringListBinding,
    #[do_not_track]
    available_governors: StringListBinding,

    #[do_not_track]
    mode: U32Binding,
    #[do_not_track]
    epp: U32Binding,
    #[do_not_track]
    governor: U32Binding,

    #[do_not_track]
    boost: BoolBinding,
    #[do_not_track]
    hwp_dyn_boost: BoolBinding,

    #[do_not_track]
    active_profile: Option<(usize, Profile)>,

    #[do_not_track]
    last_cpu_settings: Option<CPUSettings>,
}

impl CPUGroup {
    fn governor_is_performance(&self) -> bool {
        self.mode.value() == 0 && self.governor.value() == 0
    }

    fn from_cpu_settings(&mut self, cpu_settings: &CPUSettings) {
        let epps = CPU_EPPS.clone();

        let governors = if cpu_settings.mode.as_ref().unwrap_or(&"passive".to_string()) == "active"
        {
            CPU_GOVERNORS_ACTIVE.clone()
        } else {
            CPU_GOVERNORS_PASSIVE.clone()
        };

        *self.available_epps.guard() = gtk::StringList::new(&epps);
        *self.epp.guard() = epps
            .iter()
            .position(|v| *v == cpu_settings.epp.as_ref().unwrap_or(&"default".to_string()))
            .unwrap() as u32;

        *self.available_governors.guard() = gtk::StringList::new(&governors);
        *self.governor.guard() = governors
            .iter()
            .position(|v| {
                *v == cpu_settings
                    .governor
                    .as_ref()
                    .unwrap_or(&"powersave".to_string())
            })
            .unwrap() as u32;

        // Without info the allowed range would be 0,0 preventing us from
        // setting a value
        if !self.info_obtained {
            self.min_freq.guard().set_lower(f64::MIN);
            self.min_freq.guard().set_upper(f64::MAX);

            self.max_freq.guard().set_lower(f64::MIN);
            self.max_freq.guard().set_upper(f64::MAX);
        }

        self.min_freq
            .guard()
            .set_value(cpu_settings.min_freq.unwrap() as f64);
        self.max_freq
            .guard()
            .set_value(cpu_settings.max_freq.unwrap() as f64);

        let set_perf_ranges = |guard: BindingGuard<AdjustmentBinding>| {
            guard.set_step_increment(5.0);
            guard.set_lower(0.0);
            guard.set_upper(100.0);
        };
        set_perf_ranges(self.min_perf.guard());
        set_perf_ranges(self.max_perf.guard());

        self.min_perf
            .guard()
            .set_value(cpu_settings.min_perf_pct.unwrap() as f64);
        self.max_perf
            .guard()
            .set_value(cpu_settings.max_perf_pct.unwrap() as f64);

        *self.boost.guard() = cpu_settings.boost.unwrap_or_default();
        *self.hwp_dyn_boost.guard() = cpu_settings.hwp_dyn_boost.unwrap_or_default();
    }

    fn from_cpu_info(&mut self, cpu_info: &CPUInfo) {
        self.set_can_change_modes(cpu_info.mode.is_some());
        self.set_can_change_epps(cpu_info.has_epp);
        self.set_has_perf_pct(cpu_info.has_perf_pct_scaling);

        let set_freq_ranges = |guard: BindingGuard<AdjustmentBinding>| {
            guard.set_step_increment(100.0);
            guard.set_lower(cpu_info.total_min_frequency as f64);
            guard.set_upper(cpu_info.total_max_frequency as f64);
        };
        set_freq_ranges(self.min_freq.guard());
        set_freq_ranges(self.max_freq.guard());

        self.set_has_boost(cpu_info.boost.is_some());
        self.set_has_hwp_dyn_boost(cpu_info.hwp_dynamic_boost.is_some());
    }

    fn to_cpu_settings(&self) -> CPUSettings {
        let active = self.mode.value() == 0;
        let gov = self.governor.value() as usize;
        let epp = self.epp.value() as usize;

        CPUSettings {
            mode: if self.can_change_modes {
                Some(if active { "active" } else { "passive" }.to_string())
            } else {
                None
            },
            governor: Some(
                if active {
                    CPU_GOVERNORS_ACTIVE[gov]
                } else {
                    CPU_GOVERNORS_PASSIVE[gov]
                }
                .to_string(),
            ),
            epp: if *self.get_can_change_epps() {
                Some(CPU_EPPS[epp].to_string())
            } else {
                None
            },
            min_freq: Some(self.min_freq.value().value() as u32),
            max_freq: Some(self.max_freq.value().value() as u32),
            min_perf_pct: if *self.get_has_perf_pct() {
                Some(self.min_perf.value().value() as u8)
            } else {
                None
            },
            max_perf_pct: if *self.get_has_perf_pct() {
                Some(self.max_perf.value().value() as u8)
            } else {
                None
            },
            boost: if *self.get_has_boost() {
                Some(self.boost.value())
            } else {
                None
            },
            hwp_dyn_boost: if *self.get_has_hwp_dyn_boost() {
                Some(self.hwp_dyn_boost.value())
            } else {
                None
            },
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for CPUGroup {
    type Input = CPUInput;

    type Output = AppInput;

    type Init = ();

    view! {
        gtk::Box {
            set_homogeneous: true,
            set_expand: true,
            if !model.info_obtained || model.active_profile.is_none() {
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
                    set_title: "CPU settings",
                    adw::PreferencesGroup {
                        adw::ComboRow {
                            set_title: "Scaling driver operation mode",
                            #[track(model.changed(CPUGroup::can_change_modes()))]
                            set_sensitive: model.can_change_modes,
                            #[track(model.changed(CPUGroup::can_change_modes()))]
                            set_tooltip_text: if !model.can_change_modes {
                                Some("Your system does not support mode switching")
                            } else {
                                None
                            },
                            set_model: Some(&gtk::StringList::new(&["active", "passive"])),
                            add_binding: (&model.mode, "selected"),
                            connect_selected_item_notify => CPUInput::Changed,
                        },

                        adw::ComboRow {
                            set_title: "Energy Performance Preference",
                            // Watch instead of track can_change_epps because no way to track governor changes
                            #[watch]
                            set_sensitive: model.can_change_epps && !model.governor_is_performance(),
                            #[watch]
                            set_tooltip_text: if !model.can_change_epps {
                                Some("EPP options are unavailable in your system")
                            } else if model.governor_is_performance() {
                                Some("EPP is locked to the highest setting when the governor is set to performance")
                            } else {
                                None
                            },
                            add_binding: (&model.available_epps, "model"),
                            add_binding: (&model.epp, "selected"),
                            connect_selected_item_notify => CPUInput::Changed,
                        },

                        adw::ComboRow {
                            set_title: "Scaling governor",
                            add_binding: (&model.available_governors, "model"),
                            add_binding: (&model.governor, "selected"),
                            connect_selected_item_notify => CPUInput::Changed,
                        }
                    },

                    adw::PreferencesGroup {
                        adw::SpinRow {
                            set_title: "Minimum frequency (MHz)",
                            add_binding: (&model.min_freq, "adjustment"),
                            connect_value_notify => CPUInput::Changed,
                        },

                        adw::SpinRow {
                            set_title: "Maximum frequency (MHz)",
                            add_binding: (&model.max_freq, "adjustment"),
                            connect_value_notify => CPUInput::Changed,
                        }
                    },

                    adw::PreferencesGroup {
                        adw::SpinRow {
                            set_title: "Minimum performance percentage",
                            #[track(model.changed(CPUGroup::has_perf_pct()))]
                            set_sensitive: model.has_perf_pct,
                            #[track(model.changed(CPUGroup::has_perf_pct()))]
                            set_tooltip_text: if !model.has_perf_pct {
                                Some("Performance percentage scaling is unavailable in your system")
                            } else {
                                None
                            },
                            add_binding: (&model.min_perf, "adjustment"),
                            connect_value_notify => CPUInput::Changed,
                        },

                        adw::SpinRow {
                            set_title: "Maximum performance percentage",
                            #[track(model.changed(CPUGroup::has_perf_pct()))]
                            set_sensitive: model.has_perf_pct,
                            #[track(model.changed(CPUGroup::has_perf_pct()))]
                            set_tooltip_text: if !model.has_perf_pct {
                                Some("Performance percentage scaling is unavailable in your system")
                            } else {
                                None
                            },
                            add_binding: (&model.max_perf, "adjustment"),
                            connect_value_notify => CPUInput::Changed,
                        }
                    },

                    adw::PreferencesGroup {
                        adw::SwitchRow {
                            set_title: "Boost technology",
                            #[track(model.changed(CPUGroup::has_boost()))]
                            set_sensitive: model.has_boost,
                            #[track(model.changed(CPUGroup::has_boost()))]
                            set_tooltip_text: if !model.has_boost {
                                Some("CPU boosting techonologies are unavailable in your system")
                            } else {
                                None
                            },
                            add_binding: (&model.boost, "active"),
                            connect_active_notify  => CPUInput::Changed,
                        },

                        adw::SwitchRow {
                            set_title: "HWP dynamic boost",
                            #[track(model.changed(CPUGroup::has_hwp_dyn_boost()))]
                            set_sensitive: model.has_hwp_dyn_boost,
                            #[track(model.changed(CPUGroup::has_hwp_dyn_boost()))]
                            set_tooltip_text: if !model.has_hwp_dyn_boost {
                                Some("HWP Dynamic boost is unsupported in your system")
                            } else {
                                None
                            },
                            add_binding: (&model.hwp_dyn_boost, "active"),
                            connect_active_notify  => CPUInput::Changed,
                        }
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
        let model = CPUGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            CPUInput::RootRequest(request) => match request {
                RootRequest::Apply => {
                    if !(self.info_obtained && self.active_profile.is_some()) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.cpu_settings = self.to_cpu_settings();

                    tokio::spawn(async move {
                        daemon_control::set_reduced_update(power_daemon::ReducedUpdate::CPU).await;

                        daemon_control::update_profile(active_profile.0 as u32, active_profile.1)
                            .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(5.0),
                    system_info::SystemInfoSyncType::CPU,
                ),
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();

                            self.info_obtained = false;
                            fetch_info_once();

                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            self.from_cpu_settings(&profile.cpu_settings);

                            self.last_cpu_settings = Some(self.to_cpu_settings());
                        }
                    }

                    if let AppSyncUpdate::SystemInfo(ref system_info) = message {
                        if let Some(system_info) = system_info.as_ref() {
                            self.info_obtained = true;
                            self.from_cpu_info(&system_info.cpu_info);
                            // The reason we can't get away from updating just
                            // once during profile obtention is because we
                            // access system info values for conditional profile
                            // updating based on the system's features. And
                            // proper reactivity requires us to synchronize the
                            // newest settings
                            self.last_cpu_settings = Some(self.to_cpu_settings());
                        }
                    }
                }
            },
            CPUInput::Changed => {
                if let Some(ref last_settings) = self.last_cpu_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_cpu_settings(),
                            crate::SettingsGroup::CPU,
                        ))
                        .unwrap()
                }
            }
        }
    }
}

fn fetch_info_once() {
    system_info::set_system_info_sync(
        Duration::from_secs_f32(15.0),
        system_info::SystemInfoSyncType::CPU,
    );
    system_info::set_system_info_sync(
        Duration::from_secs_f32(15.0),
        system_info::SystemInfoSyncType::None,
    );
}
