use inle_cfg::{Cfg_Var, Config};
use inle_common::stringid::String_Id;
use inle_math::vector::Vec2f;
use inle_input::axes::Virtual_Axes;

#[derive(Copy, Clone, Default)]
pub struct Input_Config {
    pub joy_deadzone: Cfg_Var<f32>,
}

pub fn get_movement_from_input(
    axes: &Virtual_Axes,
    input_cfg: Input_Config,
    cfg: &Config,
) -> Vec2f {
    // @Speed @WaitForStable: compute these StringIds at compile-time
    let deadzone = input_cfg.joy_deadzone.read(cfg).abs();
    let x = axes.get_axis_value(String_Id::from("horizontal"));
    let y = axes.get_axis_value(String_Id::from("vertical"));
    Vec2f::new(
        if x.abs() > deadzone { x } else { 0.0 },
        if y.abs() > deadzone { y } else { 0.0 },
    )
}

pub fn get_normalized_movement_from_input(
    axes: &Virtual_Axes,
    input_cfg: Input_Config,
    cfg: &Config,
) -> Vec2f {
    let m = get_movement_from_input(axes, input_cfg, cfg);
    m.normalized_or_zero()
}
