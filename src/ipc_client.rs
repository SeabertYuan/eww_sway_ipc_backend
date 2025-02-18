use std::cell::RefCell;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{env, io};

use crate::json_parser;

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

pub enum IPCError {
    ConnectionError(io::Error),
    PathNotFoundError,
    GeneralError,
    WriteError(io::Error),
    ShutdownError(io::Error),
    JsonError(json_parser::JsonError),
}

struct IPCFormat {
    payload_len: u32,
    payload_type: u32,
    payload: String,
}

const MAGIC_STR: &str = "i3-ipc";

fn connect() -> Result<UnixStream, IPCError> {
    match env::var_os("SWAYSOCK") {
        Some(opt) => {
            // connect
            match UnixStream::connect(opt) {
                Ok(fd) => return Ok(fd),
                Err(e) => return Err(IPCError::ConnectionError(e)),
            }
        }
        None => {
            return Err(IPCError::PathNotFoundError);
        }
    }
}

fn send_msg() -> io::Result<()> {
    Ok(())
}

pub fn run_ipc() -> Result<(), IPCError> {
    let fd = Arc::new(Mutex::new(connect()?));

    let workspace_msg = IPCFormat {
        payload_len: 13,
        payload_type: IPCMessages::Subscribe as u32,
        payload: String::from("[\"workspace\"]"),
    };

    let workspace_thread = thread::spawn(move || {
        println!("created new thread");
        loop {
            let fd_borrow = Arc::clone(&fd);
            send(fd_borrow, &workspace_msg);
            let fd_borrow_2 = Arc::clone(&fd);
            let num_workspaces = match recv(fd_borrow_2) {
                Ok(json) => match json {
                    json_parser::JsonEntry::Array(jsarr) => jsarr.len(),
                    _ => 0 as usize,
                },
                Err(e) => 0,
            };
            println!("{}", num_workspaces);
        }
    });

    workspace_thread.join().unwrap();

    println!("done");
    Ok(())
}

fn send(fd: Arc<Mutex<UnixStream>>, message: &IPCFormat) -> Result<(), IPCError> {
    let mut payload: Vec<u8> = message.payload.as_bytes().to_vec();
    let mut header: Vec<u8> = MAGIC_STR.as_bytes().to_vec();
    header.append(&mut message.payload_len.to_ne_bytes().to_vec());
    header.append(&mut message.payload_type.to_ne_bytes().to_vec());
    header.append(&mut payload);

    let mut fd = fd.lock().unwrap();
    match fd.write(&header) {
        Ok(_) => {} // TODO: should check the number of bytes written == to size of the message, if
        // not throw a new error.
        Err(e) => return Err(IPCError::WriteError(e)),
    }

    match fd.shutdown(std::net::Shutdown::Write) {
        Ok(_) => {} // TODO: same
        Err(e) => return Err(IPCError::ShutdownError(e)),
    }

    Ok(())
}

fn recv(fd_mutex: Arc<Mutex<UnixStream>>) -> Result<json_parser::JsonEntry, IPCError> {
    let mut fd = fd_mutex.lock().unwrap();
    let mut buf_header = [0u8; 14];
    fd.read_exact(&mut buf_header);
    let payload_size: u32 =
        u32::from_ne_bytes([buf_header[6], buf_header[7], buf_header[8], buf_header[9]]);
    let mut payload = vec![0u8; payload_size as usize];
    fd.read_exact(&mut payload);
    let buf_string_json: String = String::from_utf8_lossy(&payload).into_owned();
    match buf_string_json.as_str() {
        "{\"success\": true}" => {
            drop(fd);
            Ok(recv(Arc::clone(&fd_mutex))?)
        }
        "{\"success\": false}" => Err(IPCError::GeneralError),
        _ => {
            let mut payload = MAGIC_STR.as_bytes().to_vec();
            payload.append(&mut [0u8; 4].to_vec());
            payload.append(&mut (IPCMessages::GetWorkspaces as usize).to_ne_bytes().to_vec());
            fd.write(&mut payload);
            fd.shutdown(std::net::Shutdown::Write);
            let mut buf_header = [0u8; 14];
            fd.read_exact(&mut buf_header);
            let payload_size: u32 =
                u32::from_ne_bytes([buf_header[6], buf_header[7], buf_header[8], buf_header[9]]);
            let mut payload = vec![0u8; payload_size as usize];
            fd.read_exact(&mut payload);
            let buf_string_json: String = String::from_utf8_lossy(&payload).into_owned();
            match json_parser::stojson(Rc::new(RefCell::new(buf_string_json))) {
                Ok(json) => Ok(json),
                Err(e) => Err(IPCError::JsonError(e)),
            }
        }
    }
}
