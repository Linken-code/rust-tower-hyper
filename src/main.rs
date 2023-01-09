use http::{Request, Response, StatusCode};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::make::MakeService;
use tower::Service;
use tower::{service_fn, BoxError, Service, ServiceExt};

fn main() {
    println!("Hello, world!");
    // A `MakeService`
    let make_service = service_fn(|make_req: ()| async {
        Ok::<_, Infallible>(service_fn(|req: String| async { Ok::<_, Infallible>(req) }))
    });

    // Convert the `MakeService` into a `Service`
    let mut svc = make_service.into_service();

    // Make a new service
    let mut new_svc = svc.call(()).await.unwrap();

    // Call the service
    let res = new_svc.call("foo".to_string()).await.unwrap();
}

struct HelloWorld;

impl Service<Request<Vec<u8>>> for HelloWorld {
    type Response = Response<Vec<u8>>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Vec<u8>>) -> Self::Future {
        // create the body
        let body: Vec<u8> = "hello, world!\n".as_bytes().to_owned();
        // Create the HTTP response
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");

        // create a response in a future.
        let fut = async { Ok(resp) };

        // Return the response as an immediate future
        Box::pin(fut)
    }
}
