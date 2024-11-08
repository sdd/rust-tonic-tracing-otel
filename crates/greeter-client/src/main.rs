use tokio::time::{sleep, Duration};
use opentelemetry::{global as otel};
use greeter_client::WrappedGreeterClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // very opinionated init of tracing, look as is source to make your own
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()
        .expect("init subscribers");

    let mut client = WrappedGreeterClient::new().await?;

    client.say_hello_several().await?;

    // flushes all spans to the otlp exporter
    otel::shutdown_tracer_provider();

    // A sleep is needed because init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()
    // configures the otlp exporter in batch mode with a timeout of 5s.
    // If we exit immediately then nothing will get sent via OTLP.
    // we need to wait at least 5s for the batch timeout to expire
    // and the traces to send.
    // Whilst the Otel SpanExporter trait (https://docs.rs/opentelemetry_sdk/0.26.0/x86_64-unknown-linux-gnu/opentelemetry_sdk/export/trace/trait.SpanExporter.html)
    // does contain shutdown() and force_flush() methods,
    // the OTLP Span Exporter does not implement them and so there is no
    // way at present to force flush.
    sleep(Duration::from_millis(6000)).await;

    Ok(())
}
