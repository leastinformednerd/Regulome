use x86_64::{
    structures::paging::{
        PageTable,
        RecursivePageTable,
        PageTableFlags
    },
    registers::control::Cr3,
    PhysAddr
};

pub enum RecursivePageTableCreationError {
    TooLarge,
    UnsafeFinalEntry,
    AlreadyUsed,
    Cr3ReadError,
    NotRecursive
}

impl RecursivePageTableCreationError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::TooLarge => "The index provided was too large (512 or over)",
            Self::UnsafeFinalEntry => "The index provided was 511, which is unsafe due to potential pointer overflows or UB",
            Self::AlreadyUsed => "The page table entry at the provided inddex is in use already",
            Self::Cr3ReadError => "For whatever reason it wasn't possible to read from Cr3",
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

    let page_table_ref =
        (VirtAddr::new_truncate(index << 39 | index << 30 | index << 21 | index << 12).as_u64() as *mut PageTable)
        .as_mut();
    
    RecursivePageTable::new(page_table_ref).unwrap_or(Err(RPTCE::NotRecursive))
}

#[cfg(test)]
mod tests {
    use super::*;
}
