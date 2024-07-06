use lazy_static::lazy_static;
use crate::cpu_exceptions::idt::InterruptDescriptorTable;
use crate::println;

pub mod idt;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.set_handler(0, div_by_zero_handler);

        idt
    };
}


static INITED: bool = false;

pub fn init() {
    if INITED {
        println!("Already initialized...\n")
    }

    IDT.load();
}

extern "C" fn div_by_zero_handler() -> ! {
    println!("Handling div_by_zero...");
    loop {}
}
