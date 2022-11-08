#![deny(warnings)]

use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use tokio::net::TcpListener;

async fn hello(req: Request<hyper::body::Incoming>, slab: &mut Vec<Request<hyper::body::Incoming>>) -> Result<Response<Full<Bytes>>, Infallible> {
    slab.push(req);
    if slab.len() > 20000 {
        slab.clear();
    }
    Ok(Response::new(Full::new(Bytes::from_static(b"Hello World!"))))
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    let mut v: Vec<Request<hyper::body::Incoming>> = Vec::with_capacity(20000);
    let ptr = &mut v as *mut _ as usize;

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .http1_writev(false)
                .serve_connection(stream, service_fn(|req| hello(req, unsafe { &mut *(ptr as *mut Vec<_>) })))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
