use std::os::raw::c_void as void;
use std::mem::size_of;
use std::sync::{Once, ONCE_INIT};
use std::ptr::{null, null_mut};

static REGISTER: Once = ONCE_INIT;

pub type TypeId = i32;

#[repr(C)]
pub struct EmvalStruct(void);

pub type Emval = *mut EmvalStruct;

#[repr(C)]
pub struct Emdestructors {
    handle: Emval
}

impl Default for Emdestructors {
    fn default() -> Self {
        Emdestructors {
            handle: null_mut()
        }
    }
}

impl Drop for Emdestructors {
    fn drop(&mut self) {
        unsafe {
            debug_assert!(!self.handle.is_null());
            _emval_run_destructors(self.handle)
        }
    }
}

pub type CStr = *const u8;

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

#[link_args = "--bind --js-library rustlib.js"]
extern {
    fn _embind_register_void(type_id: TypeId, name: CStr);
    fn _embind_register_bool(type_id: TypeId, name: CStr, size: usize, true_value: bool, false_value: bool);
    fn _embind_register_integer(type_id: TypeId, name: CStr, size: usize, minRange: isize, maxRange: usize);
    fn _embind_register_float(type_id: TypeId, name: CStr, size: usize);
    fn _embind_register_memory_view(type_id: TypeId, view_type: MemoryViewType, name: CStr);

    fn _embind_register_rust_string(type_id: TypeId);

    fn _emval_incref(value: Emval);
    fn _emval_decref(value: Emval);
    fn _emval_run_destructors(destructors: Emval);

    fn _emval_take_value(type_id: TypeId, ptr: *const void) -> Emval;
    fn _emval_as(value: Emval, type_id: TypeId, destructors: *mut Emdestructors) -> f64;
    fn _emval_new_array() -> Emval;
    fn _emval_new_object() -> Emval;
    fn _emval_new_cstring(s: CStr) -> Emval;

    fn _emval_get_global(name: CStr) -> Emval;
    fn _emval_get_property(obj: Emval, prop: Emval) -> Emval;
    fn _emval_set_property(obj: Emval, prop: Emval, value: Emval);

    #[doc(hidden)]
    pub fn emscripten_asm_const_int(code: CStr, ...) -> Emval;

    fn _embind_iterator_start(value: Emval) -> Emval;
    fn _embind_iterator_next(iterator: Emval) -> Emval;
}

macro_rules! em_to_js {
    ($ty:ty) => {
        impl From<$ty> for Val {
            fn from(value: $ty) -> Self {
                unsafe {
                    Val::new(&value)
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

unsafe fn inner_type_id<T: ?Sized + 'static>() -> i32 {
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

        impl<'a> From<&'a str> for Val {
            fn from(value: &'a str) -> Self {
                unsafe {
                    Val(_emval_take_value(type_id::<&str>(), &value as *const &'a str as *const void))
                }
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
                            Val::new(&MemoryView {
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

pub struct Val(pub Emval);

impl Val {
    pub unsafe fn new<T: 'static>(value: &T) -> Self {
        Val(_emval_take_value(type_id::<T>(), value as *const T as *const void))
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

    pub fn null() -> Self {
        Val(2 as _)
    }

    pub fn global() -> Self {
        Val(unsafe {
            _emval_get_global(null())
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

impl<'a> IntoIterator for &'a Val {
    type Item = Val;
    type IntoIter = EmvalIterator;

    fn into_iter(self) -> Self::IntoIter {
        EmvalIterator {
            iterator: unsafe {
                Val(_embind_iterator_start(self.0))
            }
        }
    }
}

impl IntoIterator for Val {
    type Item = Val;
    type IntoIter = EmvalIterator;

    fn into_iter(self) -> Self::IntoIter {
        (&self).into_iter()
    }
}

pub struct EmvalIterator {
    iterator: Val
}

impl Iterator for EmvalIterator {
    type Item = Val;

    fn next(&mut self) -> Option<Val> {
        unsafe {
            let result = _embind_iterator_next(self.iterator.0);
            if result.is_null() {
                None
            } else {
                Some(Val(result))
            }
        }
    }
}

#[macro_export]
macro_rules! js_val {
    ($expr:tt $(,$arg:tt)*) => ($crate::value::Val(unsafe {
        $crate::value::emscripten_asm_const_int(cstr!("return __emval_register(", $expr, ")") $(,$arg)*)
    }))
}

#[cfg(test)]
pub fn count_emval_handles() -> usize {
    unsafe {
        emscripten_asm_const_int(cstr!("return count_emval_handles()")) as usize
    }
}

#[cfg(test)]
mod tests {
    use value::*;

    static STATIC_ARRAY: [u32; 3] = [1, 2, 3];

    #[test]
    fn test_simple_values() {
        assert_eq!(count_emval_handles(), 0);

        let global = Val::global();

        assert_eq!(count_emval_handles(), 1);

        global.set("str", "hello, world");
        global.set("flag", true);
        global.set("num", 42);
        global.set("arr", &STATIC_ARRAY[..]);

        assert_eq!(count_emval_handles(), 1);

        assert_eq!(bool::from(global.get("flag")), true);
        assert_eq!(i32::from(global.get("num")), 42);
        assert_eq!(i8::from(global.get("num")), 42);
        assert_eq!(f64::from(global.get("num")), 42f64);
        assert_eq!(u32::from(js_val!("arr[1]")), 2);
        assert_eq!(String::from(global.get("str")), "hello, world");
        assert_eq!(u8::from(js_val!("str.charCodeAt(0)")) as char, 'h');
        assert_eq!(String::from(global.get("str").into_iter().next().unwrap()), "h");

        assert_eq!(count_emval_handles(), 1);
    }

    #[test]
    fn test_js_val() {
        assert_eq!(count_emval_handles(), 0);

        assert_eq!(usize::from(js_val!("Int32Array").get("BYTES_PER_ELEMENT")), size_of::<i32>());
        assert_eq!(u32::from(js_val!("$0+$1", 10, 20)), 30);

        assert_eq!(count_emval_handles(), 0);
    }
}
