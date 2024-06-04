//! Program entrypoint

use core::mem::size_of;
use linked_list_allocator::Heap;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
use static_assertions::const_assert_eq;
use std::mem::align_of;

const PROGRAM_INPUT_PARAMETERS_START_ADDRESS: usize = 0x400000000;

const FIRST_ACCOUNT_DATA_OFFSET: usize =
    /* number of accounts */
    size_of::<u64>() +
        /* duplication marker */ size_of::<u8>() +
        /* is_signer */ size_of::<u8>() +
        /* is_writable */ size_of::<u8>() +
        /* executable */ size_of::<u8>() +
        /* original_data_len */ size_of::<u32>() +
        /* key */ size_of::<Pubkey>() +
        /* owner */ size_of::<Pubkey>() +
        /* lamports */ size_of::<u64>() +
        /* data_len */ size_of::<u64>();
const_assert_eq!(FIRST_ACCOUNT_DATA_OFFSET, 96);

const FIRST_ACCOUNT_DATA_ADDRESS: usize =
    PROGRAM_INPUT_PARAMETERS_START_ADDRESS + FIRST_ACCOUNT_DATA_OFFSET;
const_assert_eq!(FIRST_ACCOUNT_DATA_ADDRESS, 0x400000060);

const STATE_PTR_ADDRESS: usize = FIRST_ACCOUNT_DATA_ADDRESS;
const_assert_eq!(STATE_PTR_ADDRESS, 0x400000060);
const_assert_eq!(STATE_PTR_ADDRESS % align_of::<*mut Vec<u8>>(), 0);

const STATE_PTR: *mut *mut Vec<u8> = STATE_PTR_ADDRESS as *mut *mut Vec<u8>;

const HEAP_START_ADDRESS: usize = FIRST_ACCOUNT_DATA_ADDRESS + size_of::<usize>();
const_assert_eq!(HEAP_START_ADDRESS, 0x400000068);
const_assert_eq!(HEAP_START_ADDRESS % align_of::<Heap>(), 0);

const HEAP_LENGTH: usize = 10 * 1024 * 1024 - size_of::<usize>();

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

#[cfg(target_os = "solana")]
mod custom_allocator {
    use super::*;

    use solana_program::msg;
    use std::alloc::Layout;
    use std::ptr::NonNull;

    pub struct SolanaAllocator;

    unsafe impl std::alloc::GlobalAlloc for SolanaAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            msg!("alloc");

            #[allow(clippy::option_if_let_else)]
            if let Ok(non_null) = heap().allocate_first_fit(layout) {
                non_null.as_ptr()
            } else {
                solana_program::log::sol_log("Allocator out of memory");
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

fn state_ptr() -> *mut Vec<u8> {
    unsafe { *STATE_PTR }
}

fn init_state(v: Vec<u8>) {
    unsafe {
        assert_eq!(state_ptr(), std::ptr::null_mut());
        *STATE_PTR = Box::into_raw(Box::new(v));
    }
}

fn state() -> &'static Vec<u8> {
    unsafe { &**STATE_PTR }
}

fn state_mut() -> &'static mut Vec<u8> {
    unsafe { &mut **STATE_PTR }
}

fn drop_state() {
    unsafe {
        drop(Box::from_raw(state_ptr()));
        *STATE_PTR = std::ptr::null_mut();
    }
}

solana_program::entrypoint!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Some(first_account_info) = accounts.first() {
        assert_eq!(
            first_account_info.data.borrow().as_ptr() as usize,
            FIRST_ACCOUNT_DATA_ADDRESS,
            "First account data should have a fixed address"
        )
    };

    msg!("hello from persistent heap {:?}", instruction_data);

    let i = instruction_data[0];

    if i == 0 {
        msg!("here_alloc");

        msg!("state_ptr: {:?}", state_ptr());

        init_state(vec![]);
        msg!("after_alloc");

        msg!("state_ptr: {:?}", state_ptr());
        msg!("state: {:?}", state());

        msg!("heap_stats: used={}, free={}", heap().used(), heap().free());

        return Ok(());
    }

    if i == 1 {
        msg!("here_dealloc");

        msg!("state: {:?}", state());
        msg!("state_ptr: {:?}", state_ptr());

        drop_state();
        msg!("after_dealloc");

        msg!("state_ptr: {:?}", state_ptr());

        msg!("heap_stats: used={}, free={}", heap().used(), heap().free());

        return Ok(());
    }

    msg!("here_use");
    msg!("state: {:?}", state());

    state_mut().push(instruction_data[0]);

    msg!("state_len: {:?}", state().len());
    msg!("state: {:?}", state());

    msg!("heap_stats: used={}, free={}", heap().used(), heap().free());

    Ok(())
}
