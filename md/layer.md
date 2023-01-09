# Layer 特征解析

## 官方`trait`定义

```rust
pub trait Layer<S> {
    type Service;

    fn layer(&self, inner: S) -> Self::Service;
}
```

## 官方示例

```rust
pub struct LogLayer {
    target: &'static str,
}

impl<S> Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, service: S) -> Self::Service {
        LogService {
            target: self.target,
            service
        }
    }
}

// This service implements the Log behavior
pub struct LogService<S> {
    target: &'static str,
    service: S,
}

impl<S, Request> Service<Request> for LogService<S>
where
    S: Service<Request>,
    Request: fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        // Insert log statement here or other functionality
        println!("request = {:?}, target = {:?}", request, self.target);
        self.service.call(request)
    }
}
```

## 源码分析

官方这个例子和`Service`非常相似。`Layer`特征其实是通过`layer`方法，将实现了`Service`的结构体进行了一层封装。
如果我们用下面的视角来看的话：

```rust
pub struct LogLayer {         layer(service)        pub struct LogService<S> {
    target: &'static str, ----------------------->             target: &'static str,
}                                                              service: S,
                                                    }
```

可以发现`layer`方法充当了类型转换的作用，通过`layer`将`LogLayer`转换为`LogService`，同时继承了参数`service`实现的`Service`特征
