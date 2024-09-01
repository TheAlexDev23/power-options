use std::{collections::HashMap, convert::identity};

use adw::prelude::*;
use log::debug;
use power_daemon::{Config, DefaultProfileType, Profile};
use relm4::{
    binding::{Binding, BoolBinding, U32Binding},
    factory::FactoryVecDeque,
    prelude::*,
    RelmObjectExt,
};

use crate::{communications::daemon_control, helpers::extra_bindings::StringListBinding};

use super::{dialog::Dialog, AppInput, AppSyncUpdate, RootRequest};

#[derive(Debug, Clone)]
pub enum SettingsInput {
    RootRequest(RootRequest),
    SetUpdating(bool),
    ManageProfiles(ProfileManagementAction),
    Changed(ChangeAction),
    AskAndRemoveProfile(DynamicIndex),
    AskAndResetProfile(DynamicIndex),
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ChangeAction {
    Bat,
    AC,
    PersistentOverride,
}

#[derive(Debug, Clone)]
pub enum ProfileManagementAction {
    CreateProfile,
    MoveProfileUp(DynamicIndex),
    MoveProfileDown(DynamicIndex),
    ChangeProfileName(String, DynamicIndex),
    ResetProfile(DynamicIndex),
    DeleteProfile(DynamicIndex),
}

impl From<RootRequest> for SettingsInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}
impl From<ProfileManagementAction> for SettingsInput {
    fn from(value: ProfileManagementAction) -> Self {
        Self::ManageProfiles(value)
    }
}

pub struct Settings {
    updating: bool,

    supressed_actions: HashMap<ChangeAction, u32>,

    last_config: Option<Config>,

    profiles: FactoryVecDeque<ProfileFactoryRenderer>,

    available_profiles: StringListBinding,
    selected_bat_profile: U32Binding,
    selected_ac_profile: U32Binding,

    default_profile_types: StringListBinding,

    new_profile_type: U32Binding,

    persistent_override_set: BoolBinding,
    selected_persitent_override: U32Binding,
}

