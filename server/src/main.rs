use std::io::prelude::*;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::sync::atomic;
use std::time::Duration;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;

use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread;

use std::collections::HashMap;
use std::collections::HashSet;
use std::net::{TcpStream, ToSocketAddrs};

type Signature = Vec<u8>;
type Connection = (Signature, TcpStream);
type ConnectionContainer = HashMap<Signature, TcpStream>;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("cannot bind to host");
    let mut connections : Arc<Mutex<ConnectionContainer>> = Arc::new(Mutex::new(HashMap::new()));
    let mut users : Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
    
    let (sender, receiver) = channel();
    start_connection_thread(&mut connections, &mut users, receiver);

    for stream in listener.incoming() {
        let stream      = stream.unwrap();
        let connections = Arc::clone(&connections);
        let users       = Arc::clone(&users);

        handle_connection(users, connections, stream);
    }
    sender.send(1).unwrap();
}

fn start_connection_thread(
    connections : &mut Arc<Mutex<ConnectionContainer>>,
    users       : &mut Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    receiver    : Receiver<u8>)
{
    let connections = Arc::clone(connections);
    let users       = Arc::clone(users);

    thread::spawn(move || {
        let mut connections = connections.lock().unwrap();
        let mut users       = users.lock().unwrap();
        loop {
            if(receiver.recv().unwrap() == 1) {
                break;
            }
            println!("{}", connections.len());
            let mut messages = Vec::new();
            for (_, mut stream) in connections.iter() {
                stream.set_read_timeout(Some(Duration::from_millis(10)));
                let mut buffer = [0; 1025];
                match stream.read(&mut buffer) {
                    Ok(n) => {
                        let signature    = buffer[0..32].to_vec();
                        let message_size = vec_to_u32(&buffer[32..36].to_vec());
                        let message      = buffer[36..(message_size as usize + 36)].to_vec();
                        messages.push((signature, message));
                    },
                    Err(error) => println!("Encountered error in notification stream: {}", error),
                }
            }

            for (signature, mut stream) in connections.iter() {
                for (signature, message) in messages.iter() {
                    let signature = signature.to_vec();
                    let responce = [
                        u32_to_vec(users.get(&signature).unwrap().len() as u32),
                        u32_to_vec(message.len() as u32),
                        users.get(&signature).unwrap().to_vec(),
                        message.to_vec()
                    ].concat();
                    stream.write(responce.as_ref()).unwrap();
                    stream.flush().unwrap();
                }
            }
        }
    });
}

fn handle_connection(users: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>, connections: Arc<Mutex<ConnectionContainer>>, mut stream : TcpStream) {
    let mut buffer = [0; 1025];
    stream.read(&mut buffer).unwrap();

    let mut connections = connections.lock().unwrap();
    let mut users = users.lock().unwrap();
    match buffer[0] {
        1 => {
            let signature = buffer[1..33].to_vec();
            let name_size = vec_to_u32(&buffer[33..37].to_vec());
            if name_size > 1025 {
                return;
            }
            let name = buffer[37..(name_size as usize + 37)].to_vec();
            connections.insert(Vec::clone(&signature), stream);
            users.insert(Vec::clone(&signature), name);
        },
        _ => println!("unknown command"),
    }
}

pub fn vec_to_u32(data : &Vec<u8>) -> u32 {
    ((data[0] as u32) << 24) + ((data[1] as u32) << 16) + ((data[2] as u32) << 8) + (data[3] as u32)
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