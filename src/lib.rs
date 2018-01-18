extern crate rocket;
extern crate tokio_core;
extern crate futures;

use tokio_core::reactor::Core;
use rocket::request::Outcome as ReqOutcome;
use rocket::Outcome;
use rocket::request::Request;
use rocket::response::{Response, Responder};
use rocket::http::Status;
use std::sync::Mutex;

pub struct Handle {
    handle: tokio_core::reactor::Handle,
}

thread_local! {
    static CORE: Mutex<Core> = Mutex::new(Core::new().unwrap());
}

pub fn request_core_run<F>(f: F) -> Result<F::Item, F::Error> 
    where F: futures::future::Future,
{
    CORE.with(|c| {
        c.lock().unwrap().run(f)
    })
}

impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for Handle {
    type Error = ();

    fn from_request(_: &rocket::Request) -> ReqOutcome<Self, Self::Error> {
        let handle = CORE.with(|f| {
            f.lock().unwrap().handle()
        });

        Outcome::Success(Handle{handle: handle})
    }
}


impl From<Handle> for tokio_core::reactor::Handle {
    fn from(h: Handle) -> tokio_core::reactor::Handle {
        h.handle
    }
}

pub struct Future<I, E>(pub Box<futures::future::Future<Item = I, Error = E>>);

impl<I, E> futures::future::Future for Future<I, E> {
    type Item = I;
    type Error = E;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

impl<'r, I, E> rocket::response::Responder<'r> for Future<I, E>
    where Result<I, E>: Responder<'r>,
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
