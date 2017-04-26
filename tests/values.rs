#![feature(link_args)]

include!("utils/header.rs");

use asmjs::value::*;
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
