pub mod components;
pub mod coroutine_extensions;
pub mod toggleable_components;
pub mod toggleable_types;

#[derive(PartialEq, Clone)]
#[allow(unused)]
pub enum TooltipDirection {
    Right,
    Left,
    Top,
    Bottom,
}

impl TooltipDirection {
    pub fn to_class_name(&self) -> String {
        String::from(match self {
            TooltipDirection::Right => "tooltip tooltip-at-right",
            TooltipDirection::Left => "tooltip tooltip-at-left",
            TooltipDirection::Top => "tooltip tooltip-at-top",
            TooltipDirection::Bottom => "tooltip tooltip-at-bottom",
        })
    }
}
