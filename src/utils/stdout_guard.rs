//! Keeps stdout reserved for the command's JSON result.
//!
//! The Steam client libraries write their own diagnostics to the process's
//! stdout, for example on shutdown: "Trying to close low level socket support,
//! but we still have sockets open!". That text ended up after the JSON and broke
//! parsing. Before Steam is initialized, stdout (the C runtime's fd 1 and the
//! Win32 standard output handle) is pointed at stderr, and the real stdout is kept
//! aside for the result alone.

use std::fs::File;
use std::io::Write;
use std::mem::ManuallyDrop;
use std::os::windows::io::FromRawHandle;

unsafe extern "C" {
    fn _dup(fd: i32) -> i32;
    fn _dup2(source: i32, target: i32) -> i32;
    fn _get_osfhandle(fd: i32) -> isize;
}

const STDOUT_FD: i32 = 1;
const STDERR_FD: i32 = 2;

pub struct StdoutGuard {
    /// A duplicate of the original stdout, or `None` if the redirect failed.
    original: Option<i32>,
}

impl StdoutGuard {
    /// Sends everything written to stdout from now on to stderr.
    pub fn redirect() -> Self {
        // SAFETY: plain C runtime calls on the standard descriptors. In console
        // apps, `_dup2` on fd 1 also updates the Win32 STD_OUTPUT_HANDLE, which
        // covers libraries with their own C runtime and Rust's std::io::stdout.
        unsafe {
            let original = _dup(STDOUT_FD);
            if original < 0 {
                return Self { original: None };
            }
            if _dup2(STDERR_FD, STDOUT_FD) != 0 {
                return Self { original: None };
            }
            Self {
                original: Some(original),
            }
        }
    }

    /// Writes `text` and a newline to the original stdout.
    pub fn write_line(&self, text: &str) {
        let Some(fd) = self.original else {
            println!("{}", text);
            return;
        };
        // SAFETY: `fd` was returned by `_dup` and stays open for the life of the
        // process; ManuallyDrop leaves closing it to the C runtime.
        let mut file = ManuallyDrop::new(unsafe {
            File::from_raw_handle(_get_osfhandle(fd) as *mut std::ffi::c_void)
        });
        let _ = writeln!(file, "{}", text);
        let _ = file.flush();
    }
}
