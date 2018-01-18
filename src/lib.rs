#![cfg_attr(test, feature(plugin, custom_derive))]
#![cfg_attr(test, plugin(rocket_codegen))]

#[cfg(test)]
mod tests;

extern crate futures;
extern crate rocket;
extern crate tokio_core;

use tokio_core::reactor::Core;
use rocket::request::Outcome as ReqOutcome;
use rocket::Outcome;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::http::Status;
use std::sync::Mutex;

///
/// The Handle struct may be used to get access to a tokio core handle for this request.
/// A future for this handle may be returned from the request handler, at which point it will be
/// resolved.
/// A future derived from this handle may also be run using the `request_core_run` method.
///
/// Handle implements 'into' for a tokio_core Handle to allow for easy conversion.
pub struct Handle {
    handle: tokio_core::reactor::Handle,
}

// Cores cannot be sent between threads safely, so we instantiate one per thread.
// Since 'running' a reactor core requires mutable ownership, we wrap it in a mutex.
thread_local! {
    static CORE: Mutex<Core> = Mutex::new(Core::new().unwrap());
}

///
/// Run a future to completion. This will block the handling of the current request until the
/// future resolves.
pub fn run<F>(f: F) -> Result<F::Item, F::Error>
where
    F: futures::future::Future,
{
    CORE.with(|c| c.lock().unwrap().run(f))
}

impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for Handle {
    type Error = ();

    fn from_request(_: &rocket::Request) -> ReqOutcome<Self, Self::Error> {
        let handle = CORE.with(|f| f.lock().unwrap().handle());

        Outcome::Success(Handle { handle: handle })
    }
}

impl From<Handle> for tokio_core::reactor::Handle {
    fn from(h: Handle) -> tokio_core::reactor::Handle {
        h.handle
    }
}

/// Future is a thin wrapper for a futures future. This type implements the Responder trait,
/// which allows it to be returned from a rocket request handler.
pub struct Future<I, E>(pub Box<futures::future::Future<Item = I, Error = E>>);
// TODO: there must be a better way than boxing it up

impl<I, E> futures::future::Future for Future<I, E> {
    type Item = I;
    type Error = E;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

impl<'r, I, E> rocket::response::Responder<'r> for Future<I, E>
where
    Result<I, E>: Responder<'r>,
{
    fn respond_to(self, request: &Request) -> Result<Response<'r>, Status> {
        CORE.with(|core_m| {
            let mut c = core_m.lock().unwrap();
            c.run(self.0).respond_to(request)
        })
    }
}

impl<I, E> From<Box<futures::future::Future<Item = I, Error = E>>> for Future<I, E> {
    fn from(f: Box<futures::future::Future<Item = I, Error = E>>) -> Self {
        Future(f)
    }
}
