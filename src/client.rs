use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct ConsulXClient {
    pub http: Client,
    pub base: String,
}

impl ConsulXClient {
    pub fn new(base: &str) -> Result<Self> {
        Ok(Self {
            http: Client::new(),
            base: base.trim_end_matches('/').to_string(),
        })
    }


    /// Build from CONSUL_HTTP_ADDR (or default http://127.0.0.1:8500)
    pub fn from_env() -> Result<Self> {
        let addr = std::env::var("CONSUL_HTTP_ADDR")
            .unwrap_or_else(|_| "http://127.0.0.1:8500".to_string());
        Self::new(&addr)
    }


    /// GET /v1/kv/<key>?raw
    pub async fn kv_get_raw(&self, key: &str) -> Result<Option<String>> {
        let url = format!("{}/v1/kv/{}?raw=true", self.base, key);
        let resp = self.http.get(url).send().await?;

        if resp.status().is_success() {
            let txt = resp.text().await?;
            return Ok(Some(txt));
        }

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }

        Err(anyhow::anyhow!("GET failed with status {}", resp.status()))
    }

    /// PUT /v1/kv/<key>
    pub async fn kv_put(&self, key: &str, value: &str) -> Result<()> {
        let url = format!("{}/v1/kv/{}", self.base, key);
        let resp = self.http.put(url).body(value.to_string()).send().await?;

        if resp.status().is_success() {
            return Ok(());
        }

        Err(anyhow::anyhow!("PUT failed with status {}", resp.status()))
    }

    /// DELETE /v1/kv/<key>
    pub async fn kv_delete(&self, key: &str) -> Result<()> {
        let url = format!("{}/v1/kv/{}", self.base, key);
        let resp = self.http.delete(url).send().await?;

        if resp.status().is_success() {
            return Ok(());
        }

        Err(anyhow::anyhow!("DELETE failed with status {}", resp.status()))
    }

    /// LIST /v1/kv/<prefix>?keys
    pub async fn kv_list(&self, prefix: &str) -> Result<Vec<String>> {
        let url = format!("{}/v1/kv/{}?keys", self.base, prefix);
        let resp = self.http.get(url).send().await?;

        if resp.status().is_success() {
            let keys = resp.json::<Vec<String>>().await?;
            return Ok(keys);
        }

        Ok(vec![]) // treat non-200 as empty list
    }

    /// WATCH a single key using blocking queries + x-consul-index
    pub async fn kv_watch(&self, key: &str, index: Option<u64>) -> Result<(u64, Option<String>)> {
        let url = match index {
            Some(i) => format!("{}/v1/kv/{}?raw=true&index={}&wait=10s", self.base, key, i),
            None => format!("{}/v1/kv/{}?raw=true&wait=10s", self.base, key),
        };

        let resp = self.http.get(url).send().await?;

        let new_index = resp
            .headers()
            .get("x-consul-index")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);

        if resp.status().is_success() {
            let val = resp.text().await?;
            return Ok((new_index, Some(val)));
        }

        if resp.status().as_u16() == 404 {
            return Ok((new_index, None));
        }

        Err(anyhow::anyhow!("WATCH request failed: {}", resp.status()))
    }

    /// WATCH prefix keys using blocking queries + ?keys
    pub async fn kv_watch_prefix(
        &self,
        prefix: &str,
        index: Option<u64>,
    ) -> Result<(u64, Vec<String>)> {
        let url = match index {
            Some(i) => format!(
                "{}/v1/kv/{}?keys&index={}&wait=10s",
                self.base, prefix, i
            ),
            None => format!("{}/v1/kv/{}?keys&wait=10s", self.base, prefix),
        };

        let resp = self.http.get(url).send().await?;

        let new_index = resp
            .headers()
            .get("x-consul-index")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);

        if resp.status().is_success() {
            let keys = resp.json::<Vec<String>>().await.unwrap_or_default();
            return Ok((new_index, keys));
        }

        Ok((new_index, vec![]))
    }
}
