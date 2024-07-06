use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// Statics are evaluated at compile time -> similar to const.
// Thus, using Mutex::new or a similar non-const function will not be evaluated.
// Lazy static instead evaluates the object the first time it is accessed.
lazy_static! {
    pub static ref WRITER: Mutex<VGAWriter> = Mutex::new(VGAWriter {
        col: 0,
        row: 0,
        fgbg: ColorCode::new(Color::LightRed, Color::Black),
        // 0xb8000 is the beginning address of the VGA buffer
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        ($crate::vga_buffer::_print(format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        ($crate::print!("\n"))
    };
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    // format: 0-7 foreground; 8-11 background
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct VGAWriter {
    col: usize,
    row: usize,
    fgbg: ColorCode,
    buffer: &'static mut Buffer,
}

impl VGAWriter {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = self.row;
                let col = self.col;
                let color_code = self.fgbg;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.col += 1;
            }
        }
    }

    // Goes to a new line. If buffer is full, moves all values up & sets self.row = BUFFER_HEIGHT - 1
    fn new_line(&mut self) {
        if self.row >= BUFFER_HEIGHT - 1 {
            // Move all rows.
            for row in 0..BUFFER_HEIGHT - 1 {
                for col in 0..BUFFER_WIDTH {
                    self.buffer.chars[row][col].write(self.buffer.chars[row + 1][col].read());
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row += 1;
        }
        self.col = 0;
    }

    pub fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: 0x0,
            color_code: self.fgbg,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }
}

// Just for a sanity check test.
pub fn hello_world() {
    let mut screen = VGAWriter {
        col: 0,
        row: 0,
        fgbg: ColorCode::new(Color::LightRed, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    screen
        .write_string("Hello World!\nHello Again!\n")
}

impl fmt::Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
