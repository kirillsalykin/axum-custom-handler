use serde_json::{Value, json};

trait Distilled: Sized {
    type Error;
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Self::Error>;
}

impl Distilled for String {
    type Error = &'static str;
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Self::Error> {
        let value = value.into().ok_or("NO_FIELD")?;
        value.as_str().map(String::from).ok_or("WRONG_TYPE")
    }
}

impl Distilled for u32 {
    type Error = &'static str;
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Self::Error> {
        let value = value.into().ok_or("NO_FIELD")?;
        let n = value.as_i64().ok_or("WRONG_TYPE")?;
        u32::try_from(n).map_err(|_| "WRONG_TYPE")
    }
}

impl<T: Distilled> Distilled for Option<T> {
    type Error = T::Error;
    fn distill_from<'a, U: Into<Option<&'a Value>>>(value: U) -> Result<Self, Self::Error> {
        match value.into() {
            Some(v) => Ok(Some(T::distill_from(v)?)),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone)]
struct Email(String);

impl Distilled for Email {
    type Error = &'static str;
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Self::Error> {
        let s = String::distill_from(value)?;
        Ok(Email(s))
    }
}

#[derive(Debug, Clone)]
struct PlainTextPassword(String);

impl Distilled for PlainTextPassword {
    type Error = &'static str;
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Self::Error> {
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
    type Error = &'static str;
    fn distill_from<'a, T: Into<Option<&'a Value>>>(value: T) -> Result<Self, Self::Error> {
        let value = value.into().ok_or("NO_FIELD")?;
        let field_string = String::distill_from(value.get("field_string"))?;
        let field_option = Option::<u32>::distill_from(value.get("field_option"))?;
        let email = Email::distill_from(value.get("email"))?;
        let password = PlainTextPassword::distill_from(value.get("password"))?;
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
        "field_string": "Some string",
        // "field_option": -1,  // Optional field example.
        "email": "user@example.com",
        "password": "password123"
    });

    // Both of these calls work:
    let result_from_value = SignUpInput::distill_from(&v);

    println!("RESULT from &Value: {:?}", result_from_value);
}
