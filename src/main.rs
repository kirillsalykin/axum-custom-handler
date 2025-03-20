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

#[derive(Clone)]
pub struct ApiHandler<F, Extractors, Input, Output> {
    inner: F,
    _marker: PhantomData<(Extractors, Input, Output)>,
}

pub trait IntoApiHandler<Extractors, Input, Output> {
    type Handler;
    fn into_api_handler(self) -> Self::Handler;
}

// provide implementation for cases when there is no `Input`
macro_rules! impl_api_handler {
    ( $( $ty:ident ),* $(,)? ) => {

        impl<F, Fut, $($ty,)* Input, Output> IntoApiHandler<( $($ty,)* ), Input, Output>
        for F
        where
            F: FnOnce( $($ty,)* Input ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = ApiResult<Output>> + Send + 'static,
            Input: DeserializeOwned + Clone + Send + Sync + 'static,
            Output: Serialize + Clone + Send + Sync + 'static,
        {
            type Handler = ApiHandler<F, ( $( $ty, )* ), Input, Output>;
            fn into_api_handler(self) -> Self::Handler {
                ApiHandler {
                    inner: self,
                    _marker: PhantomData,
                }
            }
        }

        #[allow(non_snake_case, unused_mut)]
        impl<F, Fut, S, $($ty,)* Input, Output> Handler<( $($ty,)* ), S>
        for ApiHandler<F, ( $($ty,)* ), Input, Output>
        where
            F: FnOnce( $( $ty, )* Input ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = ApiResult<Output>> + Send + 'static,
            $( $ty: FromRequestParts<S> + Clone + Send + Sync + 'static, )*
            Input: DeserializeOwned + Clone + Send + Sync + 'static,
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

                    let Json(input) = match Json::<Input>::from_request(Request::from_parts(parts, body), state).await {
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

        impl<F, $($ty,)* Input, Output> OperationHandler<ApiHandler<F, ( $($ty,)* ), Input, Output>, ApiHandler<F, ( $($ty,)* ), Input, Output>>
        for ApiHandler<F, ( $($ty,)* ), Input, Output>
        where
            $( $ty: OperationInput, )*
            Input: schemars::JsonSchema,
            Output: schemars::JsonSchema,
        {
        }

        impl<F, $($ty,)* Input, Output> OperationInput
        for ApiHandler<F, ( $($ty,)* ), Input, Output>
        where
            $( $ty: OperationInput, )*
            Input: schemars::JsonSchema,

        {
            fn operation_input( ctx: &mut aide::generate::GenContext, operation: &mut aide::openapi::Operation,) {
                $(
                    $ty::operation_input(ctx, operation);
                )*

               <Json<Input> as OperationInput>::operation_input(ctx, operation);
            }

            fn inferred_early_responses(
                ctx: &mut aide::generate::GenContext,
                operation: &mut aide::openapi::Operation,
            ) -> Vec<(Option<u16>, aide::openapi::Response)> {
                let mut responses = Vec::new();
                $(
                    responses.extend($ty::inferred_early_responses(ctx, operation));
                )*

                 responses.extend(<Json<Input> as OperationInput>::inferred_early_responses(ctx, operation));

                responses
            }
        }

        impl<F, $($ty,)* Input, Output> OperationOutput
        for ApiHandler<F, ( $($ty,)* ), Input, Output>
        where
            Output: schemars::JsonSchema,
        {
            type Inner = Json<Output>;

            fn operation_response(ctx: &mut aide::generate::GenContext, operation: &mut Operation) -> Option<aide::openapi::Response> {
                <Json<Output> as OperationOutput>::operation_response(ctx, operation)
            }

            fn inferred_responses(ctx: &mut aide::generate::GenContext, operation: &mut Operation) -> Vec<(Option<u16>, aide::openapi::Response)> {
                <Json<Output> as OperationOutput>::inferred_responses(ctx, operation)
            }
        }
    }
}

impl_api_handler!();
impl_api_handler!(T1);
impl_api_handler!(T1, T2);

///
///
///
///

async fn handler(h: HeaderMap, input: Input) -> ApiResult<Output> {
    //Err(ApiError::InternalError)
    Ok(Output { field: input.field })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Input {
    field: String,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
struct Output {
    field: String,
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[tokio::main]
async fn main() {
    aide::generate::on_error(|error| {
        println!("{error}");
    });

    let app = ApiRouter::new().api_route("/", post(handler.into_api_handler()));

    let mut api = OpenApi {
        info: Info {
            description: Some("API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    aide::generate::infer_responses(false);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.route("/swagger", get(Swagger::new("/api.json").axum_handler()))
            .route("/api.json", get(serve_api))
            .finish_api(&mut api)
            .layer(Extension(api)),
    )
    .await
    .unwrap();
}
