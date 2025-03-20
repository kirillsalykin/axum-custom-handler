use aide::IntoApi;
use aide::swagger::Swagger;
use aide::{
    axum::{
        ApiRouter, IntoApiResponse,
        routing::{get, post},
    },
    openapi::{Info, OpenApi, Operation},
    operation::{OperationHandler, OperationInput, OperationOutput},
};

use axum::http::HeaderMap;
use axum::{
    Extension,
    extract::{FromRequest, FromRequestParts, Json},
    handler::Handler,
    http::Request,
    response::{IntoResponse, Response},
};
use schemars::{self, JsonSchema};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, future::Future};
use std::{marker::PhantomData, pin::Pin};

//#[derive(Clone)]
//pub struct ApiHandler<F, Req, Res> {
//    inner: F,
//    _marker: PhantomData<(Req, Res)>,
//}
//
//impl<F, Req, Res> ApiHandler<F, Req, Res> {
//    pub fn new(inner: F) -> Self {
//        Self {
//            inner,
//            _marker: PhantomData,
//        }
//    }
//}
//
//// TODO: empty implementation for no `Res`
//macro_rules! impl_api_handler {
//    (
//        [$($ty:ident),* $(,)?]
//    ) => {
//        #[allow(non_snake_case, unused_mut)]
//        impl<F, Fut, S, Res, $( $ty, )* Req> Handler<($($ty,)* Json<Req>,), S>
//        for ApiHandler<F, Req, Res>
//        where
//            F: FnOnce( $( $ty, )* Req ) -> Fut + Clone + Send + Sync + 'static,
//            Fut: Future<Output = ApiResult<Res>> + Send + 'static,
//            $( $ty: FromRequestParts<S> + Send, )*
//            Req: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
//            Res: Serialize + Clone + Send + Sync + 'static,
//            S: Send + Sync + 'static,
//        {
//            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;
//
//            fn call(self, req: Request<axum::body::Body>, state: S) -> Self::Future {
//                Box::pin(async move {
//                    let (mut parts, body) = req.into_parts();
//                    let state = &state;
//
//                    $(
//                        let $ty = match $ty::from_request_parts(&mut parts, state).await {
//                            Ok(value) => value,
//                            Err(rejection) => return rejection.into_response(),
//                        };
//                    )*
//
//                    let Json(req) = match Json::<Req>::from_request(Request::from_parts(parts, body), state).await {
//                        Ok(value) => value,
//                        Err(rejection) => return rejection.into_response(),
//                    };
//
//                    match (self.inner)($($ty,)* req).await {
//                      Ok(res) => Json(res).into_response(),
//                      Err(err) => err.into_response()
//                    }
//                })
//            }
//        }
//
//         #[allow(non_snake_case)]
//        impl<Ret, F, Req, Res, $($ty,)*> OperationHandler<($($ty,)*), Ret::Output>
//        for ApiHandler<F, Req, Res>
//        where
//            F: FnOnce($($ty,)*) -> Ret,
//            Ret: std::future::Future,
//            Ret::Output: OperationOutput,
//            $( $ty: OperationInput, )*
//        {}
//    };
//}
//
//impl_api_handler!([]);
//impl_api_handler!([T1]);
//impl_api_handler!([T1, T2]);
//
////impl<F, Req, Res> OperationHandler<ApiHandler<F, Req, Res>, ApiHandler<F, Req, Res>>
////    for ApiHandler<F, Req, Res>
////where
////    Req: schemars::JsonSchema,
////    Res: schemars::JsonSchema,
////{
////}
////
////impl<F, Req, Res> OperationInput for ApiHandler<F, Req, Res>
////where
////    Req: schemars::JsonSchema,
////{
////    fn operation_input(
////        ctx: &mut aide::generate::GenContext,
////        operation: &mut aide::openapi::Operation,
////    ) {
////        <<Self as WithExtractors>::Extractors as OperationInput>::operation_input(ctx, operation);
////    }
////}
////
////impl<F, Req, Res> OperationOutput for ApiHandler<F, Req, Res>
////where
////    Res: schemars::JsonSchema,
////{
////    type Inner = Json<Res>;
////
////    fn operation_response(
////        ctx: &mut aide::generate::GenContext,
////        operation: &mut Operation,
////    ) -> Option<aide::openapi::Response> {
////        let r = <axum::extract::Json<Res> as OperationOutput>::operation_response(ctx, operation);
////        println!("operation_response: {:?}", r);
////        r
////    }
////
////    fn inferred_responses(
////        ctx: &mut aide::generate::GenContext,
////        operation: &mut Operation,
////    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
////        let r = <axum::extract::Json<Res> as OperationOutput>::inferred_responses(ctx, operation);
////        println!("inferred_responses: {:?}", r);
////        r
////    }
////}
//
//// ====
//
//#[derive(Clone)]
//enum ApiError {
//    InternalError,
//}
//
//type ApiResult<T> = Result<T, ApiError>;
//
//impl IntoResponse for ApiError {
//    fn into_response(self) -> axum::response::Response {
//        (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response()
//    }
//}
//
//async fn handler(h: HeaderMap, input: Input) -> ApiResult<Output> {
//    //Err(ApiError::InternalError)
//    Ok(Output { field: input.field })
//}
//
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Input {
    field: String,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
struct Output {
    field: String,
}
//
//async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
//    Json(api)
//}
//
//#[tokio::main]
//async fn main() {
//    let app = ApiRouter::new().route("/", post(ApiHandler::new(handler)));
//
//    let mut api = OpenApi {
//        info: Info {
//            description: Some("API".to_string()),
//            ..Info::default()
//        },
//        ..OpenApi::default()
//    };
//
//    aide::generate::infer_responses(false);
//
//    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//        .await
//        .unwrap();
//    println!("listening on {}", listener.local_addr().unwrap());
//    axum::serve(
//        listener,
//        app.route("/swagger", get(Swagger::new("/api.json").axum_handler()))
//            .route("/api.json", get(serve_api))
//            .finish_api(&mut api)
//            .layer(Extension(api)),
//    )
//    .await
//    .unwrap();
//}

pub struct S<F, Params> {
    inner: F,
    _marker: PhantomData<Params>,
}

use std::any::type_name;
use std::fmt;

// Custom Debug implementation that prints the type name of Params.
impl<F, Params> fmt::Debug for S<F, Params>
where
    Params: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S")
            .field("Params", &type_name::<Params>())
            .finish()
    }
}

pub trait IntoApiHandler<Params> {
    type Handler;
    fn into_api_handler(self) -> Self::Handler;
}

macro_rules! impl_api_handler {
    ( $( $ty:ident ),* $(,)? ) => {
        impl<F, Fut, $( $ty, )* Req, Res> IntoApiHandler<($( $ty, )* Req,)> for F
        where
            F: FnOnce( $( $ty, )* Req ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Res> + Send + 'static,
            Req: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
            Res: Serialize + Clone + Send + Sync + 'static,
        {
            type Handler = S<F, ( $( $ty, )* Req, )>;
            fn into_api_handler(self) -> Self::Handler {
                S {
                    inner: self,
                    _marker: PhantomData,
                }
            }
        }
    }
}

impl_api_handler!();
impl_api_handler!(T1);
impl_api_handler!(T1, T2);

async fn f1(input: Input) -> Output {
    Output { field: input.field }
}

async fn f2(_h: HeaderMap, input: Input) -> Output {
    Output { field: input.field }
}

#[tokio::main]
async fn main() {
    let s1 = f1.into_api_handler();
    println!("S1: {:?}", s1);

    let s2 = f2.into_api_handler();
    println!("S2: {:?}", s2);
}
