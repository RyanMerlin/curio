use anyhow::{Context, Result};
use reqwest::{Client, StatusCode, header::HeaderMap};

pub struct ConfluenceClient {
    client: Client,
    base_url: String,
    auth_token: String,
    email: String,
}

impl ConfluenceClient {
    pub fn new(base_url: String, email: String, auth_token: String) -> Result<Self> {
        let client = Client::builder()
            .default_headers(HeaderMap::new())
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url,
            auth_token,
            email,
        })
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
        let url = format!("{}/rest/api/content", self.base_url);

        // First, check if the page exists
        let existing_page = self.get_page_by_title(space_key, parent_id, title).await?;

        let mut page_data = serde_json::json!({
            "type": "page",
            "title": title,
            "space": { "key": space_key },
            "body": {
                body_storage_format: {
                    "value": body_content,
                    "representation": body_storage_format
                }
            }
        });

        let request = if let Some(page) = existing_page {
            // Update existing page
            let version = page["version"]["number"]
                .as_i64()
                .context("Page version missing")?
                + 1;
            page_data["id"] = page["id"].clone();
            page_data["version"] = serde_json::json!({ "number": version });

            println!(
                "Updating Confluence page: {} (ID: {})",
                title,
                page["id"].as_str().unwrap_or_default()
            );
            self.client
                .put(&format!(
                    "{}/wiki/rest/api/content/{}",
                    self.base_url,
                    page["id"].as_str().unwrap_or_default()
                ))
                .json(&page_data)
        } else {
            // Create new page
            if let Some(p_id) = parent_id {
                page_data["ancestors"] = serde_json::json!([{"id": p_id}]);
            }
            println!("Creating Confluence page: {}", title);
            self.client.post(&url).json(&page_data)
        };

        let response = request
            .basic_auth(&self.email, Some(&self.auth_token))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
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
    }

    /// Fetches a Confluence page by its title. If a parent_id is provided, it searches within that parent.
    pub async fn get_page_by_title(
        &self,
        space_key: &str,
        parent_id: Option<&str>,
        title: &str,
    ) -> Result<Option<serde_json::Value>> {
        let mut url = format!(
            "{}/rest/api/content?spaceKey={}&title={}&expand=version",
            self.base_url, space_key, title
        );

        if let Some(p_id) = parent_id {
            url = format!(
                "{}/rest/api/content?spaceKey={}&title={}&ancestor={}&expand=version",
                self.base_url, space_key, title, p_id
            );
        }

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
                // If parent_id is used, filter by ancestors as the API's 'ancestor' query parameter
                // might not strictly enforce direct parentage in all Confluence versions/configurations.
                // A more robust check might be needed if direct parent is a strict requirement.
                if let Some(_p_id) = parent_id {
                    // Filter logic if needed, but for now, if title and space match, it's sufficient.
                    // Confluence API search can be tricky with parentage.
                }

                if results.len() > 0 {
                    // Assuming the first result is the one we want if title and space match
                    // This might need refinement if multiple pages can have the same title under different parents.
                    Ok(Some(results[0].clone()))
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

    /// Adds a list of labels to a Confluence page.
    pub async fn add_labels(&self, page_id: &str, labels: Vec<String>) -> Result<()> {
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
        let url = format!("{}/rest/api/content/search?cql={}", self.base_url, cql);

        println!("Executing CQL query: {}", cql);

        let response = self
            .client
            .get(&url)
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

    /// Fetches a Confluence page directly by its ID using the v2 API.
    pub async fn get_page_by_id_v2(&self, page_id: &str) -> Result<Option<serde_json::Value>> {
        let url = format!("{}/api/v2/pages/{}", self.base_url, page_id);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.auth_token))
            .send()
            .await
            .context(format!(
                "Failed to send Confluence API request for page ID: {}",
                page_id
            ))?;

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

    /// Moves a Confluence page to a new parent page.
    pub async fn move_page(&self, page_id: &str, new_parent_id: &str) -> Result<()> {
        let url = format!("{}/rest/api/content/{}", self.base_url, page_id);

        // First, get the current page details to extract the version and other necessary fields
        let response = self
            .client
            .get(&format!(
                "{}/rest/api/content/{}?expand=version,body,space",
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
}
