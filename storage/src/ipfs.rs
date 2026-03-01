use crate::error::{StorageError, StorageResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;

const DEFAULT_IPFS_API: &str = "http://127.0.0.1:5001";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Clone, Debug)]
pub struct IpfsConfig {
    pub api_url: String,
    pub timeout: Duration,
    pub pin_content: bool,
}

impl Default for IpfsConfig {
    fn default() -> Self {
        Self {
            api_url: DEFAULT_IPFS_API.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            pin_content: true,
        }
    }
}

#[derive(Clone)]
pub struct IpfsClient {
    config: IpfsConfig,
    #[cfg(not(target_arch = "wasm32"))]
    http_client: reqwest::Client,
}

impl IpfsClient {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(config: IpfsConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, http_client }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub fn default() -> Self {
        Self::new(IpfsConfig::default())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn add(&self, data: &[u8]) -> StorageResult<String> {
        let url = format!("{}/api/v0/add", self.config.api_url);
        
        let part = reqwest::multipart::Part::bytes(data.to_vec())
            .file_name("data")
            .mime_str("application/octet-stream")
            .map_err(|e| StorageError::Ipfs(e.to_string()))?;
        
        let form = reqwest::multipart::Form::new()
            .part("file", part);
        
        let response = self.http_client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| StorageError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(StorageError::Ipfs(format!("Add failed: {}", error)));
        }
        
        let result: AddResponse = response
            .json()
            .await
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        if self.config.pin_content {
            self.pin(&result.hash).await?;
        }
        
        Ok(result.hash)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get(&self, cid: &str) -> StorageResult<Vec<u8>> {
        let url = format!("{}/api/v0/cat?arg={}", self.config.api_url, cid);
        
        let response = self.http_client
            .post(&url)
            .send()
            .await
            .map_err(|e| StorageError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(StorageError::Ipfs(format!("Cat failed: {}", error)));
        }
        
        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| StorageError::Network(e.to_string()))
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn pin(&self, cid: &str) -> StorageResult<()> {
        let url = format!("{}/api/v0/pin/add?arg={}", self.config.api_url, cid);
        
        let response = self.http_client
            .post(&url)
            .send()
            .await
            .map_err(|e| StorageError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(StorageError::Ipfs(format!("Pin failed: {}", error)));
        }
        
        Ok(())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn unpin(&self, cid: &str) -> StorageResult<()> {
        let url = format!("{}/api/v0/pin/rm?arg={}", self.config.api_url, cid);
        
        let response = self.http_client
            .post(&url)
            .send()
            .await
            .map_err(|e| StorageError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(StorageError::Ipfs(format!("Unpin failed: {}", error)));
        }
        
        Ok(())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_peers(&self) -> StorageResult<Vec<String>> {
        let url = format!("{}/api/v0/swarm/peers", self.config.api_url);
        
        let response = self.http_client
            .post(&url)
            .send()
            .await
            .map_err(|e| StorageError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Ok(Vec::new());
        }
        
        let result: PeersResponse = response
            .json()
            .await
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        Ok(result.peers.into_iter().map(|p| p.peer).collect())
    }
    
    pub fn compute_cid(data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        let multihash = Self::to_multihash(&hash);
        format!("Qm{}", base58_encode(&multihash))
    }
    
    fn to_multihash(hash: &[u8]) -> Vec<u8> {
        let mut result = vec![0x12, 0x20];
        result.extend_from_slice(hash);
        result
    }
}

#[cfg(target_arch = "wasm32")]
impl IpfsClient {
    pub fn new(config: IpfsConfig) -> Self {
        Self { config }
    }
    
    pub fn default() -> Self {
        Self::new(IpfsConfig::default())
    }
    
    pub async fn add(&self, _data: &[u8]) -> StorageResult<String> {
        Err(StorageError::Ipfs("IPFS not available in WASM".into()))
    }
    
    pub async fn get(&self, _cid: &str) -> StorageResult<Vec<u8>> {
        Err(StorageError::Ipfs("IPFS not available in WASM".into()))
    }
    
    pub async fn pin(&self, _cid: &str) -> StorageResult<()> {
        Err(StorageError::Ipfs("IPFS not available in WASM".into()))
    }
    
    pub async fn unpin(&self, _cid: &str) -> StorageResult<()> {
        Err(StorageError::Ipfs("IPFS not available in WASM".into()))
    }
    
    pub async fn get_peers(&self) -> StorageResult<Vec<String>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AddResponse {
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Size")]
    size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PeersResponse {
    #[serde(rename = "Peers")]
    peers: Vec<PeerInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PeerInfo {
    #[serde(rename = "Peer")]
    peer: String,
}

fn base58_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    
    let mut num = 0u128;
    for &byte in data {
        num = num * 256 + byte as u128;
    }
    
    let mut result = Vec::new();
    while num > 0 {
        result.push(ALPHABET[(num % 58) as usize]);
        num /= 58;
    }
    
    for &byte in data {
        if byte == 0 {
            result.push(ALPHABET[0]);
        } else {
            break;
        }
    }
    
    result.reverse();
    String::from_utf8(result).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compute_cid() {
        let data = b"Hello, IPFS!";
        let cid = IpfsClient::compute_cid(data);
        assert!(cid.starts_with('Q'));
        assert!(!cid.is_empty());
    }
    
    #[test]
    fn test_base58_encode() {
        let data = b"test";
        let encoded = base58_encode(data);
        assert!(!encoded.is_empty());
    }
}
