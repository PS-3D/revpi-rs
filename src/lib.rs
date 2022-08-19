//! Everything you need for your RevPi
//!
//! ## IO
//! [`PiControl`] gives you you basic, safe IO with the RevPi:
//! ```no_run
//! # use revpi::picontrol::{PiControl, Value};
//! let pi = PiControl::new().unwrap();
//! pi.set_value("RevPiLED", Value::Byte(42)).unwrap();
//! ```
//!
//! If you want to do more advanced, unsafe stuff like replaceing the whole
//! processimage or updating the whole firmware, you might be better off with
//! the [`raw`] module, which, contrary to [`PiControl`], provides access to *all*
//! RevPi functions:
//! ```no_run
//! # use revpi::picontrol::raw::{PiControlRaw};
//! let raw = PiControlRaw::new().unwrap();
//! unsafe { raw.update_device_firmware(31) };
//! ```
//! Usually, PiControl is enough though.
//!
//! Lastly, [`raw::raw`] provides the raw ioctl bindings needed for IO with the
//! RevPi.
//!
//! ## RSC
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
//! Note that this is only available with feature `rsc`.
//!
//! ## Macros
//! The [`revpi!`](revpi_macro) and [`revpi_from_json!`](revpi_macro) macros
//! provide the same functionality, but faster because the name doesn't have
//! to be looked up every time.
//! Note that this is only available with feature `macro`.
//!
//! # Features
//! * `rsc` will enable the ability to parse rsc files, see [`revpi_rsc`]
//! * `macro` will enable the [`revpi`] and [`revpi_from_json`] macros

mod picontrol;
pub(crate) mod util;

pub use picontrol::*;
#[cfg(any(feature = "macro"))]
pub use revpi_macro::{revpi, revpi_from_json};
#[cfg(feature = "rsc")]
pub use revpi_rsc as rsc;
