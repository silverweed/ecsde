use crate::directions::Square_Direction;
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
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

type Room_Side_Coord_Type = u8;

const MAX_ROOM_SIDE_SIZE: u32 = Room_Side_Coord_Type::MAX as _;
const ROOM_FILE_EXTENSION: &str = "txt";

// @Temporary: should we have a hardcoded enum for all entity types?
// Or something else?
// For now we just save them with their representative character in the room text file.
pub type Entity_Type = char;

#[derive(Default, Debug)]
pub struct Entity_Info {
    pub ent_type: Entity_Type,
    pub tile: Vec2u,
}

#[derive(Default)]
pub struct Room_Pool {
    pub rooms: Vec<Room>,
}

#[derive(Default)]
pub struct Room {
    pub size: Vec2u,

    pub entities: Vec<Entity_Info>,

    // Indexed by Square_Direction
    pub exits: [SmallVec<[Room_Side_Coord_Type; 4]>; 4],
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

pub struct Room_Instantiate_Args<'r, 's, 'a> {
    pub ecs_world: &'a mut Ecs_World,
    pub phys_world: &'a mut Physics_World,
    pub gres: &'a mut Gfx_Resources<'r>,
    pub shader_cache: &'a mut Shader_Cache<'s>,
    pub lights: &'a mut Lights,
    pub env: &'a Env_Info,
    pub cfg: &'a Config,
}

pub fn load_room_pool(directory: &Path) -> Room_Pool {
    let dir = std::fs::read_dir(directory).unwrap_or_else(|err| {
        fatal!(
            "Failed to read room pool from directory {}: {}",
            directory.display(),
            err
        )
    });

    let mut room_pool = Room_Pool::default();

    for entry in dir.flatten().filter(|e| match e.file_type() {
        Ok(ft) => {
            ft.is_file() && e.path().extension() == Some(std::ffi::OsStr::new(ROOM_FILE_EXTENSION))
        }
        _ => false,
    }) {
        match load_room_from_file(&entry.path()) {
            Ok(room) => room_pool.rooms.push(room),
            Err(err) => lerr!("Error loading room: {}", err),
        }
    }

    room_pool
}

#[allow(clippy::collapsible_else_if)]
pub fn load_room_from_file(filepath: &Path) -> Result<Room, Room_Load_Err> {
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
    assert!(
        room.size.y <= MAX_ROOM_SIDE_SIZE,
        "Vertical size of room {} is too big! ({}/{})",
        filepath.display(),
        room.size.y,
        MAX_ROOM_SIDE_SIZE
    );

    // NOTE: since room lines are interleaved with spaces we must take that into account:
    // w w w w w -> line len = 9, room width = 5
    room.size.x = room_lines[0].len() as u32 / 2 + 1;
    assert!(
        room.size.x <= MAX_ROOM_SIDE_SIZE,
        "Horizontal size of room {} is too big! ({}/{})",
        filepath.display(),
        room.size.x,
        MAX_ROOM_SIDE_SIZE
    );

    for (room_lineno, line) in room_lines.iter().enumerate() {
        parse_room_line(
            room_lineno as u32, // y coords go downward!
            line,
            &mut room,
        )?;
    }
    ldebug!("Room size = {:?}, exits = {:?}", room.size, room.exits);

    Ok(room)
}

fn parse_room_line(room_y: u32, line: &str, room: &mut Room) -> Result<(), Room_Load_Err> {
    let is_top_row = room_y == 0;
    let is_bottom_row = room_y == room.size.y - 1;
    for (room_x, entity_chr) in line.chars().step_by(2).enumerate() {
        let coords = v2!(room_x as _, room_y);
        let entity_info = Entity_Info {
            ent_type: entity_chr,
            tile: v2!(room_x as _, room_y as _),
        };
        let is_left_column = room_x == 0;
        let is_right_column = room_x == room.size.x as usize - 1;
        if entity_chr != ' ' {
            room.entities.push(entity_info);
        } else if is_top_row {
            room.exits[Square_Direction::Up as usize].push(room_x as _);
        } else if is_bottom_row {
            room.exits[Square_Direction::Down as usize].push(room_x as _);
        } else if is_left_column {
            room.exits[Square_Direction::Left as usize].push(room_y as _);
        } else if is_right_column {
            room.exits[Square_Direction::Right as usize].push(room_y as _);
        }
    }

    Ok(())
}

pub fn instantiate_room(
    room: &Room,
    room_setup: &Room_Setup,
    instantiate_args: &mut Room_Instantiate_Args,
) {
    for ent_info in &room.entities {
        if load_entity_from_chr(
            ent_info.ent_type,
            ent_info.tile,
            room_setup,
            instantiate_args,
        )
        .is_none()
        {
            lerr!("Unknown entity '{}'", ent_info.ent_type);
        }
    }
}

// Note: Some(INVALID) means 'empty tile', None means error.
fn load_entity_from_chr(
    entity_chr: char,
    grid_coords: Vec2u,
    room_setup: &Room_Setup,
    instantiate_args: &mut Room_Instantiate_Args,
) -> Option<Entity> {
    let transform = Transform2D::from_pos(
        Vec2f::from(grid_coords) * room_setup.tile_size + room_setup.room_offset,
    );
    match entity_chr {
        'w' => Some(entities::create_wall(
            instantiate_args.ecs_world,
            instantiate_args.phys_world,
            instantiate_args.gres,
            instantiate_args.shader_cache,
            instantiate_args.env,
            &transform,
            v2!(room_setup.tile_size, room_setup.tile_size),
            instantiate_args.cfg,
        )),

        't' => {
            instantiate_args
                .lights
                .queue_command(Light_Command::Add_Point_Light(Point_Light {
                    position: transform.position(),
                    radius: 150.,
                    attenuation: 1.0,
                    color: colors::YELLOW,
                    intensity: 1.0,
                }));
            Some(entities::create_torch(
                instantiate_args.ecs_world,
                instantiate_args.phys_world,
                instantiate_args.gres,
                instantiate_args.shader_cache,
                instantiate_args.env,
                instantiate_args.cfg,
                &transform,
            ))
        }

        ' ' => Some(Entity::INVALID),

        _ => None,
    }
}
