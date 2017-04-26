#[macro_use]
extern crate asmjs;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate test;

use std::collections::BTreeMap as Map;
use asmjs::value::Val;
use serde::Serialize;
use test::Bencher;

#[link_args = "--bind --js-library rustlib.js"]
extern "C" {}
