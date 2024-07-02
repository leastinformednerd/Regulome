#![no_std]

use elf::{ElfBytes, endian::AnyEndian, parse::ParseError, CommonElfData};

use x86_64::structures::paging::{
    RecursivePageTable
};

use x86_64::VirtAddr;

use mem::FrameAllocator;

pub enum LoadLocation {
    Any,
    Exactly(VirtAddr),
    LessThan(VirtAddr), // Probably going to be used mostly for sticking userspace programs in
                        // lower half
    GreaterThan(VirtAddr) // Probably going to be used mostly for sticking kernel in higher half
}

pub enum ElfLoadError {
    ElfHeaderParseError(ParseError),
    IncorrectType,
    WrongInstructionSet,
    CommonDataNotFound,
    MissingSymTab

    // A panic handler is not guaranteed to exist (and be pretty) so I'm going to leave it up to 
    // the caller to deal with this
    NotImplemented
}

impl From<ParseError> for ElfLoadError {
    fn from(value: ParseError) -> Self {
        Self::ElfHeaderParseError(value)
    }
}

// Medium priority. Would be good, but like I can just statically link everything to begin with,
// and any PIE when compiled for Regulome can just be PIC instead
fn load_shared_library<Allocator: FrameAllocator>(data: &[u8], allocator: &Allocator, page_table: &mut RecursivePageTable, load_location: LoadLocation, elf_bytes: ElfBytes) -> Result<x86_64::VirtAddr, ElfLoadError> {
    return Err(NotImplemented)
}

// Low priority, I think I get by with just PIC for a bit
fn load_executable<Allocator: FrameAllocator>(data: &[u8], allocator: &Allocator, page_table: &mut RecursivePageTable, load_location: LoadLocation, elf_bytes: ElfBytes) -> Result<x86_64::VirtAddr, ElfLoadError> {
    return Err(NotImplemented)
}

// High priroity, the kernel is one of these
fn load_relocatable<Allocator: FrameAllocator>(data: &[u8], allocator: &Allocator, page_table: &mut RecursivePageTable, load_location: LoadLocation, elf_bytes: ElfBytes) -> Result<x86_64::VirtAddr, ElfLoadError> {
    
}


/// return a VirtAddr that points to the entry point of the file
/// # Arguments
/// * `data` - the full file loaded in memory
///
/// * `allocator` - an allocator to get frames for page table changes and to copy the segments to
///
/// * `page_table` - the page table for the data to be mapped into (must be recursive)
///
/// * `load_location` - a hint to the location in virtual memory
///
/// #
pub fn load<Allocator: FrameAllocator>(data: &[u8], allocator: &Allocator, page_table: &mut RecursivePageTable, load_location: LoadLocation) -> Result<x86_64::VirtAddr, ElfLoadError> {
    let elf_bytes = match ElfBytes::<AnyEndian>::minimal_parse(data) {
        Ok(elf_bytes) => elf_bytes,
        Err(err) => return Err(ElfLoadError::ElfHeaderParseFail(err))
    };

    if elf_bytes.ehdr.e_machine != elf::abi::EM_X86_64 {
        return Err(WrongInstructionSet)
    }

    use elf::abi::{
        ET_DYN,
        ET_EXEC,
        ET_REL
    };

    

    return match elf_bytes.ehdr.e_type {
          ET_DYN => load_shared_library(data, allocator, page_table, load_location, elf_bytes),
          ET_EXEC => load_executable(data, allocator, page_table, load_location, elf_bytes),
          ET_REL => load_relocatable(data, allocator, page_table, load_location, elf_bytes),
          _ => Err(IncorrectType)
    }
}
