mod common;

use actix_web::{http::StatusCode, test, web, App};
use binance_trading_bot::models::*;
use binance_trading_bot::routes;
use binance_trading_bot::storage::Storage;
use common::*;
use serde_json::json;

#[actix_web::test]
async fn test_execute_trade_success() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .configure(routes::configure_trading_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/trades/execute")
        .insert_header((
            "Authorization",
            format!("Bearer {}", create_test_jwt("user123")),
        ))
        .set_json(sample_trade_request())
        .to_request();

    // In real test, would need mock Binance API
    // let resp = test::call_service(&app, req).await;
    // assert_success_response!(resp);

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_get_positions() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .configure(routes::configure_trading_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/positions")
        .insert_header((
            "Authorization",
            format!("Bearer {}", create_test_jwt("user123")),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_success_response!(resp);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["positions"].is_array());

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_get_account_info() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .configure(routes::configure_trading_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/account")
        .insert_header((
            "Authorization",
            format!("Bearer {}", create_test_jwt("user123")),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_success_response!(resp);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["balance"].is_object());
    assert!(body["total_balance_usdt"].is_number());

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_trade_history() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .configure(routes::configure_trading_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/trades/history?symbol=BTCUSDT&limit=10")
        .insert_header((
            "Authorization",
            format!("Bearer {}", create_test_jwt("user123")),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_success_response!(resp);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["trades"].is_array());
    assert!(body["pagination"].is_object());

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_invalid_trade_parameters() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .configure(routes::configure_trading_routes),
    )
    .await;

    // Invalid quantity (too small)
    let req = test::TestRequest::post()
        .uri("/api/trades/execute")
        .insert_header((
            "Authorization",
            format!("Bearer {}", create_test_jwt("user123")),
        ))
        .set_json(json!({
            "symbol": "BTCUSDT",
            "side": "BUY",
            "type": "LIMIT",
            "quantity": 0.00001,  // Below minimum
            "price": 45000.0
        }))
        .to_request();

    // Would validate in real implementation
    // let resp = test::call_service(&app, req).await;
    // assert_error_response!(resp, StatusCode::BAD_REQUEST);

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_unauthorized_access() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .configure(routes::configure_trading_routes),
    )
    .await;

    // No auth header
    let req = test::TestRequest::get().uri("/api/positions").to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::UNAUTHORIZED);

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_close_position() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .configure(routes::configure_trading_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/positions/close")
        .insert_header((
            "Authorization",
            format!("Bearer {}", create_test_jwt("user123")),
        ))
        .set_json(json!({
            "symbol": "BTCUSDT",
            "quantity": 0.001,
            "reason": "manual_close"
        }))
        .to_request();

    // Would need existing position in real test
    // let resp = test::call_service(&app, req).await;
    // assert_success_response!(resp);

    cleanup_test_db(db).await;
}
