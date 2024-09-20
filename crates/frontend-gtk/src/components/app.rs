use std::convert::identity;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use gtk::glib::clone;

use adw::prelude::*;
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::*;
use relm4::Controller;

use enumflags2::BitFlags;
use power_daemon::{Config, ProfilesInfo, SystemInfo};

use super::firmware::FirmwareGroup;
use super::groups::{
    cpu::CPUGroup, cpu_cores::CPUCoresGroup, kernel::KernelGroup, network::NetworkGroup,
    pci::PCIGroup, radio::RadioGroup, sata::SATAGroup, usb::USBGroup,
};

use super::settings::Settings;
use super::sleep::SleepGroup;
use super::Header;
use super::HeaderInput;
use crate::communications;
use crate::communications::system_info::SystemInfoSyncType;

use crate::profiles_updater;

#[derive(Debug, Clone, Copy)]
#[enumflags2::bitflags]
#[repr(u16)]
pub enum SettingsGroup {
    CPU,
    CPUCores,
    Radio,
    Network,
    PCI,
    USB,
    SATA,
    Kernel,
    Sleep,
    Firmware,
}

impl SettingsGroup {
    pub fn from_string(string: &str) -> SettingsGroup {
        match string {
            "CPU" => SettingsGroup::CPU,
            "CPU Cores" => SettingsGroup::CPUCores,
            "Radio" => SettingsGroup::Radio,
            "Network" => SettingsGroup::Network,
            "PCI" => SettingsGroup::PCI,
            "USB" => SettingsGroup::USB,
            "SATA" => SettingsGroup::SATA,
            "Kernel" => SettingsGroup::Kernel,
            "Sleep" => SettingsGroup::Sleep,
            "Firmware" => SettingsGroup::Firmware,
            _ => panic!("Unkown settings group"),
        }
    }
}

impl Display for SettingsGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SettingsGroup::CPU => "CPU",
            SettingsGroup::CPUCores => "CPU Cores",
            SettingsGroup::Radio => "Radio",
            SettingsGroup::Network => "Network",
            SettingsGroup::PCI => "PCI",
            SettingsGroup::USB => "USB",
            SettingsGroup::SATA => "SATA",
            SettingsGroup::Kernel => "Kernel",
            SettingsGroup::Sleep => "Sleep",
            SettingsGroup::Firmware => "Firmware",
        })
    }
}

#[derive(Debug)]
pub enum AppInput {
    SendRootRequestToAll(RootRequest),
    SendRootRequestToGroup(SettingsGroup, RootRequest),
    SendRootRequestToActiveGroup(RootRequest),
    /// Removes all possible None values. If not needed sends root request.
    UpdateProfilesOrSend(RootRequest),
    SetActiveGroupChanged(bool),
    SetChanged(bool, SettingsGroup),
    ToggleSettings(bool),
    ResetAllChanged,
    UpdateApplyButton,
    SetUpdating(bool),
}

#[derive(Debug, Clone)]
pub enum RootRequest {
    ReactToUpdate(AppSyncUpdate),
    ConfigureSystemInfoSync,
    Apply,
}

