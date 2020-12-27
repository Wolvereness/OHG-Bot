use serenity::model::id::*;
use serde::{Deserializer, Serializer, Serialize, Deserialize};
use byteorder::ByteOrder;
use byteorder::BE;
use serde::de::{Visitor, Error};
use serenity::static_assertions::_core::fmt::Formatter;
use wither::bson::{Bson, Array};

pub struct Optional;
impl Optional {
    #[inline]
    pub fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Copy + Into<u64>,
    {
        match value.map(Into::into).map(Required::from) {
            None => serializer.serialize_none(),
            Some(value) => serializer.serialize_some(&value),
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        u64: Into<T>,
    {
        struct Visitable;
        impl<'de> Visitor<'de> for Visitable {
            type Value = Option<Required>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "Expecting Option")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
                where E: Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, <D as Deserializer<'de>>::Error>
                where D: Deserializer<'de>,
            {
                Required::deserialize(deserializer)
                    .map(Some)
            }
        }
        deserializer
            .deserialize_option(Visitable)
            .map(|value| value.map(<Required as Into<u64>>::into).map(<u64 as Into<T>>::into))
    }
}

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Required(i32, i32);

impl Required {
    #[inline(always)]
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: Copy + Into<u64>,
    {
        <Required as Serialize>::serialize(&Required::from(T::into(*value)), serializer)
    }

    #[inline(always)]
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
        where
            D: Deserializer<'de>,
            u64: Into<T>,
    {
        <Required as Deserialize>::deserialize(deserializer)
            .map(Required::into)
            .map(<u64 as Into<T>>::into)
    }
}

impl From<u64> for Required {
    #[inline]
    fn from(value: u64) -> Self {
        let bytes = &mut [0u8;8];
        BE::write_u64(bytes, value);
        let (high, low) = bytes.split_at(4);
        Required(
            BE::read_i32(high),
            BE::read_i32(low),
        )
    }
}

impl From<Required> for u64 {
    #[inline]
    fn from(value: Required) -> Self {
        let Required(high, low) = value;
        let bytes = &mut [0u8;8];
        let (high_bytes, low_bytes) = bytes.split_at_mut(4);
        BE::write_i32(high_bytes, high);
        BE::write_i32(low_bytes, low);
        BE::read_u64(bytes)
    }
}

impl From<Required> for Bson {
    fn from(value: Required) -> Self {
        Bson::Array(Array::from(&[
            Bson::Int32(value.0),
            Bson::Int32(value.1),
        ] as &[Bson]))
    }
}

macro_rules! implement {
    ($($t:ty;)*) => {$(
        impl From<$t> for Required {
            #[inline(always)]
            fn from(value: $t) -> Self {
                Required::from(<$t as Into<u64>>::into(value))
            }
        }

        impl From<Required> for $t {
            #[inline(always)]
            fn from(value: Required) -> Self {
                <$t as From<u64>>::from(<Required as Into<u64>>::into(value))
            }
        }
    )*};
}

implement!(
    ChannelId;
    RoleId;
    GuildId;
    UserId;
    MessageId;
);
