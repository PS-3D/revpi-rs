use std::{marker::PhantomData, str::FromStr, fmt::Display};
use serde::{
    de::{Error as DeError, Visitor},
    Deserializer,
    Serializer,
};

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

pub fn de_str_i<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    deserializer.deserialize_str(IVisitor { marker: PhantomData })
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

pub fn de_str_opt_i<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    deserializer.deserialize_str(OptIVisitor { marker: PhantomData })
}

pub fn ser_str_i_padded_4<S, T>(i: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    serializer.serialize_str(&format!("{:0>4}", i))
}

pub fn ser_str_i<S, T>(i: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    serializer.serialize_str(&format!("{}", i))
}

pub fn ser_str_opt_i<S, T>(o: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    if let Some(i) = o {
        ser_str_i(&i, serializer)
    } else {
        serializer.serialize_str("")
    }
}
