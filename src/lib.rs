// This is a library intended to provide a simple way to perform tests for RustOS
// Based off https://os.phil-opp.com/testing/#integration-tests

#![no_std]

// test mode -> no main attribute applied
#![cfg_attr(test, no_main)]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod serial;
pub mod vga_buffer;
pub mod cpu_exceptions;

use core::panic::PanicInfo;

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    cpu_exceptions::init();

    test_main();

    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests!", tests.len());
    for test in tests {
        test.run(false);
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[fail]");
    serial_println!("Error: {}\n", info);
    loop {}
}

pub fn should_panic_handler(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);

    loop {}
}

pub trait Testable {
    fn run(&self, panic: bool) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self, panic: bool) -> () {
        serial_print!("{}... \t", core::any::type_name::<T>());
        self();
        if panic {
            serial_println!("[fail]");
            serial_println!("Error:\n expected test to panic but did not panic");
            exit_qemu(QemuExitCode::Failure);
        }
        serial_println!("[ok]");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port: Port<u32> = Port::new(0xf4);
        port.write(exit_code as u32)
    }
}
