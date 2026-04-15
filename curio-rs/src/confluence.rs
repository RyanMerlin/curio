use anyhow::{Context, Result};
use reqwest::{Client, StatusCode, header::HeaderMap, header::HeaderValue, multipart};
use std::time::Duration as StdDuration;
use tokio::time::{Duration, sleep};

pub struct ConfluenceClient {
    client: Client,
    base_url: String,
    auth_token: String,
    email: String,
    write_root_folder_id: Option<String>,
}

fn is_retryable_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

async fn send_with_retry<F>(action: &str, mut build_request: F) -> Result<reqwest::Response>
where
    F: FnMut() -> reqwest::RequestBuilder,
{
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 1..=3 {
        let response = build_request()
            .send()
            .await
            .with_context(|| format!("Failed to send Confluence API request for {}", action));

        let response = match response {
            Ok(response) => response,
            Err(err) => {
                last_error = Some(err);
                if attempt < 3 {
                    sleep(Duration::from_millis(500 * attempt as u64)).await;
                    continue;
                }
                break;
            }
        };

        if is_retryable_status(response.status()) {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            last_error = Some(anyhow::anyhow!(
                "Confluence API request for {} failed with retryable status {}: {}",
                action,
                status,
                body
            ));
            if attempt < 3 {
                sleep(Duration::from_millis(500 * attempt as u64)).await;
                continue;
            }
            break;
        }

        return Ok(response);
    }

    Err(last_error.unwrap_or_else(|| {
        anyhow::anyhow!("Unknown error while calling Confluence API for {}", action)
    }))
}

