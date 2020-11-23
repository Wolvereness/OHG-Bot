use serenity::model::id::*;
use serde::{Deserializer, Serializer};
use serde::ser::SerializeTuple;
use byteorder::ByteOrder;
use byteorder::BE;
use serde::de::{Visitor, SeqAccess, Error};
use serenity::static_assertions::_core::fmt::Formatter;

#[derive(Copy, Clone)]
pub struct Optional(Option<u64>);

impl Optional {
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Copy + Into<Optional>,
    {
        match value.into() {
            Optional(None) => serializer.serialize_none(),
            Optional(Some(value)) => serializer.serialize_some(&Required(value)),
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: From<Optional>,
    {
        struct Visitable;
        impl Visitor<'de> for Visitable {
            type Value = Option<Required>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "Expecting Required")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
                where E: Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, <D as Deserializer<'de>>::Error>
                where D: Deserializer<'de>,
            {
                Required::deserialize(deserializer).map(Some)
            }
        }
        Ok(match deserializer.deserialize_option(Visitable)? {
            None => Optional(None),
            Some(Required(value)) => Optional(Some(value)),
        }).map(From::from)
    }
}

#[derive(Copy, Clone)]
pub struct Required(u64);

impl Required {
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: Copy + Into<Required>,
    {
        let mut bytes = &mut [0u8;8];
        BE::write_u64(bytes, value.into().0);
        let (high, low) = bytes.split_at(4);
        let mut serializer = serializer.serialize_tuple(2)?;
        serializer.serialize_element(&BE::read_u32(high))?;
        serializer.serialize_element(&BE::read_u32(low))?;
        serializer.end()
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
        where
            D: Deserializer<'de>,
            T: From<Required>,
    {
        struct Visitable;
        impl Visitor for Visitable {
            type Value = (u32, u32);

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "Expected (u32,u32)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error>
                where A: SeqAccess<'de>,
            {
                let high = if let Some(high) = seq.next_element()? {
                    high
                } else {
                    return Err(A::Error::invalid_length(0, &"2 elements expected"))
                };

                let low = if let Some(low) = seq.next_element()? {
                    low
                } else {
                    return Err(A::Error::invalid_length(1, &"2 elements expected"))
                };

                Ok((high, low))
            }
        }

        let (high, low) = deserializer.deserialize_tuple(2, Visitable)?;
        let mut bytes = &mut [0u8;8];
        let (high_bytes, low_bytes) = bytes.split_at_mut(4);
        BE::write_u32(high_bytes, high);
        BE::write_u32(low_bytes, low);
        Ok(T::from(Required(BE::read_u64(bytes))))
    }
}

macro_rules! implement {
    ($t:ty) => {
        impl Into<$t> for Required {
            fn into(self) -> $t {
                $t(self.0)
            }
        }

        impl Into<Required> for $t {
            fn into(self) -> Required {
                Required(self.0)
            }
        }

        impl Into<Option<$t>> for Optional {
            fn into(self) -> $t {
                self.0.map($t)
            }
        }

        impl Into<Optional> for Option<$t> {
            fn into(self) -> Optional {
                Optional(self.map(|value| value.0))
            }
        }
    };
}

implement!(ChannelId);
implement!(RoleId);
implement!(GuildId);
