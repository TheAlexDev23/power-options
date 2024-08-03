use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{
    ASPMSettings, PCISettings, ProfilesInfo, ReducedUpdate, SystemInfo, WhiteBlackList,
};

use super::{ToggleableBool, ToggleableString};

use crate::communication_services::{
    ControlAction, ControlRoutine, SystemInfoRoutine, SystemInfoSyncType,
};
use crate::helpers::{ToggleableDropdown, ToggleableStringWhiteBlackList, ToggleableToggle};

#[derive(PartialEq, Clone, Default)]
struct PCIAndASPMForm {
    pub enable_pci_pm: ToggleableBool,
    pub pci_pm_whiteblacklist: (Signal<bool>, Signal<WhiteBlackList<String>>),
    pub aspm: ToggleableString,
}

impl PCIAndASPMForm {
    pub fn new(pci_settings: &PCISettings, aspm_settings: &ASPMSettings) -> PCIAndASPMForm {
        let mut ret = PCIAndASPMForm::default();
        ret.set_values(pci_settings, aspm_settings);
        ret
    }

    pub fn set_values(&mut self, pci_settings: &PCISettings, aspm_settings: &ASPMSettings) {
        self.enable_pci_pm
            .0
            .set(pci_settings.enable_power_management.is_some());
        self.enable_pci_pm
            .1
            .set(pci_settings.enable_power_management.unwrap_or_default());

        self.pci_pm_whiteblacklist
            .0
            .set(pci_settings.whiteblacklist.is_some());
        self.pci_pm_whiteblacklist
            .1
            .set(pci_settings.whiteblacklist.clone().unwrap_or_default());

        self.aspm.0.set(aspm_settings.mode.is_some());
        self.aspm
            .1
            .set(aspm_settings.mode.clone().unwrap_or_default());
    }
}

#[component]
pub fn PCIAndASPMGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
    system_info_routine: SystemInfoRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(2.0), SystemInfoSyncType::PCI));

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to daemon.." };
    }

    let pci_info = system_info().as_ref().unwrap().pci_info.clone();

    let pci_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .pci_settings
        .clone();

    let aspm_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .aspm_settings
        .clone();

    let mut form_used_settings = use_signal(|| (pci_settings.clone(), aspm_settings.clone()));
    let mut form = use_hook(|| PCIAndASPMForm::new(&pci_settings, &aspm_settings));
    if form_used_settings() != (pci_settings.clone(), aspm_settings.clone()) {
        form.set_values(&pci_settings, &aspm_settings);
        form_used_settings.set((pci_settings.clone(), aspm_settings.clone()));
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let profiles_info = profiles_info().as_ref().unwrap().clone();

        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.pci_settings = PCISettings {
            enable_power_management: if form.enable_pci_pm.0() {
                Some(form.enable_pci_pm.1())
            } else {
                None
            },
            whiteblacklist: if form.pci_pm_whiteblacklist.0() {
                Some(form.pci_pm_whiteblacklist.1())
            } else {
                None
            },
        };

        active_profile.aspm_settings = ASPMSettings {
            mode: if form.aspm.0() {
                Some(form.aspm.1())
            } else {
                None
            },
        };

        control_routine.send((
            ControlAction::SetReducedUpdate(ReducedUpdate::PCI),
            Some(awaiting_completion),
        ));
        control_routine.send((
            ControlAction::UpdateProfile(active_profile_idx as u32, active_profile),
            Some(awaiting_completion),
        ));
        control_routine.send((
            ControlAction::SetReducedUpdate(ReducedUpdate::ASPM),
            Some(awaiting_completion),
        ));
        control_routine.send((ControlAction::Update, Some(awaiting_completion)));
        control_routine.send((ControlAction::GetProfilesInfo, Some(awaiting_completion)));
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

            if pci_info.aspm_info.supported_modes.is_some() {
                h2 { "PCIe Active State Power Management" }

                div { class: "option-group",
                    div { class: "option",
                        ToggleableDropdown {
                            name: "ASPM operation mode",
                            items: pci_info.aspm_info.supported_modes.unwrap(),
                            value: form.aspm
                        }
                    }
                }
            }

            h2 { "PCI Power Management" }

            div { class: "option-group",
                div { class: "option",
                    ToggleableToggle { name: "Enable runtime power management", value: form.enable_pci_pm }
                }
            }

            if form.enable_pci_pm.1() {
                ToggleableStringWhiteBlackList {
                    value: form.pci_pm_whiteblacklist,
                    columns: ["Address".to_string(), "Device Name".to_string()],
                    rows: pci_info
                        .pci_devices
                        .iter()
                        .map(|d| [d.pci_address.clone(), d.display_name.clone()])
                        .collect::<Vec<_>>(),
                    value_index: 0
                }
            }

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
                        form.set_values(&pci_settings, &aspm_settings);
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
