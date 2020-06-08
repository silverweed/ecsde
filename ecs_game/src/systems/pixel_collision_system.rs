use ecs_engine::alloc::temp::*;
use ecs_engine::collisions::collider::{C_Phys_Data, Collider};
use ecs_engine::collisions::layers::{Collision_Layer, Collision_Matrix};
use ecs_engine::common::colors::Color;
use ecs_engine::common::math::clamp;
use ecs_engine::common::rect::Rect;
use ecs_engine::common::shapes::Circle;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::common::vector::{Vec2f, Vec2i};
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::gfx::render::{self, Image};
use ecs_engine::resources::gfx::{Gfx_Resources, Texture_Handle};
use std::collections::HashMap;

#[derive(Copy, Clone, Default)]
pub struct C_Texture_Collider {
    pub texture: Texture_Handle,
    pub layer: Collision_Layer,
}

struct Collision_Info {
    pub entity_nonpixel: Entity,
    pub entity_pixel: Entity,
    pub normal: Vec2f,
    pub restitution: f32,
}

#[derive(Default)]
pub struct Pixel_Collision_System {
    images: HashMap<Texture_Handle, Image>,
    collided_entities: Vec<Collision_Info>,
}

#[inline(always)]
fn is_solid(pixel: Color) -> bool {
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
            if is_solid(render::get_image_pixel(image, x, y)) {
                avg.x -= (i as i32 - step) as f32;
                avg.y -= (j as i32 - step) as f32;
            }
        }
    }

    avg.normalized_or_zero()
}

impl Pixel_Collision_System {
    pub fn change_pixels_rect(
        &mut self,
        texture: Texture_Handle,
        rect: &Rect<u32>,
        new_val: Color,
        gres: &mut Gfx_Resources,
    ) {
        let img = self.images.get_mut(&texture).unwrap_or_else(|| {
            fatal!(
                "Tried to update_pixels for inexisting texture {:?}",
                texture
            )
        });

        for x in rect.x..rect.x + rect.width {
            for y in rect.y..rect.y + rect.height {
                render::set_image_pixel(img, x, y, new_val);
            }
        }

        let pixels = vec![new_val; rect.width as usize * rect.height as usize];
        let texture = gres.get_texture_mut(texture);
        debug_assert!(
            render::get_texture_size(&texture).0 >= rect.width + rect.x
                && render::get_texture_size(&texture).1 >= rect.height + rect.y
        );
        render::update_texture_pixels(texture, rect, &pixels);
    }

    pub fn change_pixels_circle(
        &mut self,
        texture: Texture_Handle,
        circle: Circle,
        new_val: Color,
        gres: &mut Gfx_Resources,
    ) {
        trace!("change_pixels_circle");

        let img = self.images.get_mut(&texture).unwrap_or_else(|| {
            fatal!(
                "Tried to update_pixels for inexisting texture {:?}",
                texture
            )
        });

        let r = circle.radius as i32;
        let Vec2i { x: cx, y: cy } = Vec2i::from(circle.center);
        let size = render::get_image_size(img);
        let mut pixels = vec![new_val; 4 * (r * r) as usize];

        for (iy, y) in ((-r).max(-cy)..r.min(size.1 as i32 - cy)).enumerate() {
            for (ix, x) in ((-r).max(-cx)..r.min(size.0 as i32 - cx)).enumerate() {
                debug_assert!(cx + x >= 0 && cx + x < size.0 as i32);
                debug_assert!(cy + y >= 0 && cy + y < size.1 as i32);
                let img_x = (cx + x) as u32;
                let img_y = (cy + y) as u32;
                if x * x + y * y <= r * r {
                    render::set_image_pixel(img, img_x, img_y, new_val);
                    pixels[(iy as i32 * 2 * r) as usize + ix] = new_val;
                } else {
                    pixels[(iy as i32 * 2 * r) as usize + ix] =
                        render::get_image_pixel(img, img_x, img_y);
                }
            }
        }

        let texture = gres.get_texture_mut(texture);
        debug_assert!(
            render::get_texture_size(&texture).0 >= (r + cx) as u32
                && render::get_texture_size(&texture).1 >= (r + cy) as u32
        );
        let rect = Rect::new(
            clamp(cx - r, 0, size.0 as i32) as u32,
            clamp(cy - r, 0, size.1 as i32) as u32,
            2 * r as u32,
            2 * r as u32,
        );
        render::update_texture_pixels(texture, &rect, &pixels);
    }

