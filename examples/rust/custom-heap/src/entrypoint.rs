//! Program entrypoint

#![cfg(not(feature = "no-entrypoint"))]

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

#[cfg(target_os = "solana")]
mod custom_allocator {
    use core::mem::align_of;
    use linked_list_allocator::Heap;
    use solana_program::entrypoint::HEAP_LENGTH;
    use solana_program::msg;
    use static_assertions::const_assert_eq;
    use std::alloc::Layout;
    use std::mem::size_of;
    use std::ptr::NonNull;

    const HEAP_START_ADDRESS: usize = 0x300000000;
    const_assert_eq!(HEAP_START_ADDRESS % align_of::<Heap>(), 0);

    const HEAP_PTR: *mut Heap = HEAP_START_ADDRESS as *mut Heap;

    fn heap() -> &'static mut Heap {
        // This is legal since all-zero is a valid `Heap`-struct representation
        let heap = unsafe { &mut *HEAP_PTR };

        if heap.bottom().is_null() {
            let start = (HEAP_START_ADDRESS + size_of::<Heap>()) as *mut u8;
            let size = HEAP_LENGTH - size_of::<Heap>();
            unsafe { heap.init(start, size) };
        }

        heap
    }

    pub struct SolanaAllocator;

    unsafe impl std::alloc::GlobalAlloc for SolanaAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            msg!("alloc");

            #[allow(clippy::option_if_let_else)]
            if let Ok(non_null) = heap().allocate_first_fit(layout) {
                non_null.as_ptr()
            } else {
                solana_program::log::sol_log("EVM Allocator out of memory");
                std::ptr::null_mut()
            }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            msg!("dealloc");

            heap().deallocate(NonNull::new_unchecked(ptr), layout);
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            msg!("alloc_zeroed");

            let ptr = self.alloc(layout);

            if !ptr.is_null() {
                #[cfg(target_os = "solana")]
                solana_program::syscalls::sol_memset_(ptr, 0, layout.size() as u64);
                #[cfg(not(target_os = "solana"))]
                std::ptr::write_bytes(ptr, 0, layout.size());
            }

            ptr
        }

        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            msg!("realloc");

            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_ptr = self.alloc(new_layout);

            if !new_ptr.is_null() {
                let copy_bytes = std::cmp::min(layout.size(), new_size);

                #[cfg(target_os = "solana")]
                solana_program::syscalls::sol_memcpy_(new_ptr, ptr, copy_bytes as u64);
                #[cfg(not(target_os = "solana"))]
                std::ptr::copy_nonoverlapping(ptr, new_ptr, copy_bytes);

                self.dealloc(ptr, layout);
            }

            new_ptr
        }
    }

    #[global_allocator]
    static GLOBAL: SolanaAllocator = SolanaAllocator;
}

solana_program::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    crate::processor::process_instruction(program_id, accounts, instruction_data)
}
