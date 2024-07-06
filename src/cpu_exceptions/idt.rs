use x86_64::registers::segmentation::{self, Segment};
use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;

pub type HandlerFunc = extern "C" fn() -> !;

pub struct InterruptDescriptorTable([Entry; 16]);

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        InterruptDescriptorTable([Entry::missing(); 16])
    }

    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) -> &mut EntryOpts {
        self.0[entry as usize] = Entry::new(segmentation::CS::get_reg(), handler);
        &mut self.0[entry as usize].opts
    }

    // Ensure that the IDT being loaded has a static lifetime. It should be alive
    // for as long as the OS is running (or at least until a new IDT is loaded,
    // which is potentially never(? does a new IDT ever get loaded?))
    pub fn load(&'static self) {
        use x86_64::structures::DescriptorTablePointer;
        use x86_64::VirtAddr;
        use x86_64::instructions::tables::lidt;
        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        };
        
        unsafe { lidt(&ptr) };
        
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Entry {
    low_ptr: u16,
    gdt_sel: SegmentSelector,
    opts: EntryOpts,
    mid_ptr: u16,
    high_ptr: u32,
    reserved: u32,
}

impl Entry {
    fn new(gdt_sel: SegmentSelector, handler: HandlerFunc) -> Self {
        let ptr = handler as u64;
        Entry {
            gdt_sel,
            low_ptr: ptr as u16,
            mid_ptr: (ptr >> 16) as u16,
            high_ptr: (ptr >> 32) as u32,
            opts: EntryOpts::new(),
            reserved: 0,
        }
    }

    fn missing() -> Self {
        Entry {
            gdt_sel: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            low_ptr: 0,
            mid_ptr: 0,
            high_ptr: 0,
            opts: EntryOpts::minimal(),
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOpts(u16);

impl EntryOpts {
    const MINIMAL: u16 = 0x0e00;
    fn minimal() -> Self {
        Self(Self::MINIMAL)
    }
    fn new() -> Self {
        let mut min = Self::minimal();
        min.set_present(true).enable_interrupts(false);
        min
    }
    pub fn set_ist_idx(&mut self, idx: u16) -> &mut Self {
        assert!(idx < 0b1000);
        self.0 = (self.0 & 0xeff8) | idx;
        self
    }

    pub fn enable_interrupts(&mut self, enable: bool) -> &mut Self {
        self.0 = match enable {
            true => self.0 | 0x100,
            false => self.0 & 0xeeff,
        };
        self
    }

    pub fn set_dpl(&mut self, dpl: u16) -> &mut Self {
        // Sanity checks
        assert!(dpl < 0b100);
        self.0 = (self.0 & 0x8fff) | (dpl << 13);
        self
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0 = match present {
            true => self.0 | 0x8000,
            false => self.0 & 0x6fff,
        };
        self
    }
}
