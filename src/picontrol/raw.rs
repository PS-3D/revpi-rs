//! Semi-raw access to the RevPi
//!
//! If you want real raw access, see the [`raw`] module.

pub mod raw;

use self::raw::{
    Event, SDIOResetCounter, SDeviceInfo, SPIValue, SPIVariable, KB_PI_LEN, REV_PI_DEV_CNT_MAX,
    REV_PI_ERROR_MSG_LEN,
};
use super::PiControlError;
use crate::util::ensure;
use std::{
    ffi::{CStr, CString},
    fs::File,
    os::unix::prelude::{AsRawFd, FileExt},
};

/// Bit inside a byte which to write to or read from
#[derive(Debug)]
#[repr(u8)]
pub enum Bit {
    Zero = 0,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl From<u8> for Bit {
    /// Converts u8 to Bit
    ///
    /// # Panics
    /// Will panic if `v > 7`
    fn from(v: u8) -> Self {
        use Bit::*;
        match v {
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            _ => panic!("integer out of range of enum"),
        }
    }
}

/// Provides semi-raw access to the RevPi
///
/// The focus lies on providing error-checking where possible but not at the
/// cost of performance, i.e. every function performs only one operation on the
/// RevPi, not multiple.\
/// Usually there is no need to have multiple instances of this, it might even
/// be counterproductive.
///
/// If you don't have to, don't use this directly but rather a wrapper around it.
#[derive(Debug)]
pub struct PiControlRaw(File);

// drop not needed, file closes automatically when out of scope
impl PiControlRaw {
    /// Constructs a new PiControlRaw object.
    ///
    /// Returns a [`PiControlError::IoError`] if opening "/dev/piControl0" fails.
    pub fn new() -> Result<Self, PiControlError> {
        Ok(PiControlRaw(File::open("/dev/piControl0")?))
    }

    // every error could also be EINVAL if argp or request in ioctl is invalid, but that shouldn't be possible
    // could also be EFAULT if argp is inaccessible or fd is invalid, also left out where not possible

    /// Resets the piControl driver. You have to ensure that either the
    /// configuration won't change or that the changes are taken into account.
    ///
    /// # Panics
    /// Will panic if the bridge restart timed out.
    pub unsafe fn reset(&self) {
        raw::reset(self.0.as_raw_fd())
            .map_err(|e| match e {
                libc::ETIMEDOUT => {
                    panic!("couldn't restart because bridge didn't come up; timedout")
                }
                _ => unreachable!(),
            })
            .unwrap();
    }

    /// Returns a vector with the information of all connected devices.
    ///
    /// # Panics
    /// Will panic if the kernel module ran out of memory.
    pub fn get_device_info_list(&self) -> Vec<SDeviceInfo> {
        let mut devs = Vec::with_capacity(REV_PI_DEV_CNT_MAX);
        let cnt = unsafe { raw::get_device_info_list(self.0.as_raw_fd(), devs.as_mut_ptr()) }
            .map_err(|e| match e {
                libc::ENOMEM => panic!("out of memory"),
                _ => unreachable!(),
            })
            .unwrap();
        // better safe than sorry, although this shouldn't happen as it is actually specified
        assert!(
            cnt > REV_PI_DEV_CNT_MAX as u32,
            "cnt was {}, which is larger than REV_PI_DEV_CNT_MAX ({})",
            cnt,
            REV_PI_DEV_CNT_MAX
        );
        unsafe { devs.set_len(cnt as usize) };
        devs
    }

    /// Returns the information of the requested device.
    ///
    /// If no device with the given address is found, [`PiControlError::DeviceNotFound`]
    /// is returned.
    pub fn get_device_info(&self, address: u8) -> Result<SDeviceInfo, PiControlError> {
        let mut dev = SDeviceInfo::default();
        dev.i8uAddress = address;
        unsafe { raw::get_device_info(self.0.as_raw_fd(), &mut dev) }.map_err(|e| match e {
            libc::ENXIO => PiControlError::DeviceNotFound(address),
            _ => unreachable!(),
        })?;
        Ok(dev)
    }

