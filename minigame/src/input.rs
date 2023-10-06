use inle_cfg::{Cfg_Var, Config};
use inle_common::stringid::String_Id;
use inle_input::axes::Virtual_Axes;
use inle_math::vector::Vec2f;

#[derive(Default)]
pub struct Input_Config {
    pub joy_deadzone: Cfg_Var<f32>,
}

pub fn get_movement_from_input(
    axes: &Virtual_Axes,
    axes_to_check: [String_Id; 2],
    input_cfg: &Input_Config,
    cfg: &Config,
) -> Vec2f {
    let deadzone = input_cfg.joy_deadzone.read(cfg).abs();
    let x = if axes_to_check[0] != String_Id::INVALID {
        axes.get_axis_value(axes_to_check[0])
    } else {
        0.
    };
    let y = if axes_to_check[1] != String_Id::INVALID {
        axes.get_axis_value(axes_to_check[1])
    } else {
        0.
    };
    Vec2f::new(
        if x.abs() > deadzone { x } else { 0.0 },
        if y.abs() > deadzone { y } else { 0.0 },
    )
}

pub fn get_normalized_movement_from_input(
    axes: &Virtual_Axes,
    axes_to_check: [String_Id; 2],
    input_cfg: &Input_Config,
    cfg: &Config,
) -> Vec2f {
    let m = get_movement_from_input(axes, axes_to_check, input_cfg, cfg);
    m.normalized_or_zero()
}
