use std::time::Duration;

use adw::prelude::*;
use power_daemon::{
    IntelRaplInterfaceInfo, IntelRaplInterfaceSettings, IntelRaplSettings, Profile, ProfilesInfo,
    SystemInfo,
};
use relm4::{
    binding::{Binding, U32Binding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum RaplInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for RaplInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug)]
pub struct RaplGroup {
    initialized: bool,

    system_info: Option<SystemInfo>,
    profiles_info: Option<ProfilesInfo>,

    package: Controller<RaplInterfaceRenderer>,
    core: Controller<RaplInterfaceRenderer>,
    uncore: Controller<RaplInterfaceRenderer>,

    last_rapl_settings: Option<IntelRaplSettings>,
    active_profile: Option<(usize, Profile)>,
}

impl RaplGroup {
    #[allow(clippy::wrong_self_convention)]
    fn intialize(&mut self) {
        assert!(self.system_info.is_some() && self.profiles_info.is_some());

        let rapl_settings = &self
            .profiles_info
            .as_ref()
            .unwrap()
            .get_active_profile()
            .rapl_settings;

        let rapl_info = &self.system_info.as_ref().unwrap().rapl_info;

        if let Some(interface) = rapl_settings.package.clone() {
            self.package
                .sender()
                .send(InterfaceRendererInput::Settings(interface))
                .unwrap();
        }
        if let Some(interface) = rapl_settings.core.clone() {
            self.core
                .sender()
                .send(InterfaceRendererInput::Settings(interface))
                .unwrap();
        }
        if let Some(interface) = rapl_settings.uncore.clone() {
            self.uncore
                .sender()
                .send(InterfaceRendererInput::Settings(interface))
                .unwrap();
        }

        self.package
            .sender()
            .send(InterfaceRendererInput::Info(rapl_info.package.clone()))
            .unwrap();
        self.core
            .sender()
            .send(InterfaceRendererInput::Info(rapl_info.core.clone()))
            .unwrap();
        self.uncore
            .sender()
            .send(InterfaceRendererInput::Info(rapl_info.uncore.clone()))
            .unwrap();

        self.initialized = true;
    }

