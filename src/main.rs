use axum::{
    Router,
    body::Body,
    extract::{FromRequest, FromRequestParts, Json},
    handler::Handler,
    http::Request,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, from_value};
use specta::{NamedType, Type};
use std::{marker::PhantomData, pin::Pin};

#[derive(Clone)]
pub struct Procedure<F, Extractors, Input, Output, Error> {
    f: F,
    _marker: PhantomData<(Extractors, Input, Output, Error)>,
}

pub trait IntoProcedure<Extractors, Input, Output, Error> {
    type Procedure;
    fn into_procedure(self) -> Self::Procedure;
}

macro_rules! impl_procedure {
  ([$($ty:ident),* $(,)?] ) => {
        impl<F, Fut, $($ty,)* Input, Output, Error> IntoProcedure<( $($ty,)* ), Input, Output, Error>
        for F
        where
            F: FnOnce( $($ty,)* Input ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<Output, Error>> + Send + 'static,
            Input: Type + DeserializeOwned + Clone + Send + Sync + 'static,
            Output: Type + Serialize + Clone + Send + Sync + 'static,
            Error: Type + Clone + Send + Sync + 'static,
        {
            type Procedure = Procedure<F, ( $( $ty, )* ), Input, Output, Error>;

            fn into_procedure(self) -> Self::Procedure {
                Procedure {
                    f: self,
                    _marker: PhantomData,
                }
            }
        }

        impl<F, Fut, S, $($ty,)* Input, Output, Error> Handler< (Input, $($ty,)* Output), S>
        for Procedure<F, ( $($ty,)* ), Input, Output, Error>
        where
            F: FnOnce( $( $ty, )* Input ) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<Output, Error>> + Send,
            S: Send + Sync + 'static,
            $( $ty: FromRequestParts<S> + Clone + Send + Sync + 'static, )*
            Input: Type + DeserializeOwned + Clone + Send + Sync + 'static,
            Output: Type + Serialize + Clone + Send + Sync + 'static,
            Error: Type + Serialize + Clone + Send + Sync + 'static,
        {
            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

            fn call(self, req: Request<Body>, state: S) -> Self::Future {
                let (mut parts, body) = req.into_parts();

                Box::pin(async move {
                    $(
                        let $ty = match $ty::from_request_parts(&mut parts, &state).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };
                    )*

                    let req = Request::from_parts(parts, body);

                    let input_value = match Json::<Value>::from_request(req, &state).await {
                        Ok(value) => value.0,
                        Err(rejection) => return rejection.into_response(),
                    };

                    let input: Input = match from_value(input_value) {
                        Ok(input) => input,
                        Err(err) => {
                            println!("BAD_JSON: {:?}", err);
                            return "json_bad".into_response()
                        },
                    };

                    match (self.f)($($ty,)* input).await {
                      Ok(output) => Json::<Output>(output).into_response(),
                      Err(err) => "error".into_response()
                    }
                })
            }
        }
    }
}

impl_procedure!([]);
impl_procedure!([T1]);
impl_procedure!([T1, T2]);

///////

use specta::DataType;
use specta::TypeCollection;

pub struct Api<S> {
    router: Router<S>,
}

impl<S> Api<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            router: Router::<S>::new(),
        }
    }

    pub fn procedure<F, Extractors, Input, Output, Error, T: 'static>(
        mut self,
        name: &str,
        f: F,
    ) -> Self
    where
        F: IntoProcedure<Extractors, Input, Output, Error>,
        F::Procedure: Handler<T, S>,
        Input: Type,
        Output: Type,
        Error: Type,
    {
        println!("Registering endpoint '{}'", name);
        let mut type_collection = TypeCollection::default();
        println!(
            "Input: {:?}",
            <Input as Type>::definition(&mut type_collection)
        );
        println!(
            "Output: {:?}",
            <Output as Type>::definition(&mut type_collection)
        );
        println!("TYPES: {:?}", type_collection);

        // https://discord.com/channels/1011665225809924136/1015433186299347005/threads/1356274733712150660
        // https://github.com/specta-rs/rspc/blob/786bce8571993a7d0ca17aa023b095c9730ffdb1/src/internal/procedure/procedure_store.rs#L12

        self.router = self
            .router
            .route(&format!("/{}", name), post(f.into_procedure()));
        self
    }

    pub fn build(self) -> Router<S> {
        self.router
    }
}

#[tokio::main]
async fn main() {
    // let mut types = TypeCollection::default();

    let app = Api::new()
        .procedure("p1", p1)
        .procedure("p2", p2)
        .procedure("p3", p3)
        .build();

    // let app = Router::new().merge(api);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
struct Input {
    #[specta(inline)]
    field: Email,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
struct Email(String);

#[derive(Clone, Serialize, Deserialize, Type)]
pub struct Output {
    field: String,
}

#[derive(Clone, Serialize, Deserialize, Type)]
enum ApiError {
    InternalError,
}

async fn p1(_input: ()) -> Result<Output, ApiError> {
    println!("p1: {:?}", _input);
    Ok(Output { field: "".into() })
}

async fn p2(_h: HeaderMap, _input: ()) -> Result<Output, ApiError> {
    println!("p2: {:?}", _input);
    Ok(Output { field: "".into() })
}

async fn p3(_h: HeaderMap, _input: Input) -> Result<Output, ApiError> {
    println!("p2: {:?}", _input);
    Ok(Output {
        field: "WORKS".into(),
    })
}
