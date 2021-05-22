use core::fmt;
pub struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            for c in s.as_bytes() { // we use bytes here instead of chars to print unicode characters out as well
                _putchar(*c);
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    () => ();
    ($($arg:tt)*) => ({ 
      unsafe { crate::print::_putchar(0x27); } // start of logstring
      core::fmt::write(&mut crate::print::SerialWriter, format_args!($($arg)*)).unwrap(); 
    });
}

#[macro_export]
macro_rules! println {
    () => ({
      unsafe {
        crate::print::_putchar(10); 
        crate::print::_putchar(13);
      }
    });
    ($($arg:tt)*) => ({ 
      unsafe { crate::print::_putchar(0x27); } // start of logstring
      core::fmt::write(&mut crate::print::SerialWriter, format_args!($($arg)*)).unwrap(); 
      unsafe {
        crate::print::_putchar(10); 
        crate::print::_putchar(13);
      }
    });
}

// needed by print macros
extern "C" {
    pub fn _putchar(c: u8);
}