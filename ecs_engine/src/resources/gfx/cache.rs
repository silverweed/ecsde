use crate::gfx::render::{Font, Texture};
use crate::resources::loaders;

define_file_loader!(Texture, Texture_Loader, Texture_Cache);
define_file_loader!(Font, Font_Loader, Font_Cache);
