#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

mod code_hashes;
use code_hashes::CODE_HASH_SHARED_LIB;
use ckb_std::dynamic_loading::{CKBDLContext, Symbol};

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
// use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    entry,
    default_alloc,
    error::SysError,
};

entry!(entry);
default_alloc!();

/// Program entry
fn entry() -> i8 {
    // Call main function and return error code
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

/// Error
#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

fn main() -> Result<(), Error> {
    // Create a DL context with 64K buffer.
    let mut context = CKBDLContext::<[u8; 64 * 1024]>::new();
    // Load library
    let lib = context.load(&CODE_HASH_SHARED_LIB).expect("load shared lib");
    
    // get symbols
    unsafe {
        type Plus42 = unsafe extern "C" fn(n: usize) -> usize;
        let plus_42: Symbol<Plus42> = lib.get(b"plus_42").expect("find plus_42");
        assert_eq!(plus_42(13), 13 + 42);
    
        type Foo = unsafe extern "C" fn() -> *const u8;
        let foo: Symbol<Foo> = lib.get(b"foo").expect("find foo");
        let ptr = foo();
        let mut buf = [0u8; 3];
        buf.as_mut_ptr().copy_from(ptr, buf.len());
        assert_eq!(&buf[..], b"foo");
    }
    Ok(())
}