    fn to_rapl_settings(&self) -> IntelRaplSettings {
        IntelRaplSettings {
            package: self.package.model().to_interface_settings(),
            core: self.core.model().to_interface_settings(),
            uncore: self.uncore.model().to_interface_settings(),
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for RaplGroup {
    type Input = RaplInput;

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
                adw::PreferencesPage {
                    container_add: model.package.widget(),
                    container_add: model.core.widget(),
                    container_add: model.uncore.widget(),
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RaplGroup {
            initialized: false,
            system_info: None,
            profiles_info: None,

            package: RaplInterfaceRenderer::builder()
                .launch("Package".to_string())
                .forward(sender.input_sender(), |_| RaplInput::Changed),
            core: RaplInterfaceRenderer::builder()
                .launch("Core".to_string())
                .forward(sender.input_sender(), |_| RaplInput::Changed),
            uncore: RaplInterfaceRenderer::builder()
                .launch("Uncore".to_string())
                .forward(sender.input_sender(), |_| RaplInput::Changed),

            active_profile: None,
            last_rapl_settings: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            RaplInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(ref profiles_info) = profiles_info.as_ref() {
                            self.profiles_info = Some(profiles_info.clone());
                            self.initialized = false;
                            self.active_profile = Some((
                                profiles_info.active_profile,
                                profiles_info.get_active_profile().clone(),
                            ));
                        }
                    }
                    if let AppSyncUpdate::SystemInfo(ref system_info) = message {
                        if let Some(system_info) = system_info.as_ref() {
                            if let Some(ref local_system_info) = self.system_info {
                                if system_info != local_system_info {
                                    self.initialized = false;
                                }
                            }
                            self.system_info = Some(system_info.clone());
                        }
                    }

                    if !self.initialized
                        && self.profiles_info.is_some()
                        && self.system_info.is_some()
                    {
                        self.intialize();
                        self.last_rapl_settings = Some(self.to_rapl_settings());
                    }
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(10.0),
                    system_info::SystemInfoSyncType::None,
                ),
                RootRequest::Apply => {
                    if !(self.initialized) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.rapl_settings = self.to_rapl_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::Rapl,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            RaplInput::Changed => {
                if let Some(ref last_settings) = self.last_rapl_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_rapl_settings(),
                            crate::SettingsGroup::Rapl,
                        ))
                        .unwrap()
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum InterfaceRendererInput {
    Settings(IntelRaplInterfaceSettings),
    Info(Option<IntelRaplInterfaceInfo>),
    Changed,
}

#[derive(Debug, Clone, Default)]
struct RaplInterfaceRenderer {
    name: String,
    supported: bool,
    settings: InterfaceSettings,
    features: InterfaceFeatures,
}

impl RaplInterfaceRenderer {
    pub fn to_interface_settings(&self) -> Option<IntelRaplInterfaceSettings> {
        if self.supported {
            Some(IntelRaplInterfaceSettings {
                long_term_limit: if self.features.long_term {
                    Some(self.settings.long_term.value())
                } else {
                    None
                },
                short_term_limit: if self.features.short_term {
                    Some(self.settings.short_term.value())
                } else {
                    None
                },
                peak_power_limit: if self.features.peak_power {
                    Some(self.settings.peak_power.value())
                } else {
                    None
                },
            })
        } else {
            None
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for RaplInterfaceRenderer {
    type Input = InterfaceRendererInput;

    type Output = ();

    type Init = String;

    view! {
        adw::PreferencesGroup {
            set_title: &model.name,
            #[watch]
            set_sensitive: model.supported,
            #[watch]
            set_tooltip_text: if !model.supported {
                Some(labels::RAPL_INTERFACE_UNSUPPORTED)
            } else {
                None
            },
            adw::SpinRow::with_range(0.0, f64::MAX, 1.0) {
                set_title: labels::RAPL_LONG_TERM_TITLE,
                #[watch]
                set_sensitive: model.features.long_term,
                #[watch]
                add_binding: (&model.settings.long_term, "value"),
                #[watch]
                set_tooltip_text: if !model.features.long_term {
                    Some(labels::RAPL_CONSTRAINT_UNSUPPORTED)
                } else {
                    Some(labels::RAPL_LONG_TERM_TT)
                },
                connect_value_notify => InterfaceRendererInput::Changed,
            },
            adw::SpinRow::with_range(0.0, f64::MAX, 1.0) {
                set_title: labels::RAPL_SHORT_TERM_TITLE,
                #[watch]
                set_sensitive: model.features.short_term,
                #[watch]
                add_binding: (&model.settings.short_term, "value"),
                #[watch]
                set_tooltip_text: if !model.features.short_term {
                    Some(labels::RAPL_CONSTRAINT_UNSUPPORTED)
                } else {
                    Some(labels::RAPL_SHORT_TERM_TT)
                },
                connect_value_notify => InterfaceRendererInput::Changed,
            },
            adw::SpinRow::with_range(0.0, f64::MAX, 1.0) {
                set_title: labels::RAPL_PEAK_POWER_TITLE,
                #[watch]
                set_sensitive: model.features.peak_power,
                #[watch]
                add_binding: (&model.settings.peak_power, "value"),
                #[watch]
                set_tooltip_text: if !model.features.peak_power {
                    Some(labels::RAPL_CONSTRAINT_UNSUPPORTED)
                } else {
                    Some(labels::RAPL_PEAK_POWER_TT)
                },
                connect_value_notify => InterfaceRendererInput::Changed,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RaplInterfaceRenderer {
            name: init,
            supported: false,
            settings: InterfaceSettings::default(),
            features: InterfaceFeatures::default(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            InterfaceRendererInput::Settings(interface) => {
                self.settings.from_daemon_settings(&interface);
            }
            InterfaceRendererInput::Info(interface) => {
                if let Some(interface) = interface {
                    self.supported = true;
                    self.features = InterfaceFeatures::from_info(&interface);
                } else {
                    self.supported = false;
                }
            }
            InterfaceRendererInput::Changed => sender.output(()).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct InterfaceSettings {
    pub long_term: U32Binding,
    pub short_term: U32Binding,
    pub peak_power: U32Binding,
}

impl InterfaceSettings {
    #[allow(clippy::wrong_self_convention)]
    pub fn from_daemon_settings(&mut self, interface: &IntelRaplInterfaceSettings) {
        *self.long_term.guard() = interface.long_term_limit.unwrap_or_default();
        *self.short_term.guard() = interface.short_term_limit.unwrap_or_default();
        *self.peak_power.guard() = interface.peak_power_limit.unwrap_or_default();
    }
}

#[derive(Debug, Clone, Default)]
struct InterfaceFeatures {
    pub long_term: bool,
    pub short_term: bool,
    pub peak_power: bool,
}

impl InterfaceFeatures {
    pub fn from_info(info: &IntelRaplInterfaceInfo) -> Self {
        Self {
            long_term: info.long_term.is_some(),
            short_term: info.short_term.is_some(),
            peak_power: info.peak_power.is_some(),
        }
    }
}
