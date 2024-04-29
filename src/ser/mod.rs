//! Serialization

use std::{error, fmt, io};

use encoding_rs::Encoding;
use java_properties::PropertiesError;
use serde::{
    ser::{self, Impossible},
    Serialize,
};

use self::string::StringSerializer;

mod string;

pub use java_properties::LineEnding;

/// Serialize a structure to a properties file
pub struct Serializer<W: io::Write> {
    inner: java_properties::PropertiesWriter<W>,
}

impl<W: io::Write> Serializer<W> {
    /// Set the KV separator
    ///
    /// This method returns an error if the separator is not valid. A separator is
    /// valid if is non-empty and consists only of whitespace characters, except
    /// a single `:` or `=` character.
    pub fn set_kv_separator(&mut self, separator: &str) -> Result<(), Error> {
        self.inner.set_kv_separator(separator)?;
        Ok(())
    }

    /// Set the line ending to `\n`, `\r` or `\r\n`.
    pub fn set_line_ending(&mut self, line_ending: LineEnding) {
        self.inner.set_line_ending(line_ending);
    }

    /// Create a serializer from a [`io::Write`] implementation
    pub fn from_writer(writer: W) -> Self {
        Self {
            inner: java_properties::PropertiesWriter::new(writer),
        }
    }

    /// Create a serializer from a [`io::Write`] implementation with a specificed encoding
    pub fn from_writer_with_encoding(writer: W, encoding: &'static Encoding) -> Self {
        Self {
            inner: java_properties::PropertiesWriter::new_with_encoding(writer, encoding),
        }
    }
}

/// A serialization error
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A properties error
    Properties(PropertiesError),
    /// A message from [serde]
    Custom {
        /// The message text
        msg: String,
    },
    /// Not a map
    NotAMap,
    /// Serialization not supported
    NotSupported,
}

impl From<PropertiesError> for Error {
    fn from(e: PropertiesError) -> Self {
        Self::Properties(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Properties(e) => e.fmt(f),
            Self::Custom { msg } => write!(f, "Serialization error: {}", msg),
            Self::NotAMap => write!(f, "Can only serialize a map-like structure to properties"),
            Self::NotSupported => write!(f, "Not supported"),
        }
    }
}

impl error::Error for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom {
            msg: msg.to_string(),
        }
    }
}

impl<W: io::Write> ser::SerializeStruct for Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(StringSerializer)?;
        self.inner.write(key, &value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: io::Write> ser::SerializeStructVariant for Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(StringSerializer)?;
        self.inner.write(key, &value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// A struct to serialize a map
pub struct MapSerializer<W: io::Write> {
    inner: java_properties::PropertiesWriter<W>,
    key: Option<String>,
}

impl<W: io::Write> ser::SerializeMap for MapSerializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let str = T::serialize(key, string::StringSerializer)?;
        self.key = Some(str);
        Ok(())
    }

    /// Panics is `serialize_key` wasn't called before successfully
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let key = self.key.take().unwrap();
        let value = value.serialize(StringSerializer)?;
        self.inner.write(&key, &value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

macro_rules! not_a_map {
    ($($fn_name:ident: $ty:ty),*) => {
        $(
            fn $fn_name(self, _v: $ty) -> Result<Self::Ok, Self::Error> {
                Err(Error::NotAMap)
            }
        )*
    };
}

impl<W: io::Write> ser::Serializer for Serializer<W> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = MapSerializer<W>;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    not_a_map!(
        serialize_bool: bool,
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_str: &str,
        serialize_char: char,
        serialize_bytes: &[u8]
    );

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::NotAMap)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::NotAMap)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::NotAMap)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::NotAMap)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer {
            inner: self.inner,
            key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }
}
