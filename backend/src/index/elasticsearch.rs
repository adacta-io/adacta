use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use elasticsearch::http::transport::Transport;
use elasticsearch::{Elasticsearch, IndexParts, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::value::{RawValue, Value};

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::config::ElasticsearchIndex as Config;
use crate::index::SearchResponse;
use crate::model::DocId;
use crate::repo::Bundle;

const DOCUMENT_TYPE: &str = "document";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Source {
    text: String,
    uploaded: DateTime<Utc>,
    archived: Option<DateTime<Utc>>,
    tags: HashSet<String>,
    properties: HashMap<String, String>,
}

pub struct Index {
    client: Elasticsearch,

    index: String,
}

impl Index {
    pub async fn from_config(config: Config) -> Result<Self> {
        return Self::connect(config.url, config.index).await;
    }

    pub async fn connect(url: String, index: String) -> Result<Self> {
        let transport = Transport::single_node(&url)?;
        let client = Elasticsearch::new(transport);

        client.ping().send().await?;

        return Ok(Self { client, index });
    }

    async fn query(&self, mut query: Value) -> Result<SearchResponse> {
        // Enable exact hit count
        query["track_total_hits"] = true.into();

        // Execute the query
        let response = self
            .client
            .search(SearchParts::IndexType(&[&self.index], &[DOCUMENT_TYPE]))
            .body(query)
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(anyhow!(
                "ElasticSearch Query error: {}",
                response.read_body::<Box<RawValue>>().await?
            ));
        }

        let response = response.read_body::<Value>().await?;

        let count = response["hits"]["total"]["value"]
            .as_u64()
            .expect("no usize");

        let docs = response["hits"]["hits"]
            .as_array()
            .expect("no array")
            .iter()
            .map(|hit| hit["_id"].as_str().expect("no atr"))
            .map(DocId::from_str)
            .collect::<Result<Vec<_>>>()?;

        return Ok(SearchResponse { count, docs });
    }
}

#[async_trait]
impl super::Index for Index {
    async fn index(&self, bundle: &Bundle) -> Result<()> {
        let id = bundle.id().to_string();

        let text = bundle
            .plaintext()
            .await
            .transpose()?
            .ok_or_else(|| anyhow!("Bundle does not contain plaintext"))?;
        let meta = bundle
            .metadata()
            .await
            .transpose()?
            .ok_or_else(|| anyhow!("Bundle does not contain meta-data"))?;

        self.client
            .index(IndexParts::IndexTypeId(&self.index, DOCUMENT_TYPE, &id))
            .body(Source {
                text,
                uploaded: meta.uploaded,
                archived: meta.archived,
                tags: meta.tags,
                properties: meta.properties,
            })
            .send()
            .await?;

        return Ok(());
    }

    async fn search(&self, query: &str) -> Result<SearchResponse> {
        return self
            .query(json!({
                "query": {
                    "bool" : {
                        "must" : {
                            "simple_query_string" : {
                                "query" : query
                            }
                        }
                    }
                }
            }))
            .await;
    }

    async fn inbox(&self) -> Result<SearchResponse> {
        return self
            .query(json!({
                "query": {
                    "bool": {
                        "must_not": {
                            "exists": {
                                "field": "archived"
                            }
                        }
                    }
                }
            }))
            .await;
    }
}
