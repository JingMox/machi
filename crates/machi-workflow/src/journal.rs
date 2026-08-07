//! Append-only host-call journal for resume.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::MAX_HOST_CALLS;

/// Maximum journal file size in bytes.
pub const MAX_JOURNAL_BYTES: u64 = 64 * 1024 * 1024;

/// One recorded host call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "result contains JSON Value"
)]
pub struct JournalEntry {
    /// Dense sequence number starting at 0.
    pub seq: u64,
    /// Request kind.
    pub kind: String,
    /// Hash of canonical request payload.
    pub req_hash: String,
    /// Recorded result JSON.
    pub result: serde_json::Value,
    /// Host wall-clock ms (not used for script determinism).
    pub at_ms: u64,
}

/// Journal failures.
#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    /// I/O failure.
    #[error("journal io: {0}")]
    Io(#[from] std::io::Error),
    /// Parse failure.
    #[error("journal parse at line {line}: {error}")]
    Parse {
        /// Line number.
        line: usize,
        /// Parse error.
        error: String,
    },
    /// Sequence gap or mismatch.
    #[error("journal sequence error at index {index}: expected {expected}, found {actual}")]
    Sequence {
        /// Index.
        index: usize,
        /// Expected seq.
        expected: u64,
        /// Actual seq.
        actual: u64,
    },
    /// Replay saw a different request than recorded.
    #[error(
        "journal divergence at seq {seq} ({kind}): script issued a different call than the recorded run"
    )]
    Divergence {
        /// Sequence.
        seq: u64,
        /// Kind.
        kind: String,
    },
    /// Size or count limits.
    #[error("journal full: {0}")]
    Full(String),
}

/// In-memory journal with optional durable path.
#[derive(Debug, Default)]
pub struct Journal {
    entries: Vec<JournalEntry>,
    path: Option<PathBuf>,
}

impl Journal {
    /// Empty journal, optionally bound to a path for appends.
    #[must_use]
    pub const fn new(path: Option<PathBuf>) -> Self {
        Self {
            entries: Vec::new(),
            path,
        }
    }

    /// Load from jsonl path (missing file => empty).
    ///
    /// # Errors
    ///
    /// Returns parse/IO errors.
    pub fn load(path: PathBuf) -> Result<Self, JournalError> {
        if !path.exists() {
            return Ok(Self::new(Some(path)));
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: JournalEntry =
                serde_json::from_str(&line).map_err(|e| JournalError::Parse {
                    line: i.saturating_add(1),
                    error: e.to_string(),
                })?;
            let expected = u64::try_from(entries.len()).unwrap_or(u64::MAX);
            if entry.seq != expected {
                return Err(JournalError::Sequence {
                    index: entries.len(),
                    expected,
                    actual: entry.seq,
                });
            }
            if expected >= MAX_HOST_CALLS {
                return Err(JournalError::Full("too many entries".into()));
            }
            entries.push(entry);
        }
        Ok(Self {
            entries,
            path: Some(path),
        })
    }

    /// Number of entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Empty check.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether `seq` is already covered.
    #[must_use]
    pub fn covers(&self, seq: u64) -> bool {
        usize::try_from(seq).is_ok_and(|i| i < self.entries.len())
    }

    /// Replay a covered call or return `None` to execute live.
    ///
    /// # Errors
    ///
    /// Divergence when kind/hash mismatch.
    pub fn replay(
        &self,
        seq: u64,
        kind: &str,
        hash: &str,
    ) -> Result<Option<serde_json::Value>, JournalError> {
        let Ok(index) = usize::try_from(seq) else {
            return Ok(None);
        };
        let Some(entry) = self.entries.get(index) else {
            return Ok(None);
        };
        if entry.kind != kind || entry.req_hash != hash {
            return Err(JournalError::Divergence {
                seq,
                kind: kind.to_owned(),
            });
        }
        Ok(Some(entry.result.clone()))
    }

    /// Append a live result.
    ///
    /// # Errors
    ///
    /// Full journal or IO errors.
    pub fn record(
        &mut self,
        seq: u64,
        kind: &str,
        hash: String,
        result: serde_json::Value,
    ) -> Result<(), JournalError> {
        let expected = u64::try_from(self.entries.len()).unwrap_or(u64::MAX);
        if seq != expected {
            return Err(JournalError::Sequence {
                index: self.entries.len(),
                expected,
                actual: seq,
            });
        }
        if expected >= MAX_HOST_CALLS {
            return Err(JournalError::Full("max host calls reached".into()));
        }
        let entry = JournalEntry {
            seq,
            kind: kind.to_owned(),
            req_hash: hash,
            result,
            at_ms: unix_now_ms(),
        };
        if let Some(path) = &self.path {
            append_line(path, &entry)?;
        }
        self.entries.push(entry);
        Ok(())
    }
}

/// Hash a host request for divergence detection.
#[must_use]
pub fn request_hash(kind: &str, payload: &serde_json::Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    let canonical = serde_json::to_vec(payload).unwrap_or_default();
    hasher.update(canonical);
    encode_hex(&hasher.finalize())
}

fn encode_hex(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut s = String::with_capacity(bytes.len().saturating_mul(2));
    for &b in bytes {
        let hi = usize::from(b >> 4);
        let lo = usize::from(b & 0x0f);
        // SAFETY: hi/lo are always 0..=15.
        let h = HEX.get(hi).copied().unwrap_or(b'?');
        let l = HEX.get(lo).copied().unwrap_or(b'?');
        s.push(char::from(h));
        s.push(char::from(l));
    }
    s
}

fn append_line(path: &Path, entry: &JournalEntry) -> Result<(), JournalError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(entry).map_err(|e| JournalError::Parse {
        line: 0,
        error: e.to_string(),
    })?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

fn unix_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn replay_and_divergence() {
        let mut j = Journal::new(None);
        let hash = request_hash("spawn_agent", &json!({"prompt":"a"}));
        j.record(0, "spawn_agent", hash.clone(), json!({"ok":true}))
            .expect("record");
        let replayed = j
            .replay(0, "spawn_agent", &hash)
            .expect("replay")
            .expect("hit");
        assert_eq!(replayed.get("ok"), Some(&json!(true)));
        let err = j
            .replay(0, "spawn_agent", "deadbeef")
            .expect_err("divergence");
        assert!(matches!(err, JournalError::Divergence { .. }));
    }

    #[test]
    fn durable_round_trip() {
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("j.jsonl");
        let mut j = Journal::new(Some(path.clone()));
        let hash = request_hash("spawn_agent", &json!(1));
        j.record(0, "spawn_agent", hash, json!(42)).expect("rec");
        let loaded = Journal::load(path).expect("load");
        assert_eq!(loaded.len(), 1);
    }
}
