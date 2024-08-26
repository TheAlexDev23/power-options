macro_rules! binding {
    ($name:ident, $obj_name:literal, $ty:ty, $mod:ident) => {
        relm4::gtk::glib::wrapper! {
            #[doc = "A data binding storing a value of type [`"]
            #[doc = stringify!($ty)]
            #[doc = "`]"]
            pub struct $name(ObjectSubclass<$mod::$name>);
        }

        impl $name {
            #[doc = "Create a new [`"]
            #[doc = stringify!($name)]
            #[doc = "`]."]
            pub fn new<T: Into<$ty>>(value: T) -> Self {
                let this: Self = relm4::gtk::glib::Object::new();
                this.set_value(value.into());
                this
            }
        }

        impl Default for $name {
            fn default() -> Self {
                relm4::gtk::glib::Object::new()
            }
        }

        impl relm4::binding::Binding for $name {
            type Target = $ty;

            fn get(&self) -> Self::Target {
                self.value()
            }

            fn set(&self, value: Self::Target) {
                self.set_value(value)
            }
        }

        #[allow(missing_docs)]
        mod $mod {
            use std::cell::RefCell;

            use relm4::gtk::glib::prelude::*;
            use relm4::gtk::glib::{ParamSpec, Properties, Value};
            use relm4::gtk::subclass::prelude::ObjectImpl;
            use relm4::gtk::{
                glib,
                subclass::prelude::{DerivedObjectProperties, ObjectSubclass},
            };

            #[derive(Default, Properties, Debug)]
            #[properties(wrapper_type = super::$name)]
            /// Inner type of the data binding.
            pub struct $name {
                #[property(get, set)]
                /// The primary value.
                value: RefCell<$ty>,
            }

            impl ObjectImpl for $name {
                fn properties() -> &'static [ParamSpec] {
                    Self::derived_properties()
                }
                fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
                    self.derived_set_property(id, value, pspec)
                }
                fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
                    self.derived_property(id, pspec)
                }
            }

            #[relm4::gtk::glib::object_subclass]
            impl ObjectSubclass for $name {
                const NAME: &'static str = $obj_name;
                type Type = super::$name;
            }
        }
    };
}

binding!(
    StringListBinding,
    "StringListBinding",
    relm4::gtk::StringList,
    imp_stringlist
);

binding!(
    AdjustmentBinding,
    "AdjustmentBinding",
    relm4::gtk::Adjustment,
    imp_adjusment
);