#[derive(Debug, Clone, PartialEq)]
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

    settings_dialog: Option<AsyncController<Settings>>,

    sleep_group: Controller<SleepGroup>,
    cpu_group: Controller<CPUGroup>,
    cpu_cores_group: Controller<CPUCoresGroup>,
    radio_group: Controller<RadioGroup>,
    network_group: Controller<NetworkGroup>,
    pci_group: Controller<PCIGroup>,
    usb_group: Controller<USBGroup>,
    sata_group: Controller<SATAGroup>,
    kernel_group: Controller<KernelGroup>,
    firmware_group: Controller<FirmwareGroup>,
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
        #[root]
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

        spawn_background_sync_thread().await;

        tokio::join!(
            communications::daemon_control::get_profiles_info(),
            communications::daemon_control::get_config(),
            communications::daemon_control::get_profile_override(),
        );

        communications::system_info::start_system_info_sync_routine();
        communications::system_info::set_system_info_sync(
            Duration::from_secs_f32(5.0),
            SystemInfoSyncType::None,
        );

        let sleep_group = SleepGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let cpu_group = CPUGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let cpu_cores_group = CPUCoresGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let radio_group = RadioGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let network_group = NetworkGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let pci_group = PCIGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let usb_group = USBGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let sata_group = SATAGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let kernel_group = KernelGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let firmware_group = FirmwareGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);

        let settings_group_stack = gtk::Stack::new();
        settings_group_stack.set_transition_type(gtk::StackTransitionType::SlideUpDown);

        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(sleep_group.widget())
                .build(),
            Some("Sleep"),
            "Sleep",
        );
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
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(network_group.widget())
                .build(),
            Some("Network"),
            "Network",
        );
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(pci_group.widget())
                .build(),
            Some("PCI"),
            "PCI",
        );
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(usb_group.widget())
                .build(),
            Some("USB"),
            "USB",
        );
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(sata_group.widget())
                .build(),
            Some("SATA"),
            "SATA",
        );
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(kernel_group.widget())
                .build(),
            Some("Kernel"),
            "Kernel",
        );
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(firmware_group.widget())
                .build(),
            Some("Firmware"),
            "Firmware",
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
            settings_dialog: None,
            cpu_group,
            cpu_cores_group,
            radio_group,
            network_group,
            pci_group,
            usb_group,
            sata_group,
            kernel_group,
            sleep_group,
            firmware_group,
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
                SettingsGroup::Sleep => self.sleep_group.sender().send(request.into()).unwrap(),
                SettingsGroup::CPU => self.cpu_group.sender().send(request.into()).unwrap(),
                SettingsGroup::CPUCores => {
                    self.cpu_cores_group.sender().send(request.into()).unwrap()
                }
                SettingsGroup::Radio => self.radio_group.sender().send(request.into()).unwrap(),
                SettingsGroup::Network => self.network_group.sender().send(request.into()).unwrap(),
                SettingsGroup::PCI => self.pci_group.sender().send(request.into()).unwrap(),
                SettingsGroup::USB => self.usb_group.sender().send(request.into()).unwrap(),
                SettingsGroup::SATA => self.sata_group.sender().send(request.into()).unwrap(),
                SettingsGroup::Kernel => self.kernel_group.sender().send(request.into()).unwrap(),
                SettingsGroup::Firmware => {
                    self.firmware_group.sender().send(request.into()).unwrap()
                }
            },
            AppInput::SendRootRequestToAll(request) => {
                self.header.sender().send(request.clone().into()).unwrap();

                self.sleep_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
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
                self.network_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
                self.pci_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
                self.usb_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
                self.sata_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
                self.kernel_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();
                self.firmware_group
                    .sender()
                    .send(request.clone().into())
                    .unwrap();

                if let Some(ref settings_dialog) = self.settings_dialog {
                    settings_dialog
                        .sender()
                        .send(request.clone().into())
                        .unwrap();
                }
            }
            AppInput::UpdateProfilesOrSend(request) => {
                if !profiles_updater::remove_all_none_options().await {
                    sender.input(AppInput::SendRootRequestToAll(request));
                }
            }
            AppInput::SetActiveGroupChanged(v) => {
                sender.input(AppInput::SetChanged(v, self.get_current_active_group()));
            }
            AppInput::SetChanged(v, group) => {
                if v {
                    self.changed_groups.insert(group);
                } else {
                    self.changed_groups.remove(group);
                }
                sender.input(AppInput::UpdateApplyButton);
            }
            AppInput::ResetAllChanged => {
                self.changed_groups = BitFlags::empty();
                sender.input(AppInput::UpdateApplyButton);
            }
            AppInput::ToggleSettings(v) => {
                if v {
                    self.settings_dialog = Settings::builder()
                        .launch(())
                        .forward(sender.input_sender(), identity)
                        .into();

                    let settings_dialog = self.settings_dialog.as_ref().unwrap();

                    settings_dialog
                        .sender()
                        .send(
                            RootRequest::ReactToUpdate(AppSyncUpdate::Config(
                                communications::CONFIG.get().await.clone().into(),
                            ))
                            .into(),
                        )
                        .unwrap();
                    settings_dialog
                        .sender()
                        .send(
                            RootRequest::ReactToUpdate(AppSyncUpdate::ProfilesInfo(
                                communications::PROFILES_INFO.get().await.clone().into(),
                            ))
                            .into(),
                        )
                        .unwrap();
                    settings_dialog.widget().show();
                }
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

async fn spawn_background_sync_thread() {
    let mut last_profile_name = communications::daemon_control::get_active_profile_name().await;
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs_f32(1.0)).await;
            let name = communications::daemon_control::get_active_profile_name().await;
            if name != last_profile_name {
                // All changes made by the frontend that could influence the
                // active profile are allways followed by the obtention of
                // profiles_info, if the current syncewd profiles info
                // aren't up to date with the current profile, then we sync
                // everything because we don't know what caused the change.
                let local_info_up_to_date = if let Some(profiles_info) =
                    communications::PROFILES_INFO.get().await.as_ref()
                {
                    profiles_info.get_active_profile().profile_name == name
                } else {
                    false
                };

                if !local_info_up_to_date {
                    log::debug!(
                        "The active profile changed unexpectedly, synchornizing with the daemon..."
                    );
                    tokio::join!(
                        communications::daemon_control::get_profiles_info(),
                        communications::daemon_control::get_config(),
                        communications::daemon_control::get_profile_override(),
                    );
                }

                last_profile_name = name;
            }
        }
    });
}

async fn setup_sync_listeners(sender: AsyncComponentSender<App>) {
    communications::PROFILES_INFO
        .set_listener(clone!(
            #[strong]
            sender,
            move |profiles_info| {
                sender.input(AppInput::UpdateProfilesOrSend(RootRequest::ReactToUpdate(
                    AppSyncUpdate::ProfilesInfo(Arc::from(profiles_info.cloned())),
                )));
            }
        ))
        .await;

    communications::CONFIG
        .set_listener(clone!(
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
        .set_listener(clone!(
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
        .set_listener(clone!(
            #[strong]
            sender,
            move |temp_override| {
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
