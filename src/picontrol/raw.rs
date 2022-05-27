pub mod raw;

use self::raw::{
    SDIOResetCounter, SDeviceInfo, SPIValue, SPIVariable, REV_PI_DEV_CNT_MAX, REV_PI_ERROR_MSG_LEN,
};
use crate::{picontrol::raw::raw::KB_PI_LEN, util::ensure};
use std::{
    ffi::{CStr, CString},
    fs::File,
    io::Error as IoError,
    os::unix::prelude::AsRawFd,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PiControlRawError {
    #[error("{0} was invalid")]
    InvalidArgument(&'static str),
    #[error("Device with address {0} not found")]
    DeviceNotFound(u8),
    #[error("No variable entries")]
    NoVarEntries,
}

#[derive(Debug)]
#[repr(i32)]
pub enum Event {
    Reset = 1,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ValType {
    Zero = 0,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Byte,
}

#[derive(Debug)]
pub struct PiControlRaw(File);

// drop not needed, file closes automatically when out of scope
impl PiControlRaw {
    pub fn new() -> Result<Self, IoError> {
        Ok(PiControlRaw(File::open("/dev/piControl0")?))
    }

    // every error could also be EINVAL if argp or request in ioctl is invalid, but that shouldn't be possible
    // could also be EFAULT if argp is inaccessible or fd is invalid, also left out where not possible

    pub fn reset(&self) {
        unsafe { raw::reset(self.0.as_raw_fd()) }
            .map_err(|e| match e {
                libc::ETIMEDOUT => {
                    panic!("couldn't restart because bridge didn't come up; timedout")
                }
                _ => unreachable!(),
            })
            .unwrap();
    }

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

    pub fn get_device_info(&self, address: u8) -> Result<SDeviceInfo, PiControlRawError> {
        let mut dev = SDeviceInfo::default();
        dev.i8uAddress = address;
        unsafe { raw::get_device_info(self.0.as_raw_fd(), &mut dev) }.map_err(|e| match e {
            libc::ENXIO => PiControlRawError::DeviceNotFound(address),
            _ => unreachable!(),
        })?;
        Ok(dev)
    }

    // unsafe due to uncertainty of address
    pub unsafe fn get_value(
        &self,
        address: u16,
        bit: ValType,
    ) -> Result<SPIValue, PiControlRawError> {
        ensure!(
            (address as usize) < KB_PI_LEN,
            PiControlRawError::InvalidArgument("address")
        );
        let mut val = SPIValue {
            i16uAddress: address,
            i8uBit: bit as u8,
            i8uValue: 0,
        };
        raw::get_value(self.0.as_raw_fd(), &mut val).map_err(|e| match e {
            libc::EFAULT => panic!("bridge wasn't running"),
            _ => unreachable!(),
        })?;
        Ok(val)
    }

    // unsafe due to uncertainty of address
    pub unsafe fn set_value(
        &self,
        address: u16,
        bit: ValType,
        value: u8,
    ) -> Result<(), PiControlRawError> {
        ensure!(
            (address as usize) < KB_PI_LEN,
            PiControlRawError::InvalidArgument("address")
        );
        let mut val = SPIValue {
            i16uAddress: address,
            i8uBit: bit as u8,
            i8uValue: value,
        };
        raw::set_value(self.0.as_raw_fd(), &mut val).map_err(|e| match e {
            libc::EFAULT => panic!("bridge wasn't running"),
            _ => unreachable!(),
        })?;
        Ok(())
    }

    pub fn find_variable(&self, name: &CStr) -> Result<SPIVariable, PiControlRawError> {
        let len = name.to_bytes_with_nul().len();
        ensure!(len <= 32, PiControlRawError::InvalidArgument("length of name"));
        let mut var = SPIVariable::default();
        var.strVarName[0..len].copy_from_slice(name.to_bytes_with_nul());
        unsafe { raw::find_variable(self.0.as_raw_fd(), &mut var) }.map_err(|e| match e {
            libc::EFAULT => {
                // not specified, helpful tho, see kernel module
                if var.i16uAddress == 0xffff && var.i8uBit == 0xff && var.i16uLength == 0xffff {
                    PiControlRawError::InvalidArgument("name")
                } else {
                    panic!("bridge wasn't running")
                }
            }
            libc::ENOENT => PiControlRawError::NoVarEntries,
            _ => unreachable!(),
        })?;
        Ok(var)
    }

    // left out set_exported_outputs on purpose, because why would anyone ever
    // use that

    // same with update_device_firmware

    pub fn dio_reset_counter(
        &self,
        dio_address: u8,
        bitfield: u16,
    ) -> Result<(), PiControlRawError> {
        // this is specified in the kernel module
        ensure!(bitfield != 0, PiControlRawError::InvalidArgument("bitfield"));
        let mut ctr = SDIOResetCounter {
            i8uAddress: dio_address,
            i16uBitfield: bitfield,
        };
        unsafe { raw::dio_reset_counter(self.0.as_raw_fd(), &mut ctr) }.map_err(|e| match e {
            libc::EFAULT => panic!("bridge wasn't running"),
            libc::EPERM => panic!("this isn't a revpi core or connect"),
            libc::EINVAL => PiControlRawError::InvalidArgument("dio_address"),
            _ => unreachable!(),
        })?;
        Ok(())
    }

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

    pub fn stop_io(&self) {
        self.inner_stop_io(1);
    }

    pub fn start_io(&self) {
        self.inner_stop_io(0);
    }

    pub fn toggle_io(&self) {
        self.inner_stop_io(2);
    }

    pub fn set_output_watchdog(&self, mut millis: u32) {
        unsafe { raw::set_output_watchdog(self.0.as_raw_fd(), &mut millis) }.unwrap();
    }

    pub fn wait_for_event(&self, event: Event) {
        unsafe { raw::wait_for_event(self.0.as_raw_fd(), &mut (event as i32)) }.unwrap();
    }
}
