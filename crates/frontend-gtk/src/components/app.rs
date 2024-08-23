use std::convert::identity;
use std::sync::Arc;
use std::time::Duration;

use enumflags2::BitFlags;
use gtk::glib::clone;
use gtk::prelude::*;
use log::info;
use power_daemon::{CPUInfo, CPUSettings, RadioSettings};
use power_daemon::{Config, ProfilesInfo, SystemInfo};
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::*;
use relm4::Controller;

use super::groups::*;
use super::Header;
use super::HeaderInput;
use crate::communications::system_info::SystemInfoSyncType;
use crate::communications::{self, daemon_control};

#[derive(Debug, Clone, Copy)]
#[enumflags2::bitflags]
#[repr(u8)]
pub enum SettingsGroup {
    CPU,
    CPUCores,
    Radio,
}

impl SettingsGroup {
    pub fn from_string(string: &str) -> SettingsGroup {
        match string {
            "CPU" => SettingsGroup::CPU,
            "CPU Cores" => SettingsGroup::CPUCores,
            "Radio" => SettingsGroup::Radio,
            _ => panic!("Unkown settings group"),
        }
    }

    pub fn to_string(&self) -> String {
        String::from(match self {
            SettingsGroup::CPU => "CPU",
            SettingsGroup::CPUCores => "CPU Cores",
            SettingsGroup::Radio => "Radio",
        })
    }
}

#[derive(Debug)]
pub enum AppInput {
    SendRootRequestToAll(RootRequest),
    SendRootRequestToGroup(SettingsGroup, RootRequest),
    SendRootRequestToActiveGroup(RootRequest),
    SetChanged(bool, SettingsGroup),
    UpdateApplyButton,
    SetUpdating(bool),
}

#[derive(Debug, Clone)]
pub enum RootRequest {
    ReactToUpdate(AppSyncUpdate),
    ConfigureSystemInfoSync,
    Apply,
}

#[derive(Debug, Clone)]
pub enum AppSyncUpdate {
    ProfilesInfo(Arc<Option<ProfilesInfo>>),
    SystemInfo(Arc<Option<SystemInfo>>),
    Config(Arc<Option<Config>>),
    TemporaryOverride(Arc<Option<String>>),
}

pub struct App {
    updating: bool,
    changed_groups: BitFlags<SettingsGroup>,

    settings_group_stack: gtk::Stack,

    header: Controller<Header>,

    cpu_group: Controller<CPUGroup>,
    cpu_cores_group: Controller<CPUCoresGroup>,
    radio_group: Controller<RadioGroup>,
}

impl App {
    pub fn get_current_active_group(&self) -> SettingsGroup {
        SettingsGroup::from_string(&self.settings_group_stack.visible_child_name().unwrap())
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for App {
    type Input = AppInput;
    type Output = ();

    type Init = ();

    view! {
        gtk::ApplicationWindow {
            set_titlebar: Some(model.header.widget()),
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
                gtk::Paned {
                    set_position: 200,
                    #[wrap(Some)]
                    set_start_child= &gtk::StackSidebar {
                        set_stack = &model.settings_group_stack.clone(),
                    },
                    #[wrap(Some)]
                    set_end_child=&model.settings_group_stack.clone(),
                }

            }
        }
    }

    fn init_loading_widgets(root: Self::Root) -> Option<LoadingWidgets> {
        relm4::view! {
            #[local]
            root {
                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        communications::daemon_control::setup_control_client().await;

        tokio::join!(
            communications::daemon_control::get_profiles_info(),
            communications::daemon_control::get_profile_override(),
            communications::daemon_control::get_config(),
            communications::daemon_control::get_profile_override(),
        );

        communications::system_info::start_system_info_sync_routine();
        communications::system_info::set_system_info_sync(
            Duration::from_secs_f32(5.0),
            SystemInfoSyncType::None,
        );

        remove_all_none_options().await;

        let cpu_group = CPUGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let cpu_cores_group = CPUCoresGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let radio_group = RadioGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);

        let settings_group_stack = gtk::Stack::new();
        settings_group_stack.set_transition_type(gtk::StackTransitionType::SlideUpDown);
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(cpu_group.widget())
                .build(),
            Some("CPU"),
            "CPU",
        );
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(cpu_cores_group.widget())
                .build(),
            Some("CPU Cores"),
            "CPU Cores",
        );
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(radio_group.widget())
                .build(),
            Some("Radio"),
            "Radio",
        );

        {
            let sender = sender.clone();
            settings_group_stack.connect_visible_child_notify(move |_| {
                sender.input(AppInput::SendRootRequestToActiveGroup(
                    RootRequest::ConfigureSystemInfoSync,
                ));
                sender.input(AppInput::UpdateApplyButton);
            });
        }

        let model = App {
            updating: false,
            changed_groups: BitFlags::empty(),
            header: Header::builder()
                .launch(())
                .forward(sender.input_sender(), identity),

            settings_group_stack,
            cpu_group,
            cpu_cores_group,
            radio_group,
        };

        let widgets = view_output!();

