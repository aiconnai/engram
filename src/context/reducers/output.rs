use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Shared output contract for deterministic operational reducers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReducerOutput {
    pub summary: String,
    pub observed_facts: Vec<ObservedFact>,
    pub warnings: Vec<String>,
    pub redactions: Vec<RedactionRecord>,
    pub lossy: bool,
    pub confidence: f32,
    pub raw_required_for_full_debug: bool,
    pub preserved_evidence: Vec<EvidenceItem>,
}

impl ReducerOutput {
    pub(crate) fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            observed_facts: Vec::new(),
            warnings: Vec::new(),
            redactions: Vec::new(),
            lossy: false,
            confidence: 1.0,
            raw_required_for_full_debug: false,
            preserved_evidence: Vec::new(),
        }
    }

    pub(crate) fn add_fact(&mut self, kind: impl Into<String>, value: impl Into<String>) {
        self.observed_facts.push(ObservedFact {
            kind: kind.into(),
            value: value.into(),
            metadata: BTreeMap::new(),
        });
    }

    pub(crate) fn add_fact_with_metadata(
        &mut self,
        kind: impl Into<String>,
        value: impl Into<String>,
        metadata: BTreeMap<String, String>,
    ) {
        self.observed_facts.push(ObservedFact {
            kind: kind.into(),
            value: value.into(),
            metadata,
        });
    }

    pub(crate) fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    pub(crate) fn add_evidence(&mut self, item: impl Into<String>, preserved: bool) {
        self.preserved_evidence.push(EvidenceItem {
            item: item.into(),
            preserved,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedFact {
    pub kind: String,
    pub value: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub item: String,
    pub preserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionRecord {
    pub kind: String,
    pub replacement: String,
    pub original_len: usize,
}
