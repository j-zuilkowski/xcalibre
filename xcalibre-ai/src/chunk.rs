// stub — implemented in rmp08b
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    pub max_tokens: usize,
    pub overlap_tokens: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self { max_tokens: 512, overlap_tokens: 64 }
    }
}

pub fn chunk_text(_text: &str, _config: &ChunkConfig) -> Vec<String> {
    unimplemented!("rmp08b")
}
