use crate::{framebuffer::FrameBuffer, psf::PSFFont};
use core::fmt;
use lazy_static::lazy_static;
use limine::framebuffer;
use spin::mutex::Mutex;

// Only implementing graphics for now to keep things simple
struct ControlState {
    graphics_color_buffer: [u8; 2],
    graphics_color_index: u8,
}

impl ControlState {
    fn default() -> ControlState {
        ControlState {
            graphics_color_buffer: [0x00, 0x00],
            graphics_color_index: 0,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
enum TerminalColor {
    Black = 0xFF55555,
    Red = 0xFFFF5555,
    Green = 0xFF55FF55,
    Yellow = 0xFFFFFF55,
    Blue = 0xFF5555FF,
    Magenta = 0xFFFF55FF,
    Cyan = 0xFF55FFFF,
    White = 0xFFFFFFFF,
}

pub struct Terminal {
    font: PSFFont,
    row_pos: usize,
    col_pos: usize,
    pitch: usize,
    fb_height: usize,
    control_mode: bool,
    control_state: ControlState,
    terminal_color: TerminalColor,
}

impl Terminal {
    pub fn default() -> Terminal {
        let framebuffer = FrameBuffer::new();

        Terminal {
            font: PSFFont::default(),
            row_pos: 0,
            col_pos: 0,
            pitch: framebuffer.pitch().unwrap_or(0) as usize,
            fb_height: framebuffer.height().unwrap_or(0) as usize,
            control_mode: false,
            control_state: ControlState::default(),
            terminal_color: TerminalColor::White,
        }
    }

    fn get_row_offset(&self, row: usize) -> usize {
        return (row * self.pitch) + (self.row_pos * self.font.height as usize * self.pitch);
    }

    fn get_col_offset(&self, pixel: usize) -> usize {
        return (self.col_pos * 4 * self.font.width as usize) + pixel * 4;
    }

    fn get_word_wrap_pos(&self) -> usize {
        self.pitch / self.font.width as usize / 4
    }

    fn get_scroll_pos(&self) -> usize {
        self.fb_height / self.font.height as usize
    }

    fn increment_col(&mut self, framebuffer: &FrameBuffer) {
        self.col_pos += 1;
        if self.col_pos == self.get_word_wrap_pos() {
            self.newline(framebuffer);
        }
    }

    fn scroll(&mut self, framebuffer: &FrameBuffer) {
        self.row_pos -= 1;
        framebuffer.scroll_framebuffer(self.font.height as usize);
    }

    fn increment_row(&mut self, framebuffer: &FrameBuffer) {
        self.row_pos += 1;
        if self.row_pos == self.get_scroll_pos() {
            self.scroll(framebuffer)
        }
    }

    fn newline(&mut self, framebuffer: &FrameBuffer) {
        self.col_pos = 0;
        self.increment_row(framebuffer);
    }

    fn set_terminal_color(&mut self) {
        match self.control_state.graphics_color_buffer[0] {
            b'3' => match self.control_state.graphics_color_buffer[1] {
                b'0' => self.terminal_color = TerminalColor::Black,
                b'1' => self.terminal_color = TerminalColor::Red,
                b'2' => self.terminal_color = TerminalColor::Green,
                b'3' => self.terminal_color = TerminalColor::Yellow,
                b'4' => self.terminal_color = TerminalColor::Blue,
                b'5' => self.terminal_color = TerminalColor::Magenta,
                b'6' => self.terminal_color = TerminalColor::Cyan,
                b'7' => self.terminal_color = TerminalColor::White,
                b'9' => self.terminal_color = TerminalColor::White,
                _ => (),
            },
            _ => (),
        }
    }

    fn terminal_graphics_command(&mut self) {
        match self.control_state.graphics_color_buffer[0] {
            b'0' => self.terminal_color = TerminalColor::White,
            _ => (),
        }
    }

    pub fn write_u8(&mut self, byte: u8) {
        let framebuffer = FrameBuffer::new();

        if self.control_mode {
            // Only parsing graphics for now
            match byte {
                b'[' => {
                    self.control_state.graphics_color_index = 0;
                }
                b'0'..=b'9' => {
                    if self.control_state.graphics_color_index > 1 {
                        self.control_mode = false;
                        return;
                    }

                    self.control_state.graphics_color_buffer
                        [self.control_state.graphics_color_index as usize] = byte;
                    self.control_state.graphics_color_index += 1;
                }
                b';' => {
                    if self.control_state.graphics_color_index == 2 {
                        self.set_terminal_color();
                    }
                    if self.control_state.graphics_color_index == 1 {
                        self.terminal_graphics_command();
                    }

                    self.control_state.graphics_color_index = 0;
                }
                b'm' => {
                    if self.control_state.graphics_color_index == 2 {
                        self.set_terminal_color();
                    }
                    if self.control_state.graphics_color_index == 1 {
                        self.terminal_graphics_command();
                    }

                    self.control_mode = false;
                    self.control_state.graphics_color_index = 0;
                }
                _ => {
                    self.control_mode = false;
                    self.control_state.graphics_color_index = 0;
                }
            }
        } else {
            match byte {
                b'\n' => self.newline(&framebuffer),
                0x1b => self.control_mode = true,
                _ => {
                    for row in 0..self.font.height as usize {
                        let row_glyph = if byte != 0x81 {
                            self.font.read_glyph_row(byte as usize, row)
                        } else {
                            // Treating 0x81 as error char for now
                            if row % 2 == 0 {
                                0xAA
                            } else {
                                !0xAA
                            }
                        };

                        for pixel in 0..8_usize {
                            let bit_mask = 0x80 >> pixel;
                            let bit = row_glyph & bit_mask;
                            if bit != 0 {
                                let pixel_offset =
                                    self.get_row_offset(row) + self.get_col_offset(pixel);
                                framebuffer.put_pixel(pixel_offset, self.terminal_color as u32);
                            }
                        }
                    }

                    self.increment_col(&framebuffer);
                }
            }
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | 0x1b => self.write_u8(byte),
                _ => self.write_u8(0x81),
            }
        }
    }
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::default());
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::terminal::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ( if true {$crate::print!("\x1b[32m[DEBUG]\x1b[0m {}\n", format_args!($($arg)*)) });
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ( if true {$crate::print!("\x1b[36m[INFO]\x1b[0m {}\n", format_args!($($arg)*)) });
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ( if true {$crate::print!("\x1b[33m[WARN]\x1b[0m {}\n", format_args!($($arg)*))});
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ( if true {$crate::print!("\x1b[31m[ERROR]\x1b[0m {}\n", format_args!($($arg)*))});
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    TERMINAL.lock().write_fmt(args).unwrap();
}

// TODO
// Terminal colors with escape squence: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
