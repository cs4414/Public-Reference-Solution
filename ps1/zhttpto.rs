//
// zhttpto.rs
//
// Reference solution for PS1
//
// Special thanks to Kiet Tran for providing code we incorporated into this.
// 
// Note: it would be very unwise to run this server on a machine that is
// on the Internet and contains any sensitive files!
//
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans
// Version 0.1

extern mod extra;

use extra::uv;
use extra::{net_ip, net_tcp};
use std::{io, os, str};

static BACKLOG: uint = 5;
static PORT:    uint = 4414;
static IPV4_LOOPBACK: &'static str = "127.0.0.1";
static mut visitor_count: uint = 0;

fn new_connection_callback(new_conn :net_tcp::TcpNewConnection, _killch: std::comm::SharedChan<Option<extra::net_tcp::TcpErrData>>)
{
    do spawn {
        let accept_result = extra::net_tcp::accept(new_conn);
        match accept_result {
            Err(err) => {
               println(fmt!("Connection error: %?", err));
            }
            Ok(sock) => {
                let peer_addr: ~str = net_ip::format_addr(&sock.get_peer_addr());
                println(fmt!("Received connection from: %s", peer_addr));
                
                let read_result = net_tcp::read(&sock, 0u);
                match read_result {
                    Err(err) => {
                        println(fmt!("Receive error: %?", err));
                    }
                    Ok(bytes) => {
                        unsafe {
                            visitor_count += 1;
                        }
                        let request_str = str::from_bytes(bytes.slice(0, bytes.len() - 1));
                        
                        let path = request_str.split_iter(' ').nth(1).get();
                        println(fmt!("Request for path: %s", path));
                        
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

                            net_tcp::write(&sock, response.as_bytes_with_null_consume());
                        }
                        else {
                            println(fmt!("serve file: %?", file_path));
                            match io::read_whole_file(file_path) {
                                Ok(file_data) => {
                                    sock.write(file_data);
                                }
                                Err(err) => {
                                    println(err);
                                }
                            }
                        }
                    }
                };
            }
        };
    };
}

fn main() {
    net_tcp::listen(net_ip::v4::parse_addr(IPV4_LOOPBACK), PORT, BACKLOG,
                    &uv::global_loop::get(),
                    |_chan| { println(fmt!("Listening on tcp port %u ...", PORT)); },
                    new_connection_callback);
}
