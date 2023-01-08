# Service 特征解析

## 官方`trait`定义

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future
    where
        <Self::Future as Future>::Output == Result<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>
    ) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

## 官方示例

```rust
use http::{Request, Response, StatusCode};
use std::task::{Context, Poll};
use std::pin::Pin;

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
        let body: Vec<u8> = "hello, world!\n"
            .as_bytes()
            .to_owned();
        // Create the HTTP response
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");

        // create a response in a future.
        let fut = async {
            Ok(resp)
        };

        // Return the response as an immediate future
        Box::pin(fut)
    }
}
```

## 源码分析

`Service` 是对 `request-response` 模式的抽象。 `request-response` 模式是非常强大的，很多问题都可以用这个模式来表达。更进一步，其实任何函数都可视为 `request/response`，函数参数即 `request`，返回值即 `response`。

`poll_ready()` 用于探测 `service` 的状态，是否正常工作，是否过载等。只有当 `poll_ready()` 返回 `Poll::Ready(Ok(()))` 时，才可以调用 `call()` 处理请求。

`call()` 则是真正处理请求的地方，它返回一个 `future`，因此相当于 `async fn(Request) -> Result<Response, Error>` 。

所以这个`Service`特征，本质上是为目标结构体实现了`async fn(Request) -> Result<Response, Error>`的异步方法。入参为结构体自身以及外部的`Request`
