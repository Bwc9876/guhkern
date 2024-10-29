use crate::{
    consts::{KERNEL_START, PHYS_STOP},
    kalloc::{allocate_page, set_memory, MAX_VIRTUAL_ADDRESS, PAGE_SIZE},
    plic::PLIC,
    uart::UART_LOC0,
    virtio::VIRTIO0,
};

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
/// Represents an entry within a page table
/// 64 bit string laid out like so:
/// - 0..10: Flags
/// - 11..44: PPN
/// - 45..64: Reserved
struct PageTableEntry(pub usize);

impl PageTableEntry {
    pub const FLAG_VALID: usize = 1 << 0; // Valid entry
    pub const FLAG_READ: usize = 1 << 1;
    pub const FLAG_WRITE: usize = 1 << 2;
    pub const FLAG_EXEC: usize = 1 << 3;
    pub const FLAG_USER: usize = 1 << 4; // Can be accessed in user-mode

    #[inline]
    pub fn extract_flags(&self) -> usize {
        // Extract the 10 flag bits
        // (there's only 5 that are useful for us, hence why there's only 5 constants defined above)
        self.0 & 0x3FF
    }

    #[inline]
    pub fn extract_physical_page_number(&self) -> usize {
        // We don't want the flag bits, but we do want the PPN
        (self.0 >> 10) << 12
    }

    #[inline]
    pub fn new(physical: usize, flags: usize) -> Self {
        // Get rid of the 12-bit offset of the address at the start
        // Then get rid of the 10 reserved bit at the end
        Self(((physical >> 12) << 10) | flags)
    }

    #[inline]
    pub fn as_table(&self) -> PageTable {
        let pg = self.extract_physical_page_number();
        PageTable(pg as *mut usize)
    }

