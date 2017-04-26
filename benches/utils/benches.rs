#[bench]
fn serialize_to_embind(b: &mut Bencher) {
    let collection: TopLevel = ::serde_json::from_str(JSON).unwrap();
    let global = Val::global();
    global.set("value", ());
    b.iter(|| {
        global.set("value", collection.serialize(::asmjs::serializer::Serializer).unwrap());
    });
}

#[bench]
fn serialize_via_json(b: &mut Bencher) {
    let collection: TopLevel = ::serde_json::from_str(JSON).unwrap();
    let global = Val::global();
    global.set("value", ());
    b.iter(|| {
        let json = ::serde_json::to_string(&collection).unwrap();
        global.set("json", json);
        global.set("value", js_val!("JSON.parse(json)"));
    });
}

#[bench]
fn native_json_parse(b: &mut Bencher) {
    let global = Val::global();
    global.set("json", JSON);
    b.iter(|| {
        js_val!("JSON.parse(json)");
    });
}

#[bench]
fn serde_parse(b: &mut Bencher) {
    b.iter(|| {
        ::serde_json::from_str::<TopLevel>(JSON).unwrap();
    });
}

#[bench]
fn serde_parse_and_serialize_to_embind(b: &mut Bencher) {
    let global = Val::global();
    global.set("value", ());
    b.iter(|| {
        let collection: TopLevel = ::serde_json::from_str(JSON).unwrap();
        global.set("value", collection.serialize(::asmjs::serializer::Serializer).unwrap());
    });
}
