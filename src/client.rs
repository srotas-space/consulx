use anyhow::Result;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Serialize, de::DeserializeOwned};

/// Characters we percent-encode inside a KV key. Consul keys are path
/// segments, so `/` is deliberately preserved as a separator.
const KEY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'^')
    .add(b'|')
    .add(b'\\');


#[derive(Clone)]
pub struct ConsulXClient {
    pub http: Client,
    pub base: String,
    pub dc: Option<String>,
}


impl ConsulXClient {
    /// Build a client for `base`, picking up `CONSUL_HTTP_TOKEN` and the
    /// datacenter (`CONSUL_DATACENTER`/`CONSUL_DC`) from the environment.
    pub fn new(base: &str) -> Result<Self> {
        let token = std::env::var("CONSUL_HTTP_TOKEN").ok().filter(|t| !t.is_empty());
        let dc = std::env::var("CONSUL_DATACENTER")
            .or_else(|_| std::env::var("CONSUL_DC"))
            .ok()
            .filter(|d| !d.is_empty());
        Self::with_options(base, token, dc)
    }

    /// Build a client with an explicit ACL token and/or datacenter.
    pub fn with_options(base: &str, token: Option<String>, dc: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(t) = token.as_deref() {
            let mut value = HeaderValue::from_str(t)
                .map_err(|_| anyhow::anyhow!("CONSUL_HTTP_TOKEN contains invalid header characters"))?;
            value.set_sensitive(true);
            headers.insert("X-Consul-Token", value);
        }

        let http = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            http,
            base: base.trim_end_matches('/').to_string(),
            dc,
        })
    }


    /// Build from CONSUL_HTTP_ADDR (or default http://127.0.0.1:8500)
    pub fn from_env() -> Result<Self> {
        let addr = std::env::var("CONSUL_HTTP_ADDR")
            .unwrap_or_else(|_| "http://127.0.0.1:8500".to_string());
        Self::new(&addr)
    }

    /// Build a `/v1/kv/<key>` URL, percent-encoding the key and appending
    /// query flags plus the configured datacenter. A param with an empty
    /// value is emitted as a bare flag (e.g. `raw`, `keys`).
    fn kv_url(&self, key: &str, params: &[(&str, String)]) -> String {
        let encoded = utf8_percent_encode(key, KEY_ENCODE_SET).to_string();
        let mut url = format!("{}/v1/kv/{}", self.base, encoded);

        let mut query: Vec<String> = params
            .iter()
            .map(|(k, v)| if v.is_empty() { k.to_string() } else { format!("{k}={v}") })
            .collect();
        if let Some(dc) = &self.dc {
            query.push(format!("dc={dc}"));
        }

        if !query.is_empty() {
            url.push('?');
            url.push_str(&query.join("&"));
        }
        url
    }


    /// GET /v1/kv/<key>?raw
    pub async fn kv_get_raw(&self, key: &str) -> Result<Option<String>> {
        let url = self.kv_url(key, &[("raw", "true".into())]);
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
        let url = self.kv_url(key, &[]);
        let resp = self.http.put(url).body(value.to_string()).send().await?;

        if resp.status().is_success() {
            return Ok(());
        }

        Err(anyhow::anyhow!("PUT failed with status {}", resp.status()))
    }

    /// DELETE /v1/kv/<key>
    pub async fn kv_delete(&self, key: &str) -> Result<()> {
        let url = self.kv_url(key, &[]);
        let resp = self.http.delete(url).send().await?;

        if resp.status().is_success() {
            return Ok(());
        }

        Err(anyhow::anyhow!("DELETE failed with status {}", resp.status()))
    }

    /// LIST /v1/kv/<prefix>?keys
    pub async fn kv_list(&self, prefix: &str) -> Result<Vec<String>> {
        let url = self.kv_url(prefix, &[("keys", String::new())]);
        let resp = self.http.get(url).send().await?;

        if resp.status().is_success() {
            let keys = resp.json::<Vec<String>>().await?;
            return Ok(keys);
        }

        // An empty prefix legitimately 404s; anything else is a real error
        // (e.g. 403 from a missing/insufficient ACL token) and must not be
        // silently reported as "no keys".
        if resp.status().as_u16() == 404 {
            return Ok(vec![]);
        }

        Err(anyhow::anyhow!("LIST failed with status {}", resp.status()))
    }

    /// High-level: fetch a value and deserialize JSON into type T
    pub async fn kv_get_json<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        if let Some(raw) = self.kv_get_raw(key).await? {
            let value = serde_json::from_str::<T>(&raw)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// High-level: serialize a JSON-compatible type and store it
    pub async fn kv_put_json<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(value)?;
        self.kv_put(key, &json).await?;
        Ok(())
    }


    /// Fetch JSON under a prefix and deserialize each entry.
    ///
    /// Returns:
    ///     Vec<(key, T)>
    ///
    /// Example keys:
    ///     app/config/db      → Some JSON
    ///     app/config/cache   → Some JSON
    ///
    pub async fn kv_list_json<T>(&self, prefix: &str) -> Result<Vec<(String, T)>>
    where
        T: DeserializeOwned,
    {
        let keys = self.kv_list(prefix).await?;

        let mut out = Vec::new();

        for key in keys {
            if let Some(raw) = self.kv_get_raw(&key).await? {
                if raw.trim().is_empty() {
                    continue;
                }

                let parsed = serde_json::from_str::<T>(&raw)?;
                out.push((key, parsed));
            }
        }

        Ok(out)
    }

    /// WATCH a single key using blocking queries + x-consul-index
    pub async fn kv_watch(&self, key: &str, index: Option<u64>) -> Result<(u64, Option<String>)> {
        let mut params = vec![("raw", "true".into()), ("wait", "10s".into())];
        if let Some(i) = index {
            params.push(("index", i.to_string()));
        }
        let url = self.kv_url(key, &params);

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
        let mut params = vec![("keys", String::new()), ("wait", "10s".into())];
        if let Some(i) = index {
            params.push(("index", i.to_string()));
        }
        let url = self.kv_url(prefix, &params);

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
