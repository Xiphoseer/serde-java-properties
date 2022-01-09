//! Deserialization

use encoding::Encoding;
use java_properties::LineContent::{Comment, KVPair};
use java_properties::PropertiesIter;
use serde::de::{self, IntoDeserializer, MapAccess, Visitor};
use serde::forward_to_deserialize_any;
use std::fmt;
use std::io::{Cursor, Read};
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

use crate::UTF8_ENCODING;

mod field;

/// Read properties from a stream
///
/// This is a [serde](https://serde.rs) [`Deserializer`] implementation that
/// transforms a Java Properties file into a datastructure using
/// the [`java-properties` crate](https://crates.io/crates/java-properties).
pub struct Deserializer<R: Read> {
    inner: PropertiesIter<R>,
}

impl<R: Read> Deserializer<R> {
    /// Create a deserializer from a [`Read`] implementation
    ///
    /// **Important**: Do not use this with a [`std::io::Cursor<&str>`]. The reader
    /// expects *ISO-8859-1* by default. Use [`Deserializer::from_str`] instead, which
    /// sets the correct encoding.
    pub fn from_reader(reader: R) -> Self {
        Self {
            inner: PropertiesIter::new(reader),
        }
    }

    /// Create a deserializer from a [`Read`] implementation and the specified encoding
    pub fn from_reader_with_encoding(reader: R, encoding: &'static dyn Encoding) -> Self {
        Self {
            inner: PropertiesIter::new_with_encoding(reader, encoding),
        }
    }
}

impl<'a> Deserializer<Cursor<&'a str>> {
    /// Create a deserializer from a [`str`] slice
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &'a str) -> Self {
        Self::from_reader_with_encoding(Cursor::new(s), UTF8_ENCODING)
    }
}

impl<'a> Deserializer<Cursor<&'a [u8]>> {
    /// Create a deserializer from a byte slice
    ///
    /// **Important**: Do not pass a [`str::as_bytes`] to this function. The reader
    /// expects *ISO-8859-1* by default. Use [`Deserializer::from_str`] instead, which
    /// sets the correct encoding.
    pub fn from_slice(s: &'a [u8]) -> Self {
        Self::from_reader(Cursor::new(s))
    }

    /// Create a deserializer from a byte slice with the specified encoding
    pub fn from_slice_with_encoding(s: &'a [u8], encoding: &'static dyn Encoding) -> Self {
        Self::from_reader_with_encoding(Cursor::new(s), encoding)
    }
}

#[derive(Debug)]
#[non_exhaustive]
/// A deserialization error
pub enum Error {
    /// A message from serde
    Custom {
        /// The text of the message
        msg: String,
    },
    /// A line failed to load
    Properties(java_properties::PropertiesError),
    /// A field with type hint integer failed to parse
    ParseIntError(ParseIntError),
    /// A field with type hint float failed to parse
    ParseFloatError(ParseFloatError),
    /// A field with type hint float failed to parse
    ParseBoolError(ParseBoolError),
    /// Not supported
    NotSupported,
}

impl From<java_properties::PropertiesError> for Error {
    fn from(e: java_properties::PropertiesError) -> Self {
        Self::Properties(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

impl From<ParseFloatError> for Error {
    fn from(e: ParseFloatError) -> Self {
        Self::ParseFloatError(e)
    }
}

impl From<ParseBoolError> for Error {
    fn from(e: ParseBoolError) -> Self {
        Self::ParseBoolError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom { msg } => write!(f, "Custom: {:?}", msg),
            Self::NotSupported => write!(f, "Not supported"),
            Self::Properties(e) => e.fmt(f),
            Self::ParseIntError(e) => e.fmt(f),
            Self::ParseFloatError(e) => e.fmt(f),
            Self::ParseBoolError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom {
            msg: msg.to_string(),
        }
    }
}

impl<'de, I: Read> de::Deserializer<'de> for Deserializer<I> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(PropertiesMapAccess {
            de: self,
            line_value: None,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct PropertiesMapAccess<I: Read> {
    de: Deserializer<I>,
    line_value: Option<String>,
}

impl<'de, I: Read> MapAccess<'de> for PropertiesMapAccess<I> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        while let Some(line) = self.de.inner.next().transpose()? {
            match line.consume_content() {
                Comment(_) => {} // ignore
                KVPair(key, value) => {
                    self.line_value = Some(value);
                    return seed.deserialize(key.into_deserializer()).map(Some);
                }
            };
        }
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = self.line_value.take().unwrap();
        seed.deserialize(field::FieldDeserializer(value))
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::de::Deserializer;

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    struct Workload {
        recordcount: usize,
        operationcount: usize,
        workload: String,

        readallfields: bool,

        readproportion: f32,
        updateproportion: f32,
        scanproportion: f32,
        insertproportion: f32,

        requestdistribution: String,
    }

    #[test]
    fn test() {
        let data = "
recordcount=1000
operationcount=1000
workload=site.ycsb.workloads.CoreWorkload

readallfields=true

readproportion=0.5
updateproportion=0.5
scanproportion=0
insertproportion=0

requestdistribution=zipfian
";
        let deserializer = Deserializer::from_str(data);
        let workload_a = Workload::deserialize(deserializer).unwrap();
        assert_eq!(
            workload_a,
            Workload {
                recordcount: 1000,
                operationcount: 1000,
                workload: "site.ycsb.workloads.CoreWorkload".to_string(),

                readallfields: true,

                readproportion: 0.5,
                updateproportion: 0.5,
                scanproportion: 0.0,
                insertproportion: 0.0,

                requestdistribution: "zipfian".to_string(),
            }
        );
    }
}
