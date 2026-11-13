//! Purpose-limited compatibility over dlopen2; no ISC-licensed implementation is copied.

use std::{
    ffi::{CStr, CString, OsStr},
    fmt,
    marker::PhantomData,
    ops::Deref,
};

/// Dynamic loading failure with the original backend diagnostic.
#[derive(Debug)]
pub enum Error {
    /// Symbol lookup failed; retained for the EGL loader's version negotiation.
    DlSym { desc: String },
    /// A lookup failed without an OS diagnostic.
    DlSymUnknown,
    /// Invalid parameters or failure to open a library.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DlSym { desc } | Self::Other(desc) => f.write_str(desc),
            Self::DlSymUnknown => f.write_str("symbol lookup failed"),
        }
    }
}
impl std::error::Error for Error {}

impl From<dlopen2::Error> for Error {
    fn from(error: dlopen2::Error) -> Self {
        Self::Other(error.to_string())
    }
}

/// RAII ownership of the OS library through dlopen2.
#[derive(Debug)]
pub struct Library(Backend);

#[derive(Debug)]
enum Backend {
    Loaded(dlopen2::raw::Library),
    #[cfg(windows)]
    Existing(ModuleReference),
}

#[cfg(windows)]
#[derive(Debug)]
struct ModuleReference(std::num::NonZeroUsize);

#[cfg(windows)]
impl Drop for ModuleReference {
    fn drop(&mut self) {
        // SAFETY: this guard owns the reference acquired by GetModuleHandleExW.
        unsafe { windows_sys::Win32::Foundation::FreeLibrary(self.0.get() as _) };
    }
}

/// A typed symbol that cannot outlive its owning library.
#[derive(Debug)]
pub struct Symbol<'library, T> {
    value: T,
    library: PhantomData<&'library Library>,
}

impl<T> Deref for Symbol<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Clone> Clone for Symbol<'_, T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            library: PhantomData,
        }
    }
}

/// A symbol whose library lifetime has been explicitly released by its unsafe caller.
pub struct RawSymbol<T> {
    value: T,
}

impl<T> Symbol<'_, T> {
    /// Release the borrow for pinned loaders that retain ownership of the library separately.
    ///
    /// # Safety
    /// The caller must keep the original library alive until all symbol uses have finished.
    pub unsafe fn into_raw(self) -> RawSymbol<T> {
        RawSymbol { value: self.value }
    }
}

impl<T> RawSymbol<T> {
    /// Return the address representation expected by the pinned EGL loader.
    pub fn into_raw(self) -> *mut std::ffi::c_void {
        // SAFETY: only get() constructs symbols, after checking pointer size; T's ABI is its caller's contract.
        unsafe { std::mem::transmute_copy(&self.value) }
    }
}

impl Library {
    /// Open a library, retaining ownership until all borrowed symbols are finished.
    ///
    /// # Safety
    /// The caller must ensure the library's initializers/finalizers are safe to execute.
    pub unsafe fn new<P: AsRef<OsStr>>(filename: P) -> Result<Self, Error> {
        validate_path(filename.as_ref())?;
        dlopen2::raw::Library::open(filename)
            .map(|library| Self(Backend::Loaded(library)))
            .map_err(Into::into)
    }

    /// Resolve a typed symbol using the lifetime of this library.
    ///
    /// # Safety
    /// `T` must be the symbol's exact ABI-compatible type, including calling convention.
    pub unsafe fn get<T>(&self, name: &[u8]) -> Result<Symbol<'_, T>, Error> {
        if std::mem::size_of::<T>() != std::mem::size_of::<*const ()>() {
            return Err(Error::Other(
                "symbol type must have pointer size".to_owned(),
            ));
        }
        let name = if name.last() == Some(&0) {
            CStr::from_bytes_with_nul(name)
                .map(CowName::Borrowed)
                .map_err(|e| Error::Other(e.to_string()))?
        } else {
            CowName::Owned(CString::new(name).map_err(|e| Error::Other(e.to_string()))?)
        };
        let value = match &self.0 {
            Backend::Loaded(library) => {
                // SAFETY: the caller guarantees T's ABI; pointer size and C-string validity were checked.
                unsafe { library.symbol_cstr(name.as_c_str()) }.map_err(|error| Error::DlSym {
                    desc: error.to_string(),
                })?
            }
            #[cfg(windows)]
            Backend::Existing(module) => {
                // SAFETY: the guard owns a live module reference and name is NUL-terminated.
                let address = unsafe {
                    windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                        module.0.get() as _,
                        name.as_c_str().as_ptr().cast(),
                    )
                }
                .ok_or_else(|| Error::DlSym {
                    desc: std::io::Error::last_os_error().to_string(),
                })?;
                // SAFETY: GetProcAddress returned a pointer; size is checked and its exact ABI is the caller's contract.
                unsafe { std::mem::transmute_copy(&address) }
            }
        };
        Ok(Symbol {
            value,
            library: PhantomData,
        })
    }

    /// Open a process/library with the platform's documented dlopen flags.
    ///
    /// # Safety
    /// As with new(), library initialization and finalization must be safe for the caller.
    #[cfg(unix)]
    pub unsafe fn open<P: AsRef<OsStr>>(filename: Option<P>, flags: i32) -> Result<Self, Error> {
        match filename {
            Some(path) => {
                validate_path(path.as_ref())?;
                dlopen2::raw::Library::open_with_flags(path, Some(flags))
                    .map(|library| Self(Backend::Loaded(library)))
                    .map_err(Into::into)
            }
            None => dlopen2::raw::Library::open_self()
                .map(|library| Self(Backend::Loaded(library)))
                .map_err(Into::into),
        }
    }

    /// Resolve an existing Windows module without loading one that is absent.
    #[cfg(windows)]
    pub fn open_already_loaded<P: AsRef<OsStr>>(filename: P) -> Result<Self, Error> {
        use std::os::windows::ffi::OsStrExt;
        validate_path(filename.as_ref())?;
        let name: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();
        let mut module = std::ptr::null_mut();
        // SAFETY: name is NUL-terminated and module points to a writable HMODULE.
        if unsafe {
            windows_sys::Win32::System::LibraryLoader::GetModuleHandleExW(
                0,
                name.as_ptr(),
                &mut module,
            )
        } == 0
        {
            return Err(Error::Other(std::io::Error::last_os_error().to_string()));
        }
        let module = std::num::NonZeroUsize::new(module as usize)
            .ok_or_else(|| Error::Other("Windows returned a null module".to_owned()))?;
        Ok(Self(Backend::Existing(ModuleReference(module))))
    }
}

