use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};

use rand::Rng;

use crate::frame_buffer::FrameBuffer;
use crate::interpreter::InterpreterError::{InvalidOpcode, StackUnderflow};

pub const C8_FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

#[derive(Debug)]
pub enum InterpreterError {
    StackUnderflow { pc: u16 },
    InvalidOpcode { pc: u16, opcode: u16 },
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            StackUnderflow { pc } => {
                write!(f, "Stack underflow at PC={:#04x}", pc)
            }

            InvalidOpcode { pc, opcode } => {
                write!(f, "Invalid opcode={:#06x} at PC={:#04x}", opcode, pc)
            }
        }
    }
}

impl std::error::Error for InterpreterError {}

// Two below structs used to keep SDL and interpreter module separate
#[derive(PartialEq)]
pub enum CalicoEvent {
    KeyDown,
    KeyUp,
    Other,
}

#[derive(PartialEq)]
pub enum CalicoKey {
    Mk1,
    Mk2,
    Mk3,
    Mk4,
    Q,
    W,
    E,
    R,
    A,
    S,
    D,
    F,
    Z,
    X,
    C,
    V,
    Other,
}

pub struct Chip8Interpreter {
    pub frame_buffer: FrameBuffer,
    pub draw_flag: bool,
    memory: [u8; 4096],
    stack: Vec<u16>,
    keypad_status: [bool; 16],
    general_registers: [u8; 16],
    register_pc: u16,
    register_i: u16,
    delay_timer: u8,
    sound_timer: u8,
    sound_enabled: bool,
    current_opcode: u16,
}

impl Chip8Interpreter {
    pub fn new(sound_enabled: bool) -> Chip8Interpreter {
        let mut interpreter = Chip8Interpreter {
            frame_buffer: FrameBuffer::new(),
            draw_flag: false,
            memory: [0; 4096],
            stack: vec![],
            keypad_status: [false; 16],
            general_registers: [0x00; 16],
            register_pc: 0x200,
            register_i: 0x00,
            delay_timer: 0x00,
            sound_timer: 0x00,
            sound_enabled,
            current_opcode: 0x0000,
        };

        for i in 0..C8_FONT_SET.len() {
            interpreter.memory[i + 0x050] = C8_FONT_SET[i];
        }

        interpreter
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), std::io::Error> {
        let mut binary_file = File::open(path)?;
        let mut binary_data = Vec::new();

        binary_file.read_to_end(&mut binary_data)?;

        if binary_data.len() > 4096 - 0x200 {
            // ErrorKind::FileToLarge unstable for now..
            return Err(Error::new(ErrorKind::Other, "Binary too big for CHIP8"));
        }

        for i in 0..binary_data.len() {
            self.memory[i + 0x200] = binary_data[i];
        }

        Ok(())
    }

