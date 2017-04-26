#![feature(link_args)]

include!("utils/header.rs");

extern crate serde;

#[macro_use]
extern crate serde_derive;

use asmjs::serializer::Serializer;
use serde::Serialize;

#[test]
fn test_serialize_struct() {
    #[derive(Serialize)]
    struct S {
        x: &'static str,
        y: u64,
        z: [f64; 2]
    }

    let s = S {
        x: "hello, world",
        y: 42,
        z: [123.456, 789.]
    };

    let val = s.serialize(Serializer).unwrap();

    assert_eq!(String::from(js_val!("JSON.stringify(requireHandle($0))", val.0)), r#"{"x":"hello, world","y":42,"z":[123.456,789]}"#);
}