use crate::render_window::Render_Window_Handle;
use inle_core::rand::{Default_Rng, Precomputed_Rand_Pool};
use inle_math::angle::{self, Angle};
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_resources::gfx::Texture_Handle;
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
    pub fn par_iter_mut(
        &mut self,
    ) -> impl rayon::iter::ParallelIterator<Item = ((&mut Transform2D, &mut Vec2f), &mut Duration)>
    {
        self.transforms
            .par_iter_mut()
            .zip_eq(self.velocities.par_iter_mut())
            .zip_eq(self.remaining_life.par_iter_mut())
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
    pub initial_rotation: Range<Angle>,
    pub initial_scale: Range<f32>,
    pub spread: Angle,
    pub acceleration: f32,
    pub texture: Texture_Handle,
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

pub fn update_particles(particles: &mut Particles, dt: &Duration) {
    trace!("update_particles");

    // Update lifetime and remove expired ones
    //{
    //let mut n_removed = 0;
    //let len = particles.remaining_life.len();
    //let head = particles.remaining_life.as_ptr();
    //for (i, rem_life) in particles.remaining_life.iter_mut().enumerate() {
    //if rem_life.checked_sub(*dt).is_none() {
    //if i < len - 1 {
    //let last_elem = unsafe { *head.add(len - n_removed - 1) };
    //*rem_life = last_elem;
    //let last_pos = particles.positions[len - n_removed - 1];
    //particles.positions[i] = last_pos;
    //let last_vel = particles.velocities[len - n_removed - 1];
    //particles.velocities[i] = last_vel;
    //}
    //n_removed += 1;
    //}
    //}
    //particles.positions.truncate(len - n_removed);
    //particles.velocities.truncate(len - n_removed);
    //particles.remaining_life.truncate(len - n_removed);
    //}

    let precomp_rng = &particles.precomp_rng;
    let props = &particles.props;
    let iter = particles
        .transforms
        .par_iter_mut()
        .zip_eq(particles.velocities.par_iter_mut())
        .zip_eq(particles.remaining_life.par_iter_mut());
    iter.for_each(|((transform, velocity), rem_life)| {
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
    });
}

pub fn render_particles(
    particles: &Particles,
    window: &mut Render_Window_Handle,
    camera: &Transform2D,
) {
    trace!("render_particles");

    use inle_common::colors;
    use inle_math::rect::{Rect, Rectf};
    // @Temporary
    for transf in &particles.transforms {
        let rect = Rect::new(-1.0, -1.0, 2.0, 2.0);
        let transform = particles.transform.combine(transf);
        crate::render::render_rect_ws(window, rect, colors::RED, &transform, camera);
    }
}

fn random_pos_in(shape: &Emission_Shape, rng: &Precomputed_Rand_Pool) -> Vec2f {
    match shape {
        &Emission_Shape::Point => Vec2f::default(),
        &Emission_Shape::Circle { radius } => {
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

#[derive(Default)]
pub struct Particle_Manager {
    active_particles: Vec<Particles>,
}

impl Particle_Manager {
    pub fn add_particles(
        &mut self,
        props: &Particle_Props,
        rng: &mut Default_Rng,
    ) -> Particles_Handle {
        self.active_particles.push(create_particles(props, rng));
        assert!(self.active_particles.len() < std::isize::MAX as usize);
        Particles_Handle(self.active_particles.len() as isize - 1)
    }

    pub fn get_particles_mut(&mut self, handle: Particles_Handle) -> &mut Particles {
        assert!(handle.is_valid());
        &mut self.active_particles[handle.0 as usize]
    }

    pub fn update(&mut self, dt: &Duration) {
        self.active_particles.par_iter_mut().for_each(|particles| {
            update_particles(particles, dt);
        });
    }

    pub fn render(&self, window: &mut Render_Window_Handle, camera: &Transform2D) {
        for particles in &self.active_particles {
            render_particles(particles, window, camera);
        }
    }
}
