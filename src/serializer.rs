use value::*;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeStruct, SerializeStructVariant, SerializeTupleVariant};
use std::fmt::{self, Display, Formatter};
use std::error::Error as StdError;
use std::result::Result as StdResult;

#[derive(Debug)]
pub enum Error {
    BigU64(u64),
    BigI64(i64),
    Custom(String)
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            Error::BigU64(ref v) => {
                write!(fmt, "{} is too big to fit into JavaScript number", v)
            },
            Error::BigI64(ref v) => {
                write!(fmt, "{} is too big to fit into JavaScript number", v)
            },
            Error::Custom(ref v) => {
                v.fmt(fmt)
            }
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BigU64(_) | Error::BigI64(_) => "given 64-bit value was too big to fit into JavaScript number",
            Error::Custom(_) => "unknown error"
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

type Result<V=Val> = StdResult<V, Error>;

pub struct Serializer;

macro_rules! serialize_from {
    ($name:ident, $ty:ty) => {
        fn $name(self, value: $ty) -> Result {
            Ok(Val::from(value))
        }
    }
}

impl ser::Serializer for Serializer {
    type Ok = Val;
    type Error = Error;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeTupleStruct = Self::SerializeTuple;
    type SerializeTupleVariant = VariantSerializer<Self::SerializeTuple>;
    type SerializeMap = MapSerializer;
    type SerializeStruct = Self::SerializeMap;
    type SerializeStructVariant = VariantSerializer<Self::SerializeStruct>;

    serialize_from!(serialize_bool, bool);
    serialize_from!(serialize_i8, i8);
    serialize_from!(serialize_u8, u8);
    serialize_from!(serialize_i16, i16);
    serialize_from!(serialize_u16, u16);
    serialize_from!(serialize_i32, i32);
    serialize_from!(serialize_u32, u32);
    serialize_from!(serialize_f32, f32);
    serialize_from!(serialize_f64, f64);
    serialize_from!(serialize_char, char);
    serialize_from!(serialize_str, &str);

    fn serialize_none(self) -> Result {
        self.serialize_unit()
    }

    fn serialize_unit(self) -> Result {
        Ok(Val::from(()))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, _name: &str) -> Result {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _name: &str, _variant_index: u32, variant: &str) -> Result {
        self.serialize_str(variant)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &str, len: usize) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(self, _name: &str, _variant_index: u32, variant: &str, _len: usize) -> Result<Self::SerializeTupleVariant> {
        Ok(VariantSerializer::new(variant))
    }

    fn serialize_i64(self, value: i64) -> Result {
        if value.abs() >> 53 == 0 {
            Ok(Val::from(value as f64))
        } else {
            Err(Error::BigI64(value))
        }
    }

    fn serialize_u64(self, value: u64) -> Result {
        if value >> 53 == 0 {
            Ok(Val::from(value as f64))
        } else {
            Err(Error::BigU64(value))
        }
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &str, value: &T) -> Result {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _name: &str, _variant_index: u32, variant: &str, value: &T) -> Result {
        VariantSerializer::new(variant).end_with(|_: ()| {
            value.serialize(self)
        })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(Self::SerializeSeq::default())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(Self::SerializeStruct::default())
    }

    fn serialize_struct_variant(self, _name: &str, _variant_index: u32, variant: &str, _len: usize) -> Result<Self::SerializeStructVariant> {
        Ok(VariantSerializer::new(variant))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(Self::SerializeMap::default())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result {
        let ptr = value.as_ptr();
        Ok(js_val!("HEAPU8.slice($0, $1)", ptr, ptr.offset(value.len() as isize)))
    }
}

pub struct MapSerializer {
    map: Val,
    key: Option<Val>
}

impl Default for MapSerializer {
    fn default() -> Self {
        MapSerializer {
            map: Val::object(),
            key: None
        }
    }
}

impl SerializeMap for MapSerializer {
    type Ok = Val;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.key = Some(key.serialize(Serializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.map.set(self.key.take().unwrap(), value.serialize(Serializer)?);
        Ok(())
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(&mut self, key: &K, value: &V) -> Result<()> {
        self.map.set(key.serialize(Serializer)?, value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result {
        Ok(self.map)
    }
}

impl SerializeStruct for MapSerializer {
    type Ok = Val;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        self.serialize_entry(key, value)
    }

    fn end(self) -> Result {
        SerializeMap::end(self)
    }
}

pub struct VariantSerializer<T> {
    key: Val,
    inner: T
}

impl<T: Default> VariantSerializer<T> {
    fn new(variant: &str) -> Self {
        VariantSerializer {
            key: Val::from(variant),
            inner: T::default()
        }
    }

    fn end_with<F: FnOnce(T) -> Result>(self, f: F) -> Result {
        let object = Val::object();
        object.set(self.key, f(self.inner)?);
        Ok(object)
    }
}

impl SerializeStructVariant for VariantSerializer<MapSerializer> {
    type Ok = Val;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<()> {
        self.inner.serialize_field(key, value)
    }

    fn end(self) -> Result {
        self.end_with(SerializeStruct::end)
    }
}

impl SerializeTupleVariant for VariantSerializer<SeqSerializer> {
    type Ok = Val;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        SerializeSeq::serialize_element(&mut self.inner, value)
    }

    fn end(self) -> Result {
        self.end_with(SerializeSeq::end)
    }
}

pub struct SeqSerializer {
    seq: Val
}

impl SerializeSeq for SeqSerializer {
    type Ok = Val;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        extern {
            fn _emval_array_push(dest: Emval, src: Emval);
        }

        unsafe {
            _emval_array_push(self.seq.0, value.serialize(Serializer)?.0);
        }

        Ok(())
    }

    fn end(self) -> Result {
        Ok(self.seq)
    }
}

impl SerializeTuple for SeqSerializer {
    type Ok = Val;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for SeqSerializer {
    type Ok = Val;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result {
        SerializeSeq::end(self)
    }
}

impl Default for SeqSerializer {
    fn default() -> Self {
        SeqSerializer {
            seq: Val::array()
        }
    }
}
