use dioxus::prelude::*;
use power_daemon::WhiteBlackList;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct ToggleableString(pub Signal<bool>, pub Signal<String>);

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct ToggleableInt(pub Signal<bool>, pub Signal<i32>);

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct ToggleableBool(pub Signal<bool>, pub Signal<bool>);

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct ToggleableWhiteBlackList(pub Signal<bool>, pub Signal<WhiteBlackList>);

impl ToggleableString {
    pub fn from(&mut self, other: Option<String>) {
        self.0.set(other.is_some());
        self.1.set(other.unwrap_or_default());
    }
    pub fn from_or(&mut self, other: Option<String>, fallback: String) {
        self.0.set(other.is_some());
        self.1.set(other.unwrap_or(fallback));
    }

    pub fn into_base(&self) -> Option<String> {
        if self.0() {
            Some(self.1())
        } else {
            None
        }
    }
}

impl ToggleableInt {
    pub fn from(&mut self, other: Option<i32>) {
        self.0.set(other.is_some());
        self.1.set(other.unwrap_or_default());
    }
    pub fn from_u32(&mut self, other: Option<u32>) {
        self.0.set(other.is_some());
        self.1.set(other.unwrap_or_default() as i32);
    }
    pub fn from_u8(&mut self, other: Option<u8>) {
        self.0.set(other.is_some());
        self.1.set(other.unwrap_or_default() as i32);
    }

    pub fn into_base(&self) -> Option<i32> {
        if self.0() {
            Some(self.1())
        } else {
            None
        }
    }
    pub fn into_u32(&self) -> Option<u32> {
        if self.0() {
            Some(self.1() as u32)
        } else {
            None
        }
    }
    pub fn into_u8(&self) -> Option<u8> {
        if self.0() {
            Some(self.1() as u8)
        } else {
            None
        }
    }
}

impl ToggleableBool {
    pub fn from(&mut self, other: Option<bool>) {
        self.0.set(other.is_some());
        self.1.set(other.unwrap_or_default());
    }

    pub fn into_base(&self) -> Option<bool> {
        if self.0() {
            Some(self.1())
        } else {
            None
        }
    }
}

impl ToggleableWhiteBlackList {
    pub fn from(&mut self, other: Option<WhiteBlackList>) {
        self.0.set(other.is_some());
        self.1.set(other.unwrap_or_default());
    }

    pub fn into_base(&self) -> Option<WhiteBlackList> {
        if self.0() {
            Some(self.1())
        } else {
            None
        }
    }
}
