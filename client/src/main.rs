mod ui;

fn main() {
    // check_daemon();

    // let stream = connect_to_daemon();

    ui::run()
}

fn check_daemon() {
    // make sure that the daemon is running
    todo!()
}
fn connect_to_daemon() -> std::net::TcpStream {
    // may be better to return a custom socket
    todo!()
}
