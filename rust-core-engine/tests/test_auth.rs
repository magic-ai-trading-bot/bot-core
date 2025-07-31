mod common;

use actix_web::{http::StatusCode, test, web, App};
use binance_trading_bot::auth::{handlers, middleware::AuthMiddleware, models::*};
use binance_trading_bot::storage::Storage;
use common::*;
use serde_json::json;

#[actix_web::test]
async fn test_register_success() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app =
        test::init_service(App::new().app_data(web::Data::new(storage)).service(
            web::scope("/api/auth").route("/register", web::post().to(handlers::register)),
        ))
        .await;

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({
            "email": "test@example.com",
            "password": "secure_password123",
            "full_name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_success_response!(resp);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["email"], "test@example.com");
    assert!(body["token"].is_string());

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_register_duplicate_email() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app =
        test::init_service(App::new().app_data(web::Data::new(storage)).service(
            web::scope("/api/auth").route("/register", web::post().to(handlers::register)),
        ))
        .await;

    let register_data = json!({
        "email": "duplicate@example.com",
        "password": "password123",
        "full_name": "Test User"
    });

    // First registration
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_success_response!(resp);

    // Duplicate registration
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::CONFLICT);

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_login_success() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new().app_data(web::Data::new(storage)).service(
            web::scope("/api/auth")
                .route("/register", web::post().to(handlers::register))
                .route("/login", web::post().to(handlers::login)),
        ),
    )
    .await;

    // Register first
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({
            "email": "login@example.com",
            "password": "password123",
            "full_name": "Login Test"
        }))
        .to_request();
    test::call_service(&app, req).await;

    // Login
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({
            "email": "login@example.com",
            "password": "password123"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_success_response!(resp);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "login@example.com");

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_login_invalid_credentials() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .service(web::scope("/api/auth").route("/login", web::post().to(handlers::login))),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({
            "email": "nonexistent@example.com",
            "password": "wrongpassword"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::UNAUTHORIZED);

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_auth_middleware() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create a protected endpoint
    async fn protected_handler() -> actix_web::Result<String> {
        Ok("Protected content".to_string())
    }

    let app = test::init_service(
        App::new().app_data(web::Data::new(storage)).service(
            web::scope("/api")
                .wrap(AuthMiddleware)
                .route("/protected", web::get().to(protected_handler)),
        ),
    )
    .await;

    // Without token
    let req = test::TestRequest::get().uri("/api/protected").to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::UNAUTHORIZED);

    // With valid token (mocked)
    let req = test::TestRequest::get()
        .uri("/api/protected")
        .insert_header(("Authorization", "Bearer valid_token_here"))
        .to_request();
    // In real test, would need proper JWT validation setup

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_password_validation() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app =
        test::init_service(App::new().app_data(web::Data::new(storage)).service(
            web::scope("/api/auth").route("/register", web::post().to(handlers::register)),
        ))
        .await;

    // Too short password
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({
            "email": "test@example.com",
            "password": "short",
            "full_name": "Test User"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::BAD_REQUEST);

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_email_validation() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app =
        test::init_service(App::new().app_data(web::Data::new(storage)).service(
            web::scope("/api/auth").route("/register", web::post().to(handlers::register)),
        ))
        .await;

    // Invalid email format
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({
            "email": "not-an-email",
            "password": "password123",
            "full_name": "Test User"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::BAD_REQUEST);

    cleanup_test_db(db).await;
}
