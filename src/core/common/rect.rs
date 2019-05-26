#[cfg(feature = "use-sdl")]
#[derive(Copy, Clone, Debug)]
pub struct Rect(sdl2::rect::Rect);

#[cfg(feature = "use-sfml")]
#[derive(Copy, Clone, Debug)]
pub struct Rect(sfml::graphics::IntRect);

// The most boring facade ever written.
impl Rect {
    #[cfg(feature = "use-sdl")]
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect(sdl2::rect::Rect::new(x, y, w, h))
    }

    #[cfg(feature = "use-sfml")]
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect(sfml::graphics::IntRect::new(x, y, w as i32, h as i32))
    }

    #[cfg(feature = "use-sdl")]
    pub fn width(&self) -> u32 {
        self.0.width()
    }

    #[cfg(feature = "use-sfml")]
    pub fn width(&self) -> u32 {
        self.0.width as u32
    }

    #[cfg(feature = "use-sdl")]
    pub fn height(&self) -> u32 {
        self.0.height()
    }

    #[cfg(feature = "use-sfml")]
    pub fn height(&self) -> u32 {
        self.0.height as u32
    }

    #[cfg(feature = "use-sdl")]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[cfg(feature = "use-sfml")]
    pub fn x(&self) -> i32 {
        self.0.left
    }

    #[cfg(feature = "use-sdl")]
    pub fn y(&self) -> i32 {
        self.0.y
    }

    #[cfg(feature = "use-sfml")]
    pub fn y(&self) -> i32 {
        self.0.top
    }

    #[cfg(feature = "use-sdl")]
    pub fn set_x(&mut self, x: i32) {
        self.0.x = x;
    }

    #[cfg(feature = "use-sfml")]
    pub fn set_x(&mut self, x: i32) {
        self.0.left = x;
    }

    #[cfg(feature = "use-sdl")]
    pub fn set_y(&mut self, y: i32) {
        self.0.t = y;
    }

    #[cfg(feature = "use-sfml")]
    pub fn set_y(&mut self, y: i32) {
        self.0.top = y;
    }
}

impl std::ops::Deref for Rect {
    #[cfg(feature = "use-sdl")]
    type Target = sdl2::rect::Rect;

    #[cfg(feature = "use-sfml")]
    type Target = sfml::graphics::IntRect;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Rect {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
