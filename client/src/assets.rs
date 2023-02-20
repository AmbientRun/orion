use std::{borrow::BorrowMut, marker::PhantomData, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use elements_asset_cache::{AssetCache, AsyncAssetKey, SyncAssetKey, SyncAssetKeyExt};
use futures::Future;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Semaphore;

pub type AssetResult<T> = Result<T, AssetError>;

#[derive(Clone, Error)]
#[error(transparent)]
pub struct AssetError(Arc<anyhow::Error>);

impl From<anyhow::Error> for AssetError {
    fn from(err: anyhow::Error) -> Self {
        Self(Arc::new(err))
    }
}
impl std::fmt::Debug for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct AssertErrorString(pub String);
impl From<AssetError> for AssertErrorString {
    fn from(err: AssetError) -> Self {
        Self(format!("{err:#}"))
    }
}
impl From<anyhow::Error> for AssertErrorString {
    fn from(err: anyhow::Error) -> Self {
        Self(format!("{err:#}"))
    }
}

#[derive(Clone, Debug)]
pub struct AssetsCacheDir;
impl SyncAssetKey<PathBuf> for AssetsCacheDir {
    fn load(&self, _assets: AssetCache) -> PathBuf {
        std::env::current_dir().unwrap().join("tmp")
    }
}

#[derive(Clone, Debug)]
pub struct ReqwestClientKey;
impl SyncAssetKey<reqwest::Client> for ReqwestClientKey {
    fn load(&self, _assets: AssetCache) -> reqwest::Client {
        reqwest::Client::new()
    }
}

/// Download with retries and a global rate limiting sempahore
pub async fn download<T, F: Future<Output = anyhow::Result<T>>>(
    assets: &AssetCache,
    url: impl reqwest::IntoUrl,
    map: impl Fn(reqwest::Response) -> F,
) -> anyhow::Result<T> {
    let client = ReqwestClientKey.get(assets);
    let url_str = url.as_str().to_string();
    let url_short = if url_str.len() > 200 {
        format!("{}...", &url_str[..200])
    } else {
        url_str.to_string()
    };
    let url: Url = url.into_url()?;

    let max_retries = 12;
    for i in 0..max_retries {
        let semaphore = DownloadSemaphore.get(assets);
        tracing::info!("download [pending ] {}", url_short);
        let _permit = semaphore.acquire().await.unwrap();
        tracing::info!("download [download] {}", url_short);
        let resp = client
            .get(url.clone())
            .send()
            .await
            .with_context(|| format!("Failed to download {url_str}"))?;
        if !resp.status().is_success() {
            tracing::warn!("Request for {} failed: {:?}", url_str, resp.status());
            return Err(anyhow!(
                "Downloading {url_str} failed, bad status code: {:?}",
                resp.status()
            ));
        }
        match map(resp).await {
            Ok(res) => {
                tracing::info!("download [complete] {}", url_short);
                return Ok(res);
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to read body of {url_str}, retrying ({i}/{max_retries}): {:?}",
                    err
                );
                tokio::time::sleep(Duration::from_millis(2u64.pow(i))).await;
            }
        }
    }
    Err(anyhow::anyhow!("Failed to download body of {}", url_str))
}

#[derive(Clone, Debug)]
pub struct BytesFromUrl {
    pub url: String,
    pub cache_on_disk: bool,
}
impl BytesFromUrl {
    pub fn new(url: String, cache_on_disk: bool) -> Self {
        Self { url, cache_on_disk }
    }
    pub fn parse_url(url: impl AsRef<str>, cache_on_disk: bool) -> anyhow::Result<Self> {
        Ok(Self {
            url: url.as_ref().into(),
            cache_on_disk,
        })
    }
}
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<Vec<u8>>>> for BytesFromUrl {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<Vec<u8>>> {
        let body = download(&assets, self.url.clone(), |resp| async {
            Ok(resp.bytes().await?)
        })
        .await?
        .to_vec();
        assert!(!body.is_empty());
        Ok(Arc::new(body))
    }
    fn cpu_size(&self, value: &AssetResult<Arc<Vec<u8>>>) -> Option<usize> {
        value.as_ref().ok().map(|v| v.len())
    }
}

/// Limit the number of concurent file reads to 10
#[derive(Debug)]
struct FileReadSemaphore;
impl SyncAssetKey<Arc<Semaphore>> for FileReadSemaphore {
    fn load(&self, _assets: AssetCache) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(10))
    }
}

/// Limit the number of concurent downloads to 5
#[derive(Debug)]
struct DownloadSemaphore;
impl SyncAssetKey<Arc<Semaphore>> for DownloadSemaphore {
    fn load(&self, _assets: AssetCache) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(5))
    }
}