pub fn http_timeout_seconds() -> u64 {
    std::env::var("CURIO_HTTP_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(45)
}

pub fn http_timeout_duration() -> StdDuration {
    StdDuration::from_secs(http_timeout_seconds())
}

impl ConfluenceClient {
    pub fn new(
        base_url: String,
        email: String,
        auth_token: String,
        write_root_folder_id: Option<String>,
    ) -> Result<Self> {
        let client = Client::builder()
            .default_headers(HeaderMap::new())
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url,
            auth_token,
            email,
            write_root_folder_id,
        })
    }

    pub async fn get_current_user(&self) -> Result<serde_json::Value> {
        let url = format!("{}/rest/api/user/current", self.base_url);
        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to send Confluence API request for current user")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body for current user")?;

        if status.is_success() {
            serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for current user")
        } else {
            anyhow::bail!(
                "Confluence auth check failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    async fn assert_within_write_root(&self, page_id: &str) -> Result<()> {
        let Some(write_root_folder_id) = self.write_root_folder_id.as_deref() else {
            return Ok(());
        };

        if page_id == write_root_folder_id {
            return Ok(());
        }

        if self
            .page_is_descendant_of(page_id, write_root_folder_id)
            .await?
        {
            Ok(())
        } else {
            anyhow::bail!(
                "Refusing to write page {} because it is outside the configured CURIO output root page {}",
                page_id,
                write_root_folder_id
            );
        }
    }

    async fn assert_parent_within_write_root(&self, parent_id: Option<&str>) -> Result<()> {
        let Some(parent_id) = parent_id else {
            return Ok(());
        };

        self.assert_within_write_root(parent_id).await
    }

    /// Creates a new Confluence page or updates it if it exists.
    /// Returns the ID of the created/updated page.
    pub async fn create_or_update_page(
        &self,
        space_key: &str,
        parent_id: Option<&str>,
        title: &str,
        body_storage_format: &str, // e.g., "storage", "editor2"
        body_content: &str,
    ) -> Result<String> {
        let effective_parent_id = parent_id.or(self.write_root_folder_id.as_deref());
        self.assert_parent_within_write_root(effective_parent_id)
            .await?;

        // First, check if the page exists
        let existing_page = self
            .get_page_by_title(space_key, effective_parent_id, title)
            .await?;

        let existing_page = if let Some(page) = existing_page {
            let page_id = page["id"]
                .as_str()
                .context("Page ID missing from existing page lookup")?;
            if self.write_root_folder_id.is_some()
                && !self
                    .page_is_descendant_of(page_id, self.write_root_folder_id.as_deref().unwrap())
                    .await?
            {
                None
            } else {
                Some(page)
            }
        } else {
            None
        };

        if let Some(page) = existing_page {
            let page_id = page["id"]
                .as_str()
                .context("Page ID missing from existing page lookup")?;
            let current_page = self
                .get_page_by_id_v2(page_id)
                .await?
                .context("Existing page could not be loaded via v2 API")?;
            let version = current_page["version"]["number"]
                .as_i64()
                .context("Page version missing")?
                + 1;

            let mut page_data = serde_json::json!({
                "id": page_id,
                "status": "current",
                "title": title,
                "body": {
                    "representation": body_storage_format,
                    "value": body_content
                },
                "version": { "number": version }
            });

            if let Some(space_id) = current_page["spaceId"].as_str() {
                page_data["spaceId"] = serde_json::json!(space_id);
            }
            if let Some(parent_id) = current_page["parentId"].as_str() {
                page_data["parentId"] = serde_json::json!(parent_id);
            }

            println!(
                "Updating Confluence page via v2: {} (ID: {})",
                title, page_id
            );
            let response = self
                .client
                .put(&format!("{}/api/v2/pages/{}", self.base_url, page_id))
                .json(&page_data)
                .basic_auth(&self.email, Some(&self.auth_token))
                .send()
                .await
                .with_context(|| {
                    format!("Failed to send Confluence API request for page: {}", title)
                })?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .context("Failed to read response body")?;

            if status.is_success() {
                let json: serde_json::Value = serde_json::from_str(&response_text)
                    .context("Failed to parse Confluence API response")?;
                json["id"]
                    .as_str()
                    .map(|s| s.to_string())
                    .context("Page ID not found in response")
            } else {
                anyhow::bail!(
                    "Confluence API request failed with status {}: {}",
                    status,
                    response_text
                );
            }
        } else {
            // Create new page via v1 API — v2 rejects storage-format macros (ac:structured-macro).
            // v1 accepts full Confluence storage format including info/tip/note/warning panels.
            let mut page_data = serde_json::json!({
                "type": "page",
                "status": "current",
                "title": title,
                "body": {
                    "storage": {
                        "representation": "storage",
                        "value": body_content
                    }
                }
            });
            if let Some(pid) = effective_parent_id {
                page_data["ancestors"] = serde_json::json!([{"id": pid}]);
            }
            page_data["space"] = serde_json::json!({"key": space_key});

            println!("Creating Confluence page: {}", title);
            let response = self
                .client
                .post(&format!("{}/rest/api/content", self.base_url))
                .basic_auth(&self.email, Some(&self.auth_token))
                .json(&page_data)
                .send()
                .await
                .with_context(|| {
                    format!("Failed to send Confluence API request for page: {}", title)
                })?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .context("Failed to read response body")?;

            if status.is_success() {
                let json: serde_json::Value = serde_json::from_str(&response_text)
                    .context("Failed to parse Confluence API response")?;
                json["id"]
                    .as_str()
                    .map(|s| s.to_string())
                    .context("Page ID not found in response")
            } else if status == StatusCode::BAD_REQUEST
                && response_text.contains("A page already exists with the same TITLE")
            {
                anyhow::bail!(
                    "Confluence title collision for '{}': {} {}",
                    title,
                    status,
                    response_text
                );
            } else {
                anyhow::bail!(
                    "Confluence API request failed with status {}: {}",
                    status,
                    response_text
                );
            }
        }
    }

    pub async fn update_page_body_by_id(
        &self,
        page_id: &str,
        body_storage_format: &str,
        body_content: &str,
    ) -> Result<()> {
        self.assert_within_write_root(page_id).await?;
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 0..3 {
            let current_page = self
                .get_page_by_id_with_body_v1(page_id)
                .await?
                .context("Page not found while updating page body")?;
            let version = current_page["version"]["number"]
                .as_i64()
                .context("Page version missing")?
                + 1;
            let title = current_page["title"]
                .as_str()
                .context("Page title missing from current page")?;
            let space_key = current_page["space"]["key"]
                .as_str()
                .context("Space key missing from current page")?;
            let body_key = if body_storage_format == "atlas_doc_format" {
                "atlas_doc_format"
            } else {
                "storage"
            };
            let body_storage = serde_json::json!({
                body_key: {
                    "value": body_content,
                    "representation": body_storage_format,
                }
            });
            let mut page_data = serde_json::json!({
                "id": page_id,
                "type": "page",
                "title": title,
                "space": { "key": space_key },
                "body": body_storage,
                "version": { "number": version }
            });
            if let Some(parent_id) = current_page["ancestors"]
                .as_array()
                .and_then(|ancestors| ancestors.last())
                .and_then(|ancestor| ancestor["id"].as_str())
            {
                page_data["ancestors"] = serde_json::json!([{ "id": parent_id }]);
            }

            println!(
                "Updating Confluence page via v1: {} (ID: {})",
                title, page_id
            );
            let response = send_with_retry("update page via v1", || {
                self.client
                    .put(&format!("{}/rest/api/content/{}", self.base_url, page_id))
                    .basic_auth(&self.email, Some(&self.auth_token))
                    .json(&page_data)
            })
            .await
            .with_context(|| {
                format!("Failed to send Confluence API request for page: {}", title)
            })?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .context("Failed to read response body after updating page body")?;

            if status.is_success() {
                return Ok(());
            }

            let is_stale = status == StatusCode::CONFLICT
                || response_text.contains("StaleStateException")
                || response_text.contains("expected: 1");
            last_error = Some(anyhow::anyhow!(
                "Confluence API request to update page body failed with status {}: {}",
                status,
                response_text
            ));
            if is_stale && attempt < 2 {
                sleep(Duration::from_millis(500)).await;
                continue;
            }
            break;
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error updating page body")))
    }

    /// Fetches a Confluence page by its title. If a parent_id is provided, it searches within that parent.
    pub async fn get_page_by_title(
        &self,
        space_key: &str,
        parent_id: Option<&str>,
        title: &str,
    ) -> Result<Option<serde_json::Value>> {
        // Build the URL with proper percent-encoding for the title so characters like
        // spaces, brackets, and colons don't cause partial-match false positives.
        let base = format!("{}/rest/api/content", self.base_url);
        let mut url_builder = reqwest::Url::parse(&base)
            .unwrap_or_else(|_| reqwest::Url::parse("http://invalid").unwrap());
        {
            let mut pairs = url_builder.query_pairs_mut();
            pairs.append_pair("spaceKey", space_key);
            pairs.append_pair("title", title);
            pairs.append_pair("expand", "version,ancestors");
            if let Some(p_id) = parent_id {
                pairs.append_pair("ancestor", p_id);
            }
        }
        let url = url_builder.to_string();

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to send Confluence API request for page by title")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for page by title")?;

            if let Some(results) = json["results"].as_array() {
                if let Some(p_id) = parent_id {
                    let matched = results.iter().find(|page| {
                        page["ancestors"]
                            .as_array()
                            .and_then(|ancestors| ancestors.last())
                            .and_then(|ancestor| ancestor["id"].as_str())
                            == Some(p_id)
                    });
                    Ok(matched.cloned())
                } else if let Some(first) = results.first() {
                    Ok(Some(first.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else if status == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            anyhow::bail!(
                "Confluence API request failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    pub async fn get_attachment_by_filename(
        &self,
        page_id: &str,
        filename: &str,
    ) -> Result<Option<serde_json::Value>> {
        let base = format!(
            "{}/rest/api/content/{}/child/attachment",
            self.base_url, page_id
        );
        let mut url_builder = reqwest::Url::parse(&base)
            .unwrap_or_else(|_| reqwest::Url::parse("http://invalid").unwrap());
        {
            let mut pairs = url_builder.query_pairs_mut();
            pairs.append_pair("filename", filename);
            pairs.append_pair("expand", "version");
        }
        let url = url_builder.to_string();

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to send Confluence API request for attachment lookup")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body for attachment lookup")?;

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence attachment lookup response")?;
            if let Some(results) = json["results"].as_array() {
                Ok(results.first().cloned())
            } else {
                Ok(None)
            }
        } else if status == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            anyhow::bail!(
                "Confluence attachment lookup failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    pub async fn upload_attachment(
        &self,
        page_id: &str,
        filename: &str,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<()> {
        self.assert_within_write_root(page_id).await?;

        let part = multipart::Part::bytes(bytes)
            .file_name(filename.to_string())
            .mime_str(content_type)
            .context("Invalid attachment content type")?;
        let form = multipart::Form::new().part("file", part);

        if let Some(existing) = self.get_attachment_by_filename(page_id, filename).await? {
            let attachment_id = existing["id"]
                .as_str()
                .context("Attachment ID missing from attachment lookup")?;
            let url = format!(
                "{}/rest/api/content/{}/child/attachment/{}/data",
                self.base_url, page_id, attachment_id
            );
            let response = self
                .client
                .post(&url)
                .basic_auth(&self.email, Some(&self.auth_token))
                .header("X-Atlassian-Token", HeaderValue::from_static("no-check"))
                .multipart(form)
                .send()
                .await
                .context("Failed to send Confluence attachment update request")?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .context("Failed to read response body for attachment update")?;

            if !status.is_success() {
                anyhow::bail!(
                    "Confluence attachment update failed with status {}: {}",
                    status,
                    response_text
                );
            }
            return Ok(());
        }

        let url = format!(
            "{}/rest/api/content/{}/child/attachment",
            self.base_url, page_id
        );
        let response = self
            .client
            .post(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .header("X-Atlassian-Token", HeaderValue::from_static("no-check"))
            .multipart(form)
            .send()
            .await
            .context("Failed to send Confluence attachment create request")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body for attachment create")?;

        if status.is_success() {
            Ok(())
        } else {
            anyhow::bail!(
                "Confluence attachment create failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    /// Adds a list of labels to a Confluence page.
    pub async fn add_labels(&self, page_id: &str, labels: Vec<String>) -> Result<()> {
        self.assert_within_write_root(page_id).await?;
        let url = format!("{}/rest/api/content/{}/label", self.base_url, page_id);
        let labels_json: Vec<serde_json::Value> = labels
            .into_iter()
            .map(|label| serde_json::json!({ "name": label }))
            .collect();

        println!("Adding labels to page {}: {:?}", page_id, labels_json);

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .json(&labels_json)
            .send()
            .await
            .context("Failed to send Confluence API request to add labels")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            println!("Labels added successfully to page {}.", page_id);
            Ok(())
        } else {
            anyhow::bail!(
                "Confluence API request to add labels failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    /// Removes a single label from a Confluence page.
    pub async fn remove_label(&self, page_id: &str, label: &str) -> Result<()> {
        self.assert_within_write_root(page_id).await?;
        let url = format!(
            "{}/rest/api/content/{}/label/{}",
            self.base_url, page_id, label
        );

        println!("Removing label '{}' from page {}", label, page_id);

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to send Confluence API request to remove label")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            println!(
                "Label '{}' removed successfully from page {}.",
                label, page_id
            );
            Ok(())
        } else if status == StatusCode::NOT_FOUND {
            println!(
                "Label '{}' not found on page {}. Assuming already removed.",
                label, page_id
            );
            Ok(())
        } else {
            anyhow::bail!(
                "Confluence API request to remove label failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    /// Creates or updates a content property for a Confluence page.
    pub async fn set_content_property(
        &self,
        page_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<()> {
        self.assert_within_write_root(page_id).await?;
        // First, try to get the existing property to determine its version.
        let existing_property = self.get_content_property(page_id, key).await?;

        let version = if let Some(prop) = existing_property {
            prop["version"]["number"]
                .as_i64()
                .context("Content property version missing")?
                + 1
        } else {
            1
        };

        let url = format!(
            "{}/rest/api/content/{}/property/{}",
            self.base_url, page_id, key
        );

        let property_data = serde_json::json!({
            "key": key,
            "value": value,
            "version": {
                "number": version
            }
        });

        println!("Setting content property '{}' for page {}", key, page_id);

        let response = self
            .client
            .put(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .json(&property_data)
            .send()
            .await
            .context("Failed to send Confluence API request to set content property")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            println!(
                "Content property '{}' set successfully for page {}.",
                key, page_id
            );
            Ok(())
        } else {
            anyhow::bail!(
                "Confluence API request to set content property failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    /// Retrieves a content property for a Confluence page.
    pub async fn get_content_property(
        &self,
        page_id: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>> {
        let url = format!(
            "{}/rest/api/content/{}/property/{}",
            self.base_url, page_id, key
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to send Confluence API request to get content property")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for content property")?;
            Ok(Some(json))
        } else if status == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            anyhow::bail!(
                "Confluence API request to get content property failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    /// Executes a Confluence Query Language (CQL) query and returns a list of matching pages.
    pub async fn execute_cql(&self, cql: &str) -> Result<Vec<serde_json::Value>> {
        self.execute_cql_with_limit(cql, None).await
    }

    pub async fn execute_cql_with_limit(
        &self,
        cql: &str,
        limit: Option<u32>,
    ) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/rest/api/content/search", self.base_url);
        let mut query_params = vec![("cql".to_string(), cql.to_string())];
        if let Some(limit) = limit {
            query_params.push(("limit".to_string(), limit.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to send Confluence API request for CQL query")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for CQL query")?;

            if let Some(results) = json["results"].as_array() {
                Ok(results.clone().into_iter().collect())
            } else {
                Ok(Vec::new())
            }
        } else {
            anyhow::bail!(
                "Confluence API request for CQL query failed with status {}: {}",
                status,
                response_text
            );
        }
    }

    /// Fetches a Confluence page by ID including the storage-format body.
    pub async fn get_page_body(&self, page_id: &str) -> Result<Option<serde_json::Value>> {
        let url = format!(
            "{}/api/v2/pages/{}?body-format=storage",
            self.base_url, page_id
        );

        let response = send_with_retry("get page body", || {
            self.client
                .get(&url)
                .basic_auth(&self.email, Some(&self.auth_token))
        })
        .await
        .with_context(|| format!("Failed to fetch body for Confluence page {}", page_id))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence page body response")?;
            Ok(Some(json))
        } else if status == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            anyhow::bail!(
                "Failed to fetch Confluence page {} body ({}): {}",
                page_id,
                status,
                response_text
            );
        }
    }

    /// Fetches a Confluence page directly by its ID using the v2 API.
    pub async fn get_page_by_id_v2(&self, page_id: &str) -> Result<Option<serde_json::Value>> {
        let url = format!("{}/api/v2/pages/{}", self.base_url, page_id);

        let response = send_with_retry("get page by id v2", || {
            self.client
                .get(&url)
                .basic_auth(&self.email, Some(&self.auth_token))
        })
        .await
        .with_context(|| {
            format!(
                "Failed to send Confluence API request for page ID: {}",
                page_id
            )
        })?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for get_page_by_id_v2")?;
            Ok(Some(json))
        } else if status == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            anyhow::bail!(
                "Confluence API request for page ID {} failed with status {}: {}",
                page_id,
                status,
                response_text
            );
        }
    }

    pub async fn get_folder_by_id_v2(&self, folder_id: &str) -> Result<Option<serde_json::Value>> {
        let url = format!("{}/api/v2/folders/{}", self.base_url, folder_id);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context(format!(
                "Failed to send Confluence API request for folder ID: {}",
                folder_id
            ))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for get_folder_by_id_v2")?;
            Ok(Some(json))
        } else if status == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            anyhow::bail!(
                "Confluence API request for folder ID {} failed with status {}: {}",
                folder_id,
                status,
                response_text
            );
        }
    }

    pub async fn get_page_by_id_with_body_v1(
        &self,
        page_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let url = format!(
            "{}/rest/api/content/{}?expand=body.storage,space,version,title",
            self.base_url, page_id
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context(format!(
                "Failed to send Confluence API request for page body: {}",
                page_id
            ))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&response_text).context(
                "Failed to parse Confluence API response for get_page_by_id_with_body_v1",
            )?;
            Ok(Some(json))
        } else if status == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            anyhow::bail!(
                "Confluence API request for page body {} failed with status {}: {}",
                page_id,
                status,
                response_text
            );
        }
    }

    pub async fn get_folder_descendants_v2(
        &self,
        folder_id: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let mut descendants = Vec::new();
        let mut next_url = Some(format!(
            "{}/api/v2/folders/{}/descendants?limit=200",
            self.base_url, folder_id
        ));

        while let Some(url) = next_url.take() {
            let response = self
                .client
                .get(&url)
                .basic_auth(&self.email, Some(&self.auth_token))
                .send()
                .await
                .context(format!(
                    "Failed to send Confluence API request for folder descendants: {}",
                    folder_id
                ))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .context("Failed to read response body")?;

            if !status.is_success() {
                anyhow::bail!(
                    "Confluence API request for folder descendants {} failed with status {}: {}",
                    folder_id,
                    status,
                    response_text
                );
            }

            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for folder descendants")?;

            if let Some(results) = json["results"].as_array() {
                descendants.extend(results.iter().cloned());
            }

            next_url = json["_links"]["next"].as_str().map(|next| {
                if next.starts_with("http://") || next.starts_with("https://") {
                    next.to_string()
                } else {
                    format!("{}{}", self.base_url, next)
                }
            });
        }

        Ok(descendants)
    }

    /// Returns only the direct children of a page (non-recursive), using the v2 API.
    pub async fn get_direct_children_v2(&self, page_id: &str) -> Result<Vec<serde_json::Value>> {
        let mut children = Vec::new();
        let mut next_url = Some(format!(
            "{}/api/v2/pages/{}/children?limit=200",
            self.base_url, page_id
        ));

        while let Some(url) = next_url.take() {
            let response = self
                .client
                .get(&url)
                .basic_auth(&self.email, Some(&self.auth_token))
                .send()
                .await
                .context(format!(
                    "Failed to send Confluence API request for direct children: {}",
                    page_id
                ))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .context("Failed to read response body")?;

            if !status.is_success() {
                anyhow::bail!(
                    "Confluence API request for direct children of {} failed with status {}: {}",
                    page_id,
                    status,
                    response_text
                );
            }

            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for direct children")?;

            if let Some(results) = json["results"].as_array() {
                children.extend(results.iter().cloned());
            }

            next_url = json["_links"]["next"].as_str().map(|next| {
                if next.starts_with("http://") || next.starts_with("https://") {
                    next.to_string()
                } else {
                    format!("{}{}", self.base_url, next)
                }
            });
        }

        Ok(children)
    }

    pub async fn get_page_descendants_v2(&self, page_id: &str) -> Result<Vec<serde_json::Value>> {
        let mut descendants = Vec::new();
        let mut next_url = Some(format!(
            "{}/api/v2/pages/{}/descendants?limit=200",
            self.base_url, page_id
        ));

        while let Some(url) = next_url.take() {
            let response = self
                .client
                .get(&url)
                .basic_auth(&self.email, Some(&self.auth_token))
                .send()
                .await
                .context(format!(
                    "Failed to send Confluence API request for page descendants: {}",
                    page_id
                ))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .context("Failed to read response body")?;

            if !status.is_success() {
                anyhow::bail!(
                    "Confluence API request for page descendants {} failed with status {}: {}",
                    page_id,
                    status,
                    response_text
                );
            }

            let json: serde_json::Value = serde_json::from_str(&response_text)
                .context("Failed to parse Confluence API response for page descendants")?;

            if let Some(results) = json["results"].as_array() {
                descendants.extend(results.iter().cloned());
            }

            next_url = json["_links"]["next"].as_str().map(|next| {
                if next.starts_with("http://") || next.starts_with("https://") {
                    next.to_string()
                } else {
                    format!("{}{}", self.base_url, next)
                }
            });
        }

        Ok(descendants)
    }

    pub async fn get_content_tree_item_by_id_v2(
        &self,
        item_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        if let Some(page) = self.get_page_by_id_v2(item_id).await? {
            return Ok(Some(page));
        }

        self.get_folder_by_id_v2(item_id).await
    }

    /// Returns the numeric space ID (e.g. "98304") for a given space key (e.g. "CURIO").
    /// The v2 pages API requires the numeric ID, not the key string.
    pub async fn get_numeric_space_id(&self, space_key: &str) -> Result<String> {
        let url = format!("{}/api/v2/spaces?keys={}", self.base_url, space_key);
        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to fetch space info")?;
        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to read space response")?;
        if !status.is_success() {
            anyhow::bail!(
                "Failed to look up space {}: {} — {}",
                space_key,
                status,
                text
            );
        }
        let json: serde_json::Value =
            serde_json::from_str(&text).context("Failed to parse space response")?;
        json["results"][0]["id"]
            .as_str()
            .map(|s| s.to_string())
            .context(format!(
                "Numeric space ID not found for key '{}'",
                space_key
            ))
    }

    pub async fn page_is_descendant_of(&self, page_id: &str, ancestor_id: &str) -> Result<bool> {
        if page_id == ancestor_id {
            return Ok(true);
        }

        let mut current_page_id = page_id.to_string();

        loop {
            let current_page = match self
                .get_content_tree_item_by_id_v2(&current_page_id)
                .await?
            {
                Some(page) => page,
                None => return Ok(false),
            };

            let Some(parent_id) = current_page["parentId"].as_str() else {
                return Ok(false);
            };

            if parent_id == ancestor_id {
                return Ok(true);
            }

            current_page_id = parent_id.to_string();
        }
    }

    pub fn page_web_url(&self, page_id: &str) -> String {
        format!(
            "{}/pages/viewpage.action?pageId={}",
            self.base_url.trim_end_matches('/'),
            page_id
        )
    }

    /// Moves a Confluence page to a new parent page.
    pub async fn move_page(&self, page_id: &str, new_parent_id: &str) -> Result<()> {
        self.assert_within_write_root(page_id).await?;
        self.assert_within_write_root(new_parent_id).await?;
        let url = format!("{}/rest/api/content/{}", self.base_url, page_id);

        // First, get the current page details to extract the version and other necessary fields
        let response = self
            .client
            .get(&format!(
                "{}/rest/api/content/{}?expand=version,body.storage,space",
                self.base_url, page_id
            ))
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context(format!(
                "Failed to fetch current page details for moving page {}",
                page_id
            ))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body for current page details")?;

        if !status.is_success() {
            anyhow::bail!(
                "Failed to get current page details for moving page {}. Status: {}. Response: {}",
                page_id,
                status,
                response_text
            );
        }
        let current_page: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse current page details for moving page")?;

        let current_version = current_page["version"]["number"]
            .as_i64()
            .context("Page version missing")?
            + 1;
        let space_key = current_page["space"]["key"]
            .as_str()
            .context("Space key missing from current page")?;
        let title = current_page["title"]
            .as_str()
            .context("Page title missing from current page")?;
        let body_content = current_page["body"]["storage"]["value"]
            .as_str()
            .unwrap_or_default(); // Get current body

        let update_payload = serde_json::json!({
            "id": page_id,
            "type": "page",
            "title": title,
            "space": { "key": space_key },
            "body": {
                "storage": {
                    "value": body_content,
                    "representation": "storage"
                }
            },
            "ancestors": [{"id": new_parent_id}],
            "version": { "number": current_version }
        });

        println!(
            "Moving page {} (ID: {}) to new parent {}",
            title, page_id, new_parent_id
        );

        let response = self
            .client
            .put(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&update_payload)
            .send()
            .await
            .context(format!(
                "Failed to send Confluence API request to move page {}",
                page_id
            ))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body after moving page")?;

        if status.is_success() {
            println!(
                "Page {} moved successfully to parent {}.",
                page_id, new_parent_id
            );
            Ok(())
        } else {
            anyhow::bail!(
                "Confluence API request to move page {} failed with status {}: {}",
                page_id,
                status,
                response_text
            );
        }
    }

    /// Moves a page under a managed parent while allowing the source page to live outside
    /// the current write root. Use this only for migration of legacy pages into the scoped tree.
    pub async fn migrate_page_to_parent(&self, page_id: &str, new_parent_id: &str) -> Result<()> {
        self.assert_within_write_root(new_parent_id).await?;
        let url = format!("{}/rest/api/content/{}", self.base_url, page_id);

        let response = self
            .client
            .get(&format!(
                "{}/rest/api/content/{}?expand=version,body.storage,space",
                self.base_url, page_id
            ))
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context(format!(
                "Failed to fetch current page details for migrating page {}",
                page_id
            ))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body for current page details")?;

        if !status.is_success() {
            anyhow::bail!(
                "Failed to get current page details for migrating page {}. Status: {}. Response: {}",
                page_id,
                status,
                response_text
            );
        }
        let current_page: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse current page details for migrating page")?;

        let current_version = current_page["version"]["number"]
            .as_i64()
            .context("Page version missing")?
            + 1;
        let space_key = current_page["space"]["key"]
            .as_str()
            .context("Space key missing from current page")?;
        let title = current_page["title"]
            .as_str()
            .context("Page title missing from current page")?;
        let body_content = current_page["body"]["storage"]["value"]
            .as_str()
            .unwrap_or_default();

        let update_payload = serde_json::json!({
            "id": page_id,
            "type": "page",
            "title": title,
            "space": { "key": space_key },
            "body": {
                "storage": {
                    "value": body_content,
                    "representation": "storage"
                }
            },
            "ancestors": [{"id": new_parent_id}],
            "version": { "number": current_version }
        });

        println!(
            "Migrating page {} (ID: {}) to new parent {}",
            title, page_id, new_parent_id
        );

        let response = self
            .client
            .put(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&update_payload)
            .send()
            .await
            .context(format!(
                "Failed to send Confluence API request to migrate page {}",
                page_id
            ))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body after migrating page")?;

        if status.is_success() {
            println!(
                "Page {} migrated successfully to parent {}.",
                page_id, new_parent_id
            );
            Ok(())
        } else {
            anyhow::bail!(
                "Confluence API request to migrate page {} failed with status {}: {}",
                page_id,
                status,
                response_text
            );
        }
    }

    // ─── Feedback read/write endpoints ────────────────────────────────────────

    /// GET {base_url}/api/v2/pages/{id}/labels — returns label names
    pub async fn get_page_labels_v2(&self, page_id: &str) -> Result<Vec<String>> {
        let url = format!("{}/api/v2/pages/{}/labels", self.base_url, page_id);
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to fetch page labels")?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse labels response")?;
        let labels = body["results"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|l| l["name"].as_str().map(|s| s.to_string()))
            .collect();
        Ok(labels)
    }

    /// GET {base_url}/api/v2/pages/{id}/footer-comments — returns comment objects
    pub async fn get_page_footer_comments(&self, page_id: &str) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}/api/v2/pages/{}/footer-comments?body-format=storage",
            self.base_url, page_id
        );
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to fetch footer comments")?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse footer comments")?;
        Ok(body["results"].as_array().cloned().unwrap_or_default())
    }

    /// GET {base_url}/api/v2/pages/{id}/inline-comments — returns comment objects
    pub async fn get_page_inline_comments(&self, page_id: &str) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}/api/v2/pages/{}/inline-comments?body-format=storage",
            self.base_url, page_id
        );
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to fetch inline comments")?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse inline comments")?;
        Ok(body["results"].as_array().cloned().unwrap_or_default())
    }

    /// GET {base_url}/api/v2/footer-comments/{id}/reactions — returns reaction objects
    pub async fn get_comment_reactions(&self, comment_id: &str) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}/api/v2/footer-comments/{}/reactions",
            self.base_url, comment_id
        );
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to fetch comment reactions")?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let body: serde_json::Value = resp.json().await.context("Failed to parse reactions")?;
        Ok(body["results"].as_array().cloned().unwrap_or_default())
    }

    /// POST {base_url}/api/v2/footer-comments — create a footer comment on a page.
    /// Returns the new comment ID.
    pub async fn create_footer_comment(
        &self,
        page_id: &str,
        body_storage_xml: &str,
    ) -> Result<String> {
        let url = format!("{}/api/v2/footer-comments", self.base_url);
        let payload = serde_json::json!({
            "pageId": page_id,
            "body": {
                "representation": "storage",
                "value": body_storage_xml
            }
        });
        let resp = self
            .client
            .post(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .json(&payload)
            .send()
            .await
            .context("Failed to create footer comment")?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .context("Failed to read create-comment response")?;
        if !status.is_success() {
            anyhow::bail!("create_footer_comment failed {}: {}", status, text);
        }
        let val: serde_json::Value =
            serde_json::from_str(&text).context("Failed to parse create-comment response")?;
        val["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No id in create-comment response: {}", text))
    }

    /// PUT {base_url}/api/v2/footer-comments/{id} — update an existing footer comment body.
    pub async fn update_footer_comment(
        &self,
        comment_id: &str,
        body_storage_xml: &str,
    ) -> Result<()> {
        let url = format!("{}/api/v2/footer-comments/{}", self.base_url, comment_id);
        // Need current version first
        let get_resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context("Failed to fetch comment for update")?;
        let current: serde_json::Value = get_resp.json().await.unwrap_or_default();
        let version = current["version"]["number"].as_u64().unwrap_or(0) + 1;

        let payload = serde_json::json!({
            "version": { "number": version },
            "body": {
                "representation": "storage",
                "value": body_storage_xml
            }
        });
        let resp = self
            .client
            .put(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .json(&payload)
            .send()
            .await
            .context("Failed to update footer comment")?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("update_footer_comment failed {}: {}", status, text);
        }
        Ok(())
    }

    pub async fn delete_page(&self, page_id: &str) -> Result<()> {
        self.assert_within_write_root(page_id).await?;
        let url = format!("{}/rest/api/content/{}", self.base_url, page_id);

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context(format!(
                "Failed to send Confluence API request to delete page {}",
                page_id
            ))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body after deleting page")?;

        if status.is_success() || status == StatusCode::NOT_FOUND {
            Ok(())
        } else {
            anyhow::bail!(
                "Confluence API request to delete page {} failed with status {}: {}",
                page_id,
                status,
                response_text
            );
        }
    }
}
