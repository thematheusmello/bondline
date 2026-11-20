//! An ordinary child spawn can overlap listener shutdown before the new image executes.
#![cfg(target_os = "macos")]

#[allow(dead_code)]
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod support;

use bondline_commands::Engine;
use bondline_mcp::control::LoopbackServer;
use nix::{
    fcntl::OFlag,
    spawn::{PosixSpawnAttr, PosixSpawnFileActions, posix_spawn},
    sys::{
        stat::Mode,
        wait::{WaitStatus, waitpid},
    },
    unistd::mkfifo,
};
use rustix::io::{Errno, FdFlags, fcntl_getfd, fcntl_setfd};
use std::{
    ffi::CStr,
    fs::{File, OpenOptions},
    io,
    net::TcpListener,
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

fn kernel_supports_close_on_fork() -> io::Result<bool> {
    let file = File::open("/dev/null")?;
    let flag = FdFlags::from_bits_retain(2);
    match fcntl_setfd(&file, fcntl_getfd(&file)? | flag) {
        Ok(()) => Ok(fcntl_getfd(&file)?.contains(flag)),
        Err(Errno::INVAL | Errno::NOTSUP | Errno::OPNOTSUPP) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn spawn_through_fifo(marker: &Path, gate: &Path, slots: [File; 2]) -> nix::Result<WaitStatus> {
    let mut actions = PosixSpawnFileActions::init()?;
    let owner = Mode::S_IRUSR | Mode::S_IWUSR;
    actions.add_open(
        slots[0].as_raw_fd(),
        marker,
        OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_WRONLY,
        owner,
    )?;
    actions.add_open(slots[1].as_raw_fd(), gate, OFlag::O_RDONLY, Mode::empty())?;
    let args = [c"/usr/bin/true"];
    let environment: [&CStr; 0] = [];
    let child = posix_spawn(
        args[0],
        &actions,
        &PosixSpawnAttr::init()?,
        &args,
        &environment,
    )?;
    waitpid(child, None)
}

#[test]
fn a_spawn_started_after_publication_cannot_hold_the_stopped_listener_on_supported_macos()
-> Result<(), Box<dyn std::error::Error>> {
    if !kernel_supports_close_on_fork()? {
        // Earlier kernels retain the ordinary shutdown and exchange contract in control.rs.
        return Ok(());
    }
    let directory = support::FileTestDirectory::new()?;
    let marker = directory.path("spawn-entered");
    let gate = directory.path("before-exec");
    mkfifo(&gate, Mode::S_IRUSR | Mode::S_IWUSR)?;

    // Reserve the two file-action targets before the listener exists.
    let slots = [File::open("/dev/null")?, File::open("/dev/null")?];
    let server = LoopbackServer::bind(0, Arc::new(Mutex::new(Engine::default())), || {})?;
    let address = server.address();
    let worker = thread::spawn({
        let marker = marker.clone();
        let gate = gate.clone();
        move || spawn_through_fifo(&marker, &gate, slots)
    });

    let deadline = Instant::now() + Duration::from_secs(3);
    while !marker.exists() && !worker.is_finished() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
    }
    let entered = marker.exists();
    // Perform exactly one replacement bind. Keep its result until the child is released.
    let replacement = server
        .shutdown()
        .map_err(io::Error::other)
        .and_then(|()| TcpListener::bind(address));
    let release = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(nix::libc::O_NONBLOCK)
        .open(&gate)?;
    let status = worker.join().map_err(|_| "spawn worker failed")??;
    drop(release);
    assert!(
        entered,
        "child never entered its ordered spawn file actions"
    );
    assert!(matches!(status, WaitStatus::Exited(_, 0)));
    assert_eq!(replacement?.local_addr()?, address);
    Ok(())
}
