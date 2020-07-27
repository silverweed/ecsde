use crate::common::serialize::{Binary_Serializable, Byte_Stream};
use crate::input::events::Input_Raw_Event;
use crate::input::{keyboard, mouse};
use std::io;

const PRE_KEY_PRESSED: u8 = 0x0;
const PRE_KEY_RELEASED: u8 = 0x1;
const PRE_JOY_PRESSED: u8 = 0x2;
const PRE_JOY_RELEASED: u8 = 0x3;
const PRE_MOUSE_PRESSED: u8 = 0x4;
const PRE_MOUSE_RELEASED: u8 = 0x5;
const PRE_WHEEL_SCROLLED: u8 = 0x6;

pub fn should_event_be_serialized(event: &Input_Raw_Event) -> bool {
    let mut bs = Byte_Stream::new();
    if event.serialize(&mut bs).is_ok() {
        bs.pos() > 0
    } else {
        false
    }
}

impl Binary_Serializable for Input_Raw_Event {
    fn serialize(&self, output: &mut Byte_Stream) -> io::Result<()> {
        match self {
            Input_Raw_Event::Key_Pressed { code } => {
                output.write_u8(PRE_KEY_PRESSED)?;
                output.write_u16(*code as u16)?;
            }
            Input_Raw_Event::Key_Released { code } => {
                output.write_u8(PRE_KEY_RELEASED)?;
                output.write_u16(*code as u16)?;
            }
            Input_Raw_Event::Joy_Button_Pressed {
                joystick_id,
                button,
            } => {
                output.write_u8(PRE_JOY_PRESSED)?;
                output.write_u8(*joystick_id as u8)?;
                output.write_u8(*button as u8)?;
            }
            Input_Raw_Event::Joy_Button_Released {
                joystick_id,
                button,
            } => {
                output.write_u8(PRE_JOY_RELEASED)?;
                output.write_u8(*joystick_id as u8)?;
                output.write_u8(*button as u8)?;
            }
            Input_Raw_Event::Mouse_Button_Pressed { button } => {
                output.write_u8(PRE_MOUSE_PRESSED)?;
                output.write_u8(*button as u8)?;
            }
            Input_Raw_Event::Mouse_Button_Released { button } => {
                output.write_u8(PRE_MOUSE_RELEASED)?;
                output.write_u8(*button as u8)?;
            }
            Input_Raw_Event::Mouse_Wheel_Scrolled { delta } => {
                output.write_u8(PRE_WHEEL_SCROLLED)?;
                output.write_f32(*delta)?;
            }
            _ => (),
        }

        Ok(())
    }

    fn deserialize(input: &mut Byte_Stream) -> io::Result<Input_Raw_Event> {
        let prelude = input.read_u8()?;
        match prelude {
            PRE_KEY_PRESSED => {
                let code = input.read_u16()?;
                let code = keyboard::num_to_key(code as _).ok_or(io::ErrorKind::InvalidData)?;
                Ok(Input_Raw_Event::Key_Pressed { code })
            }
            PRE_KEY_RELEASED => {
                let code = input.read_u16()?;
                let code = keyboard::num_to_key(code as _).ok_or(io::ErrorKind::InvalidData)?;
                Ok(Input_Raw_Event::Key_Released { code })
            }
            PRE_JOY_PRESSED => {
                let joystick_id = input.read_u8()?.into();
                let button = input.read_u8()?.into();
                Ok(Input_Raw_Event::Joy_Button_Pressed {
                    joystick_id,
                    button,
                })
            }
            PRE_JOY_RELEASED => {
                let joystick_id = input.read_u8()?.into();
                let button = input.read_u8()?.into();
                Ok(Input_Raw_Event::Joy_Button_Released {
                    joystick_id,
                    button,
                })
            }
            PRE_MOUSE_PRESSED => {
                let button = input.read_u8()?;
                let button =
                    mouse::num_to_mouse_btn(button as usize).ok_or(io::ErrorKind::InvalidData)?;
                Ok(Input_Raw_Event::Mouse_Button_Pressed {
                    button: button.into(),
                })
            }
            PRE_MOUSE_RELEASED => {
                let button = input.read_u8()?;
                let button =
                    mouse::num_to_mouse_btn(button as usize).ok_or(io::ErrorKind::InvalidData)?;
                Ok(Input_Raw_Event::Mouse_Button_Released {
                    button: button.into(),
                })
            }
            PRE_WHEEL_SCROLLED => {
                let delta = input.read_f32()?;
                Ok(Input_Raw_Event::Mouse_Wheel_Scrolled { delta })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid prelude: {}", prelude),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyboard::sfml::{keypressed, keyreleased};
    use keyboard::Key;
    use mouse::sfml::{mousepressed, mousereleased};
    use sfml::window::mouse::Button;

    #[test]
    fn serialize_deserialize() {
        let mut byte_stream = Byte_Stream::new();

        let events = [
            Input_Raw_Event::Key_Pressed { code: Key::A },
            Input_Raw_Event::Key_Released { code: Key::Escape },
            Input_Raw_Event::Mouse_Button_Pressed {
                button: Button::Right,
            },
            Input_Raw_Event::Mouse_Button_Released {
                button: Button::Left,
            },
            Input_Raw_Event::Joy_Button_Pressed {
                joystickid: 0,
                button: 2,
            },
            Input_Raw_Event::Joy_Button_Released {
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
                Input_Raw_Event::deserialize(&mut byte_stream)
                    .unwrap_or_else(|err| panic!("Failed to deserialize event: {}", err)),
            );
        }

        assert_eq!(events.len(), deser_events.len());
        for i in 0..events.len() {
            assert_eq!(events[i], deser_events[i]);
        }
    }
}
