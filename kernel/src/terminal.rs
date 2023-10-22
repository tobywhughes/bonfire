use core::{fmt, ptr::NonNull};

use lazy_static::lazy_static;
use limine::{Framebuffer, NonNullPtr};
use spin::Mutex;

use crate::{
    framebuffer::FRAME_BUFFER,
    psf::{PSFHeader, PSF_FONT},
};

const PADDING: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Color {
    White = 0xFFFFFFFF,
    Red = 0xFFFF0000,
    Green = 0xFF00FF00,
    Blue = 0xFF0000FF,
}
pub struct TerminalWriter {
    col_position: usize,
    row_position: usize,
    terminal_color: Color,
    font_header: PSFHeader,
}

impl TerminalWriter {
    pub fn new() -> TerminalWriter {
        TerminalWriter {
            col_position: 0_usize,
            row_position: 0_usize,
            terminal_color: Color::White,
            font_header: PSFHeader::default(),
        }
    }

    pub fn write_char(&mut self, character: char) {
        let offset = 0x20 + (character as u32 * 0x10);

        if character == '\n' {
            self.new_row();
            return;
        }

        for row in 0..(self.font_header.height as usize) {
            if self.col_offset() + self.pixel_width() > FRAME_BUFFER.pitch() as usize {
                self.new_row();
            }

            let row_value: u8 = PSF_FONT[offset as usize + row];

            // TODO: Only works for 8 pixel width PSF
            for pixel in 0..8_usize {
                let mask = 0b10000000;
                let masked_value = row_value & (mask >> pixel);

                if masked_value > 0 {
                    let pixel_offset = self.row_offset(row)
                        + self.col_offset()
                        + (pixel * FRAME_BUFFER.bytes_per_pixel());
                    FRAME_BUFFER.put_pixel(pixel_offset, self.terminal_color as u32);
                }
            }
        }

        self.col_position += 1;
    }

    fn new_row(&mut self) {
        self.row_position += 1;
        self.col_position = 0;

        if self.row_position * self.font_header.height as usize >= FRAME_BUFFER.height() as usize {
            self.row_position -= 1;
            self.scroll();
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for character in s.chars() {
            match character as u32 {
                0x00000000..=0x000000FF => self.write_char(character),
                _ => self.write_char(0x00000002 as char),
            }
        }
    }

    pub fn scroll(&mut self) {
        let pixel_start = FRAME_BUFFER.pitch() * PADDING as u64;
        let range = FRAME_BUFFER.width() * FRAME_BUFFER.height()
            - (pixel_start / FRAME_BUFFER.bytes_per_pixel() as u64);
        let offset = FRAME_BUFFER.width() * self.font_header.height as u64;

        FRAME_BUFFER.move_range(pixel_start as usize, range as usize, offset as usize);
    }

    pub fn clear_screen(&mut self) {
        FRAME_BUFFER.clear_screen();
        self.col_position = 0;
        self.row_position = 0;
    }

    pub fn set_terminal_color(&mut self, color: Color) {
        self.terminal_color = color;
    }

    fn pixel_width(&self) -> usize {
        self.font_header.width as usize * FRAME_BUFFER.bytes_per_pixel()
    }

    fn col_offset(&self) -> usize {
        (self.col_position * self.font_header.width as usize + PADDING)
            * FRAME_BUFFER.bytes_per_pixel()
    }

    fn row_offset(&self, row: usize) -> usize {
        let row_position_offset: usize = self.row_position * self.font_header.height as usize;
        return (row + PADDING + row_position_offset) * FRAME_BUFFER.pitch() as usize;
    }
}

impl fmt::Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref TERMINAL_WRITER: Mutex<TerminalWriter> = Mutex::new(TerminalWriter::new());
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

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    TERMINAL_WRITER.lock().write_fmt(args).unwrap();
}
