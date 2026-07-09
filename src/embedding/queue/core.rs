//! In-process async channel queue for embedding requests.

use async_channel::{bounded, Receiver, Sender};

use super::types::EmbeddingRequest;
use crate::error::{EngramError, Result};
use crate::types::MemoryId;

/// Embedding queue for async processing
pub struct EmbeddingQueue {
    sender: Sender<EmbeddingRequest>,
    receiver: Receiver<EmbeddingRequest>,
    batch_size: usize,
}

impl EmbeddingQueue {
    /// Create a new embedding queue
    pub fn new(batch_size: usize) -> Self {
        let (sender, receiver) = bounded(10000); // Buffer up to 10k requests
        Self {
            sender,
            receiver,
            batch_size,
        }
    }

    /// Queue a memory for embedding
    pub async fn queue(&self, memory_id: MemoryId, content: String) -> Result<()> {
        self.sender
            .send(EmbeddingRequest { memory_id, content })
            .await
            .map_err(|e| EngramError::Embedding(format!("Queue send error: {}", e)))?;
        Ok(())
    }

    /// Queue a memory (blocking version for sync contexts)
    pub fn queue_blocking(&self, memory_id: MemoryId, content: String) -> Result<()> {
        self.sender
            .send_blocking(EmbeddingRequest { memory_id, content })
            .map_err(|e| EngramError::Embedding(format!("Queue send error: {}", e)))?;
        Ok(())
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.receiver.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }

    /// Get receiver for worker
    pub fn receiver(&self) -> Receiver<EmbeddingRequest> {
        self.receiver.clone()
    }
}

impl Clone for EmbeddingQueue {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
            batch_size: self.batch_size,
        }
    }
}
