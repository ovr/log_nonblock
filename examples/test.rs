use rust_nonblocking_logger::NonBlockingLogger;

fn main() {
    NonBlockingLogger::new().init().unwrap();

    log::warn!("This is an example message.");

    let large_string = "a".repeat(10000000);
    log::warn!("{}", large_string);

    log::warn!("This is an example message.");
}