        setup_sync_listeners(sender).await;

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        match message {
            AppInput::SendRootRequestToActiveGroup(request) => {
                sender.input(AppInput::SendRootRequestToGroup(
                    self.get_current_active_group(),
                    request,
                ));
            }
            AppInput::SendRootRequestToGroup(group, request) => match group {
                SettingsGroup::CPU => self.cpu_group.sender().send(request.into()).unwrap(),
                SettingsGroup::CPUCores => {
                    self.cpu_cores_group.sender().send(request.into()).unwrap()
                }
                SettingsGroup::Radio => self.radio_group.sender().send(request.into()).unwrap(),
            },
            AppInput::SendRootRequestToAll(request) => {
                self.header.sender().send(request.clone().into()).unwrap();
                self.cpu_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
                self.cpu_cores_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
                self.radio_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
            }
            AppInput::SetChanged(v, group) => {
                if v {
                    self.changed_groups.insert(group);
                } else {
                    self.changed_groups.remove(group);
                }
                sender.input(AppInput::UpdateApplyButton);
            }
            AppInput::SetUpdating(v) => {
                self.updating = v;
            }
            AppInput::UpdateApplyButton => self
                .header
                .sender()
                .send(HeaderInput::AllowApplyButton(
                    self.changed_groups
                        .contains(self.get_current_active_group()),
                ))
                .unwrap(),
        }
    }
}

async fn setup_sync_listeners(sender: AsyncComponentSender<App>) {
    communications::PROFILES_INFO
        .listen(clone!(
            #[strong]
            sender,
            move |profiles_info| {
                sender.input(AppInput::SendRootRequestToAll(RootRequest::ReactToUpdate(
                    AppSyncUpdate::ProfilesInfo(Arc::from(profiles_info.cloned())),
                )));
            }
        ))
        .await;

    communications::CONFIG
        .listen(clone!(
            #[strong]
            sender,
            move |config| {
                sender.input(AppInput::SendRootRequestToAll(RootRequest::ReactToUpdate(
                    AppSyncUpdate::Config(Arc::from(config.cloned())),
                )));
            }
        ))
        .await;

    communications::SYSTEM_INFO
        .listen(clone!(
            #[strong]
            sender,
            move |system_info| {
                sender.input(AppInput::SendRootRequestToAll(RootRequest::ReactToUpdate(
                    AppSyncUpdate::SystemInfo(Arc::from(system_info.cloned())),
                )));
            }
        ))
        .await;

    communications::PROFILE_OVERRIDE
        .listen(clone!(
            #[strong]
            sender,
            move |temp_override| {
                //
                let mut temp_override = temp_override.cloned();
                if temp_override.is_none() {
                    temp_override = Some(None);
                }

                sender.input(AppInput::SendRootRequestToAll(RootRequest::ReactToUpdate(
                    AppSyncUpdate::TemporaryOverride(Arc::from(temp_override.unwrap())),
                )));
            }
        ))
        .await;
}

/// Iterates through all profiles and removes all possible None options. Except
/// for those that the system does not support and need to be set to None.
async fn remove_all_none_options() {
    info!("The GTK frontend does not support configurations with ignored settings (unless those settings are unsupported by the system). Updating profiles if neccessary now.");

    communications::system_info::obtain_full_info_once().await;

    assert!(!communications::SYSTEM_INFO.is_none().await);
    assert!(!communications::PROFILES_INFO.is_none().await);

    let info = communications::SYSTEM_INFO.get().await.clone().unwrap();

    for (idx, mut profile) in communications::PROFILES_INFO
        .get()
        .await
        .as_ref()
        .unwrap()
        .profiles
        .clone()
        .into_iter()
        .enumerate()
    {
        let initial = profile.clone();

        default_cpu_settings(&mut profile.cpu_settings, &info.cpu_info);

        // The CPU core settings component works by reading system info not
        // profiles info, so there is no need to update the individual core settings. As
        // those will be updated on demand by the component.

        default_radio_settings(&mut profile.radio_settings);

        if initial != profile {
            daemon_control::reset_reduced_update().await;
            daemon_control::update_profile(idx as u32, profile).await;
        }
    }

    daemon_control::get_profiles_info().await;
}

fn default_cpu_settings(settings: &mut CPUSettings, cpu_info: &CPUInfo) {
    if settings.mode.is_none() {
        // cpu info mode will be none if unsupported so we won't be overriding
        // unsupported settings
        settings.mode = cpu_info.mode.clone();
    }

    if settings.governor.is_none() {
        // Available in both passive and active, the safest option
        settings.governor = String::from("powersave").into();
    }
    if settings.epp.is_none() && cpu_info.has_epp {
        settings.epp = String::from("default").into();
    }

    if settings.min_freq.is_none() {
        settings.min_freq = cpu_info.total_min_frequency.into();
    }
    if settings.max_freq.is_none() {
        settings.max_freq = cpu_info.total_max_frequency.into();
    }

    if settings.min_perf_pct.is_none() && cpu_info.has_perf_pct_scaling {
        settings.min_perf_pct = 0.into();
    }
    if settings.max_perf_pct.is_none() && cpu_info.has_perf_pct_scaling {
        settings.max_perf_pct = 100.into();
    }

    if settings.boost.is_none() {
        settings.boost = cpu_info.boost;
    }
    if settings.hwp_dyn_boost.is_none() {
        settings.boost = cpu_info.hwp_dynamic_boost;
    }
}

fn default_radio_settings(settings: &mut RadioSettings) {
    if settings.block_bt.is_none() {
        settings.block_bt = Some(false);
    }
    if settings.block_wifi.is_none() {
        settings.block_wifi = Some(false);
    }
    if settings.block_nfc.is_none() {
        settings.block_nfc = Some(false);
    }
}
