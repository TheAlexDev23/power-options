use std::convert::identity;
use std::sync::Arc;

use gtk::glib::clone;
use gtk::prelude::*;
use power_daemon::{Config, ProfilesInfo, SystemInfo};
use relm4::prelude::*;
use relm4::Controller;

use super::groups::*;
use super::Header;
use super::HeaderInput;
use crate::communications;

#[derive(Debug)]
pub enum AppInput {
    ApplySettings,
    SetChanged(bool),
    SetUpdating(bool),
}

#[derive(Debug, Clone)]
pub enum AppSyncUpdate {
    ProfilesInfo(Arc<Option<ProfilesInfo>>),
    SystemInfo(Arc<Option<SystemInfo>>),
    Config(Arc<Option<Config>>),
}

pub struct App {
    updating: bool,

    settings_group_stack: gtk::Stack,

    header: Controller<Header>,

    cpu_group: Controller<CPUGroup>,
    cpu_cores_group: Controller<CPUCoresGroup>,
    radio_group: Controller<RadioGroup>,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for App {
    type Input = AppInput;
    type Output = AppSyncUpdate;

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

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
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
            let cpu_group = cpu_group.sender().clone();
            let cpu_cores_group = cpu_cores_group.sender().clone();
            let radio_group = radio_group.sender().clone();
            settings_group_stack.connect_visible_child_notify(move |stack| {
                sender.input(AppInput::SetChanged(false));
                let name = stack.visible_child_name().unwrap();
                if name == "CPU" {
                    cpu_group.send(CPUInput::ConfigureSysinfo).unwrap();
                } else if name == "CPU Cores" {
                    cpu_cores_group
                        .send(CPUCoresInput::ConfigureSysinfo)
                        .unwrap();
                } else if name == "Radio" {
                    radio_group.send(RadioInput::ConfigureSysinfo).unwrap();
                }
            });
        }

        let model = App {
            updating: false,
            header: Header::builder()
                .launch(())
                .forward(sender.input_sender(), identity),

            settings_group_stack,
            cpu_group,
            cpu_cores_group,
            radio_group,
        };

        let widgets = view_output!();

        model.setup_sync_listeners().await;

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            AppInput::ApplySettings => {
                if let Some(name) = self.settings_group_stack.visible_child_name() {
                    if name == "CPU" {
                        self.cpu_group.sender().send(CPUInput::Apply).unwrap();
                    } else if name == "CPU Cores" {
                        self.cpu_cores_group
                            .sender()
                            .send(CPUCoresInput::Apply)
                            .unwrap();
                    } else if name == "Radio" {
                        self.radio_group.sender().send(RadioInput::Apply).unwrap()
                    }
                }
            }
            AppInput::SetChanged(v) => {
                self.header
                    .sender()
                    .send(HeaderInput::AllowApplyButton(v))
                    .unwrap();
            }
            AppInput::SetUpdating(v) => {
                self.updating = v;
            }
        }
    }
}

impl App {
    async fn setup_sync_listeners(&self) {
        let send_to_all = {
            let header_sender = self.header.sender().clone();
            let cpu_sender = self.cpu_group.sender().clone();
            let cpu_cores_sender = self.cpu_cores_group.sender().clone();
            let radio_sender = self.radio_group.sender().clone();
            move |msg: AppSyncUpdate| {
                header_sender.send(msg.clone().into()).unwrap();
                cpu_sender.send(msg.clone().into()).unwrap();
                cpu_cores_sender.send(msg.clone().into()).unwrap();
                radio_sender.send(msg.clone().into()).unwrap();
            }
        };

        communications::PROFILES_INFO
            .listen(clone!(
                #[strong]
                send_to_all,
                move |profiles_info| {
                    send_to_all(AppSyncUpdate::ProfilesInfo(Arc::from(
                        profiles_info.cloned(),
                    )));
                }
            ))
            .await;

        communications::CONFIG
            .listen(clone!(
                #[strong]
                send_to_all,
                move |config| {
                    send_to_all(AppSyncUpdate::Config(Arc::from(config.cloned())));
                }
            ))
            .await;

        communications::SYSTEM_INFO
            .listen(clone!(
                #[strong]
                send_to_all,
                move |system_info| {
                    send_to_all(AppSyncUpdate::SystemInfo(Arc::from(system_info.cloned())));
                }
            ))
            .await;
    }
}
