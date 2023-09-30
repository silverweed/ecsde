use inle_common::stringid::String_Id;
use inle_gfx::res::{Gfx_Resources, Texture_Handle};
use inle_gfx::sprites::Sprite;
use inle_math::rect::{Rect, Recti};
use inle_math::vector::Vec2i;
use smallvec::SmallVec;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[derive(Clone)]
struct Animation {
    pub name: String_Id,

    // starting (col, row). Note that all frames of an animation
    // must be consecutive. The animation can wrap to the next column(s), but it cannot wrap
    // from the end to the start of the spritesheet.
    pub start: (u16, u16),
    // ending (row, col)
    pub end: (u16, u16),

    pub frame_duration: Duration,
    pub frame_time: Duration,
}

#[derive(Clone)]
pub struct Anim_Sprite {
    pub sprite: Sprite,

    pub(self) cur_anim: String_Id,
    pub(self) animations: SmallVec<[Animation; 1]>,

    pub(self) n_frame_cols: u16,
    pub(self) n_frame_rows: u16,
    pub(self) cur_frame_col: u16,
    pub(self) cur_frame_row: u16,
}

impl Deref for Anim_Sprite {
    type Target = Sprite;

    fn deref(&self) -> &Self::Target {
        &self.sprite
    }
}

impl DerefMut for Anim_Sprite {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sprite
    }
}

impl From<Sprite> for Anim_Sprite {
    fn from(sprite: Sprite) -> Self {
        Self::from_sprite(sprite, (1, 1), Duration::default())
    }
}

impl Anim_Sprite {
    pub fn new(
        gres: &Gfx_Resources,
        tex: Texture_Handle,
        frame_rect: Recti,
        frame_duration: Duration,
    ) -> Self {
        let sprite = Sprite::new(gres, tex);
        Self::from_sprite_with_rect(sprite, frame_rect, frame_duration)
    }

    pub fn from_tex_path_rect(
        gres: &mut Gfx_Resources,
        path: &std::path::Path,
        frame_rect: Recti,
        frame_duration: Duration,
    ) -> Self {
        let sprite = Sprite::from_tex_path(gres, path);
        Self::from_sprite_with_rect(sprite, frame_rect, frame_duration)
    }

    pub fn from_tex_path(
        gres: &mut Gfx_Resources,
        path: &std::path::Path,
        n_frames: (u16, u16),
        frame_duration: Duration,
    ) -> Self {
        let sprite = Sprite::from_tex_path(gres, path);
        Self::from_sprite(sprite, n_frames, frame_duration)
    }

    pub fn from_sprite_with_rect(
        mut sprite: Sprite,
        frame_rect: Recti,
        frame_duration: Duration,
    ) -> Self {
        let spritesheet_rect = sprite.rect;
        sprite.rect = frame_rect;

        let n_frame_rows = spritesheet_rect.height / frame_rect.height;
        let n_frame_cols = spritesheet_rect.width / frame_rect.width;

        let mut sprite = Self {
            sprite,
            n_frame_rows: n_frame_rows.try_into().unwrap(),
            n_frame_cols: n_frame_cols.try_into().unwrap(),
            cur_anim: sid!("default"),
            animations: smallvec![],
            cur_frame_col: 0,
            cur_frame_row: 0,
        };
        sprite.add_animation(
            sid!("default"),
            (0, 0),
            (n_frame_cols as u16 - 1, n_frame_rows as u16 - 1),
            frame_duration,
        );
        sprite
    }

    pub fn from_sprite_with_size(mut sprite: Sprite, size: Vec2i) -> Self {
        let spritesheet_rect = sprite.rect;
        sprite.rect = Rect::new(0, 0, size.x, size.y);

        let n_frame_rows = spritesheet_rect.height / size.y;
        let n_frame_cols = spritesheet_rect.width / size.x;

        Self {
            sprite,
            n_frame_rows: n_frame_rows.try_into().unwrap(),
            n_frame_cols: n_frame_cols.try_into().unwrap(),
            cur_anim: sid!(""),
            animations: smallvec![],
            cur_frame_col: 0,
            cur_frame_row: 0,
        }
    }

    pub fn from_sprite(
        sprite: Sprite,
        (n_cols, n_rows): (u16, u16),
        frame_duration: Duration,
    ) -> Self {
        let spritesheet_rect = sprite.rect;
        let frame_rect = Rect::new(
            0,
            0,
            spritesheet_rect.width / n_cols as i32,
            spritesheet_rect.height / n_rows as i32,
        );
        Self::from_sprite_with_rect(sprite, frame_rect, frame_duration)
    }

    pub fn add_animation(
        &mut self,
        name: String_Id,
        start: (u16, u16),
        end: (u16, u16),
        frame_duration: Duration,
    ) {
        self.animations.push(Animation {
            name,
            start,
            end,
            frame_time: Duration::default(),
            frame_duration,
        });
    }

    pub fn play(&mut self, name: String_Id) {
        if let Some(anim) = self.animations.iter().find(|a| a.name == name) {
            self.cur_anim = name;
            self.cur_frame_col = anim.start.0;
            self.cur_frame_row = anim.start.1;
        } else {
            lerr!("Failed to play animation {:?}: it doesn't exist!", name);
        }
    }
}

pub fn update_anim_sprites<'a>(
    dt: Duration,
    anim_sprites: impl IntoIterator<Item = &'a mut Anim_Sprite>,
) {
    let dt_secs = dt.as_secs_f32();

    for sprite in anim_sprites {
        if sprite.n_frame_cols * sprite.n_frame_rows <= 1 {
            continue;
        }

        if let Some(anim) = sprite
            .animations
            .iter_mut()
            .find(|a| a.name == sprite.cur_anim)
        {
            if anim.frame_duration == Duration::default() {
                continue;
            }
            anim.frame_time += dt;

            if anim.frame_time >= anim.frame_duration {
                anim.frame_time -= anim.frame_duration;

                sprite.cur_frame_col += 1;

                if sprite.cur_frame_col == sprite.n_frame_cols {
                    sprite.cur_frame_col = 0;
                    sprite.cur_frame_row += 1;
                }
                if sprite.cur_frame_row == anim.end.1 {
                    if sprite.cur_frame_col == anim.end.0 {
                        sprite.cur_frame_row = anim.start.1;
                        sprite.cur_frame_col = anim.start.0;
                    }
                }

                let r = sprite.sprite.rect;

                sprite.sprite.rect = Rect::new(
                    sprite.cur_frame_col as i32 * r.width,
                    sprite.cur_frame_row as i32 * r.height,
                    r.width,
                    r.height,
                );
            }
        }
    }
}

pub fn render_anim_sprite(
    window: &mut inle_gfx::render_window::Render_Window_Handle,
    batches: &mut inle_gfx::render::batcher::Batches,
    sprite: &Anim_Sprite,
) {
    inle_gfx::sprites::render_sprite(window, batches, &sprite.sprite);
}
