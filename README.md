# tracing-opentelemetry-fmt

> A library for adding `trace.id` and `span.id` to tracing log

## About

Implement a `Layer` of `tracing-subscriber` which combines a `OpentelmetryLayer` and `FmtLayer`. This layer adds `trace.id` and `span.id` to `FmtLayer` so that we can log `trace.id` and `span.id` from opentelemetry.

## Examples

```
use std::io;

use opentelemetry::sdk::export::trace::stdout;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_opentelemetry_fmt::OpenTelemetryFmtLayerBuilder;

fn main() {
    let fmt_layer = fmt::layer()
        .with_thread_ids(true)
        .with_target(true)
        .with_span_events(FmtSpan::FULL);
    let tracer = stdout::new_pipeline().with_writer(io::sink()).install_simple();
    let opentelemetry_layer = tracing_opentelemetry::layer()
        .with_exception_field_propagation(true)
        .with_threads(true)
        .with_tracer(tracer);

    let opentelemetry_fmt_layer =
        OpenTelemetryFmtLayerBuilder::new(opentelemetry_layer, fmt_layer).build();
    tracing_subscriber::registry()
        .with(opentelemetry_fmt_layer)
        .try_init()
        .expect("It should be successful");

    tracing::info_span!("span1").in_scope(|| {
        tracing::info!("in span1");
    });
}
```
