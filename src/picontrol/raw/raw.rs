//! Raw bindings and struct definitions for piControl

use libc;
use std::os::unix::prelude::RawFd;

// TODO possibly do without libc?

pub const REV_PI_DEV_FIRST_RIGHT: usize = 32;
/// Maxmium device count
pub const REV_PI_DEV_CNT_MAX: usize = 64;
/// Maximium length of an error message
pub const REV_PI_ERROR_MSG_LEN: usize = 256;
/// Length of the processimage
pub const KB_PI_LEN: usize = 4096;
/// Location of the running config
pub const PICONFIG_FILE: &str = "/etc/revpi/config.rsc";
/// Location of the running config on wheezy
pub const PICONFIG_FILE_WHEEZY: &str = "/opt/KUNBUS/config.rsc";

/// Rust binding for the `SDeviceInfo` struct defined in [`piControl.h`](https://github.com/RevolutionPi/piControl/blob/master/piControl.h#L124)
#[allow(non_snake_case)]
#[derive(Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct SDeviceInfo {
    pub i8uAddress: u8,
    pub i32uSerialNumber: u32,
    pub i16uModuleType: u16,
    pub i16uHW_Revision: u16,
    pub i16uSW_Major: u16,
    pub i16uSW_Minor: u16,
    pub i32uSVN_Revision: u32,
    pub i16uInputLength: u16,
    pub i16uOutputLength: u16,
    pub i16uConfigLength: u16,
    pub i16uBaseOffset: u16,
    pub i16uInputOffset: u16,
    pub i16uOutputOffset: u16,
    pub i16uConfigOffset: u16,
    pub i16uFirstEntry: u16,
    pub i8uModuleState: u8,
    pub i8uActive: u8,
    pub i8uReserve: [u8; 30],
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

// TODO Bindings for module types

/// Rust binding for the `SPIValue` struct defined in [`piControl.h`](https://github.com/RevolutionPi/piControl/blob/master/piControl.h#L163)
#[allow(non_snake_case)]
#[derive(Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct SPIValue {
    pub i16uAddress: u16,
    pub i8uBit: u8,
    pub i8uValue: u8,
}

/// Rust binding for the `SPIVariable` struct defined in [`piControl.h`](https://github.com/RevolutionPi/piControl/blob/master/piControl.h#L170)
#[allow(non_snake_case)]
#[derive(Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct SPIVariable {
    pub strVarName: [u8; 32],
    pub i16uAddress: u16,
    pub i8uBit: u8,
    pub i16uLength: u16,
}

/// Rust binding for the `SDIOResetCounter` struct defined in [`piControl.h`](https://github.com/RevolutionPi/piControl/blob/master/piControl.h#L178)
#[allow(non_snake_case)]
#[derive(Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct SDIOResetCounter {
    pub i8uAddress: u8,
    pub i16uBitfield: u16,
}

/// Rust bindings for the Events defined in [`piControl.h`](https://github.com/RevolutionPi/piControl/blob/master/piControl.h#L116)
///
/// Currently only Reset is supported
#[derive(Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Event {
    /// Occurs if the driver gets reset
    Reset = 1,
}

// #[allow(non_snake_case)]
// #[derive(Debug, Default)]
// #[repr(C)]
// pub struct SConfigData {
//     bLeft: u8,
//     i16uLen: u16,
//     acData: [u8; 256]
// }

/// Rust bindings for the ioctls defined in [`piControl.h`](https://github.com/RevolutionPi/piControl/blob/master/piControl.h#L94)
#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum KBRequests {
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

unsafe fn ioctl<T>(fd: RawFd, request: KBRequests, argp: T) -> Result<u32, i32> {
    let res = libc::ioctl(fd, request as libc::c_ulong, argp);
    if res <= -1 {
        Err(*libc::__errno_location())
    } else {
        Ok(res as u32)
    }
}

/// Resets the the RevPi I/O module comms and config
///
/// # Errors
/// If the bridge takes too long to come up, `libc::ETIMEDOUT` is returened.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn reset(fd: RawFd) -> Result<u32, i32> {
    ioctl(fd, KBRequests::Reset, 0u64)
}

/// Gets the device information of all connected modules
///
/// `devs` must be a pointer to an array of at least [`REV_PI_DEV_CNT_MAX`] [`SDeviceInfo`] entries.
///
/// # Errors
/// If successful, the number of devices written will be returned.\
/// If the kernel module ran out of memory, `libc::ENOMEM` is returned.
/// If `devs` wasn't accessible libc::EFAULT` is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn get_device_info_list(fd: RawFd, devs: *mut SDeviceInfo) -> Result<u32, i32> {
    ioctl(fd, KBRequests::GetDeviceInfoList, devs)
}

// TODO by module type of this? see manual
/// Gets the device information of specified device
///
/// `dev` must point to a [`SDeviceInfo`] struct with `i8uAddress` set to the
/// desired device address.
///
/// # Errors
/// If the device wasn't found, libc::ENXIO` is returned.\
/// If `dev` wasn't accessible `libc::EFAULT` is returned.\
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.\
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// libc::ENOTTY` is returened.\
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn get_device_info(fd: RawFd, dev: *mut SDeviceInfo) -> Result<u32, i32> {
    ioctl(fd, KBRequests::GetDeviceInfo, dev)
}

