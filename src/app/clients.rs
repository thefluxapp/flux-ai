use flux_lib::error::Error;
use flux_messages_api::messages_service_client::MessagesServiceClient;
use flux_users_api::users_service_client::UsersServiceClient;
use tonic::transport::Channel;

use super::settings::ClientsSettings;

pub struct AppClients {
    // pub settings: ClientsSettings,
    pub users_service_client: UsersServiceClient<Channel>,
    pub messages_service_client: MessagesServiceClient<Channel>,
}

impl AppClients {
    pub async fn new(settings: ClientsSettings) -> Result<Self, Error> {
        let users_service_client =
            UsersServiceClient::connect(settings.flux_users.endpoint.clone()).await?;

        let messages_service_client =
            MessagesServiceClient::connect(settings.flux_messages.endpoint.clone()).await?;

        Ok(Self {
            // settings,
            users_service_client,
            messages_service_client,
        })
    }
}
