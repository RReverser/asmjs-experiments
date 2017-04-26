#![feature(link_args)]

include!("utils/header.rs");

use asmjs::value::*;
use asmjs::types::*;
use asmjs::structs::*;

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
            Val::new_simple(&Box::into_raw(self))
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