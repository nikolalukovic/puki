use libc::*;
use std::io::Result;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::os::fd::RawFd;
use std::thread;
use std::time::Duration;

#[link(name = "server", kind = "static")]
unsafe extern "C" {
    fn start_server(port: c_int, shutdown_event_fd: c_int) -> c_int;
}

fn main() -> Result<()> {
    println!("Starting");
    let shutdown_event_fd: RawFd = unsafe { eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK) };

    if shutdown_event_fd == -1 {
        eprintln!("eventfd: {}", std::io::Error::last_os_error());
        return Err(std::io::Error::last_os_error());
    }

    let mut shutdown_event_fd_file = unsafe { std::fs::File::from_raw_fd(shutdown_event_fd) };

    let server_handle = thread::spawn(move || {
        let result = unsafe { start_server(8080, shutdown_event_fd) };

        if result != 0 {
            eprintln!("start_server: {}", std::io::Error::last_os_error());
        }
    });

    thread::sleep(Duration::from_secs(50));

    let signal_value: u64 = 1;
    match shutdown_event_fd_file.write_all(&signal_value.to_ne_bytes()) {
        Ok(_) => println!("Shutdown"),
        Err(e) => eprintln!("Shutdown error: {}", e),
    }

    let _ = shutdown_event_fd_file.flush();

    match server_handle.join() {
        Ok(_) => println!("Done"),
        Err(e) => eprintln!("Error: {:?}", e),
    };

    Ok(())
}
