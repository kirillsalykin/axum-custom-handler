use specta::Type;

use std::marker::PhantomData;

#[derive(Clone)]
pub struct Procedure<F, Extractors, Input, Output, Error> {
    inner: F,
    _marker: PhantomData<(Extractors, Input, Output, Error)>,
}

pub trait IntoProcedure<const ARITY: usize, Extractors, Input, Output, Error> {
    type Procedure;
    fn into_procedure(self) -> Self::Procedure;
}

pub struct NoInput;

impl<F, Fut, Input, Output, Error> IntoProcedure<1, (), Input, Output, Error> for F
where
    F: FnOnce(Input) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Output, Error>> + Send + 'static,
    Input: Type + Clone + Send + Sync + 'static,
    Output: Type + Clone + Send + Sync + 'static,
    Error: Type + Clone + Send + Sync + 'static,
{
    type Procedure = Procedure<F, (), Input, Output, Error>;

    fn into_procedure(self) -> Self::Procedure {
        Procedure {
            inner: self,
            _marker: PhantomData,
        }
    }
}

impl<F, Fut, T1, Output, Error> IntoProcedure<1, (T1,), (), Output, Error> for F
where
    F: FnOnce(T1) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Output, Error>> + Send + 'static,
    Output: Type + Clone + Send + Sync + 'static,
    Error: Type + Clone + Send + Sync + 'static,
{
    type Procedure = Procedure<F, (T1,), (), Output, Error>;

    fn into_procedure(self) -> Self::Procedure {
        Procedure {
            inner: self,
            _marker: PhantomData,
        }
    }
}

impl<F, Fut, T1, Input, Output, Error> IntoProcedure<2, (T1,), Input, Output, Error> for F
where
    F: FnOnce(T1, Input) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Output, Error>> + Send + 'static,
    Input: Type + Clone + Send + Sync + 'static,
    Output: Type + Clone + Send + Sync + 'static,
    Error: Type + Clone + Send + Sync + 'static,
{
    type Procedure = Procedure<F, (T1,), Input, Output, Error>;

    fn into_procedure(self) -> Self::Procedure {
        Procedure {
            inner: self,
            _marker: PhantomData,
        }
    }
}

//macro_rules! impl_into {
//  ([$($ty:ident),* $(,)?]) => {
//        impl<F, Fut, $($ty,)* Output, Error> IntoProcedure<( $($ty,)* ), (), Output, Error>
//        for F
//        where
//            F: FnOnce( $($ty,)* ) -> Fut + Clone + Send + Sync + 'static,
//            Fut: Future<Output = Result<Output, Error>> + Send + 'static,
//            Output: Type + Clone + Send + Sync + 'static,
//            Error: Type + Clone + Send + Sync + 'static,
//        {
//            type Procedure = Procedure<F, ( $( $ty, )* ), (), Output, Error>;
//
//            fn into_procedure(self) -> Self::Procedure {
//                Procedure {
//                    inner: self,
//                    _marker: PhantomData,
//                }
//            }
//        }
//
//        impl<F, Fut, $($ty,)* Input, Output, Error> IntoProcedure<( $($ty,)* ), Input, Output, Error>
//        for F
//        where
//            F: FnOnce( $($ty,)* Input ) -> Fut + Clone + Send + Sync + 'static,
//            Fut: Future<Output = Result<Output, Error>> + Send + 'static,
//            Input: Type + Clone + Send + Sync + 'static,
//            Output: Type + Clone + Send + Sync + 'static,
//            Error: Type + Clone + Send + Sync + 'static,
//        {
//            type Procedure = Procedure<F, ( $( $ty, )* ), Input, Output, Error>;
//
//            fn into_procedure(self) -> Self::Procedure {
//                Procedure {
//                    inner: self,
//                    _marker: PhantomData,
//                }
//            }
//        }
//    }
//}
//
////impl_into!([]);
//impl_into!([T1]);
//impl_into!([T1, T2]);

#[tokio::main]
async fn main() {
    //let p1 = p1.into_procedure();
    let p2 = p2.into_procedure();
    let p3 = p3.into_procedure();
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
struct Output {
    field: String,
}

#[derive(Clone, Serialize, Deserialize, Type)]
enum ApiError {
    InternalError,
}

async fn p1() -> Result<Output, ApiError> {
    Ok(Output { field: "".into() })
}

async fn p2(_h: HeaderMap) -> Result<Output, ApiError> {
    Ok(Output { field: "".into() })
}

async fn p3(_h: HeaderMap, _input: Input) -> Result<Output, ApiError> {
    Ok(Output {
        field: "WORKS".into(),
    })
}
