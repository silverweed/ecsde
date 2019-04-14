use crate::core::common;
use crate::core::common::transform::C_Transform2D;
use crate::core::common::vector::Vec2f;
use crate::core::env::Env_Info;
use crate::core::input;
use crate::core::time;
use crate::ecs::components as comp;
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use crate::gfx;
use crate::resources::resources::{tex_path, Resources};
use sdl2::rect::Rect;
use std::time::Duration;

pub struct Gameplay_System {
    entity_manager: Entity_Manager,
    entities: Vec<Entity>,
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            entity_manager: Entity_Manager::new(),
            entities: vec![],
        }
    }

    pub fn init(&mut self, env: &Env_Info, rsrc: &mut Resources) -> common::Maybe_Error {
        self.register_all_components();

        self.init_demo_sprites(env, rsrc);

        Ok(())
    }

    pub fn update(
        &mut self,
        dt: &Duration,
        actions: &input::Action_List,
        camera: &mut C_Transform2D,
    ) {
        self.update_animated_sprites(dt);
        self.move_camera(dt, actions, camera);
        self.update_controllables(dt);
    }

    pub fn get_renderable_entities(&self) -> Vec<(&comp::C_Renderable, &C_Transform2D)> {
        self.entity_manager
            .get_component_tuple::<comp::C_Renderable, C_Transform2D>()
            .collect()
    }

    fn register_all_components(&mut self) {
        let em = &mut self.entity_manager;

        em.register_component::<C_Transform2D>();
        em.register_component::<comp::C_Renderable>();
        em.register_component::<comp::C_Controllable>();
    }

    fn move_camera(
        &mut self,
        dt: &Duration,
        actions: &input::Action_List,
        camera: &mut C_Transform2D,
    ) {
        use crate::core::common::direction::Direction;
        use input::Action;

        let mut movement = Vec2f::new(0.0, 0.0);
        if actions.has_action(&Action::Move(Direction::Left)) {
            movement.x -= 1.0;
        }
        if actions.has_action(&Action::Move(Direction::Right)) {
            movement.x += 1.0;
        }
        if actions.has_action(&Action::Move(Direction::Up)) {
            movement.y -= 1.0;
        }
        if actions.has_action(&Action::Move(Direction::Down)) {
            movement.y += 1.0;
        }

        let t = movement * time::to_secs_frac(dt) * 300f32;
        camera.translate(t.x, t.y);

        for a in actions.iter() {
            match a {
                Action::Zoom(factor) => {
                    camera.add_scale(0.01 * *factor as f32, 0.01 * *factor as f32)
                }
                _ => (),
            }
        }
    }

    fn update_animated_sprites(&mut self, dt: &Duration) {
        let mut anim_sprites = self
            .entity_manager
            .get_components_mut::<comp::C_Renderable>();
        gfx::animation_system::update_animated_sprites(&dt, &mut anim_sprites);
    }

    fn update_controllables(&mut self, dt: &Duration) {
        let em = &mut self.entity_manager;

        let controllables: Vec<&Entity> = self
            .entities
            .iter()
            .filter(|&&e| {
                em.has_component::<C_Transform2D>(e) && em.has_component::<comp::C_Controllable>(e)
            })
            .collect();
        for &ctrl in controllables {
            let speed = em
                .get_component::<comp::C_Controllable>(ctrl)
                .unwrap()
                .speed;
            //let velocity = movement * speed * time::to_secs_frac(dt);
            let tr = em.get_component_mut::<C_Transform2D>(ctrl).unwrap();
            tr.rotate(cgmath::Rad(3.0 * time::to_secs_frac(dt)));
        }
    }

    // #DEMO
    fn init_demo_sprites(&mut self, env: &Env_Info, rsrc: &mut Resources) {
        let em = &mut self.entity_manager;
        let yv = em.new_entity();
        self.entities.push(yv);
        {
            let tr = em.add_component::<C_Transform2D>(yv);
            tr.set_position(300.0, 200.0);
            tr.set_scale(3.0, 3.0);
        }
        {
            let mut rend = em.add_component::<comp::C_Renderable>(yv);
            rend.texture = rsrc.load_texture(&tex_path(&env, "yv.png"));
            rend.rect = Rect::new(0, 0, 148, 125);
        }
        {
            let mut ctrl = em.add_component::<comp::C_Controllable>(yv);
            ctrl.speed = 300.0;
        }

        let plant = em.new_entity();
        self.entities.push(plant);
        {
            let tr = em.add_component::<C_Transform2D>(plant);
            tr.set_position(400.0, 500.0);
        }
        {
            let mut rend = em.add_component::<comp::C_Renderable>(plant);
            rend.texture = rsrc.load_texture(&tex_path(&env, "plant.png"));
            rend.rect = Rect::new(0, 0, 96, 96);
            rend.n_frames = 4;
            rend.frame_time = 0.1;
        }
    }
}
