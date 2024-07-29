pub mod cpu;
pub mod radio;

use dioxus::prelude::*;

pub type ToggleableString = (Signal<bool>, Signal<String>);
pub type ToggleableInt = (Signal<bool>, Signal<i32>);
pub type ToggleableBool = (Signal<bool>, Signal<bool>);
