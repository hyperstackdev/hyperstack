use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::collections::HashMap;
use std::fmt::Display;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};
use xxhash_rust::xxh64::xxh64;

pub struct VectorStore<T: Display + Clone> {
    index: Index,
    metadata_store: HashMap<u64, T>,
    embedding_model: TextEmbedding,
}

impl<T: Display + Clone> VectorStore<T> {
    pub fn new(capacity: usize) -> Result<Self> {
        let embedding_model =
            TextEmbedding::try_new(InitOptions::new(EmbeddingModel::JinaEmbeddingsV2BaseCode))?;
        let index = Index::new(&IndexOptions {
            dimensions: 768,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            ..Default::default()
        })?;
        index.reserve(capacity)?;

        Ok(Self {
            index,
            metadata_store: HashMap::new(),
            embedding_model,
        })
    }

    pub fn embed(&mut self, item: T) -> Result<u64> {
        let node_hash: u64 = xxh64(format!("{}", item.to_string()).as_bytes(), 0);
        let embedding = &self.embedding_model.embed(vec![item.to_string()], None)?[0];
        self.index.add(node_hash, embedding)?;
        self.metadata_store.insert(node_hash, item);
        self.index.save("index.usearch")?;

        Ok(node_hash)
    }

    pub fn query(&mut self, q: String, limit: usize) -> Result<Vec<T>> {
        let mut items: Vec<T> = vec![];
        let query_embedding = &self.embedding_model.embed(vec![q], None)?[0];
        let matches = self.index.search(&query_embedding, limit)?;

        for key in matches.keys.iter() {
            if let Some(item) = self.metadata_store.get(&key) {
                items.push(item.clone());
            }
        }

        Ok(items)
    }
}
