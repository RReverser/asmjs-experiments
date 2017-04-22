#[cfg(test)]
mod tests {
    use value::*;

    #[test]
    fn test_register_rust_fn() {
        #[allow(improper_ctypes)]
        extern {
            fn _embind_register_function(
                name: CStr,
                arg_count: usize,
                arg_types: *const TypeId,
                signature: CStr,
                invoker: extern fn (f: fn (u32, u32) -> u32, a: u32, b: u32) -> u32,
                ctor: fn (u32, u32) -> u32
            );
        }

        unsafe {
            extern fn invoker(f: fn (u32, u32) -> u32, a: u32, b: u32) -> u32 {
                f(a, b)
            }

            fn adder(a: u32, b: u32) -> u32 {
                a + b
            }

            _embind_register_function(
                cstr!("fast_add"),
                3,
                [type_id::<u32>(), type_id::<u32>(), type_id::<u32>()].as_ptr(),
                cstr!("iiii"),
                invoker,
                adder
            );
        }
    }
}