    #[inline]
    pub fn allocate_as_new_table(flags: usize) -> Option<(Self, PageTable)> {
        let table = PageTable::new()?;
        Some((Self::new(table.0 as usize, flags), table))
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
struct VirtualAddr(usize);

impl VirtualAddr {
    #[inline]
    pub fn extract_index_at_level(&self, level: usize) -> usize {
        (self.0 >> (12 + (level * 9))) & 0x1FF // Want 9 bits
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
/// Represents a page table, contains a pointer to the physical page containing the table entries
struct PageTable(pub *mut usize);

impl PageTable {
    #[inline]
    pub fn new() -> Option<Self> {
        allocate_page().map(|page| {
            set_memory(page, PAGE_SIZE, 0);
            Self(page as *mut usize)
        })
    }

    pub fn set(&mut self, index: usize, entry: PageTableEntry) {
        if index >= 512 {
            panic!("page_table_set");
        } else {
            let ptr = ((self.0 as usize) + index * 8) as *mut usize;
            unsafe {
                *ptr = entry.0;
            }
        }
    }

    pub fn lookup(&self, index: usize) -> Option<PageTableEntry> {
        if index >= 512 {
            panic!("page_table_lookup");
        } else {
            let potential_entry = ((self.0 as usize) + index * 8) as *mut usize;
            unsafe {
                if (*potential_entry & PageTableEntry::FLAG_VALID) != 0 {
                    Some(PageTableEntry(*potential_entry))
                } else {
                    None
                }
            }
        }
    }

    pub fn get_ref(&self, index: usize) -> *mut PageTableEntry {
        if index > 511 {
            panic!("page_table_lookup");
        } else {
            ((self.0 as usize) + index * 8) as *mut PageTableEntry
        }
    }

    pub fn dump(&self) {
        for idx in 0..512 {
            if let Some(a) = self.lookup(idx) {
                println!(
                    "{idx} -> {:#x} : {:#b}",
                    a.extract_physical_page_number(),
                    a.extract_flags()
                );
                for idx2 in 0..512 {
                    if let Some(b) = a.as_table().lookup(idx2) {
                        println!(
                            "   {idx2} -> {:#x} : {:#b}",
                            b.extract_physical_page_number(),
                            b.extract_flags()
                        );
                        for idx3 in 0..512 {
                            if let Some(c) = b.as_table().lookup(idx3) {
                                println!(
                                    "      {idx3} -> {:#x} : {:#b}",
                                    c.extract_physical_page_number(),
                                    c.extract_flags()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn walk(&self, va: VirtualAddr, alloc: bool) -> Option<*mut PageTableEntry> {
        let mut current_table = self.clone();

        for level in [2, 1] {
            let idx = va.extract_index_at_level(level);
            let tbl = current_table.lookup(idx).map(|pte| pte.as_table());
            if let Some(tbl) = tbl {
                current_table = tbl;
            } else if alloc {
                let (pte, new_table) =
                    PageTableEntry::allocate_as_new_table(PageTableEntry::FLAG_VALID)?;
                current_table.set(idx, pte);
                current_table = new_table;
            } else {
                return None;
            }
        }

        let idx = va.extract_index_at_level(0);
        Some(current_table.get_ref(idx))
    }

    pub fn kvm_map(
        &mut self,
        virtual_addr: usize,
        size: usize,
        physical_address: usize,
        perm: usize,
    ) {
        println!(
            "kvm_map: {virtual_addr:#x}-{:#x} -> {physical_address:#x} ({size:#x})",
            virtual_addr + size
        );
        self.map_pages(virtual_addr, size, physical_address, perm)
            .expect("Uh oh");
    }

    pub fn map_pages(
        &mut self,
        virtual_addr: usize,
        size: usize,
        mut physical_address: usize,
        perm: usize,
    ) -> Option<()> {
        if virtual_addr % PAGE_SIZE != 0 {
            panic!("map_pages: va not aligned");
        }

        if size % PAGE_SIZE != 0 || size == 0 {
            panic!("map_pages: size not aligned or is 0")
        }

        let mut a = virtual_addr;
        let last = virtual_addr + size - PAGE_SIZE;

        loop {
            let entry = self.walk(VirtualAddr(a), true)?;
            unsafe {
                if ((*entry).extract_flags() & PageTableEntry::FLAG_VALID) != 0 {
                    panic!("map_pages: remap");
                }
                *entry = PageTableEntry::new(physical_address, perm | PageTableEntry::FLAG_VALID);
                if a == last {
                    break;
                }
                a += PAGE_SIZE;
                physical_address += PAGE_SIZE;
            }
        }

        Some(())
    }
}

const TRAMPOLINE: usize = MAX_VIRTUAL_ADDRESS - PAGE_SIZE;

extern "system" {
    static text_end: u8;
}

fn kvm_make() -> Option<PageTable> {
    let mut kernel_table = PageTable::new().expect("kvm_init: page alloc failed");

    const RW: usize = PageTableEntry::FLAG_READ | PageTableEntry::FLAG_WRITE;

    kernel_table.kvm_map(UART_LOC0, PAGE_SIZE, UART_LOC0, RW);

    kernel_table.kvm_map(VIRTIO0, PAGE_SIZE, VIRTIO0, RW);

    kernel_table.kvm_map(PLIC, 0x4000000, PLIC, RW);

    let etext = unsafe { &text_end as *const u8 as usize };

    kernel_table.kvm_map(
        KERNEL_START,
        etext - KERNEL_START,
        KERNEL_START,
        PageTableEntry::FLAG_EXEC | PageTableEntry::FLAG_READ,
    );

    kernel_table.kvm_map(etext, PHYS_STOP - etext, etext, RW);

    //kernel_table.kvm_map(TRAMPOLINE, PAGE_SIZE, TRAMPOLINE, PageTableEntry::FLAG_EXEC | PageTableEntry::FLAG_READ);

    Some(kernel_table)
}

static mut KERNEL_TABLE: Option<PageTable> = None;

pub fn kvm_init_base() {
    unsafe {
        KERNEL_TABLE = Some(kvm_make().expect("kvm_init: table"));
    }
}

pub fn kvm_init_hart() {
    unsafe {
        // Ensure page table memory has been cleared
        riscv::asm::sfence_vma(0, 0);

        riscv::register::satp::write((8_usize << 60) | (KERNEL_TABLE.unwrap().0 as usize >> 12));

        // Flush stale entries
        riscv::asm::sfence_vma(0, 0);
    }
}