fn validate_path(path: &OsStr) -> Result<(), Error> {
    if path.as_encoded_bytes().contains(&0) {
        return Err(Error::Other("library path contains NUL".to_owned()));
    }
    Ok(())
}

enum CowName<'a> {
    Borrowed(&'a CStr),
    Owned(CString),
}
impl CowName<'_> {
    fn as_c_str(&self) -> &CStr {
        match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s.as_c_str(),
        }
    }
}

/// Platform API aliases needed by pinned graphics dependencies.
pub mod os {
    #[cfg(unix)]
    pub mod unix {
        pub use crate::Library;
        pub use libc::{RTLD_LAZY, RTLD_LOCAL, RTLD_NOW};
    }
    #[cfg(windows)]
    pub mod windows {
        pub use crate::Library;
    }
}

#[cfg(test)]
mod tests {
    use super::Library;

    #[cfg(windows)]
    #[test]
    fn already_loaded_lookup_is_safe_and_does_not_load_absent_libraries()
    -> Result<(), Box<dyn std::error::Error>> {
        assert!(Library::open_already_loaded("bondline_missing_library_6439").is_err());
        let library = Library::open_already_loaded("kernel32.dll")?;
        // SAFETY: the documented Win32 function takes no parameters and returns a DWORD.
        let pid =
            unsafe { library.get::<unsafe extern "system" fn() -> u32>(b"GetCurrentProcessId")? };
        // SAFETY: the symbol has the documented ABI and its module is kept alive.
        assert_eq!(unsafe { pid() }, std::process::id());
        Ok(())
    }

    #[test]
    fn resolves_platform_pid_against_rust_standard_library()
    -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        let name = "/usr/lib/libSystem.B.dylib";
        #[cfg(all(unix, not(target_os = "macos")))]
        let name = "libc.so.6";
        #[cfg(windows)]
        let name = "kernel32.dll";
        // SAFETY: the platform system library remains owned until after the symbol is used.
        let library = unsafe { Library::new(name)? };
        #[cfg(unix)]
        let symbol_name = b"getpid".as_slice();
        #[cfg(windows)]
        let symbol_name = b"GetCurrentProcessId".as_slice();
        #[cfg(unix)]
        // SAFETY: getpid has the platform C ABI and returns pid_t.
        let pid = unsafe { library.get::<unsafe extern "C" fn() -> libc::pid_t>(symbol_name)? };
        #[cfg(windows)]
        // SAFETY: GetCurrentProcessId has the Windows system ABI and returns DWORD.
        let pid = unsafe { library.get::<unsafe extern "system" fn() -> u32>(symbol_name)? };
        #[cfg(unix)]
        // SAFETY: the function signature is exact and its library remains alive.
        let actual = u32::try_from(unsafe { pid() })?;
        #[cfg(windows)]
        // SAFETY: the symbol has the documented platform signature and its library is alive.
        let actual = unsafe { pid() };
        assert_eq!(actual, std::process::id());
        // SAFETY: the library remains in scope after explicitly erasing the symbol's borrow.
        let raw = unsafe { pid.into_raw() };
        assert!(!raw.into_raw().is_null());
        // SAFETY: requesting an invalid type is rejected before dlopen2 can panic.
        assert!(unsafe { library.get::<u128>(symbol_name) }.is_err());
        assert!(matches!(
            // SAFETY: a nonexistent symbol must fail, never be called.
            unsafe { library.get::<*const ()>(b"bondline_missing_symbol_6439") },
            Err(super::Error::DlSym { .. } | super::Error::DlSymUnknown)
        ));
        // SAFETY: invalid names are rejected before resolving a symbol.
        assert!(unsafe { library.get::<*const ()>(b"getpid\0extra") }.is_err());
        Ok(())
    }

    #[test]
    fn missing_libraries_return_errors() {
        // SAFETY: the nonexistent library cannot execute an initializer.
        assert!(unsafe { Library::new("bondline_missing_library_6439") }.is_err());
    }
}
