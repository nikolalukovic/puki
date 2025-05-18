use std::{
    net::Ipv4Addr, os::fd::RawFd, slice, thread::{self, JoinHandle}
};

use libc::{c_int, size_t};

#[link(name = "pk_server", kind = "static")]
unsafe extern "C" {
    fn start_server(
        port: c_int,
        shutdown_event_fd: c_int,
        new_conn_cb: extern "C" fn(fd: c_int, ip: u32, port: u16),
        data_cb: extern "C" fn(fd: c_int, data: *const u8, len: size_t),
        close_cb: extern "C" fn(fd: c_int),
    ) -> c_int;
}

#[unsafe(no_mangle)]
extern "C" fn handle_new_connection(fd: c_int, ip_addr_net: u32, port_net: u16) {
    let ip_addr = Ipv4Addr::from(u32::from_be(ip_addr_net));
    let port = u16::from_be(port_net);

    log::info!("fd={}, addr={}:{}", fd, ip_addr, port);
}

#[unsafe(no_mangle)]
extern "C" fn handle_new_data(fd: c_int, data_ptr: *const u8, len: size_t) {
    let data_slices = unsafe { slice::from_raw_parts(data_ptr, len) };

    let data_str = String::from_utf8_lossy(data_slices);

    log::info!("Received data: fd={}, data={}", fd, data_str);
}

#[unsafe(no_mangle)]
extern "C" fn handle_closed_connection(fd: c_int) {
    log::info!("connection closed: fd={}", fd);
}

pub fn init_server(port: i32, shutdown_event_fd: RawFd) -> JoinHandle<()> {
    thread::spawn(move || {
        let result = unsafe {
            start_server(
                port,
                shutdown_event_fd,
                handle_new_connection,
                handle_new_data,
                handle_closed_connection,
            )
        };

        if result != 0 {
            log::error!("start_server: {}", std::io::Error::last_os_error());
        }
    })
}
