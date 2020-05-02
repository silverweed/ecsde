use ecs_engine::alloc::temp::*;
use ecs_engine::collisions::collider::Collider;
use ecs_engine::common::colors::Color;
use ecs_engine::common::vector::Vec2f;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::gfx::render::{self, Image};
use ecs_engine::resources::gfx::{Gfx_Resources, Texture_Handle};
use std::collections::HashMap;

#[derive(Copy, Clone, Default)]
pub struct C_Texture_Collider {
    pub texture: Texture_Handle,
}

struct Collision_Info {
    pub entity_nonpixel: Entity,
    pub entity_pixel: Entity,
    pub normal: Vec2f,
}

#[derive(Default)]
pub struct Pixel_Collision_System {
    images: HashMap<Texture_Handle, Image>,
    collided_entities: Vec<Collision_Info>,
}

#[inline(always)]
fn is_solid(pixel: &Color) -> bool {
    pixel.a > 0
}

fn approx_normal(image: &Image, x: u32, y: u32, step: i32) -> Vec2f {
    trace!("approx_normal");

    let mut avg = Vec2f::default();
    let size = render::get_image_size(image);
    let x_range = (x as i32 - step).max(0) as u32..(x as i32 + step).min(size.0 as i32) as u32;
    let y_range = (y as i32 - step).max(0) as u32..(y as i32 + step).min(size.0 as i32) as u32;
    for (i, x) in x_range.enumerate() {
        for (j, y) in y_range.clone().enumerate() {
            if is_solid(&render::get_pixel(image, x, y)) {
                avg.x -= (i as i32 - step) as f32;
                avg.y -= (j as i32 - step) as f32;
            }
        }
    }

    avg.normalized()
}

impl Pixel_Collision_System {
    pub fn update(
        &mut self,
        world: &mut Ecs_World,
        gres: &Gfx_Resources,
        temp_alloc: &mut Temp_Allocator,
    ) {
        trace!("pixel_collision::update");

        let mut colliding_positions = excl_temp_array(temp_alloc);

        foreach_entity!(world, +Collider, +C_Spatial2D, |entity| {
            let s = world.get_component::<C_Spatial2D>(entity).unwrap();
            let c = world.get_component::<Collider>(entity).unwrap();
            if !c.is_static {
                colliding_positions.push((entity, s.transform.position(), c.shape.extent()));
            }
        });

        self.collided_entities.clear();

        foreach_entity!(world, +C_Texture_Collider, |entity| {
            let tex_cld = world.get_component::<C_Texture_Collider>(entity).unwrap().texture;
            let img = self.images.entry(tex_cld).or_insert_with(||
                render::copy_texture_to_image(gres.get_texture(tex_cld))
            );

            let (iw, ih) = render::get_image_size(img);
            let iw = iw as i32;
            let ih = ih as i32;

            for (e, pos, extent) in &colliding_positions {
                trace!("pixel_collision::narrow");

                let x_range = ((pos.x - extent.x * 0.5).floor() as i32 + iw / 2).max(0) ..
                                ((pos.x + extent.x * 0.5).floor() as i32 + iw / 2).min(iw);
                let y_range = ((pos.y - extent.y * 0.5).floor() as i32 + ih / 2).max(0) ..
                                ((pos.y + extent.y * 0.5).floor() as i32 + ih / 2).min(ih);
                for x in x_range {
                    for y in y_range.clone() {
                        debug_assert!(x >= 0 && x < iw);
                        debug_assert!(y >= 0 && y < ih);
                        let pixel = render::get_pixel(img, x as u32, y as u32);
                        if pixel.a > 0 {
                            self.collided_entities.push(Collision_Info {
                                entity_nonpixel: *e,
                                entity_pixel: entity,
                                normal: approx_normal(img, x as u32, y as u32, 6),
                            });
                        }
                    }
                }
            }
        });

        for info in &self.collided_entities {
            world
                .get_component_mut::<Collider>(info.entity_nonpixel)
                .unwrap()
                .colliding_with = Some(info.entity_pixel);
            let spat = world
                .get_component_mut::<C_Spatial2D>(info.entity_nonpixel)
                .unwrap();
            let speed = spat.velocity.magnitude();
            spat.velocity = speed * info.normal;
        }
    }
}
