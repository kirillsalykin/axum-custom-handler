use axum::http::HeaderMap;
use axum::{
    Router,
    extract::{FromRequest, FromRequestParts, Json},
    handler::Handler,
    http::Request,
    response::{IntoResponse, Response},
    routing::post,
};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, future::Future};
use std::{marker::PhantomData, pin::Pin};

#[derive(Clone)]
pub struct ApiHandler<F, Extractors, Input, Output> {
    inner: F,
    _marker: PhantomData<(Extractors, Input, Output)>,
}

pub trait IntoApiHandler<Extractors, Input, Output> {
    type Handler;
    fn into_api_handler(self) -> Self::Handler;
}

impl<F, Fut, Output> IntoApiHandler<(), (), Output> for F
where
    F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = ApiResult<Output>> + Send + 'static,
    Output: Serialize + Clone + Send + Sync + 'static,
{
    type Handler = ApiHandler<F, (), (), Output>;
    fn into_api_handler(self) -> Self::Handler {
        ApiHandler {
            inner: self,
            _marker: PhantomData,
        }
    }
}

impl<F, Fut, S, Output> Handler<(), S> for ApiHandler<F, (), (), Output>
where
    F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = ApiResult<Output>> + Send + 'static,
    Output: Serialize + Clone + Send + Sync + 'static,
    S: Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, _req: Request<axum::body::Body>, _state: S) -> Self::Future {
        Box::pin(async move {
            match (self.inner)().await {
                Ok(output) => Json::<Output>(output).into_response(),
                Err(err) => err.into_response(),
            }
        })
    }
}

macro_rules! impl_api_handler {
    (
        [$($ty:ident),*], $Input:ident
    ) => {
        impl<F, Fut, $($ty,)* $Input, Output> IntoApiHandler<( $($ty,)* ), $Input, Output>
        for F
        where
            F: FnOnce( $($ty,)* $Input ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = ApiResult<Output>> + Send + 'static,
            $Input: DeserializeOwned + Clone + Send + Sync + 'static,
            Output: Serialize + Clone + Send + Sync + 'static,
        {
            type Handler = ApiHandler<F, ( $( $ty, )* ), $Input, Output>;

            fn into_api_handler(self) -> Self::Handler {
                ApiHandler {
                    inner: self,
                    _marker: PhantomData,
                }
            }
        }

        #[allow(non_snake_case, unused_mut)]
        impl<F, Fut, S, $($ty,)* $Input, Output> Handler<( $($ty,)* $Input, ), S>
        for ApiHandler<F, ( $($ty,)* ), $Input, Output>
        where
            F: FnOnce( $( $ty, )* $Input, ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = ApiResult<Output>> + Send + 'static,
            $( $ty: FromRequestParts<S> + Clone + Send + Sync + 'static, )*
            $Input: DeserializeOwned + Clone + Send + Sync + 'static,
            Output: Serialize + Clone + Send + Sync + 'static,
            S: Send + Sync + 'static,
        {
            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

            fn call(self, req: Request<axum::body::Body>, state: S) -> Self::Future {
                Box::pin(async move {
                    let (mut parts, body) = req.into_parts();
                    let state = &state;

                    $(
                        let $ty = match $ty::from_request_parts(&mut parts, state).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };
                    )*

                    let Json(input) = match Json::<$Input>::from_request(Request::from_parts(parts, body), state).await {
                        Ok(value) => value,
                        Err(rejection) => return rejection.into_response(),
                    };

                    match (self.inner)($($ty,)* input).await {
                      Ok(output) => Json::<Output>(output).into_response(),
                      Err(err) => err.into_response()
                    }
                })
            }
        }
    }
}

//impl_api_handler!([], T1);
impl_api_handler!([T1], T2);
impl_api_handler!([T1, T2], T3);

///
///
///
///

async fn handler(h: HeaderMap, input: Input) -> ApiResult<Output> {
    //Err(ApiError::InternalError)
    Ok(Output { field: input.field })
}

async fn empty() -> ApiResult<Output> {
    //Err(ApiError::InternalError)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Input {
    field: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Output {
    field: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", post(handler.into_api_handler()))
        .route("/", post(empty.into_api_handler()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
