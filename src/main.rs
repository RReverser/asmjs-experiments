#![feature(link_args)]
#![feature(core_intrinsics)]

use std::ffi::CString;
use std::os::raw::c_void as void;
use std::intrinsics;

mod value;

use value::*;

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
            Val::new(Box::into_raw(self))
        }
    }
}

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

    fn _embind_register_class_constructor(
        cls_type: TypeId,
        arg_count: usize,
        arg_types: *const TypeId,
        invoker_signature: CStr,
        invoker: extern fn (fn () -> Box<void>) -> *mut void,
        ctor: fn () -> Box<void>
    );
}

fn register_class<T: 'static>() {
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
            b"ii\0".as_ptr(),
            get_actual_type::<T>,
            b"v\0".as_ptr(),
            noop,
            b"v\0".as_ptr(),
            noop,
            CString::new(intrinsics::type_name::<T>()).unwrap().as_ptr(),
            b"vi\0".as_ptr(),
            destructure::<T>
        )
    }
}

fn register_class_default_ctor<T: 'static + Default>() {
    extern fn invoker(f: fn () -> Box<void>) -> *mut void {
        let p = Box::into_raw(f());
        println!("constructed {:?}", p);
        p
    }

    unsafe {
        let arg_types = [type_id::<*mut T>()];
        _embind_register_class_constructor(
            type_id::<T>(),
            arg_types.len(),
            arg_types.as_ptr(),
            b"ii\0".as_ptr(),
            invoker,
            ::std::mem::transmute(Box::<T>::default as fn () -> Box<_>)
        )
    }
}

fn main() {
    use value::Val;

    register_class::<MyStruct>();
    register_class_default_ctor::<MyStruct>();

    let global = Val::global();

    global.set("str", "hello, world");
    global.set("flag", true);
    global.set("mystruct", MyStruct { x: 42 });

    println!("{}", usize::from(global.get("navigator").get("plugins").get(Val::cstring(b"length\0" as _))));
}
