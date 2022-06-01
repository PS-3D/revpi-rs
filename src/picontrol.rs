pub mod raw;

use std::{ffi::CString, io};
use crate::util::ensure;
use self::raw::{raw::SPIVariable, Bit, PiControlRaw};
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
}


