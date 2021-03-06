use super::element::Debug_Element;
use super::fadeout_overlay;
use super::frame_scroller::Debug_Frame_Scroller;
use super::graph;
use super::log::Debug_Log;
use super::overlay;
use inle_alloc::temp;
use inle_common::stringid::String_Id;
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::input_state::Input_State;
use inle_resources::gfx::Gfx_Resources;
use std::any::type_name;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub struct Debug_Ui_System_Config {
    pub ui_scale: f32,
    pub target_win_size: (u32, u32),
    pub font: String,
}

impl Default for Debug_Ui_System_Config {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            target_win_size: (800, 600),
            font: String::default(),
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
    fadeout_overlays: Debug_Element_Container<fadeout_overlay::Fadeout_Debug_Overlay>,
    graphs: Debug_Element_Container<graph::Debug_Graph_View>,
    pub frame_scroller: Debug_Frame_Scroller,
    pub cfg: Debug_Ui_System_Config,
}

macro_rules! add_debug_elem {
    ($type: ty, $cfg_type: ty, $container: ident, $create_fn: ident, $get_fn: ident, $enable_fn: ident) => {
        pub fn $create_fn(&mut self, id: String_Id, config: $cfg_type) -> Option<&mut $type> {
            let elem = <$type>::new(config);
            insert_debug_element(id, &mut self.$container, elem)
        }

        pub fn $get_fn(&mut self, id: String_Id) -> &mut $type {
            self.$container.get_debug_element(id)
        }

        pub fn $enable_fn(&mut self, id: String_Id, enabled: bool) {
            self.$container.set_enabled(id, enabled);
        }
    };
}

impl Debug_Ui_System {
    pub fn new() -> Debug_Ui_System {
        Debug_Ui_System {
            overlays: Debug_Element_Container::new(),
            fadeout_overlays: Debug_Element_Container::new(),
            graphs: Debug_Element_Container::new(),
            frame_scroller: Debug_Frame_Scroller::default(),
            cfg: Debug_Ui_System_Config::default(),
        }
    }

    add_debug_elem!(
        overlay::Debug_Overlay,
        overlay::Debug_Overlay_Config,
        overlays,
        create_overlay,
        get_overlay,
        set_overlay_enabled
    );

    add_debug_elem!(
        fadeout_overlay::Fadeout_Debug_Overlay,
        fadeout_overlay::Fadeout_Debug_Overlay_Config,
        fadeout_overlays,
        create_fadeout_overlay,
        get_fadeout_overlay,
        set_fadeout_overlay_enabled
    );

    add_debug_elem!(
        graph::Debug_Graph_View,
        graph::Debug_Graph_View_Config,
        graphs,
        create_graph,
        get_graph,
        set_graph_enabled
    );

    pub fn update_and_draw(
        &mut self,
        dt: &Duration,
        window: &mut Render_Window_Handle,
        gres: &mut Gfx_Resources,
        input_state: &Input_State,
        log: &Debug_Log,
        frame_alloc: &mut temp::Temp_Allocator,
    ) {
        trace!("debug_ui::update_and_draw");

        for elem in &mut self.graphs.actives {
            elem.update(dt, window, input_state);
            elem.draw(window, gres, frame_alloc);
        }

        for elem in &mut self.overlays.actives {
            elem.update(dt, window, input_state);
            elem.draw(window, gres, frame_alloc);
        }

        for elem in &mut self.fadeout_overlays.actives {
            elem.update(dt, window, input_state);
            elem.draw(window, gres, frame_alloc);
        }

        self.frame_scroller.update(window, log);
        self.frame_scroller.draw(window, gres, log);
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
