use axum::http::HeaderMap;
use axum::{
    Router,
    extract::{FromRequest, FromRequestParts, Json},
    handler::Handler,
    http::Request,
    response::{IntoResponse, Response},
    routing::post,
};

use serde::{Deserialize, Serialize};
use specta::{Type, ts};
use std::{fmt::Debug, future::Future};
use std::{marker::PhantomData, pin::Pin};

#[derive(Clone)]
pub struct ApiHandler<F, Input, Output> {
    inner: F,
    _marker: PhantomData<(Input, Output)>,
}

pub trait IntoApiHandler<Input, Output> {
    type Handler;
    fn into_api_handler(self) -> Self::Handler;
}

macro_rules! impl_iah {
  ([$($ty:ident),* $(,)?]) => {
        impl<F, Fut, $($ty,)* Output> IntoApiHandler<( $($ty,)* ), Output>
        for F
        where
            F: FnOnce( $($ty,)* ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = ApiResult<Output>> + Send + 'static,
            Output: Serialize + Clone + Send + Sync + 'static,
        {
            type Handler = ApiHandler<F, ( $( $ty, )* ), Output>;

            fn into_api_handler(self) -> Self::Handler {
                ApiHandler {
                    inner: self,
                    _marker: PhantomData,
                }
            }
        }
    }
}

impl_iah!([]);
impl_iah!([T1]);
impl_iah!([T1, T2]);

macro_rules! impl_into_api_handler {
    ([$($ty:ident),* $(,)?]) => {
        impl<F, Fut, $($ty,)* Output> From<F> for ApiHandler<F, ( $($ty,)* ), Output>
        where
            F: FnOnce( $($ty,)* ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = ApiResult<Output>> + Send + 'static,
            Output: Clone + Send + Sync + 'static,
        {
            fn from(f: F) -> Self {
                ApiHandler {
                    inner: f,
                    _marker: PhantomData,
                }
            }
        }
    }
}

impl_into_api_handler!([]);
impl_into_api_handler!([T1]);
impl_into_api_handler!([T1, T2]);

macro_rules! impl_handler {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<F, Fut, S, $($ty,)* $last, Output> Handler<(Output, $($ty,)* $last, ), S>
        for ApiHandler<F, ( $($ty,)* $last,), Output>
        where
            F: FnOnce( $( $ty, )* $last, ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = ApiResult<Output>> + Send,
            S: Send + Sync + 'static,
            $( $ty: FromRequestParts<S> + Clone + Send + Sync + 'static, )*
            $last: FromRequest<S> + Clone + Send + Sync + 'static,
            Output: Serialize + Clone + Send + Sync + 'static,
        {
            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

            fn call(self, req: Request<axum::body::Body>, state: S) -> Self::Future {
                let (mut parts, body) = req.into_parts();

                Box::pin(async move {
                    $(
                        let $ty = match $ty::from_request_parts(&mut parts, &state).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };
                    )*

                    let req = Request::from_parts(parts, body);

                    let $last = match $last::from_request(req, &state).await {
                        Ok(value) => value,
                        Err(rejection) => return rejection.into_response(),
                    };

                    match (self.inner)($($ty,)* $last).await {
                      Ok(output) => Json::<Output>(output).into_response(),
                      Err(err) => err.into_response()
                    }
                })
            }
        }
    }
}

impl_handler!([], T1);
impl_handler!([T1], T2);
impl_handler!([T1, T2], T3);

///
///
///
///

async fn handler(_h: HeaderMap, input: Json<Input>) -> ApiResult<Output> {
    Ok(Output {
        field: "WORKS".into(),
    })
}

async fn empty() -> ApiResult<Output> {
    Ok(Output { field: "".into() })
}

async fn empty1(_h: HeaderMap) -> ApiResult<Output> {
    Ok(Output { field: "".into() })
}

#[derive(Clone)]
enum ApiError {
    InternalError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
struct Input {
    #[specta(inline)]
    field: Email,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
struct Email(String);

#[derive(Clone, Serialize, Deserialize, Type)]
struct Output {
    field: String,
}

pub struct ApiBuilder<S> {
    router: Router<S>,
}

impl<S> ApiBuilder<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            router: Router::<S>::new(),
        }
    }

    pub fn add<H, T: 'static, I, O>(mut self, name: &str, handler: H) -> Self
    where
        H: IntoApiHandler<I, O>,
        H::Handler: Handler<T, S>,
    {
        let config = specta::ts::ExportConfiguration::default();

        let input_schema = specta::ts::export::<Input>(&config).unwrap();
        let output_schema = specta::ts::export::<Output>(&config).unwrap();

        println!("Registering endpoint '{}'", name);
        println!("Input schema: {}", input_schema);
        println!("Output schema: {}", output_schema);

        self.router = self
            .router
            .route(&format!("/{}", name), post(handler.into_api_handler()));
        self
    }

    pub fn build(self) -> Router<S> {
        self.router
    }
}

#[tokio::main]
async fn main() {
    let api = ApiBuilder::new()
        //.add("empty1", empty1)
        .add("handler", handler)
        .build();

    let app = Router::new().merge(api);

    //    //.route("empty", post(empty.into_api_handler()))
    //    //.route("empty1", post(empty1.into()))
    //    //.route("handler", post(handler.into())
    //.route("handler", post(handler.into_api_handler()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
