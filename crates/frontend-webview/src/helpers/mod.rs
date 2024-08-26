pub mod components;
pub mod coroutine_extensions;
pub mod toggleable_components;
pub mod toggleable_types;

#[derive(PartialEq, Clone)]
#[allow(unused)]
pub enum TooltipDirection {
    AtRight,
    AtLeft,
    AtTop,
    AtBottom,
}

impl TooltipDirection {
    pub fn to_class_name(&self) -> String {
        String::from(match self {
            TooltipDirection::AtRight => "tooltip tooltip-at-right",
            TooltipDirection::AtLeft => "tooltip tooltip-at-left",
            TooltipDirection::AtTop => "tooltip tooltip-at-top",
            TooltipDirection::AtBottom => "tooltip tooltip-at-bottom",
        })
    }
}
