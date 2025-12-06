use anyhow::Context;
use axum::routing::{delete, get, post, put};
use axum::{Router, middleware};
use back_end::logs_db;
use back_end::{AppState, api_docs::ApiDoc, db, handlers, rate_limit};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

    if log_format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_target(true)
            .with_current_span(false)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true)
            .init();
    }

    let connect_options = SqliteConnectOptions::new()
        .filename("auth.db")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .pragma("cache_size", "2000")
        .pragma("temp_store", "memory")
        .pragma("mmap_size", "268435456")
        .pragma("foreign_keys", "ON")
        .pragma("busy_timeout", "30000");

    let auth_db_sqlite_pool = SqlitePoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .max_lifetime(Some(std::time::Duration::from_secs(1800)))
        .idle_timeout(Some(std::time::Duration::from_secs(600)))
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect_with(connect_options)
        .await
        .with_context(|| "Failed to create auth_db_sqlite_pool")
        .expect("Cannot create authentication db");

    db::init_db(&auth_db_sqlite_pool)
        .await
        .expect("Failed to initialize database");

    let rate_limit_state = rate_limit::RateLimitState::new();
    rate_limit_state.clone().spawn_cleanup_task();

    let metrics = back_end::metrics::Metrics::new();
    metrics.clone().spawn_logging_task();

    let state = AppState {
        sqlite: auth_db_sqlite_pool,
        rate_limit: rate_limit_state.clone(),
        metrics,
        mongodb: logs_db::init_mongodb()
            .await
            .expect("Failed to initialize MongoDB"),
    };

    let api_routes = Router::new()
        .route("/auth/register", post(handlers::register_company_admin))
        .route("/auth/login", post(handlers::login))
        .route("/auth/verify", post(handlers::verify_token))
        .route(
            "/auth/invitations/accept",
            post(handlers::accept_invitation),
        )
        .route(
            "/auth/invitations/details",
            get(handlers::get_invitation_details),
        )
        .route(
            "/auth/password/request-reset",
            post(handlers::request_password_reset),
        )
        .route("/auth/password/reset", post(handlers::reset_password))
        .route("/auth/me", get(handlers::get_current_user))
        .route("/auth/profile", put(handlers::update_profile))
        .route("/auth/invitations/send", post(handlers::invite_user))
        .route("/auth/company/members", get(handlers::get_company_members))
        .route("/logs/templates", post(handlers::add_template))
        .route("/logs/templates", get(handlers::get_template))
        .route("/logs/templates/all", get(handlers::get_all_templates))
        .route("/logs/templates/update", put(handlers::update_template))
        .route("/logs/templates/rename", put(handlers::rename_template))
        .route("/logs/templates", delete(handlers::delete_template))
        .route("/logs/entries/due", get(handlers::list_due_forms_today))
        .route("/logs/entries", post(handlers::create_log_entry))
        .route("/logs/entries", get(handlers::list_user_log_entries))
        .route("/logs/entries/{entry_id}", get(handlers::get_log_entry))
        .route("/logs/entries/{entry_id}", put(handlers::update_log_entry))
        .route(
            "/logs/entries/{entry_id}",
            delete(handlers::delete_log_entry),
        )
        .route(
            "/logs/entries/{entry_id}/submit",
            post(handlers::submit_log_entry),
        )
        .route(
            "/logs/entries/{entry_id}/unsubmit",
            post(handlers::unsubmit_log_entry),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit::rate_limit_middleware,
        ))
        .with_state(state);

    let swagger_router: Router = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .into();

    let allowed_origins = [
        "http://localhost:5173".parse().unwrap(),
        "http://logsmart.app".parse().unwrap(),
        "https://logsmart.app".parse().unwrap(),
    ];

    let app = swagger_router.merge(api_routes).layer(
        tower_http::cors::CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
            .allow_credentials(true),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6767")
        .await
        .expect("Failed to bind to port 6767");

    tracing::info!("Server running on http://0.0.0.0:6767");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Server error");
}
