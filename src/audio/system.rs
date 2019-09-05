use crate::resources::audio::{Audio_Resources, Sound_Handle};
use ears::{AudioController, Sound};
use std::vec::Vec;

pub struct Audio_System_Config {
    pub max_concurrent_sounds: usize,
}

pub struct Audio_System {
    sounds_playing: Vec<Sound>,
    max_concurrent_sounds: usize,
}

impl Audio_System {
    pub fn new(cfg: &Audio_System_Config) -> Audio_System {
        Audio_System {
            sounds_playing: Vec::with_capacity(cfg.max_concurrent_sounds),
            max_concurrent_sounds: cfg.max_concurrent_sounds,
        }
    }

    pub fn update(&mut self) {
        let mut i = 0;
        while i < self.sounds_playing.len() {
            let sound = &self.sounds_playing[i];
            if !sound.is_playing() {
                self.sounds_playing.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn play_sound(&mut self, rsrc: &Audio_Resources, sound_handle: Sound_Handle) {
        if self.sounds_playing.len() == self.max_concurrent_sounds {
            // @Incomplete: remove oldest sound
            self.sounds_playing.pop();
        }
        let sound_data = rsrc.get_sound(sound_handle);
        let mut sound = Sound::new_with_data(sound_data.clone()).unwrap();
        sound.play();
        self.sounds_playing.push(sound)
    }

    pub fn n_sounds_playing(&self) -> usize {
        self.sounds_playing.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::audio::sound_path;
    use crate::test_common;

    #[test]
    fn max_concurrent_sounds() {
        let max_conc_sounds = 5;
        let mut a_sys = Audio_System::new(&Audio_System_Config {
            max_concurrent_sounds: max_conc_sounds,
        });
        let (_, mut ares, env) = test_common::create_test_resources_and_env();
        let snd_handle = ares.load_sound(&sound_path(&env, "coin.ogg"));

        a_sys.play_sound(&ares, snd_handle);
        assert_eq!(a_sys.n_sounds_playing(), 1);

        a_sys.play_sound(&ares, snd_handle);
        a_sys.play_sound(&ares, snd_handle);
        a_sys.play_sound(&ares, snd_handle);
        a_sys.play_sound(&ares, snd_handle);
        a_sys.play_sound(&ares, snd_handle);
        a_sys.play_sound(&ares, snd_handle);
        assert_eq!(a_sys.n_sounds_playing(), max_conc_sounds);
    }
}
