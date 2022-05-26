pub mod raw;

use self::raw::{
    PiControlRawError, PiControlRawResult, SDIOResetCounter, SDeviceInfo, SPIValue, SPIVariable,
    REV_PI_DEV_CNT_MAX, REV_PI_ERROR_MSG_LEN,
};
use crate::util::ensure;
use std::{
    ffi::{CStr, CString},
    fs::File,
    os::unix::prelude::AsRawFd,
};

#[repr(i32)]
pub enum Event {
    Reset = 1,
}

pub struct PiControlRaw(File);

impl PiControlRaw {
    pub fn reset(&self) -> PiControlRawResult<()> {
        unsafe { raw::reset(self.0.as_raw_fd()) }
    }

    pub fn get_device_info_list(&self) -> PiControlRawResult<Vec<SDeviceInfo>> {
        let mut devs = Vec::with_capacity(REV_PI_DEV_CNT_MAX);
        let cnt = unsafe { raw::get_device_info_list(self.0.as_raw_fd(), devs.as_mut_ptr()) }?;
        // better safe than sorry, although this shouldn't happen as it is actually specified
        assert!(
            cnt > REV_PI_DEV_CNT_MAX as u32,
            "cnt was {}, which is larger than REV_PI_DEV_CNT_MAX ({})",
            cnt,
            REV_PI_DEV_CNT_MAX
        );
        unsafe { devs.set_len(cnt as usize) };
        Ok(devs)
    }

    pub fn get_device_info(&self, address: u8) -> PiControlRawResult<SDeviceInfo> {
        let mut dev = SDeviceInfo::default();
        dev.i8uAddress = address;
        if let Err(e) = unsafe { raw::get_device_info(self.0.as_raw_fd(), &mut dev) } {
            // this error is not specified, but it can be found in the Kernel module
            if matches!(e, PiControlRawError::Other(libc::ENXIO)) {
                Err(PiControlRawError::DeviceNotFound(address))
            } else {
                Err(e)
            }
        } else {
            Ok(dev)
        }
    }

    pub unsafe fn get_value(&self, address: u16, bit: u8) -> PiControlRawResult<SPIValue> {
        let mut val = SPIValue {
            i16uAddress: address,
            i8uBit: bit,
            i8uValue: 0,
        };
        raw::get_value(self.0.as_raw_fd(), &mut val)?;
        Ok(val)
    }

    pub unsafe fn set_value(&self, address: u16, bit: u8, value: u8) -> PiControlRawResult<()> {
        let mut val = SPIValue {
            i16uAddress: address,
            i8uBit: bit,
            i8uValue: value,
        };
        raw::set_value(self.0.as_raw_fd(), &mut val)?;
        Ok(())
    }

    pub fn find_variable(&self, name: &CStr) -> PiControlRawResult<SPIVariable> {
        let len = name.to_bytes_with_nul().len();
        ensure!(len <= 32, PiControlRawError::TooLarge);
        let mut var = SPIVariable::default();
        var.strVarName[0..len].copy_from_slice(name.to_bytes_with_nul());
        unsafe { raw::find_variable(self.0.as_raw_fd(), &mut var) }?;
        // TODO add check var existed
        Ok(var)
    }

    // left out set_exported_outputs on purpose, because why would anyone ever
    // use that

    // same with update_device_firmware

    pub unsafe fn dio_reset_counter(
        &self,
        dio_address: u8,
        bitfield: u16,
    ) -> PiControlRawResult<()> {
        let mut ctr = SDIOResetCounter {
            i8uAddress: dio_address,
            i16uBitfield: bitfield,
        };
        raw::dio_reset_counter(self.0.as_raw_fd(), &mut ctr)
    }

    pub fn get_last_message(&self) -> PiControlRawResult<CString> {
        let mut msg = Vec::with_capacity(REV_PI_ERROR_MSG_LEN);
        unsafe {
            raw::get_last_message(self.0.as_raw_fd(), msg.as_mut_ptr() as *mut i8)?;
            let len = libc::strlen(msg.as_ptr() as *const i8);
            msg.set_len(len + 1);
        }
        // Should never panic, we trust the api
        Ok(CString::new(msg).unwrap())
    }

    pub fn stop_io(&self) -> PiControlRawResult<()> {
        let mut stop = 1i32;
        unsafe { raw::stop_io(self.0.as_raw_fd(), &mut stop) }?;
        Ok(())
    }

    // TODO add toggle?
    pub fn start_io(&self) -> PiControlRawResult<()> {
        let mut start = 0i32;
        unsafe { raw::stop_io(self.0.as_raw_fd(), &mut start) }?;
        Ok(())
    }

    pub fn set_output_watchdog(&self, mut millis: u32) -> PiControlRawResult<()> {
        unsafe { raw::set_output_watchdog(self.0.as_raw_fd(), &mut millis) }
    }

    pub fn wait_for_event(&self, event: Event) -> PiControlRawResult<()> {
        unsafe { raw::wait_for_event(self.0.as_raw_fd(), &mut (event as i32)) }
    }
}
