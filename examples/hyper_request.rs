#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate futures;
extern crate hyper;
extern crate multi_reactor_drifting;
extern crate rocket;

use hyper::client::Client;
use rocket::response::content;
use futures::future::Future;
use futures::Stream;

// This example mirrors "example.com" locally. 'example.com' was chosen so we don't have to bother
// with hyper-tls to get this to work, and to show returning a future.
#[get("/", format = "text/html")]
fn mirror_example(
    handle: multi_reactor_drifting::Handle,
) -> content::Html<multi_reactor_drifting::Future<String, hyper::Error>> {
    let http_client = Client::new(&handle.into());

    let body_future = http_client
        .get("http://example.com".parse().unwrap())
        .and_then(|resp| {
            resp.body()
                .concat2()
                .map(|chunk| String::from_utf8_lossy(&chunk.to_vec()).to_string())
        });

    content::Html(multi_reactor_drifting::Future(Box::new(body_future)))
}

fn main() {
    rocket::ignite().mount("/", routes![mirror_example]).launch();
}
