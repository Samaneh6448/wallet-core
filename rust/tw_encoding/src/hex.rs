// SPDX-License-Identifier: Apache-2.0
//
// Copyright © 2017 Trust Wallet.

pub use hex::FromHexError;
use std::ops::Deref;
use tw_memory::Data;

pub type FromHexResult<T> = Result<T, FromHexError>;

pub trait ToHex {
    fn to_hex(&self) -> String;

    fn to_hex_prefixed(&self) -> String;
}

impl<T> ToHex for T
where
    T: AsRef<[u8]>,
{
    fn to_hex(&self) -> String {
        encode(self, false)
    }

    fn to_hex_prefixed(&self) -> String {
        encode(self, true)
    }
}

pub trait DecodeHex {
    fn decode_hex(&self) -> FromHexResult<Data>;
}

impl<T> DecodeHex for T
where
    T: Deref<Target = str>,
{
    fn decode_hex(&self) -> FromHexResult<Data> {
        decode(self.deref())
    }
}

/// Decodes the given hexadecimal string.
pub fn decode(data: &str) -> FromHexResult<Data> {
    let hex_string = data.trim_start_matches("0x");
    hex::decode(hex_string)
}

/// Decodes the given hexadecimal string leniently allowing to pass odd number of chars.
/// For example, `0x0` is extended to `0x00`, `0x123` is extended to `0x0123`.
pub fn decode_lenient(data: &str) -> FromHexResult<Data> {
    let hex_string = data.trim_start_matches("0x");
    if hex_string.len() % 2 == 0 {
        hex::decode(hex_string)
    } else {
        // Insert a leading 0.
        let standard_hex = format!("0{hex_string}");
        hex::decode(standard_hex)
    }
}

pub fn encode<T: AsRef<[u8]>>(data: T, prefixed: bool) -> String {
    let encoded = hex::encode(data.as_ref());
    if prefixed {
        return format!("0x{encoded}");
    }
    encoded
}

pub mod as_hex {
    use super::*;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;

    /// Helps to serialize certain types as hex.
    /// For example,
    /// ```rust
    /// # use serde::Serialize;
    /// # use tw_encoding::hex::as_hex::AsHex;
    ///
    /// #[derive(Serialize)]
    /// struct Foo {
    ///   // Compile error (as_hex doesn't work for optional):
    ///   // #[serde(with = "tw_encoding::hex::as_hex")]
    ///   // data: Option<Vec<u8>>,
    ///   data: Option<AsHex<Vec<u8>>>,
    /// }
    /// ```
    ///
    /// Consider using `#[serde(with = "as_hex")]` when possible.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct AsHex<T>(pub T);

    impl<T: ToHex> Serialize for AsHex<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serialize(&self.0, serializer)
        }
    }

    impl<'de, T, E> Deserialize<'de> for AsHex<T>
    where
        T: for<'a> TryFrom<&'a [u8], Error = E>,
        E: fmt::Debug,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize(deserializer).map(AsHex)
        }
    }

    /// Serializes the `value` as a hex.
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ToHex,
        S: Serializer,
    {
        value.to_hex().serialize(serializer)
    }

    pub fn deserialize<'de, D, T, E>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: for<'a> TryFrom<&'a [u8], Error = E>,
        E: fmt::Debug,
    {
        let s = String::deserialize(deserializer)?;
        let data = decode(&s).map_err(|e| Error::custom(format!("{e:?}")))?;
        T::try_from(&data).map_err(|e| Error::custom(format!("Error parsing from bytes: {e:?}")))
    }
}

pub mod as_hex_prefixed {
    use crate::hex::encode;
    use serde::{Deserializer, Serialize, Serializer};
    use std::fmt;

    /// Serializes the `value` as a `0x` prefixed hex.
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: AsRef<[u8]>,
        S: Serializer,
    {
        encode(value, true).serialize(serializer)
    }

    pub fn deserialize<'de, D, T, E>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: for<'a> TryFrom<&'a [u8], Error = E>,
        E: fmt::Debug,
    {
        // `as_hex::deserialize` handles the prefix already.
        super::as_hex::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_encode_prefixed() {
        let v: &[u8] = &[
            45, 181, 0, 172, 145, 156, 221, 227, 81, 172, 54, 227, 113, 29, 131, 44, 109, 185, 118,
            105,
        ];
        assert_eq!(
            encode(v, true),
            "0x2db500ac919cdde351ac36e3711d832c6db97669"
        );
    }

    #[test]
    fn test_vec_decode_prefixed() {
        let expected = vec![
            45, 181, 0, 172, 145, 156, 221, 227, 81, 172, 54, 227, 113, 29, 131, 44, 109, 185, 118,
            105,
        ];
        assert_eq!(
            decode("0x2db500ac919cdde351ac36e3711d832c6db97669"),
            Ok(expected)
        );
    }
}
