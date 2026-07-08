use std::collections::HashMap;

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::Result;

use super::{QualityIssue, QualityReport};

// Quality Report (ENG-64)
// ============================================================================

/// Generate a quality report for a workspace
pub fn generate_quality_report(
    conn: &Connection,
    workspace: Option<&str>,
) -> Result<QualityReport> {
    let workspace_filter = workspace.unwrap_or("default");

    // Total memories
    let total_memories: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE workspace = ? AND deleted_at IS NULL",
        params![workspace_filter],
        |row| row.get(0),
    )?;

    // Average quality
    let average_quality: f32 = conn
        .query_row(
            "SELECT COALESCE(AVG(quality_score), 0.5) FROM memories WHERE workspace = ? AND deleted_at IS NULL",
            params![workspace_filter],
            |row| row.get(0),
        )
        .unwrap_or(0.5);

    // Quality distribution
    let mut distribution = HashMap::new();
    let grades = ['A', 'B', 'C', 'D', 'F'];
    for grade in grades {
        let (min, max) = match grade {
            'A' => (0.9, 1.1),
            'B' => (0.8, 0.9),
            'C' => (0.7, 0.8),
            'D' => (0.6, 0.7),
            _ => (0.0, 0.6),
        };
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories WHERE workspace = ? AND deleted_at IS NULL AND quality_score >= ? AND quality_score < ?",
            params![workspace_filter, min, max],
            |row| row.get(0),
        ).unwrap_or(0);
        distribution.insert(grade, count);
    }

    // Conflicts count
    let conflicts_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_conflicts WHERE resolved_at IS NULL",
        [],
        |row| row.get(0),
    )?;

    // Duplicates count
    let duplicates_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duplicate_candidates WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;

    // Top issues
    let mut top_issues = Vec::new();

    if conflicts_count > 0 {
        top_issues.push(QualityIssue {
            issue_type: "conflicts".to_string(),
            count: conflicts_count,
            severity: "high".to_string(),
            description: format!("{} unresolved conflicts detected", conflicts_count),
        });
    }

    if duplicates_count > 0 {
        top_issues.push(QualityIssue {
            issue_type: "duplicates".to_string(),
            count: duplicates_count,
            severity: "medium".to_string(),
            description: format!("{} potential duplicates found", duplicates_count),
        });
    }

    // Low quality count
    let low_quality_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE workspace = ? AND deleted_at IS NULL AND quality_score < 0.5",
        params![workspace_filter],
        |row| row.get(0),
    ).unwrap_or(0);

    if low_quality_count > 0 {
        top_issues.push(QualityIssue {
            issue_type: "low_quality".to_string(),
            count: low_quality_count,
            severity: "medium".to_string(),
            description: format!("{} memories with low quality scores", low_quality_count),
        });
    }

    let suggestions_summary = vec![
        format!("Average quality score: {:.0}%", average_quality * 100.0),
        format!("Total memories: {}", total_memories),
        if conflicts_count > 0 {
            format!(
                "Resolve {} conflicts to improve consistency",
                conflicts_count
            )
        } else {
            "No conflicts detected".to_string()
        },
    ];

    Ok(QualityReport {
        total_memories,
        average_quality,
        quality_distribution: distribution,
        top_issues,
        conflicts_count,
        duplicates_count,
        suggestions_summary,
        generated_at: Utc::now(),
    })
}

// ============================================================================
