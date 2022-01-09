# Serde Java Properties

[![Docs](https://img.shields.io/docsrs/serde-java-properties)](https://docs.rs/serde-java-properties)
[![License](https://img.shields.io/crates/l/serde-java-properties)](https://github.com/Xiphoseer/serde-java-properties/tree/main/LICENSE)
[![Version](https://img.shields.io/crates/v/serde-java-properties)](https://crates.io/crates/serde-java-properties)

[Java Properties](https://en.wikipedia.org/wiki/.properties) is a simple, line-oriented
format for specifying key-value resources used in Java programs. This crate offers
basic (de-)serializers for use with [serde](https://serde.rs)-enabled datastructures.

```properties
field_a: a value
field_b: 100
field_c: true
```

## Implementation

Internally, the [`java-properties` crate](https://crates.io/crates/java-properties) is used
for iterating key-value pairs in an input stream, and writing key-value pairs to an output
stream.

## Deserializing a struct

Usually, the format is untyped i.e. it deserialises to a map from `String` to `String`. This
crate uses the default `std::str::FromStr` implementations for integers, floats and booleans to
provide a typed interface on top of that. That way, simple structures or maps that implement
`serde::Deserialize` can be loaded from properties files.

```rs
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
struct Data {
    field_a: String,
    field_b: usize,
    field_c: bool,
}
let text = "
field_a: a value
field_b: 100
field_c: true
";

let data: Data = serde_java_properties::from_str(text).unwrap();

assert_eq!(data.field_a, "a value");
assert_eq!(data.field_b, 100);
assert_eq!(data.field_c, true);
```

## Serializing a struct

Serialization uses the default `std::fmt::Display` implementations for each primitive type.

Supported in the top-level `Serializer`:
- Maps
- Structs
- Enums of struct variants
- Options of all of these

Supported in the field-level `Serializer`:
- Integers (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`)
- Floats (`f32`, `f64`)
- Booleans (`true` or `false`)
- Strings
- Enums of unit variants
- Options of all of these

```rs
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
struct Data {
    field_a: String,
    field_b: usize,
    field_c: bool,
}

let data = Data { field_a: "value".to_string(), field_b: 100, field_c: true };
let string = serde_java_properties::to_string(&data).unwrap();

assert_eq!(string, "field_a=value\nfield_b=100\nfield_c=true\n");
```

## Alternatives

Similar to the [`java-properties` crate](https://crates.io/crates/java-properties) itself,
this crate is supposed to be an exact match to the format
[as specified in Java](https://docs.oracle.com/javase/10/docs/api/java/util/Properties.html#load(java.io.Reader)).
If you need a more powerful configuration syntax, that supports nested structs, you
should probably use [HOCON](https://crates.io/crates/hocon).