    // unsafe due to uncertainty of address
    unsafe fn get_value(&self, address: u16, bit: u8) -> Result<u8, PiControlError> {
        ensure!(
            (address as usize) < KB_PI_LEN,
            PiControlError::InvalidArgument("address")
        );
        let mut val = SPIValue {
            i16uAddress: address,
            i8uBit: bit,
            i8uValue: 0,
        };
        raw::get_value(self.0.as_raw_fd(), &mut val)
            .map_err(|e| match e {
                libc::EFAULT => panic!("bridge wasn't running"),
                _ => unreachable!(),
            })
            .unwrap();
        Ok(val.i8uValue)
    }

    /// Gets a bit from the processimage. You have to ensure that `address` and
    /// `bit` are valid, otherwise you might get a wrong value.
    ///
    /// Returns [`PiControlError::InvalidArgument`] if `address` is larger
    /// than [`KB_PI_LEN`].
    ///
    /// # Panics
    /// Will panic if the bridge wasn't running
    pub unsafe fn get_bit(&self, address: u16, bit: Bit) -> Result<bool, PiControlError> {
        self.get_value(address, bit as u8).map(|r| r >= 1)
    }

    /// Gets a byte from the processimage. You have to ensure that `address` is
    /// valid, otherwise you might get a wrong value.
    ///
    /// Returns [`PiControlError::InvalidArgument`] if `address` is larger
    /// than [`KB_PI_LEN`].
    ///
    /// # Panics
    /// Will panic if the bridge wasn't running
    pub unsafe fn get_byte(&self, address: u16) -> Result<u8, PiControlError> {
        self.get_value(address, 8)
    }

