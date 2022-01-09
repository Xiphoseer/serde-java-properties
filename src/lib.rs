#![warn(missing_docs)]

//! # Serde Java Properties
//!
//! [Java Properties](https://en.wikipedia.org/wiki/.properties) is a simple, line-oriented
//! format for specifying key-value resources used in Java programs. This crate offers
//! basic (de-)serializers for use with [serde](https://serde.rs)-enabled datastructures.
//!
//! ```properties
//! field_a: a value
//! field_b: 100
//! field_c: true
//! ```
//!
//! ## Implementation
//!
//! Internally, the [`java-properties` crate](https://crates.io/crates/java-properties) is used
//! for iterating key-value pairs in an input stream, and writing key-value pairs to an output
//! stream.
//!
//! ## Deserializing a struct
//!
//! Usually, the format is untyped i.e. it deserialises to a map from [`String`] to [`String`]. This
//! crate uses the default [`std::str::FromStr`] implementations for integers, floats and [`bool`] to
//! provide a typed interface on top of that. That way, simple structures or maps that implement
//! [`serde::Deserialize`] can be loaded from properties files.
//!
//! ```
//! # use serde::Deserialize;
//! #
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Data {
//!     field_a: String,
//!     field_b: usize,
//!     field_c: bool,
//! }
//! let text = "
//! field_a: a value
//! field_b: 100
//! field_c: true
//! ";
//!
//! let data: Data = serde_java_properties::from_str(text).unwrap();
//!
//! assert_eq!(data.field_a, "a value");
//! assert_eq!(data.field_b, 100);
//! assert_eq!(data.field_c, true);
//! ```
//!
//! ## Serializing a struct
//!
//! Serialization uses the default [`std::fmt::Display`] implementations for each primitive type.
//!
//! Supported in the top-level [`Serializer`]:
//! - Maps
//! - Structs
//! - Enums of struct variants
//! - Options of all of these
//!
//! Supported in the field-level Serializer:
//! - Integers (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`)
//! - Floats (`f32`, `f64`)
//! - Booleans (`true` or `false`)
//! - Strings
//! - Enums of unit variants
//! - Options of all of these
//!
//! ```
//! # use serde::Serialize;
//! #
//! #[derive(Debug, PartialEq, Serialize)]
//! struct Data {
//!     field_a: String,
//!     field_b: usize,
//!     field_c: bool,
//! }
//!
//! let data = Data { field_a: "value".to_string(), field_b: 100, field_c: true };
//! let string = serde_java_properties::to_string(&data).unwrap();
//!
//! assert_eq!(string, "field_a=value\nfield_b=100\nfield_c=true\n");
//! ```
//!
//! ## Tagged Enums
//!
//! Internally tagged enums are generally supported.
//!
//! Because of a limitation in serde, type hints are not available in this case, which
//! means that the [`serde::Deserializer::deserialize_any`] method on the `FieldDeserializer`
//! is called which only implements a limited heuristic as to which [`serde::de::Visitor`]
//! method to call.
//!
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! #[serde(tag = "type")]
//! pub enum Test {
//!     Var1 { key: usize, },
//!     Var2 { msg: String, },
//! }
//!
//! let test1 = Test::Var1 { key: 1000 };
//! let test2 = Test::Var2 { msg: "serde".to_string() };
//!
//! let text1 = serde_java_properties::to_string(&test1).unwrap();
//! let text2 = serde_java_properties::to_string(&test2).unwrap();
//!
//! assert_eq!(text1, "type=Var1\nkey=1000\n");
//! assert_eq!(text2, "type=Var2\nmsg=serde\n");
//!
//! let re1: Test = serde_java_properties::from_str(&text1).unwrap();
//! let re2: Test = serde_java_properties::from_str(&text2).unwrap();
//! ```
//!
//! ## Unit Struct Variants
//!
//! For simple enums, the name of the variant is used as the value
//!
//! ```
//! # use serde::{Serialize, Deserialize};
//! #
//! #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
//! enum Switch { On, Off }
//! #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
//! struct House { light: Switch, }
//!
//! let text = "light: On";
//! let data: House = serde_java_properties::from_str(&text).unwrap();
//!
//! assert_eq!(data.light, Switch::On);
//! let out = serde_java_properties::to_string(&data).unwrap();
//!
//! assert_eq!(out, "light=On\n");
//! ```
//!
//! ## Alternatives
//!
//! Similar to the [`java-properties` crate](https://crates.io/crates/java-properties) itself,
//! this crate is supposed to be an exact match to the format
//! [as specified in Java](https://docs.oracle.com/javase/10/docs/api/java/util/Properties.html#load(java.io.Reader)).
//!
//! If you need a more powerful configuration syntax, that supports nested structs, you
//! should probably use [HOCON](https://crates.io/crates/hocon).

pub mod de;
pub mod ser;

use std::io::{self, Read};

pub use de::Deserializer;
pub use ser::Serializer;

use de::Error;
use encoding::Encoding;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Turn a string into a value of `T`
///
/// This should technically be `T: DeserializeOwned`, but the implementation may change in the future
pub fn from_str<'a, T: Deserialize<'a>>(input: &'a str) -> Result<T, Error> {
    T::deserialize(de::Deserializer::from_str(input))
}

/// Turn a byte slice into a value of `T`
///
/// This should technically be `T: DeserializeOwned`, but the implementation may change in the future
///
/// **Important**: Do not pass a [`str::as_bytes`] to this function. The reader
/// expects *ISO-8859-1* by default. Use [`from_str`] instead, which sets the correct encoding.
pub fn from_slice<'a, T: Deserialize<'a>>(input: &'a [u8]) -> Result<T, Error> {
    T::deserialize(de::Deserializer::from_slice(input))
}

/// Turn a byte slice into a value of `T` using the given encoding
///
/// This should technically be `T: DeserializeOwned`, but the implementation may change in the future
pub fn from_slice_with_encoding<'a, T: Deserialize<'a>>(
    input: &'a [u8],
    encoding: &'static dyn Encoding,
) -> Result<T, Error> {
    T::deserialize(de::Deserializer::from_slice_with_encoding(input, encoding))
}

/// Turn a reader into a value of `T`
///
/// **Important**: Do not use this with a [`std::io::Cursor<&str>`]. The reader expects
/// *ISO-8859-1* by default. Use [`from_str`] instead, which sets the correct encoding.
pub fn from_reader<T: DeserializeOwned, R: Read>(reader: R) -> Result<T, Error> {
    T::deserialize(de::Deserializer::from_reader(reader))
}

/// Turn a reader into a value of `T` using the given encoding
pub fn from_reader_with_encoding<T: DeserializeOwned, R: Read>(
    reader: R,
    encoding: &'static dyn Encoding,
) -> Result<T, Error> {
    T::deserialize(de::Deserializer::from_reader_with_encoding(
        reader, encoding,
    ))
}

/// UTF-8 Encoding from the [`encoding`](https://crates.io/crates/encoding) crate for use with
/// the `*_with_encoding` functions.
pub const UTF8_ENCODING: &'static dyn Encoding = &encoding::codec::utf_8::UTF8Encoding;

/// Write a properties file to a string
///
/// *Important*: This uses UTF-8 encoding as the result is a Rust [String]
pub fn to_string<T: Serialize>(value: &T) -> Result<String, ser::Error> {
    let mut buffer = String::new();
    let writer = unsafe { buffer.as_mut_vec() };
    to_writer_with_encoding(value, writer, UTF8_ENCODING)?;
    Ok(buffer)
}

/// Write a properties file to a byte buffer with the specified encoding
pub fn to_vec_with_encoding<T: Serialize>(
    value: &T,
    encoding: &'static dyn Encoding,
) -> Result<Vec<u8>, ser::Error> {
    let mut buffer = Vec::new();
    to_writer_with_encoding(value, &mut buffer, encoding)?;
    Ok(buffer)
}

/// Write a properties file to a byte buffer
///
/// **Important**: This uses the default encoding *ISO-8859-1*
pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, ser::Error> {
    let mut buffer = Vec::new();
    to_writer(value, &mut buffer)?;
    Ok(buffer)
}

/// Write a properties file to a [io::Write] implementation
///
/// **Important**: This uses the default encoding *ISO-8859-1*
pub fn to_writer<T: Serialize, W: io::Write>(value: &T, writer: W) -> Result<(), ser::Error> {
    let serializer = ser::Serializer::from_writer(writer);
    value.serialize(serializer)?;
    Ok(())
}

/// Write a properties file to a [io::Write] implementation using the provided encoding
pub fn to_writer_with_encoding<T: Serialize, W: io::Write>(
    value: &T,
    writer: W,
    encoding: &'static dyn Encoding,
) -> Result<(), ser::Error> {
    let serializer = ser::Serializer::from_writer_with_encoding(writer, encoding);
    value.serialize(serializer)?;
    Ok(())
}
