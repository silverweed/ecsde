use super::element::{Debug_Element, Draw_Args, Update_Args, Update_Res};
use super::frame_scroller::Debug_Frame_Scroller;
use super::graph;
use super::log::Debug_Log;
use super::log_window;
use super::overlay;
use inle_alloc::temp;
use inle_cfg::{Cfg_Var, Config};
use inle_common::stringid::String_Id;
use inle_gfx::render_window::Render_Window_Handle;
use inle_gfx::res::{Font_Handle, Gfx_Resources};
use inle_input::input_state::Input_State;
use std::any::type_name;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub struct Debug_Ui_System_Config {
    pub ui_scale: Cfg_Var<f32>,
    pub target_win_size: (u32, u32),
    pub font_name: Cfg_Var<String>,
    pub font_size: Cfg_Var<u32>,
}

impl Default for Debug_Ui_System_Config {
    fn default() -> Self {
        Self {
            ui_scale: Cfg_Var::new_from_val(1.),
            font_size: Cfg_Var::new_from_val(10),
            target_win_size: (800, 600),
            font_name: Cfg_Var::default(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum Active_State {
    Active,
    Inactive,
}

#[derive(Default)]
struct Debug_Element_Container<T> {
    pub actives: Vec<T>,
    pub inactives: Vec<T>,
    pub all: HashMap<String_Id, (Active_State, usize)>,
}

impl<T> Debug_Element_Container<T> {
    fn new() -> Self {
        Self {
            actives: vec![],
            inactives: vec![],
            all: HashMap::new(),
        }
    }

    fn get_debug_element(&mut self, id: String_Id) -> &mut T {
        match self
            .all
            .get(&id)
            .unwrap_or_else(|| fatal!("Tried to get inexisting {} {}", type_name::<T>(), id))
        {
            (Active_State::Active, idx) => &mut self.actives[*idx],
            (Active_State::Inactive, idx) => &mut self.inactives[*idx],
        }
    }

    fn is_enabled(&self, id: String_Id) -> bool {
        match self
            .all
            .get(&id)
            .unwrap_or_else(|| fatal!("Tried to get inexisting {} {}", type_name::<T>(), id))
        {
            (Active_State::Active, _) => true,
            (Active_State::Inactive, _) => false,
        }
    }

    fn set_enabled(&mut self, id: String_Id, enabled: bool) {
        let (old_idx, new_idx, old_state, new_state, idx_to_patch) =
            match self.all.get(&id).unwrap_or_else(|| {
                fatal!(
                    "Tried to set_enabled inexisting {} {}",
                    type_name::<T>(),
                    id
                )
            }) {
                (Active_State::Active, idx) => {
                    if enabled {
                        return;
                    }
                    debug_assert!(*idx < self.actives.len());
                    let elem = self.actives.swap_remove(*idx);
                    self.inactives.push(elem);
                    (
                        *idx,
                        self.inactives.len() - 1,
                        Active_State::Active,
                        Active_State::Inactive,
                        if *idx < self.actives.len() {
                            Some(self.actives.len())
                        } else {
                            None
                        },
                    )
                }
                (Active_State::Inactive, idx) => {
                    if !enabled {
                        return;
                    }
                    debug_assert!(*idx < self.inactives.len());
                    let elem = self.inactives.swap_remove(*idx);
                    self.actives.push(elem);
                    (
                        *idx,
                        self.actives.len() - 1,
                        Active_State::Inactive,
                        Active_State::Active,
                        if *idx < self.inactives.len() {
                            Some(self.inactives.len())
                        } else {
                            None
                        },
                    )
                }
            };

        self.all.insert(id, (new_state, new_idx));

        // Patch the index of the element moved with swap_remove
        if let Some(idx_to_patch) = idx_to_patch {
            let entry = self
                .all
                .iter_mut()
                .find(|(_, (state, idx))| *state == old_state && *idx == idx_to_patch)
                .unwrap();
            *entry.1 = (old_state, old_idx);
            debug_assert!(
                old_idx
                    < if old_state == Active_State::Inactive {
                        self.inactives.len()
                    } else {
                        self.actives.len()
                    }
            );
        }
    }
}

#[derive(Default)]
pub struct Debug_Ui_System {
    overlays: Debug_Element_Container<overlay::Debug_Overlay>,
    graphs: Debug_Element_Container<graph::Debug_Graph_View>,
    log_windows: Debug_Element_Container<log_window::Log_Window>,
    font: Font_Handle,

    pub frame_scroller: Debug_Frame_Scroller,
    pub cfg: Debug_Ui_System_Config,
}

macro_rules! add_debug_elem {
    ($type: ty, $cfg_type: ty, $container: ident, $create_fn: ident, $get_fn: ident, $enable_fn: ident, $is_enabled_fn: ident) => {
        pub fn $create_fn(&mut self, id: String_Id, config: &$cfg_type) -> Option<&mut $type> {
            let elem = <$type>::new(config);
            insert_debug_element(id, &mut self.$container, elem)
        }

        pub fn $get_fn(&mut self, id: String_Id) -> &mut $type {
            self.$container.get_debug_element(id)
        }

        pub fn $enable_fn(&mut self, id: String_Id, enabled: bool) {
            self.$container.set_enabled(id, enabled);
        }

        pub fn $is_enabled_fn(&self, id: String_Id) -> bool {
            self.$container.is_enabled(id)
        }
    };
}

macro_rules! update_and_draw_elems {
    ($self: expr, $container: ident, $dt: expr, $window: expr, $input_state: expr,
     $gres: expr, $config: expr, $frame_alloc: expr) => {
        let mut to_disable = vec![];
        for (i, elem) in $self.$container.actives.iter_mut().enumerate() {
            let res = elem.update(Update_Args {
                dt: $dt,
                window: $window,
                input_state: $input_state,
                config: $config,
                gres: $gres,
            });

            if res == Update_Res::Disable_Self {
                for (id, (_, idx)) in &$self.$container.all {
                    if *idx == i {
                        to_disable.push(*id);
                        break;
                    }
                }
            } else {
                elem.draw(Draw_Args {
                    window: $window,
                    gres: $gres,
                    input_state: $input_state,
                    frame_alloc: $frame_alloc,
                    config: $config,
                });
            }
        }

        for id in to_disable {
            $self.$container.set_enabled(id, false);
        }
    };
}

impl Debug_Ui_System {
    add_debug_elem!(
        overlay::Debug_Overlay,
        overlay::Debug_Overlay_Config,
        overlays,
        create_overlay,
        get_overlay,
        set_overlay_enabled,
        is_overlay_enabled
    );

    add_debug_elem!(
        graph::Debug_Graph_View,
        graph::Debug_Graph_View_Config,
        graphs,
        create_graph,
        get_graph,
        set_graph_enabled,
        is_graph_enabled
    );

    add_debug_elem!(
        log_window::Log_Window,
        log_window::Log_Window_Config,
        log_windows,
        create_log_window,
        get_log_window,
        set_log_window_enabled,
        is_log_window_enabled
    );

    #[inline(always)]
    pub fn get_font(&self) -> Font_Handle {
        self.font
    }

    // Note: we have getter/setter for the font because we may want to make the font
    // dynamically changeable, in which case this function should also set the font
    // for all child debug elements.
    #[inline]
    pub fn set_font(&mut self, font: Font_Handle) {
        self.font = font;
    }

    pub fn update_and_draw(
        &mut self,
        dt: &Duration,
        window: &mut Render_Window_Handle,
        gres: &mut Gfx_Resources,
        input_state: &Input_State,
        log: &Debug_Log,
        config: &Config,
        frame_alloc: &mut temp::Temp_Allocator,
    ) {
        trace!("debug_ui::update_and_draw");

        update_and_draw_elems!(
            self,
            log_windows,
            dt,
            window,
            input_state,
            gres,
            config,
            frame_alloc
        );
        update_and_draw_elems!(
            self,
            graphs,
            dt,
            window,
            input_state,
            gres,
            config,
            frame_alloc
        );
        update_and_draw_elems!(
            self,
            overlays,
            dt,
            window,
            input_state,
            gres,
            config,
            frame_alloc
        );

        self.frame_scroller.update(window, log, input_state);
        if !self.frame_scroller.hidden {
            self.frame_scroller.draw(window, gres, log, config);
        }
    }
}

fn insert_debug_element<T: Debug_Element>(
    id: String_Id,
    container: &mut Debug_Element_Container<T>,
    element: T,
) -> Option<&mut T> {
    match container.all.entry(id) {
        Entry::Occupied(_) => {
            lwarn!(
                "{} '{}' already exists: won't overwrite.",
                type_name::<T>(),
                id
            );
            None
        }
        Entry::Vacant(v) => {
            container.actives.push(element);
            let idx = container.actives.len() - 1;
            v.insert((Active_State::Active, idx));
            Some(&mut container.actives[idx])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::Debug_Overlay_Config;

    #[test]
    fn debug_ui_set_enabled() {
        let mut debug_ui = Debug_Ui_System::default();
        debug_ui.create_overlay(sid!("test"), &Debug_Overlay_Config::default());

        assert!(debug_ui.is_overlay_enabled(sid!("test")));

        debug_ui.set_overlay_enabled(sid!("test"), false);
        assert!(!debug_ui.is_overlay_enabled(sid!("test")));

        debug_ui.create_overlay(sid!("foo"), &Debug_Overlay_Config::default());
        assert!(debug_ui.is_overlay_enabled(sid!("foo")));
        assert!(!debug_ui.is_overlay_enabled(sid!("test")));

        debug_ui.set_overlay_enabled(sid!("foo"), false);
        assert!(!debug_ui.is_overlay_enabled(sid!("foo")));
        assert!(!debug_ui.is_overlay_enabled(sid!("test")));

        debug_ui.set_overlay_enabled(sid!("test"), true);
        assert!(!debug_ui.is_overlay_enabled(sid!("foo")));
        assert!(debug_ui.is_overlay_enabled(sid!("test")));

        debug_ui.set_overlay_enabled(sid!("test"), false);
        assert!(!debug_ui.is_overlay_enabled(sid!("foo")));
        assert!(!debug_ui.is_overlay_enabled(sid!("test")));

        debug_ui.set_overlay_enabled(sid!("foo"), true);
        debug_ui.set_overlay_enabled(sid!("test"), true);
        assert!(debug_ui.is_overlay_enabled(sid!("foo")));
        assert!(debug_ui.is_overlay_enabled(sid!("test")));
    }
}
