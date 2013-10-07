//
// zhttpto.rs
//
// Reference solution for PS1
// Running on Rust 0.8
//
// Note that this code has serious security risks!  You should not run it 
// on any system with access to sensitive files.
//
// Special thanks to Kiet Tran for providing code we incorporated into this.
// 
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans
// Version 0.2

extern mod extra;

use std::rt::io::*;
use std::rt::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::println;
use std::cell::Cell;
use std::task;
use std::{os, str, io};

static PORT:    int = 4414;
static IPV4_LOOPBACK: &'static str = "127.0.0.1";
static mut visitor_count: uint = 0;


fn main() {
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: Ipv4Addr(127,0,0,1), port: PORT as u16});
    
    println(fmt!("Listening on tcp port %d ...", PORT));
    let mut acceptor = socket.listen().unwrap();
    
    // we can limit the incoming connection count.
    //for stream in acceptor.incoming().take(10 as uint) {
    for stream in acceptor.incoming() {
        println!("Saw connection!");
        let stream = Cell::new(stream);
        // Start a task to handle the connection
        do task::spawn {
            unsafe {
                visitor_count += 1;
            }
            let mut stream = stream.take();
            let mut buf = [0, ..500];
            stream.read(buf);
            let request_str = str::from_utf8(buf);
            
            let req_group : ~[&str]= request_str.splitn_iter(' ', 3).collect();
            if req_group.len() > 2 {
                let path = req_group[1];
                println(fmt!("Request for path: \n%?", path));
                
                let file_path = &os::getcwd().push(path.replace("/../", ""));
                if !os::path_exists(file_path) || os::path_is_dir(file_path) {
                    println(fmt!("Request received:\n%s", request_str));
                    let response: ~str = fmt!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body { background-color: #111; color: #FFEEAA }
                                h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}
                                h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty!</h1>
                         <h2>Visitor count: %u</h2>
                         </body></html>\r\n", unsafe{visitor_count});

                    stream.write(response.as_bytes());
                }
                else {
                    println(fmt!("serve file: %?", file_path));
                    match io::read_whole_file(file_path) {
                        Ok(file_data) => {
                            stream.write(file_data);
                        }
                        Err(err) => {
                            println(err);
                        }
                    }
                }
            }
            println!("connection terminates")
        }
    }
}
