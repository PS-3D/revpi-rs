mod util;

use self::util::{de_str_i, de_str_opt_i, ser_str_i, ser_str_i_padded_4, ser_str_opt_i};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub version: String,
    #[serde(rename = "saveTS")]
    pub save_ts: String,
    pub language: String,
    pub layout: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub inp_total: usize,
    pub out_total: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InOutMem {
    pub name: String,
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    pub default: u64,
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    pub bit_length: u8,
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    pub offset: u64,
    pub exported: bool,
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i_padded_4")]
    pub sort_pos: u16,
    pub comment: String,
    #[serde(deserialize_with = "de_str_opt_i", serialize_with = "ser_str_opt_i")]
    pub bit_position: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    #[serde(rename = "GUID")]
    pub guid: String,
    pub id: String,
    #[serde(rename = "type")]
    pub dev_type: String,
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    #[serde(rename = "productType")]
    pub product_type: u64,
    #[serde(deserialize_with = "de_str_i", serialize_with = "ser_str_i")]
    pub position: u64,
    pub name: String,
    pub bmk: String,
    #[serde(rename = "inpVariant")]
    pub inp_variant: u64,
    #[serde(rename = "outVariant")]
    pub out_variant: u64,
    pub comment: String,
    pub offset: u64,
    pub inp: BTreeMap<u64, InOutMem>,
    pub out: BTreeMap<u64, InOutMem>,
    pub mem: BTreeMap<u64, InOutMem>,
    pub extend: Value,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RSC {
    #[serde(rename = "App")]
    pub app: App,
    #[serde(rename = "Summary")]
    pub summary: Summary,
    #[serde(rename = "Devices")]
    pub devices: Vec<Device>,
}
