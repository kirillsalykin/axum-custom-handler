use serde_json::{Value, json};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
struct SignUpInput {
    // #[rule(length(min=10))]
    field_string: String,
    field_option: Option<u32>,
    email: Email,
    password: PlainTextPassword,
    // field_vec: Vec<i32>,
    // field_struct: InnerStruct;
}

#[derive(Debug, Clone)]
struct Email(String);

impl TryFrom<&Value> for Email {
    type Error = &'static str;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Some(v) = value.as_str() {
            Ok(Self(v.into()))
        } else {
            Err("NOT_STRING")
        }
    }
}

#[derive(Debug, Clone)]
struct PlainTextPassword(String);

impl TryFrom<&Value> for PlainTextPassword {
    type Error = &'static str;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Some(v) = value.as_str() {
            Ok(Self(v.into()))
        } else {
            Err("NOT_STRING")
        }
    }
}

impl TryFrom<&Value> for SignUpInput {
    type Error = &'static str;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let field_string: String =
            if let Some(v) = value.get("field_string").and_then(Value::as_str) {
                match String::try_from(v) {
                    Ok(v) => Ok(v),
                    Err(_) => Err("WRONG_TYPE"),
                }
            } else {
                Err("NO_FIELD")
            }?;

        let field_option: Option<u32> =
            if let Some(v) = value.get("field_option").and_then(Value::as_i64) {
                match u32::try_from(v) {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => Err("WRONG_TYPE"),
                }
            } else {
                Ok(None)
            }?;

        let email: Email = if let Some(v) = value.get("email") {
            Email::try_from(v)
        } else {
            Err("NO_FIELD")
        }?;

        let password: PlainTextPassword = if let Some(v) = value.get("password") {
            PlainTextPassword::try_from(v)
        } else {
            Err("NO_FIELD")
        }?;

        Ok(SignUpInput {
            field_string,
            field_option,
            email,
            password,
        })
    }
}

#[tokio::main]
async fn main() {
    let v = json!({
        "field_string": "String",
        // "field_option": -1,
        "email": "email",
        "password": "password"
    });

    let i = SignUpInput::try_from(&v);

    println!("RESULT: {:?}", i);
}
