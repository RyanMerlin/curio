//! Deterministic, offline evaluation for the published-page retriever.

use crate::retrieval::{RetrieveRequest, retrieve_published};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct EvaluationCorpus {
    pub version: u32,
    pub as_of: String,
    pub stale_before: String,
    pub queries: Vec<EvaluationQuery>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvaluationQuery {
    pub id: String,
    pub query: String,
    pub expected_ids: Vec<String>,
    #[serde(default)]
    pub unauthorized_ids: Vec<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EvaluationReport {
    pub version: u32,
    pub as_of: String,
    pub stale_before: String,
    pub query_count: usize,
    pub metrics: EvaluationMetrics,
    pub queries: Vec<QueryEvaluation>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EvaluationMetrics {
    pub recall_at_k: f64,
    pub mean_reciprocal_rank: f64,
    pub citation_coverage: f64,
    pub stale_result_rate: f64,
    pub acl_leak_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct QueryEvaluation {
    pub id: String,
    pub query: String,
    pub limit: usize,
    pub expected_count: usize,
    pub retrieved_ids: Vec<String>,
    pub recall_at_k: f64,
    pub reciprocal_rank: f64,
    pub citation_coverage: f64,
    pub stale_result_count: usize,
    pub acl_leak_count: usize,
}

pub fn load_corpus(path: &Path) -> Result<EvaluationCorpus> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read evaluation corpus {}", path.display()))?;
    let corpus: EvaluationCorpus = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse evaluation corpus {}", path.display()))?;
    validate_corpus(&corpus)?;
    Ok(corpus)
}

pub fn evaluate(corpus: &EvaluationCorpus, wiki_dir: &Path) -> Result<EvaluationReport> {
    validate_corpus(corpus)?;
    if corpus.queries.is_empty() {
        bail!("Evaluation corpus must contain at least one query.");
    }

    let mut query_reports = Vec::with_capacity(corpus.queries.len());
    for query in &corpus.queries {
        let limit = query.limit.unwrap_or(5);
        let response = retrieve_published(
            wiki_dir,
            &RetrieveRequest {
                query: query.query.clone(),
                category: None,
                limit,
            },
        )?;
        let retrieved_ids: Vec<String> = response
            .results
            .iter()
            .map(|result| result.id.clone())
            .collect();
        let expected_hits = query
            .expected_ids
            .iter()
            .filter(|id| retrieved_ids.contains(id))
            .count();
        let recall_at_k = if query.expected_ids.is_empty() {
            1.0
        } else {
            expected_hits as f64 / query.expected_ids.len() as f64
        };
        let reciprocal_rank = query
            .expected_ids
            .iter()
            .filter_map(|id| retrieved_ids.iter().position(|found| found == id))
            .min()
            .map(|position| 1.0 / (position + 1) as f64)
            .unwrap_or(0.0);
        let citation_count = response
            .results
            .iter()
            .filter(|result| {
                !result.path.trim().is_empty()
                    && !result.excerpt.trim().is_empty()
                    && (result.source_uri.is_some() || !result.path.trim().is_empty())
            })
            .count();
        let citation_coverage = if response.results.is_empty() {
            1.0
        } else {
            citation_count as f64 / response.results.len() as f64
        };
        let stale_result_count = response
            .results
            .iter()
            .filter(|result| result.updated_at.as_str() < corpus.stale_before.as_str())
            .count();
        let acl_leak_count = response
            .results
            .iter()
            .filter(|result| query.unauthorized_ids.contains(&result.id))
            .count();

        query_reports.push(QueryEvaluation {
            id: query.id.clone(),
            query: query.query.clone(),
            limit,
            expected_count: query.expected_ids.len(),
            retrieved_ids,
            recall_at_k,
            reciprocal_rank,
            citation_coverage,
            stale_result_count,
            acl_leak_count,
        });
    }

    let query_count = query_reports.len();
    let total_results: usize = query_reports
        .iter()
        .map(|report| report.retrieved_ids.len())
        .sum();
    let metrics = EvaluationMetrics {
        recall_at_k: query_reports
            .iter()
            .map(|report| report.recall_at_k)
            .sum::<f64>()
            / query_count as f64,
        mean_reciprocal_rank: query_reports
            .iter()
            .map(|report| report.reciprocal_rank)
            .sum::<f64>()
            / query_count as f64,
        citation_coverage: query_reports
            .iter()
            .map(|report| report.citation_coverage * report.retrieved_ids.len() as f64)
            .sum::<f64>()
            / total_results.max(1) as f64,
        stale_result_rate: query_reports
            .iter()
            .map(|report| report.stale_result_count)
            .sum::<usize>() as f64
            / total_results.max(1) as f64,
        acl_leak_count: query_reports
            .iter()
            .map(|report| report.acl_leak_count)
            .sum(),
    };

    Ok(EvaluationReport {
        version: corpus.version,
        as_of: corpus.as_of.clone(),
        stale_before: corpus.stale_before.clone(),
        query_count,
        metrics,
        queries: query_reports,
    })
}

fn validate_corpus(corpus: &EvaluationCorpus) -> Result<()> {
    if corpus.version != 1 {
        bail!("Unsupported evaluation corpus version {}.", corpus.version);
    }
    if corpus.as_of.trim().is_empty() || corpus.stale_before.trim().is_empty() {
        bail!("Evaluation corpus requires non-empty as_of and stale_before dates.");
    }
    for query in &corpus.queries {
        if query.id.trim().is_empty() || query.query.trim().is_empty() {
            bail!("Every evaluation query requires a non-empty id and query.");
        }
        if query.expected_ids.is_empty() {
            bail!("Evaluation query {} must define expected_ids.", query.id);
        }
        if query.limit == Some(0) {
            bail!("Evaluation query {} must use a positive limit.", query.id);
        }
    }
    Ok(())
}
