use x86_64::{
    structures::paging::{
        PageTable,
        RecursivePageTable,
        PageTableFlags
    },
    registers::control::Cr3,
    PhysAddr, VirtAddr
};

const PAGE_SIZE: usize = 4096;
const FRAME_SIZE: usize = 4096;

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

#[derive(Clone, Copy)]
struct PhysMemGaps {
    start_addr: u64,
    page_count: u32
}

impl PhysMemGaps {
    fn new() -> Self {
        Self {
            start_addr: 0,
            page_count: 0
        }
    }
}

/// A struct to manage physical memory that lives on the stack (i.e *it's* not heap allocated)
/// This is used to bootstrap an allocator that can use allocated memory
///
/// The generic parameter `N` determines the max size of the array that stores tracking infomation (12
/// bytes). i.e `N` = 100 means that the array will be 12*100 = 1200 bytes big
///
/// It isn't good but who cares it needs to exist for about half of a second
pub struct NonAllocatingStackFrameManager<const N: usize> {
    head_ind: usize,
    gaps: [PhysMemGaps; N]
}

impl<const N: usize> NonAllocatingStackFrameManager<N> {
    /// Create a new NonAllocatingStackFrameManager
    ///
    /// # Arguments
    /// * `memory_size` - The number of bytes in the system (or at least managed by the alloactor)
    pub fn new(memory_size: u64 = 33_418_117_120) -> Self {
        let mut ret = Self {
            head_ind: 0,
            gaps: [PhysMemGaps::new(); N]
        };
        
        ret.gaps[0] = PhysMemGaps {
            start_addr: 0,
            page_count: memory_size / PAGE_SIZE as u64
        };

        ret
    }

    /// Allocates `size` bytes of contiguous memory somewhere and returns a pointer to the start of that block
    /// Is really bad and should be blown up and the author killed but it gets called like 4 times.
    pub fn allocate(&mut self, size: u64) -> u64 {
        let page_count = size / PAGE_SIZE + if (size % PAGE_SIZE) != 0 {1} else {0};
            
        let flag = true;

        while (flag) {
            if self.gaps[head].page_count != 0 && self.gaps[head].page_count > page_count {
                return {
                    // Why did I do it like this in a returning block?
                    // Oh well can't be bothered doing it normally
                    let addr = self.gaps[head].start_addr;
                    self.gaps[head].page_count -= page_count;
                    self.gaps[head].start_addr += page_count * PAGE_SIZE as u64;
                    addr
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