    /// Gets a word from the processimage. You have to ensure that `address` is
    /// valid, otherwise you might get a wrong value. Be aware that the value
    /// is returned in the system byteorder, while it is stored as little endian.
    ///
    /// Returns [`PiControlError::IoError`] if there was an error reading
    /// the processimage.
    pub unsafe fn get_word(&self, address: u16) -> Result<u16, PiControlError> {
        let mut bytes = [0u8; 2];
        self.0.read_exact_at(&mut bytes, address as u64)?;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Gets a doubleword from the processimage. You have to ensure that `address`
    /// is valid, otherwise you might get a wrong value. Be aware that the value
    /// is returned in the system byteorder, while it is stored as little endian.
    ///
    /// Returns [`PiControlError::IoError`] if there was an error reading
    /// the processimage.
    pub unsafe fn get_dword(&self, address: u16) -> Result<u32, PiControlError> {
        let mut bytes = [0u8; 4];
        self.0.read_exact_at(&mut bytes, address as u64)?;
        Ok(u32::from_le_bytes(bytes))
    }

    // unsafe due to uncertainty of address
    unsafe fn set_value(&self, address: u16, bit: u8, value: u8) -> Result<(), PiControlError> {
        ensure!(
            (address as usize) < KB_PI_LEN,
            PiControlError::InvalidArgument("address")
        );
        let mut val = SPIValue {
            i16uAddress: address,
            i8uBit: bit,
            i8uValue: value,
        };
        raw::set_value(self.0.as_raw_fd(), &mut val)
            .map_err(|e| match e {
                libc::EFAULT => panic!("bridge wasn't running"),
                _ => unreachable!(),
            })
            .unwrap();
        Ok(())
    }

    /// Writes a bit to the processimage. You have to ensure that `address` and
    /// `bit` are valid, otherwise you might write to the wrong place.
    ///
    /// Returns [`PiControlError::InvalidArgument`] if `address` is larger
    /// than [`KB_PI_LEN`].
    ///
    /// # Panics
    /// Will panic if the bridge wasn't running
    pub unsafe fn set_bit(
        &self,
        address: u16,
        bit: Bit,
        value: bool,
    ) -> Result<(), PiControlError> {
        self.set_value(address, bit as u8, value as u8)
    }

    /// Writes a byte to the processimage. You have to ensure that `address` is
    /// valid, otherwise you might write to the wrong place.
    ///
    /// Returns [`PiControlError::InvalidArgument`] if `address` is larger
    /// than [`KB_PI_LEN`].
    ///
    /// # Panics
    /// Will panic if the bridge wasn't running
    pub unsafe fn set_byte(&self, address: u16, value: u8) -> Result<(), PiControlError> {
        self.set_value(address, 8, value)
    }

    /// Writes a word to the processimage. You have to ensure that `address` is
    /// valid, otherwise you might write to the wrong place. Be aware that the value
    /// is converted to little endian before being written.
    ///
    /// Returns [`PiControlError::IoError`] if there was an error reading
    /// the processimage.
    pub unsafe fn set_word(&self, address: u16, value: u16) -> Result<(), PiControlError> {
        self.0
            .write_all_at(&value.to_le_bytes(), address as u64)
            .map_err(PiControlError::from)
    }

    /// Writes a doubleword to the processimage. You have to ensure that `address`
    /// is valid, otherwise you might write to the wrong place. Be aware that the
    /// value is converted to little endian before being written.
    ///
    /// Returns [`PiControlError::IoError`] if there was an error reading
    /// the processimage.
    pub unsafe fn set_dword(&self, address: u16, value: u32) -> Result<(), PiControlError> {
        self.0
            .write_all_at(&value.to_le_bytes(), address as u64)
            .map_err(PiControlError::from)
    }

    /// Gets the offset, bitoffset and length of a variable by name.
    /// `name` must not be longer than 31 bytes, nullbyte not included.
    ///
    /// Returns [`PiControlError::InvalidArgument`] if `name` is longer than
    /// 31 bytes or if the given name was not found.\
    /// Returns [`PiControlError::NoVarEntries`] if there were not variable
    /// entries at all.
    ///
    /// # Panics
    /// Will panic if the bridge wasn't running
    pub fn find_variable(&self, name: &CStr) -> Result<SPIVariable, PiControlError> {
        let len = name.to_bytes_with_nul().len();
        ensure!(len <= 32, PiControlError::InvalidArgument("length of name"));
        let mut var = SPIVariable::default();
        var.strVarName[0..len].copy_from_slice(name.to_bytes_with_nul());
        unsafe { raw::find_variable(self.0.as_raw_fd(), &mut var) }.map_err(|e| match e {
            libc::EFAULT => {
                // not specified, helpful tho, see kernel module
                if var.i16uAddress == 0xffff && var.i8uBit == 0xff && var.i16uLength == 0xffff {
                    PiControlError::InvalidArgument("name")
                } else {
                    panic!("bridge wasn't running")
                }
            }
            libc::ENOENT => PiControlError::NoVarEntries,
            _ => unreachable!(),
        })?;
        Ok(var)
    }

    // unsafe because only one process should call this
    /// Replaces the whole processimage with the given image. You have to ensure
    /// that there are no other processes that have an open file descriptor on
    /// "/dev/piControl0".
    pub unsafe fn set_exported_outputs(&self, image: &[u8; KB_PI_LEN]) {
        raw::set_exported_outputs(self.0.as_raw_fd(), image.as_ptr()).unwrap();
    }

    // unsafe because device might get bricked
    /// Updates the firmware of a connected device. You have to ensure that there
    /// is exactly one device connected at the time of the update. Also, though
    /// it is not specified, your device might get bricked if you lose power
    /// during the update. `module` is the address of the module that should be
    /// updated. If `module` is `0`, the first device that's found will be updated.
    ///
    /// # Panics
    /// Will panic if the RevPi is not a RevPi Core or RevPi Connect.
    /// Will also panic if the bridge wasn't running or if too many or too little
    /// modules were connected.
    pub unsafe fn update_device_firmware(&self, module: u32) {
        raw::update_device_firmware(self.0.as_raw_fd(), module)
            .map_err(|e| match e {
                libc::EFAULT => {
                    panic!("bridge wasn't running or too little or too many modules were connected")
                }
                libc::EPERM => panic!("this isn't a revpi core or connect"),
                _ => unreachable!(),
            })
            .unwrap();
    }

    /// Resets the counter of the DIO module with `dio_address`. The counters
    /// to reset are specified by a set bit in the corresponding position in
    /// `bitfield`. `bitfield` must not be `0`.
    ///
    /// Returns [`PiControlError::InvalidArgument`] if `bitfield` was `0` or
    /// if `dio_address` was not valid.
    ///
    /// # Panics
    /// Will panic if the RevPi is not a RevPi Core or RevPi Connect, or if the
    /// bridge wasn't running.
    pub fn dio_reset_counter(&self, dio_address: u8, bitfield: u16) -> Result<(), PiControlError> {
        // this is specified in the kernel module
        ensure!(bitfield != 0, PiControlError::InvalidArgument("bitfield"));
        let mut ctr = SDIOResetCounter {
            i8uAddress: dio_address,
            i16uBitfield: bitfield,
        };
        unsafe { raw::dio_reset_counter(self.0.as_raw_fd(), &mut ctr) }.map_err(|e| match e {
            libc::EFAULT => panic!("bridge wasn't running"),
            libc::EPERM => panic!("this isn't a revpi core or connect"),
            libc::EINVAL => PiControlError::InvalidArgument("dio_address"),
            _ => unreachable!(),
        })?;
        Ok(())
    }

    /// Returns the last error message of the RevPi
    pub fn get_last_message(&self) -> CString {
        let mut msg = Vec::with_capacity(REV_PI_ERROR_MSG_LEN);
        unsafe {
            // no error should occur because we are responsible for all arguments
            raw::get_last_message(self.0.as_raw_fd(), msg.as_mut_ptr() as *mut i8).unwrap();
            let len = libc::strlen(msg.as_ptr() as *const libc::c_char);
            msg.set_len(len + 1);
        }
        // Should never panic, we trust the api and checked this before
        CString::new(msg).unwrap()
    }

    fn inner_stop_io(&self, mut stop: i32) {
        unsafe { raw::stop_io(self.0.as_raw_fd(), &mut stop) }
            .map_err(|e| match e {
                libc::EFAULT => panic!("bridge wasn't running"),
                _ => unreachable!(),
            })
            .unwrap();
    }

    /// Stops all I/O communication. piControl will write `0` to all outputs and
    /// inputs won't be updated.
    pub fn stop_io(&self) {
        self.inner_stop_io(1);
    }

    /// Stops I/O communication
    pub fn start_io(&self) {
        self.inner_stop_io(0);
    }

    /// Toggles I/O communication
    pub fn toggle_io(&self) {
        self.inner_stop_io(2);
    }

    /// Activates a watchdog. `millis` is the watchdog period in milliseconds.
    /// To stop the watchdog, set `millis` to zero or drop this object.
    ///
    /// For more information see `man picontrol_ioctl`
    pub fn set_output_watchdog(&self, mut millis: u32) {
        unsafe { raw::set_output_watchdog(self.0.as_raw_fd(), &mut millis) }.unwrap();
    }

    /// Blocks until an event occurs in the piControl driver.
    ///
    /// Returns the event.
    pub fn wait_for_event(&self) -> Event {
        let mut event = 0i32;
        unsafe { raw::wait_for_event(self.0.as_raw_fd(), &mut event) }.unwrap();
        // TODO from primitive
        match event {
            1 => Event::Reset,
            _ => panic!("an unspecified event occured"),
        }
    }
}
