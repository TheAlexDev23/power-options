use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{
    IntelRaplInterfaceInfo, IntelRaplInterfaceSettings, IntelRaplSettings, ProfilesInfo,
    ReducedUpdate, SystemInfo,
};

use crate::communication_services::{
    control_routine_send_multiple, ControlAction, ControlRoutine, SystemInfoRoutine,
    SystemInfoSyncType,
};
use crate::helpers::toggleable_components::ToggleableNumericField;
use crate::helpers::toggleable_types::ToggleableInt;

#[derive(Default, PartialEq, Clone, Debug)]
struct IntelRaplForm {
    package: Signal<RaplInterface>,
    core: Signal<RaplInterface>,
    uncore: Signal<RaplInterface>,
}

impl IntelRaplForm {
    pub fn new(rapl_settings: &IntelRaplSettings) -> IntelRaplForm {
        let mut form = IntelRaplForm::default();
        form.set_values(rapl_settings);
        form
    }

    pub fn set_values(&mut self, rapl_settings: &IntelRaplSettings) {
        (self.package)().set_values(rapl_settings.package.as_ref());
        (self.core)().set_values(rapl_settings.core.as_ref());
        (self.uncore)().set_values(rapl_settings.uncore.as_ref());
    }
}

#[derive(Default, PartialEq, Clone, Debug)]
struct RaplInterface {
    pub long_term: ToggleableInt,
    pub short_term: ToggleableInt,
    pub peak_power: ToggleableInt,
}

impl RaplInterface {
    pub fn set_values(&mut self, interface: Option<&IntelRaplInterfaceSettings>) {
        if let Some(interface) = interface {
            self.long_term.from_u32(interface.long_term_limit);
            self.short_term.from_u32(interface.short_term_limit);
            self.peak_power.from_u32(interface.peak_power_limit);
        }
    }

    pub fn to_interface_settings(&self) -> Option<IntelRaplInterfaceSettings> {
        Some(IntelRaplInterfaceSettings {
            long_term_limit: self.long_term.into_u32(),
            short_term_limit: self.short_term.into_u32(),
            peak_power_limit: self.peak_power.into_u32(),
        })
    }
}

#[component]
pub fn RaplGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::None));

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to the daemon..." };
    }

    let rapl_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .rapl_settings
        .clone();

    let rapl_info = system_info().as_ref().unwrap().rapl_info.clone();

    let mut form_used_settings = use_signal(|| rapl_settings.clone());
    let mut form = use_hook(|| IntelRaplForm::new(&rapl_settings));
    if form_used_settings() != rapl_settings {
        form.set_values(&rapl_settings);
        form_used_settings.set(rapl_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let active_profile_idx = profiles_info().as_ref().unwrap().active_profile;
        let mut active_profile = profiles_info()
            .as_ref()
            .unwrap()
            .get_active_profile()
            .clone();

        active_profile.rapl_settings = IntelRaplSettings {
            package: (form.package)().to_interface_settings(),
            core: (form.core)().to_interface_settings(),
            uncore: (form.uncore)().to_interface_settings(),
        };

        control_routine_send_multiple(
            control_routine,
            &[
                ControlAction::UpdateProfileReduced(
                    active_profile_idx as u32,
                    active_profile.into(),
                    ReducedUpdate::Rapl,
                ),
                ControlAction::GetProfilesInfo,
            ],
            Some(awaiting_completion),
        );
    };

    rsx! {
        form {
            onchange: move |_| {
                changed.set(true);
            },
            onsubmit: move |_| {
                onsubmit();
                changed.set(false);
            },

            RaplInterfaceRenderer { name: "Package", info: rapl_info.package, constraint: form.package }
            RaplInterfaceRenderer { name: "Core", info: rapl_info.core, constraint: form.core }
            RaplInterfaceRenderer { name: "Uncore", info: rapl_info.uncore, constraint: form.uncore }

            div { class: "confirm-buttons",
                button {
                    r#type: "submit",
                    disabled: !changed() || awaiting_completion(),
                    if awaiting_completion() {
                        div { class: "spinner" }
                    }
                    label { "Apply" }
                }
                input {
                    onclick: move |_| {
                        form.set_values(&rapl_settings);
                        changed.set(false);
                    },
                    r#type: "button",
                    value: "Cancel"
                }
            }

            br {}
            br {}
            br {}
        }
    }
}

#[component]
fn RaplInterfaceRenderer(
    name: String,
    info: Option<IntelRaplInterfaceInfo>,
    constraint: Signal<RaplInterface>,
) -> Element {
    rsx! {
        div {
            h3 { "{name}" }
            if info.is_none() {
                p { "{labels::RAPL_INTERFACE_UNSUPPORTED}" }
            } else {
                div { class: "option-group",
                    div { class: "option",
                        ToggleableNumericField {
                            name: labels::RAPL_LONG_TERM_TITLE,
                            disabled: info.as_ref().unwrap().long_term.is_none(),
                            tooltip: if info.as_ref().unwrap().long_term.is_none() {
                                Some(labels::RAPL_CONSTRAINT_UNSUPPORTED.to_string())
                            } else {
                                Some(labels::RAPL_LONG_TERM_TT.to_string())
                            },
                            value: constraint().long_term
                        }
                    }
                    div { class: "option",
                        ToggleableNumericField {
                            name: labels::RAPL_SHORT_TERM_TITLE,
                            disabled: info.as_ref().unwrap().short_term.is_none(),
                            tooltip: if info.as_ref().unwrap().short_term.is_none() {
                                Some(labels::RAPL_CONSTRAINT_UNSUPPORTED.to_string())
                            } else {
                                Some(labels::RAPL_SHORT_TERM_TT.to_string())
                            },
                            value: constraint().short_term
                        }
                    }
                    div { class: "option",
                        ToggleableNumericField {
                            name: labels::RAPL_PEAK_POWER_TITLE,
                            disabled: info.as_ref().unwrap().peak_power.is_none(),
                            tooltip: if info.as_ref().unwrap().peak_power.is_none() {
                                Some(labels::RAPL_CONSTRAINT_UNSUPPORTED.to_string())
                            } else {
                                Some(labels::RAPL_PEAK_POWER_TT.to_string())
                            },
                            value: constraint().peak_power
                        }
                    }
                }
            }
        }
    }
}
