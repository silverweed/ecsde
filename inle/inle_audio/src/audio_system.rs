use super::sound;
use crate::res::{Audio_Resources, Sound_Handle};

pub struct Audio_System_Config {
    pub max_concurrent_sounds: usize,
}

pub struct Audio_System {
    max_concurrent_sounds: usize,
    sounds_playing: Vec<sound::Sound>,
}

impl Audio_System {
    pub fn new(cfg: &Audio_System_Config) -> Self {
        Audio_System {
            max_concurrent_sounds: cfg.max_concurrent_sounds,
            sounds_playing: vec![],
        }
    }

    pub fn update(&mut self) {
        let mut i = 0;
        while i < self.sounds_playing.len() {
            let sound = &self.sounds_playing[i];
            if !sound::sound_playing(sound) {
                self.sounds_playing.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn play_sound(&mut self, rsrc: &Audio_Resources, sound_handle: Sound_Handle) {
        if self.sounds_playing.len() == self.max_concurrent_sounds {
            // @Incomplete: this is not necessarily what we'll want
            self.sounds_playing.swap_remove(0);
        }
        let sound_buf = rsrc.get_sound_buffer(sound_handle);
        let mut sound = sound::create_sound_with_buffer(sound_buf);
        sound::play_sound(&mut sound);
        self.sounds_playing.push(sound);
    }

    pub fn n_sounds_playing(&self) -> usize {
        self.sounds_playing.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::res::sound_path;

    #[test]
    fn max_concurrent_sounds() {
        let env = inle_core::env::Env_Info::gather().unwrap();
        let mut ares = Audio_Resources::new();
        let snd_handle = ares.load_sound(&sound_path(&env, "coin.ogg"));

        {
            let max_conc_sounds = 5;
            let mut a_sys = Audio_System::new(&Audio_System_Config {
                max_concurrent_sounds: max_conc_sounds,
            });
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
}
