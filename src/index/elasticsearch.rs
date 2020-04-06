use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use elasticsearch::{Elasticsearch, IndexParts, SearchParts};
use elasticsearch::http::transport::Transport;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::config::ElasticsearchIndexConfig;
use crate::model::DocId;
use crate::repo::Bundle;

const DOCUMENT_TYPE: &'static str = "document";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Source {
    text: String,
    uploaded: DateTime<Utc>,
    reviewed: Option<DateTime<Utc>>,
    tags: HashSet<String>,
    properties: HashMap<String, String>,
}

pub struct Index {
    client: Elasticsearch,

    index: String,
}

impl Index {
    pub async fn from_config(config: ElasticsearchIndexConfig) -> Result<Self> {
        return Self::connect(config.url, config.index).await;
    }

    pub async fn connect(url: String,
                         index: String) -> Result<Self> {
        let transport = Transport::single_node(&url)?;
        let client = Elasticsearch::new(transport);

        return Ok(Self {
            client,
            index,
        });
    }

    async fn query(&self, query: Value) -> Result<Value> {
        let response = self.client.search(SearchParts::IndexType(&[&self.index], &[DOCUMENT_TYPE]))
            .body(query)
            .send()
            .await?
            .error_for_status_code()?;

        let response = response.read_body::<Value>().await?;

        return Ok(response);
    }
}

#[async_trait]
impl super::Index for Index {
    async fn index(&self, bundle: &Bundle) -> Result<()> {
        let id = bundle.id().to_string();

        let text = bundle.plaintext().await.transpose()?
            .ok_or_else(|| anyhow!("Bundle does not contain plaintext"))?;
        let meta = bundle.metadata().await.transpose()?
            .ok_or_else(|| anyhow!("Bundle does not contain meta-data"))?;

        self.client.index(IndexParts::IndexTypeId(&self.index, DOCUMENT_TYPE, &id))
            .body(Source {
                text,
                uploaded: meta.uploaded,
                reviewed: meta.reviewed,
                tags: meta.tags,
                properties: meta.properties,
            })
            .send()
            .await?;

        return Ok(());
    }

    async fn search(&self, query: &str) -> Result<Vec<DocId>> {
        let response = self.query(json!({
                "query": {
                    "bool" : {
                        "must" : {
                            "query_string" : {
                                "query" : query
                            }
                        }
                    }
                }
            })).await?;

        let hits = response["hits"]["hits"].as_array().expect("no array")
            .into_iter()
            .map(|hit| hit["id"].as_str().expect("no atr"))
            .map(DocId::from_str)
            .collect::<Result<Vec<_>>>()?;

        return Ok(hits);
    }

    async fn inbox(&self) -> Result<Vec<DocId>> {
        unimplemented!()
    }
}
