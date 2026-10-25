//! First pass checks every token, including fields later refused by typed parsing.
//! No map/Value which could erase a duplicate is ever constructed.
use super::ManifestError;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::{collections::BTreeSet, fmt};

pub(crate) fn check(bytes: &[u8]) -> Result<(), ManifestError> {
    if bytes.len() > 65_536 {
        return Err(ManifestError::Budget);
    }
    let mut d = serde_json::Deserializer::from_slice(bytes);
    let mut remaining = 8192;
    Scan {
        depth: 0,
        remaining: &mut remaining,
    }
    .deserialize(&mut d)
    .map_err(|_| ManifestError::Wire)?;
    d.end().map_err(|_| ManifestError::Wire)
}
struct Scan<'a> {
    depth: usize,
    remaining: &'a mut usize,
}
impl<'de> DeserializeSeed<'de> for Scan<'_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        if self.depth > 16 || *self.remaining == 0 {
            return Err(de::Error::custom("budget"));
        }
        *self.remaining -= 1;
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Scan<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded source")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    // float/exponent and integers outside i64/u64 refuse, never round through f64.
    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, s: &str) -> Result<(), E> {
        if s.len() > 4096 {
            Err(E::custom("string budget"))
        } else {
            Ok(())
        }
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        let mut n = 0;
        while a
            .next_element_seed(Scan {
                depth: self.depth + 1,
                remaining: self.remaining,
            })?
            .is_some()
        {
            n += 1;
            if n > 1024 {
                return Err(de::Error::custom("array budget"));
            }
        }
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(k) = a.next_key::<String>()? {
            if k.len() > 128 || keys.len() >= 32 || !keys.insert(k) {
                return Err(de::Error::custom("object budget or duplicate"));
            }
            a.next_value_seed(Scan {
                depth: self.depth + 1,
                remaining: self.remaining,
            })?;
        }
        Ok(())
    }
}
