//! Windows and Linux specifics; the only place for `cfg(target_os)` branches.
//!
//! VestaDesk supports only Windows and Linux: building it for any other system fails with a clear message.

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("VestaDesk supports only Windows and Linux.");
