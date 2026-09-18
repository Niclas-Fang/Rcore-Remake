use crate::{
    config::{STD_INPUT, STD_OUTPUT},
    mm::buffer_from_path,
    sbi::{getchar, putchar},
    task::{current_user_token, suspend_current_and_run_next},
};

pub fn sys_write(fd: usize, ptr: *const u8, len: usize) -> isize {
    match fd {
        STD_OUTPUT => {
            let token = current_user_token();
            let buffers = buffer_from_path(token, ptr, len);
            for buf in buffers {
                for &byte in buf.iter() {
                    putchar(byte);
                }
            }
            len as isize
        }
        _ => -1,
    }
}

pub fn sys_read(fd: usize, buffer: *const u8, len: usize) -> isize {
    match fd {
        STD_INPUT => {
            assert_eq!(len, 1, "Only support reading of length 1!");
            let mut c: i8;
            loop {
                c = getchar() as i8;
                if c == -1 {
                    suspend_current_and_run_next();
                    continue;
                } else {
                    break;
                }
            }
            let mut buffers = buffer_from_path(current_user_token(), buffer, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(c as u8);
            }
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
