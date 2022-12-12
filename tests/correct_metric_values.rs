use opentelemetry::{sdk::{metrics::{controllers, processors, selectors}, export::{metrics::aggregation}}, metrics::Unit, global, Context};
use opentelemetry_semantic_conventions::trace;
use prometheus::{Registry, TextEncoder};

#[test]
fn correct_metric_values() -> color_eyre::Result<()> {
    let prometheus_controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 70.0, 100.0, 200.0, 300.0, 400.0, 500.0, 700.0, 1000.0, 1500.0, 3000.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();

    let exporter = opentelemetry_prometheus::exporter(prometheus_controller)
        .with_registry(
            Registry::new_custom(None, None)
                .expect("create prometheus registry"),
        )
        .init();

    let meter = global::meter("poem");

    let duration_ms = meter
        .f64_histogram("poem_request_duration_ms")
        .with_unit(Unit::new("milliseconds"))
        .with_description(
            "request duration histogram (in milliseconds, since start of service)",
        )
        .init();

    let cx = Context::new();
    let mut labels = Vec::with_capacity(3);
    labels.push(trace::HTTP_METHOD.string("GET".to_string()));
    duration_ms.record(&cx, 207.0, &labels);

    let encoder = TextEncoder::new();
    let metric_families = exporter.registry().gather();
    let metric_output = encoder.encode_to_string(&metric_families)?;
    println!("metric_output\n{metric_output}");

    // This is as expected!

    assert_eq!(
        "# HELP poem_request_duration_ms request duration histogram (in milliseconds, since start of service)\n\
        # TYPE poem_request_duration_ms histogram\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"1\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"2\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"5\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"10\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"20\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"50\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"70\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"100\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"200\"} 0\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"300\"} 1\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"400\"} 1\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"500\"} 1\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"700\"} 1\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"1000\"} 1\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"1500\"} 1\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"3000\"} 1\n\
        poem_request_duration_ms_bucket{http_method=\"GET\",service_name=\"unknown_service\",le=\"+Inf\"} 1\n\
        poem_request_duration_ms_sum{http_method=\"GET\",service_name=\"unknown_service\"} 207\n\
        poem_request_duration_ms_count{http_method=\"GET\",service_name=\"unknown_service\"} 1\n",
        &metric_output
    );

    Ok(())
}