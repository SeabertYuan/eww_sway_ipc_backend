//use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::{env, io};

#[repr(u32)]
enum IPCMessages {
    RunCommand = 0u32,
    GetWorkspaces = 1u32,
    Subscribe = 2u32,
    GetOutputs = 3u32,
    GetTree = 4u32,
    GetMarks = 5u32,
    GetBarConfig = 6u32,
    GetVersion = 7u32,
    GetBindingModes = 8u32,
    GetConfig = 9u32,
    SendTick = 10u32,
    Sync = 11u32,
    GetBindingState = 12u32,
    GetInputs = 100u32,
    GetSeats = 101u32,
}

#[repr(u32)]
enum IPCEvents {
    Workspace = (1u32 << 31) | 0,
    Output = (1u32 << 31) | 1,
    Mode = (1u32 << 31) | 2,
    Window = (1u32 << 31) | 3,
    BarConfigUpdate = (1u32 << 31) | 4,
    Binding = (1u32 << 31) | 5,
    Shutdown = (1u32 << 31) | 6,
    Tick = (1u32 << 31) | 7,
    BarStateUpdate = (1u32 << 31) | 0x14,
    Input = (1u32 << 31) | 0x15,
}

struct IPCFormat {
    payload_len: u32,
    payload_type: u32,
    payload: String,
}

const MAGIC_STR: &str = "i3-ipc";

fn connect() -> io::Result<()> {
    if let Some(opt) = env::var_os("SWAYSOCK") {
        // connect
        let fd = UnixStream::connect(opt)?;
    };
    Ok(())
}

fn send_msg() -> io::Result<()> {
    Ok(())
}
