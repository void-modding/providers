use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct User {
    pub(super) name: Option<String>,
    pub(super) avatar: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Thumbnail {
    pub(super) file: Option<String>
}

#[derive(Deserialize)]
pub(super) struct Tag {
    pub(super) name: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Mod {
    pub(super) id: Option<i64>,
    pub(super) name: Option<String>,
    pub(super) desc: Option<String>,
    pub(super) short_desc: Option<String>,
    pub(super) downloads: Option<u64>,
    pub(super) views: Option<u64>,
    pub(super) likes: Option<u64>,
    pub(super) thumbnail: Option<Thumbnail>,
    pub(super) user: Option<User>,
    pub(super) tags: Option<Vec<Tag>>,
}

#[derive(Deserialize)]
pub(super) struct Meta {
    pub(super) current_page: Option<u64>,
    pub(super) per_page: Option<u64>,
    pub(super) last_page: Option<u64>,
    pub(super) total: Option<u64>
}

#[derive(Deserialize)]
pub(super) struct DiscoverResponse {
    pub(super) data: Option<Vec<Mod>>,
    pub(super) meta: Option<Meta>
}

// Extended types
#[derive(Deserialize)]
pub(super) struct Banner {
    pub(super) file: Option<String>
}

#[derive(Deserialize)]
pub(super) struct Image {
    pub(super) file: Option<String>
}

#[derive(Deserialize)]
pub(super) struct ExtendedResponse {
    pub(super) banner: Option<Banner>,
    pub(super) images: Option<Vec<Image>>,
    pub(super) version: Option<String>,
    pub(super) description: Option<String>,
}
