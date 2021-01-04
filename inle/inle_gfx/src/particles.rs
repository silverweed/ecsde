use crate::render::Shader;
use crate::render::{self, Primitive_Type};
use crate::render_window::Render_Window_Handle;
use crate::vbuf_holder::Vertex_Buffer_Holder;
use inle_alloc::temp;
use inle_cfg::{self, Cfg_Var};
use inle_common::colors;
use inle_core::env::Env_Info;
use inle_core::rand::{Default_Rng, Precomputed_Rand_Pool};
use inle_math::angle::{self, Angle};
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_resources::gfx::{
    shader_path, Gfx_Resources, Shader_Cache, Shader_Handle, Texture_Handle,
};
use rayon::prelude::*;
use std::ops::Range;
use std::time::Duration;

pub struct Particles {
    pub transform: Transform2D,
    pub transforms: Vec<Transform2D>,
    pub velocities: Vec<Vec2f>,
    pub remaining_life: Vec<Duration>,
    pub props: Particle_Props,

    precomp_rng: Precomputed_Rand_Pool,
}

impl Particles {
    fn par_iter_mut(
        &mut self,
    ) -> impl rayon::iter::ParallelIterator<Item = ((&mut Transform2D, &mut Vec2f), &mut Duration)>
    {
        self.transforms
            .par_iter_mut()
            .zip_eq(self.velocities.par_iter_mut())
            .zip_eq(self.remaining_life.par_iter_mut())
    }

