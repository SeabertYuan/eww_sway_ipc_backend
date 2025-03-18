use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::thread;

use eww_sway_ipc_backend::*;

fn main() -> Result<(), Box<dyn Error>> {
    crate::run();
    Ok(())
}
