use std::collections::HashMap;
use opentelemetry::propagation::{Extractor, TextMapPropagator};
use pyo3::{FromPyObject, PyResult, Python};
use opentelemetry::Context;
use pyo3::types::IntoPyDict;

const TRACEPARENT_HEADER: &str = "traceparent";
const TRACESTATE_HEADER: &str = "tracestate";

pub fn get_context_from_python(py: Python<'_>) -> PyResult<Context> {
    let get_current_context = py.import("opentelemetry.context")?.getattr("get_current")?;
    let inject = py.import("opentelemetry.propagate")?.getattr("inject")?;

    let current_context = get_current_context.call0()?;
    let data = pyo3::types::PyDict::new(py);
    let kwargs = [("context", current_context), ("carrier", data)].into_py_dict(py);
    inject.call((), Some(kwargs))?;

    let data: HashMap<String, String> = data.extract()?;
    let carrier: Carrier = data.into();

    let propagator = opentelemetry_sdk::propagation::TraceContextPropagator::new();
    let context = propagator.extract(&carrier);

    Ok(context)
}

#[derive(Default, Clone, Debug, FromPyObject)]
pub struct Carrier {
    /// The context [traceparent](https://www.w3.org/TR/trace-context/#traceparent-header).
    #[pyo3(item)]
    traceparent: Option<String>,
    /// The context [tracestate](https://www.w3.org/TR/trace-context/#tracestate-header).
    #[pyo3(item)]
    tracestate: Option<String>,
}

impl Extractor for Carrier {
    fn get(&self, key: &str) -> Option<&str> {
        match key.to_lowercase().as_str() {
            TRACEPARENT_HEADER => self.traceparent.as_deref(),
            TRACESTATE_HEADER => self.tracestate.as_deref(),
            _ => None,
        }
    }

    fn keys(&self) -> Vec<&str> {
        vec![TRACEPARENT_HEADER, TRACESTATE_HEADER]
    }
}

impl From<HashMap<String, String>> for Carrier {
    fn from(value: HashMap<String, String>) -> Self {
        Self {
            tracestate: value.get(TRACESTATE_HEADER).cloned(),
            traceparent: value.get(TRACEPARENT_HEADER).cloned(),
        }
    }
}
