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

use hyper::{client::HttpConnector, Client as HyperClient, Uri};
use hyper_tls::HttpsConnector;
use ruma_api::{
    error::{FromHttpRequestError, FromHttpResponseError},
    Endpoint, Outgoing,
};
use std::{convert::TryFrom, str::FromStr};
use url::Url;

use crate::error::Error;

#[derive(Debug)]
pub struct Client {
    homeserver_url: Url,
    hyper: HyperClient<HttpsConnector<HttpConnector>>,
    access_token: String,
    device_id: String,
}

impl Client {
    pub async fn create(
        homeserver_url: &str,
        user_id: &str,
        access_token: &str,
    ) -> Result<Client, Error> {
        use ruma_client_api::r0::session::login;

        let mut client = Client {
            homeserver_url: Url::parse(homeserver_url).unwrap(),
            hyper: HyperClient::builder()
                .keep_alive(true)
                .build(HttpsConnector::new()),
            access_token: String::new(),
            device_id: String::new(),
        };

        let response = client
            .request(login::Request {
                user: login::UserInfo::MatrixId(user_id.to_string()),
                login_info: login::LoginInfo::Token {
                    token: access_token.to_string(),
                },
                device_id: None,
                initial_device_display_name: None,
            })
            .await?;

        client.access_token = response.access_token;
        client.device_id = response.device_id;

        Ok(client)
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub async fn request<Request: Endpoint>(
        &self,
        request: Request,
    ) -> Result<<Request::Response as Outgoing>::Incoming, Error>
    where
        Request::Incoming: TryFrom<http::Request<Vec<u8>>, Error = FromHttpRequestError>,
        <Request::Response as Outgoing>::Incoming:
            TryFrom<http::Response<Vec<u8>>, Error = FromHttpResponseError>,
    {
        let mut hyper_request = request.try_into()?.map(hyper::Body::from);

        let mut url = self.homeserver_url.clone();

        {
            let uri = hyper_request.uri();

            url.set_path(uri.path());
            url.set_query(uri.query());

            if Request::METADATA.requires_authentication {
                url.query_pairs_mut()
                    .append_pair("access_token", &self.access_token);
            }
        }

        *hyper_request.uri_mut() = Uri::from_str(url.as_ref())?;

        let hyper_response = self.hyper.request(hyper_request).await?;
        let (head, body) = hyper_response.into_parts();
        let full_body = hyper::body::to_bytes(body).await?;
        let full_response = http::Response::from_parts(head, full_body.as_ref().to_owned());

        Ok(<Request::Response as Outgoing>::Incoming::try_from(
            full_response,
        )?)
    }

    pub async fn sync(
        &self,
    ) -> Result<ruma_client_api::r0::sync::sync_events::IncomingResponse, Error> {
        use ruma_client_api::r0::sync::sync_events;

        self.request(sync_events::Request {
            filter: None,
            since: None,
            full_state: None,
            set_presence: None,
            timeout: None,
        })
        .await
    }
}
