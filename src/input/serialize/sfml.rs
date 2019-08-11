use crate::core::common::serialize::{Binary_Serializable, Byte_Stream};
use crate::input::bindings::{keymap, mouse};
use sfml::window::Event;

const PRE_KEY_PRESSED: u8 = 0x0;
const PRE_KEY_RELEASED: u8 = 0x1;
const PRE_JOY_PRESSED: u8 = 0x2;
const PRE_JOY_RELEASED: u8 = 0x3;
const PRE_MOUSE_PRESSED: u8 = 0x4;
const PRE_MOUSE_RELEASED: u8 = 0x5;

impl Binary_Serializable for Event {
    fn serialize(&self, output: &mut Byte_Stream) -> std::io::Result<()> {
        match self {
            // Note: Event::Key is #[repr(i32)]
            Event::KeyPressed { code, .. } => {
                output.write_u8(PRE_KEY_PRESSED)?;
                output.write_u16(*code as u16)?;
            }
            Event::KeyReleased { code, .. } => {
                output.write_u8(PRE_KEY_RELEASED)?;
                output.write_u16(*code as u16)?;
            }
            Event::JoystickButtonPressed { joystickid, button } => {
                output.write_u8(PRE_JOY_PRESSED)?;
                output.write_u8(*joystickid as u8)?;
                output.write_u8(*button as u8)?;
            }
            Event::JoystickButtonReleased { joystickid, button } => {
                output.write_u8(PRE_JOY_RELEASED)?;
                output.write_u8(*joystickid as u8)?;
                output.write_u8(*button as u8)?;
            }
            // Note: Mouse::Button is #[repr(u32)]
            Event::MouseButtonPressed { button, .. } => {
                output.write_u8(PRE_MOUSE_PRESSED)?;
                output.write_u8(*button as u8)?;
            }
            Event::MouseButtonReleased { button, .. } => {
                output.write_u8(PRE_MOUSE_RELEASED)?;
                output.write_u8(*button as u8)?;
            }
            _ => (),
        }

        Ok(())
    }

    fn deserialize(input: &mut Byte_Stream) -> std::io::Result<Event> {
        let prelude = input.read_u8()?;
        match prelude {
            PRE_KEY_PRESSED => {
                let code = input.read_u16()?;
                let code = keymap::num_to_key(code as usize)
                    .unwrap_or_else(|| panic!("Invalid keycode {}", code));
                Ok(keymap::sfml::keypressed(code))
            }
            PRE_KEY_RELEASED => {
                let code = input.read_u16()?;
                let code = keymap::num_to_key(code as usize)
                    .unwrap_or_else(|| panic!("Invalid keycode {}", code));
                Ok(keymap::sfml::keyreleased(code))
            }
            PRE_JOY_PRESSED => {
                let joystickid = input.read_u8()?.into();
                let button = input.read_u8()?.into();
                Ok(Event::JoystickButtonPressed { joystickid, button })
            }
            PRE_JOY_RELEASED => {
                let joystickid = input.read_u8()?.into();
                let button = input.read_u8()?.into();
                Ok(Event::JoystickButtonReleased { joystickid, button })
            }
            PRE_MOUSE_PRESSED => {
                let button = input.read_u8()?;
                let button = mouse::num_to_mouse_btn(button as usize)
                    .unwrap_or_else(|| panic!("Invalid button {}", button));
                Ok(mouse::sfml::mousepressed(button.into()))
            }
            PRE_MOUSE_RELEASED => {
                let button = input.read_u8()?;
                let button = mouse::num_to_mouse_btn(button as usize)
                    .unwrap_or_else(|| panic!("Invalid button {}", button));
                Ok(mouse::sfml::mousereleased(button.into()))
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keymap::sfml::{keypressed, keyreleased};
    use mouse::sfml::{mousepressed, mousereleased};
    use sfml::window::mouse::Button;
    use sfml::window::Key;

    #[test]
    fn serialize_deserialize() {
        let mut byte_stream = Byte_Stream::new();

        let events = [
            keypressed(Key::A),
            keyreleased(Key::Escape),
            mousepressed(Button::Right),
            mousereleased(Button::Left),
            Event::JoystickButtonPressed {
                joystickid: 0,
                button: 2,
            },
            Event::JoystickButtonReleased {
                joystickid: 3,
                button: 0,
            },
        ];

        for event in events.iter() {
            event
                .serialize(&mut byte_stream)
                .unwrap_or_else(|err| panic!("Failed to serialize event: {}", err));
        }

        byte_stream.seek(0);

        let mut deser_events = vec![];

        while (byte_stream.pos() as usize) < byte_stream.len() {
            deser_events.push(
                Event::deserialize(&mut byte_stream)
                    .unwrap_or_else(|err| panic!("Failed to deserialize event: {}", err)),
            );
        }

        assert_eq!(events.len(), deser_events.len());
        for i in 0..events.len() {
            assert_eq!(events[i], deser_events[i]);
        }
    }
}
