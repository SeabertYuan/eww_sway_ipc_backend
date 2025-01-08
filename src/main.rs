use std::ffi::OsString;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::{env, io, str};

use eww_sway_ipc_backend::*;

fn main() -> io::Result<()> {
    eww_sway_ipc_backend::run();
    let socket_path_opt: Option<std::ffi::OsString> = env::var_os("SWAYSOCK");
    let socket_path: OsString = match socket_path_opt {
        Some(path) => path,
        None => panic!("Sway IPC socket path not found"),
    };
    let path_string: String = socket_path
        .clone()
        .into_string()
        .unwrap_or_else(|res| panic!("something went wrong with result: {res:?}"));
    println!("got path: {}", path_string);

    let mut socket = UnixStream::connect(socket_path)?;

    let message_b: [u8; 14] = [
        0x69, 0x33, 0x2d, 0x69, 0x70, 0x63, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    ];
    let message_b_2: [u8; 16] = [
        0x69, 0x33, 0x2d, 0x69, 0x70, 0x63, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65,
        0x78,
    ];
    socket.write_all(&message_b)?;
    println!("Written message!");
    socket.shutdown(std::net::Shutdown::Write)?;
    println!("shut down!");
    let mut buf_string: String = String::new();
    let mut buf: [u8; 32] = [0; 32];
    while let Ok(bytes_read) = socket.read(&mut buf) {
        //println!("{bytes_read}");
        if bytes_read < 32 {
            buf_string.push_str(str::from_utf8(&buf[0..bytes_read]).unwrap());
            break;
        } else {
            buf_string.push_str(str::from_utf8(&buf).unwrap());
        }
    }
    println!("Read the message!");
    //println!("Returned the message: {buf_string}");

    let buf_string_json: &str = &buf_string[16..buf_string.len() - 2];

    let num_workspaces: i32 = get_num_workspaces(buf_string_json);

    println!("The number of workspaces is: {num_workspaces}");

    Ok(())
}
