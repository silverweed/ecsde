use super::sound;
use crate::res::{Audio_Resources, Sound_Buffer_Handle};

pub struct Audio_System_Config {
    pub max_concurrent_sounds: usize,
}

type Gen_Type = u32;

struct Playing_Sound {
    pub sound: Option<sound::Sound>,
    pub gen: Gen_Type,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sound_Handle {
    idx: u32,
    gen: Gen_Type,
}

impl Sound_Handle {
    pub const INVALID: Sound_Handle = Sound_Handle {
        idx: u32::MAX,
        gen: Gen_Type::MAX,
    };
}

pub struct Audio_System {
    max_concurrent_sounds: usize,
    sounds_playing: Vec<Playing_Sound>,
}

enum Playing_Sound_Slot_Res {
    Replace { idx: usize, old_gen: Gen_Type },
    Append,
    No_Free_Slot,
}

impl Audio_System {
    pub fn new(cfg: &Audio_System_Config) -> Self {
        assert!(cfg.max_concurrent_sounds < u32::MAX as usize - 1); // -1 since MAX is reserved for invalid handle
        Audio_System {
            max_concurrent_sounds: cfg.max_concurrent_sounds,
            sounds_playing: vec![],
        }
    }

    pub fn update(&mut self) {
        let mut i = 0;
        while i < self.sounds_playing.len() {
            let Playing_Sound { sound, .. } = &mut self.sounds_playing[i];
            if sound.is_some() && !sound::sound_playing(sound.as_ref().unwrap()) {
                // destroy the sound to release the OpenAL handle.
                // We don't increment the generation here as it's already obvious that any handle to this slot
                // are invalid (since sound is now None). We only increment the generation when we place a new
                // sound in this slot.
                sound.take();
            } else {
                i += 1;
            }
        }
    }

    pub fn play_sound(
        &mut self,
        rsrc: &Audio_Resources,
        sound_buf_handle: Sound_Buffer_Handle,
    ) -> Sound_Handle {
        let sound_buf = rsrc.get_sound_buffer(sound_buf_handle);
        let mut sound = sound::create_sound_with_buffer(sound_buf);

        sound::play_sound(&mut sound);

        match self.get_first_free_sound_slot() {
            Playing_Sound_Slot_Res::Append => {
                self.sounds_playing.push(Playing_Sound {
                    sound: Some(sound),
                    gen: 1,
                });
                Sound_Handle {
                    idx: self.sounds_playing.len() as u32 - 1,
                    gen: 1,
                }
            }

            Playing_Sound_Slot_Res::Replace { idx, old_gen } => {
                self.sounds_playing.insert(
                    idx,
                    Playing_Sound {
                        sound: Some(sound),
                        gen: old_gen + 1,
                    },
                );
                Sound_Handle {
                    idx: idx as _,
                    gen: old_gen + 1,
                }
            }

            Playing_Sound_Slot_Res::No_Free_Slot => {
                // replace first sound (TODO: we might want a better approach, like LRU or something)
                if self.sounds_playing.len() > 0 {
                    let old_gen = self.sounds_playing[0].gen;
                    self.sounds_playing[0] = Playing_Sound {
                        sound: Some(sound),
                        gen: old_gen + 1,
                    };
                    Sound_Handle {
                        idx: 0,
                        gen: old_gen + 1,
                    }
                } else {
                    Sound_Handle::INVALID
                }
            }
        }
    }

    pub fn get_sound(&self, hdl: Sound_Handle) -> Option<&sound::Sound> {
        if hdl == Sound_Handle::INVALID {
            lwarn!("get_sound failed because handle {:?} is invalid", hdl);
            None
        } else {
            if let Some(slot) = self.sounds_playing.get(hdl.idx as usize) {
                if slot.gen == hdl.gen && slot.sound.is_some() {
                    return Some(slot.sound.as_ref().unwrap());
                }
            }
            lwarn!(
                "get_sound failed because handle {:?} is obsolete or invalid",
                hdl
            );
            None
        }
    }

    pub fn get_sound_mut(&mut self, hdl: Sound_Handle) -> Option<&mut sound::Sound> {
        if hdl == Sound_Handle::INVALID {
            lwarn!("get_sound_mut failed because handle {:?} is invalid", hdl);
            None
        } else {
            if let Some(slot) = self.sounds_playing.get_mut(hdl.idx as usize) {
                if slot.gen == hdl.gen && slot.sound.is_some() {
                    return Some(slot.sound.as_mut().unwrap());
                }
            }
            lwarn!(
                "get_sound_mut failed because handle {:?} is obsolete or invalid",
                hdl
            );
            None
        }
    }

    pub fn n_sounds_playing(&self) -> usize {
        self.sounds_playing
            .iter()
            .filter(|slot| slot.sound.is_some())
            .count()
    }

    // Looks for the best slot to fit a new sound in.
    fn get_first_free_sound_slot(&self) -> Playing_Sound_Slot_Res {
        debug_assert!(self.sounds_playing.len() < u32::MAX as usize - 1);

        if self.sounds_playing.len() < self.max_concurrent_sounds {
            return Playing_Sound_Slot_Res::Append;
        }
        for (i, slot) in self.sounds_playing.iter().enumerate() {
            if slot.sound.is_none() {
                return Playing_Sound_Slot_Res::Replace {
                    idx: i,
                    old_gen: slot.gen,
                };
            }
        }
        Playing_Sound_Slot_Res::No_Free_Slot
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