    pub fn count(&self) -> usize {
        self.transforms.len()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Emission_Shape {
    Point,
    Circle { radius: f32 },
}

#[derive(Clone, Debug)]
pub struct Particle_Props {
    pub n_particles: usize,
    pub lifetime: Range<Duration>,
    pub emission_shape: Emission_Shape,
    pub initial_speed: Range<f32>,
    // @Incomplete: currently unused
    pub initial_rotation: Range<Angle>,
    // @Incomplete: currently unused
    pub initial_scale: Range<f32>,
    pub spread: Angle,
    pub acceleration: f32,
    pub texture: Texture_Handle,
    // @Incomplete: this is unused in the shader!
    pub color: colors::Color,
}

impl Default for Particle_Props {
    fn default() -> Self {
        Self {
            n_particles: 0,
            lifetime: Duration::default()..Duration::default(),
            emission_shape: Emission_Shape::Point,
            initial_speed: 0.0..0.0,
            initial_rotation: angle::rad(0.0)..angle::rad(0.0),
            initial_scale: 1.0..1.0,
            spread: Angle::default(),
            texture: None,
            acceleration: 0.0,
            color: colors::WHITE,
        }
    }
}

pub fn create_particles(props: &Particle_Props, rng: &mut Default_Rng) -> Particles {
    let n_particles = props.n_particles;
    let mut particles = Particles {
        transform: Transform2D::default(),
        props: props.clone(),
        transforms: Vec::with_capacity(n_particles),
        velocities: Vec::with_capacity(n_particles),
        remaining_life: Vec::with_capacity(n_particles),
        precomp_rng: Precomputed_Rand_Pool::default(),
    };

    particles
        .transforms
        .resize(n_particles, Transform2D::default());
    particles.velocities.resize(n_particles, Vec2f::default());
    particles
        .remaining_life
        .resize(n_particles, Duration::default());

    let precomp_rng = Precomputed_Rand_Pool::with_size(rng, (4.5 * n_particles as f32) as usize);

    particles
        .par_iter_mut()
        .for_each(|((position, velocity), remaining_life)| {
            let (pos, vel, life) = init_particle(props, &precomp_rng);
            *position = pos;
            *velocity = vel;
            *remaining_life = life;
        });

    particles.precomp_rng = precomp_rng;

    particles
}

fn init_particle(
    props: &Particle_Props,
    precomp_rng: &Precomputed_Rand_Pool,
) -> (Transform2D, Vec2f, Duration) {
    let pos = random_pos_in(&props.emission_shape, &precomp_rng);
    let rot = angle::rad(precomp_rng.rand_range(
        props.initial_rotation.start.as_rad(),
        props.initial_rotation.end.as_rad(),
    ));
    let scale = precomp_rng.rand_range(props.initial_scale.start, props.initial_scale.end);
    let speed = precomp_rng.rand_range(props.initial_speed.start, props.initial_speed.end);
    let vel = speed
        * v2!(1.0, 0.0).rotated(angle::rad(
            precomp_rng.rand_range(-0.5 * props.spread.as_rad(), 0.5 * props.spread.as_rad()),
        ));
    let life = Duration::from_secs_f32(precomp_rng.rand_range(
        props.lifetime.start.as_secs_f32(),
        props.lifetime.end.as_secs_f32(),
    ));
    (
        Transform2D::from_pos_rot_scale(pos, rot, v2!(scale, scale)),
        vel,
        life,
    )
}

pub fn update_particles(particles: &mut Particles, dt: &Duration, chunk_size: usize) {
    trace!("update_particles");

    // @Speed: find out the best chunk size.
    // It looks like if the particles are few, updating them in singlethread is much faster than
    // multithread. However, if they are a lot, multithread wins.

    let precomp_rng = &particles.precomp_rng;
    let props = &particles.props;
    let iter = particles
        .transforms
        .par_chunks_mut(chunk_size)
        .zip_eq(particles.velocities.par_chunks_mut(chunk_size))
        .zip_eq(particles.remaining_life.par_chunks_mut(chunk_size));
    iter.for_each(|((transforms, velocities), rem_lifes)| {
        for i in 0..transforms.len() {
            let transform = &mut transforms[i];
            let velocity = &mut velocities[i];
            let rem_life = &mut rem_lifes[i];
            if let Some(life) = rem_life.checked_sub(*dt) {
                let dt = dt.as_secs_f32();
                let old_pos = transform.position();
                let old_vel = *velocity;
                let old_speed = old_vel.magnitude();
                *rem_life = life;
                transform.set_position_v(old_pos + old_vel * dt);
                *velocity = old_vel.normalized_or_zero() * (old_speed + props.acceleration * dt);
            } else {
                let (transf, vel, life) = init_particle(props, precomp_rng);
                *transform = transf;
                *velocity = vel;
                *rem_life = life;
            }
        }
    });
}

pub fn render_particles(
    particles: &Particles,
    window: &mut Render_Window_Handle,
    gres: &Gfx_Resources,
    shader: &mut Shader,
    _camera: &Transform2D,
    vbuf: &mut Vertex_Buffer_Holder,
    frame_alloc: &mut temp::Temp_Allocator,
) {
    trace!("render_particles");

    let texture = particles
        .props
        .texture
        .map(|tex| gres.get_texture(Some(tex)));

    let mut vertices = temp::excl_temp_array(frame_alloc);
    for &transf in &particles.transforms {
        // @Incomplete or @Redundant: passing the vertex color is useless right now
        vertices.push(render::new_vertex(
            transf.position(),
            particles.props.color,
            Vec2f::default(),
        ));
    }

    vbuf.update(&mut vertices, particles.transforms.len() as u32);

    if let Some(texture) = texture {
        render::set_uniform(shader, c_str!("tex"), texture);
    }
    // @Incomplete: set mvp uniform
    render::render_vbuf_with_shader(window, &vbuf.vbuf, shader);
}

fn random_pos_in(shape: &Emission_Shape, rng: &Precomputed_Rand_Pool) -> Vec2f {
    match *shape {
        Emission_Shape::Point => Vec2f::default(),
        Emission_Shape::Circle { radius } => {
            let r = rng.rand_range(0.0, radius);
            let a = rng.rand_range(0.0, angle::TAU);
            Vec2f::from_polar(r, a)
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Particles_Handle(isize);

impl Particles_Handle {
    pub fn is_valid(self) -> bool {
        self.0 >= 0
    }
}

pub struct Particle_Manager {
    particle_shader: Shader_Handle,
    active_particles: Vec<Particles>,
    // This is in a separate array because we want Particles to be processable in parallel
    active_particles_vbufs: Vec<Vertex_Buffer_Holder>,
    coarse_chunk_size: Cfg_Var<i32>,
    narrow_chunk_size: Cfg_Var<i32>,
}

impl Particle_Manager {
    pub fn new(shader_cache: &mut Shader_Cache, env: &Env_Info, cfg: &inle_cfg::Config) -> Self {
        let particle_shader = shader_cache.load_shader_with_geom(&shader_path(env, "particles"));
        Self {
            particle_shader,
            active_particles: vec![],
            active_particles_vbufs: vec![],
            coarse_chunk_size: Cfg_Var::new("engine/particles/update_coarse_chunk_size", cfg),
            narrow_chunk_size: Cfg_Var::new("engine/particles/update_narrow_chunk_size", cfg),
        }
    }

    pub fn add_particles(
        &mut self,
        window: &mut Render_Window_Handle,
        props: &Particle_Props,
        rng: &mut Default_Rng,
    ) -> Particles_Handle {
        debug_assert!(props.n_particles < std::u32::MAX as usize);
        self.active_particles.push(create_particles(props, rng));
        self.active_particles_vbufs
            .push(Vertex_Buffer_Holder::with_initial_vertex_count(
                window,
                props.n_particles as u32,
                Primitive_Type::Points,
                #[cfg(debug_assertions)]
                format!("{:?}", props),
            ));
        debug_assert_eq!(
            self.active_particles.len(),
            self.active_particles_vbufs.len()
        );

        debug_assert!(self.active_particles.len() < std::isize::MAX as usize);
        Particles_Handle(self.active_particles.len() as isize - 1)
    }

    pub fn get_particles_mut(&mut self, handle: Particles_Handle) -> &mut Particles {
        debug_assert!(handle.is_valid());
        &mut self.active_particles[handle.0 as usize]
    }

    pub fn update(&mut self, dt: &Duration, cfg: &inle_cfg::Config) {
        let coarse_chunk_size = self.coarse_chunk_size.read(cfg) as usize;
        let narrow_chunk_size = self.narrow_chunk_size.read(cfg) as usize;
        self.active_particles
            .par_chunks_mut(coarse_chunk_size)
            .for_each(|chunk| {
                for particles in chunk {
                    update_particles(particles, dt, narrow_chunk_size);
                }
            });
    }

    pub fn render(
        &mut self,
        window: &mut Render_Window_Handle,
        gres: &Gfx_Resources,
        shader_cache: &mut Shader_Cache,
        camera: &Transform2D,
        frame_alloc: &mut temp::Temp_Allocator,
    ) {
        if !render::shaders_are_available() {
            return;
        }

        let shader = shader_cache.get_shader_mut(self.particle_shader);
        let (ww, wh) = inle_win::window::get_window_real_size(window);
        render::set_uniform(shader, c_str!("camera_scale"), 1.0 / camera.scale().x);

        for (particles, vbuf) in self
            .active_particles
            .iter()
            .zip(self.active_particles_vbufs.iter_mut())
        {
            // @Incomplete: we must handle the case where the texture is unset.
            if let Some(tex) = particles
                .props
                .texture
                .map(|tex| gres.get_texture(Some(tex)))
            {
                let (tw, th) = render::get_texture_size(tex);
                render::set_uniform(
                    shader,
                    c_str!("tex_size_normalized"),
                    v2!(tw as f32 / ww as f32, th as f32 / wh as f32),
                );
            }

            // @Speed: we *may* want to run this in parallel and see if it gives benefits.
            // We could prealloc an Excl_Temp_Array (we know the exact size we need) and split it
            // as needed for every active Particles we need to render.
            // Then we could fill those arrays in parallel and only after copy the memory to the
            // vbufs (sequentially).
            // This doesn't look very promising though, as the big chunk of work is probably the
            // memory transfer, so I doubt this would give a noticeable improvement (in fact it
            // may make things worse).
            render_particles(particles, window, gres, shader, camera, vbuf, frame_alloc);
        }
    }
}
