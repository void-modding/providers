use std::sync::Arc;
use lib_vmm::{api::ProviderApi, net::{HttpError, ProviderHttpClient, ProviderHttpClientTypedExt, ReqwestProviderHttpClient}, traits::{discovery::{DiscoveryError, DiscoveryMeta, DiscoveryQuery, DiscoveryResult, ModExtendedMetadata, ModSummary, PaginationMeta}, mod_provider::{ModDownloadResult, ModProvider, ModProviderFeatures}}};
use reqwest::Url;
// use super::types::{DiscoverResponse, ExtendedResponse};

pub struct NexusProvider {
    api: Arc<dyn ProviderApi>,
    http: Arc<dyn ProviderHttpClient>
}

impl NexusProvider {
    pub fn new(api: Arc<dyn ProviderApi>) -> Self {
        let http = ReqwestProviderHttpClient::new();
        Self { api, http }
    }
}