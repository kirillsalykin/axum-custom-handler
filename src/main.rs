use std::{collections::HashMap, hash::Hash};

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

impl Distilled for SignUpInput {
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Error> {
        let value = value.into().ok_or(Error::entry("missing_field"))?;

        let mut errors = HashMap::with_capacity(4);

        let field_string = String::distill_from(value.get("field_string"));
        let field_option = Option::<u32>::distill_from(value.get("field_option"));
        let email = Email::distill_from(value.get("email"));
        let password = PlainTextPassword::distill_from(value.get("password"));

        if field_string.is_ok() && field_option.is_ok() && email.is_ok() && password.is_ok() {
            Ok(SignUpInput {
                field_string: field_string.unwrap(),
                field_option: field_option.unwrap(),
                email: email.unwrap(),
                password: password.unwrap(),
            })
        } else {
            if field_string.is_err() {
                errors.insert("field_string".into(), field_string.err().unwrap());
            }

            if field_option.is_err() {
                errors.insert("field_option".into(), field_option.err().unwrap());
            }

            if email.is_err() {
                errors.insert("email".into(), email.err().unwrap());
            }

            if password.is_err() {
                errors.insert("password".into(), password.err().unwrap());
            }

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
