#![no_std]

use elf::{ElfBytes, endian::AnyEndian};

use x86_64::structures::paging::{FrameAllocator, RecursivePageTable, page::Size4KiB};

/// Load an elf file in memory (the parameter data) into the correct virtual memory location and
/// return a VirtAddr that points to the entry point of the file
/// # Arguments
/// * `data` - the full file loaded in memory
///
/// * `allocator` - an allocator to get frames for page table changes and to copy the segments to
fn load<Allocator: FrameAllocator<Size4KiB>>(data: &[u8], allocator: &Allocator, page_table: &mut RecursivePageTable, preferred_load_location: u64) -> Result<x86_64::VirtAddr, &'static str> {
    let elf_bytes = match ElfBytes::<AnyEndian>::minimal_parse(data) {
        Ok(elf_bytes) => elf_bytes,
        Err(_) => return Err("Failed to parse the elf header")
    };

    let elf_common_data = match elf_bytes.find_common_data() {
        Ok(common_data) => common_data,
        Err(_) => return Err("Failed to find common elf data")
    };

    if let Some(symtab) = elf_common_data.sym_tab {
        
    }

    return Err("This managed to make it this far without returning earlier :/")
}
