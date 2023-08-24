use std::thread::JoinHandle;

pub fn start() -> JoinHandle<Result<(),String>>
{
    let builder = std::thread::Builder::new().name("connection".to_string());
    builder.spawn(|| {
        crate::network::connection::run()
    }).expect("Failed to spawn thread")
}
