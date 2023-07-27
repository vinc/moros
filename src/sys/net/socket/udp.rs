use crate::sys;

use crate::api::fs::{FileIO, IO};
use crate::sys::net::SocketStatus;

use alloc::vec;
use bit_field::BitField;
use lazy_static::lazy_static;
use smoltcp::iface::SocketHandle;
use smoltcp::iface::SocketSet;
use smoltcp::socket::udp;
use smoltcp::time::Duration;
use smoltcp::wire::{IpAddress, IpEndpoint, IpListenEndpoint};
use spin::Mutex;

lazy_static! {
    pub static ref SOCKETS: Mutex<SocketSet<'static>> = Mutex::new(SocketSet::new(vec![]));
}

fn udp_socket_status(socket: &udp::Socket) -> u8 {
    let mut status = 0;
    status.set_bit(SocketStatus::IsOpen as usize, socket.is_open());
    status.set_bit(SocketStatus::CanSend as usize, socket.can_send());
    status.set_bit(SocketStatus::CanRecv as usize, socket.can_recv());
    status
}

fn random_port() -> u16 {
    49152 + sys::random::get_u16() % 16384
}

fn wait(duration: Duration) {
    sys::time::sleep((duration.total_micros() as f64) / 1000000.0);
}

#[derive(Debug, Clone)]
pub struct UdpSocket {
    pub handle: SocketHandle,
    pub remote_endpoint: Option<IpEndpoint>,
}

impl UdpSocket {
    pub fn new() -> Self {
        let mut sockets = SOCKETS.lock();
        let udp_rx_buffer = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY], vec![0; 1024]);
        let udp_tx_buffer = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY], vec![0; 1024]);
        let udp_socket = udp::Socket::new(udp_rx_buffer, udp_tx_buffer);
        let handle = sockets.add(udp_socket);
        let remote_endpoint = None;

        Self { handle, remote_endpoint }
    }

    pub fn connect(&mut self, addr: IpAddress, port: u16) -> Result<(), ()> {
        let timeout = 5.0;
        let started = sys::clock::realtime();
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            loop {
                if sys::clock::realtime() - started > timeout {
                    return Err(());
                }
                let mut sockets = SOCKETS.lock();
                iface.poll(sys::net::time(), device, &mut sockets);
                let socket = sockets.get_mut::<udp::Socket>(self.handle);
        
                if !socket.is_open() {
                    let local_endpoint = IpListenEndpoint::from(random_port());
                    socket.bind(local_endpoint).unwrap();
                    break;
                }

                if let Some(duration) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(duration);
                }
                sys::time::halt();
            }
        }
        self.remote_endpoint = Some(IpEndpoint::new(addr, port));
        Ok(())
    }

    pub fn listen(&mut self, _port: u16) -> Result<(), ()> {
        todo!()
    }

    pub fn accept(&mut self) -> Result<IpAddress, ()> {
        todo!()
    }
}

impl FileIO for UdpSocket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let timeout = 5.0;
        let started = sys::clock::realtime();
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            let bytes;
            let mut sockets = SOCKETS.lock();
            loop {
                if sys::clock::realtime() - started > timeout {
                    return Err(());
                }
                iface.poll(sys::net::time(), device, &mut sockets);
                let socket = sockets.get_mut::<udp::Socket>(self.handle);

                if buf.len() == 1 { // 1 byte status read
                    buf[0] = udp_socket_status(socket);
                    return Ok(1);
                }

                if socket.can_recv() {
                    (bytes, _) = socket.recv_slice(buf).map_err(|_| ())?;
                    break;
                }
                if let Some(duration) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(duration);
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
                let socket = sockets.get_mut::<udp::Socket>(self.handle);

                if sent {
                    break;
                }
                if socket.can_send() {
                    if let Some(remote_endpoint) = self.remote_endpoint {
                        if socket.send_slice(buf.as_ref(), remote_endpoint).is_err() {
                            return Err(());
                        }
                    } else {
                        return Err(());
                    }
                    sent = true; // Break after next poll
                }

                if let Some(duration) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(duration);
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
                let socket = sockets.get_mut::<udp::Socket>(self.handle);

                if closed {
                    break;
                }
                socket.close();
                closed = true;

                if let Some(duration) = iface.poll_delay(sys::net::time(), &sockets) {
                    wait(duration);
                }
                sys::time::halt();
            }
        }
    }

    fn poll(&mut self, event: IO) -> bool {
        if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
            let mut sockets = SOCKETS.lock();
            iface.poll(sys::net::time(), device, &mut sockets);
            let socket = sockets.get_mut::<udp::Socket>(self.handle);

            match event {
                IO::Read => socket.can_recv(),
                IO::Write => socket.can_send(),
            }
        } else {
            false
        }
    }
}
