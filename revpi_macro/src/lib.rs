use std::fs::File;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use revpi_rsc::{InOutMem, RSC};
use serde_json;
use syn::{parse::Parse, parse_macro_input, Ident, LitStr};

struct JsonInput {
    path: LitStr,
    name: Ident,
}

impl Parse for JsonInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(JsonInput {
            path: input.parse()?,
            name: input.parse()?,
        })
    }
}

fn u8_to_bit(b: u8) -> String {
    let bit = match b {
        0 => "Zero",
        1 => "One",
        2 => "Two",
        3 => "Three",
        4 => "Four",
        5 => "Five",
        6 => "Six",
        7 => "Seven",
        _ => panic!("integer out of range of enum"),
    };
    format!("revpi::picontrol::raw::Bit::{}", bit)
}

fn get_fn(mod_offset: u64, item: &InOutMem) -> TokenStream2 {
    let name = format_ident!("get_{}", item.name);
    let address = (mod_offset + item.offset) as u16;
    let (ret, function, fnargs) = match item.bit_length {
        1 => (
            "bool",
            "get_bit",
            format!("{}, {}", address, u8_to_bit(item.bit_position.unwrap())),
        ),
        8 => ("u8", "get_byte", format!("{}", address)),
        16 => ("u16", "get_word", format!("{}", address)),
        32 => ("u32", "get_dword", format!("{}", address)),
        _ => panic!("invalid bitlength"),
    };

    format!(
        "pub fn {}() -> Result<{}, revpi::picontrol::raw::PiControRawError> {{
    self.inner.{}({})
}}",
        name, ret, function, fnargs
    )
    .parse()
    .unwrap()
}

fn set_fn(mod_offset: u64, item: &InOutMem) -> TokenStream2 {
    let name = format_ident!("set_{}", item.name);
    let address = (mod_offset + item.offset) as u16;
    let (args, function, fnargs) = match item.bit_length {
        1 => (
            "bit: bool",
            "set_bit",
            format!(
                "{}, {}, bit",
                address,
                u8_to_bit(item.bit_position.unwrap())
            ),
        ),
        8 => ("byte: u8", "set_byte", format!("{}, byte", address)),
        16 => ("word: u16", "set_word", format!("{}, word", address)),
        32 => ("dword: u32", "set_dword", format!("{}, dword", address)),
        _ => panic!("invalid bitlength"),
    };

    format!(
        "pub fn {}({}) -> Result<(), revpi::picontrol::raw::PiControlRawError> {{
    self.inner.{}({})
}}",
        name, args, function, fnargs
    )
    .parse()
    .unwrap()
}

fn from_json(rsc: &RSC, name: Ident) -> TokenStream2 {
    let mut functions = TokenStream2::default();
    for d in rsc.devices.iter() {
        for i in d.inp.values() {
            functions.extend(get_fn(d.offset, i));
        }
        for o in d.inp.values() {
            functions.extend(get_fn(d.offset, o));
            functions.extend(set_fn(d.offset, o));
        }
        for m in d.mem.values() {
            functions.extend(get_fn(d.offset, m));
            functions.extend(set_fn(d.offset, m));
        }
    }
    quote!(struct #name {
        inner: revpi::picontrol::raw::PiControlRaw,
    }
    impl #name {
        pub fn new() -> Result<Self, revpi::picontrol::raw::PiControlRawError> {
            Ok(Self {
                inner: revpi::picontrol::raw::PiControlRaw::new()?,
            })
        }

        #functions
    })
    .into()
}

#[proc_macro]
pub fn revpi_from_json(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as JsonInput);
    let f = File::open(input.path.value()).unwrap();
    let rsc: RSC = serde_json::from_reader(f).unwrap();
    from_json(&rsc, input.name).into()
}

#[proc_macro]
pub fn revpi(stream: TokenStream) -> TokenStream {
    let name = parse_macro_input!(stream as Ident);
    let f = match File::open("/etc/revpi/config.rsc") {
        Ok(f) => f,
        Err(_) => File::open("/opt/KUNBUS/config.rsc").unwrap(),
    };
    let rsc: RSC = serde_json::from_reader(f).unwrap();
    from_json(&rsc, name).into()
}
