use std::convert::identity;
use std::sync::Arc;

use gtk::glib::clone;
use gtk::prelude::*;
use power_daemon::{Config, ProfilesInfo, SystemInfo};
use relm4::prelude::*;
use relm4::Controller;

use super::groups::*;
use super::Header;
use crate::communications;

#[derive(Debug)]
pub enum AppInput {
    ApplySettings,
}

#[derive(Debug, Clone)]
pub enum AppSyncUpdate {
    ProfilesInfo(Arc<Option<ProfilesInfo>>),
    SystemInfo(Arc<Option<SystemInfo>>),
    Config(Arc<Option<Config>>),
}

pub struct App {
    header: Controller<Header>,
    cpu_group: Controller<CPUGroup>,
    settings_group_stack: gtk::Stack,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for App {
    type Input = AppInput;
    type Output = AppSyncUpdate;

    type Init = ();

    view! {
        gtk::ApplicationWindow {
            set_titlebar: Some(model.header.widget()),
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

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let cpu_group = CPUGroup::builder()
            .launch(())
            .forward(sender.input_sender(), identity);

        let settings_group_stack = gtk::Stack::new();
        settings_group_stack.add_titled(
            &gtk::ScrolledWindow::builder()
                .child(cpu_group.widget())
                .build(),
            Some("CPU"),
            "CPU",
        );

        let model = App {
            header: Header::builder()
                .launch(())
                .forward(sender.input_sender(), identity),

            cpu_group,
            settings_group_stack,
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
                    }
                }
            }
        }
    }
}

impl App {
    async fn setup_sync_listeners(&self) {
        let send_to_all = {
            let header_sender = self.header.sender().clone();
            let cpu_sender = self.cpu_group.sender().clone();
            move |msg: AppSyncUpdate| {
                header_sender.send(msg.clone().into()).unwrap();
                cpu_sender.send(msg.clone().into()).unwrap();
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
