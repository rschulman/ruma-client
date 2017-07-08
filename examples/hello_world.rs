#![feature(try_from)]
#![feature(conservative_impl_trait)]

extern crate futures;
extern crate ruma_client;
extern crate ruma_client_api;
extern crate ruma_events;
extern crate ruma_identifiers;
extern crate tokio_core;
extern crate url;

use std::convert::TryFrom;

use futures::Future;
use ruma_client::Client;
use ruma_client_api::r0::alias::get_alias;
use ruma_client_api::r0::membership::join_room_by_id;
use ruma_client_api::r0::send::send_message_event;
use ruma_client_api::r0::sync::sync_events;
use ruma_events::EventType;
use ruma_events::room::message::{MessageEventContent, MessageType, TextMessageEventContent};
use ruma_identifiers::RoomId;
use tokio_core::reactor::Core;
use url::Url;

fn hello_world<'a>(client: &'a Client) -> impl Future<Item = (), Error = ruma_client::Error> + 'a {
    let msg = MessageEventContent::Text(TextMessageEventContent {
        body: "Hello World!".to_owned(),
        msgtype: MessageType::Text,
    });

    client.request::<get_alias::Endpoint>(get_alias::Request {
        room_alias: RoomAliasId::try_from("#ruma-client-test:matrix.org").unwrap(),
    }).and_then(move |response| {
        let id = response.room_id;

        client.request::<join_room_by_id::Endpoint>(join_room_by_id::Request {
            room_id: id.clone(),
            third_party_signed: None,
        }).and_then(move |_| {
            client.request::<send_message_event::Endpoint>(send_message_event::Request {
                room_id: id,
                event_type: EventType::RoomMessage,
                txn_id: "1".to_owned(),
                data: msg,
            })
        })
    }).map(|_| ())
}

fn run(conn: Client) -> impl Future<Item=(), Error=()> {
    loop {
        let mut since_time: Option<String> = None;
        client.request::<sync_events::Endpoint>(sync_events::Request {
            filter: None,
            since: since_time,
            full_state: None,
            set_presence: None,
            timeout: None, // Should probably set a timeout...
        }).and_then(move |response| {
            since_time = Some(response.next_batch);
            if response.rooms.invite.len() > 0 {
                for room in response.rooms.invite.key() {
                    client.request::<join_room_by_id::Endpoint(join_room_by_id::Request {
                        room_id: RoomId::try_from()
                    })
                }
            }
        })
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let server = Url::parse("https://matrix.org/").unwrap();

    let mut client = Client::new(&handle, server);

    core.run(client.login("northship", "thisisapass"))
    .and_then(run);
    //core.run(client.guest_session()).unwrap();
    //core.run(hello_world(&client)).unwrap();
}
