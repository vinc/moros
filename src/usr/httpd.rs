use crate::{sys, usr};
use crate::api::syscall;
use crate::api::fs;
use crate::api::console::Style;
use alloc::collections::vec_deque::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use smoltcp::socket::{TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::phy::Device;
use time::OffsetDateTime;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    let port = 80;

    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        println!("{}HTTP Server listening on 0.0.0.0:{}{}", csi_color, port, csi_reset);

        let mtu = iface.device().capabilities().max_transmission_unit;
        let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
        let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
        let tcp_handle = iface.add_socket(tcp_socket);

        let mut send_queue: VecDeque<Vec<u8>> = VecDeque::new();
        loop {
            if sys::console::end_of_text() {
                iface.remove_socket(tcp_handle);
                println!();
                return usr::shell::ExitCode::CommandSuccessful;
            }

            let timestamp = Instant::from_micros((syscall::realtime() * 1000000.0) as i64);
            if let Err(e) = iface.poll(timestamp) {
                error!("Network Error: {}", e);
            }

            let socket = iface.get_socket::<TcpSocket>(tcp_handle);

            if !socket.is_open() {
                socket.listen(port).unwrap();
            }
            let addr = socket.remote_endpoint().addr;
            if socket.may_recv() {
                let res = socket.recv(|buffer| {
                    let mut res = String::new();
                    let req = String::from_utf8_lossy(buffer);
                    if !req.is_empty() {
                        let mut verb = "";
                        let mut path = "";
                        let mut header = true;
                        let mut contents = String::new();
                        for (i, line) in req.lines().enumerate() {
                            if i == 0 {
                                let fields: Vec<_> = line.split(' ').collect();
                                if fields.len() >= 2 {
                                    verb = fields[0];
                                    path = fields[1];
                                }
                            } else if header && line.is_empty() {
                                header = false;
                            } else if !header {
                                contents.push_str(&format!("{}\n", line));
                            }
                        }
                        let date = strftime("%d/%b/%Y:%H:%M:%S %z");
                        let code;
                        let mime;
                        let mut body;
                        match verb {
                            "GET" => {
                                if path.len() > 1 && path.ends_with('/') {
                                    code = 301;
                                    res.push_str("HTTP/1.0 301 Moved Permanently\r\n");
                                    res.push_str(&format!("Location: {}\r\n", path.trim_end_matches('/')));
                                    body = "<h1>Moved Permanently</h1>\r\n".to_string();
                                    mime = "text/html";
                                } else if let Ok(contents) = fs::read_to_string(path) {
                                    code = 200;
                                    res.push_str("HTTP/1.0 200 OK\r\n");
                                    body = contents.replace("\n", "\r\n");
                                    mime = "text/plain";
                                } else if let Ok(mut files) = fs::read_dir(path) {
                                    code = 200;
                                    res.push_str("HTTP/1.0 200 OK\r\n");
                                    body = format!("<h1>Index of {}</h1>\r\n", path);
                                    files.sort_by_key(|f| f.name());
                                    for file in files {
                                        let sep = if path == "/" { "" } else { "/" };
                                        let path = format!("{}{}{}", path, sep, file.name());
                                        body.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", path, file.name()));
                                    }
                                    mime = "text/html";
                                } else {
                                    code = 404;
                                    res.push_str("HTTP/1.0 404 Not Found\r\n");
                                    body = "<h1>Not Found</h1>\r\n".to_string();
                                    mime = "text/plain";
                                }
                            },
                            "PUT" => {
                                if path.ends_with('/') { // Write directory
                                    let path = path.trim_end_matches('/');
                                    if fs::exists(path) {
                                        code = 403;
                                        res.push_str("HTTP/1.0 403 Forbidden\r\n");
                                    } else if let Some(handle) = fs::create_dir(path) {
                                        syscall::close(handle);
                                        code = 200;
                                        res.push_str("HTTP/1.0 200 OK\r\n");
                                    } else {
                                        code = 500;
                                        res.push_str("HTTP/1.0 500 Internal Server Error\r\n");
                                    }
                                } else { // Write file
                                    if fs::write(path, contents.as_bytes()).is_ok() {
                                        code = 200;
                                        res.push_str("HTTP/1.0 200 OK\r\n");
                                    } else {
                                        code = 500;
                                        res.push_str("HTTP/1.0 500 Internal Server Error\r\n");
                                    }
                                }
                                body = "".to_string();
                                mime = "text/plain";
                            },
                            "DELETE" => {
                                if fs::exists(path) {
                                    if fs::delete(path).is_ok() {
                                        code = 200;
                                        res.push_str("HTTP/1.0 200 OK\r\n");
                                    } else {
                                        code = 500;
                                        res.push_str("HTTP/1.0 500 Internal Server Error\r\n");
                                    }
                                } else {
                                    code = 404;
                                    res.push_str("HTTP/1.0 404 Not Found\r\n");
                                }
                                body = "".to_string();
                                mime = "text/plain";
                            },
                            _ => {
                                res.push_str("HTTP/1.0 400 Bad Request\r\n");
                                code = 400;
                                body = "<h1>Bad Request</h1>\r\n".to_string();
                                mime = "text/plain";
                            },
                        }
                        let size = body.len();
                        res.push_str(&format!("Server: MOROS/{}\r\n", env!("CARGO_PKG_VERSION")));
                        res.push_str(&format!("Date: {}\r\n", strftime("%a, %d %b %Y %H:%M:%S GMT")));
                        res.push_str(&format!("Content-Type: {}; charset=utf-8\r\n", mime));
                        res.push_str(&format!("Content-Length: {}\r\n", size));
                        res.push_str("Connection: close\r\n");
                        res.push_str("\r\n");
                        res.push_str(&body);
                        println!("{} - - [{}] \"{} {}\" {} {}", addr, date, verb, path, code, size);
                    }
                    (buffer.len(), res)
                }).unwrap();
                for chunk in res.as_bytes().chunks(mtu) {
                    send_queue.push_back(chunk.to_vec());
                }
                if socket.can_send() {
                    if let Some(chunk) = send_queue.pop_front() {
                        socket.send_slice(&chunk).unwrap();
                    }
                }
            } else if socket.may_send() {
                socket.close();
                send_queue.clear();
            }
            if let Some(wait_duration) = iface.poll_delay(timestamp) {
                syscall::sleep((wait_duration.total_micros() as f64) / 1000000.0);
            }
        }
    } else {
        error!("Could not find network interface");
        usr::shell::ExitCode::CommandError
    }
}

fn strftime(format: &str) -> String {
    let timestamp = syscall::realtime();
    OffsetDateTime::from_unix_timestamp(timestamp as i64).format(format)
}
