use std::cell::RefCell;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
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

impl From<json_parser::JsonError> for IPCError {
    fn from(e: json_parser::JsonError) -> IPCError {
        return IPCError::JsonError(e);
    }
}

struct IPCFormat {
    payload_len: u32,
    payload_type: u32,
    payload: String,
}

const MAGIC_STR: &str = "i3-ipc";

enum WorkspaceEvent_T {
    Focused,
    Initialized,
    Empty,
}

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
    let (tx, rx) = mpsc::channel();

    let workspace_msg = IPCFormat {
        payload_len: 13,
        payload_type: IPCMessages::Subscribe as u32,
        payload: String::from("[\"workspace\"]"),
    };

    let event_listener_thread = thread::spawn(move || {
        let fd = Arc::new(Mutex::new(connect()?));
        println!("created new thread");
        loop {
            println!("loopstart");
            let fd_borrow = Arc::clone(&fd);
            send(fd_borrow, &workspace_msg);
            let fd_borrow_2 = Arc::clone(&fd);
            if let Ok(json_string) = recv(fd_borrow_2) {
                match json_string.as_str() {
                    "{\"success\": true}" => {
                        println!("subscribed!");
                        // TODO handle this?
                        if let Ok(json_event_string) = recv(Arc::clone(&fd)) {
                            ws_event_handler(Arc::clone(&fd), json_event_string.as_str());
                        }
                    }
                    "{\"success\": false}" => panic!("RUH ROH FAILED OT SUBSCRIPT"), // TODO handle
                                                                                     // this better
                    _ => {
                        println!("event triggered");
                        // TODO handle this
                        if let Ok(json_event_string) = recv(Arc::clone(&fd)) {
                            ws_event_handler(Arc::clone(&fd), json_event_string.as_str());
                        }
                    }
                }
            }
        }
    });

    let workspace_status_thread = thread::spawn(move || {
        let fd = Arc::new(Mutex::new(connect()?));
        // TODO send and receive messages from event_listener_thread
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
    match fd.write_all(&header) {
        Ok(_) => {} // TODO: should check the number of bytes written == to size of the message, if
        // not throw a new error.
        Err(e) => return Err(IPCError::WriteError(e)),
    }
    Ok(())
}

fn recv(fd_mutex: Arc<Mutex<UnixStream>>) -> Result<String, IPCError> {
    let mut fd = fd_mutex.lock().unwrap();
    let mut buf_header = [0u8; 14];
    fd.read_exact(&mut buf_header);
    let payload_size: u32 =
        u32::from_ne_bytes([buf_header[6], buf_header[7], buf_header[8], buf_header[9]]);
    let mut payload = vec![0u8; payload_size as usize];
    fd.read_exact(&mut payload);
    Ok(String::from_utf8_lossy(&payload).into_owned())
}

fn ws_event_handler(fd_mutex: Arc<Mutex<UnixStream>>, json_str: &str) -> Result<(), IPCError> {
    match client_state_mux(json_str) {
        Ok(WorkspaceEvent_T::Focused) => {
            ws_focus_handler(Arc::clone(&fd_mutex));
        }
        Ok(WorkspaceEvent_T::Initialized) | Ok(WorkspaceEvent_T::Empty) => {
            // TODO handle
            if let Ok(focus_json_str) = recv(Arc::clone(&fd_mutex)) {
                ws_event_handler(Arc::clone(&fd_mutex), &focus_json_str.as_str());
            }
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

fn ws_focus_handler(fd_mutex: Arc<Mutex<UnixStream>>) -> Result<(), IPCError> {
    let mut fd = fd_mutex.lock().unwrap();
    let mut payload = MAGIC_STR.as_bytes().to_vec();
    payload.append(&mut [0u8; 4].to_vec());
    payload.append(&mut (IPCMessages::GetWorkspaces as u32).to_ne_bytes().to_vec());
    println!("sending: {:?}", payload);
    if let Err(e) = fd.write_all(&mut payload) {
        println!("1: {e}");
    }
    //if let Err(e) = fd.shutdown(std::net::Shutdown::Write) {
    //    println!("2: {e}");
    //}
    let mut buf_header = [0u8; 14];
    if let Err(e) = fd.read_exact(&mut buf_header) {
        println!("3: {e}");
    }
    let payload_size: u32 =
        u32::from_ne_bytes([buf_header[6], buf_header[7], buf_header[8], buf_header[9]]);
    let mut payload = vec![0u8; payload_size as usize];
    fd.read_exact(&mut payload);
    let buf_string_json: String = String::from_utf8_lossy(&payload).into_owned();
    println!("{}", buf_string_json);
    if let Ok(json_parser::JsonEntry::Array(workspace_json)) = json_parser::stojson(Rc::new(RefCell::new(buf_string_json))) {
        println!("Workspaces: {}", workspace_json.len());
    }

    Ok(())
}

fn client_state_mux(ipc_message: &str) -> Result<WorkspaceEvent_T, IPCError> {
    let match_key: &str = "{ \"change\": ";
    return match &ipc_message[0..12] {
        match_key => {
            if &ipc_message[13..18] == "focus" {
                return Ok(WorkspaceEvent_T::Focused);
            } else if &ipc_message [13..17] == "init" {
                return Ok(WorkspaceEvent_T::Initialized);
            } else if &ipc_message [13..18] == "empty" {
                return Ok(WorkspaceEvent_T::Empty);
            } else {
                return Err(IPCError::GeneralError);
            }
        }
        _ => Err(IPCError::GeneralError),
   }
}
