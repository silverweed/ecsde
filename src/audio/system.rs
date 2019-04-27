use ears::{AudioController, Sound, SoundData};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

pub struct Audio_System {
    sounds_playing: Vec<Sound>,
    max_concurrent_sounds: usize,
}

impl Audio_System {
    pub fn new(max_concurrent_sounds: usize) -> Audio_System {
        Audio_System {
            sounds_playing: Vec::with_capacity(max_concurrent_sounds),
            max_concurrent_sounds,
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

    pub fn play_sound(&mut self, sound_data: Rc<RefCell<SoundData>>) {
        if self.sounds_playing.len() == self.max_concurrent_sounds {
            // @Incomplete: remove oldest sound
            self.sounds_playing.pop();
        }
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
    use crate::resources;
    use crate::test_common;

    #[test]
    fn max_concurrent_sounds() {
        let max_conc_sounds = 5;
        let mut a_sys = Audio_System::new(max_conc_sounds);
        let (mut rsrc, env) = test_common::create_test_resources_and_env();
        let snd_handle = rsrc.load_sound(&resources::sound_path(&env, "coin.ogg"));
        let snd_data = rsrc.get_sound(snd_handle);

        a_sys.play_sound(snd_data.clone());
        assert_eq!(a_sys.n_sounds_playing(), 1);

        a_sys.play_sound(snd_data.clone());
        a_sys.play_sound(snd_data.clone());
        a_sys.play_sound(snd_data.clone());
        a_sys.play_sound(snd_data.clone());
        a_sys.play_sound(snd_data.clone());
        a_sys.play_sound(snd_data.clone());
        assert_eq!(a_sys.n_sounds_playing(), max_conc_sounds);
    }
}
