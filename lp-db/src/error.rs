use aws_smithy_http::result::CreateUnhandledError;
use std::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    SerializeError(serde_json::Error),
    Validation(String),
    AwsSdk(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::SerializeError(e) => Some(e),
            _ => None,
        }
    }
}
impl<T: std::error::Error + 'static + Send + Sync + CreateUnhandledError>
    From<aws_sdk_dynamodb::error::SdkError<T>> for Error
{
    fn from(err: aws_sdk_dynamodb::error::SdkError<T>) -> Self {
        let msg = format!("{:?}", err);
        Self::AwsSdk(msg)
    }
}
impl From<serde_dynamo::Error> for Error {
    fn from(err: serde_dynamo::Error) -> Self {
        Self::Validation(err.to_string())
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::SerializeError(e) => return e.fmt(f),
            Error::Validation(e) => e,
            Error::AwsSdk(e) => {
                writeln!(f, "AwsSdk error: {}", e)?;
                "Aws sdk error"
            }
        };
        write!(f, "{}", msg)
    }
}
