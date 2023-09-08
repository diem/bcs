use crate::error::{Error, Result};
use serde::de::{self, DeserializeSeed, IntoDeserializer, Visitor};
use std::io::Read;

use super::BcsDeserializer;

/// Deserializes BCS from a [`std::io::Read`]er
pub struct DeserializeReader<'de, R> {
    reader: TeeReader<'de, R>,
    max_remaining_depth: usize,
}

impl<'de, R: Read> DeserializeReader<'de, R> {
    /// Wraps the provided reader in  a new [`DeserializeReader`]
    fn new(reader: &'de mut R, max_remaining_depth: usize) -> Self {
        DeserializeReader {
            reader: TeeReader::new(reader),
            max_remaining_depth,
        }
    }
}

impl<'de, R: Read> BcsDeserializer for DeserializeReader<'de, R> {
    fn fill_slice(&mut self, slice: &mut [u8]) -> Result<()> {
        Ok(self.reader.read_exact(&mut slice[..])?)
    }

    fn max_remaining_depth(&mut self) -> usize {
        self.max_remaining_depth
    }

    fn max_remaining_depth_mut(&mut self) -> &mut usize {
        &mut self.max_remaining_depth
    }
}

impl<'de, R: Read> DeserializeReader<'de, R> {
    /// Parse a vector of bytes from the reader
    fn parse_vec(&mut self) -> Result<Vec<u8>> {
        let len = self.parse_length()?;
        let mut output = vec![0; len];
        self.fill_slice(&mut output)?;
        Ok(output)
    }

    /// Parse a String from the reader
    fn parse_string(&mut self) -> Result<String> {
        let bytes = self.parse_vec()?;
        String::from_utf8(bytes).map_err(|_| Error::Utf8)
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut DeserializeReader<'de, R> {
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
        visitor.visit_string(self.parse_string()?)
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
        visitor.visit_byte_buf(self.parse_vec()?)
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

struct SeqDeserializer<'a, 'de: 'a, R> {
    de: &'a mut DeserializeReader<'de, R>,
    remaining: usize,
}
#[allow(clippy::needless_borrow)]
impl<'a, 'de: 'a, R> SeqDeserializer<'a, 'de, R> {
    fn new(de: &'a mut DeserializeReader<'de, R>, remaining: usize) -> Self {
        Self { de, remaining }
    }
}

impl<'de, 'a, R: Read> de::SeqAccess<'de> for SeqDeserializer<'a, 'de, R> {
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

/// A reader that can optionally capture all bytes from an underlying [`Read`]er
pub struct TeeReader<'a, R> {
    reader: &'a mut R,
    capture_buffer: Option<Vec<u8>>,
}

impl<'a, R> TeeReader<'a, R> {
    /// Wrapse the provided reader in a new [`TeeReader`].
    pub fn new(reader: &'a mut R) -> Self {
        Self {
            reader,
            capture_buffer: Default::default(),
        }
    }
}

impl<'a, R: Read> Read for TeeReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.reader.read(buf)?;
        if let Some(ref mut buffer) = self.capture_buffer {
            buffer.extend_from_slice(&buf[..bytes_read]);
        }
        Ok(bytes_read)
    }
}

struct MapDeserializer<'a, 'de: 'a, R> {
    de: &'a mut DeserializeReader<'de, R>,
    remaining: usize,
    previous_key_bytes: Option<Vec<u8>>,
}

impl<'a, 'de, R: Read> MapDeserializer<'a, 'de, R> {
    fn new(de: &'a mut DeserializeReader<'de, R>, remaining: usize) -> Self {
        Self {
            de,
            remaining,
            previous_key_bytes: None,
        }
    }
}

impl<'de, 'a, R: Read> de::MapAccess<'de> for MapDeserializer<'a, 'de, R>
where
    'de: 'a,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.remaining.checked_sub(1) {
            None => Ok(None),
            Some(remaining) => {
                self.de.reader.capture_buffer = Some(Vec::new());
                let key_value = seed.deserialize(&mut *self.de)?;
                let key_bytes = self.de.reader.capture_buffer.take().unwrap();

                if let Some(ref previous_key_bytes) = self.previous_key_bytes {
                    if previous_key_bytes.as_slice() >= key_bytes.as_slice() {
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

impl<'a, 'de: 'a, R: Read> de::EnumAccess<'de> for &'a mut DeserializeReader<'de, R> {
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

impl<'a, 'de: 'a, R: Read> de::VariantAccess<'de> for &'a mut DeserializeReader<'de, R> {
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
