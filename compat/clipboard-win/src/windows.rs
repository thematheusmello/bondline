//! Win32 ownership, global memory and standard clipboard formats.

use std::{
    ffi::OsString, io, marker::PhantomData, num::NonZeroU32, os::windows::ffi::OsStringExt,
    path::PathBuf, ptr,
};
use windows_sys::Win32::{
    Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND},
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
            OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
        },
        Memory::{GHND, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
    },
    UI::{
        Shell::DragQueryFileW,
        WindowsAndMessaging::{CreateWindowExW, DestroyWindow, HWND_MESSAGE},
    },
};

const MAX_BYTES: usize = 32 * 1024 * 1024;

/// Standard Win32 format identifiers used by the pinned Arboard backend.
pub mod formats {
    pub const CF_UNICODETEXT: u32 = 13;
    pub const CF_DIBV5: u32 = 17;
}

/// Thread-bound clipboard transaction with a real owner window.
pub struct Clipboard {
    owner: HWND,
    thread: PhantomData<*mut ()>,
}

impl Clipboard {
    /// Open the clipboard. Contention is reported to Arboard's retry loop.
    pub fn new() -> io::Result<Self> {
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        // SAFETY: STATIC is an OS-defined class; all pointers live through this call.
        let owner = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        };
        if owner.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: owner is a live message-only window on this thread.
        if unsafe { OpenClipboard(owner) } == 0 {
            let error = io::Error::last_os_error();
            // SAFETY: this call destroys only the window just allocated above.
            unsafe {
                DestroyWindow(owner);
            }
            return Err(error);
        }
        Ok(Self {
            owner,
            thread: PhantomData,
        })
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        // SAFETY: this non-Send guard owns the open clipboard and owner window on this thread.
        unsafe {
            CloseClipboard();
            DestroyWindow(self.owner);
        }
    }
}

/// Whether Windows can supply a format; callers must still hold a clipboard transaction.
pub fn is_format_avail(format: u32) -> bool {
    // SAFETY: this query has no pointer parameters.
    unsafe { IsClipboardFormatAvailable(format) != 0 }
}

/// Register a named format without embedded NUL or overlong UTF-16 names.
pub fn register_format(name: &str) -> Option<NonZeroU32> {
    let name: Vec<u16> = name.encode_utf16().collect();
    if name.is_empty() || name.len() > 255 || name.contains(&0) {
        return None;
    }
    let name: Vec<u16> = name.into_iter().chain(Some(0)).collect();
    // SAFETY: the format name is a validated NUL-terminated UTF-16 string.
    NonZeroU32::new(unsafe { RegisterClipboardFormatW(name.as_ptr()) })
}

/// Clear data while the caller owns an open clipboard transaction.
pub fn empty() -> io::Result<()> {
    // SAFETY: Windows enforces transaction ownership for this operation.
    if unsafe { EmptyClipboard() } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

struct Locked(HGLOBAL);
impl Drop for Locked {
    fn drop(&mut self) {
        // SAFETY: this guard is created only after a successful GlobalLock.
        unsafe {
            GlobalUnlock(self.0);
        }
    }
}
struct Owned(HGLOBAL);
impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: this guard owns its allocation until ownership is transferred to Windows.
        unsafe {
            GlobalFree(self.0);
        }
    }
}

fn read(format: u32) -> io::Result<Vec<u8>> {
    // SAFETY: Windows validates the open transaction and format identifier.
    let memory = unsafe { GetClipboardData(format) } as HGLOBAL;
    if memory.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: size is queried before any memory is read.
    let length = unsafe { GlobalSize(memory) };
    if length == 0 || length > MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "clipboard allocation size exceeds limits",
        ));
    }
    // SAFETY: memory belongs to an open clipboard transaction; GlobalLock pins its data.
    let data = unsafe { GlobalLock(memory) };
    if data.is_null() {
        return Err(io::Error::last_os_error());
    }
    let _lock = Locked(memory);
    // SAFETY: GlobalSize bounded the live locked allocation; the copy completes before unlock.
    Ok(unsafe { std::slice::from_raw_parts(data.cast::<u8>(), length) }.to_vec())
}

