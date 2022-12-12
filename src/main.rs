use std::time::Duration;

use eyre::Context;
use opentelemetry::sdk::{metrics::{controllers, processors, selectors}, export::metrics::aggregation};
use poem::{Server, listener::TcpListener, Route, get, EndpointExt, middleware::OpenTelemetryMetrics, handler, web::Path, endpoint::PrometheusExporter};
use tokio::time::sleep;


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let prometheus_controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 70.0, 100.0, 200.0, 300.0, 400.0, 500.0, 700.0, 1000.0, 1500.0, 3000.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();

    println!("Starting server at localhost:8080");
    Server::new(TcpListener::bind("localhost:8080"))
            .run(
                Route::new()
                    .at("/hello/:name", get(hello))
                    .nest("/prometheus_metrics", PrometheusExporter::with_controller(prometheus_controller))
                    .with(OpenTelemetryMetrics::new()))

            .await
            .context("while running public API")?;

    Ok(())
}

#[handler]
async fn hello(Path(name): Path<String>) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("hello: {}", name)
}