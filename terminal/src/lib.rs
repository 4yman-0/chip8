#![no_std]
#![warn(clippy::alloc_instead_of_core)]
extern crate alloc;
use alloc::{boxed::Box, vec};

fn copy_slice_min<T: Copy>(dst: &mut [T], src: &[T]) {
    let min = dst.len().min(src.len());
    dst[..min].copy_from_slice(&src[..min]);
}

pub const FONT_OCTO: &[u8] = &[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const FONT_FISH: &[u8] = &[
    0x60, 0xA0, 0xA0, 0xA0, 0xC0, 0x40, 0xC0, 0x40, 0x40, 0xE0, 0xC0, 0x20, 0x40, 0x80, 0xE0, 0xC0,
    0x20, 0x40, 0x20, 0xC0, 0x20, 0xA0, 0xE0, 0x20, 0x20, 0xE0, 0x80, 0xC0, 0x20, 0xC0, 0x40, 0x80,
    0xC0, 0xA0, 0x40, 0xE0, 0x20, 0x60, 0x40, 0x40, 0x40, 0xA0, 0x40, 0xA0, 0x40, 0x40, 0xA0, 0x60,
    0x20, 0x40, 0x40, 0xA0, 0xE0, 0xA0, 0xA0, 0xC0, 0xA0, 0xC0, 0xA0, 0xC0, 0x60, 0x80, 0x80, 0x80,
    0x60, 0xC0, 0xA0, 0xA0, 0xA0, 0xC0, 0xE0, 0x80, 0xC0, 0x80, 0xE0, 0xE0, 0x80, 0xC0, 0x80, 0x80,
];

const MEMORY_SIZE: usize = 4096;
const START_PC: usize = 512;
const START_PC_U16: u16 = START_PC as u16;
const STACK_LEN: usize = 16;

pub type Chip8Result<T> = Result<T, Chip8Error>;

#[derive(Debug, Clone)]
pub enum Chip8Error {
    UnknownOp(u16),
    StackOverflow,
    StackUnderflow,
}

impl core::fmt::Display for Chip8Error {
    fn fmt(&self, w: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::UnknownOp(op) => {
                write!(w, "Unknown op: 0x{op:04X}")
            }
            Self::StackOverflow => {
                write!(w, "Stack overflow")
            }
            Self::StackUnderflow => {
                write!(w, "Stack underflow")
            }
        }
    }
}

impl core::error::Error for Chip8Error {}

#[derive(Clone, Copy)]
pub enum OpFlag {
    None,
    WaitVblank,
    WaitInput,
    Exit,
}

pub struct Settings {
    pub draw_wait: bool,
    pub shift_quirks: bool,
    pub vf_order_quirks: bool,
    pub logic_quirks: bool,
    pub scroll_quirks: bool,
    pub res_mode_quirks: bool,
    pub load_store_quirks: bool,
    pub jump_quirks: bool,
}

impl Settings {
    pub const fn chip8() -> Self {
        Self {
            draw_wait: true,
            shift_quirks: false,
            vf_order_quirks: false,
            logic_quirks: true,
            scroll_quirks: false,
            res_mode_quirks: false,
            load_store_quirks: false,
            jump_quirks: false,
        }
    }
    pub const fn schip() -> Self {
        Self {
            draw_wait: true,
            shift_quirks: true,
            vf_order_quirks: false,
            logic_quirks: false,
            scroll_quirks: false,
            res_mode_quirks: false,
            load_store_quirks: true,
            jump_quirks: true,
        }
    }
    pub const fn schip_modern() -> Self {
        Self {
            draw_wait: false,
            ..Self::schip()
        }
    }
    pub const fn xochip() -> Self {
        // TODO: almost there, just needs clipping quirk
        Self {
            draw_wait: false,
            shift_quirks: false,
            vf_order_quirks: false,
            logic_quirks: false,
            scroll_quirks: false,
            res_mode_quirks: false,
            load_store_quirks: false,
            jump_quirks: false,
        }
    }
}

