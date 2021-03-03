use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Response, Server};
use hyper::service::{service_fn, make_service_fn};
use futures::future::{self, Future};
use std::collections::HashMap;

type BoxFut = Box<dyn Future<Item=Response<Body>, Error=hyper::Error> + Send>;

fn blank_response(_: Request<Body>) -> BoxFut {
    let response = Response::new(Body::from(""));
    Box::new(future::ok(response))
}

// Inspired by https://docs.rs/hyper-reverse-proxy/0.4.0/hyper_reverse_proxy/ example
fn main() {
    // This is our socket address...
    let addr = ([0, 0, 0, 0], 80).into();

    // A `Service` is needed for every connection.
    let make_svc = make_service_fn(|socket: &AddrStream| {
        let mut routes = HashMap::new();
        routes.insert("black.allthings.red", "1024");
        routes.insert("allredchristmastraditions.allthings.red", "1025");
        routes.insert("library.allthings.red", "1026");
        routes.insert("lr.allthings.red", "1027");
        routes.insert("pic.allthings.red", "1028");
        let remote_addr = socket.remote_addr();
        service_fn(move |req: Request<Body>| { // returns BoxFut
            if let Some(host_str) = req.uri().host() {
                if let Some(port) = routes.get(host_str) {
                    return hyper_reverse_proxy::call(remote_addr.ip(), format!("http://127.0.0.1:{}", port).as_str(), req)
                }
            };

            if let Some(host) = req.headers().get("Host") {
                if let Ok(host_str) = host.to_str() {
                    if let Some(port) = routes.get(host_str) {
                        return hyper_reverse_proxy::call(remote_addr.ip(), format!("http://127.0.0.1:{}", port).as_str(), req)
                    }
                }
            }

            blank_response(req)
        })
    });

    let server = Server::bind(&addr)
        .serve(make_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Running proxy on {:?}", addr);

    hyper::rt::run(server);
}