impl Settings {
    pub fn from_config(&mut self, config: &Config) {
        self.supressed_actions.clear();

        self.supressed_actions.insert(ChangeAction::Bat, 2);
        self.supressed_actions.insert(ChangeAction::AC, 2);

        self.supressed_actions.insert(
            ChangeAction::PersistentOverride,
            // When profile override is none, the only update done to the
            // component will be the model update. Otherwise two updates will be performed
            if config.profile_override.is_some() {
                2
            } else {
                1
            },
        );

        *self.available_profiles.guard() = gtk::StringList::new(
            &config
                .profiles
                .iter()
                .map(|e| &e as &str)
                .collect::<Vec<&str>>(),
        );

        *self.selected_bat_profile.guard() = config
            .profiles
            .iter()
            .position(|p| *p == config.bat_profile)
            .unwrap() as u32;
        *self.selected_ac_profile.guard() = config
            .profiles
            .iter()
            .position(|p| *p == config.ac_profile)
            .unwrap() as u32;

        if let Some(ref persistent_override) = config.profile_override {
            *self.persistent_override_set.guard() = true;
            *self.selected_persitent_override.guard() = config
                .profiles
                .iter()
                .position(|p| p == persistent_override)
                .unwrap() as u32;
        } else {
            *self.persistent_override_set.guard() = false;
        }
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for Settings {
    type Input = SettingsInput;

    type Output = AppInput;

    type Init = ();

    view! {
        gtk::Box {
            if model.updating {
                gtk::Box {
                    set_align: gtk::Align::Center,
                    gtk::Label::new(Some("Applying...")),
                    gtk::Spinner {
                        set_spinning: true,
                        set_visible: true,
                    }
                }
            } else {
                adw::PreferencesPage {
                    set_expand: true,

                    adw::PreferencesGroup {
                        adw::ComboRow {
                            set_title: "Battery profile",
                            set_tooltip_text: Some("The profile used for when the system is on battery power"),
                            add_binding: (&model.available_profiles, "model"),
                            add_binding: (&model.selected_bat_profile, "selected"),
                            connect_selected_item_notify => SettingsInput::Changed(ChangeAction::Bat),
                        },
                        adw::ComboRow {
                            set_title: "AC profile",
                            set_tooltip_text: Some("The profile used for when the system is connected to the wall"),
                            add_binding: (&model.available_profiles, "model"),
                            add_binding: (&model.selected_ac_profile, "selected"),
                            connect_selected_item_notify => SettingsInput::Changed(ChangeAction::AC),
                        },
                        adw::SwitchRow {
                            set_title: "Create a persistent override",
                            set_tooltip_text: Some("If a profile override is set, the predefined battery and ac profiles will not be selected on demand, and this profile will be prefered. Note that unlike a temporary override selected by pressing the individual profiles in the UI, a persistent override is saved in the configuration and will persist among reboots, however a temporary override is prioritized over persistent overrides."),
                            add_binding: (&model.persistent_override_set, "active"),
                            connect_active_notify => SettingsInput::Changed(ChangeAction::PersistentOverride),
                        },
                        adw::ComboRow {
                            set_title: "Persistent override",
                            add_binding: (&model.available_profiles, "model"),
                            add_binding: (&model.selected_persitent_override, "selected"),
                            add_binding: (&model.persistent_override_set, "visible"),
                            connect_selected_item_notify => SettingsInput::Changed(ChangeAction::PersistentOverride),
                        },
                    },

                    adw::PreferencesGroup {
                        adw::ComboRow {
                            set_title: "New profile type",
                            add_binding: (&model.available_profiles, "model"),
                            add_binding: (&model.new_profile_type, "selected"),
                        },
                        gtk::Button {
                            set_label: "Create profile",
                            connect_clicked => SettingsInput::ManageProfiles(ProfileManagementAction::CreateProfile),
                        },
                    },

                    adw::PreferencesGroup {
                        container_add: model.profiles.widget(),
                    }
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Settings {
            updating: false,

            supressed_actions: HashMap::new(),

            last_config: None,

            profiles: FactoryVecDeque::builder()
                .launch(gtk::Box::new(gtk::Orientation::Vertical, 0))
                .forward(sender.input_sender(), identity),

            default_profile_types: StringListBinding::new(gtk::StringList::new(&[
                &DefaultProfileType::Superpowersave.get_name(),
                &DefaultProfileType::Powersave.get_name(),
                &DefaultProfileType::Balanced.get_name(),
                &DefaultProfileType::Performance.get_name(),
                &DefaultProfileType::Ultraperformance.get_name(),
            ])),

            new_profile_type: Default::default(),
            available_profiles: Default::default(),
            selected_bat_profile: Default::default(),
            selected_ac_profile: Default::default(),
            persistent_override_set: Default::default(),
            selected_persitent_override: Default::default(),
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        match message {
            SettingsInput::RootRequest(request) => {
                if let RootRequest::ReactToUpdate(update) = request {
                    if let AppSyncUpdate::ProfilesInfo(profiles_info) = &update {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let one_profile = profiles_info.profiles.len() == 1;

                            let mut guard = self.profiles.guard();
                            guard.clear();
                            for (idx, profile) in
                                profiles_info.profiles.clone().into_iter().enumerate()
                            {
                                guard.push_back(ProfileFactoryRenderer {
                                    is_active_profile: profiles_info.active_profile == idx,
                                    only_one_profile: one_profile,
                                    profile,
                                });
                            }
                        }
                    }
                    if let AppSyncUpdate::Config(config) = &update {
                        if let Some(config) = config.as_ref() {
                            self.from_config(config);
                        }
                        self.last_config = (**config).clone();
                    }
                }
            }
            SettingsInput::ManageProfiles(change) => match change {
                ProfileManagementAction::CreateProfile => {
                    let profile_type = DefaultProfileType::from_name(
                        self.default_profile_types
                            .value()
                            .string(self.new_profile_type.value())
                            .unwrap()
                            .to_string(),
                    )
                    .unwrap();

                    tokio::spawn(async move {
                        sender.input(SettingsInput::SetUpdating(true));

                        daemon_control::create_profile(profile_type).await;
                        daemon_control::get_config().await;
                        daemon_control::get_profiles_info().await;

                        sender.input(SettingsInput::SetUpdating(false));
                    });
                }
                ProfileManagementAction::MoveProfileUp(idx) => {
                    let idx = idx.current_index();
                    let new_idx = idx + 1;
                    if new_idx < self.profiles.guard().len() {
                        tokio::spawn(async move {
                            sender.input(SettingsInput::SetUpdating(true));

                            daemon_control::swap_profiles(idx as u32, new_idx as u32).await;

                            daemon_control::get_config().await;
                            daemon_control::get_profiles_info().await;

                            sender.input(SettingsInput::SetUpdating(false));
                        });
                    }
                }
                ProfileManagementAction::MoveProfileDown(idx) => {
                    let idx = idx.current_index();
                    if idx > 0 {
                        let new_idx = idx - 1;
                        tokio::spawn(async move {
                            sender.input(SettingsInput::SetUpdating(true));

                            daemon_control::swap_profiles(idx as u32, new_idx as u32).await;

                            daemon_control::get_config().await;
                            daemon_control::get_profiles_info().await;

                            sender.input(SettingsInput::SetUpdating(false));
                        });
                    }
                }
                ProfileManagementAction::ChangeProfileName(name, idx) => {
                    let profile_idx = idx.current_index();
                    tokio::spawn(async move {
                        sender.input(SettingsInput::SetUpdating(true));

                        daemon_control::update_profile_name(profile_idx as u32, name).await;

                        daemon_control::get_config().await;
                        daemon_control::get_profiles_info().await;
                        daemon_control::get_profile_override().await;

                        sender.input(SettingsInput::SetUpdating(false));
                    });
                }
                ProfileManagementAction::ResetProfile(idx) => {
                    let idx = idx.current_index() as u32;
                    tokio::spawn(async move {
                        sender.input(SettingsInput::SetUpdating(true));

                        daemon_control::reset_profile(idx).await;
                        daemon_control::get_profiles_info().await;

                        sender.input(SettingsInput::SetUpdating(false));
                    });
                }
                ProfileManagementAction::DeleteProfile(idx) => {
                    let idx = idx.current_index() as u32;
                    tokio::spawn(async move {
                        sender.input(SettingsInput::SetUpdating(true));

                        daemon_control::remove_profile(idx).await;
                        daemon_control::get_config().await;
                        daemon_control::get_profiles_info().await;
                        daemon_control::get_profile_override().await;

                        sender.input(SettingsInput::SetUpdating(false));
                    });
                }
            },
            SettingsInput::AskAndRemoveProfile(idx) => {
                let dialog = Dialog {
                    heading: "Are you sure you want to remove this profile?".to_string(),
                    body: "This action is irreversible".to_string(),
                    accept_is_danger: true,
                    accept_label: "Remove".to_string(),
                    deny_label: "Cancel".to_string(),
                };

                if dialog.show().await {
                    sender.input(SettingsInput::ManageProfiles(
                        ProfileManagementAction::DeleteProfile(idx),
                    ))
                }
            }
            SettingsInput::AskAndResetProfile(idx) => {
                let dialog = Dialog {
                    heading: "Are you sure you want to reset this profile?".to_string(),
                    body: "This action is irreversible".to_string(),
                    accept_is_danger: true,
                    accept_label: "Reset".to_string(),
                    deny_label: "Cancel".to_string(),
                };

                if dialog.show().await {
                    sender.input(SettingsInput::ManageProfiles(
                        ProfileManagementAction::ResetProfile(idx),
                    ))
                }
            }
            SettingsInput::SetUpdating(v) => self.updating = v,
            SettingsInput::Changed(changed) => {
                if let Some(val) = self.supressed_actions.get_mut(&changed) {
                    if *val != 0 {
                        debug!("Action {changed:?} supressed");
                        *val -= 1;
                    } else if let Some(mut config) = self.last_config.clone() {
                        debug!("Applying action {changed:?}");
                        match changed {
                            ChangeAction::Bat => {
                                config.bat_profile = config.profiles
                                    [self.selected_bat_profile.value() as usize]
                                    .clone();
                                tokio::spawn(async move {
                                    sender.input(SettingsInput::SetUpdating(true));

                                    daemon_control::update_config(config).await;
                                    daemon_control::get_config().await;
                                    daemon_control::get_profiles_info().await;

                                    sender.input(SettingsInput::SetUpdating(false));
                                });
                            }
                            ChangeAction::AC => {
                                config.ac_profile = config.profiles
                                    [self.selected_ac_profile.value() as usize]
                                    .clone();
                                tokio::spawn(async move {
                                    sender.input(SettingsInput::SetUpdating(true));

                                    daemon_control::update_config(config).await;
                                    daemon_control::get_config().await;
                                    daemon_control::get_profiles_info().await;

                                    sender.input(SettingsInput::SetUpdating(false));
                                });
                            }
                            ChangeAction::PersistentOverride => {
                                config.profile_override = if self.persistent_override_set.value() {
                                    Some(
                                        config.profiles
                                            [self.selected_persitent_override.value() as usize]
                                            .clone(),
                                    )
                                } else {
                                    None
                                };

                                tokio::spawn(async move {
                                    sender.input(SettingsInput::SetUpdating(true));

                                    daemon_control::update_config(config).await;
                                    daemon_control::get_config().await;
                                    daemon_control::get_profiles_info().await;

                                    sender.input(SettingsInput::SetUpdating(false));
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ProfileFactoryRenderer {
    only_one_profile: bool,
    is_active_profile: bool,
    profile: Profile,
}

#[relm4::factory]
impl FactoryComponent for ProfileFactoryRenderer {
    type Init = ProfileFactoryRenderer;
    type Input = ();
    type Output = SettingsInput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_align: gtk::Align::Center,
            set_spacing: 10,

            gtk::EditableLabel {
                #[watch]
                #[block_signal(changed_handler)]
                set_text: &self.profile.profile_name,
                connect_editing_notify[sender, index, old_name = self.profile.profile_name.clone()] => move |v| {
                    if !v.is_editing() && v.text() != old_name {
                        sender.output(ProfileManagementAction::ChangeProfileName(v.text().into(), index.clone()).into()).unwrap();
                    }
                } @changed_handler
            },
            gtk::Button {
                set_label: "Up",
                connect_clicked[sender, index] => move |_| {
                    sender.output(ProfileManagementAction::MoveProfileUp(index.clone()).into()).unwrap();
                }
            },
            gtk::Button {
                set_label: "Down",
                connect_clicked[sender, index] => move |_| {
                    sender.output(ProfileManagementAction::MoveProfileDown(index.clone()).into()).unwrap();
                }
            },
            gtk::Button {
                set_label: "Reset to defaults",
                connect_clicked[sender, index] => move |_| {
                    sender.output(SettingsInput::AskAndResetProfile(index.clone())).unwrap();
                }
            },
            gtk::Button {
                set_label: "Delete",
                #[watch]
                set_sensitive: !self.only_one_profile && !self.is_active_profile,
                connect_clicked[sender, index] => move |_| {
                    sender.output(SettingsInput::AskAndRemoveProfile(index.clone())).unwrap();
                }
            },
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        value
    }
}
