use std::io;
use std::io::Write;

#[cfg(unix)]
use std::os::fd::{AsRawFd, RawFd};

/// Sets a file descriptor to non-blocking mode on Unix systems
#[cfg(unix)]
pub fn set_nonblocking(fd: RawFd) -> Result<(), io::Error> {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags == -1 {
            return Err(io::Error::last_os_error());
        }

        if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) == -1 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

/// Waits for a file descriptor to become writable using poll().
/// This is more efficient than sleeping when handling WouldBlock errors.
/// Returns Ok(()) if the fd becomes writable, or Err if poll fails.
#[cfg(unix)]
pub(crate) fn wait_writable(fd: RawFd) -> Result<(), io::Error> {
    unsafe {
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLOUT,
            revents: 0,
        };

        // Wait indefinitely for the fd to become writable
        let ret = libc::poll(&mut pollfd as *mut libc::pollfd, 1, -1);

        if ret == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

macro_rules! write_with_retry_internal {
    ($out:expr, $msg:expr) => {{
        let mut out = $out;
        let bytes = $msg.as_bytes();
        let mut written = 0;

        #[cfg(unix)]
        let raw_fd = out.as_raw_fd();

        while written < bytes.len() {
            match out.write(&bytes[written..]) {
                Ok(0) => {
                    #[cfg(unix)]
                    {
                        // Nothing accepted, wait for fd to become writable
                        if wait_writable(raw_fd).is_err() {
                            // If poll fails, give up
                            break;
                        }
                    }

                    #[cfg(windows)]
                    {
                        // On Windows, just retry
                    }
                }
                Ok(n) => {
                    // Remove written bytes
                    written += n;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    #[cfg(unix)]
                    {
                        // Wait for fd to become writable
                        if wait_writable(raw_fd).is_err() {
                            // If poll fails, give up
                            break;
                        }
                    }

                    #[cfg(windows)]
                    {
                        // On Windows, just retry
                    }
                }
                Err(_) => {
                    // Hard error, give up
                    break;
                }
            }
        }
    }};
}

/// Internal function for writing error messages to STDERR with retry logic.
#[allow(unused)]
pub(crate) fn write_stderr_with_retry_internal(msg: &str) {
    let out = io::stderr();
    let formatted = format!("[log_nonblock error] {}\n", msg);
    write_with_retry_internal!(out.lock(), &formatted);
}

/// Internal function for writing error messages to STDOUT with retry logic.
#[allow(unused)]
pub(crate) fn write_stdout_with_retry_internal(msg: &str) {
    let out = io::stdout();
    let formatted = format!("[log_nonblock error] {}\n", msg);
    write_with_retry_internal!(out.lock(), &formatted);
}

/// Writes a message to stdout with retry logic, without adding any prefix.
/// This function is used by the `println!` macro when the `macros` feature is enabled.
#[doc(hidden)]
#[cfg(feature = "macros")]
pub fn write_stdout_with_retry(msg: &str) {
    let out = io::stdout();
    write_with_retry_internal!(out.lock(), msg);
}

/// Writes a message to stderr with retry logic, without adding any prefix.
/// This function is used by the `eprintln!` macro when the `macros` feature is enabled.
#[doc(hidden)]
#[cfg(feature = "macros")]
pub fn write_stderr_with_retry(msg: &str) {
    let out = io::stderr();
    write_with_retry_internal!(out.lock(), msg);
}
