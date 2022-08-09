use std::future::{Ready, ready};

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, http::header, HttpResponseBuilder};
use actix_web::body::{EitherBody};
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JsonErrorMessage {
    /// A JSON Serializable Struct for an Error Response
    pub error: u16,
    pub message: String,
}

pub struct JsonErrorMiddlewareDefinition<S> {
    /// A Middleware Definition Struct for The Service Component of the Middleware
    service: S,
}


impl<S, B> Service<ServiceRequest> for JsonErrorMiddlewareDefinition<S>
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        async move {
            let res_result: Result<ServiceResponse<B>, Error> = fut.await;
            let mut res = res_result.ok().expect("response found");

            // Check status_code value. If above 299, create a response that has a json blob
            // generated from JsonErrorMessage
            let status_code = res.status();

            if status_code.as_u16() > 299 {
                // generate an EitherBody, which is an either a success, and thus just json content-type
                // or the specialized JSONErrorMessageResponse
                let response = HttpResponseBuilder::new(status_code).json(
                    JsonErrorMessage {
                        error: status_code.as_u16(),
                        message: status_code.to_string(),
                    }
                ).map_into_right_body();
                return Ok(ServiceResponse::into_response(res, response));
            } else {
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"));
                Ok(res.map_into_left_body())
            }
        }.boxed_local()
    }
}


pub struct JsonMiddleware;

impl JsonMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for JsonMiddleware
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = JsonErrorMiddlewareDefinition<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JsonErrorMiddlewareDefinition { service: service }))
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{App, test};
    use test::TestRequest;

    use super::*;


    /// Basic Endpoint Check
    ///
    /// takes an arbitrary endpoint that is not handler, sending a GET request knowing it should
    /// return with a `404 NOT FOUND` status and does checks that:
    /// - content-type header is application/json
    /// - HTTP status code is 404
    /// - The response body is JSON
    /// - The JSON response body has a key/value pair of `{"error": 404}`
    #[actix_web::test]
    async fn test_get_non_status_endpoint() {
        let test_uri = "/foo";
        let app = test::init_service(
            App::new()
                .wrap(JsonMiddleware)
        ).await;

        let req = TestRequest::get().uri(test_uri).to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 404);

        assert_eq!(resp.headers().get("content-type").unwrap(), "application/json");

        let req_json = TestRequest::get().uri(test_uri).to_request();
        let resp_json: JsonErrorMessage = test::call_and_read_body_json(&app, req_json).await;
        assert_eq!(resp_json.error, 404)
    }
}