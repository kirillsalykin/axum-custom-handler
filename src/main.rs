use serde_json::{Value, json};

trait Distilled: Sized {
    type Error;
    fn distill_from(value: Option<&Value>) -> Result<Self, Self::Error>;
}

impl Distilled for String {
    type Error = &'static str;
    fn distill_from(value: Option<&Value>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => v.as_str().map(String::from).ok_or("WRONG_TYPE"),
            None => Err("NO_FIELD"),
        }
    }
}

impl Distilled for u32 {
    type Error = &'static str;
    fn distill_from(value: Option<&Value>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => {
                if let Some(n) = v.as_i64() {
                    u32::try_from(n).map_err(|_| "WRONG_TYPE")
                } else {
                    Err("WRONG_TYPE")
                }
            }
            None => Err("NO_FIELD"),
        }
    }
}

impl<T: Distilled> Distilled for Option<T> {
    type Error = T::Error;
    fn distill_from(value: Option<&Value>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => Ok(Some(T::distill_from(Some(v))?)),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone)]
struct Email(String);

impl Distilled for Email {
    type Error = &'static str;
    fn distill_from(value: Option<&Value>) -> Result<Self, Self::Error> {
        let s = String::distill_from(value)?;
        Ok(Email(s))
    }
}

#[derive(Debug, Clone)]
struct PlainTextPassword(String);

impl Distilled for PlainTextPassword {
    type Error = &'static str;
    fn distill_from(value: Option<&Value>) -> Result<Self, Self::Error> {
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
    fn distill_from(value: Option<&Value>) -> Result<Self, Self::Error> {
        let value = value.ok_or("NO_FIELD")?;
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

    // Changed the call from `try_from` to `distill_from` using fully-qualified syntax:
    let result = <SignUpInput as Distilled>::distill_from(Some(&v));
    println!("RESULT: {:?}", result);
}
