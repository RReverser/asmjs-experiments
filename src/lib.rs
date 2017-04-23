#![feature(link_args)]
#![feature(core_intrinsics)]

macro_rules! cstr {
    // Note: we use raw casts and not .as_ptr() as indirection
    // breaks emscripten_asm_* functions in debug mode
    ($($val:expr),+) => (concat!($($val,)+ "\0") as *const str as *const u8)
}

mod types;

#[macro_use]
pub mod value;

pub mod functions;
pub mod structs;
