use std::sync::Arc;
use lib_vmm::{api::ProviderApi, net::{HttpError, ProviderHttpClient, ProviderHttpClientTypedExt, ReqwestProviderHttpClient}, traits::{discovery::{DiscoveryError, DiscoveryMeta, DiscoveryQuery, DiscoveryResult, ModExtendedMetadata, ModSummary, PaginationMeta}, mod_provider::{ModDownloadResult, ModProvider, ModProviderFeatures}}};
use reqwest::Url;
use super::types::{DiscoverResponse, ExtendedResponse};

pub struct ModWorkShopProvider {
    api: Arc<dyn ProviderApi>,
    http: Arc<dyn ProviderHttpClient>
}

impl ModWorkShopProvider {
    pub fn new(api: Arc<dyn ProviderApi>) -> Self {
        let http = ReqwestProviderHttpClient::new();
        Self { api, http }
    }

    fn build_url(&self, query: &DiscoveryQuery) -> Result<Url, DiscoveryError> {
        let game = self.api.context()
            .get_game_provider(&query.game_id)
            .map_err(|e| DiscoveryError::InvalidQuery(format!("ID {} not loaded", query.game_id)))?;

        let game_id = game.get_external_id();

        let base = format!("https://api.modworkshop.net/games/{}/mods", game_id);
        let mut url = Url::parse(&base).map_err(|e| DiscoveryError::Internal(e.to_string()))?;

        {
            let mut qp = url.query_pairs_mut();

            if let Some(page) = query.page {
                let page = if page == 0 { 1 } else { page };
                qp.append_pair("page", &page.to_string());
            }
        }
        Ok(url)
    }

    #[inline]
    fn map_http_error(&self, err: HttpError) -> DiscoveryError {
        match err {
            HttpError::Network(s) => DiscoveryError::Network(s),
            HttpError::Parse(s) => DiscoveryError::Internal(format!("parse: {}", s)),
            HttpError::Schema(s) => DiscoveryError::Internal(format!("schema: {}", s)),
            HttpError::Internal(s) => DiscoveryError::Internal(s)
        }
    }

}

#[async_trait::async_trait]
impl ModProvider for ModWorkShopProvider {
async fn discover(&self, query: &DiscoveryQuery) -> Result<DiscoveryResult, DiscoveryError> {
        let target = self.build_url(&query)?;
        let resp: DiscoverResponse = self
            .http
            .get_typed(target.as_str())
            .await
            .map_err(|e| self.map_http_error(e))?;

        let mods_vec = resp
            .data
            .ok_or_else(|| DiscoveryError::Internal("Missing data[]".into()))?;

        let meta = resp
            .meta
            .ok_or_else(|| DiscoveryError::Internal("malformed response: Missing meta{}".into()))?;

        let mut summaries = Vec::with_capacity(mods_vec.len());

        for m in mods_vec {
                   let thumb_file = m
                       .thumbnail
                       .as_ref()
                       .and_then(|t| t.file.as_ref())
                       .filter(|f| !f.is_empty())
                       .map(|f| format!("https://storage.modworkshop.net/mods/images/{}", f))
                       .unwrap_or_else(|| "https://modworkshop.net/assets/no-preview.webp".to_owned());

                   let user_name = m
                       .user
                       .as_ref()
                       .and_then(|u| u.name.as_ref())
                       .cloned()
                       .unwrap_or_else(|| "error".to_owned());

                   let user_avatar = m
                       .user
                       .as_ref()
                       .and_then(|u| u.avatar.as_ref())
                       .map(|a| {
                           if a.starts_with("http://") || a.starts_with("https://") {
                               a.to_owned()
                           } else {
                               format!("https://storage.modworkshop.net/users/images/{}", a)
                           }
                       })
                       .unwrap_or_else(|| "error".to_owned());

                   let tags: Vec<String> = m
                       .tags
                       .unwrap_or_default()
                       .into_iter()
                       .filter_map(|t| t.name)
                       .collect();

                   summaries.push(ModSummary {
                       name: m.name.unwrap_or_else(|| "error".to_owned()),
                       id: m.id.unwrap_or_default().to_string(),
                       description: m.desc.unwrap_or_else(|| "error".to_owned()),
                       short_description: m.short_desc.unwrap_or_else(|| "error".to_owned()),
                       downloads: m.downloads.unwrap_or(0),
                       views: m.views.unwrap_or(0),
                       likes: m.likes.unwrap_or(0),
                       thumbnail_image: thumb_file,
                       user_name,
                       user_avatar,
                       tags,
                   });
        }

        Ok(DiscoveryResult {
            meta: DiscoveryMeta {
                provider_id: self.register(),
                game_id: query.game_id.clone(),
                pagination: PaginationMeta {
                    current: meta.current_page.unwrap_or(1),
                    page_size: meta.per_page.unwrap_or(50),
                    total_pages: Some(meta.last_page.unwrap_or(1)),
                    total_items: Some(meta.total.unwrap_or(0))
                },
                applied_tags: vec![],
                available_tags: Some(vec![]),
            },
            mods: summaries,
        })
    }

 async fn download_mod(&self, _mod_id: String) -> ModDownloadResult {
        // TODO: make this use mod_id
        //     -> This would require us make an API to convert the numeric mod_id into a download link (like how extended_metadata works)
        let target = String::from("https://storage.modworkshop.net/mods/files/53461_71246_ERjHBd1mwDsnSW70RlJ2meqkucPO3JtAsXfpyDU5.zip?filename=Rich%20Presence%20Musical.zip");
        let mut rx = self.api.queue_download(target).await;

        use ModDownloadResult::*;
        loop {
            if rx.changed().await.is_err() {
                return Failed("Download task ended unexpectedly".into());
            }
            match &*rx.borrow() {
                InProgress(p) => return InProgress(p.clone()),
                Completed(p) => return Completed(p.clone()),
                Failed(e) => return Failed(e.clone()),
                Cancelled => return Cancelled,
                _ => {}
            }
        }
    }

    async fn get_extended_mod(&self, id: &str) -> ModExtendedMetadata {
        let url = format!("https://api.modworkshop.net/mods/{}", id);
        let parsed: ExtendedResponse = match self.http.get_typed(&url).await {
            Ok(v) => v,
            Err(e) => {
                return ModExtendedMetadata {
                    header_image: "https://modworkshop.net/assets/default-banner.webp".to_owned(),
                    carousel_images: vec![],
                    version: "".to_owned(),
                    installed: false,
                    description: format!("Failed to fetch: {}", e),
                }
            }
        };

        let header_image = parsed
            .banner
            .as_ref()
            .and_then(|b| b.file.as_ref())
            .filter(|f| !f.is_empty())
            .map(|f| format!("https://storage.modworkshop.net/mods/images/{}", f))
            .unwrap_or_else(|| "https://modworkshop.net/assets/default-banner.webp".to_owned());

        let carousel_images = parsed
            .images
            .unwrap_or_default()
            .into_iter()
            .filter_map(|img| img.file)
            .filter(|f| !f.is_empty())
            .map(|f| format!("https://storage.modworkshop.net/mods/images/{}", f))
            .collect::<Vec<String>>();

        ModExtendedMetadata {
            header_image,
            carousel_images,
            version: parsed.version.unwrap_or_default(),
            installed: false,
            description: parsed.description.unwrap_or_default()
        }
    }

    fn configure(&self) -> &ModProviderFeatures {
        todo!("configure");
    }

    fn register(&self) -> String {
        "core:modworkshop".into()
    }

}
