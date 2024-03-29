use inle_cfg::{Cfg_Var, Config};
use inle_input::axes::Virtual_Axes;
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Default)]
pub struct Input_Config {
    pub joy_deadzone: Cfg_Var<f32>,
}

pub fn get_movement_from_input(
    axes: &Virtual_Axes,
    input_cfg: Input_Config,
    cfg: &Config,
) -> Vec2f {
    let deadzone = input_cfg.joy_deadzone.read(cfg).abs();
    let x = axes.get_axis_value(sid!("horizontal"));
    let y = axes.get_axis_value(sid!("vertical"));
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
