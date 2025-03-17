use axum::{
    extract::{FromRequest, Json},
    handler::Handler,
    http::Request,
    response::{IntoResponse, Response},
};
use axum::{routing::post, Router};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::{marker::PhantomData, pin::Pin};

#[derive(Clone)]
pub struct ApiHandler<F, Req, Res> {
    inner: F,
    _marker: PhantomData<(Req, Res)>,
}

impl<F, Req, Res> ApiHandler<F, Req, Res> {
    pub fn new(inner: F) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<F, Fut, Req, Res, S> Handler<Request<axum::body::Body>, S> for ApiHandler<F, Req, Res>
where
    F: FnOnce(Req) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    Req: DeserializeOwned + Clone + Send + Sync + 'static,
    Res: Serialize + Clone + Send + Sync + 'static,
    S: Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: Request<axum::body::Body>, state: S) -> Self::Future {
        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let state = &state;

            let Json(req) =
                match Json::<Req>::from_request(Request::from_parts(parts, body), state).await {
                    Ok(value) => value,
                    Err(rejection) => return rejection.into_response(),
                };

            let res: Res = (self.inner)(req).await;
            return axum::response::Json(res).into_response();
        })
    }
}

#[axum::debug_handler]`
pub async fn handler(_input: Input) -> Output {
    Output {
        field: "bar".to_string(),
    }
}

#[derive(Serialize, Deserialize)]
struct Input {
    field: String,
}

#[derive(Serialize, Deserialize)]
struct Output {
    field: String,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/", post(ApiHandler::new(handler)));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
