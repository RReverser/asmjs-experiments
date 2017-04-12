use std::ops::{Index, IndexMut};
use std::os::raw::c_void as void;
use std::intrinsics;
use std::mem::size_of;

const UNDEFINED: usize = 1;
const NULL: usize = 2;
const TRUE: usize = 3;
const FALSE: usize = 4;

pub type TypeId = i32;

pub fn type_id<T: ?Sized + 'static>() -> TypeId {
    unsafe {
        intrinsics::type_id::<T>() as _
    }
}

#[repr(C)]
struct EmvalStruct(void);

pub type Emval = *mut EmvalStruct;

pub type CStr = *const u8;

#[link_args = "--bind --js-library rustlib.js"]
extern {
    fn _embind_register_void(type_id: TypeId, name: CStr);
    fn _embind_register_bool(type_id: TypeId, name: CStr, size: usize, true_value: bool, false_value: bool);
    fn _embind_register_integer(type_id: TypeId, name: CStr, size: usize, minRange: i32, maxRange: u32);
    fn _embind_register_float(type_id: TypeId, name: CStr, size: usize);
    fn _embind_register_rust_string(type_id: TypeId, name: CStr);

    fn _emval_incref(value: Emval);
    fn _emval_decref(value: Emval);

    fn _emval_take_value(type_id: TypeId, ptr: *const void) -> Emval;
    fn _emval_new_array() -> Emval;
    fn _emval_new_object() -> Emval;
    fn _emval_new_cstring(s: CStr) -> Emval;

    fn _emval_get_global(name: CStr) -> Emval;
    fn _emval_get_property(obj: Emval, prop: Emval) -> Emval;
    fn _emval_set_property(obj: Emval, prop: Emval, value: Emval);
}

pub struct Val(Emval);

impl Val {
    pub fn register() {
        unsafe {
            _embind_register_void(type_id::<()>(), b"()\0".as_ptr());
            _embind_register_void(type_id::<void>(), b"std::os::raw::c_void\0".as_ptr());

            _embind_register_bool(type_id::<bool>(), b"bool\0".as_ptr(), size_of::<bool>(), false, true);

            macro_rules! register_int {
                ($name:ident) => (_embind_register_integer(type_id::<$name>(), concat!(stringify!($name), "\0").as_ptr(), size_of::<$name>(), ::std::$name::MIN as _, ::std::$name::MAX as _))
            }

            register_int!(i8);
            register_int!(u8);
            register_int!(i16);
            register_int!(u16);
            register_int!(i32);
            register_int!(u32);

            macro_rules! register_float {
                ($name:ident) => (_embind_register_float(type_id::<$name>(), concat!(stringify!($name), "\0").as_ptr(), size_of::<$name>()))
            }

            register_float!(f32);
            register_float!(f64);

            _embind_register_rust_string(type_id::<&str>(), b"&str\0".as_ptr());
        }
    }

    pub fn new<T: 'static>(value: &T) -> Self {
        Val(unsafe {
            _emval_take_value(type_id::<T>(), value as *const T as *const void)
        })
    }

    pub fn array() -> Self {
        Val(unsafe {
            _emval_new_array()
        })
    }

    pub fn object() -> Self {
        Val(unsafe {
            _emval_new_object()
        })
    }

    pub fn cstring(s: CStr) -> Self {
        Val(unsafe {
            _emval_new_cstring(s)
        })
    }

    pub fn root() -> Self {
        Self::global(0 as _)
    }

    pub fn global(name: CStr) -> Self {
        Val (unsafe {
            _emval_get_global(name)
        })
    }

    pub fn get_computed(&self, prop: Val) -> Self {
        Val(unsafe {
            _emval_get_property(self.0, prop.0)
        })
    }

    pub fn set_computed(&self, prop: Val, value: Val) {
        unsafe {
            _emval_set_property(self.0, prop.0, value.0)
        }
    }

    pub fn get(&self, prop: CStr) -> Self {
        self.get_computed(Val::cstring(prop))
    }

    pub fn set(&self, prop: CStr, value: Val) {
        self.set_computed(Val::cstring(prop), value);
    }
}

impl Clone for Val {
    fn clone(&self) -> Val {
        unsafe {
            _emval_incref(self.0);
            Val(self.0)
        }
    }
}

impl Drop for Val {
    fn drop(&mut self) {
        unsafe {
            _emval_decref(self.0);
        }
    }
}
