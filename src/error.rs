// Copyright (c) 2016 Jimmy Cuadra
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.


//! Error conditions.

use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

use ruma_api::error::{FromHttpResponseError, IntoHttpError};

/// An error that can occur during client operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Queried endpoint requires authentication but was called on an anonymous client.
    AuthenticationRequired,
    /// Construction of the HTTP request failed (this should never happen).
    IntoHttp(IntoHttpError),
    /// The request's URL is invalid (this should never happen).
    Url(UrlError),
    /// Couldn't obtain an HTTP response (e.g. due to network or DNS issues).
    Response(ResponseError),
    /// Converting the HTTP response to one of ruma's types failed.
    FromHttpResponse(FromHttpResponseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::AuthenticationRequired => {
                write!(f, "The queried endpoint requires authentication but was called with an anonymous client.")
            }
            Self::IntoHttp(err) => write!(f, "HTTP request construction failed: {}", err),
            Self::Url(UrlError(err)) => write!(f, "Invalid URL: {}", err),
            Self::Response(ResponseError(err)) => write!(f, "Couldn't obtain a response: {}", err),
            Self::FromHttpResponse(err) => write!(f, "HTTP response conversion failed: {}", err),
        }
    }
}

impl From<IntoHttpError> for Error {
    fn from(err: IntoHttpError) -> Self {
        Error::IntoHttp(err)
    }
}

#[doc(hidden)]
impl From<http::uri::InvalidUri> for Error {
    fn from(err: http::uri::InvalidUri) -> Self {
        Error::Url(UrlError(err))
    }
}

#[doc(hidden)]
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::Response(ResponseError(err))
    }
}

impl From<FromHttpResponseError> for Error {
    fn from(err: FromHttpResponseError) -> Self {
        Error::FromHttpResponse(err)
    }
}

impl StdError for Error {}

#[derive(Debug)]
pub struct UrlError(http::uri::InvalidUri);

#[derive(Debug)]
pub struct ResponseError(hyper::Error);
