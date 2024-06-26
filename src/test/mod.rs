use crate::{print, println};

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {

    println!("Running {} tests!", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial_assertion... ");
    assert_eq!(1, 1);
    println!(" [ok]");

    exit_qemu(QemuExitCode::Success);
}

// Function to exit qemu

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port:Port<u32> = Port::new(0xf4);
        port.write(exit_code as u32)
    }
}
