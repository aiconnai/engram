use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;

use crate::error::{EngramError, Result};

pub(super) enum CloudBackend {
    S3(S3Client),
    #[cfg(test)]
    Fixture(InMemoryCloudStore),
}

#[derive(Debug, Clone)]
pub(super) struct CloudObject {
    pub(super) body: Vec<u8>,
    pub(super) metadata: HashMap<String, String>,
    pub(super) size: u64,
    pub(super) last_modified: Option<String>,
    pub(super) etag: Option<String>,
}

pub(super) enum UploadCondition {
    DoesNotExist,
    Matches(String),
}

impl CloudBackend {
    pub(super) async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        metadata: HashMap<String, String>,
        condition: UploadCondition,
    ) -> Result<()> {
        match self {
            CloudBackend::S3(client) => {
                let request = client
                    .put_object()
                    .bucket(bucket)
                    .key(key)
                    .set_metadata((!metadata.is_empty()).then_some(metadata))
                    .body(ByteStream::from(body));
                let request = match condition {
                    UploadCondition::DoesNotExist => request.if_none_match("*"),
                    UploadCondition::Matches(etag) => request.if_match(etag),
                };
                request
                    .send()
                    .await
                    .map_err(|e| EngramError::CloudStorage(e.to_string()))?;
                Ok(())
            }
            #[cfg(test)]
            CloudBackend::Fixture(store) => store.put(
                CloudObject {
                    size: body.len() as u64,
                    body,
                    metadata,
                    last_modified: None,
                    etag: None,
                },
                condition,
            ),
        }
    }

    pub(super) async fn get_object(&self, bucket: &str, key: &str) -> Result<CloudObject> {
        match self {
            CloudBackend::S3(client) => {
                let response = client
                    .get_object()
                    .bucket(bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| EngramError::CloudStorage(e.to_string()))?;
                let metadata = response.metadata().cloned().unwrap_or_default();
                let etag = response.e_tag().map(String::from);
                let body = response
                    .body
                    .collect()
                    .await
                    .map_err(|e| EngramError::CloudStorage(e.to_string()))?
                    .into_bytes()
                    .to_vec();
                Ok(CloudObject {
                    size: body.len() as u64,
                    body,
                    metadata,
                    last_modified: None,
                    etag,
                })
            }
            #[cfg(test)]
            CloudBackend::Fixture(store) => store.get().ok_or_else(|| {
                EngramError::CloudStorage(format!("object not found at s3://{bucket}/{key}"))
            }),
        }
    }

    pub(super) async fn head_object(&self, bucket: &str, key: &str) -> Result<CloudObject> {
        match self {
            CloudBackend::S3(client) => {
                let response = client
                    .head_object()
                    .bucket(bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| EngramError::CloudStorage(e.to_string()))?;
                Ok(CloudObject {
                    body: Vec::new(),
                    metadata: response.metadata().cloned().unwrap_or_default(),
                    size: response.content_length().unwrap_or(0) as u64,
                    last_modified: response.last_modified().map(|dt| dt.to_string()),
                    etag: response.e_tag().map(String::from),
                })
            }
            #[cfg(test)]
            CloudBackend::Fixture(store) => store.get().ok_or_else(|| {
                EngramError::CloudStorage(format!("object not found at s3://{bucket}/{key}"))
            }),
        }
    }

    pub(super) async fn delete_object(&self, bucket: &str, key: &str) -> Result<()> {
        match self {
            CloudBackend::S3(client) => {
                client
                    .delete_object()
                    .bucket(bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| EngramError::CloudStorage(e.to_string()))?;
                Ok(())
            }
            #[cfg(test)]
            CloudBackend::Fixture(store) => {
                store.delete();
                Ok(())
            }
        }
    }

    pub(super) async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool> {
        match self {
            CloudBackend::S3(client) => {
                match client.head_object().bucket(bucket).key(key).send().await {
                    Ok(_) => Ok(true),
                    Err(e) => {
                        let service_error = e.into_service_error();
                        if service_error.is_not_found() {
                            Ok(false)
                        } else {
                            Err(EngramError::CloudStorage(service_error.to_string()))
                        }
                    }
                }
            }
            #[cfg(test)]
            CloudBackend::Fixture(store) => Ok(store.get().is_some()),
        }
    }
}

#[cfg(test)]
#[derive(Clone, Default)]
pub(super) struct InMemoryCloudStore {
    object: Arc<Mutex<Option<CloudObject>>>,
    revision: Arc<std::sync::atomic::AtomicU64>,
}

#[cfg(test)]
impl InMemoryCloudStore {
    pub(super) fn put(&self, mut object: CloudObject, condition: UploadCondition) -> Result<()> {
        let mut stored = self
            .object
            .lock()
            .expect("test fixture lock is not poisoned");
        let condition_matches = match (&condition, stored.as_ref()) {
            (UploadCondition::DoesNotExist, None) => true,
            (UploadCondition::Matches(expected), Some(current)) => {
                current.etag.as_deref() == Some(expected.as_str())
            }
            _ => false,
        };
        if !condition_matches {
            return Err(EngramError::CloudStorage(
                "conditional cloud upload rejected stale object state".to_string(),
            ));
        }
        let revision = self
            .revision
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        object.etag = Some(format!("fixture-{revision}"));
        *stored = Some(object);
        Ok(())
    }

    pub(super) fn get(&self) -> Option<CloudObject> {
        self.object
            .lock()
            .expect("test fixture lock is not poisoned")
            .clone()
    }

    pub(super) fn delete(&self) {
        *self
            .object
            .lock()
            .expect("test fixture lock is not poisoned") = None;
    }

    pub(super) fn snapshot(&self) -> Option<CloudObject> {
        self.get()
    }

    pub(super) fn remove_metadata(&self, key: &str) {
        if let Some(object) = self
            .object
            .lock()
            .expect("test fixture lock is not poisoned")
            .as_mut()
        {
            object.metadata.remove(key);
        }
    }

    pub(super) fn clear_metadata(&self) {
        if let Some(object) = self
            .object
            .lock()
            .expect("test fixture lock is not poisoned")
            .as_mut()
        {
            object.metadata.clear();
        }
    }
}
