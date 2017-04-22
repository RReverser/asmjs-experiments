use value::*;
use std::os::raw::c_void as void;
use std::intrinsics::type_name;
use std::ffi::CString;

extern {
    fn _embind_register_class(
        cls_type: TypeId,
        ptr_type: TypeId,
        const_ptr_type: TypeId,
        base_cls_type: TypeId, // 0
        get_actual_type_signature: CStr, // ii
        get_actual_type: extern fn (*const void) -> TypeId,
        upcast_signature: CStr, // v
        upcast: extern fn () -> (), // 0
        downcast_signature: CStr, // v
        downcast: extern fn () -> (),
        cls_name: CStr,
        destructor_signature: CStr, // vi
        destructor: extern fn (*mut void) -> ()
    );
}

pub fn register_class<T: 'static>() {
    extern fn get_actual_type<T: 'static>(arg: *const void) -> TypeId {
        unsafe {
            println!("reporting actual type of {:?} as {:?}", arg, type_id::<T>());
            type_id::<T>()
        }
    }

    extern fn noop() {
        println!("noop");
    }

    extern fn destructure<T: 'static>(arg: *mut void) {
        unsafe {
            println!("destructure {:?} as {:?}", arg, type_id::<T>());
            Box::from_raw(arg as *mut T);
        }
    }

    unsafe {
        _embind_register_class(
            type_id::<T>(),
            type_id::<*mut T>(),
            type_id::<*const T>(),
            0,
            cstr!("ii"),
            get_actual_type::<T>,
            cstr!("v"),
            noop,
            cstr!("v"),
            noop,
            CString::new(type_name::<T>()).unwrap().as_ptr(),
            cstr!("vi"),
            destructure::<T>
        )
    }
}

pub fn register_class_default_ctor<T: 'static + Default>() {
    extern fn invoker(f: fn () -> Box<void>) -> *mut void {
        let p = Box::into_raw(f());
        println!("constructed {:?}", p);
        p
    }

    #[allow(improper_ctypes)]
    extern {
        fn _embind_register_class_constructor(
            cls_type: TypeId,
            arg_count: usize,
            arg_types: *const TypeId,
            invoker_signature: CStr,
            invoker: extern fn (fn () -> Box<void>) -> *mut void,
            ctor: fn () -> Box<void>
        );
    }

    unsafe {
        let arg_types = [type_id::<*mut T>()];
        _embind_register_class_constructor(
            type_id::<T>(),
            arg_types.len(),
            arg_types.as_ptr(),
            cstr!("ii"),
            invoker,
            ::std::mem::transmute(Box::<T>::default as fn () -> Box<_>)
        )
    }
}

#[cfg(test)]
mod tests {
    use value::*;
    use structs::*;

    #[repr(C)]
    #[derive(Debug)]
    pub struct MyStruct {
        x: u32
    }

    impl Default for MyStruct {
        fn default() -> MyStruct {
            println!("constructed MyStruct");
            MyStruct {
                x: 0
            }
        }
    }

    impl Drop for MyStruct {
        fn drop(&mut self) {
            println!("dropped MyStruct");
        }
    }

    impl Into<Val> for MyStruct {
        fn into(self) -> Val {
            Box::new(self).into()
        }
    }

    impl Into<Val> for Box<MyStruct> {
        fn into(self) -> Val {
            unsafe {
                Val::new(&Box::into_raw(self))
            }
        }
    }

    #[test]
    fn test_works() {
        register_class::<MyStruct>();
        register_class_default_ctor::<MyStruct>();

        unsafe {
            extern {
                fn _embind_register_class_property(
                    cls_type: TypeId,
                    field_name: CStr,
                    getter_ret_type: TypeId,
                    getter_signature: CStr,
                    getter: extern fn (ctx: u32, ptr: *const MyStruct) -> u32,
                    getter_context: u32,
                    setter_arg_type: TypeId,
                    setter_signature: CStr,
                    setter: extern fn (ctx: u32, ptr: *mut MyStruct, value: u32) -> (),
                    setter_context: u32
                );
            }

            extern fn getter(_ctx: u32, ptr: *const MyStruct) -> u32 {
                unsafe { (*ptr).x }
            }

            extern fn setter(_ctx: u32, ptr: *mut MyStruct, value: u32) {
                unsafe { (*ptr).x = value }
            }

            _embind_register_class_property(
                type_id::<MyStruct>(),
                cstr!("x"),
                type_id::<u32>(),
                cstr!("iii"),
                getter,
                0,
                type_id::<u32>(),
                cstr!("viii"),
                setter,
                0
            );
        }

        assert_eq!(count_emval_handles(), 0);

        let global = Val::global();

        assert_eq!(count_emval_handles(), 1);

        global.set("mystruct", MyStruct { x: 42 });

        assert_eq!(count_emval_handles(), 1);

        assert_eq!(u32::from(global.get("mystruct").get("x")), 42);

        assert_eq!(count_emval_handles(), 1);
    }
}