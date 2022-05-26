use crate::util::ensure;
use libc;
use std::{
    ffi::{CStr, CString},
    os::unix::prelude::AsRawFd,
    ptr::null_mut,
};
use thiserror::Error;

// TODO possibly do without libc?

const REV_PI_DEV_FIRST_RIGHT: usize = 32;
const REV_PI_DEV_CNT_MAX: usize = 64;
const REV_PI_ERROR_MSG_LEN: usize = 256;

#[allow(non_snake_case)]
#[derive(Debug, Default)]
#[repr(C)]
pub struct SDeviceInfo {
    i8uAddress: u8,
    i32uSerialNumber: u32,
    i16uModuleType: u16,
    i16uHW_Revision: u16,
    i16uSW_Major: u16,
    i16uSW_Minor: u16,
    i32uSVN_Revision: u32,
    i16uInputLength: u16,
    i16uOutputLength: u16,
    i16uConfigLength: u16,
    i16uBaseOffset: u16,
    i16uInputOffset: u16,
    i16uOutputOffset: u16,
    i16uConfigOffset: u16,
    i16uFirstEntry: u16,
    i8uModuleState: u8,
    i8uActive: u8,
    i8uReserve: [u8; 30],
}

// #[derive(Debug)]
// #[repr(u8)]
// pub enum EntryInfoType {
//     Input = 1,
//     Output,
//     Memory,
//     Config,
// }

// #[allow(non_snake_case)]
// #[derive(Debug)]
// #[repr(C)]
// pub struct SEntryInfo {
//     i8uAddress: u8,
//     i8uType: EntryInfoType,
//     i16uIndex: u16,
//     i16uBitLength: u16,
//     i8uBitPos: u8,
//     i16uOffset: u16,
//     i32uDefault: u32,
//     strVarName: [u8; 32],
// }

#[allow(non_snake_case)]
#[derive(Debug, Default)]
#[repr(C)]
pub struct SPIValue {
    i16uAddress: u16,
    i8uBit: u8,
    i8uValue: u8,
}

#[allow(non_snake_case)]
#[derive(Debug, Default)]
#[repr(C)]
pub struct SPIVariable {
    strVarName: [u8; 32],
    i16uAddress: u16,
    i8uBit: u8,
    i16uLength: u16,
}

#[allow(non_snake_case)]
#[derive(Debug, Default)]
#[repr(C)]
pub struct SDIOResetCounter {
    i8uAddress: u8,
    i16uBitfield: u16,
}

// #[allow(non_snake_case)]
// #[derive(Debug, Default)]
// #[repr(C)]
// pub struct SConfigData {
//     bLeft: u8,
//     i16uLen: u16,
//     acData: [u8; 256]
// }

// from piControl.h
#[repr(u64)]
enum KBRequests {
    // reset the piControl driver including the config file
    Reset = 0x4b0c,
    // get the device info of all detected devices
    GetDeviceInfoList,
    // get the device info of one device
    GetDeviceInfo,
    // get the value of one bit in the process image
    GetValue,
    // set the value of one bit in the process image
    SetValue,
    // find a varible defined in piCtory
    FindVariable,
    // copy the exported outputs from a application process image to the real process image
    SetExportedOutputs,
    // try to update the firmware of connected devices
    UpdateDeviceFirmware,
    // set a counter or endocder to 0
    DIOResetCounter,
    // copy the last error message
    GetLastMessage,
    // stop/start IO communication, can be used for I/O simulation
    StopIO,
    // for download of configuration to Master Gateway: stop IO communication completely
    //ConfigStop,
    // for download of configuration to Master Gateway: download config data
    //ConfigSend,
    // for download of configuration to Master Gateway: restart IO communication
    //ConfigStart,
    // activate a watchdog for this handle. If write is not called for a given period all outputs are set to 0
    SetOutputWatchdog,
    // set the f_pos, the unsigned int * is used to interpret the pos value
    //SetPos,
    //AIOCalibrate,
    // wait for an event. This call is normally blocking
    WaitForEvent = 0x4b32,
}

#[repr(i32)]
pub enum Event {
    Reset = 1,
}

#[derive(Debug, Error)]
pub enum PiControlRawError {
    #[error("either request or argp were invalid")]
    InvalidArgument,
    #[error("request does not apply to object type fd refers to")]
    WrongObjectType,
    #[error("Device with address {0} not found")]
    DeviceNotFound(u8),
    #[error("Argument was too large")]
    TooLarge,
    #[error("was other, non-specified error: {0}")]
    Other(i32),
}

unsafe fn ioctl<F: AsRawFd, T: ?Sized>(
    fd: F,
    request: KBRequests,
    argp: *mut T,
) -> Result<u32, PiControlRawError> {
    let res = libc::ioctl(fd.as_raw_fd(), request as libc::c_ulong, argp);
    if res <= -1 {
        match *libc::__errno_location() {
            libc::EBADF => panic!("{} was not a valid file descriptor", fd.as_raw_fd()),
            libc::EFAULT => panic!("argp pointed to an inaccessible memory area"),
            libc::EINVAL => Err(PiControlRawError::InvalidArgument),
            libc::ENOTTY => Err(PiControlRawError::WrongObjectType),
            _ => Err(PiControlRawError::Other(*libc::__errno_location())),
        }
    } else {
        Ok(res as u32)
    }
}

