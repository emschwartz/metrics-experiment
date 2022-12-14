use const_format::{formatcp, str_replace};
use metrics::{decrement_gauge, histogram, increment_counter, increment_gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Layer;
use std::time::{Instant, SystemTime};
use tracing::{debug, error, event, info, instrument, span, trace, Event, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() {
    // Prepare tracing.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .with(MetricsLayer::new())
        .init();

    // Prepare metrics.
    let recorder = PrometheusBuilder::new().build_recorder();
    let recorder = TracingContextLayer::all().layer(recorder);
    metrics::set_boxed_recorder(Box::new(recorder)).unwrap();
    println!("Hello, world!");

    for i in 0..10 {
        create_notebook(i / 3, i, vec![0; i as usize + 1]).ok();
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