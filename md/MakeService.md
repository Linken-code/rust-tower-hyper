# Layer 特征解析

## 官方`trait`定义

```rust
pub trait MakeService<Target, Request>: Sealed<(Target, Request)> {
    type Response;
    type Error;
    type Service: Service<Request, Response = Self::Response, Error = Self::Error>;
    type MakeError;
    type Future: Future<Output = Result<Self::Service, Self::MakeError>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>
    ) -> Poll<Result<(), Self::MakeError>>;
    fn make_service(&mut self, target: Target) -> Self::Future;

    fn into_service(self) -> IntoService<Self, Request>
    where
        Self: Sized,
    { ... }

    fn as_service(&mut self) -> AsService<'_, Self, Request>
    where
        Self: Sized,
    { ... }
}

impl<M, S, Target, Request> MakeService<Target, Request> for M
where
    M: Service<Target, Response = S>,
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Service = S;
    type MakeError = M::Error;
    type Future = M::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::MakeError>> {
        Service::poll_ready(self, cx)
    }

    fn make_service(&mut self, target: Target) -> Self::Future {
        Service::call(self, target)
    }
}

/// [into]: MakeService::into_service
pub struct IntoService<M, Request> {
    make: M,
    _marker: PhantomData<Request>,
}
impl<M, S, Target, Request> Service<Target> for IntoService<M, Request>
where
    M: Service<Target, Response = S>,
    S: Service<Request>,
{
    type Response = M::Response;
    type Error = M::Error;
    type Future = M::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.make.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, target: Target) -> Self::Future {
        self.make.make_service(target)
    }
}

/// [as]: MakeService::as_service
pub struct AsService<'a, M, Request> {
    make: &'a mut M,
    _marker: PhantomData<Request>,
}
impl<M, S, Target, Request> Service<Target> for AsService<'_, M, Request>
where
    M: Service<Target, Response = S>,
    S: Service<Request>,
{
    type Response = M::Response;
    type Error = M::Error;
    type Future = M::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.make.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, target: Target) -> Self::Future {
        self.make.make_service(target)
    }
}

```

## 官方示例

### into_service 示例

```rust
use std::convert::Infallible;
use tower::Service;
use tower::make::MakeService;
use tower::service_fn;

// A `MakeService`
let make_service = service_fn(|make_req: ()| async {
    Ok::<_, Infallible>(service_fn(|req: String| async {
        Ok::<_, Infallible>(req)
    }))
});

// Convert the `MakeService` into a `Service`
let mut svc = make_service.into_service();

// Make a new service
let mut new_svc = svc.call(()).await.unwrap();

// Call the service
let res = new_svc.call("foo".to_string()).await.unwrap();
```

### as_service 示例

```rust
use std::convert::Infallible;
use tower::Service;
use tower::make::MakeService;
use tower::service_fn;

// A `MakeService`
let mut make_service = service_fn(|make_req: ()| async {
    Ok::<_, Infallible>(service_fn(|req: String| async {
        Ok::<_, Infallible>(req)
    }))
});

// Convert the `MakeService` into a `Service`
let mut svc = make_service.as_service();

// Make a new service
let mut new_svc = svc.call(()).await.unwrap();

// Call the service
let res = new_svc.call("foo".to_string()).await.unwrap();

// The original `MakeService` is still accessible
let new_svc = make_service.make_service(()).await.unwrap();
```

## service_fn

```rust
pub fn service_fn<T>(f: T) -> ServiceFn<T> {
    ServiceFn { f }
}

/// A [`Service`] implemented by a closure.
///
/// See [`service_fn`] for more details.
#[derive(Copy, Clone)]
pub struct ServiceFn<T> {
    f: T,
}

impl<T, F, Request, R, E> Service<Request> for ServiceFn<T>
where
    T: FnMut(Request) -> F,
    F: Future<Output = Result<R, E>>,
{
    type Response = R;
    type Error = E;
    type Future = F;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), E>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request) -> Self::Future {
        (self.f)(req)
    }
}
```

## 源码分析

结合`service_fn`函数，我们可以发现经过`service_fn`的层层套娃，`MakeService`其实是个总包。`into_service`和`as_service`其实是所有权的归属差异，我们以`into_service`为例，观察一下整个结构：

```rust
IntoService<M, Request> {
    make: ServiceFn<T> {
          f: ServiceFn<T> {
                    f:fn(req)->(req),
          		      call(&mut self, req: Request) -> Self::Future {
                          (self.f)(req)
                    },
                 },
           call(&mut self, req: Request) -> Self::Future {
                  (self.f)(req)
            }
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::MakeError>> {
        Service::poll_ready(self, cx)
    }

    fn make_service(&mut self, target: Target) -> Self::Future {
        Service::call(self, target)
    }
    },
    call(&mut self, target: Target) -> Self::Future {
        self.make.make_service(target)
    }
}

```

从上面的结构可以清晰地看到，函数是如何层层调用的。`IntoService::call`函数里调用下一层的`make.make_service`，而`make.make_service`又是调用`Service`特征的`Service::call`即`ServiceFn.call()`，经过层层传递最终触发最后一层的`fn`。
