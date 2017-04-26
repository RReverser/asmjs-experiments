#[macro_use] extern crate asmjs;

#[link_args = "--bind --js-library rustlib.js"]
extern {}

pub fn count_emval_handles() -> usize {
    unsafe {
        asmjs::value::emscripten_asm_const_int(cstr!("return count_emval_handles()")) as usize
    }
}
