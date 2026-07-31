use core::slice::from_raw_parts;

use crate::config::STD_OUTPUT;

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> isize {
    match fd {
        STD_OUTPUT => {
            let slice = unsafe { from_raw_parts(buffer, len) };
            let str = core::str::from_utf8(slice).unwrap();
            crate::print!("{str}");
            len as isize
        }
        _ => {
            panic!("Unsupported output file descripter!");
        }
    }
}
