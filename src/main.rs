use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, Server};
use const_format::{formatcp, str_replace};
use metrics::{decrement_gauge, histogram, increment_counter, increment_gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Layer;
use std::net::SocketAddr;
use std::time::{Instant, SystemTime};
use tracing::{debug, error, event, info, instrument, span, trace, Event, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    // Prepare tracing.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .with(MetricsLayer::new())
        .init();

    // Prepare metrics.
    let (recorder, exporter) = PrometheusBuilder::new()
        .with_http_listener(([127, 0, 0, 1], 9000))
        .build()
        .unwrap();
    let recorder = TracingContextLayer::all().layer(recorder);
    metrics::set_boxed_recorder(Box::new(recorder)).unwrap();

    tokio::spawn(exporter);

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/random", get(handlers::random));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

mod handlers {
    use super::*;

    const METRIC_BASE: &str = str_replace!(module_path!(), "::", "_");
    const DURATION_METRIC: &str = formatcp!("{METRIC_BASE}_duration_seconds");
    const TOTAL_METRIC: &str = formatcp!("{METRIC_BASE}_total");

    #[instrument]
    pub async fn root() -> &'static str {
        let start = Instant::now();

        let result = "Hello world!";

        histogram!(DURATION_METRIC, start.elapsed().as_secs_f64(), "handler" => "root");
        increment_counter!(TOTAL_METRIC, "handler" => "root", "result" => "ok");

        result
    }

    #[instrument(err)]
    pub async fn random() -> Result<String, StatusCode> {
        let start = Instant::now();

        let r: f64 = rand::random::<f64>();
        let result = if r < 0.3 {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        } else {
            Ok(format!("{}", r))
        };

        histogram!(DURATION_METRIC, start.elapsed().as_secs_f64(), "handler" => "random");
        if result.is_ok() {
            increment_counter!(TOTAL_METRIC, "handler" => "random", "result" => "ok");
        } else {
            increment_counter!(TOTAL_METRIC, "handler" => "random", "result" => "err");
        }

        result
    }
}

// #[instrument(err, ok, duration, total, concurrent, labels(%workspace_id), size(data))]
// #[instrument_full]

#[instrument]
fn create_notebook(workspace_id: u64, user_id: u64, data: Vec<u8>) -> Result<(), ()> {
    let start = Instant::now();

    // Should it be based on explicit logging or just assume the thing happened
    // if the function returned without error
    increment_counter!("create_notebook_total");
    increment_gauge!("create_notebook_concurrent", 1f64);

    let result = get_random_result();

    // Would be nice to have a helper method or something that figures out
    // that you want the duration of the span
    // - Maybe every span could have a duration?
    // - Or the instrument macro would time it
    let duration = start.elapsed();
    histogram!("create_notebook_duration_seconds", duration.as_secs_f64());
    decrement_gauge!("create_notebook_concurrent", 1f64);

    match &result {
        Ok(result) => {
            debug!("create notebook");
        }
        Err(err) => {
            error!(?err);
            increment_counter!("create_notebook_errors_total");
            // alert!();
        }
    }

    result
}

fn get_random_result() -> Result<(), ()> {
    let result = rand::random::<bool>();
    if result {
        Ok(())
    } else {
        Err(())
    }
}
