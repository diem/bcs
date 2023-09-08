use crate::error::{Error, Result};
use serde::de::{self, Deserialize, DeserializeSeed, IntoDeserializer, Visitor};

use super::BcsDeserializer;

/// Deserializes a `&[u8]` into a type.
///
/// This function will attempt to interpret `bytes` as the BCS serialized form of `T` and
/// deserialize `T` from `bytes`.
///
/// # Examples
///
/// ```
/// use bcs::from_bytes;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Ip([u8; 4]);
///
/// #[derive(Deserialize)]
/// struct Port(u16);
///
/// #[derive(Deserialize)]
/// struct SocketAddr {
///     ip: Ip,
///     port: Port,
/// }
///
/// let bytes = vec![0x7f, 0x00, 0x00, 0x01, 0x41, 0x1f];
/// let socket_addr: SocketAddr = from_bytes(&bytes).unwrap();
///
/// assert_eq!(socket_addr.ip.0, [127, 0, 0, 1]);
/// assert_eq!(socket_addr.port.0, 8001);
/// ```
pub fn from_bytes<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(bytes, crate::MAX_CONTAINER_DEPTH);
    let t = T::deserialize(&mut deserializer)?;
    deserializer.end().map(move |_| t)
}

/// Perform a stateful deserialization from a `&[u8]` using the provided `seed`.
pub fn from_bytes_seed<'a, T>(seed: T, bytes: &'a [u8]) -> Result<T::Value>
where
    T: DeserializeSeed<'a>,
{
    let mut deserializer = Deserializer::new(bytes, crate::MAX_CONTAINER_DEPTH);
    let t = seed.deserialize(&mut deserializer)?;
    deserializer.end().map(move |_| t)
}

impl<'de> BcsDeserializer for Deserializer<'de> {
    fn next(&mut self) -> Result<u8> {
        let byte = self.peek()?;
        self.input = &self.input[1..];
        Ok(byte)
    }

    fn max_remaining_depth(&mut self) -> usize {
        self.max_remaining_depth
    }

    fn max_remaining_depth_mut(&mut self) -> &mut usize {
        &mut self.max_remaining_depth
    }

    fn fill_slice(&mut self, slice: &mut [u8]) -> Result<()> {
        for byte in slice {
            *byte = self.next()?;
        }
        Ok(())
    }
}

/// Deserialization implementation for BCS
struct Deserializer<'de> {
    input: &'de [u8],
    max_remaining_depth: usize,
}

impl<'de> Deserializer<'de> {
    /// Creates a new `Deserializer` which will be deserializing the provided
    /// input.
    fn new(input: &'de [u8], max_remaining_depth: usize) -> Self {
        Deserializer {
            input,
            max_remaining_depth,
        }
    }

    /// The `Deserializer::end` method should be called after a type has been
    /// fully deserialized. This allows the `Deserializer` to validate that
    /// the there are no more bytes remaining in the input stream.
    fn end(&mut self) -> Result<()> {
        if self.input.is_empty() {
            Ok(())
        } else {
            Err(Error::RemainingInput)
        }
    }
}

impl<'de> Deserializer<'de> {
    fn peek(&mut self) -> Result<u8> {
        self.input.first().copied().ok_or(Error::Eof)
    }

    fn parse_string_borrowed(&mut self) -> Result<&'de str> {
        let slice = self.parse_bytes_borrowed()?;
        std::str::from_utf8(slice).map_err(|_| Error::Utf8)
    }

    fn parse_bytes_borrowed(&mut self) -> Result<&'de [u8]> {
        let len = self.parse_length()?;
        let slice = self.input.get(..len).ok_or(Error::Eof)?;
        self.input = &self.input[len..];
        Ok(slice)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // BCS is not a self-describing format so we can't implement `deserialize_any`
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_any"))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_u8()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_u16()? as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_u32()? as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_u64()? as i64)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i128(self.parse_u128()? as i128)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_u16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_u32()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_u64()?)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u128(self.parse_u128()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_f32"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_f64"))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_char"))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string_borrowed()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.parse_bytes_borrowed()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let byte = self.next()?;

        match byte {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(Error::ExpectedOption),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.enter_named_container(name)?;
        let r = self.deserialize_unit(visitor);
        self.leave_named_container();
        r
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.enter_named_container(name)?;
        let r = visitor.visit_newtype_struct(&mut *self);
        self.leave_named_container();
        r
    }
    #[allow(clippy::needless_borrow)]
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.parse_length()?;
        visitor.visit_seq(SeqDeserializer::new(&mut self, len))
    }
    #[allow(clippy::needless_borrow)]
    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new(&mut self, len))
    }
    #[allow(clippy::needless_borrow)]
    fn deserialize_tuple_struct<V>(
        mut self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.enter_named_container(name)?;
        let r = visitor.visit_seq(SeqDeserializer::new(&mut self, len));
        self.leave_named_container();
        r
    }
    #[allow(clippy::needless_borrow)]
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.parse_length()?;
        visitor.visit_map(MapDeserializer::new(&mut self, len))
    }
    #[allow(clippy::needless_borrow)]
    fn deserialize_struct<V>(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.enter_named_container(name)?;
        let r = visitor.visit_seq(SeqDeserializer::new(&mut self, fields.len()));
        self.leave_named_container();
        r
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.enter_named_container(name)?;
        let r = visitor.visit_enum(&mut *self);
        self.leave_named_container();
        r
    }

    // BCS does not utilize identifiers, so throw them away
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(_visitor)
    }

    // BCS is not a self-describing format so we can't implement `deserialize_ignored_any`
    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_ignored_any"))
    }

    // BCS is not a human readable format
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct SeqDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
}
#[allow(clippy::needless_borrow)]
impl<'a, 'de> SeqDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, remaining: usize) -> Self {
        Self { de, remaining }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            Ok(None)
        } else {
            self.remaining -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

struct MapDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
    previous_key_bytes: Option<&'a [u8]>,
}

impl<'a, 'de> MapDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, remaining: usize) -> Self {
        Self {
            de,
            remaining,
            previous_key_bytes: None,
        }
    }
}

impl<'de, 'a> de::MapAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.remaining.checked_sub(1) {
            None => Ok(None),
            Some(remaining) => {
                let previous_input_slice = self.de.input;
                let key_value = seed.deserialize(&mut *self.de)?;
                let key_len = previous_input_slice
                    .len()
                    .saturating_sub(self.de.input.len());
                let key_bytes = &previous_input_slice[..key_len];
                if let Some(previous_key_bytes) = self.previous_key_bytes {
                    if previous_key_bytes >= key_bytes {
                        return Err(Error::NonCanonicalMap);
                    }
                }
                self.remaining = remaining;
                self.previous_key_bytes = Some(key_bytes);
                Ok(Some(key_value))
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

impl<'de, 'a> de::EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant_index = self.parse_u32_from_uleb128()?;
        let result: Result<V::Value> = seed.deserialize(variant_index.into_deserializer());
        Ok((result?, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}