    pub fn update(
        &mut self,
        world: &mut Ecs_World,
        gres: &Gfx_Resources,
        collision_matrix: &Collision_Matrix,
        temp_alloc: &mut Temp_Allocator,
    ) {
        trace!("pixel_collision::update");

        let mut colliding_positions = excl_temp_array(temp_alloc);

        struct Potential_Colliding_Entity_Info {
            pub entity: Entity,
            pub transform: Transform2D,
            pub velocity: Vec2f,
            pub extent: Vec2f,
            pub layer: Collision_Layer,
        }

        foreach_entity!(world, +Collider, +C_Spatial2D, |entity| {
            let s = world.get_component::<C_Spatial2D>(entity).unwrap();
            let c = world.get_component::<Collider>(entity).unwrap();
            if !c.is_static {
                colliding_positions.push(Potential_Colliding_Entity_Info {
                    entity,
                    transform: s.transform,
                    extent: c.shape.extent(),
                    velocity: s.velocity,
                    layer: c.layer,
                });
            }
        });

        self.collided_entities.clear();

        foreach_entity!(world, +C_Texture_Collider, +C_Spatial2D, |entity| {
            let tex_transform = world.get_component::<C_Spatial2D>(entity).unwrap().transform;
            let tex_cld = world.get_component::<C_Texture_Collider>(entity).unwrap();
            let img = self.images.entry(tex_cld.texture).or_insert_with(||
                render::copy_texture_to_image(gres.get_texture(tex_cld.texture))
            );

            let (iw, ih) = render::get_image_size(img);
            let (iw, ih) = (iw as i32, ih as i32);

            let tex_inv_transform = tex_transform.inverse();

            for info in &colliding_positions {
                trace!("pixel_collision::narrow");

                let Potential_Colliding_Entity_Info {
                    entity: e,
                    transform,
                    extent,
                    velocity,
                    layer,
                } = info;

                if !collision_matrix.layers_collide(tex_cld.layer, *layer) {
                    continue;
                }

                // Convert entity in local space
                let colliding_local_transform = tex_inv_transform.combine(&transform);
                let pos = colliding_local_transform.position();
                let extent = *extent * colliding_local_transform.scale();
                // @TODO: consider rotation

                let x_range = ((pos.x - extent.x * 0.5).floor() as i32 + iw / 2).max(0) ..
                                ((pos.x + extent.x * 0.5).floor() as i32 + iw / 2).min(iw);
                let y_range = ((pos.y - extent.y * 0.5).floor() as i32 + ih / 2).max(0) ..
                                ((pos.y + extent.y * 0.5).floor() as i32 + ih / 2).min(ih);

                // @Speed: we may cycle in an inward spiral in the hope of using less iteration to
                // find the first colliding pixel. Or even only cycle the border.
                'outer:
                for x in x_range {
                    for y in y_range.clone() {
                        debug_assert!(x >= 0 && x < iw);
                        debug_assert!(y >= 0 && y < ih);
                        let dir_to_pixel = v2!((x - iw / 2) as f32, (y - ih / 2) as f32) - pos;
                        if dir_to_pixel.dot(*velocity) >= 0. {
                            let pixel = render::get_image_pixel(img, x as u32, y as u32);
                            if pixel.a > 0 {
                                self.collided_entities.push(Collision_Info {
                                    entity_nonpixel: *e,
                                    entity_pixel: entity,
                                    normal: approx_normal(img, x as u32, y as u32, 6),
                                    restitution: world.get_component::<C_Phys_Data>(*e)
                                                    .map(|pd| pd.restitution).unwrap_or(1.),
                                });
                                break 'outer;
                            }
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
            spat.velocity = speed * info.normal * info.restitution;
        }
    }
}
