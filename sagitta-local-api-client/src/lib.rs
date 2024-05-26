use sagitta_local_api_schema::v1::sync::{V1SyncRequest, V1SyncResponse};

#[derive(Debug, Clone)]
pub struct SagittaLocalApiClient {
    pub base_url: String,
}

#[derive(Debug)]
pub enum SagittaLocalApiClientError {
    Ureq(Box<ureq::Error>),
    IO(Box<std::io::Error>),
}

impl SagittaLocalApiClient {
    pub fn new(base_url: String) -> Self {
        let mut base_url = base_url;
        if base_url.ends_with('/') {
            base_url.pop();
        }
        Self { base_url }
    }

    pub fn v1_sync(
        &self,
        request: V1SyncRequest,
    ) -> Result<V1SyncResponse, SagittaLocalApiClientError> {
        let url = format!("{}/v1/sync", self.base_url);
        let sync_res: V1SyncResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaLocalApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaLocalApiClientError::IO(Box::new(e)))?;
        Ok(sync_res)
    }
}
