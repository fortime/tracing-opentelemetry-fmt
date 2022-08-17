//! # tracing-opentelemetry-fmt
//!
//! > A library for adding `trace.id` and `span.id` to tracing log
//!
//! ## About
//! Implement a `Layer` of `tracing-subscriber` which combines a `OpentelmetryLayer` and `FmtLayer`. This layer adds `trace.id` and `span.id` to `FmtLayer` so that we can log `trace.id` and `span.id` from opentelemetry.
//!
//! ## Examples
//!
//! ```
//! use std::io;
//!
//! use opentelemetry::sdk::export::trace::stdout;
//! use tracing_subscriber::{
//!     fmt::{self, format::FmtSpan},
//!     layer::SubscriberExt,
//!     util::SubscriberInitExt,
//! };
//! use tracing_opentelemetry_fmt::OpenTelemetryFmtLayerBuilder;
//!
//! fn main() {
//!     let fmt_layer = fmt::layer()
//!         .with_thread_ids(true)
//!         .with_target(true)
//!         .with_span_events(FmtSpan::FULL);
//!     let tracer = stdout::new_pipeline().with_writer(io::sink()).install_simple();
//!     let opentelemetry_layer = tracing_opentelemetry::layer()
//!         .with_exception_field_propagation(true)
//!         .with_threads(true)
//!         .with_tracer(tracer);
//!
//!     let opentelemetry_fmt_layer =
//!         OpenTelemetryFmtLayerBuilder::new(opentelemetry_layer, fmt_layer).build();
//!     tracing_subscriber::registry()
//!         .with(opentelemetry_fmt_layer)
//!         .try_init()
//!         .expect("It should be successful");
//!
//!     tracing::info_span!("span1").in_scope(|| {
//!         tracing::info!("in span1");
//!     });
//! }
//! ```
use std::any::TypeId;

use opentelemetry::trace::{TraceContextExt, Tracer};
use tracing::{
    field::FieldSet,
    metadata::LevelFilter,
    span::{Attributes, Record},
    subscriber::Interest,
    Event, Id, Metadata, Span, Subscriber, Value,
};
use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt, PreSampledTracer};
use tracing_subscriber::{
    fmt::{FormatEvent, FormatFields, Layer as FmtLayer, MakeWriter},
    layer::{Context, Layered},
    registry::LookupSpan,
    Layer,
};

pub struct OpenTelemetryFmtLayerBuilder<S, T1, N2, E2, W2> {
    opentelemetry_layer: OpenTelemetryLayer<S, T1>,
    fmt_layer: FmtLayer<S, N2, E2, W2>,
    field_names: &'static [&'static str; 2],
}

impl<S, T1, N2, E2, W2> OpenTelemetryFmtLayerBuilder<S, T1, N2, E2, W2> {
    pub fn new(
        opentelemetry_layer: OpenTelemetryLayer<S, T1>,
        fmt_layer: FmtLayer<S, N2, E2, W2>,
    ) -> Self {
        Self {
            opentelemetry_layer,
            fmt_layer,
            field_names: &["trace.id", "span.id"],
        }
    }

    pub fn with_field_names(mut self, field_names: &'static [&'static str; 2]) -> Self {
        self.field_names = field_names;
        self
    }
}

impl<S, T1, N2, E2, W2> OpenTelemetryFmtLayerBuilder<S, T1, N2, E2, W2>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    T1: Tracer + PreSampledTracer + 'static,
    N2: for<'writer> FormatFields<'writer> + 'static,
    E2: FormatEvent<S, N2> + 'static,
    W2: for<'writer> MakeWriter<'writer> + 'static,
{
    pub fn build(
        self,
    ) -> Layered<OpenTelemetryFmtLayer<S, N2, E2, W2>, OpenTelemetryLayer<S, T1>, S> {
        let Self {
            opentelemetry_layer,
            fmt_layer,
            field_names,
        } = self;
        let opentelemetry_fmt_layer = OpenTelemetryFmtLayer {
            fmt_layer,
            field_names,
        };
        opentelemetry_layer.and_then(opentelemetry_fmt_layer)
    }
}

