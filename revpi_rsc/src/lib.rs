//! This module provides structs for parsing a rsc file
//!
//! rsc files are used by the RevPi for its configuration. The documentation can
//! be found [here](https://revolutionpi.de/tabellarische-auflistung-aller-json-attribute-einer-rsc-datei/).
//! The running config can be found under `"/etc/revpi/config.rsc"` or, on older
//! variants, under `"/opt/KUNBUS/config.rsc"`.
//!
//! # Usage
//! Every struct implements the [`Serialize`] and [`Deserialize`] trait from
//! [Serde](https://serde.rs/). This means the config can easily be read in from
//! e.g. a rsc file, using [`serde_json`]
//! ```
//! use revpi_rsc::RSC;
//! use serde_json;
//! use std::fs::File;
//!
//! let f = File::open("/etc/revpi/config.rsc").unwrap();
//! let rsc: RSC = serde_json::from_reader(f).unwrap();
//! println!("{:?}", rsc);
//! ```

mod util;

use self::util::{de_str_i, de_str_opt_i, ser_str_i, ser_str_i_padded_4, ser_str_opt_i};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

// unfortunately we have to implement custom serializers and deserializers because
// KUNBUS chose to wrap some integer types into strings, which can even be empty
// for some values

/// Representing the app
///
/// That means this is a struct for ID A in the [documentation](https://revolutionpi.de/tabellarische-auflistung-aller-json-attribute-einer-rsc-datei/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct App {
    /// ID A.1
    pub name: String,
    /// ID A.2
    pub version: String,
    /// ID A.3
    #[serde(rename = "saveTS")]
    pub save_ts: String,
    /// ID A.4
    pub language: String,
    /// ID A.5
    ///
    /// Lower layers are omitted due to there being no need for them as of yet
    pub layout: Value,
}

/// Representing the summary
///
/// That means this is a struct for ID B in the [documentation](https://revolutionpi.de/tabellarische-auflistung-aller-json-attribute-einer-rsc-datei/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    /// ID B.1
    pub inp_total: usize,
    /// ID B.2
    pub out_total: usize,
}

/// Representing the list found under `inp`, `out` and `mem`
///
/// That means this is a struct for ID C.13, C.14 and C.15 in the
/// [documentation](https://revolutionpi.de/tabellarische-auflistung-aller-json-attribute-einer-rsc-datei/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InOutMem {
    /// IDs C13.2, C14.2 and C15.2
    pub name: String,
    /// IDs C13.3, C14.3 and C15.3
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    pub default: u64,
    /// IDs C13.4, C14.4 and C15.4
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    pub bit_length: u8,
    /// IDs C13.5, C14.5 and C15.5
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    pub offset: u64,
    /// IDs C13.6, C14.6 and C15.6
    pub exported: bool,
    /// IDs C13.7, C14.7 and C15.7
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i_padded_4")]
    pub sort_pos: u16,
    /// IDs C13.8, C14.8 and C15.8
    pub comment: String,
    /// IDs C13.9, C14.9 and C15.9
    #[serde(deserialize_with = "de_str_opt_i", serialize_with = "ser_str_opt_i")]
    pub bit_position: Option<u8>,
}

/// Representing a singular device
///
/// That means this is a struct for section C in the [documentation](https://revolutionpi.de/tabellarische-auflistung-aller-json-attribute-einer-rsc-datei/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Device {
    /// ID C.2
    #[serde(rename = "GUID")]
    pub guid: String,
    /// ID C.3
    pub id: String,
    /// ID C.4
    #[serde(rename = "type")]
    pub dev_type: String,
    /// ID C.5
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    #[serde(rename = "productType")]
    pub product_type: u64,
    /// ID C.6
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    pub position: u64,
    /// ID C.7
    pub name: String,
    /// ID C.8
    pub bmk: String,
    /// ID C.9
    #[serde(rename = "inpVariant")]
    pub inp_variant: u64,
    /// ID C.10
    #[serde(rename = "outVariant")]
    pub out_variant: u64,
    /// ID C.11
    pub comment: String,
    /// ID C.12
    pub offset: u64,
    /// ID C.13
    pub inp: BTreeMap<u64, InOutMem>,
    /// ID C.14
    pub out: BTreeMap<u64, InOutMem>,
    /// ID C.15
    pub mem: BTreeMap<u64, InOutMem>,
    /// ID C.16
    ///
    /// Lower layers are omitted due to there being no documentation for them
    pub extend: Value,
    /// has no id
    pub active: Option<bool>,
}

/// Struct of the whole RSC file
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RSC {
    /// ID A
    #[serde(rename = "App")]
    pub app: App,
    /// ID B
    #[serde(rename = "Summary")]
    pub summary: Summary,
    /// ID C
    #[serde(rename = "Devices")]
    pub devices: Vec<Device>,
}
