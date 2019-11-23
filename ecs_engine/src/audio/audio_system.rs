use crate::resources::audio::{Audio_Resources, Sound, Sound_Handle};

pub struct Audio_System_Config {
    // @Incomplete: currently unused
    pub max_concurrent_sounds: usize,
}

pub struct Audio_System {
    max_concurrent_sounds: usize,
}

impl Audio_System {
    pub fn new(cfg: &Audio_System_Config) -> Audio_System {
        Audio_System {
            max_concurrent_sounds: cfg.max_concurrent_sounds,
        }
    }

    pub fn play_sound(&mut self, rsrc: &Audio_Resources, sound_handle: Sound_Handle) {
        let sound_buf = rsrc.get_sound(sound_handle);
        let mut sound = Sound::with_buffer(&sound_buf);
        sound.play();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::audio::sound_path;
    use crate::test_common;

    //#[test]
    //fn max_concurrent_sounds() {
    //let max_conc_sounds = 5;
    //let mut a_sys = Audio_System::new(&Audio_System_Config {
    //max_concurrent_sounds: max_conc_sounds,
    //});
    //let (_, mut ares, env) = test_common::create_test_resources_and_env();
    //let snd_handle = ares.load_sound(&sound_path(&env, "coin.ogg"));

    //a_sys.play_sound(&ares, snd_handle);
    //assert_eq!(a_sys.n_sounds_playing(), 1);

    //a_sys.play_sound(&ares, snd_handle);
    //a_sys.play_sound(&ares, snd_handle);
    //a_sys.play_sound(&ares, snd_handle);
    //a_sys.play_sound(&ares, snd_handle);
    //a_sys.play_sound(&ares, snd_handle);
    //a_sys.play_sound(&ares, snd_handle);
    //assert_eq!(a_sys.n_sounds_playing(), max_conc_sounds);
    //}
}
