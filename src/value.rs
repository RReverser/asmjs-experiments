use std::os::raw::c_void as void;
use std::intrinsics;
use std::mem::size_of;

pub type TypeId = i32;

pub fn type_id<T: ?Sized + 'static>() -> TypeId {
    unsafe {
        intrinsics::type_id::<T>() as _
    }
}

#[repr(C)]
pub struct EmvalStruct(void);

pub type Emval = *mut EmvalStruct;

pub type CStr = *const u8;

#[link_args = "--bind --js-library rustlib.js"]
extern {
    fn _embind_register_void(type_id: TypeId, name: CStr);
    fn _embind_register_bool(type_id: TypeId, name: CStr, size: usize, true_value: bool, false_value: bool);
    fn _embind_register_integer(type_id: TypeId, name: CStr, size: usize, minRange: i32, maxRange: u32);
    fn _embind_register_float(type_id: TypeId, name: CStr, size: usize);

    fn _embind_register_rust_string(type_id: TypeId);

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

macro_rules! embindable {
    ($ty:ty) => {
        impl Into<Val> for $ty {
            fn into(self) -> Val {
                Val::new(self)
            }
        }
    }
}

pub struct Val(Emval);

impl Val {
    pub fn register() {
        unsafe {
            macro_rules! register_void {
                ($ty:ty) => {{
                    impl Into<Val> for $ty {
                        fn into(self) -> Val {
                            Val(1 as _)
                        }
                    }

                    _embind_register_void(type_id::<$ty>(), concat!(stringify!($ty), "\0").as_ptr());
                }}
            }

            register_void!(());
            register_void!(void);

            impl Into<Val> for bool {
                fn into(self) -> Val {
                    Val(if self { 3 } else { 4 } as _)
                }
            }

            _embind_register_bool(type_id::<bool>(), b"bool\0".as_ptr(), size_of::<bool>(), false, true);

            macro_rules! register_int {
                ($name:ident) => {{
                    embindable!($name);
                    _embind_register_integer(type_id::<$name>(), concat!(stringify!($name), "\0").as_ptr(), size_of::<$name>(), ::std::$name::MIN as _, ::std::$name::MAX as _);
                }}
            }

            register_int!(i8);
            register_int!(u8);
            register_int!(i16);
            register_int!(u16);
            register_int!(i32);
            register_int!(u32);
            register_int!(isize);
            register_int!(usize);

            macro_rules! register_float {
                ($name:ident) => {{
                    embindable!($name);
                    _embind_register_float(type_id::<$name>(), concat!(stringify!($name), "\0").as_ptr(), size_of::<$name>());
                }}
            }

            register_float!(f32);
            register_float!(f64);

            embindable!(&'static str);
            _embind_register_rust_string(type_id::<&'static str>());
        }
    }

    fn new<T: 'static>(value: T) -> Self {
        Val(unsafe {
            _emval_take_value(type_id::<T>(), &value as *const T as *const void)
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

    pub fn null() -> Self {
        Val(2 as _)
    }

    pub fn global() -> Self {
        Val(unsafe {
            _emval_get_global(0 as _)
        })
    }

    pub fn get<P: Into<Val>>(&self, prop: P) -> Self {
        Val(unsafe {
            _emval_get_property(self.0, prop.into().0)
        })
    }

    pub fn set<P: Into<Val>, V: Into<Val>>(&self, prop: P, value: V) {
        unsafe {
            _emval_set_property(self.0, prop.into().0, value.into().0)
        }
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
