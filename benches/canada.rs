#![feature(link_args, test)]

include!("utils/header.rs");

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

type TopLevel = FeatureCollection;

include!("utils/benches.rs");
