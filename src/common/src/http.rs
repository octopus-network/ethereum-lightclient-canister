pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
pub use icp::{get, post};

#[cfg(not(target_arch = "wasm32"))]
pub use native::{get, post};

#[cfg(target_arch = "wasm32")]
mod icp {
    use ic_cdk::api::management_canister::http_request::{
        http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
        HttpResponse as CanisterHttpResponse,
    };

    use crate::errors::HttpError;
    use crate::icp::DEFAULT_HTTP_OUTCALL_COST;

    use super::HttpResponse;

    // TODO: should this be configurable?
    const MAX_REDIRECTS: usize = 2;

    pub async fn get(url: &str) -> Result<HttpResponse, HttpError> {
        let mut url = url.to_owned();

        for _ in 0..=MAX_REDIRECTS {
            let req = CanisterHttpRequestArgument {
                url: url.to_owned(),
                method: HttpMethod::GET,
                ..Default::default()
            };
            let resp = http_request(req, DEFAULT_HTTP_OUTCALL_COST).await?.0;

            match http_status_from_nat(&resp.status)? {
                // moved_permamently / found / see_other / temporary_redirect / permament_redirect
                // redirect and retry
                301..=303 | 307..=308 => {
                    url = resp
                        .headers
                        .iter()
                        .find_map(|header| {
                            if header.name.eq_ignore_ascii_case("location") {
                                Some(header.value.clone())
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| {
                            HttpError::Other(format!(
                                "Got {} status code without 'Location' header",
                                resp.status
                            ))
                        })?;
                }
                _ => return resp.try_into(),
            }
        }

        Err(HttpError::Other(
            "Exceeded redirection limit: {MAX_REDIRECTS}".to_owned(),
        ))
    }

    pub async fn post(
        url: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Result<HttpResponse, HttpError> {
        let headers = headers
            .iter()
            .map(|(name, value)| HttpHeader {
                name: name.to_string(),
                value: value.to_string(),
            })
            .collect();
        let req = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::POST,
            headers,
            body: Some(body),
            ..Default::default()
        };
        let resp = http_request(req, DEFAULT_HTTP_OUTCALL_COST).await?;
        resp.0.try_into()
    }

    impl TryFrom<CanisterHttpResponse> for HttpResponse {
        type Error = HttpError;

        fn try_from(value: CanisterHttpResponse) -> Result<Self, Self::Error> {
            Ok(Self {
                status: http_status_from_nat(&value.status)?,
                body: value.body,
            })
        }
    }

    fn http_status_from_nat(status: &candid::Nat) -> Result<u16, HttpError> {
        (&status.0).try_into().map_err(|err| {
            HttpError::Other(format!("Status should be a 3 digit number, got: {err}"))
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

    use crate::errors::HttpError;

    use super::HttpResponse;

    pub async fn get(url: &str) -> Result<HttpResponse, HttpError> {
        let resp = reqwest::get(url)
            .await
            .map_err(|err| HttpError::Other(format!("Request failed: {err}")))?;
        HttpResponse::from_reqwest(resp).await
    }

    pub async fn post(
        url: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Result<HttpResponse, HttpError> {
        let mut header_map = HeaderMap::new();
        for &(name, value) in headers {
            let name: HeaderName = name
                .parse()
                .map_err(|_| HttpError::Other(format!("Invalid header name: {name}")))?;
            let value: HeaderValue = value
                .parse()
                .map_err(|_| HttpError::Other(format!("Invalid header value: {value}")))?;
            header_map.insert(name, value);
        }

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .body(body)
            .headers(header_map)
            .send()
            .await
            .map_err(|err| HttpError::Other(format!("Request failed: {err}")))?;

        HttpResponse::from_reqwest(resp).await
    }

    impl HttpResponse {
        async fn from_reqwest(resp: reqwest::Response) -> Result<Self, HttpError> {
            let status = resp.status().as_u16();
            let body = resp
                .bytes()
                .await
                .map_err(|err| HttpError::Other(format!("Couldn't read response body: {err}")))?;

            Ok(Self {
                status,
                body: body.into(),
            })
        }
    }
}
