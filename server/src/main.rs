use std::io::prelude::*;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::sync::atomic;

use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread;

use std::collections::HashMap;
use std::collections::HashSet;
use std::net::{TcpStream, ToSocketAddrs};

struct Server {
    user_to_address : HashMap<String, String>
}

impl Server {
    fn new() ->  Result<Server, String> {
        let user_to_address = HashMap::new();
        Ok(Server {
            user_to_address
        })
    }

    fn requestHandler(&mut self, mut stream : TcpStream) {
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();
        let mut query_buffer :Vec<u8> = Vec::new();
        for i in 0..512 {
            if buffer[i] == 0 {
                break;
            }
            query_buffer.push(buffer[i]);
        }
        let query = std::str::from_utf8(&query_buffer).unwrap();

        println!("query: {}", query);
        let split = query.split(" ");
        let query_list : Vec<&str> = split.collect();
        println!("query_list: {}", query_list.join(","));
    
        match query_list[0]  {
            "login" => {
                println!("stream address: {}", stream.peer_addr().unwrap());
                self.user_to_address.insert(query_list[1].to_string(), query_list[2].to_string());
            },
            "msg" => {
                println!("stream address: {}", stream.peer_addr().unwrap());
                
                let sender = query_list[1].to_string();
                let receiver = (query_list.get(2).expect("you didn't send any id")).to_string();
                let response = format!("{}>{}", sender, query_list[3..].join(" "));
                
                let mut server = self.user_to_address.get(&receiver).unwrap().to_socket_addrs().unwrap();
                let mut server_real = "127.0.0.1:7875".to_socket_addrs().unwrap();
            
                println!("Server Server Real {:?} {:?}", server, server_real);
                match TcpStream::connect(server.next().unwrap()) {
                    Ok(mut receiverStream) => {
                        receiverStream.write(response.as_bytes()).unwrap();
                        receiverStream.flush().unwrap();
                    },
                    Err(error) => {
                        println!("error: {}", error);    
                    }
                }
                

            },
            _ => {
                println!("stream address: {}", stream.peer_addr().unwrap());
                
                stream.write(b"unknown command").unwrap();
            }
        }
        println!("I am after match");
    }
}


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("cannot bind to host");
    let mut server = Arc::new(Mutex::new(Server::new().unwrap()));
    for stream in listener.incoming() {
        println!("new stream");
        let stream = stream.unwrap();
        let server = Arc::clone(&server);
        // thread::spawn(move || {
            let mut server = server.lock().unwrap();
            server.requestHandler(stream);
            println!("I am after request handler");
        // });
    }
    
}

/*протокол
    client
    
    login id

    msg toid message

    server

    response

    login -> "You are successfully logged in"
    msg -> id>message
*/
