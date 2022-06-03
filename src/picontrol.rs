//! Provides IO functionality for the RevPi
//!
//! Basic, safe IO is provided by [`PiControl`]:
//! ```no_run
//! # use revpi::picontrol::{PiControl, Value};
//! let pi = PiControl::new().unwrap();
//! pi.set_value("RevPiLED", Value::Byte(42)).unwrap();
//! ```
//!
//! If you want to do more advanced also unsafe stuff like replaceing the whole
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

pub mod raw;

use self::raw::{raw::SPIVariable, PiControlRaw};
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

/// Value that can be set or read from the revpi
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Value {
    Bit(bool),
    Byte(u8),
    Word(u16),
    DWord(u32),
}

impl Value {
    /// Returns the number of bits this value occupies in the processimage
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
    /// Returns a [`Value::Bit`] encapsulating the given bool
    fn from(b: bool) -> Self {
        Value::Bit(b)
    }
}

impl From<u8> for Value {
    /// Returns a [`Value::Byte`] encapsulating the given u8
    fn from(b: u8) -> Self {
        Value::Byte(b)
    }
}

impl From<u16> for Value {
    /// Returns a [`Value::Word`] encapsulating the given u16
    fn from(w: u16) -> Self {
        Value::Word(w)
    }
}

impl From<u32> for Value {
    /// Returns a [`Value::DWord`] encapsulating the given u32
    fn from(d: u32) -> Self {
        Value::DWord(d)
    }
}

/// Provides safe RevPi IO
#[derive(Debug)]
pub struct PiControl {
    inner: PiControlRaw,
}

impl PiControl {
    /// Creates a new PiControl object
    ///
    /// # Errors
    /// Will return a [`PiControlError::IoError`] if the processimage can't be
    /// opened
    ///
    /// # Example
    /// ```no_run
    /// # use revpi::picontrol::PiControl;
    /// let pi = PiControl::new().unwrap();
    /// ```
    pub fn new() -> Result<Self, PiControlError> {
        Ok(Self {
            inner: PiControlRaw::new()?,
        })
    }

    fn find_variable(&self, name: &str) -> Result<SPIVariable, PiControlError> {
        self.inner
            .find_variable(&CString::new(name).map_err(PiControlError::from)?)
    }

    /// Sets the given value in the processimage. `name` is the name given to the
    /// field that should be written to in PiCtory.
    ///
    /// # Errors
    /// If the length found in the name lookup and the length of `value` don't
    /// match, a [`PiControlError::InvalidArgument`] is returned. Same thing if
    /// the name can't be found
    ///
    /// # Example
    /// ```no_run
    /// # use revpi::picontrol::{PiControl, Value};
    /// let pi = PiControl::new().unwrap();
    /// pi.set_value("RevPiLED", Value::Byte(42)).unwrap();
    /// ```
    pub fn set_value(&self, name: &str, value: Value) -> Result<(), PiControlError> {
        let name = self.find_variable(name)?;
        ensure!(
            name.i16uLength as usize == value.bitcnt(),
            PiControlError::InvalidArgument("value or str")
        );
        match value {
            Value::Bit(b) => unsafe {
                self.inner
                    .set_bit(name.i16uAddress, name.i8uBit, b)
            },
            Value::Byte(b) => unsafe { self.inner.set_byte(name.i16uAddress, b) },
            Value::Word(w) => unsafe { self.inner.set_word(name.i16uAddress, w) },
            Value::DWord(d) => unsafe { self.inner.set_dword(name.i16uAddress, d) },
        }
    }

    /// Gets the given value from the processimage. `name` is the name given to the
    /// field that should be written to in PiCtory. The variant of the returned
    /// [`Value`] depends on the length of the field that is read.
    ///
    /// # Errors
    /// If the name can't be found, a [`PiControlError::InvalidArgument`] is
    /// returned.
    ///
    /// # Example
    /// ```no_run
    /// # use revpi::picontrol::{PiControl, Value};
    /// let pi = PiControl::new().unwrap();
    /// let val = pi.get_value("Core_Temperature").unwrap();
    /// assert_eq!(val, Value::Byte(42)); // just an example value
    /// ```
    pub fn get_value(&self, name: &str) -> Result<Value, PiControlError> {
        let name = self.find_variable(name)?;
        match name.i16uLength {
            1 => unsafe { self.inner.get_bit(name.i16uAddress, name.i8uBit) }
                .map(Value::from),
            8 => unsafe { self.inner.get_byte(name.i16uAddress) }.map(Value::from),
            16 => unsafe { self.inner.get_word(name.i16uAddress) }.map(Value::from),
            32 => unsafe { self.inner.get_dword(name.i16uAddress) }.map(Value::from),
            _ => panic!("invalid bitlength from piControl"),
        }
    }
}
