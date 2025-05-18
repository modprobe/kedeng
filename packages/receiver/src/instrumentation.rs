use opentelemetry::global::BoxedTracer;
use opentelemetry::metrics::{Meter, MeterProvider};
use opentelemetry::{global, InstrumentationScope, KeyValue};
use opentelemetry_otlp::{MetricExporter, SpanExporter};
use opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use opentelemetry_semantic_conventions::SCHEMA_URL;
use std::sync::OnceLock;

pub(crate) fn init_tracer_provider() -> anyhow::Result<()> {
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(SpanExporter::builder().with_tonic().build()?)
        .build();

    global::set_tracer_provider(provider);

    Ok(())
}

fn get_metrics_provider() -> &'static SdkMeterProvider {
    static METRICS_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();
    METRICS_PROVIDER.get_or_init(|| {
        SdkMeterProvider::builder()
            .with_reader(
                PeriodicReader::builder(
                    MetricExporter::builder().with_tonic().build().unwrap(),
                    Tokio,
                )
                .build(),
            )
            .build()
    })
}

fn get_instrumentation_scope() -> &'static InstrumentationScope {
    static SCOPE: OnceLock<InstrumentationScope> = OnceLock::new();
    SCOPE.get_or_init(|| {
        InstrumentationScope::builder("kedeng/receiver")
            .with_attributes([KeyValue::new(SERVICE_NAME, "kedeng/receiver")])
            .with_schema_url(SCHEMA_URL)
            .build()
    })
}

pub(crate) fn get_tracer() -> &'static BoxedTracer {
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer_with_scope(get_instrumentation_scope().clone()))
}

pub(crate) fn get_meter() -> &'static Meter {
    static METER: OnceLock<Meter> = OnceLock::new();
    METER.get_or_init(|| {
        get_metrics_provider().meter_with_scope(get_instrumentation_scope().clone())
    })
}

pub(crate) fn shutdown_metrics() -> anyhow::Result<()> {
    get_metrics_provider().shutdown()?;

    Ok(())
}
