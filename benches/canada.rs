#![feature(link_args, test)]

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

#[derive(Serialize, Deserialize)]
pub enum ObjType {
    FeatureCollection,
    Feature,
    Polygon,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub features: Vec<Feature>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub properties: Map<String, String>,
    pub geometry: Geometry,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub coordinates: Vec<Vec<(f32, f32)>>,
}

static JSON: &'static str = include_str!("data/canada.json");

#[bench]
fn bench_serialize_to_embind(b: &mut Bencher) {
    let collection: FeatureCollection = ::serde_json::from_str(JSON).unwrap();
    let global = Val::global();
    global.set("value", ());
    b.iter(|| {
        global.set("value", collection.serialize(::asmjs::serializer::Serializer).unwrap());
    });
    global.set("value", ());
}

#[bench]
fn bench_serialize_to_json_and_parse(b: &mut Bencher) {
    let global = Val::global();
    global.set("value", ());
    b.iter(|| {
        global.set("value", js_val!("JSON.parse($0)", Val::from(JSON).0));
    });
    global.set("value", ());
}

#[bench]
fn bench_parse_and_serialize_to_embind(b: &mut Bencher) {
    let global = Val::global();
    global.set("value", ());
    b.iter(|| {
        let collection: FeatureCollection = ::serde_json::from_str(JSON).unwrap();
        global.set("value", collection.serialize(::asmjs::serializer::Serializer).unwrap());
    });
    global.set("value", ());
}
