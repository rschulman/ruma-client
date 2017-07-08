//! Crate `ruma_client` is a [Matrix](https://matrix.org/) client library.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![feature(conservative_impl_trait, try_from)]

extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate ruma_api;
pub extern crate ruma_client_api;
extern crate ruma_identifiers;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio_core;
extern crate url;

use std::convert::TryInto;

use futures::{Future, IntoFuture};
use futures::future::FutureFrom;
use hyper::Client as HyperClient;
use hyper_tls::HttpsConnector;
use ruma_api::Endpoint;
use tokio_core::reactor::Handle;
use url::Url;
use url::Host;

pub use error::Error;
pub use session::Session;

mod error;
mod session;

use ruma_client_api::r0::account::register;
use ruma_client_api::r0::session::login;

/// A client for the Matrix client-server API.
#[derive(Debug)]
pub struct Client {
    homeserver_url: Url,
    hyper: HyperClient<HttpsConnector>,
    session: Option<Session>,
}

impl Client {
    /// Creates a new client for making requests to the given homeserver.
    pub fn new(handle: &Handle, homeserver_url: Url) -> Self {
        Client {
            homeserver_url,
            hyper: HyperClient::configure()
                .connector(HttpsConnector::new(/* DNS worker threads: */ 1, &handle))
                .keep_alive(true)
                .build(handle),
            session: None,
        }
    }

    /// Makes a request to a Matrix API endpoint.
    pub fn request<E: Endpoint>(
        &self,
        request: <E as Endpoint>::Request,
    ) -> impl Future<Item = E::Response, Error = Error> {
        let cloned_hyper = self.hyper.clone();
        let mut url = self.homeserver_url.clone();

        request
            .try_into()
            .map_err(Error::from)
            .into_future()
            .and_then(move |mut hyper_request| {
                // Combine homeserver URL from self with path and query params from hyper_request
                // TODO: Rewrite this when Uri supports it directly - https://github.com/hyperium/hyper/issues/1102
                url.set_path(hyper_request.uri().path());
                url.set_query(hyper_request.uri().query());

                // Every valid url is a valid uri
                let uri = url.into_string().parse().unwrap();

                hyper_request.set_uri(uri);
                cloned_hyper.request(hyper_request).map_err(Error::from)
            })
            .and_then(
                |hyper_response| E::Response::future_from(hyper_response).map_err(Error::from),
            )
    }

    /// Logs in as a given user
    pub fn login<'a>(&'a mut self, username: String, password: String) -> impl Future<Item = (), Error = Error> + 'a {
        self.request::<login::Endpoint>(
            login::Request {
                password: password,
                medium: None,
                kind: login::LoginType::Password,
                user: username,
                address: None,
            }
        ).and_then(
            move |response: login::Response| {
                self.session = Some(
                    Session {
                        access_token: response.access_token,
                        homeserver: Host::parse(&response.home_server)?,
                        user_id: response.user_id,
                    }
                );

                Ok(())
            }
        )
    }

    /// Registers as guest
    pub fn guest_session<'a>(&'a mut self) -> impl Future<Item = (), Error = Error> + 'a {
        self.request::<register::Endpoint>(
                register::Request {
                    bind_email: None,
                    password: None,
                    username: None,
                    device_id: None,
                    initial_device_display_name: None,
                    auth: None,
                    kind: Some(register::RegistrationKind::Guest),
                }
            )
            .and_then(
                move |response: register::Response| {
                    self.session = Some(
                        Session {
                            access_token: response.access_token,
                            homeserver: Host::parse(&response.home_server)?,
                            user_id: response.user_id,
                        }
                    );

                    Ok(())
                }
            )
    }
}
