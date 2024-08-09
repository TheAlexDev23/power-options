use std::convert::identity;
use std::sync::Arc;

use gtk::glib::clone;
use gtk::prelude::*;
use power_daemon::{Config, ProfilesInfo, SystemInfo};
use relm4::prelude::*;
use relm4::Controller;

use super::Header;
use crate::communications::{self};

#[derive(Debug)]
pub enum AppInput {
    ChangeSettingsGroup(u32),
}

#[derive(Debug, Clone)]
pub enum AppSyncOutput {
    ProfilesInfo(Arc<Option<ProfilesInfo>>),
    SystemInfo(Arc<Option<SystemInfo>>),
    Config(Arc<Option<Config>>),
}

pub struct App {
    header: Controller<Header>,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for App {
    type Input = AppInput;
    type Output = AppSyncOutput;

    type Init = ();

    view! {
        gtk::ApplicationWindow {
            set_titlebar: Some(model.header.widget()),
            gtk::Label {
                set_label: "hi",
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = App {
            header: Header::builder()
                .launch(init)
                .forward(sender.input_sender(), identity),
        };

        model.setup_sync_listeners().await;

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            AppInput::ChangeSettingsGroup(_) => todo!(),
        }
    }
}

impl App {
    async fn setup_sync_listeners(&self) {
        let send_to_all = {
            let header_sender = self.header.sender().clone();
            move |msg: AppSyncOutput| {
                header_sender.send(msg.into()).unwrap();
            }
        };

        communications::PROFILES_INFO
            .listen(clone!(
                #[strong]
                send_to_all,
                move |profiles_info| {
                    send_to_all(AppSyncOutput::ProfilesInfo(Arc::from(
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
                    send_to_all(AppSyncOutput::Config(Arc::from(config.cloned())));
                }
            ))
            .await;

        communications::SYSTEM_INFO
            .listen(clone!(
                #[strong]
                send_to_all,
                move |system_info| {
                    send_to_all(AppSyncOutput::SystemInfo(Arc::from(system_info.cloned())));
                }
            ))
            .await;
    }
}
