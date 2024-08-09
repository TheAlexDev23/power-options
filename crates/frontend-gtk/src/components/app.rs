use std::convert::identity;

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
    // TODO: use rc or arc instead of cloning
    ProfilesInfo(Option<ProfilesInfo>),
    SystemInfo(Option<SystemInfo>),
    Config(Option<Config>),
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
        let header_sender = self.header.sender().clone();

        communications::PROFILES_INFO
            .listen(clone!(
                #[strong]
                header_sender,
                move |profiles_info| {
                    header_sender
                        .send(AppSyncOutput::ProfilesInfo(profiles_info.cloned()).into())
                        .unwrap();
                }
            ))
            .await;

        communications::CONFIG
            .listen(clone!(
                #[strong]
                header_sender,
                move |config| {
                    header_sender
                        .send(AppSyncOutput::Config(config.cloned()).into())
                        .unwrap();
                }
            ))
            .await;

        communications::SYSTEM_INFO
            .listen(clone!(
                #[strong]
                header_sender,
                move |system_info| {
                    header_sender
                        .send(AppSyncOutput::SystemInfo(system_info.cloned()).into())
                        .unwrap();
                }
            ))
            .await;
    }
}
