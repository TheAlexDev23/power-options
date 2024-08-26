use std::{cell::RefCell, rc::Rc, time::Duration};

use gtk::ListStore;
use power_daemon::{CPUCoreSettings, CPUInfo, CoreSetting, Profile};

use adw::prelude::*;
use relm4::prelude::*;

use super::{CPU_EPPS, CPU_GOVERNORS_ACTIVE, CPU_GOVERNORS_PASSIVE};
use crate::{
    communications::{daemon_control, system_info},
    AppInput, AppSyncUpdate, RootRequest,
};

const CPU_IDX: u32 = 0;
const ONLINE_IDX: u32 = 1;
const MIN_IDX: u32 = 2;
const MAX_IDX: u32 = 3;
const EPP_IDX: u32 = 4;
const GOV_IDX: u32 = 5;
const TOTAL_MIN_IDX: u32 = 6;
const TOTAL_MAX_IDX: u32 = 7;

#[derive(Debug, Clone)]
pub enum CPUCoresInput {
    RootRequest(RootRequest),
    OnlineChanged(gtk::TreePath, bool),
    GovChanged(gtk::TreePath, gtk::glib::Value),
    EppChanged(gtk::TreePath, gtk::glib::Value),
    /// if 0 == true, then minimum freq. If 2 is not a valid string that can be
    /// converted to an integer, the value won't be changed.
    FreqLimChanged(bool, gtk::TreePath, String),
    Reset,
}

unsafe impl Send for CPUCoresInput {}

impl From<RootRequest> for CPUCoresInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug)]
pub struct CPUCoresGroup {
    info_obtained: bool,

    has_epp: Rc<RefCell<bool>>,
    mode: Rc<RefCell<u8>>,

    cores: gtk::ListStore,

    available_epps: ListStore,
    available_governors: ListStore,

    active_profile: Option<(usize, Profile)>,

    last_settings: Option<CPUCoreSettings>,
}

impl Default for CPUCoresGroup {
    fn default() -> Self {
        use gtk::glib::Type;
        Self {
            info_obtained: Default::default(),
            has_epp: Default::default(),
            mode: Default::default(),
            cores: gtk::ListStore::new(&[
                Type::STRING,
                Type::BOOL,
                Type::U32,
                Type::U32,
                Type::STRING,
                Type::STRING,
                Type::U32,
                Type::U32,
            ]),
            available_epps: gtk::ListStore::new(&[Type::STRING]),
            available_governors: gtk::ListStore::new(&[Type::STRING]),
            active_profile: Default::default(),
            last_settings: Default::default(),
        }
    }
}

impl CPUCoresGroup {
    fn from_cpu_info(&mut self, info: &CPUInfo) {
        self.cores.clear();
        for core_info in &info.cores {
            let core_id = if let Some(p_core) = core_info.is_performance_core {
                if core_info.online.unwrap_or(true) {
                    if p_core {
                        format!(
                            "P ({} - {})",
                            core_info.physical_core_id, core_info.logical_cpu_id
                        )
                    } else {
                        format!(
                            "E ({} - {})",
                            core_info.physical_core_id, core_info.logical_cpu_id
                        )
                    }
                } else {
                    if p_core {
                        format!("P (n.a - {})", core_info.logical_cpu_id)
                    } else {
                        format!("E (n.a - {})", core_info.logical_cpu_id)
                    }
                }
            } else {
                if core_info.online.unwrap_or(true) {
                    format!(
                        "{} - {}",
                        core_info.physical_core_id, core_info.logical_cpu_id
                    )
                } else {
                    format!("n.a - {}", core_info.logical_cpu_id)
                }
            };

            self.cores.set(
                &self.cores.append(),
                &[
                    (CPU_IDX, &core_id),
                    (ONLINE_IDX, &core_info.online.unwrap_or(true)),
                    (MIN_IDX, &core_info.scaling_min_frequency),
                    (MAX_IDX, &core_info.scaling_max_frequency),
                    (
                        EPP_IDX,
                        &core_info.epp.clone().unwrap_or("default".to_string()),
                    ),
                    (GOV_IDX, &core_info.governor),
                    (TOTAL_MIN_IDX, &core_info.total_min_frequency),
                    (TOTAL_MAX_IDX, &core_info.total_max_frequency),
                ],
            );
        }

        *self.has_epp.borrow_mut() = info.has_epp;
        let active = info.mode.as_ref().unwrap_or(&"passive".to_string()) == "active";
        *self.mode.borrow_mut() = if active { 0 } else { 1 };

        let epps = CPU_EPPS.clone();
        let governors = if active {
            CPU_GOVERNORS_ACTIVE.clone()
        } else {
            CPU_GOVERNORS_PASSIVE.clone()
        };

        fn set_liststore(list_store: &mut ListStore, items: &[&str]) {
            *list_store = gtk::ListStore::new(&[gtk::glib::Type::STRING]);
            for item in items.iter() {
                list_store.set(&list_store.append(), &[(0, item)]);
            }
        }

        set_liststore(&mut self.available_epps, &epps);
        set_liststore(&mut self.available_governors, &governors);
    }