pub struct OpenTelemetryFmtLayer<S, N2, E2, W2> {
    fmt_layer: FmtLayer<S, N2, E2, W2>,
    field_names: &'static [&'static str; 2],
}

impl<S, N2, E2, W2> Layer<S> for OpenTelemetryFmtLayer<S, N2, E2, W2>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N2: for<'writer> FormatFields<'writer> + 'static,
    E2: FormatEvent<S, N2> + 'static,
    W2: for<'writer> MakeWriter<'writer> + 'static,
{
    fn on_layer(&mut self, subscriber: &mut S) {
        self.fmt_layer.on_layer(subscriber)
    }

    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        self.fmt_layer.register_callsite(metadata)
    }

    fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool {
        self.fmt_layer.enabled(metadata, ctx)
    }

    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        self.fmt_layer.on_new_span(attrs, id, ctx)
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        self.fmt_layer.max_level_hint()
    }

    fn on_record(&self, span: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        self.fmt_layer.on_record(span, values, ctx)
    }

    fn on_follows_from(&self, span: &Id, follows: &Id, ctx: Context<'_, S>) {
        self.fmt_layer.on_follows_from(span, follows, ctx)
    }

    fn event_enabled(&self, event: &Event<'_>, ctx: Context<'_, S>) -> bool {
        self.fmt_layer.event_enabled(event, ctx)
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        self.fmt_layer.on_event(event, ctx)
    }

    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        let span_context = Span::current().context();
        let opentelemetry_span = span_context.span();
        let ids = if opentelemetry_span.span_context().is_valid() {
            Some((
                opentelemetry_span.span_context().trace_id().to_string(),
                opentelemetry_span.span_context().span_id().to_string(),
            ))
        } else {
            None
        };

        self.fmt_layer.on_enter(id, ctx.clone());

        if let Some(ids) = ids {
            let field_set = FieldSet::new(
                self.field_names,
                ctx.metadata(id)
                    .expect("Metadata not found, this is a bug")
                    .callsite(),
            );
            let mut it = field_set.iter();
            let trace_field = it.next().expect("Trace field not found, this is a bug");
            let span_field = it.next().expect("Span field not found, this is a bug");
            let values = [
                (&trace_field, Some(&ids.0 as &dyn Value)),
                (&span_field, Some(&ids.1 as &dyn Value)),
            ];
            let values = field_set.value_set(&values);
            let record = Record::new(&values);
            self.fmt_layer.on_record(id, &record, ctx.clone());
        }
    }

    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        self.fmt_layer.on_exit(id, ctx)
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        self.fmt_layer.on_close(id, ctx)
    }

    fn on_id_change(&self, old: &Id, new: &Id, ctx: Context<'_, S>) {
        self.fmt_layer.on_id_change(old, new, ctx)
    }

    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        self.fmt_layer.downcast_raw(id)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use opentelemetry::sdk::export::trace::stdout;
    use tracing_subscriber::{
        fmt::{self, format::FmtSpan},
        layer::SubscriberExt,
        util::SubscriberInitExt,
    };

    use super::*;

    #[test]
    fn test_with_field_names() {
        let fmt_layer = fmt::layer()
            .with_thread_ids(true)
            .with_target(true)
            .with_span_events(FmtSpan::FULL);
        let tracer = stdout::new_pipeline()
            .with_writer(io::sink())
            .install_simple();
        let opentelemetry_layer = tracing_opentelemetry::layer()
            .with_exception_field_propagation(true)
            .with_threads(true)
            .with_tracer(tracer);

        let opentelemetry_fmt_layer =
            OpenTelemetryFmtLayerBuilder::new(opentelemetry_layer, fmt_layer)
                .with_field_names(&["custom.trace.id", "custom.span.id"])
                .build();
        tracing_subscriber::registry()
            .with(opentelemetry_fmt_layer)
            .try_init()
            .expect("It should be successful");

        tracing::info_span!("span1").in_scope(|| {
            tracing::info!("in span1");
        });
    }
}
