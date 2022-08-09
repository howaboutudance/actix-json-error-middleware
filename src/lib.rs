//     Copyright 2022 Michael Penhallegon <mike@hematite.tech>
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0

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

            let status_code = res.status();
            if status_code.as_u16() > 299 {
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
    use std::ops::Range;

    use actix_web::{App, HttpResponse, HttpResponseBuilder, test, web};
    use actix_web::http::StatusCode;
    use test::TestRequest;

    use super::*;

    // Test Handlers

    /// Test handler for status code responses
    ///
    ///  # Arguments
    ///  * `path` - A wen::Path object with the values after /status/
    ///
    /// Take a response from parameterized path `/status/` and returns
    /// a response with the corresponding  HTTP status code
    ///
    /// # Examples
    /// ```
    /// use actix_web::{App, web, test};
    /// use test::TestRequest;
    /// let app = test::init_service(
    ///     App::new()
    ///         .route("/status/{code}", web::route().to(status_handler))
    /// ).await;
    ///
    /// let req = TestRequest::get().uri("/status/4o4").to_request();
    /// let resp = test::call_service(&app, req).await;
    ///
    /// assert_eq!(resp.status().as_u16(), 404);
    /// ```
    async fn status_handler(path: web::Path<(u16, )>) -> HttpResponse {
        let status_code = path.into_inner().0;
        let status_code_obj = StatusCode::from_u16(status_code).unwrap();
        HttpResponseBuilder::new(status_code_obj).finish()
    }

    /// Base test to check test handler functionality
    /// request an endpoint that returns 404 and checks content-type
    #[actix_web::test]
    async fn test_get_404_json_content_type() {
        let app = test::init_service(
            App::new()
                .wrap(JsonMiddleware)
                .route("/status/{code}", web::route().to(status_handler))
        ).await;

        let req = TestRequest::get().uri("/4o4").to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
        assert_eq!(resp.headers().get("content-type").unwrap(), "application/json")
    }

    /// Arbitrary Endpoint Check
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

    // Status Code Range Tests

    /// Tests Iteratively GET Requests For HTTP Status Codes 200-299
    #[actix_web::test]
    async fn test_200s_get_errors() {
        help_test_get_by_range(400u16..499u16).await
    }

    /// Tests Iteratively GET Requests For HTTP Status Codes 300-399
    #[actix_web::test]
    async fn test_300s_get_errors() {
        help_test_get_by_range(300u16..399u16).await
    }

    /// Tests Iteratively GET Requests For HTTP Status Codes 400-499
    #[actix_web::test]
    async fn test_400s_get_errors() {
        help_test_get_by_range(400u16..499u16).await
    }

    /// Tests Iteratively GET Requests For HTTP Status Codes 500-512
    #[actix_web::test]
    async fn test_500s_get_errors() {
        help_test_get_by_range(500u16..512u16).await
    }

    /// Tests Iteratively POST Requests For HTTP Status Codes 200-299
    #[actix_web::test]
    async fn test_200s_post_errors() {
        help_test_post_by_range(200u16..299u16).await
    }

    /// Tests Iteratively POST Requests For HTTP Status Codes 300-399
    #[actix_web::test]
    async fn test_300s_post_errors() {
        help_test_post_by_range(300u16..399u16).await
    }

    /// Tests Iteratively POST Requests For HTTP Status Codes 400-499
    #[actix_web::test]
    async fn test_400s_post_errors() {
        help_test_post_by_range(400u16..499u16).await
    }

    /// Tests Iteratively POST Requests For HTTP Status Codes 500-512
    #[actix_web::test]
    async fn test_500s_post_errors() {
        help_test_post_by_range(500u16..512u16).await
    }


    /// Tests Iteratively PUT Requests For HTTP Status Codes 200-299
    #[actix_web::test]
    async fn test_200s_put_errors() {
        help_test_put_by_range(200u16..299u16).await
    }

    /// Tests Iteratively PUT Requests For HTTP Status Codes 300-399
    #[actix_web::test]
    async fn test_300s_put_errors() {
        help_test_put_by_range(300u16..399u16).await
    }

    /// Tests Iteratively PUT Requests For HTTP Status Codes 400-499
    #[actix_web::test]
    async fn test_400s_put_errors() {
        help_test_put_by_range(400u16..499u16).await
    }

    /// Tests Iteratively PUT Requests For HTTP Status Codes 500-512
    #[actix_web::test]
    async fn test_500s_put_errors() {
        help_test_put_by_range(500u16..412u16).await
    }


    // Test Helpers

    /// Helps Iteratively Test a Range of Status Codes with POST Requests
    async fn help_test_post_by_range(status_range: Range<u16>) {
        let app = test::init_service(
            App::new()
                .wrap(JsonMiddleware)
                .route("/status/{code}", web::route().to(status_handler))
        ).await;

        for status_code in status_range {
            let status_code_endpoint = format!("/status/{}", status_code);
            let req = TestRequest::post().uri(status_code_endpoint.as_str()).to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status().as_u16(), status_code);
            assert_eq!(resp.headers().get("content-type").unwrap(), "application/json");

            if status_code > 299 {
                let req_json = TestRequest::put().uri(status_code_endpoint.as_str()).to_request();
                let resp_json: JsonErrorMessage = test::call_and_read_body_json(&app, req_json).await;
                assert_eq!(resp_json.error, status_code)
            }
        }
    }


    /// Helps Iteratively Test a Range of Status Codes with GET Requests
    async fn help_test_get_by_range(status_range: Range<u16>) {
        let app = test::init_service(
            App::new()
                .wrap(JsonMiddleware)
                .route("/status/{code}", web::route().to(status_handler))
        ).await;

        for status_code in status_range {
            let status_code_endpoint = format!("/status/{}", status_code);
            let req = TestRequest::get().uri(status_code_endpoint.as_str()).to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status().as_u16(), status_code);
            assert_eq!(resp.headers().get("content-type").unwrap(), "application/json");

            if status_code > 299 {
                let req_json = TestRequest::put().uri(status_code_endpoint.as_str()).to_request();
                let resp_json: JsonErrorMessage = test::call_and_read_body_json(&app, req_json).await;
                assert_eq!(resp_json.error, status_code)
            }
        }
    }

    /// Helps Iteratively Test a Range of Status Codes with PUT Requests
    async fn help_test_put_by_range(status_range: Range<u16>) {
        let app = test::init_service(
            App::new()
                .wrap(JsonMiddleware)
                .route("/status/{code}", web::route().to(status_handler))
        ).await;

        for status_code in status_range {
            let status_code_endpoint = format!("/status/{}", status_code);
            let req = TestRequest::put().uri(status_code_endpoint.as_str()).to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status().as_u16(), status_code);
            assert_eq!(resp.headers().get("content-type").unwrap(), "application/json");

            if status_code > 299 {
                let req_json = TestRequest::put().uri(status_code_endpoint.as_str()).to_request();
                let resp_json: JsonErrorMessage = test::call_and_read_body_json(&app, req_json).await;
                assert_eq!(resp_json.error, status_code)
            }
        }
    }
}