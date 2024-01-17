use crate::sys;

use crate::api::fs::{FileIO, IO};
use crate::sys::net::SocketStatus;

use super::SOCKETS;
use super::{random_port, wait};

use alloc::vec;
use bit_field::BitField;
use smoltcp::iface::SocketHandle;
use smoltcp::phy::Device;
use smoltcp::socket::tcp;
use smoltcp::wire::IpAddress;

fn tcp_socket_status(socket: &tcp::Socket) -> u8 {
    let mut status = 0;
    status.set_bit(SocketStatus::IsListening as usize, socket.is_listening());
    status.set_bit(SocketStatus::IsActive as usize, socket.is_active());
    status.set_bit(SocketStatus::IsOpen as usize, socket.is_open());
    status.set_bit(SocketStatus::MaySend as usize, socket.may_send());
    status.set_bit(SocketStatus::CanSend as usize, socket.can_send());
    status.set_bit(SocketStatus::MayRecv as usize, socket.may_recv());
    status.set_bit(SocketStatus::CanRecv as usize, socket.can_recv());
    status
}

#[derive(Debug, Clone)]
pub struct TcpSocket {
    pub handle: SocketHandle,
}

impl TcpSocket {
    pub fn size() -> usize {
        if let Some((_, ref mut device)) = *sys::net::NET.lock() {
            let mtu = device.capabilities().max_transmission_unit;
            let eth_header = 14;
            let ip_header = 20;
            let tcp_header = 20;
            mtu - eth_header - ip_header - tcp_header
        } else {
            1
        }
    }

    pub fn new() -> Self {
        let mut sockets = SOCKETS.lock();
        let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
        let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
        let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);
        let handle = sockets.add(tcp_socket);

        Self { handle }
    }

    pub fn connect(&mut self, addr: IpAddress, port: u16) -> Result<(), ()> {
        let mut connecting = false;
        let timeout = 5.0;
        let started = sys::clock::realtime();
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            loop {
                if sys::clock::realtime() - started > timeout {
                    return Err(());
                }
                let mut sockets = SOCKETS.lock();
                iface.poll(sys::net::time(), device, &mut sockets);
                let socket = sockets.get_mut::<tcp::Socket>(self.handle);

                match socket.state() {
                    tcp::State::Closed => {
                        if connecting {
                            return Err(());
                        }
                        let cx = iface.context();
                        let dest = (addr, port);
                        if socket.connect(cx, dest, random_port()).is_err() {
                            return Err(());
                        }
                        connecting = true;
                    }
                    tcp::State::SynSent => {}
                    tcp::State::Established => {
                        break;
                    }
                    _ => {
                        // Did something get sent before the connection closed?
                        return if socket.can_recv() { Ok(()) } else { Err(()) };
                    }
                }

                if let Some(d) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(d);
                }
                sys::time::halt();
            }
        }
        Ok(())
    }

    pub fn listen(&mut self, port: u16) -> Result<(), ()> {
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            let mut sockets = SOCKETS.lock();
            iface.poll(sys::net::time(), device, &mut sockets);
            let socket = sockets.get_mut::<tcp::Socket>(self.handle);

            if socket.listen(port).is_err() {
                return Err(());
            }

            if let Some(d) = iface.poll_delay(sys::net::time(), &sockets) {
                wait(d);
            }
            sys::time::halt();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn accept(&mut self) -> Result<IpAddress, ()> {
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            loop {
                let mut sockets = SOCKETS.lock();
                iface.poll(sys::net::time(), device, &mut sockets);
                let socket = sockets.get_mut::<tcp::Socket>(self.handle);

                if let Some(endpoint) = socket.remote_endpoint() {
                    return Ok(endpoint.addr);
                }

                if let Some(d) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(d);
                }
                sys::time::halt();
            }
        } else {
            Err(())
        }
    }
}

impl FileIO for TcpSocket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let timeout = 5.0;
        let started = sys::clock::realtime();
        let mut bytes = 0;
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            let mut sockets = SOCKETS.lock();
            loop {
                if sys::clock::realtime() - started > timeout {
                    return Err(());
                }
                iface.poll(sys::net::time(), device, &mut sockets);
                let socket = sockets.get_mut::<tcp::Socket>(self.handle);

                if buf.len() == 1 {
                    // 1 byte status read
                    buf[0] = tcp_socket_status(socket);
                    return Ok(1);
                }

                if socket.can_recv() {
                    bytes = socket.recv_slice(buf).map_err(|_| ())?;
                    break;
                }
                if !socket.may_recv() {
                    break;
                }
                if let Some(d) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(d);
                }
                sys::time::halt();
            }
            Ok(bytes)
        } else {
            Err(())
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let timeout = 5.0;
        let started = sys::clock::realtime();
        let mut sent = false;
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            let mut sockets = SOCKETS.lock();
            loop {
                if sys::clock::realtime() - started > timeout {
                    return Err(());
                }
                iface.poll(sys::net::time(), device, &mut sockets);
                let socket = sockets.get_mut::<tcp::Socket>(self.handle);

                if sent {
                    break;
                }
                if socket.can_send() {
                    if socket.send_slice(buf.as_ref()).is_err() {
                        return Err(());
                    }
                    sent = true; // Break after next poll
                }

                if let Some(d) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(d);
                }
                sys::time::halt();
            }
            Ok(buf.len())
        } else {
            Err(())
        }
    }

    fn close(&mut self) {
        let mut closed = false;
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            let mut sockets = SOCKETS.lock();
            loop {
                iface.poll(sys::net::time(), device, &mut sockets);
                let socket = sockets.get_mut::<tcp::Socket>(self.handle);

                if closed {
                    break;
                }
                socket.close();
                closed = true;

                if let Some(d) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(d);
                }
                sys::time::halt();
            }
        }
    }

    fn poll(&mut self, event: IO) -> bool {
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            let mut sockets = SOCKETS.lock();
            iface.poll(sys::net::time(), device, &mut sockets);
            let socket = sockets.get_mut::<tcp::Socket>(self.handle);

            match event {
                IO::Read => socket.can_recv(),
                IO::Write => socket.can_send(),
            }
        } else {
            false
        }
    }
}
