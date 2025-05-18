use std::{
    io::{Result, Write},
    os::fd::{FromRawFd, RawFd},
    sync::mpsc::{self, Receiver, Sender},
};

use libc::{EFD_CLOEXEC, EFD_NONBLOCK, eventfd};

mod internal;
mod logger;

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    logger::init_logger(true);

    #[cfg(not(debug_assertions))]
    logger::init_logger(false);

    let (sender, receiver): (Sender<()>, Receiver<()>) = mpsc::channel();

    ctrlc::set_handler(move || {
        sender.send(()).expect("MSPC send failed");
    })
    .expect("Error ctrlc set_handler");

    let shutdown_event_fd: RawFd = unsafe { eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK) };

    let server_handle = internal::init_server(8080, shutdown_event_fd);

    receiver.recv().expect("Failed to receive signal");

    if shutdown_event_fd == -1 {
        log::error!("eventfd: {}", std::io::Error::last_os_error());
        return Err(std::io::Error::last_os_error());
    }

    let mut shutdown_event_fd_file = unsafe { std::fs::File::from_raw_fd(shutdown_event_fd) };

    let signal_value: u64 = 1;
    shutdown_event_fd_file
        .write_all(&signal_value.to_ne_bytes())
        .expect("Failed to write to shutdown_event_fd");

    let _ = shutdown_event_fd_file.flush();

    server_handle.join().expect("Failed to join server thread");

    log::info!("Bye");

    Ok(())
}
