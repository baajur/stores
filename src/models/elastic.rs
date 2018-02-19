#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Shards {
    total: u32,
    successful: u32,
    failed: u32,
}

#[derive(Deserialize, Debug)]
pub struct IndexResponse {
    #[serde(rename = "_shards")] shards: Shards,
    #[serde(rename = "_index")] index: String,
    #[serde(rename = "_type")] ty: String,
    #[serde(rename = "_id")] id: String,
    #[serde(rename = "_version")] version: Option<u32>,
    #[serde(rename = "_seq_no")] seq_no: Option<u32>,
    #[serde(rename = "_primary_term")] primary_term: Option<u32>,
    result: String,
}

impl IndexResponse {
    pub fn created(&self) -> bool {
        self.result == "created".to_string()
    }
}
