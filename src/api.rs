use base64;
use hex;
use minreq;
use nop_json::{DebugToJson, Reader, TryFromJson};
use sha1::{Digest, Sha1};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("[{}] API Request failed: {}", .status, .reason)]
    Request { status: i32, reason: String },
    #[error("Fetching failed: {}", .0)]
    Fetch(#[from] minreq::Error),
    #[error("Unable to parse data: {}", .0)]
    Parse(#[from] std::io::Error),
    #[error("{}", .0)]
    Skin(#[from] SkinError),
}

#[derive(Error, Debug)]
pub enum SkinError {
    #[error("Error decoding texture data: {}", .0)]
    Decoding(#[from] base64::DecodeError),
    #[error("Error reading decoded texture data: {}", .0)]
    Reader(#[from] std::io::Error),
}

#[derive(TryFromJson, Debug)]
struct UserProfile {
    name: String,
    id: String,
}

#[derive(TryFromJson, Debug)]
pub struct UsernameHistoryEntry {
    pub name: String,
    // todo: rename;
    #[json(changedToAt)]
    pub changed_to_at: Option<i64>,
}

pub type UsernameHistory = Vec<UsernameHistoryEntry>;

#[derive(TryFromJson, Debug)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub properties: Vec<ProfileProperty>,
}

#[derive(TryFromJson, Debug)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
}

impl ProfileProperty {
    pub fn textures(&self) -> Result<TexturesEntry> {
        let decoded = base64::decode(&self.value).map_err(SkinError::Decoding)?;
        let read = Reader::new(decoded.into_iter())
            .read()
            .map_err(SkinError::Reader)?;
        Ok(read)
    }
}

#[derive(TryFromJson, Debug)]
pub struct TexturesEntry {
    pub timestamp: i64,
    #[json(profileId)]
    pub profile_id: String,
    #[json(profileName)]
    pub profile_name: String,
    pub textures: Textures,
}

#[derive(TryFromJson, DebugToJson)]
pub struct Textures {
    #[json(SKIN)]
    pub skin: SkinData,
    #[json(CAPE)]
    pub cape: Option<CapeData>,
}

#[derive(TryFromJson, DebugToJson)]
pub struct SkinData {
    pub url: String,
    pub metadata: Option<SkinMetadata>,
}
#[derive(TryFromJson, DebugToJson)]
pub struct CapeData {
    pub url: String,
}

#[derive(TryFromJson, DebugToJson)]
pub struct SkinMetadata {
    pub model: String,
}

pub type Result<T> = std::result::Result<T, ApiError>;

fn fetch<U: Into<minreq::URL>>(url: U) -> Result<String> {
    let res = minreq::get(url).send().map_err(ApiError::Fetch)?;
    if res.status_code == 200 {
        Ok(res.as_str().unwrap().to_owned())
    } else {
        Err(ApiError::Request {
            status: res.status_code,
            reason: res.reason_phrase,
        })
    }
}

fn api_get<U: Into<minreq::URL>, T: TryFromJson>(url: U) -> Result<T> {
    let res = minreq::get(url).send_lazy().map_err(ApiError::Fetch)?;
    if res.status_code == 200 {
        let bytes = res.map(|b| b.unwrap().0);
        Ok(Reader::new(bytes).read()?)
    } else {
        Err(ApiError::Request {
            status: res.status_code,
            reason: res.reason_phrase,
        })
    }
}

pub fn get_id_at(username: &str, at: i64) -> Result<String> {
    let url = format!(
        "https://api.mojang.com/users/profiles/minecraft/{}?at={}",
        username, at
    );
    let profile: UserProfile = api_get(url)?;
    Ok(profile.id)
}

pub fn get_username_history(id: String) -> Result<UsernameHistory> {
    let url = format!("https://api.mojang.com/user/profiles/{}/names", id);
    api_get(url)
}

pub fn get_skin(id: String) -> Result<Profile> {
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}",
        id
    );
    api_get(url)
}

// this is implemented naively to be more complient with the algorithm
fn is_ipv4_address(address: &str) -> bool {
    address.split('.').all(|x| x.parse::<u8>().is_ok())
}

// todo: iso8859-1 encoding
pub fn find_blocked_pattern(address: &str) -> Result<Option<String>> {
    let res = fetch("https://sessionserver.mojang.com/blockedservers")?;
    // let encoded_address = textcode::iso8859_1::encode_to_vec(&address);

    let address_parts: Vec<&str> = address.split(|b| b == '.').collect();
    let len = address_parts.len();

    let wildcards: Vec<String> = if is_ipv4_address(&address) {
        (0..len)
            .map(|i| format!("{}.*", address_parts[..i - 1].join(".")))
            .collect()
    } else {
        (0..len)
            .map(|i| format!("*.{}", address_parts[i..].join(".")))
            .collect()
    };

    let sha_wildcards: Vec<String> = wildcards
        .iter()
        .map(|addr| hex::encode(Sha1::digest(&addr.as_bytes())))
        .collect();

    let sha_address = hex::encode(Sha1::digest(&address.as_bytes()));
    let blocked_hashes: Vec<_> = res.lines().collect();

    let is_blocked = blocked_hashes
        .iter()
        .find(|&hash| *hash == sha_address)
        .map(|_v| sha_address)
        .or_else(|| {
            sha_wildcards
                .iter()
                .position(|sha_addr| blocked_hashes.iter().any(|&hash| sha_addr == hash))
                .map(|s| wildcards[s].clone())
        });

    Ok(is_blocked)
}
