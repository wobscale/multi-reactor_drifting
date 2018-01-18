use rocket;
use super::*;
use rocket::local::Client;
use futures;

#[get("/")]
fn future_string() -> Future<String, ()> {
    Future(Box::new(futures::future::ok("hello world".to_owned())))
}

#[test]
fn it_works() {
    let rkt = rocket::ignite().mount("/", routes![future_string]);
    let c = Client::new(rkt).unwrap();

    let mut resp = c.get("/").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    assert_eq!(resp.body_string(), Some("hello world".to_owned()));
}

#[get("/")]
fn mid_request_future() -> String {
    let s = future_string();

    run(s).unwrap()
}

#[test]
fn request_core_run_works() {
    let rkt = rocket::ignite().mount("/", routes![mid_request_future]);
    let c = Client::new(rkt).unwrap();

    let mut resp = c.get("/").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    assert_eq!(resp.body_string(), Some("hello world".to_owned()));
}
