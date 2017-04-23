use std::os::raw::c_void as void;
use std::ptr::{null, null_mut};
use types::*;

#[repr(C)]
pub struct EmvalStruct(usize);

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

#[link_args = "--bind --js-library rustlib.js"]
extern {
    fn _emval_incref(value: Emval);
    fn _emval_decref(value: Emval);
    fn _emval_run_destructors(destructors: Emval);

    fn _emval_take_value(type_id: TypeId, ptr: *const void) -> Emval;

    fn _emval_new_array() -> Emval;
    fn _emval_new_object() -> Emval;

    fn _emval_get_global(name: CStr) -> Emval;
    fn _emval_get_property(obj: Emval, prop: Emval) -> Emval;
    fn _emval_set_property(obj: Emval, prop: Emval, value: Emval);

    #[doc(hidden)]
    pub fn emscripten_asm_const_int(code: CStr, ...) -> Emval;

    fn _embind_iterator_start(value: Emval) -> Emval;
    fn _embind_iterator_next(iterator: Emval) -> Emval;
}

pub struct Val(pub Emval);

impl Val {
    pub unsafe fn new<RegisteredType: 'static, ActualType>(value: &ActualType) -> Self {
        Val(_emval_take_value(type_id::<RegisteredType>(), value as *const ActualType as *const void))
    }

    pub unsafe fn new_simple<T: 'static>(value: &T) -> Self {
        Val::new::<T, T>(value)
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
    ($expr:expr $(,$arg:expr)*) => ($crate::value::Val(unsafe {
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
    use std::mem::size_of;

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
        global.set("ch", 'c');

        assert_eq!(count_emval_handles(), 1);

        assert_eq!(bool::from(global.get("flag")), true);
        assert_eq!(i32::from(global.get("num")), 42);
        assert_eq!(i8::from(global.get("num")), 42);
        assert_eq!(f64::from(global.get("num")), 42f64);
        assert_eq!(u32::from(global.get("arr").get(1)), 2);
        assert_eq!(String::from(global.get("str")), "hello, world");
        assert_eq!(String::from(global.get("ch")), "c");
        assert_eq!(char::from(global.get("ch")), 'c');
        assert_eq!(char::from(global.get("num")), '*');
        assert_eq!(char::from(global.get("str").get(0)), 'h');
        assert_eq!(char::from(js_val!("str.charCodeAt(0)")), 'h');
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