    pub fn handle_event(&mut self, event: CalicoEvent, key: CalicoKey) {
        if key == CalicoKey::Other || event == CalicoEvent::Other {
            return;
        }

        let key_index = key as usize;

        self.keypad_status[key_index] = if event == CalicoEvent::KeyDown {
            true
        } else {
            false
        };
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn should_play_sound(&self) -> bool {
        self.sound_timer != 0 && self.sound_enabled
    }

    fn get_x_from_opcode(&self) -> usize {
        ((self.current_opcode & 0x0F00) >> 8) as usize
    }

    fn get_y_from_opcode(&self) -> usize {
        ((self.current_opcode & 0x00F0) >> 4) as usize
    }

    fn get_nnn_from_opcode(&self) -> u16 {
        self.current_opcode & 0x0FFF
    }

    fn get_nn_from_opcode(&self) -> u8 {
        (self.current_opcode & 0x00FF) as u8
    }

    fn get_n_from_opcode(&self) -> u8 {
        (&self.current_opcode & 0x000F) as u8
    }

    fn draw(&mut self, x: usize, y: usize, height: u8) {
        let x_cord = self.general_registers[x];
        let y_cord = self.general_registers[y];

        let mut pixel_flipped = false;

        for diff_y in 0..height {
            let r = self.memory[(self.register_i + diff_y as u16) as usize];

            for diff_x in 0..8 {
                if r & (1 << (7 - diff_x)) != 0 {
                    self.frame_buffer.flip_pixel(x_cord + diff_x, y_cord + diff_y);
                    if !self.frame_buffer.get_pixel(x_cord + diff_x, y_cord + diff_y) {
                        pixel_flipped = true;
                    }
                }
            }
        }

        self.general_registers[0xF] = pixel_flipped as u8;
        self.draw_flag = true;
    }

    fn fn_call(&mut self, address: u16) {
        self.stack.push(self.register_pc);
        self.register_pc = address;
    }

    fn fn_return(&mut self) -> Result<(), InterpreterError> {
        match self.stack.pop() {
            Some(val) => self.register_pc = val,
            None => return Err(StackUnderflow { pc: self.register_pc })
        }

        Ok(())
    }

    pub fn execute_next_instruction(&mut self) -> Result<(), InterpreterError> {
        let hi_byte = self.memory[self.register_pc as usize];
        let lo_byte = self.memory[(self.register_pc + 1) as usize];

        self.current_opcode = (hi_byte as u16) << 8 | lo_byte as u16;
        self.register_pc += 2;

        match self.current_opcode & 0xF000 {
            0x0000 => {
                match self.current_opcode {
                    0x00ee => self.fn_return()?,

                    0x00e0 => {
                        self.frame_buffer.clear();
                        self.draw_flag = true;
                    }

                    _ => self.fn_call(self.get_nnn_from_opcode())
                }
            }

            0x1000 => self.register_pc = self.get_nnn_from_opcode(),

            0x2000 => self.fn_call(self.get_nnn_from_opcode()),

            0x3000 => {
                if self.general_registers[self.get_x_from_opcode()] == self.get_nn_from_opcode() {
                    self.register_pc += 2;
                }
            }

            0x4000 => {
                if self.general_registers[self.get_x_from_opcode()] != self.get_nn_from_opcode() {
                    self.register_pc += 2;
                }
            }

            0x5000 => {
                if self.general_registers[self.get_x_from_opcode()] == self.general_registers[self.get_y_from_opcode()] {
                    self.register_pc += 2;
                }
            }

            0x6000 => self.general_registers[self.get_x_from_opcode()] = self.get_nn_from_opcode(),

            0x7000 => {
                let res = self.general_registers[self.get_x_from_opcode()].wrapping_add(self.get_nn_from_opcode());

                self.general_registers[self.get_x_from_opcode()] = res;
            }

            0x8000 => {
                match self.current_opcode & 0x000F {
                    0x0 => self.general_registers[self.get_x_from_opcode()] = self.general_registers[self.get_y_from_opcode()],

                    0x1 => self.general_registers[self.get_x_from_opcode()] |= self.general_registers[self.get_y_from_opcode()],

                    0x2 => self.general_registers[self.get_x_from_opcode()] &= self.general_registers[self.get_y_from_opcode()],

                    0x3 => self.general_registers[self.get_x_from_opcode()] ^= self.general_registers[self.get_y_from_opcode()],

                    0x4 => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];
                        let reg_y = self.general_registers[self.get_y_from_opcode()];

                        let result = reg_x as u16 + reg_y as u16;

                        self.general_registers[self.get_x_from_opcode()] = result as u8;
                        self.general_registers[0xF] = (result > 0xFF) as u8;
                    }

                    0x5 => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];
                        let reg_y = self.general_registers[self.get_y_from_opcode()];

                        let result = (reg_x as i16) - (reg_y as i16);

                        self.general_registers[self.get_x_from_opcode()] = (result % (0x100 as i16)) as u8;
                        self.general_registers[0xF] = (result >= 0) as u8;
                    }

                    0x6 => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];