/// Gets a value from the processimage
///
/// `val` must point to a [`SPIValue`] struct with `i16uAddress` and `i8uBit` set
/// to the desired value. If `0 <= i8uBit <= 7` then a single bit is read, `i8uValue`
/// will be either `1` or `0`. If `i8uBit >= 8` then a whole byte will be read.
///
/// # Errors
/// If the address was larger than [`KB_PI_LEN`], the bridge wasn't running or
/// `val` wasn't accessible libc::EFAULT` is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn get_value(fd: RawFd, val: *mut SPIValue) -> Result<u32, i32> {
    ioctl(fd, KBRequests::GetValue, val)
}

/// Sets a value in the processimage
///
/// `val` must point to a [`SPIValue`] struct with its members initialized properly.
///
/// # Errors
/// If the address was larger than [`KB_PI_LEN`], the bridge wasn't running or
/// `val` wasn't accessible `libc::EFAULT` is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see [`get_value`], `man ioctl`, `man picontrol_ioctl`
/// or the kernel module
pub unsafe fn set_value(fd: RawFd, val: *mut SPIValue) -> Result<u32, i32> {
    ioctl(fd, KBRequests::SetValue, val)
}

/// Finds a variables address and length by name
///
/// `var` must point to a [`SPIVariable`] struct with `strVarName` set to a
/// null-terminated string with the name of the desired variable set in Pictory.
///
/// # Errors
/// If the variable wasn't found, the bridge wasn't running or var wasn't accessible
/// `libc::EFAULT` is returned.
/// If there were no variable entries, `libc::ENOENT` is returned.
/// If fd is not a valid file descriptor, libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn find_variable(fd: RawFd, var: *mut SPIVariable) -> Result<u32, i32> {
    ioctl(fd, KBRequests::FindVariable, var)
}

/// Replaces the whole processimage
///
/// `image` must point to the new processimage. It needs to be [`KB_PI_LEN`] bytes
/// long. Currently only one process should call this ioctl.
///
/// # Errors
/// If `image` wasn't accessible libc::EFAULT` is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn set_exported_outputs(fd: RawFd, image: *const u8) -> Result<u32, i32> {
    ioctl(fd, KBRequests::SetExportedOutputs, image)
}

/// Updates the firmware of the given module
///
/// `module` must be a valid address of a module. This can only be done while
/// exactly one module is connected.
///
/// # Errors
/// If the RevPi isnt a Core or Connect, `libc::EPERM` is returned.
/// If the bridge isn't running or too many or too little modules are connected,
/// `libc::EFAULT` is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn update_device_firmware(fd: RawFd, module: u32) -> Result<u32, i32> {
    ioctl(fd, KBRequests::UpdateDeviceFirmware, module)
}

/// Resets the given counters in a DIO module
///
/// `ctr` must point to a [`SDIOResetCounter`] struct with `i8uAddress` set to
/// the address of the desired module. For each counter to be reset, the corresponding
/// bit in `i16uBitfield` must be set. `i16uBitfield` must not be `0`.
///
/// # Errors
/// If the RevPi isnt a Core or Connect, `libc::EPERM` is returned.
/// If the module wasn't found or if the bitfield was `0`, `libc::EINVAL`
/// is returned.
/// If the bridge wasn't running or ctr wasn't accessible `libc::EFAULT`
/// is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn dio_reset_counter(fd: RawFd, ctr: *mut SDIOResetCounter) -> Result<u32, i32> {
    ioctl(fd, KBRequests::DIOResetCounter, ctr)
}

/// Copies the last error message
///
/// `msg` must point to a string with a length of at least [`REV_PI_ERROR_MSG_LEN`]
/// bytes. The message will be written into it.
///
/// # Errors
/// If `msg` wasn't accessible `libc::EFAULT` is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn get_last_message(fd: RawFd, msg: *mut i8) -> Result<u32, i32> {
    ioctl(fd, KBRequests::GetLastMessage, msg)
}

/// Stop, start or toggle I/O communication
///
/// `stop` must point to `0` to start, `1` to stop or `2` to toggle I/O communication.
///
/// # Errors
/// If the call is successfull, the new mode will be returned.\
/// If the bridge wasn't running or stop wasn't accessible `libc::EFAULT`
/// is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn stop_io(fd: RawFd, stop: *mut i32) -> Result<u32, i32> {
    ioctl(fd, KBRequests::StopIO, stop)
}

/// Activate an application watchdog
///
/// `millis` must point to the watchdog period in milliseconds.
///
/// # Errors
/// If `millis` wasn't accessible `libc::EFAULT` is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn set_output_watchdog(fd: RawFd, millis: *mut u32) -> Result<u32, i32> {
    ioctl(fd, KBRequests::SetOutputWatchdog, millis)
}

/// Wait for an event from piControl
///
/// Writes the event that happened into `event`. Currently only a reset
/// of the driver is supported.\
/// This is a blocking call.
///
/// # Errors
/// If `event` wasn't accessible `libc::EFAULT` is returned.
/// If fd is not a valid file descriptor, `libc::EBADF` is returened.
/// If fd is not a character special device or doesn't refer to `"/dev/piControl0"`,
/// `libc::ENOTTY` is returened.
///
/// # Further Informentation
/// For more information see `man ioctl`, `man picontrol_ioctl` or the kernel module
pub unsafe fn wait_for_event(fd: RawFd, event: *mut i32) -> Result<u32, i32> {
    ioctl(fd, KBRequests::WaitForEvent, event)
}
