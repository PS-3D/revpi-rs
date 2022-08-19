//! Provides macros to automatically generate functions from the rsc
//!
//! The macros [`revpi!`] and [`revpi_from_json!`] generate functions to read and
//! write values by name directly from an rsc file. This is faster than
//! looking up the address everytime while still being safe.
//!
//! [`revpi!`] and [`revpi_from_json!`] basically both do the same. The only
//! difference is that [`revpi!`] reads the config from the standard locations
//! (`"/etc/revpi/config.rsc"` or `"/opt/KUNBUS/config.rsc"`) while
//! [`revpi_from_json!`] reads it from a given path.
//!
//! # Usage
//! [`revpi!`] just needs the name of the struct it should produce, while
//! [`revpi_from_json!`] also needs a path to an rsc file.
//!
//! # Output
//! Both output a struct with the given name. That struct contains functions
//! of the form\
//! `get_<name>` and `set_<name>`, where `<name>` is the name given
//! to the field in PiCtory. Inputs only have getters, while outputs and memory
//! fields also have setters.
//! ## Getters
//! Getters need no arguments and their return value depends on the type of
//! the field they read out. Getters return `Result<<type>, PiControlError>`
//! where `<type>` is the type of the field they read out. So a getter could look
//! like this:
//! ```ignore
//! pub fn get_RevPiStatus() -> Result<u8, PiControlError> {...}
//! ```
//! ## Setters
//! Setters take an argument, the type of which depends on the type of field they
//! set. They return `Result<(), PiControlError>`. So a setter could look like
//! this:
//! ```ignore
//! pub fn set_RevPiLED(byte: u8) -> Result<(), PiControlError> {...}
//! ```
//!
//! # Examples
//! Let's assume the file `/etc/revpi/config.rsc` of the RevPi looks like this:
//! ```json
//! {
//!   "App": {
//!     "name": "PiCtory",
//!     "version": "2.0.6",
//!     "saveTS": "20220523193431",
//!     "language": "en",
//!     "layout": {},
//!   "Summary": {
//!     "inpTotal": 96,
//!     "outTotal": 27
//!   },
//!   "Devices": [
//!     {
//!       "GUID": "deadbeef-1337-e35a-bf89-4242deadbeef",
//!       "id": "device_RevPiCore_20190503_2_3_007",
//!       "type": "BASE",
//!       "productType": "95",
//!       "position": "0",
//!       "name": "RevPi Core/3/3+/S",
//!       "bmk": "RevPi Core/3/3+/S",
//!       "inpVariant": 0,
//!       "outVariant": 0,
//!       "comment": "This is a RevPiCore Device",
//!       "offset": 0,
//!       "inp": {
//!         "0": [
//!           "RevPiStatus",
//!           "0",
//!           "8",
//!           "0",
//!           true,
//!           "0000",
//!           "",
//!           ""
//!         ]
//!       },
//!       "out": {
//!         "0": [
//!           "RevPiLED",
//!           "0",
//!           "8",
//!           "1",
//!           true,
//!           "0001",
//!           "",
//!           ""
//!         ],
//!         "1": [
//!           "RS485ErrorLimit1",
//!           "10",
//!           "16",
//!           "2",
//!           false,
//!           "0002",
//!           "",
//!           ""
//!         ],
//!       },
//!       "mem": {},
//!       "extend": {}
//!     }
//!   ],
//!   "Connections": []
//! }
//! ```
//! `revpi!(RevPi)` and `revpi_from_json!(RevPi, "/etc/revpi/config.rsc")` both
//! would then yield the folliowing code:
//! ```ignore
//! struct RevPi {...}
//!
//! impl RevPi {
//!     pub fn get_RevPiStatus() -> Result<u8, PiControlError> {...}
//!     pub fn get_RevPiLED() -> Result<u8, PiControlError> {...}
//!     pub fn set_RevPiLED(byte: u8) -> Result<(), PiControlError> {...}
//!     pub fn get_RS485ErrorLimit1() -> Result<u16, PiControlError> {...}
//!     pub fn set_RS485ErrorLimit1(word: u16) -> Result<(), PiControlError> {...}
//! }
//! ```
//!

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use revpi_rsc::{InOutMem, RSC};
use serde_json;
use std::fs::File;
use syn::{parse::Parse, parse_macro_input, Ident, LitStr, Token};

