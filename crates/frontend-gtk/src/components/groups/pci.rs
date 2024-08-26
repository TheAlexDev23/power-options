use std::time::Duration;

use adw::prelude::*;
use power_daemon::{ASPMSettings, PCISettings, Profile, ProfilesInfo, SystemInfo};
use relm4::{
    binding::{Binding, BoolBinding, U32Binding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    helpers::extra_bindings::StringListBinding,
    whiteblacklist::{
        WhiteBlackListRenderer, WhiteBlackListRendererInit, WhiteBlackListRendererInput,
    },
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum PCIInput {
    RootRequest(RootRequest),
    WhiteBlackListChanged,
    Changed,
}

impl From<RootRequest> for PCIInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug)]
pub struct PCIGroup {
    initialized: bool,

    system_info: Option<SystemInfo>,
    profiles_info: Option<ProfilesInfo>,

    supports_aspm: bool,

    aspm_modes: StringListBinding,
    aspm_mode: U32Binding,

    enable_pci_pm: BoolBinding,

    pci_pm_whiteblacklist_renderer: Controller<WhiteBlackListRenderer>,

    awaiting_whiteblacklist_renderer_init: bool,

    last_pci_settings: Option<PCISettings>,
    last_aspm_settings: Option<ASPMSettings>,

    active_profile: Option<(usize, Profile)>,
}

impl PCIGroup {
    fn initialize_from_profile_and_system_info(&mut self) {
        assert!(self.profiles_info.is_some() && self.system_info.is_some());

        let profile = self.profiles_info.as_ref().unwrap().get_active_profile();
        let info = self.system_info.as_ref().unwrap();

        *self.enable_pci_pm.guard() = profile.pci_settings.enable_power_management.unwrap();

        self.awaiting_whiteblacklist_renderer_init = true;
        self.pci_pm_whiteblacklist_renderer
            .sender()
            .send(WhiteBlackListRendererInput::Init(
                WhiteBlackListRendererInit {
                    list: profile.pci_settings.whiteblacklist.clone().unwrap(),
                    rows: info
                        .pci_info
                        .pci_devices
                        .clone()
                        .into_iter()
                        .map(|d| {
                            [
                                d.pci_address,
                                d.display_name.strip_suffix("\n").unwrap().to_string(),
                            ]
                        })
                        .collect(),
                },
            ))
            .unwrap();

        if let Some(ref modes) = info.pci_info.aspm_info.supported_modes {
            *self.aspm_modes.guard() =
                gtk::StringList::new(&modes.iter().map(|v| v.as_str()).collect::<Vec<_>>());
            *self.aspm_mode.guard() = modes
                .iter()
                .position(|m| *m == *profile.aspm_settings.mode.as_ref().unwrap())
                .unwrap() as u32;
            self.supports_aspm = true;
        } else {
            self.supports_aspm = false;
        }

        self.initialized = true;
    }

    fn to_pci_settings(&self) -> PCISettings {
        PCISettings {
            enable_power_management: self.enable_pci_pm.value().into(),
            whiteblacklist: self
                .pci_pm_whiteblacklist_renderer
                .model()
                .to_whiteblacklist()
                .into(),
        }
    }
    fn to_aspm_settings(&self) -> ASPMSettings {
        ASPMSettings {
            mode: if self.supports_aspm {
                Some(
                    self.aspm_modes
                        .value()
                        .string(self.aspm_mode.value())
                        .unwrap()
                        .into(),
                )
            } else {
                None
            },
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for PCIGroup {
    type Input = PCIInput;

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
                            set_title: "PCIe Active State Power Management",
                            adw::ComboRow {
                                set_title: "ASPM operation mode",
                                #[watch]
                                set_sensitive: model.supports_aspm,
                                #[watch]
                                set_tooltip_text: if !model.supports_aspm {
                                    Some("Your system does not PCIe Active State Power Management")
                                } else {
                                    None
                                },
                                add_binding: (&model.aspm_modes, "model"),
                                add_binding: (&model.aspm_mode, "selected"),
                                connect_selected_item_notify => PCIInput::Changed,
                            },
                        },
                        adw::PreferencesGroup {
                            set_title: "PCI Power Management",
                            adw::SwitchRow {
                                set_title: "Enable PCI Power Management",
                                add_binding: (&model.enable_pci_pm, "active"),
                                connect_active_notify => PCIInput::Changed,
                            },

                        },
                        model.pci_pm_whiteblacklist_renderer.widget(),
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
        let pci_pm_whiteblacklist_renderer = WhiteBlackListRenderer::builder()
            .launch(())
            .forward(sender.input_sender(), |_| PCIInput::WhiteBlackListChanged);

        let model = PCIGroup {
            initialized: Default::default(),
            profiles_info: Default::default(),
            system_info: Default::default(),
            supports_aspm: Default::default(),
            aspm_modes: Default::default(),
            aspm_mode: Default::default(),
            enable_pci_pm: Default::default(),
            pci_pm_whiteblacklist_renderer,
            awaiting_whiteblacklist_renderer_init: Default::default(),
            last_pci_settings: Default::default(),
            last_aspm_settings: Default::default(),
            active_profile: Default::default(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            PCIInput::RootRequest(request) => match request {
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
                        self.last_pci_settings = Some(self.to_pci_settings());
                        self.last_aspm_settings = Some(self.to_aspm_settings());
                    }
                    sender.input(PCIInput::Changed);
                }
                RootRequest::ConfigureSystemInfoSync => {
                    system_info::set_system_info_sync(
                        Duration::from_secs_f32(10.0),
                        system_info::SystemInfoSyncType::PCI,
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
                    active_profile.1.pci_settings = self.to_pci_settings();
                    active_profile.1.aspm_settings = self.to_aspm_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::PCI,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            PCIInput::WhiteBlackListChanged => {
                if self.awaiting_whiteblacklist_renderer_init {
                    self.last_pci_settings = Some(self.to_pci_settings());
                    self.awaiting_whiteblacklist_renderer_init = false;
                }
                sender.input(PCIInput::Changed);
            }
            PCIInput::Changed => {
                if let Some(ref last_pci_settings) = self.last_pci_settings {
                    if let Some(ref last_aspm_settings) = self.last_aspm_settings {
                        sender
                            .output(AppInput::SetChanged(
                                *last_pci_settings != self.to_pci_settings()
                                    || *last_aspm_settings != self.to_aspm_settings(),
                                crate::SettingsGroup::PCI,
                            ))
                            .unwrap()
                    }
                }
            }
        }
    }
}
