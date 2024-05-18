use x86_64::{
    structures::paging::{
        PageTable,
        RecursivePageTable,
        PageTableFlags
    },
    registers::control::Cr3,
    PhysAddr, VirtAddr
};

pub const PAGE_SIZE: usize = 4096;
pub const FRAME_SIZE: usize = 4096;

pub enum RecursivePageTableCreationError {
    TooLarge,
    UnsafeFinalEntry,
    AlreadyUsed,
    Cr3ReadError,
    WrongAddr,
    NotRecursive
}

impl RecursivePageTableCreationError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::TooLarge => "The index provided was too large (512 or over)",
            Self::UnsafeFinalEntry => "The index provided was 511, which is unsafe due to potential pointer overflows or UB",
            Self::AlreadyUsed => "The page table entry at the provided inddex is in use already",
            Self::Cr3ReadError => "For whatever reason it wasn't possible to read from Cr3",
            Self::WrongAddr => "Somehow the wrong address fell through for the page table",
            Self::NotRecursive => "I'm a dumbass and for whatever reason it's not recursive"
        }
    }
}

/// Tries to add a recursive entry to the active page table
///
/// This should probably be the first thing that happens when memory is taken control of from UEFI
/// when the memory is still identity mapped
///
/// # Arguments
///
/// * `index` - the index of entry in the page table that will point to itself if successful. Must
/// be less than 511 (511 is the last index, and can't be used due to overflox / UB issues).
pub fn create_recursive_page_table(index: usize) -> Result<RecursivePageTable<'static>, RecursivePageTableCreationError>{
    use RecursivePageTableCreationError as RPTCE;
    if index >= 512 {
        return Err(RPTCE::TooLarge)
    }
    
    if index == 511 {
        return Err(RPTCE::UnsafeFinalEntry)
    }

    let (current_page_table, addr) = unsafe {
        let reg = Cr3::read().0.start_address().as_u64(); 
        (
            match (reg as *mut PageTable).as_mut() {
                Some(refer) => refer,
                None => return Err(RPTCE::Cr3ReadError)
            },
            reg
        )
    };
    
    {
        let entry = &mut current_page_table[index];
        if entry.is_unused() {
            entry.set_addr(
                PhysAddr::new(addr),
                PageTableFlags::empty() |
                PageTableFlags::PRESENT |
                PageTableFlags::GLOBAL |
                PageTableFlags::NO_EXECUTE
            );
        } else {
            return Err(RPTCE::AlreadyUsed)
        }
    }

    let page_table_ref = if let Some(table) = unsafe {
        let index64 = index as u64; 
        (VirtAddr::new_truncate(index64 << 39 | index64 << 30 | index64 << 21 | index64 << 12).as_u64() as *mut PageTable)
        .as_mut()
    } {
        table
    } else {return Err(RPTCE::WrongAddr)};
    
    RecursivePageTable::new(page_table_ref).map_err(|_| RPTCE::NotRecursive)
}

trait FrameAllocator {
    /// Must allocate `size` / `FRAME_SIZE` contiguous frames of memory and return the physical
    /// address at the start of that series, or an error if there isn't enough available memory
    fn allocate(size: usize) -> Result<usize, ()>;

    fn deallocate(start_addr: usize) -> ();
}

struct BootstrapFrameInfo {
    // The largest bit (1<<63) of `start_addr` is used to track if the frame is taken
    start_addr: u64,
}

/// A new, actually useful way of allocating physical memory to bootstrap the OS's memory
/// management.
/// Basically just hands out frames in massive bundles (or you can think of it as massive frames)
pub struct BootstrapFrameManager {
    frame_info: [FrameInfo; 4096]
}

impl BootstrapFrameManager {
    const BLOCK_SIZE: u64 = 33_418_117_120u64 / 4096;
    pub const FRAMES_PER_BLOCKS: u64 = Self::BLOCK_SIZE / FRAME_SIZE as u64;

    /// Creates a new BootstrapFrameManager 
    fn new() -> Self {
        BootstrapFrameManager {
            frame_info: core::array::from_fn(|index| 
                FrameInfo {
                    start_addr: index as u64 * Self::BLOCK_SIZE,
                }
            )
        }
    }

    /// From an iterator of (start address, size in bytes) initialises the frame manager to know
    /// that those frames are taken. It's not good or smart but it doesn't need to be.
    fn initialise_from_existing_mem_map<I>(mut target: Self, existing: I) -> Self
    where I: Iterator<Item = (u64, u64)> {
        for entry in existing {
            let start_index = entry.0 / Self::BLOCK_SIZE; 
            let num_blocks = entry.1 / Self::BLOCK_SIZE + if entry.1 % Self::BLOCK_SIZE != 0 {1} else {0};
            
            let index = start_index;
            while index < start_index + num_blocks {
                target.frame_info[index as usize].start_addr = target.frame_info[index as usize].start_addr | 1<<63
            }
        }

        return target
    }

    /// Constructs a new frame manager and initialises to be vaguely aware of existing allocated frames
    pub fn new_initialised<I>(existing: I) -> Self
    where I: Iterator<Item = (u64, u64)> {
        return Self::initialise_from_existing_mem_map(Self::new(), existing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
