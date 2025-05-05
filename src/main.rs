use libc::*;
use std::io::Result;
use std::io::Write;
use std::net::Ipv4Addr;
use std::os::fd::FromRawFd;
use std::os::fd::RawFd;
use std::slice;
use std::thread;
use std::time::Duration;

mod logger;

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

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    logger::init_logger(true);

    #[cfg(not(debug_assertions))]
    logger::init_logger(false);

    log::info!("Starting...");

    let shutdown_event_fd: RawFd = unsafe { eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK) };

    if shutdown_event_fd == -1 {
        log::error!("eventfd: {}", std::io::Error::last_os_error());
        return Err(std::io::Error::last_os_error());
    }

    let mut shutdown_event_fd_file = unsafe { std::fs::File::from_raw_fd(shutdown_event_fd) };

    let server_handle = thread::spawn(move || {
        let result = unsafe {
            start_server(
                8080,
                shutdown_event_fd,
                handle_new_connection,
                handle_new_data,
                handle_closed_connection,
            )
        };

        if result != 0 {
            log::error!("start_server: {}", std::io::Error::last_os_error());
        }
    });

    thread::sleep(Duration::from_secs(50));

    let signal_value: u64 = 1;
    match shutdown_event_fd_file.write_all(&signal_value.to_ne_bytes()) {
        Ok(_) => log::info!("Shutdown"),
        Err(e) => log::error!("Shutdown error: {}", e),
    }

    let _ = shutdown_event_fd_file.flush();

    match server_handle.join() {
        Ok(_) => log::info!("Done"),
        Err(e) => log::error!("Error: {:?}", e),
    };

    Ok(())
}
