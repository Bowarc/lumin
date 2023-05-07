use windres::Build;

fn main() {
    Build::new().compile("resources/tray-example.rc").unwrap();
}
