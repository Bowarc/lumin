pub fn start_daemon() {
    // start daemon
    debug!("Starting the daemon. . .");

    const DAEMON_EXE_NAME: &str = "lumin_daemon.exe";
    let mut path = std::env::current_exe().unwrap();

    path.set_file_name(DAEMON_EXE_NAME);

    let child = std::process::Command::new(path)
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();

    debug!("{child:?}");

    debug!("The daemon has been started");
}

pub fn is_daemon_running() -> bool {
    use sysinfo::SystemExt as _;

    let s = sysinfo::System::new_all();

    let daemon: Vec<&sysinfo::Process> = s.processes_by_name("lumin_daemon").collect();

    if daemon.len() > 1 {
        error!("Why do we have multiple daemons running ?");
    }

    !daemon.is_empty()
}

pub fn try_connect_to_daemon() -> Option<
    shared::networking::Socket<
        shared::networking::DaemonMessage,
        shared::networking::ClientMessage,
    >,
> {
    // may be better to return a custom socket

    let addr = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(127, 0, 0, 1),
        14045,
    ));

    let stream =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_nanos(50)).ok()?;
    stream.set_nonblocking(true).ok()?;

    Some(shared::networking::Socket::new(stream))
}
