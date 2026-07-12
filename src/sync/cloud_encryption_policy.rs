use super::*;

impl CloudStorage {
    pub(super) fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let provider = self.configured_key_provider()?;
        encrypt_data_with_provider(provider, data)
    }

    pub(super) fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let provider = self.configured_key_provider()?;
        decrypt_data_with_provider(provider, data)
    }

    pub(super) fn configured_key_provider(&self) -> Result<&ConfiguredCloudKeyProvider> {
        self.key_provider.as_ref().ok_or_else(|| {
            EngramError::Encryption(
                "cloud encryption requires a configured durable key provider".to_string(),
            )
        })
    }

    pub(super) fn encryption_object_metadata(&self) -> Result<HashMap<String, String>> {
        let metadata = self.encryption_rotation_metadata().ok_or_else(|| {
            EngramError::Encryption("cloud encryption requires rotation metadata".to_string())
        })?;
        Ok(HashMap::from([
            (
                ENCRYPTION_KEY_ID_METADATA.to_string(),
                metadata.active_key_id.clone(),
            ),
            (
                ENCRYPTION_FORMAT_VERSION_METADATA.to_string(),
                metadata.format_version.to_string(),
            ),
            (
                ENCRYPTION_ALGORITHM_METADATA.to_string(),
                metadata.algorithm.to_string(),
            ),
        ]))
    }

    pub(super) async fn ensure_remote_object_is_replaceable(&self) -> Result<UploadCondition> {
        if !self.backend.object_exists(&self.bucket, &self.key).await? {
            return Ok(UploadCondition::DoesNotExist);
        }

        let object = self.backend.get_object(&self.bucket, &self.key).await?;
        if self.encrypt {
            self.decrypt_encrypted_object(&object)?;
        } else if Self::has_encryption_identity(&object) {
            return self.reject_encryption_audit(
                "encrypted cloud object requires an encryption key; refusing plaintext overwrite",
            );
        } else if !Self::is_known_plaintext_object(&object) {
            return self.reject_encryption_audit(
                "remote cloud object format is unidentified; refusing plaintext overwrite",
            );
        }
        object.etag.map(UploadCondition::Matches).ok_or_else(|| {
            EngramError::CloudStorage(
                "remote cloud object has no version identifier; refusing unconditional overwrite"
                    .to_string(),
            )
        })
    }

    pub(super) fn has_encryption_identity(object: &CloudObject) -> bool {
        object.metadata.contains_key(ENCRYPTION_KEY_ID_METADATA)
            || object
                .metadata
                .contains_key(ENCRYPTION_FORMAT_VERSION_METADATA)
            || object.metadata.contains_key(ENCRYPTION_ALGORITHM_METADATA)
            || is_versioned_encrypted_payload(&object.body)
    }

    pub(super) fn is_known_plaintext_object(object: &CloudObject) -> bool {
        object
            .metadata
            .get(OBJECT_FORMAT_METADATA)
            .is_some_and(|value| value == PLAINTEXT_OBJECT_FORMAT)
            || (object.metadata.is_empty() && object.body.starts_with(SQLITE_HEADER))
    }

    pub(super) fn decrypt_encrypted_object(&self, object: &CloudObject) -> Result<Vec<u8>> {
        let result = if object.metadata.is_empty() {
            tracing::warn!(
                bucket = %self.bucket,
                key = %self.key,
                "Encrypted cloud object uses embedded key identity without object metadata"
            );
            self.decrypt_data(&object.body)
        } else {
            self.audit_encryption_metadata(&object.metadata)
                .and_then(|key_id| {
                    let provider = self.configured_key_provider()?;
                    decrypt_data_with_provider_for_key_id(provider, &object.body, key_id)
                })
        };
        if let Err(error) = &result {
            tracing::warn!(
                bucket = %self.bucket,
                key = %self.key,
                error = %error,
                "Rejected encrypted cloud object"
            );
        }
        result
    }

    pub(super) fn audit_encryption_metadata<'a>(
        &self,
        metadata: &'a HashMap<String, String>,
    ) -> Result<&'a str> {
        let provider = self.configured_key_provider()?;
        let Some(key_id) = metadata.get(ENCRYPTION_KEY_ID_METADATA) else {
            return Err(EngramError::Encryption(
                "missing encryption key id metadata; refusing encrypted download".to_string(),
            ));
        };
        let Some(format_version) = metadata.get(ENCRYPTION_FORMAT_VERSION_METADATA) else {
            return Err(EngramError::Encryption(
                "missing encryption format version metadata; refusing encrypted download"
                    .to_string(),
            ));
        };
        let expected_version = provider.rotation_metadata().format_version.to_string();
        if format_version != &expected_version {
            return Err(EngramError::Encryption(format!(
                "encrypted object format version {format_version} does not match configured version {expected_version}; refusing encrypted download"
            )));
        }
        let Some(algorithm) = metadata.get(ENCRYPTION_ALGORITHM_METADATA) else {
            return Err(EngramError::Encryption(
                "missing encryption algorithm metadata; refusing encrypted download".to_string(),
            ));
        };
        let expected_algorithm = provider.rotation_metadata().algorithm;
        if algorithm != expected_algorithm {
            return Err(EngramError::Encryption(format!(
                "encrypted object algorithm {algorithm} does not match configured algorithm {expected_algorithm}; refusing encrypted download"
            )));
        }
        if !provider.can_decrypt_key_id(key_id) {
            return Err(EngramError::Encryption(format!(
                "encrypted object key id {key_id} is not configured for decryption; refusing encrypted download"
            )));
        }
        Ok(key_id)
    }

    pub(super) fn reject_encryption_audit<T>(&self, message: &str) -> Result<T> {
        tracing::warn!(
            bucket = %self.bucket,
            key = %self.key,
            reason = message,
            "Rejected encrypted cloud object"
        );
        Err(EngramError::Encryption(message.to_string()))
    }
}
