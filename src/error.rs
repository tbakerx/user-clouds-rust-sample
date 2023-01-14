use std::borrow::Cow;

use mendes::application::IntoResponse;
use mendes::http::request::Parts;
use mendes::http::{Response, StatusCode};
use mendes::hyper::Body;
use mendes::Application;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Mendes(mendes::Error),
    #[error("serialization error: {0:?}")]
    SerializeJson(#[from] serde_json::Error),
    #[error("{1}")]
    String(StatusCode, Cow<'static, str>),
}

impl Error {
    pub fn internal(s: impl Into<Cow<'static, str>>) -> Self {
        Self::String(StatusCode::INTERNAL_SERVER_ERROR, s.into())
    }
    fn response(&self, _: &Parts) -> Response<Body> {
        Response::builder()
            .status(StatusCode::from(self))
            .body(self.to_string().into())
            .unwrap()
    }
}

impl From<mendes::Error> for Error {
    fn from(e: mendes::Error) -> Self {
        Error::Mendes(e)
    }
}

impl From<&Error> for StatusCode {
    fn from(e: &Error) -> StatusCode {
        use Error::*;
        match e {
            Mendes(e) => e.into(),
            SerializeJson(_) => StatusCode::INTERNAL_SERVER_ERROR,
            String(status, _) => *status,
        }
    }
}

impl<A> IntoResponse<A> for Error
where
    A: Application<ResponseBody = Body, Error = Error>,
{
    fn into_response(self, _: &A, req: &Parts) -> Response<Body> {
        self.response(req)
    }
}
