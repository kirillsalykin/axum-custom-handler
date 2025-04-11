use std::collections::HashMap;

use serde_json::{Value, json};
mod error;
use error::Error;

trait Distilled: Sized {
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Error>;
}

impl Distilled for String {
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Error> {
        let value = value.into().ok_or(Error::entry("missing_field"))?;
        value
            .as_str()
            .map(String::from)
            .ok_or(Error::entry("wrong_type"))
    }
}

impl Distilled for u32 {
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Error> {
        let value = value.into().ok_or(Error::entry("missing_field"))?;
        let n = value.as_i64().ok_or(Error::entry("wrong_type"))?;
        u32::try_from(n).map_err(|_| Error::entry("wrong_type"))
    }
}

impl<T: Distilled> Distilled for Option<T> {
    fn distill_from<'a, U: Into<Option<&'a Value>>>(value: U) -> Result<Self, Error> {
        match value.into() {
            Some(v) => Ok(Some(T::distill_from(v)?)),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone)]
struct Email(String);

impl Distilled for Email {
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Error> {
        let s = String::distill_from(value)?;
        Ok(Email(s))
    }
}

#[derive(Debug, Clone)]
struct PlainTextPassword(String);

impl Distilled for PlainTextPassword {
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Error> {
        let s = String::distill_from(value)?;
        Ok(PlainTextPassword(s))
    }
}

#[derive(Debug, Clone)]
struct SignUpInput {
    field_string: String,
    field_option: Option<u32>,
    email: Email,
    password: PlainTextPassword,
}

#[macro_export]
macro_rules! try_field {
    ($errors:ident, $field_name:expr, $expr:expr) => {
        match $expr {
            Ok(val) => Some(val),
            Err(err) => {
                $errors.insert($field_name.into(), err);
                None
            }
        }
    };
}

impl Distilled for SignUpInput {
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Error> {
        let value = value.into().ok_or(Error::entry("missing_field"))?;

        let mut errors = HashMap::with_capacity(4);

        let field_string = try_field!(
            errors,
            "field_string",
            String::distill_from(value.get("field_string"))
        );
        let field_option = try_field!(
            errors,
            "field_option",
            Option::<u32>::distill_from(value.get("field_option"))
        );
        let email = try_field!(errors, "email", Email::distill_from(value.get("email")));
        let password = try_field!(
            errors,
            "password",
            PlainTextPassword::distill_from(value.get("password"))
        );

        if errors.is_empty() {
            Ok(SignUpInput {
                field_string: field_string.unwrap(),
                field_option: field_option.unwrap(),
                email: email.unwrap(),
                password: password.unwrap(),
            })
        } else {
            Err(Error::Struct(errors))
        }
    }
}

#[tokio::main]
async fn main() {
    let v = json!({
         "field_string": "Some string",
         "field_option": -1,  // Optional field example.
        "email": "user@example.com",
        "password": "password123"
    });

    let result_from_value = SignUpInput::distill_from(&v);

    println!("RESULT from &Value: {:?}", result_from_value);
}
