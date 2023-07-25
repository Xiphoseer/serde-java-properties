use super::Error;
use serde::{
    de::{self, IntoDeserializer},
    forward_to_deserialize_any,
};

pub(crate) struct FieldDeserializer(pub String);

macro_rules! make_fn {
    ($deserialize_fn:ident, $visit_fn:ident) => {
        fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.$visit_fn(self.0.parse()?)
        }
    };
}

pub(crate) struct UnitDeserializer;

impl<'de> de::VariantAccess<'de> for UnitDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::NotSupported)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotSupported)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotSupported)
    }
}

impl<'de> de::EnumAccess<'de> for FieldDeserializer {
    type Error = Error;

    type Variant = UnitDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.0.into_deserializer())
            .map(|v| (v, UnitDeserializer))
    }
}

impl<'de> de::Deserializer<'de> for FieldDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Ok(v) = self.0.parse::<u128>() {
            match v {
                0..=0xFF => return visitor.visit_u8(v as u8),
                0x100..=0xFFFF => return visitor.visit_u16(v as u16),
                0x10000..=0xFFFFFFFF => return visitor.visit_u32(v as u32),
                0x100000000..=0xFFFFFFFFFFFFFFFF => return visitor.visit_u64(v as u64),
                _ => return visitor.visit_u128(v),
            }
        }
        if let Ok(v) = self.0.parse::<i128>() {
            match v {
                -0x80..=-0x1 => return visitor.visit_i8(v as i8),
                -0x8000..=-0x81 => return visitor.visit_i16(v as i16),
                -0x80000000..=-0x8001 => return visitor.visit_i32(v as i32),
                -0x8000000000000000..=-0x80000001 => return visitor.visit_i64(v as i64),
                _ => return visitor.visit_i128(v),
            }
        }
        if self.0 == "true" {
            return visitor.visit_bool(true);
        } else if self.0 == "false" {
            return visitor.visit_bool(false);
        }
        if let Ok(v) = self.0.parse::<f32>() {
            return visitor.visit_f32(v);
        }
        if let Ok(v) = self.0.parse::<f64>() {
            return visitor.visit_f64(v);
        }
        visitor.visit_string(self.0)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.0)
    }

    make_fn!(deserialize_bool, visit_bool);
    make_fn!(deserialize_u8, visit_u8);
    make_fn!(deserialize_u16, visit_u16);
    make_fn!(deserialize_u32, visit_u32);
    make_fn!(deserialize_u64, visit_u64);
    make_fn!(deserialize_u128, visit_u128);

    make_fn!(deserialize_i8, visit_i8);
    make_fn!(deserialize_i16, visit_i16);
    make_fn!(deserialize_i32, visit_i32);
    make_fn!(deserialize_i64, visit_i64);
    make_fn!(deserialize_i128, visit_i128);

    make_fn!(deserialize_f32, visit_f32);
    make_fn!(deserialize_f64, visit_f64);

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        char str
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserializer;

    use super::FieldDeserializer;

    struct Visitor;

    macro_rules! is_ok {
        ($fn:ident: $ty:ident) => {
            fn $fn<E>(self, _: $ty) -> Result<Type, E>
            where
                E: serde::de::Error,
            {
                Ok(Type::$ty)
            }
        };
    }

    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Type {
        bool,
        i8,
        i16,
        i32,
        i64,
        i128,
        u8,
        u16,
        u32,
        u64,
        u128,
    }

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Type;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "TEST VALUE")
        }

        is_ok!(visit_bool: bool);
        is_ok!(visit_i8: i8);
        is_ok!(visit_i16: i16);
        is_ok!(visit_i32: i32);
        is_ok!(visit_i64: i64);
        is_ok!(visit_i128: i128);
        is_ok!(visit_u8: u8);
        is_ok!(visit_u16: u16);
        is_ok!(visit_u32: u32);
        is_ok!(visit_u64: u64);
        is_ok!(visit_u128: u128);
    }

    fn check(ty: Type, v: String) {
        assert_eq!(ty, FieldDeserializer(v).deserialize_any(Visitor).unwrap());
    }

    #[test]
    fn test_deserialize_any() {
        // In bounds
        check(Type::u8, format!("{}", u8::MAX));
        check(Type::u16, format!("{}", u16::MAX));
        check(Type::u32, format!("{}", u32::MAX));
        check(Type::u64, format!("{}", u64::MAX));

        check(Type::i8, format!("{}", i8::MIN));
        check(Type::i16, format!("{}", i16::MIN));
        check(Type::i32, format!("{}", i32::MIN));
        check(Type::i64, format!("{}", i64::MIN));

        // Out of bounds
        check(Type::u16, format!("{}", u16::from(u8::MAX) + 1));
        check(Type::u32, format!("{}", u32::from(u16::MAX) + 1));
        check(Type::u64, format!("{}", u64::from(u32::MAX) + 1));
        check(Type::u128, format!("{}", u128::from(u64::MAX) + 1));

        check(Type::i16, format!("{}", i16::from(i8::MIN) - 1));
        check(Type::i32, format!("{}", i32::from(i16::MIN) - 1));
        check(Type::i64, format!("{}", i64::from(i32::MIN) - 1));
        check(Type::i128, format!("{}", i128::from(i64::MIN) - 1));
    }
}
