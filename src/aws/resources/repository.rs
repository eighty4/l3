use crate::aws::clients::AwsClients;
use crate::aws::resources::repository::AwsResourceMessage::*;
use crate::aws::resources::state::DeployedProjectState;
use crate::aws::resources::{
    AwsGatewayIntegration, AwsGatewayRoute, AwsLambdaFunction, AwsLambdaResources,
};
use crate::aws::AwsApiGateway;
use crate::lambda::LambdaFn;
use crate::ui::exit::err_exit;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

enum AwsResourceMessage {
    CreatedLambdaFunction(Arc<AwsLambdaFunction>),
    CreatedGatewayIntegration(Arc<AwsGatewayIntegration>),
    CreatedGatewayRoute(Arc<AwsGatewayRoute>),
    DeletedLambdaFunction(Arc<AwsLambdaFunction>),
    DeletedGatewayIntegration(Arc<AwsGatewayIntegration>),
    DeletedGatewayRoute(Arc<AwsGatewayRoute>),
    RefreshState(oneshot::Sender<()>),
    RequestResources(Arc<LambdaFn>, oneshot::Sender<AwsLambdaResources>),
}

struct AwsResourcesEventLoop {
    api_gateway: Arc<AwsApiGateway>,
    msg_rx: UnboundedReceiver<AwsResourceMessage>,
    project_name: String,
    sdk_clients: Arc<AwsClients>,
    state: DeployedProjectState,
}

impl AwsResourcesEventLoop {
    fn new(
        api_gateway: Arc<AwsApiGateway>,
        msg_rx: UnboundedReceiver<AwsResourceMessage>,
        project_name: String,
        sdk_clients: Arc<AwsClients>,
    ) -> Self {
        Self {
            api_gateway,
            msg_rx,
            project_name,
            sdk_clients,
            state: Default::default(),
        }
    }

    async fn start(&mut self) {
        loop {
            if let Some(msg) = self.msg_rx.recv().await {
                match msg {
                    CreatedLambdaFunction(lambda_config) => {
                        self.state.created_lambda_function(lambda_config)
                    }
                    CreatedGatewayIntegration(gateway_integration) => {
                        self.state.created_gateway_integration(gateway_integration)
                    }
                    CreatedGatewayRoute(gateway_route) => {
                        self.state.created_gateway_route(gateway_route)
                    }
                    DeletedLambdaFunction(lambda_config) => {
                        self.state.deleted_lambda_function(lambda_config)
                    }
                    DeletedGatewayIntegration(gateway_integration) => {
                        self.state.deleted_gateway_integration(gateway_integration)
                    }
                    DeletedGatewayRoute(gateway_route) => {
                        self.state.deleted_gateway_route(gateway_route)
                    }
                    RefreshState(tx) => self.refresh_state(tx).await,
                    RequestResources(lambda_fn, tx) => self.request_resources(lambda_fn, tx),
                }
            }
        }
    }

    async fn refresh_state(&mut self, tx: oneshot::Sender<()>) {
        match DeployedProjectState::fetch_from_aws(
            &self.sdk_clients,
            &self.project_name,
            &self.api_gateway,
        )
        .await
        {
            Ok(state) => {
                self.state = state;
                tx.send(()).unwrap()
            }
            Err(err) => err_exit(err.to_string().as_str()),
        }
    }

    fn request_resources(&self, lambda_fn: Arc<LambdaFn>, tx: oneshot::Sender<AwsLambdaResources>) {
        let _ = tx.send(self.state.get_deployed_components(&lambda_fn));
    }
}

pub struct AwsResources {
    msg_tx: UnboundedSender<AwsResourceMessage>,
}

impl AwsResources {
    pub fn new(
        api_gateway: Arc<AwsApiGateway>,
        project_name: String,
        sdk_clients: Arc<AwsClients>,
    ) -> Arc<Self> {
        let (msg_tx, msg_rx) = unbounded_channel();
        let mut event_loop =
            AwsResourcesEventLoop::new(api_gateway, msg_rx, project_name, sdk_clients);
        tokio::spawn(async move { event_loop.start().await });
        Arc::new(Self { msg_tx })
    }

    pub fn created_lambda_function(
        &self,
        lambda_config: Arc<AwsLambdaFunction>,
    ) -> Result<(), anyhow::Error> {
        self.msg_tx.send(CreatedLambdaFunction(lambda_config))?;
        Ok(())
    }

    pub fn created_gateway_integration(
        &self,
        gateway_integration: Arc<AwsGatewayIntegration>,
    ) -> Result<(), anyhow::Error> {
        self.msg_tx
            .send(CreatedGatewayIntegration(gateway_integration))?;
        Ok(())
    }

    pub fn created_gateway_route(
        &self,
        gateway_route: Arc<AwsGatewayRoute>,
    ) -> Result<(), anyhow::Error> {
        self.msg_tx.send(CreatedGatewayRoute(gateway_route))?;
        Ok(())
    }

    pub fn deleted_lambda_function(
        &self,
        lambda_config: Arc<AwsLambdaFunction>,
    ) -> Result<(), anyhow::Error> {
        self.msg_tx.send(DeletedLambdaFunction(lambda_config))?;
        Ok(())
    }

    pub fn deleted_gateway_integration(
        &self,
        gateway_integration: Arc<AwsGatewayIntegration>,
    ) -> Result<(), anyhow::Error> {
        self.msg_tx
            .send(DeletedGatewayIntegration(gateway_integration))?;
        Ok(())
    }

    pub fn deleted_gateway_route(
        &self,
        gateway_route: Arc<AwsGatewayRoute>,
    ) -> Result<(), anyhow::Error> {
        self.msg_tx.send(DeletedGatewayRoute(gateway_route))?;
        Ok(())
    }

    pub async fn refresh_state(&self) -> Result<(), anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        self.msg_tx.send(RefreshState(tx))?;
        rx.await?;
        Ok(())
    }

    pub async fn resources_for_fn(
        &self,
        lambda_fn: &Arc<LambdaFn>,
    ) -> Result<AwsLambdaResources, anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        self.msg_tx.send(RequestResources(lambda_fn.clone(), tx))?;
        Ok(rx.await?)
    }
}