struct JsonInput {
    name: Ident,
    path: LitStr,
}

impl Parse for JsonInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token!(,)>()?;
        let path = input.parse()?;
        Ok(JsonInput { name, path })
    }
}

// produces a getter of the given InOutMem
// since InOutMem only contains the offset inside the module, we also need
// the module offset
fn get_fn(mod_offset: u64, item: &InOutMem) -> TokenStream2 {
    let name = format_ident!("get_{}", item.name);
    let address = (mod_offset + item.offset) as u16;
    let (ret, function, fnargs) = match item.bit_length {
        1 => (
            "bool",
            "get_bit",
            format!("{}, {}", address, item.bit_position.unwrap()),
        ),
        8 => ("u8", "get_byte", format!("{}", address)),
        16 => ("u16", "get_word", format!("{}", address)),
        32 => ("u32", "get_dword", format!("{}", address)),
        _ => panic!("invalid bitlength"),
    };

    format!(
        "pub fn {}(&self) -> Result<{}, revpi::PiControlError> {{
    unsafe {{ self.inner.{}({}) }}
}}",
        name, ret, function, fnargs
    )
    .parse()
    .unwrap()
}

// produces a setter of the given InOutMem
// since InOutMem only contains the offset inside the module, we also need
// the module offset
fn set_fn(mod_offset: u64, item: &InOutMem) -> TokenStream2 {
    let name = format_ident!("set_{}", item.name);
    let address = (mod_offset + item.offset) as u16;
    let (args, function, fnargs) = match item.bit_length {
        1 => (
            "bit: bool",
            "set_bit",
            format!("{}, {}, bit", address, item.bit_position.unwrap()),
        ),
        8 => ("byte: u8", "set_byte", format!("{}, byte", address)),
        16 => ("word: u16", "set_word", format!("{}, word", address)),
        32 => ("dword: u32", "set_dword", format!("{}, dword", address)),
        _ => panic!("invalid bitlength"),
    };

    format!(
        "pub fn {}(&self, {}) -> Result<(), revpi::PiControlError> {{
    unsafe {{ self.inner.{}({}) }}
}}",
        name, args, function, fnargs
    )
    .parse()
    .unwrap()
}

// produce the struct and impl withe the given name from the given rsc
fn from_json(rsc: &RSC, name: Ident) -> TokenStream2 {
    let mut functions = TokenStream2::default();
    for d in rsc.devices.iter() {
        for i in d.inp.values() {
            functions.extend(get_fn(d.offset, i));
        }
        for o in d.out.values() {
            functions.extend(get_fn(d.offset, o));
            functions.extend(set_fn(d.offset, o));
        }
        for m in d.mem.values() {
            functions.extend(get_fn(d.offset, m));
            functions.extend(set_fn(d.offset, m));
        }
    }
    quote!(struct #name {
        inner: revpi::raw::PiControlRaw,
    }
    impl #name {
        pub fn new() -> Result<Self, revpi::PiControlError> {
            Ok(Self {
                inner: revpi::raw::PiControlRaw::new()?,
            })
        }

        #functions
    })
    .into()
}

/// See the [crate documentation](self)
#[proc_macro]
pub fn revpi_from_json(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as JsonInput);
    let f = File::open(input.path.value()).unwrap();
    let rsc: RSC = serde_json::from_reader(f).unwrap();
    from_json(&rsc, input.name).into()
}

/// See the [crate documentation](self)
#[proc_macro]
pub fn revpi(stream: TokenStream) -> TokenStream {
    let name = parse_macro_input!(stream as Ident);
    // on older models the file can still under /opt so we gotta check for that
    let f = match File::open("/etc/revpi/config.rsc") {
        Ok(f) => f,
        Err(_) => File::open("/opt/KUNBUS/config.rsc").unwrap(),
    };
    let rsc: RSC = serde_json::from_reader(f).unwrap();
    from_json(&rsc, name).into()
}
