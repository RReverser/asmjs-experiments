use std::sync::{Once, ONCE_INIT};
use std::os::raw::c_void as void;
use std::mem::size_of;
use value::*;

static REGISTER: Once = ONCE_INIT;

#[repr(C)]
pub struct TypeIdStruct(usize);

pub type TypeId = *const TypeIdStruct;

#[repr(u32)]
#[allow(non_camel_case_types)]
enum MemoryViewType {
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    f32,
    f64,
}

macro_rules! em_to_js {
    ($ty:ty) => {
        impl From<$ty> for Val {
            fn from(value: $ty) -> Self {
                unsafe {
                    Val::new_simple(&value)
                }
            }
        }
    }
}

macro_rules! em_from_js {
    ($ty:ty) => {
        impl From<Val> for $ty {
            fn from(value: Val) -> $ty {
                let mut destructors = Emdestructors::default();
                unsafe {
                    _emval_as(value.0, type_id::<$ty>(), &mut destructors) as _
                }
            }
        }
    }
}

macro_rules! em_js {
    ($ty: ty) => {{
        em_from_js!($ty);
        em_to_js!($ty);
    }}
}

macro_rules! cname {
    ($tt:tt) => (cstr!(stringify!($tt)))
}

#[link_args = "--bind --js-library rustlib.js"]
extern {
    fn _embind_register_void(type_id: TypeId, name: CStr);
    fn _embind_register_bool(type_id: TypeId, name: CStr, size: usize, true_value: bool, false_value: bool);
    fn _embind_register_integer(type_id: TypeId, name: CStr, size: usize, minRange: isize, maxRange: usize);
    fn _embind_register_float(type_id: TypeId, name: CStr, size: usize);
    fn _embind_register_memory_view(type_id: TypeId, view_type: MemoryViewType, name: CStr);

    fn _embind_register_rust_string(type_id: TypeId);
    fn _embind_register_rust_char(type_id: TypeId);

    fn _emval_as(value: Emval, type_id: TypeId, destructors: *mut Emdestructors) -> f64;
    fn _emval_new_cstring(s: CStr) -> Emval;
}

unsafe fn inner_type_id<T: ?Sized + 'static>() -> TypeId {
    ::std::intrinsics::type_id::<T>() as _
}

pub unsafe fn type_id<T: ?Sized + 'static>() -> TypeId {
    REGISTER.call_once(|| {
        macro_rules! register_void {
            ($ty:ty) => {{
                impl From<$ty> for Val {
                    fn from(_: $ty) -> Val {
                        Val(1 as _)
                    }
                }

                _embind_register_void(inner_type_id::<$ty>(), cname!($ty));
            }}
        }

        register_void!(());
        register_void!(void);

        impl From<bool> for Val {
            fn from(b: bool) -> Val {
                Val(if b { 3 } else { 4 } as _)
            }
        }

        impl From<Val> for bool {
            fn from(value: Val) -> bool {
                match value.0 as u32 {
                    3 => true,
                    4 => false,
                    _ => u32::from(value) != 0
                }
            }
        }

        _embind_register_bool(inner_type_id::<bool>(), cname!(bool), size_of::<bool>(), false, true);

        macro_rules! register_int {
            ($name:ident) => {{
                em_js!($name);
                _embind_register_integer(inner_type_id::<$name>(), cname!($name), size_of::<$name>(), ::std::$name::MIN as _, ::std::$name::MAX as _);
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
                em_js!($name);
                _embind_register_float(inner_type_id::<$name>(), cname!($name), size_of::<$name>());
            }}
        }

        register_float!(f32);
        register_float!(f64);

        impl From<Val> for char {
            fn from(value: Val) -> char {
                let mut destructors = Emdestructors::default();
                ::std::char::from_u32(unsafe {
                    _emval_as(value.0, type_id::<char>(), &mut destructors) as _
                }).unwrap()
            }
        }

        em_to_js!(char);
        _embind_register_rust_char(inner_type_id::<char>());

        impl<'a> From<&'a str> for Val {
            fn from(value: &'a str) -> Self {
                unsafe {
                    Val::new::<&str, _>(&value)
                }
            }
        }

        impl From<String> for Val {
            fn from(value: String) -> Self {
                Val::from(value.as_str())
            }
        }

        impl From<Val> for Box<str> {
            fn from(value: Val) -> Box<str> {
                #[allow(improper_ctypes)] // pretend that I know what I'm doing
                extern {
                    fn _emval_get_string(value: Emval) -> Box<str>;
                }

                unsafe {
                    _emval_get_string(value.0)
                }
            }
        }

        impl From<Val> for String {
            fn from(value: Val) -> String {
                Box::<str>::from(value).into_string()
            }
        }

        _embind_register_rust_string(inner_type_id::<&str>());

        struct MemoryView<T> {
            pub size: usize,
            pub data: *const T
        }

        macro_rules! register_memory_view {
            ($item_type:ident) => {{
                impl From<&'static [$item_type]> for Val {
                    fn from(slice: &'static [$item_type]) -> Val {
                        unsafe {
                            Val::new_simple(&MemoryView {
                                size: slice.len(),
                                data: slice.as_ptr()
                            })
                        }
                    }
                }

                _embind_register_memory_view(inner_type_id::<MemoryView<$item_type>>(), MemoryViewType::$item_type, cstr!("&", stringify!($item_type), "[]"))
            }}
        }

        register_memory_view!(i8);
        register_memory_view!(u8);
        register_memory_view!(i16);
        register_memory_view!(u16);
        register_memory_view!(i32);
        register_memory_view!(u32);
        register_memory_view!(f32);
        register_memory_view!(f64);

        impl<'a> From<&'a ::std::ffi::CStr> for Val {
            fn from(s: &'a ::std::ffi::CStr) -> Val {
                Val(unsafe {
                    _emval_new_cstring(s.as_ptr())
                })
            }
        }
    });

    inner_type_id::<T>()
}