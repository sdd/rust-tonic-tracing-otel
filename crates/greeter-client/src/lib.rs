use tonic::transport::Channel;
use tonic_tracing_opentelemetry::middleware::client::{OtelGrpcLayer, OtelGrpcService};
use tower::ServiceBuilder;
use greeter_core::greeter_client::GreeterClient;
use greeter_core::HelloRequest;

#[derive(Debug)]
pub struct WrappedGreeterClient {
    client: GreeterClient<OtelGrpcService<Channel>>
}

impl WrappedGreeterClient {
    pub async fn new() -> anyhow::Result<WrappedGreeterClient> {
        let channel = Channel::from_static("http://127.0.0.1:50051")
            .connect()
            .await?;
        let channel = ServiceBuilder::new().layer(OtelGrpcLayer).service(channel);

        Ok(Self {
            client: GreeterClient::new(channel)
        })
    }

    #[tracing::instrument]
    pub async fn say_hello_several(&mut self) -> anyhow::Result<()> {
        for _i in 0..3 {
            self.say_hello().await?;
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn say_hello(&mut self) -> anyhow::Result<()> {
        let request = tonic::Request::new(HelloRequest {
            name: "Tonic".into(),
        });

        let response = self.client.say_hello(request).await?;

        println!("RESPONSE={:?}", response);

        Ok(())
    }
}