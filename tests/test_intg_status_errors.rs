use actix_web_json_error_middleware::{JsonMiddleware, JsonErrorMessage};

use std::ops::Range;

use actix_web::{App, HttpResponse, HttpResponseBuilder, test, web};
use actix_web::http::StatusCode;
use test::TestRequest;

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
async fn test_middleware_test_handler() {
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