                        self.general_registers[0xF] = ((reg_x & 1) == 1) as u8;
                        self.general_registers[self.get_x_from_opcode()] >>= 1;
                    }

                    0x7 => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];
                        let reg_y = self.general_registers[self.get_y_from_opcode()];

                        let result = reg_y as i16 - reg_x as i16;

                        self.general_registers[self.get_x_from_opcode()] = (result % (0x100 as i16)) as u8;
                        self.general_registers[0xF] = (result >= 0) as u8;
                    }

                    0xE => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];

                        self.general_registers[0xF] = (reg_x & 0b10000000 == 0b10000000) as u8;
                        self.general_registers[self.get_x_from_opcode()] <<= 1;
                    }

                    _ => return Err(InvalidOpcode { pc: self.register_pc - 2, opcode: self.current_opcode })
                }
            }

            0x9000 => {
                if self.general_registers[self.get_x_from_opcode()] != self.general_registers[self.get_y_from_opcode()] {
                    self.register_pc += 2;
                }
            }

            0xA000 => self.register_i = self.get_nnn_from_opcode(),

            0xB000 => self.register_pc = self.get_nnn_from_opcode().wrapping_add(self.general_registers[0] as u16),

            0xC000 => {
                let random_byte = rand::thread_rng().gen::<u8>() & self.get_nn_from_opcode();

                self.general_registers[self.get_x_from_opcode()] = random_byte;
            }

            0xD000 => self.draw(self.get_x_from_opcode(), self.get_y_from_opcode(), self.get_n_from_opcode()),

            0xE000 => {
                match self.current_opcode & 0x00FF {
                    0x9E => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];

                        if self.keypad_status[reg_x as usize] {
                            self.register_pc += 2;
                        }
                    }

                    0xA1 => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];

                        if !self.keypad_status[reg_x as usize] {
                            self.register_pc += 2;
                        }
                    }

                    _ => return Err(InvalidOpcode { pc: self.register_pc - 2, opcode: self.current_opcode })
                }
            }

            0xF000 => {
                match self.current_opcode & 0x00FF {
                    0x07 => self.general_registers[self.get_x_from_opcode()] = self.delay_timer,

                    0x0A => {
                        let mut key_pressed = false;

                        for i in 1..16 {
                            if self.keypad_status[i] {
                                self.general_registers[self.get_x_from_opcode()] = i as u8;
                                key_pressed = true;
                            }
                        }

                        // If not pressed, stay on this instruction until pressed
                        if !key_pressed
                        {
                            self.register_pc -= 2;
                        }
                    }

                    0x15 => self.delay_timer = self.general_registers[self.get_x_from_opcode()],

                    0x18 => self.sound_timer = self.general_registers[self.get_x_from_opcode()],

                    0x1E => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];

                        self.register_i = self.register_i.wrapping_add(reg_x as u16);
                    }

                    0x29 => self.register_i = (self.general_registers[self.get_x_from_opcode()] as u16) * 5,

                    0x33 => {
                        let reg_x = self.general_registers[self.get_x_from_opcode()];

                        self.memory[self.register_i as usize] = reg_x / 100;
                        self.memory[self.register_i as usize + 1] = (reg_x / 10) % 10;
                        self.memory[self.register_i as usize + 2] = reg_x % 10;
                    }

                    0x55 => {
                        let end_index = self.get_x_from_opcode();

                        for i in 0..end_index + 1 {
                            self.memory[self.register_i as usize + i] = self.general_registers[i];
                        }
                    }

                    0x65 => {
                        for i in 0..=self.get_x_from_opcode() {
                            self.general_registers[i] = self.memory[self.register_i as usize + i];
                        }
                    }

                    _ => return Err(InvalidOpcode { pc: self.register_pc - 2, opcode: self.current_opcode })
                }
            }

            _ => return Err(InvalidOpcode { pc: self.register_pc - 2, opcode: self.current_opcode })
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_function_call() {
        let mut interpreter = Chip8Interpreter::new(false);
        let after_jump_pc = interpreter.register_pc;

        interpreter.fn_call(0x2540);
        interpreter.fn_return().unwrap();

        assert_eq!(after_jump_pc, interpreter.register_pc);
    }
}
