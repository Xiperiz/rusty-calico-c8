use std::time::Duration;

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use crate::ApplicationCmdSettings;
use crate::audio::SquareWave;
use crate::interpreter::{CalicoEvent, CalicoKey, Chip8Interpreter};

// TODO move away from SDL2 to some graphics library

pub struct Emulator {
    parsed_args: ApplicationCmdSettings,
    interpreter: Chip8Interpreter,
}

impl Emulator {
    pub(crate) fn new<'a>(parsed_args: ApplicationCmdSettings) -> Emulator {
        Emulator {
            interpreter: Chip8Interpreter::new(parsed_args.sound_enabled),
            parsed_args,
        }
    }

    fn get_calico_event_from_sdl_event(event: Event) -> CalicoEvent {
        match event {
            Event::KeyUp { .. } => CalicoEvent::KeyUp,
            Event::KeyDown { .. } => CalicoEvent::KeyDown,
            _ => CalicoEvent::Other
        }
    }

    fn get_calico_key_from_sdl_keycode(key: Keycode) -> CalicoKey {
        match key {
            Keycode::Kp1 => CalicoKey::Mk1,
            Keycode::Kp2 => CalicoKey::Mk2,
            Keycode::Kp3 => CalicoKey::Mk3,
            Keycode::Kp4 => CalicoKey::Mk4,
            Keycode::Q => CalicoKey::Q,
            Keycode::W => CalicoKey::W,
            Keycode::E => CalicoKey::E,
            Keycode::R => CalicoKey::R,
            Keycode::A => CalicoKey::A,
            Keycode::S => CalicoKey::S,
            Keycode::D => CalicoKey::D,
            Keycode::F => CalicoKey::F,
            Keycode::Z => CalicoKey::Z,
            Keycode::X => CalicoKey::X,
            Keycode::C => CalicoKey::C,
            Keycode::V => CalicoKey::V,

            _ => CalicoKey::Other
        }
    }

    pub fn run(&mut self, rom_path: &String) -> Result<(), String> {
        self.interpreter.load_rom(rom_path)
            .map_err(|e| e.to_string())?; // TODO fix error, add path

        let sdl_context = sdl2::init()?;
        let sdl_video = sdl_context.video()?;
        let sdl_audio = sdl_context.audio()?;
        let mut sdl_timer = sdl_context.timer()?;

        // Audio

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),  // mono
            samples: None,       // default sample size
        };

        let audio_device = sdl_audio.open_playback(None, &desired_spec, |spec| {
            SquareWave::new(440.0 / spec.freq as f32, 0.0, 0.25)
        })?;

        // Graphics

        let window = sdl_video
            .window("Rusty-Calico-C8",
                    self.parsed_args.window_size_x,
                    self.parsed_args.window_size_y)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
            .map_err(|e| e.to_string())?;

        let mut event_pump = sdl_context.event_pump()?;

        'running: loop {
            let start_timer = sdl_timer.performance_counter();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,

                    Event::KeyDown { keycode, .. } |
                    Event::KeyUp { keycode, .. } => {
                        match keycode {
                            Some(key) => {
                                self.interpreter.handle_event(Emulator::get_calico_event_from_sdl_event(event),
                                                              Emulator::get_calico_key_from_sdl_keycode(key));
                            }
                            _ => ()
                        }
                    }

                    _ => {}
                }
            }

            for _ in 0..self.parsed_args.cpu_clock_speed / 60 {
                self.interpreter.execute_next_instruction()
                    .map_err(|e| e.to_string())?;
            }

            self.interpreter.tick_timers();

            if self.interpreter.should_play_sound() {
                audio_device.resume();
                std::thread::sleep(Duration::from_millis(10));
                audio_device.pause();
            }

            if self.interpreter.draw_flag {
                texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for y in 0..32 {
                        for x in 0..64 {
                            let offset = y * pitch + x * 3;
                            let pixel_state = self.interpreter.frame_buffer.get_pixel(x as u8, y as u8);

                            buffer[offset] = if pixel_state { 255 } else { 0 };
                            buffer[offset + 1] = if pixel_state { 255 } else { 0 };
                            buffer[offset + 2] = if pixel_state { 255 } else { 0 };
                        }
                    }
                })?;

                canvas.clear();
                canvas.copy(&texture, None, Some(Rect::new(0, 0,
                                                           self.parsed_args.window_size_x,
                                                           self.parsed_args.window_size_y)))?;
                canvas.present();

                self.interpreter.draw_flag = false;
            }

            let end_timer = sdl_timer.performance_counter();

            let elapsed_ms = (end_timer - start_timer) as f32 / (sdl_timer.performance_frequency() * 1000) as f32;

            // Limit FPS to 60
            sdl_timer.delay((16.666f32 - elapsed_ms).floor() as u32);
        }

        Ok(())
    }
}