pub fn reset<F: AsRawFd>(fd: F) -> Result<(), PiControlRawError> {
    unsafe { ioctl::<F, u8>(fd, KBRequests::Reset, null_mut()) }.map(|_| ())
}

pub fn get_device_info_list<F: AsRawFd>(fd: F) -> Result<Vec<SDeviceInfo>, PiControlRawError> {
    let mut devs = Vec::with_capacity(REV_PI_DEV_CNT_MAX);
    let cnt = unsafe { ioctl(fd, KBRequests::GetDeviceInfoList, devs.as_mut_ptr()) }?;
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

// TODO by module type of this? see manuam
pub fn get_device_info<F: AsRawFd>(fd: F, address: u8) -> Result<SDeviceInfo, PiControlRawError> {
    let mut dev = SDeviceInfo::default();
    dev.i8uAddress = address;
    if let Err(e) = unsafe { ioctl(fd, KBRequests::GetDeviceInfo, &mut dev) } {
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

// In theory this could be safe since the piControl module checks whether the
// index is inside the bounds, but nevertheless, we could read at any random
// point, interpreting the value in a certain way, which also makes this sorta
// unsafe
pub unsafe fn get_value<F: AsRawFd>(
    fd: F,
    address: u16,
    bit: u8,
) -> Result<SPIValue, PiControlRawError> {
    let mut val = SPIValue {
        i16uAddress: address,
        i8uBit: bit,
        i8uValue: 0,
    };
    ioctl(fd, KBRequests::GetValue, &mut val)?;
    Ok(val)
}

pub unsafe fn set_value<F: AsRawFd>(
    fd: F,
    address: u16,
    bit: u8,
    value: u8,
) -> Result<(), PiControlRawError> {
    let mut val = SPIValue {
        i16uAddress: address,
        i8uBit: bit,
        i8uValue: value,
    };
    ioctl(fd, KBRequests::SetValue, &mut val)?;
    Ok(())
}

pub fn find_variable<F: AsRawFd>(fd: F, name: &CStr) -> Result<SPIVariable, PiControlRawError> {
    let len = name.to_bytes_with_nul().len();
    ensure!(len <= 32, PiControlRawError::TooLarge);
    let mut var = SPIVariable::default();
    var.strVarName[0..len].copy_from_slice(name.to_bytes_with_nul());
    unsafe { ioctl(fd, KBRequests::FindVariable, &mut var) }?;
    // TODO add check var existed
    Ok(var)
}

// image.len() must be the same as processimage length
pub unsafe fn set_exported_outputs<F: AsRawFd>(
    fd: F,
    image: &mut [u8],
) -> Result<(), PiControlRawError> {
    ioctl(fd, KBRequests::SetExportedOutputs, image).map(|_| ())
}

pub fn update_device_firmware<F: AsRawFd>(fd: F, module: u32) -> Result<(), PiControlRawError> {
    todo!();
    //unsafe { ioctl(fd, KBRequests::UpdateDeviceFirmware, module) }.map(|_| ())
}

// dio_address must be valid
pub unsafe fn dio_reset_counter<F: AsRawFd>(
    fd: F,
    dio_address: u8,
    bitfield: u16,
) -> Result<(), PiControlRawError> {
    let mut ctr = SDIOResetCounter {
        i8uAddress: dio_address,
        i16uBitfield: bitfield,
    };
    ioctl(fd, KBRequests::DIOResetCounter, &mut ctr).map(|_| ())
}

pub fn get_last_message<F: AsRawFd>(fd: F) -> Result<CString, PiControlRawError> {
    let mut msg = Vec::with_capacity(REV_PI_ERROR_MSG_LEN);
    unsafe {
        ioctl(fd, KBRequests::GetLastMessage, &mut msg)?;
        let len = libc::strlen(msg.as_ptr() as *const i8);
        msg.set_len(len + 1);
    }
    // Should never panic, we trust the api
    Ok(CString::new(msg).unwrap())
}

pub fn stop_io<F: AsRawFd>(fd: F) -> Result<(), PiControlRawError> {
    let mut stop = 1i32;
    unsafe { ioctl(fd, KBRequests::StopIO, &mut stop) }?;
    Ok(())
}

// TODO add toggle?
pub fn start_io<F: AsRawFd>(fd: F) -> Result<(), PiControlRawError> {
    let mut start = 0i32;
    unsafe { ioctl(fd, KBRequests::StopIO, &mut start) }?;
    Ok(())
}

pub fn set_output_watchdog<F: AsRawFd>(fd: F, mut millis: u32) -> Result<(), PiControlRawError> {
    unsafe { ioctl(fd, KBRequests::SetOutputWatchdog, &mut millis) }.map(|_| ())
}

pub fn wait_for_event<F: AsRawFd>(fd: F, mut event: Event) -> Result<(), PiControlRawError> {
    unsafe { ioctl(fd, KBRequests::WaitForEvent, &mut event) }.map(|_| ())
}
