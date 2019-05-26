#[cfg(feature = "gfx_sdl")]
#[derive(Copy, Clone, Debug)]
pub struct Rect(sdl2::rect::Rect);

#[cfg(feature = "gfx_sfml")]
#[derive(Copy, Clone, Debug)]
pub struct Rect(sfml::graphics::IntRect);

// The most boring facade ever written.
impl Rect {
    #[cfg(feature = "gfx_sdl")]
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect(sdl2::rect::Rect::new(x, y, w, h))
    }

    #[cfg(feature = "gfx_sfml")]
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect(sfml::graphics::IntRect::new(x, y, w as i32, h as i32))
    }

    #[cfg(feature = "gfx_sdl")]
    pub fn width(&self) -> u32 {
        self.0.width
    }

    #[cfg(feature = "gfx_sfml")]
    pub fn width(&self) -> u32 {
        self.0.width as u32
    }

    #[cfg(feature = "gfx_sdl")]
    pub fn height(&self) -> u32 {
        self.0.height
    }

    #[cfg(feature = "gfx_sfml")]
    pub fn height(&self) -> u32 {
        self.0.height as u32
    }

    #[cfg(feature = "gfx_sdl")]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[cfg(feature = "gfx_sfml")]
    pub fn x(&self) -> i32 {
        self.0.left
    }

    #[cfg(feature = "gfx_sdl")]
    pub fn y(&self) -> i32 {
        self.0.y
    }

    #[cfg(feature = "gfx_sfml")]
    pub fn y(&self) -> i32 {
        self.0.top
    }

    #[cfg(feature = "gfx_sdl")]
    pub fn set_x(&mut self, x: i32) {
        self.0.x = x;
    }

    #[cfg(feature = "gfx_sfml")]
    pub fn set_x(&mut self, x: i32) {
        self.0.left = x;
    }

    #[cfg(feature = "gfx_sdl")]
    pub fn set_y(&mut self, y: i32) {
        self.0.t = y;
    }

    #[cfg(feature = "gfx_sfml")]
    pub fn set_y(&mut self, y: i32) {
        self.0.top = y;
    }
}
