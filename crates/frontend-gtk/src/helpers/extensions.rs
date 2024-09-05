use gtk::prelude::*;
use relm4::prelude::*;

pub trait StringListExt {
    fn to_vec(&self) -> Vec<String>;
}

impl StringListExt for gtk::StringList {
    fn to_vec(&self) -> Vec<String> {
        let mut ret = Vec::new();
        for i in 0..self.n_items() {
            ret.push(self.string(i).unwrap().to_string());
        }
        ret
    }
}
