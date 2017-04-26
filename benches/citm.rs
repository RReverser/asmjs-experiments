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
use serde::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer, Unexpected};
use test::Bencher;
use std::fmt::{self, Display};
use std::str::FromStr;

#[link_args = "--bind --js-library rustlib.js"]
extern "C" {}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct PrimStr<T>(T) where T: Copy + Ord + Display + FromStr;

impl<T> Serialize for PrimStr<T>
    where T: Copy + Ord + Display + FromStr,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

impl<'de, T> Deserialize<'de> for PrimStr<T>
    where T: Copy + Ord + Display + FromStr,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        use std::marker::PhantomData;
        struct Visitor<T>(PhantomData<T>);

        impl<'v, T> de::Visitor<'v> for Visitor<T>
            where T: Copy + Ord + Display + FromStr,
        {
            type Value = PrimStr<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("number represented as string")
            }

            fn visit_str<E>(self, value: &str) -> Result<PrimStr<T>, E>
                where E: de::Error,
            {
                match T::from_str(value) {
                    Ok(id) => Ok(PrimStr(id)),
                    Err(_) => Err(E::invalid_value(Unexpected::Str(value), &self)),
                }
            }
        }

        deserializer.deserialize_str(Visitor(PhantomData))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CitmCatalog {
    #[serde(rename="areaNames")]
    pub area_names: Map<IdStr, String>,
    #[serde(rename="audienceSubCategoryNames")]
    pub audience_sub_category_names: Map<IdStr, String>,
    #[serde(rename="blockNames")]
    pub block_names: Map<IdStr, String>,
    pub events: Map<IdStr, Event>,
    pub performances: Vec<Performance>,
    #[serde(rename="seatCategoryNames")]
    pub seat_category_names: Map<IdStr, String>,
    #[serde(rename="subTopicNames")]
    pub sub_topic_names: Map<IdStr, String>,
    #[serde(rename="subjectNames")]
    pub subject_names: Map<IdStr, String>,
    #[serde(rename="topicNames")]
    pub topic_names: Map<IdStr, String>,
    #[serde(rename="topicSubTopics")]
    pub topic_sub_topics: Map<IdStr, Vec<Id>>,
    #[serde(rename="venueNames")]
    pub venue_names: Map<String, String>,
}

pub type Id = u32;
pub type IdStr = PrimStr<u32>;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub description: (),
    pub id: Id,
    pub logo: Option<String>,
    pub name: String,
    #[serde(rename="subTopicIds")]
    pub sub_topic_ids: Vec<Id>,
    #[serde(rename="subjectCode")]
    pub subject_code: (),
    pub subtitle: (),
    #[serde(rename="topicIds")]
    pub topic_ids: Vec<Id>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Performance {
    #[serde(rename="eventId")]
    pub event_id: Id,
    pub id: Id,
    pub logo: Option<String>,
    pub name: (),
    pub prices: Vec<Price>,
    #[serde(rename="seatCategories")]
    pub seat_categories: Vec<SeatCategory>,
    #[serde(rename="seatMapImage")]
    pub seat_map_image: (),
    pub start: u64,
    #[serde(rename="venueCode")]
    pub venue_code: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
    pub amount: u32,
    #[serde(rename="audienceSubCategoryId")]
    pub audience_sub_category_id: Id,
    #[serde(rename="seatCategoryId")]
    pub seat_category_id: Id,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeatCategory {
    pub areas: Vec<Area>,
    #[serde(rename="seatCategoryId")]
    pub seat_category_id: Id,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    #[serde(rename="areaId")]
    pub area_id: Id,
    #[serde(rename="blockIds")]
    pub block_ids: Vec<Id>,
}

static JSON: &'static str = include_str!("data/citm.json");

#[bench]
fn bench_serialize_to_embind(b: &mut Bencher) {
    let collection: CitmCatalog = ::serde_json::from_str(JSON).unwrap();
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
        let collection: CitmCatalog = ::serde_json::from_str(JSON).unwrap();
        global.set("value", collection.serialize(::asmjs::serializer::Serializer).unwrap());
    });
    global.set("value", ());
}
