use adw::prelude::*;
use relm4::{
    binding::{Binding, BoolBinding},
    prelude::*,
    RelmObjectExt,
};

use power_daemon::{WhiteBlackList, WhiteBlackListType};

#[derive(Debug, Clone)]
pub struct WhiteBlackListRendererInit {
    pub list: WhiteBlackList,
    // A vector of columns, column no. 0 will be the one that is used to
    // identify the item in the whiteblacklist
    pub rows: Vec<[String; 2]>,
}

#[derive(Debug, Clone)]
pub enum WhiteBlackListRendererInput {
    Init(WhiteBlackListRendererInit),
    Changed,
    IncludedChanged(gtk::TreePath, bool),
}

#[derive(Debug, Clone)]
pub struct WhiteBlackListRenderer {
    init: Option<WhiteBlackListRendererInit>,
    is_whitelist: BoolBinding,
    model: gtk::ListStore,
}

#[relm4::component(pub)]
impl SimpleComponent for WhiteBlackListRenderer {
    type Input = WhiteBlackListRendererInput;

    type Output = ();

    type Init = ();

    view! {
        adw::PreferencesGroup {
            set_title: "Custom exclusion or inclusion list",
            adw::SwitchRow {
                set_title: "Make custom list of type whitelist",
                set_tooltip_text: Some(&"If enabled the list will act as a whitelist, otherwise as a blacklist."),
                add_binding: (&model.is_whitelist, "active"),
                connect_active_notify  => WhiteBlackListRendererInput::Changed,
            },
            gtk::TreeView {
                #[watch]
                set_model: Some(&model.model),
                append_column=&gtk::TreeViewColumn {
                    set_title: "Included",
                    pack_start[true]: cell_included= &gtk::CellRendererToggle {
                        connect_toggled[sender] => move |renderer, path| {
                            sender.input(WhiteBlackListRendererInput::IncludedChanged(path, !renderer.is_active()))
                        }
                    },
                    add_attribute: (&cell_included, "active", 0)
                },
                append_column=&gtk::TreeViewColumn {
                    set_title: "Id",
                    pack_start[true]: cell_id= &gtk::CellRendererText { },
                    add_attribute: (&cell_id, "text", 1)
                },
                append_column=&gtk::TreeViewColumn {
                    set_title: "Name",
                    pack_start[true]: cell_name= &gtk::CellRendererText { },
                    add_attribute: (&cell_name, "text", 2)
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        use gtk::glib::Type;

        let list_store = gtk::ListStore::new(&[Type::BOOL, Type::STRING, Type::STRING]);

        let model: WhiteBlackListRenderer = WhiteBlackListRenderer {
            is_whitelist: BoolBinding::default(),
            init: None,
            model: list_store,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            WhiteBlackListRendererInput::IncludedChanged(path, v) => {
                self.model
                    .set_value(&self.model.iter(&path).unwrap(), 0, &v.to_value())
            }
            WhiteBlackListRendererInput::Changed => (),
            WhiteBlackListRendererInput::Init(init) => {
                self.model.clear();

                for row in &init.rows {
                    self.model.set(
                        &self.model.append(),
                        &[
                            (0, &init.list.items.iter().any(|i| *i == row[0])),
                            (1, &row[0]),
                            (2, &row[1]),
                        ],
                    );
                }

                *self.is_whitelist.guard() = init.list.list_type == WhiteBlackListType::Whitelist;

                self.init = Some(init);
            }
        }
        sender.output(()).unwrap();
    }
}

impl WhiteBlackListRenderer {
    pub fn to_whiteblacklist(&self) -> WhiteBlackList {
        let iter = self.model.iter_first();
        let mut items = Vec::new();

        if let Some(iter) = iter {
            loop {
                if self.model.get::<bool>(&iter, 0) {
                    items.push(self.model.get::<String>(&iter, 1));
                }

                if !self.model.iter_next(&iter) {
                    break;
                }
            }
        }

        WhiteBlackList {
            items,
            list_type: if self.is_whitelist.value() {
                WhiteBlackListType::Whitelist
            } else {
                WhiteBlackListType::Blacklist
            },
        }
    }
}
