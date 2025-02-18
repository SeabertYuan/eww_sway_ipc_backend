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

fn bruh() -> Result<(), Box<dyn Error>> {
    eww_sway_ipc_backend::run();
    // get workspaces
    let message_b: [u8; 14] = [
        0x69, 0x33, 0x2d, 0x69, 0x70, 0x63, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    ];
    // sway exit command
    let message_b_2: [u8; 18] = [
        0x69, 0x33, 0x2d, 0x69, 0x70, 0x63, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65,
        0x78, 0x69, 0x74,
    ];

    let workspace_thread = thread::spawn(|| -> Result<(), Box<dyn Error + Send>> {
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
        let mut socket = UnixStream::connect(&socket_path).unwrap();
        // subscribe
        let mut workspace_b: Vec<u8> = vec![
            0x69, 0x33, 0x2d, 0x69, 0x70, 0x63, 0x0d, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        ];
        b"[\"workspace\"]".iter().for_each(|b| workspace_b.push(*b));

        socket.write(&workspace_b);
        socket.shutdown(std::net::Shutdown::Write);

        let mut buf_header: [u8; 14] = [0u8; 14];
        socket.read_exact(&mut buf_header);

        let payload_size: u32 =
            u32::from_ne_bytes([buf_header[6], buf_header[7], buf_header[8], buf_header[9]]);
        println!("Parsed payload size!");

        let mut payload = vec![0u8; payload_size as usize];
        socket.read_exact(&mut payload);
        //println!("Returned the message: {buf_string}");

        let buf_string_json: String = String::from_utf8_lossy(&payload).into_owned();

        println!("{}", buf_string_json);
        let mut buf_header: [u8; 14] = [0u8; 14];
        socket.read_exact(&mut buf_header);

        let payload_size: u32 =
            u32::from_ne_bytes([buf_header[6], buf_header[7], buf_header[8], buf_header[9]]);
        println!("Parsed payload size!");

        let mut payload = vec![0u8; payload_size as usize];
        socket.read_exact(&mut payload);
        //println!("Returned the message: {buf_string}");

        let buf_string_json: String = String::from_utf8_lossy(&payload).into_owned();

        println!("{}", buf_string_json);

        /*
        if let Ok(eww_sway_ipc_backend::json_parser::JsonEntry::Array(res)) =
            eww_sway_ipc_backend::json_parser::stojson_list(Rc::new(RefCell::new(buf_string_json)))
        {
            println!("Number of workspaces: {}", res.len());
        }
        */
        println!("waiting for event");

        let mut buf_header_2: [u8; 14] = [0u8; 14];
        socket.read_exact(&mut buf_header_2);
        let payload_size_2: u32 = u32::from_ne_bytes([
            buf_header_2[6],
            buf_header_2[7],
            buf_header_2[8],
            buf_header_2[9],
        ]);
        let mut payload_2 = vec![0u8; payload_size_2 as usize];
        socket.read_exact(&mut payload_2);

        let buf_string_json: String = String::from_utf8_lossy(&payload_2).into_owned();

        println!("{}", buf_string_json);
        println!("workspace thread done");
        Ok(())
    });
    let mut window_b: Vec<u8> = vec![
        0x69, 0x33, 0x2d, 0x69, 0x70, 0x63, 0x0a, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
    ];
    let window_thread = thread::spawn(|| -> Result<(), Box<dyn Error + Send>> {
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
        let mut socket = UnixStream::connect(socket_path).unwrap();
        let mut window_b: Vec<u8> = vec![
            0x69, 0x33, 0x2d, 0x69, 0x70, 0x63, 0x0a, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        ];

        b"[\"window\"]".iter().for_each(|b| window_b.push(*b));

        socket.write(&window_b);
        socket.shutdown(std::net::Shutdown::Write);

        let mut buf_header: [u8; 14] = [0u8; 14];
        socket.read_exact(&mut buf_header);

        let payload_size: u32 =
            u32::from_ne_bytes([buf_header[6], buf_header[7], buf_header[8], buf_header[9]]);
        println!("Parsed payload size!");

        let mut payload = vec![0u8; payload_size as usize];
        socket.read_exact(&mut payload);
        //println!("Returned the message: {buf_string}");

        let buf_string_json: String = String::from_utf8_lossy(&payload).into_owned();

        println!("{}", buf_string_json);
        let mut buf_header: [u8; 14] = [0u8; 14];
        socket.read_exact(&mut buf_header);

        let payload_size: u32 =
            u32::from_ne_bytes([buf_header[6], buf_header[7], buf_header[8], buf_header[9]]);
        println!("Parsed payload size!");

        let mut payload = vec![0u8; payload_size as usize];
        socket.read_exact(&mut payload);
        //println!("Returned the message: {buf_string}");

        let buf_string_json: String = String::from_utf8_lossy(&payload).into_owned();

        println!("{}", buf_string_json);

        /*
        if let Ok(eww_sway_ipc_backend::json_parser::JsonEntry::Array(res)) =
            eww_sway_ipc_backend::json_parser::stojson_list(Rc::new(RefCell::new(buf_string_json)))
        {
            println!("Number of workspaces: {}", res.len());
        }
        */
        println!("waiting for event");

        let mut buf_header_2: [u8; 14] = [0u8; 14];
        socket.read_exact(&mut buf_header_2);
        let payload_size_2: u32 = u32::from_ne_bytes([
            buf_header_2[6],
            buf_header_2[7],
            buf_header_2[8],
            buf_header_2[9],
        ]);
        let mut payload_2 = vec![0u8; payload_size_2 as usize];
        socket.read_exact(&mut payload_2);

        let buf_string_json: String = String::from_utf8_lossy(&payload_2).into_owned();

        println!("{}", buf_string_json);
        println!("window thread done");
        Ok(())
    });

    workspace_thread.join().unwrap();
    window_thread.join().unwrap();

    println!("done");

    Ok(())
}
