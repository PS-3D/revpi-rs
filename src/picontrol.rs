pub mod raw;

use self::raw::{raw::SPIVariable, Bit, PiControlRaw};
use crate::util::ensure;
use std::{
    ffi::{self, CString},
    io,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PiControlError {
    /// If an argument given to a method of PiControlRaw was invalid, e.g. too big
    /// or too small
    #[error("{0} was invalid")]
    InvalidArgument(&'static str),
    /// Returned by [`PiControlRaw::get_device_info`] if the requested device
    /// wasn't found
    #[error("Device with address {0} not found")]
    DeviceNotFound(u8),
    /// Returned by [`PiControlRaw::find_variable`] if there were no variable
    /// entries at all
    #[error("No variable entries")]
    NoVarEntries,
    /// Wrapper around [`io::Error`]
    #[error(transparent)]
    IoError(#[from] io::Error),
    /// Wrapper around [`ffi::NulError`]
    #[error(transparent)]
    NulError(#[from] ffi::NulError),
}

#[derive(Debug)]
pub struct PiControl {
    inner: PiControlRaw,
}

#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Value {
    Bit(bool),
    Byte(u8),
    Word(u16),
    DWord(u32),
}

impl Value {
    pub fn bitcnt(&self) -> usize {
        use Value::*;
        match self {
            Bit(_) => 1,
            Byte(_) => u8::BITS as usize,
            Word(_) => u16::BITS as usize,
            DWord(_) => u32::BITS as usize,
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bit(b)
    }
}

impl From<u8> for Value {
    fn from(b: u8) -> Self {
        Value::Byte(b)
    }
}

impl From<u16> for Value {
    fn from(w: u16) -> Self {
        Value::Word(w)
    }
}

impl From<u32> for Value {
    fn from(d: u32) -> Self {
        Value::DWord(d)
    }
}

impl PiControl {
    pub fn new() -> Result<Self, PiControlError> {
        Ok(Self {
            inner: PiControlRaw::new()?,
        })
    }

    fn find_variable(&self, name: &str) -> Result<SPIVariable, PiControlError> {
        self.inner
            .find_variable(&CString::new(name).map_err(PiControlError::from)?)
    }

    pub fn set_value(&self, name: &str, value: Value) -> Result<(), PiControlError> {
        let name = self.find_variable(name)?;
        ensure!(
            name.i16uLength as usize == value.bitcnt(),
            PiControlError::InvalidArgument("value or str")
        );
        match value {
            Value::Bit(b) => unsafe {
                self.inner
                    .set_bit(name.i16uAddress, Bit::from(name.i8uBit), b)
            },
            Value::Byte(b) => unsafe { self.inner.set_byte(name.i16uAddress, b) },
            Value::Word(w) => unsafe { self.inner.set_word(name.i16uAddress, w) },
            Value::DWord(d) => unsafe { self.inner.set_dword(name.i16uAddress, d) },
        }
    }

    pub fn get_value(&self, name: &str) -> Result<Value, PiControlError> {
        let name = self.find_variable(name)?;
        match name.i16uLength {
            1 => unsafe { self.inner.get_bit(name.i16uAddress, Bit::from(name.i8uBit)) }
                .map(Value::from),
            8 => unsafe { self.inner.get_byte(name.i16uAddress) }.map(Value::from),
            16 => unsafe { self.inner.get_word(name.i16uAddress) }.map(Value::from),
            32 => unsafe { self.inner.get_dword(name.i16uAddress) }.map(Value::from),
            _ => panic!("invalid bitlength from piControl"),
        }
    }
}