    fn to_core_settings(&self) -> CPUCoreSettings {
        let iter = self.cores.iter_first();
        let mut cores = Vec::new();
        let mut idx = 0;

        if let Some(iter) = iter {
            loop {
                cores.push(CoreSetting {
                    cpu_id: idx,
                    online: self.cores.get::<bool>(&iter, ONLINE_IDX as i32).into(),
                    min_frequency: Some(self.cores.get::<u32>(&iter, MIN_IDX as i32)).into(),
                    max_frequency: Some(self.cores.get::<u32>(&iter, MAX_IDX as i32)).into(),
                    governor: self.cores.get::<String>(&iter, GOV_IDX as i32).into(),
                    epp: if *self.has_epp.borrow() {
                        self.cores.get::<String>(&iter, EPP_IDX as i32).into()
                    } else {
                        None
                    },
                });

                idx += 1;
                if !self.cores.iter_next(&iter) {
                    break;
                }
            }
        }

        CPUCoreSettings {
            cores: cores.into(),
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for CPUCoresGroup {
    type Input = CPUCoresInput;

    type Output = AppInput;

    type Init = ();

    view! {
        gtk::Box {
            set_expand: true,
            set_homogeneous: true,
            if !model.info_obtained || model.active_profile.is_none() {
                gtk::Box {
                    set_align: gtk::Align::Center,
                    gtk::Label::new(Some("Connecting to the daemon...")),
                    gtk::Spinner {
                        set_spinning: true,
                        set_visible: true,
                    }
                }
            } else {
                gtk::TreeView {
                    set_halign: gtk::Align::Center,
                    set_model: Some(&model.cores),
                    append_column= &gtk::TreeViewColumn {
                        set_title: "Id",
                        pack_start[true]: cell_id= &gtk::CellRendererText {
                        },
                        add_attribute: (&cell_id, "text", CPU_IDX as i32),
                    },
                    append_column= &gtk::TreeViewColumn {
                        set_title: "Online",
                        pack_start[true]: cell_online= &gtk::CellRendererToggle {
                            connect_toggled[sender] => move |renderer, path| {
                                sender.input(CPUCoresInput::OnlineChanged(path, !renderer.is_active()))
                            }
                        },
                        add_attribute: (&cell_online, "active", ONLINE_IDX as i32)
                    },
                    append_column= &gtk::TreeViewColumn {
                        set_title: "Min",
                        pack_start[true]: cell_min= &gtk::CellRendererText {
                            set_editable: true,
                            connect_edited[sender] => move |_renderer, path, new_text| {
                                sender.input(CPUCoresInput::FreqLimChanged(true, path, new_text.to_string()))
                            },
                        },
                        add_attribute: (&cell_min, "text", MIN_IDX as i32),
                        set_cell_data_func: (&cell_min, move |_column, cell, tree_model, iter| {
                            cell.set_visible(
                                tree_model.get_value(&iter, ONLINE_IDX as i32).get::<bool>().unwrap()
                            );
                        }),
                    },
                    append_column= &gtk::TreeViewColumn {
                        set_title: "Max",
                        pack_start[true]: cell_max= &gtk::CellRendererText {
                            set_editable: true,
                            connect_edited[sender] => move |_renderer, path, new_text| {
                                sender.input(CPUCoresInput::FreqLimChanged(false, path, new_text.to_string()))
                            },
                        },
                        add_attribute: (&cell_max, "text", MAX_IDX as i32),
                        set_cell_data_func: (&cell_max, move |_column, cell, tree_model, iter| {
                            cell.set_visible(
                                tree_model.get_value(&iter, ONLINE_IDX as i32).get::<bool>().unwrap()
                            );
                        }),
                    },
                    append_column= &gtk::TreeViewColumn {
                        set_title: "EPP",
                        pack_start[true]: cell_epp= &gtk::CellRendererCombo {
                            set_editable: true,
                            set_text_column: 0,
                            set_has_entry: false,
                            #[watch]
                            set_model: Some(&model.available_epps),
                            connect_changed[sender] => move |renderer, path, new_text| {
                                sender.input(CPUCoresInput::EppChanged(path, renderer.model().unwrap().get_value(new_text, 0)))
                            },
                        },
                        add_attribute: (&cell_epp, "text", EPP_IDX as i32),
                        set_cell_data_func: (&cell_epp, {
                            let epp = model.has_epp.clone().clone();
                            let mode = model.mode.clone().clone();
                            move |column, cell, tree_model, iter| {

                            column.set_visible(*epp.borrow());
                            cell.set_visible(
                                !(*mode.borrow() == 0 && tree_model.get_value(&iter, GOV_IDX as i32).get::<String>().unwrap() == "performance")
                                && tree_model.get_value(&iter, ONLINE_IDX as i32).get::<bool>().unwrap()
                            );
                        }}),
                    },
                    append_column= &gtk::TreeViewColumn {
                        set_title: "Governor",
                        pack_start[true]: cell_gov= &gtk::CellRendererCombo {
                            set_editable: true,
                            set_text_column: 0,
                            set_has_entry: false,
                            #[watch]
                            set_model: Some(&model.available_governors),
                            connect_changed[sender] => move |renderer, path, new_text| {
                                sender.input(CPUCoresInput::GovChanged(path, renderer.model().unwrap().get_value(new_text, 0)))
                            },
                        },
                        add_attribute: (&cell_gov, "text", GOV_IDX as i32),
                        set_cell_data_func: (&cell_gov, move |_column, cell, tree_model, iter| {
                            cell.set_visible(
                                tree_model.get_value(&iter, ONLINE_IDX as i32).get::<bool>().unwrap()
                            );
                        }),
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
        let model = CPUCoresGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match &message {
            CPUCoresInput::RootRequest(message) => match message {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();
                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            sender.input(CPUCoresInput::Reset);
                        }
                    }

                    if let AppSyncUpdate::SystemInfo(ref system_info) = message {
                        if let Some(system_info) = system_info.as_ref() {
                            self.info_obtained = true;
                            self.from_cpu_info(&system_info.cpu_info);
                            self.last_settings = Some(self.to_core_settings());
                        }
                    }
                }
                RootRequest::ConfigureSystemInfoSync => fetch_cpu_info_once(),
                RootRequest::Apply => {
                    if !(self.info_obtained && self.active_profile.is_some()) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.cpu_core_settings = self.to_core_settings();

                    let sender = sender.clone();
                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::CPUCores,
                        )
                        .await;

                        sender.input(CPUCoresInput::Reset);
                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            CPUCoresInput::FreqLimChanged(min, path, v) => {
                let min = *min;
                let iter = self.cores.iter(&path).unwrap();

                let total_min = self.cores.get::<u32>(&iter, TOTAL_MIN_IDX as i32);
                let total_max = self.cores.get::<u32>(&iter, TOTAL_MAX_IDX as i32);

                let current_min = self.cores.get::<u32>(&iter, MIN_IDX as i32);
                let current_max = self.cores.get::<u32>(&iter, MAX_IDX as i32);

                if let Ok(mut v) = v.parse::<u32>() {
                    if v < total_min {
                        v = total_min;
                    }
                    if v > total_max {
                        v = total_max;
                    }
                    if min && v > current_max {
                        v = current_max - 1;
                    }
                    if !min && v < current_min {
                        v = current_min + 1;
                    }

                    self.cores
                        .set_value(&iter, if min { MIN_IDX } else { MAX_IDX }, &v.to_value())
                }
            }
            CPUCoresInput::OnlineChanged(path, v) => {
                self.cores
                    .set_value(&self.cores.iter(&path).unwrap(), ONLINE_IDX, &v.to_value())
            }
            CPUCoresInput::EppChanged(path, v) => {
                self.cores
                    .set_value(&self.cores.iter(&path).unwrap(), EPP_IDX, &v)
            }
            CPUCoresInput::GovChanged(path, v) => {
                self.cores
                    .set_value(&self.cores.iter(&path).unwrap(), GOV_IDX, &v)
            }
            CPUCoresInput::Reset => {
                fetch_cpu_info_once();
            }
        }

        match &message {
            CPUCoresInput::RootRequest(_) => {}

            CPUCoresInput::GovChanged(_, _)
            | CPUCoresInput::EppChanged(_, _)
            | CPUCoresInput::FreqLimChanged(_, _, _)
            | CPUCoresInput::OnlineChanged(_, _)
            | CPUCoresInput::Reset => {
                if let Some(ref last_settings) = self.last_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_core_settings(),
                            crate::SettingsGroup::CPUCores,
                        ))
                        .unwrap()
                }
            }
        }
    }
}

fn fetch_cpu_info_once() {
    system_info::set_system_info_sync(
        Duration::from_secs(10),
        system_info::SystemInfoSyncType::CPU,
    );
    system_info::set_system_info_sync(
        Duration::from_secs(10),
        system_info::SystemInfoSyncType::None,
    );
}
