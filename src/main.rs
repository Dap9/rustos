#![no_std]
// Main function is usually the entry point of the program. However, this apparently requires std.
// Thus we use no_main to declare that there is no main and that we define our own custom
// entrypoint
// In rust, execution begins at a C runtime library called 'crt0'. This does setup (of the stack
// and placing args in the right registers) then invokes the 'main' function which is the called by
// the 'start' language item after it does some rust specific setup.
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod test;
mod vga_buffer;

// No std -> no default panic handler. This means when a panic occurs it doesn't know what to do.
// Thus, we make one below

// Issue: by default rust does stack unwinding -> runs the destructors of all objects on the stack
// in the case of a panic. This frees all memory used by these objects, and allows the parent
// thread to continue execution after catching the panic. However, for the simple OS
// that we are making, we do not want to do this since it requires some OS-specific libraries (e.g.
// libunwind)
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}


// `extern "C"` -> use C calling convention for this function. Ensure name isn't mangled -> get a
// function with the name _start. This is the default entry point name for most systems (which the
// linker will look for). marked as diverging because it should never return. It should perform a
// 'shutdown' or some action when exiting the OS, e.g. by turning off the machine.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World!");

    // Only compiled on cargo test. Doesn't exist on regular runs.
    #[cfg(test)]
    test_main();

    loop {}
}

fn simple_print_hello_world() {
    let hello: &[u8] = b"Hello World!";

    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in hello.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
}

// Linker issue: Rust by default assumes we are dependent on C runtime. This is not true
// for us. Thus, it throws a linker error.
// How to resolve this?
// rust takes uses a 'target triple' of the host containing the arch, vendor, OS and ABI to decide
// how to build the target such that it can run on the host. We just need to set the target
// to be one that has no underlying OS.
// Chosen: thumbv7em-none-eabihf
// This is:
// Armv7E-M architecture family
// None underlying OS
// eabihf ABI
