use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::io::{Read, Write};
use std::io;
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let (sender, receiver) = channel::<i32>();

    let addrip = "127.0.0.1:7875";
    let name = "alex";

    let initQuery = format!("login {} {}", name, addrip.clone());
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
    stream.write(initQuery.as_bytes()).unwrap();
    stream.flush().unwrap();

    thread::spawn(move || {
        let listener = TcpListener::bind(addrip).expect("cannot bind host");
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut bytes = [0; 512];
            stream.read(&mut bytes).unwrap();
            println!("{}", std::str::from_utf8(&bytes).unwrap());
            if(receiver.recv().unwrap() == -1) {
                break;
            }
        }
    });
    loop {
        let mut input = String::new();
        
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if input.starts_with("stop") {
                    sender.send(-1).unwrap();
                    break;
                }
                let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
                stream.write(input.as_bytes()).expect("Cannot send command to server");
                stream.flush().unwrap();
            },
            Err(error) => {
                println!("error: {}", error);
            }
        }
    }
}
