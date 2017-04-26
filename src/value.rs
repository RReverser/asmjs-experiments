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
