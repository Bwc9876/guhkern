// RAW POINTER W/ OPTION

// This module is responsible for allocating memory to user processes,
// kernel stacks, page-table pages, and pipe buffers.
// We'll see what those are later but for now, let's focus on the memory allocation.

use core::{
    alloc::GlobalAlloc,
    arch::asm,
    ptr::{addr_of_mut, null_mut},
};

use crate::{
    consts::{KERNEL_START, PHYS_STOP},
    println,
    spinlock::Spinlock,
};

// This is the size of each page in memory
pub const PAGE_SIZE: usize = 4096;

// This is the maximum virtual address that we can allocate memory to
pub const MAX_VIRTUAL_ADDRESS: usize = 1 << (9 + 9 + 9 + 12 - 1); // 2^(9+9+9+12-1) = 2^39

#[inline]
pub fn g_kernel_end() -> usize {
    let a: usize;
    unsafe {
        asm!("la {}, kernel_end", out(reg) a);
    }
    a
}

#[repr(transparent)]
struct Run {
    next: Option<*mut Run>,
}

struct KernelMemory {
    lock: Option<Spinlock>,
    free: Option<*mut Run>,
}

static mut KERNEL_MEMORY: KernelMemory = KernelMemory {
    lock: None,
    free: None,
};

// Initialize the kernel memory allocator's spinlock and
// free list of memory chunks
pub fn kinit() {
    unsafe {
        let lock = &mut KERNEL_MEMORY.lock;
        Spinlock::init(lock);
    }
    // Free all memory from the end of the kernel to the end of physical memory
    // This takes care of setting up all pages of memory to be free
    // Then we have them available in KernelMemory::free which is a linked list of free pages
    let kernel_end = g_kernel_end();
    free_range(kernel_end, PHYS_STOP);
}

#[inline]
/// Given a size of memory, get the next page size up (e.g. 4097 -> 8192, 4096 -> 4096, 4 -> 4096)
const fn get_page_round_up(n: usize) -> usize {
    (n + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

#[inline]
/// Given a size of memory, get the next page size down (e.g. 4097 -> 4096, 4096 -> 4096, 4 -> 0)
const fn get_page_round_down(n: usize) -> usize {
    n & !(PAGE_SIZE - 1)
}

// Free a range of pages of memory given a start and end physical address
fn free_range(page_start: usize, page_end: usize) {
    let mut page = get_page_round_down(page_start);
    println!("free_range: {:#x} to {:#x}", page, page_end);
    let mut c = 0;
    while page + PAGE_SIZE <= page_end {
        if c % 5000 == 0 {
            println!("free_page: {:#x}", page);
        }
        free_page(page as *mut u8);
        page += PAGE_SIZE;
        c += 1;
    }
    println!("Alloc'd {c} pages");
}

// Utility to set the memory at a given address from start to start + size to a given value
pub fn set_memory(start: *mut u8, size: usize, value: u8) -> *mut u8 {
    //println!("set_memory: {:#x}: +{:#x} to {:#x}", start as usize, size, value);
    if (start as usize) < g_kernel_end() || (start as usize) >= PHYS_STOP {
        panic!("set_mem");
    }

    for i in 0..size {
        unsafe {
            // Here we're using the add method on the pointer to get the memory at the given index
            // and then we're writing the value to that memory
            start.add(i).write(value);
        }
    }
    start
}

// Free a page of memory
// This takes a pointer gotten from allocate_page and adds it to the free list
pub fn free_page(page: *mut u8) {
    let page_num = page as usize;
    // Some sanity checks to make sure we're not freeing memory we shouldn't, would be bad
    // 1. The page number is a multiple of the page size
    // 2. The page number is greater than the end of the kernel memory (otherwise we're freeing kernel memory)
    // 3. The page number is less than the end of physical memory (otherwise we're freeing memory we don't have)
    if page_num % PAGE_SIZE != 0 || page_num < g_kernel_end() || page_num >= PHYS_STOP {
        panic!("free_page");
    }

    // Initialize a new Run struct
    let run: *mut Run;

    // Set the memory of the page to 1, this is just to make sure we're not using uninitialized memory
    // and dangling pointers
    set_memory(page, PAGE_SIZE, b'U');

    // Now we set the run to point to the page
    run = page as *mut Run;

    unsafe {
        // Now we lock the kernel memory allocator's spinlock and
        // add the page to the free list, setting the next page to the current free list's head
        let lock = &mut KERNEL_MEMORY.lock;
        let guard = Spinlock::acquire((*lock).as_mut());
        (*run).next = KERNEL_MEMORY.free;
        KERNEL_MEMORY.free = Some(run);
        drop(guard);
    }
}

// Allocate a new page of memory
// this will return a pointer to the newly allocated page
pub fn allocate_page() -> Option<*mut u8> {
    // We need to pop the head off the free list,
    // so we lock the kernel memory allocator's spinlock
    // Grab the head of the free list and replace it with the next page
    // If there is no head, we're out of memory
    unsafe {
        let lock = addr_of_mut!(KERNEL_MEMORY.lock);
        let guard = Spinlock::acquire((*lock).as_mut());
        let run = KERNEL_MEMORY.free.take();
        if let Some(run) = run {
            let page = run as *mut u8;
            KERNEL_MEMORY.free = (*run).next;
            drop(guard);
            set_memory(page, PAGE_SIZE, 0);
            Some(page)
        } else {
            println!("boo-womp no more pages");
            None
        }
    }
}

struct GuhAlloc;

#[global_allocator]
static ALLOCATOR: GuhAlloc = GuhAlloc;

unsafe impl GlobalAlloc for GuhAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        println!("alloc: {layout:?}");
        allocate_page().unwrap_or(null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        println!("free: {layout:?}");
        free_page(ptr);
    }
}
