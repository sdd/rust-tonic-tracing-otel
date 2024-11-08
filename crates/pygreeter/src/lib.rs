// use std::future::Future;
// use std::sync::Arc;
// use std::pin::Pin;
use pyo3::{prelude::*, wrap_pyfunction};

// use async_once_cell::Lazy;

use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing::{instrument, Span};

use greeter_client::WrappedGreeterClient;
mod otel_python;

// type H = impl Future<Output=WrappedGreeterClient>;
//
// #[pyclass]
// struct PyGreeter {
//     client: Pin<Lazy<WrappedGreeterClient, H>>,
// }
//
// #[pymethods]
// impl PyGreeter {
//     #[new]
//     fn new() -> Self {
//         init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().expect("init subscribers");
//
//         Self {
//             client: Arc::pin(Lazy::new(async move {
//                 WrappedGreeterClient::new().await
//             }))
//         }
//     }
//
//     fn greet(&self, py: Python) -> PyResult<&PyAny> {
//         let context = otel_python::get_context_from_python(py)?;
//
//         pyo3_asyncio::tokio::future_into_py(py, async move {
//             let pygreeter_root_span: Span = tracing::info_span!("pygreeter_rust_greet_root");
//             // Set the parent context to propagate the trace from python into rust
//             pygreeter_root_span.set_parent(context);
//
//             let _guard = pygreeter_root_span.enter();
//             let c = self.client.as_ref();
//             c.await.say_hello_several().await.unwrap();
//
//             Ok(())
//         })
//     }
// }

#[pyfunction]
#[instrument(skip(py))]
fn greet(py: Python) -> PyResult<&PyAny> {
    let context = otel_python::get_context_from_python(py)?;

    pyo3_asyncio::tokio::future_into_py(py, async move {

        unsafe { std::env::set_var("OTEL_SERVICE_NAME", "greeter_client"); }
        init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().expect("init subscribers");
        let mut client = WrappedGreeterClient::new().await.unwrap();

        // Generate a new tracing span that will be the root span within pygreeter
        let pygreeter_root_span: Span = tracing::info_span!("pygreeter_rust_greet_root");
        // Set the parent context to propagate the trace from python into Rust
        pygreeter_root_span.set_parent(context);

        // actually enter the span we just created
        let _guard = pygreeter_root_span.enter();

        // do some work
        client.say_hello_several().await.unwrap();

        // guard implicitly dropped when it goes out of scope here to end this span
        Ok(())
    })
}

#[pymodule]
fn pygreeter(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    Ok(())
}


