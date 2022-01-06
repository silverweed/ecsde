use crate::entities;
use inle_cfg::config::Config;
use inle_common::colors;
use inle_core::env::Env_Info;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_gfx::light::{Light_Command, Lights, Point_Light};
use inle_math::transform::Transform2D;
use inle_math::vector::{Vec2f, Vec2u};
use inle_physics::phys_world::Physics_World;
use inle_resources::gfx::{Gfx_Resources, Shader_Cache};
use std::borrow::Cow;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Default)]
pub struct Room {
    pub size: Vec2u,

    pub entities: Vec<Entity>,
}

#[derive(Debug)]
pub struct Room_Load_Err {
    msg: Cow<'static, str>,
}

impl Display for Room_Load_Err {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Room_Load_Err {}

pub struct Room_Setup {
    /// Offset of the top-left corner
    pub room_offset: Vec2f,
    pub tile_size: f32,
}

pub struct Load_Args<'r, 's, 'a> {
    pub ecs_world: &'a mut Ecs_World,
    pub phys_world: &'a mut Physics_World,
    pub gres: &'a mut Gfx_Resources<'r>,
    pub shader_cache: &'a mut Shader_Cache<'s>,
    pub lights: &'a mut Lights,
    pub env: &'a Env_Info,
    pub cfg: &'a Config,
}

#[allow(clippy::collapsible_else_if)]
pub fn load_room_from_file(
    filepath: &Path,
    room_setup: &Room_Setup,
    mut load_args: Load_Args,
) -> Result<Room, Room_Load_Err> {
    let file = File::open(filepath).map_err(|err| Room_Load_Err {
        msg: Cow::Owned(format!("Error opening file: {}", err)),
    })?;
    let lines = BufReader::new(file).lines().filter_map(|l| l.ok());

    let mut room = Room::default();

    let mut within_begin_and_end = false;
    let mut room_lines = vec![];
    for (lineno, line) in lines.enumerate() {
        if within_begin_and_end {
            if line == "END" {
                break;
            } else {
                room_lines.push(line);
            }
        } else {
            if line == "BEGIN" {
                within_begin_and_end = true;
            } else {
                lwarn!(
                    "{}:{}: ignored line {} before 'BEGIN'.",
                    filepath.display(),
                    lineno,
                    line
                );
            }
        }
    }

    if room_lines.is_empty() {
        return Err(Room_Load_Err {
            msg: Cow::Borrowed("Room is empty."),
        });
    }

    room.size.y = room_lines.len() as _;
    // NOTE: since room lines are interleaved with spaces we must take that into account:
    // w w w w w -> line len = 9, room width = 5
    room.size.x = room_lines[0].len() as u32 / 2 + 1;

    ldebug!("Room size = {:?}", room.size);

    for (room_lineno, line) in room_lines.iter().enumerate() {
        parse_room_line(
            room_lineno as u32, // y coords go downward!
            line,
            &mut room,
            room_setup,
            &mut load_args,
        )?;
    }

    Ok(room)
}

fn parse_room_line(
    room_y: u32,
    line: &str,
    room: &mut Room,
    room_setup: &Room_Setup,
    load_args: &mut Load_Args,
) -> Result<(), Room_Load_Err> {
    for (room_x, entity_chr) in line.chars().step_by(2).enumerate() {
        let coords = v2!(room_x as _, room_y);
        let entity = load_entity_from_chr(entity_chr, coords, room_setup, load_args).ok_or(
            Room_Load_Err {
                msg: Cow::Owned(format!("Unknown entity '{}'", entity_chr)),
            },
        )?;

        if entity != Entity::INVALID {
            room.entities.push(entity);
        }
    }

    Ok(())
}

fn load_entity_from_chr(
    entity_chr: char,
    grid_coords: Vec2u,
    room_setup: &Room_Setup,
    load_args: &mut Load_Args,
) -> Option<Entity> {
    let transform = Transform2D::from_pos(
        Vec2f::from(grid_coords) * room_setup.tile_size + room_setup.room_offset,
    );
    match entity_chr {
        'w' => Some(entities::create_wall(
            load_args.ecs_world,
            load_args.phys_world,
            load_args.gres,
            load_args.shader_cache,
            load_args.env,
            &transform,
            v2!(room_setup.tile_size, room_setup.tile_size),
            load_args.cfg,
        )),

        't' => {
            load_args
                .lights
                .queue_command(Light_Command::Add_Point_Light(Point_Light {
                    position: transform.position(),
                    radius: 150.,
                    attenuation: 1.0,
                    color: colors::YELLOW,
                    intensity: 1.0,
                }));
            Some(entities::create_torch(
                load_args.ecs_world,
                load_args.phys_world,
                load_args.gres,
                load_args.shader_cache,
                load_args.env,
                load_args.cfg,
                &transform,
            ))
        }

        ' ' => Some(Entity::INVALID),

        _ => None,
    }
}
