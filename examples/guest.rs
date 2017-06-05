extern crate ruma_client;
extern crate tokio_core;
extern crate url;

use ruma_client::Client;
use tokio_core::reactor::Core;
use url::Url;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let server = Url::parse("https://matrix.org/").unwrap();
    println!("Create client");
    let mut client = Client::new(&handle, server);
    println!("Client is not conneted: {:?}", client);
    core.run(client.guest_session()).unwrap();
    println!("Client is connected:  {:?}", client);
}