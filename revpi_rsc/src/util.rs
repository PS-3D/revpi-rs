use serde::{
    de::{Error as DeError, Visitor},
    Deserializer, Serializer,
};
use std::{fmt::Display, marker::PhantomData, str::FromStr};

pub struct IVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for IVisitor<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string with form \"<integer>\"")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        v.parse().map_err(DeError::custom)
    }
}

// unfortunately we have to implement these custom deserializers because
// KUNBUS chose to wrap some integer types into strings
pub fn de_str_i<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    deserializer.deserialize_str(IVisitor {
        marker: PhantomData,
    })
}

pub struct OptIVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for OptIVisitor<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string with form \"<integer>\" or \"\"")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if v.is_empty() {
            Ok(None)
        } else {
            v.parse::<T>().map(|i| Some(i)).map_err(DeError::custom)
        }
    }
}

// unfortunately we have to implement these custom deserializers because
// KUNBUS chose to wrap some integer types into strings, which can be empty
pub fn de_str_opt_i<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    deserializer.deserialize_str(OptIVisitor {
        marker: PhantomData,
    })
}

// serializes integer wrapped in string
pub fn ser_str_i<S, T>(i: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    serializer.serialize_str(&format!("{}", i))
}
