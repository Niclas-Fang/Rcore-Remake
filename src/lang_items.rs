use crate::{println, sbi};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log::error!(
            "[Kernel] panicked at {}:{} {}!",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        println!("[Kernel] panicked: {}", info.message());
    }
    sbi::shutdown();
}