fn write(format: u32, data: &[u8], clear: bool) -> io::Result<()> {
    if data.is_empty() || data.len() > MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "clipboard data size exceeds limits",
        ));
    }
    // SAFETY: allocation size is bounded and GHND provides movable, zero-initialized memory.
    let memory = unsafe { GlobalAlloc(GHND, data.len()) };
    if memory.is_null() {
        return Err(io::Error::last_os_error());
    }
    let owned = Owned(memory);
    // SAFETY: memory is the newly owned valid global allocation.
    let destination = unsafe { GlobalLock(memory) };
    if destination.is_null() {
        return Err(io::Error::last_os_error());
    }
    {
        let _lock = Locked(memory);
        // SAFETY: destination has exactly data.len() writable bytes and does not alias input.
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), destination.cast(), data.len());
        }
    }
    if clear {
        empty()?;
    }
    // SAFETY: the caller holds a clipboard transaction; the unlocked movable block is transferred.
    if unsafe { SetClipboardData(format, memory as HANDLE) }.is_null() {
        return Err(io::Error::last_os_error());
    }
    std::mem::forget(owned);
    Ok(())
}

/// Raw operations used by Arboard while its Clipboard guard is alive.
pub mod raw {
    pub use super::empty;
    use super::*;

    pub fn get_vec(format: u32, output: &mut Vec<u8>) -> io::Result<usize> {
        let bytes = read(format)?;
        let count = bytes.len();
        output.extend(bytes);
        Ok(count)
    }
    pub fn get_string(output: &mut Vec<u8>) -> io::Result<usize> {
        let text = crate::decode_text(&read(formats::CF_UNICODETEXT)?)?;
        let count = text.len();
        output.extend_from_slice(text.as_bytes());
        Ok(count)
    }
    pub fn get_html(format: u32, output: &mut Vec<u8>) -> io::Result<usize> {
        let bytes = read(format)?;
        let range = crate::html_range(&bytes)?;
        let fragment = bytes
            .get(range)
            .ok_or_else(|| io::Error::other("invalid HTML range"))?;
        output.extend_from_slice(fragment);
        Ok(fragment.len())
    }
    pub fn set_string(text: &str) -> io::Result<()> {
        if text.contains('\0') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "clipboard text contains NUL",
            ));
        }
        let bytes: Vec<u8> = text
            .encode_utf16()
            .chain(Some(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        write(formats::CF_UNICODETEXT, &bytes, true)
    }
    pub fn set_without_clear(format: u32, bytes: &[u8]) -> io::Result<()> {
        write(format, bytes, false)
    }
    pub fn get_file_list_path(output: &mut Vec<PathBuf>) -> io::Result<usize> {
        // SAFETY: Windows validates the CF_HDROP handle while the clipboard is open.
        let handle = unsafe { GetClipboardData(15) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: querying count does not write through a buffer.
        let count = unsafe { DragQueryFileW(handle, u32::MAX, ptr::null_mut(), 0) };
        if count > 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "too many clipboard paths",
            ));
        }
        let mut total = 0_usize;
        for index in 0..count {
            // SAFETY: index is within the count returned by Windows.
            let length = unsafe { DragQueryFileW(handle, index, ptr::null_mut(), 0) };
            total = total.saturating_add(length as usize);
            if length > 32768 || total > MAX_BYTES / 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "clipboard paths exceed limits",
                ));
            }
            let mut name = vec![0_u16; length as usize + 1];
            // SAFETY: the buffer holds the queried length plus terminator.
            let copied = unsafe { DragQueryFileW(handle, index, name.as_mut_ptr(), length + 1) };
            if copied != length {
                return Err(io::Error::other("clipboard path changed during query"));
            }
            output.push(PathBuf::from(OsString::from_wide(
                name.get(..length as usize)
                    .ok_or_else(|| io::Error::other("invalid clipboard path range"))?,
            )));
        }
        Ok(count as usize)
    }
}
