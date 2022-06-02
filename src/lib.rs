//! Everything you need for your RevPi
//!
//! [`rsc`] provides datastructures to read and write rsc files:
//! ```no_run
//! use revpi::rsc::RSC;
//! use serde_json;
//! use std::fs::File;
//!
//! let f = File::open("/etc/revpi/config.rsc").unwrap();
//! let rsc: RSC = serde_json::from_reader(f).unwrap();
//! println!("{:?}", rsc);
//! ```
//!
//! [`picontrol`] gives you resources for IO with the RevPi:
//! ```no_run
//! # use revpi::picontrol::{PiControl, Value};
//! let pi = PiControl::new().unwrap();
//! pi.set_value("RevPiLED", Value::Byte(42)).unwrap();
//! ```
//! The [`revpi!`](revpi_macro) and [`revpi_from_json!`](revpi_macro) macros
//! provide the same functionality, but faster because the name doesn't have
//! to be looked up every time.
//!
//! # Features
//! This crate has features to enable or disable the [macros](revpi_macro) and
//! [rsc]. [rsc] is enabled by default, while [macro](revpi_macro) is not.

pub mod picontrol;
#[cfg(feature = "macro")]
pub use revpi_macro::{revpi, revpi_from_json};
#[cfg(feature = "rsc")]
pub use revpi_rsc as rsc;
pub(crate) mod util;
