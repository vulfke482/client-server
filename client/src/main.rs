extern crate rand;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::io::{Read, Write};
use std::io;
use std::sync::mpsc::channel;
use std::thread;



fn main() {

    let name = "denis";
    let signature: Vec<u8> = (0..32).map(|_| { rand::random::<u8>() }).collect(); 
    let init_request = [[1].to_vec(), Vec::clone(&signature),u32_to_vec(name.len() as u32), name.as_bytes().to_vec()].concat();

    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
   
    stream.write(init_request.as_ref()).unwrap();
    stream.flush().unwrap();

    start_connection_thread(&stream);


    loop {
        let mut input = String::new();
        
        match io::stdin().read_line(&mut input) {
            Ok(n) => {

                let input_list : Vec<&str>= input.split(" ").collect();
                match input_list[0] {
                    "stop" => break,
                    "msg" => {
                        let message = input_list[1..].join(" ").as_bytes().to_vec();
                        let message_size = u32_to_vec(message.len() as u32);
                        
                        let request = [
                            Vec::clone(&signature),
                            message_size,
                            message
                        ].concat();

                        stream.write(&request).unwrap();
                        stream.flush().unwrap();

                        println!("{}> {}", name, input_list[1..].join(" "));
                    },
                    _ => println!("unknown command"),
                }
                
            },
            Err(error) => {
                println!("error: {}", error);
            }
        }
    }
}

fn start_connection_thread(stream : &TcpStream) {
    let mut stream = TcpStream::try_clone(stream).unwrap();
    thread::spawn(move || {
        loop {
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer).unwrap();
            if n == 0 {
                continue;
            }
            let name_size = vec_to_u32(&buffer[0..4].to_vec());
            let message_size = vec_to_u32(&buffer[4..8].to_vec());
            let name = buffer[8..(name_size as usize + 8)].to_vec();
            let message = buffer[(name_size as usize + 8)..((name_size as usize + 8) + message_size as usize)].to_vec();
            println!("{}> {}", std::str::from_utf8(&name).unwrap(), std::str::from_utf8(&message).unwrap());
        }
    });
}

fn u32_to_vec(num: u32) -> Vec<u8> {
    let mask = (1 << 8) - 1;
    vec![
        (num & (mask << 24)) as u8,
        (num & (mask << 16)) as u8,
        (num & (mask << 8)) as u8,
        (num & mask) as u8,
    ]
}

pub fn vec_to_u32(data : &Vec<u8>) -> u32 {
    ((data[0] as u32) << 24) + ((data[1] as u32) << 16) + ((data[2] as u32) << 8) + (data[3] as u32)
}