pub struct Screen {
    pub fb: [bool; 128 * 64],
    pub is_hires: bool,
}

impl Screen {
    pub const fn width(&self) -> usize {
        if self.is_hires { 128 } else { 64 }
    }

    pub const fn height(&self) -> usize {
        if self.is_hires { 64 } else { 32 }
    }

    pub fn as_slice(&self) -> &[bool] {
        if self.is_hires {
            &self.fb
        } else {
            &self.fb[..64 * 32]
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [bool] {
        if self.is_hires {
            &mut self.fb
        } else {
            &mut self.fb[..64 * 32]
        }
    }
}

pub struct Context<T: rand_core::Rng> {
    pub settings: Settings,
    pub memory: Box<[u8]>,
    // many kilobytes, seems OK for stack
    pub screen: Screen,
    pub gpr: [u8; 16],
    pub idx: u16,
    pub pc: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [u16; STACK_LEN],
    pub sp: u16,
    pub keys: [bool; 16],
    pub keys_modified: [bool; 16],
    pub rng: T,
}

impl<T: rand_core::Rng> Context<T> {
    pub fn new(code: &[u8], rng: T) -> Self {
        let mut memory = vec![0; MEMORY_SIZE].into_boxed_slice();
        copy_slice_min(&mut memory[START_PC..], code);
        Self {
            settings: Settings::chip8(),
            memory,
            screen: Screen {
                fb: [false; 128 * 64],
                is_hires: false,
            },
            gpr: [0; 16],
            idx: 0,
            pc: START_PC_U16,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_LEN],
            sp: 0,
            keys: [false; 16],
            keys_modified: [false; 16],
            rng,
        }
    }

    pub fn reset(&mut self, code: &[u8]) {
        self.memory[START_PC..].fill(0x0);
        copy_slice_min(&mut self.memory[START_PC..], code);
        self.pc = START_PC_U16;
        self.sp = 0;
        self.screen.as_mut_slice().fill(false);
        self.gpr.fill(0);
    }

    pub fn write_font(&mut self, font: &[u8]) {
        copy_slice_min(&mut self.memory[..START_PC], font);
    }

    pub fn update_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    pub fn update_input(&mut self) {
        self.keys_modified.fill(false);
    }

    fn stack_push(&mut self, pc: u16) -> Chip8Result<()> {
        self.sp += 1;
        if self.sp as usize >= self.stack.len() {
            return Err(Chip8Error::StackOverflow);
        }
        self.stack[self.sp as usize] = pc;
        Ok(())
    }

    fn stack_pop(&mut self) -> Chip8Result<u16> {
        if self.sp == 0 {
            return Err(Chip8Error::StackUnderflow);
        }
        let pc = self.stack[self.sp as usize];
        self.sp -= 1;
        Ok(pc)
    }

    fn write_carry(&mut self, dst: usize, value: u8, flag: u8) {
        self.gpr[dst] = value;
        self.gpr[0xf] = flag;
        if self.settings.vf_order_quirks {
            self.gpr[dst] = value;
        }
    }

    pub fn emulate_cycle(&mut self) -> Chip8Result<OpFlag> {
        // CHIP8 is big endian
        let op = u16::from_be_bytes([
            self.memory[self.pc as usize],
            self.memory[self.pc as usize + 1],
        ]);

        let o = (op & 0xF000) >> 12;
        let x: usize = ((op & 0x0F00) >> 8).into();
        let y: usize = ((op & 0x00F0) >> 4).into();
        let nnn = op & 0x0FFF;
        let kk = (nnn & 0x00FF) as u8;
        let n = kk & 0x0F;

        match o {
            0x0 => {
                //if op & 0xFFF0 == 0x00C0 {
                //    let _dist = n as usize;
                //    unimplemented!("SuperChip scroll down");
                //}
                match op {
                    // From Octo
                    //0x0000 => return Ok(OpFlag::Exit),
                    // From CHIP-8 Classic / Color
                    0x0000 => {}
                    0x00E0 => {
                        self.screen.as_mut_slice().fill(false);
                    }
                    0x00EE => {
                        self.pc = self.stack_pop()?;
                    }
                    0x00FB => {
                        let w = self.screen.width();
                        let dist = if !self.screen.is_hires && self.settings.scroll_quirks {
                            2
                        } else {
                            4
                        };
                        for line in self.screen.as_mut_slice().chunks_exact_mut(w) {
                            line.rotate_right(dist);
                            line[..dist].fill(false);
                        }
                    }
                    0x00FC => {
                        let w = self.screen.width();
                        let dist = if !self.screen.is_hires && self.settings.scroll_quirks {
                            2
                        } else {
                            4
                        };
                        for line in self.screen.as_mut_slice().chunks_exact_mut(w) {
                            line.rotate_left(dist);
                            line[w - dist..].fill(false);
                        }
                    }
                    0x00FD => return Ok(OpFlag::Exit),
                    0x00FE => {
                        self.screen.is_hires = false;
                        if !self.settings.res_mode_quirks {
                            self.screen.as_mut_slice().fill(false);
                        }
                    }
                    0x00FF => {
                        self.screen.is_hires = true;
                        if !self.settings.res_mode_quirks {
                            self.screen.as_mut_slice().fill(false);
                        }
                    }
                    _ => return Err(Chip8Error::UnknownOp(op)),
                }
            }
            0x1 => {
                self.pc = nnn;
                return Ok(OpFlag::None);
            }
            0x2 => {
                self.stack_push(self.pc)?;
                self.pc = nnn;
                return Ok(OpFlag::None);
            }
            0x3 => {
                if self.gpr[x] == kk {
                    self.pc += 2;
                }
            }
            0x4 => {
                if self.gpr[x] != kk {
                    self.pc += 2;
                }
            }
            0x5 => {
                if self.gpr[x] == self.gpr[y] {
                    self.pc += 2;
                }
            }
            0x6 => {
                self.gpr[x] = kk;
            }
            0x7 => {
                self.gpr[x] = self.gpr[x].wrapping_add(kk);
            }
            0x8 => match n {
                0x0 => {
                    self.gpr[x] = self.gpr[y];
                }
                0x1 => {
                    self.gpr[x] |= self.gpr[y];
                    if self.settings.logic_quirks {
                        self.gpr[0xf] = 0;
                    }
                }
                0x2 => {
                    self.gpr[x] &= self.gpr[y];
                    if self.settings.logic_quirks {
                        self.gpr[0xf] = 0;
                    }
                }
                0x3 => {
                    self.gpr[x] ^= self.gpr[y];
                    if self.settings.logic_quirks {
                        self.gpr[0xf] = 0;
                    }
                }
                0x4 => {
                    let (sum, carry) = self.gpr[x].overflowing_add(self.gpr[y]);
                    self.write_carry(x, sum, u8::from(carry));
                }
                0x5 => {
                    let (diff, carry) = self.gpr[x].overflowing_sub(self.gpr[y]);
                    self.write_carry(x, diff, u8::from(!carry));
                }
                0x6 => {
                    if self.settings.shift_quirks {
                        self.gpr[y] = self.gpr[x];
                    }
                    self.write_carry(x, self.gpr[y] >> 1, self.gpr[y] & 0x1);
                }
                0x7 => {
                    let (diff, carry) = self.gpr[y].overflowing_sub(self.gpr[x]);
                    self.gpr[x] = diff;
                    self.gpr[0xf] = u8::from(!carry);
                }
                0xE => {
                    if self.settings.shift_quirks {
                        self.gpr[y] = self.gpr[x];
                    }
                    self.write_carry(x, self.gpr[y] << 1, self.gpr[y] >> 7);
                }
                _ => return Err(Chip8Error::UnknownOp(op)),
            },
            0x9 => {
                if self.gpr[x] != self.gpr[y] {
                    self.pc += 2;
                }
            }
            0xa => {
                self.idx = nnn;
            }
            0xb => {
                if self.settings.jump_quirks {
                    let reg = ((nnn >> 8) & 0xF) as usize;
                    let reg = u16::from(self.gpr[reg]);
                    self.pc = nnn + reg;
                } else {
                    self.pc = nnn.wrapping_add(u16::from(self.gpr[0]));
                }
                return Ok(OpFlag::None);
            }
            0xc => {
                let mut rand = [0_u8];
                self.rng.fill_bytes(&mut rand);
                self.gpr[x] = rand[0] & kk;
            }
            0xd => {
                let scr_w = self.screen.width();
                let scr_h = self.screen.height();
                let h: usize = n.into();
                let x = self.gpr[x] as usize % scr_w;
                let y = self.gpr[y] as usize % scr_h;
                self.gpr[0xf] = 0;
                'draw: for yline in 0..h {
                    let idx_y = y + yline;
                    if idx_y >= scr_h {
                        break 'draw;
                    }
                    let pixel = self.memory[self.idx as usize + yline];
                    for xline in 0..8 {
                        if pixel & (0x80 >> xline) != 0 {
                            let idx_x = x + xline;
                            if idx_x >= scr_w {
                                break 'draw;
                            }
                            let idx = idx_y * scr_w + idx_x;
                            self.gpr[0xF] |= u8::from(self.screen.as_slice()[idx]);
                            self.screen.as_mut_slice()[idx] ^= true;
                        }
                    }
                }
                if self.settings.draw_wait {
                    // ?
                    self.pc += 2;
                    return Ok(OpFlag::WaitVblank);
                }
            }
            0xe => {
                let key = (self.gpr[x] & 0x0f) as usize;
                match kk {
                    0x9e => {
                        if self.keys[key] {
                            self.pc += 2;
                        }
                    }
                    0xa1 => {
                        if !self.keys[key] {
                            self.pc += 2;
                        }
                    }
                    _ => return Err(Chip8Error::UnknownOp(op)),
                }
            }
            0xf => match kk {
                0x07 => {
                    self.gpr[x] = self.delay_timer;
                }
                0x0a => {
                    for idx in 0..self.keys.len() {
                        if self.keys_modified[idx] && !self.keys[idx] {
                            self.gpr[x] = idx as u8;
                            self.pc += 2;
                        }
                    }
                    return Ok(OpFlag::WaitInput);
                }
                0x15 => {
                    self.delay_timer = self.gpr[x];
                }
                0x18 => {
                    self.sound_timer = self.gpr[x];
                }
                0x1e => {
                    self.idx = self.idx.wrapping_add(u16::from(self.gpr[x]));
                }
                0x29 => {
                    self.idx = u16::from(self.gpr[x].wrapping_mul(5));
                }
                0x30 => {
                    // TODO: test this
                    self.idx = u16::from(self.gpr[x].wrapping_mul(10));
                }
                0x33 => {
                    let idx = self.idx as usize;
                    let x = self.gpr[x];
                    self.memory[idx] = x / 100;
                    self.memory[idx + 1] = (x / 10) % 10;
                    self.memory[idx + 2] = x % 10;
                }
                0x55 => {
                    let idx = self.idx as usize;
                    for offset in 0..=x {
                        self.memory[idx + offset] = self.gpr[offset];
                    }
                    if !self.settings.load_store_quirks {
                        self.idx = self.idx.wrapping_add(x as u16).wrapping_add(1);
                    }
                }
                0x65 => {
                    let idx = self.idx as usize;
                    for offset in 0..=x {
                        self.gpr[offset] = self.memory[idx + offset];
                    }
                    if !self.settings.load_store_quirks {
                        self.idx = self.idx.wrapping_add(x as u16).wrapping_add(1);
                    }
                }
                //0x75 => todo!("STORX persistent storage"),
                //0x85 => todo!("RSTRX persistent storage"),
                _ => return Err(Chip8Error::UnknownOp(op)),
            },
            _ => return Err(Chip8Error::UnknownOp(op)),
        }
        self.pc += 2;
        Ok(OpFlag::None)
    }
}
