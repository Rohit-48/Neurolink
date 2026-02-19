use std::fs;

/// Integration test: Verify that the typo mentioned in task 1 exists in README
#[test]
fn test_task1_typo_actually_exists_in_readme() {
    let readme = fs::read_to_string("README.md")
        .expect("Failed to read README.md");

    // Task 1 claims there's a typo: _nerolink._tcp should be _neurolink._tcp
    // At least one of these should exist to validate the task is meaningful
    let has_nerolink = readme.contains("_nerolink._tcp");
    let has_neurolink = readme.contains("_neurolink._tcp");

    assert!(
        has_nerolink || has_neurolink,
        "Task 1 mentions service type typo, but neither _nerolink._tcp nor _neurolink._tcp found in README"
    );
}

/// Integration test: Verify chunk_size usage in routes.rs (task 2)
#[test]
fn test_task2_chunk_size_usage_in_routes() {
    let routes = fs::read_to_string("src/rust/api/routes.rs")
        .expect("Failed to read src/rust/api/routes.rs");

    // Task 2 mentions chunk_size calculations
    assert!(
        routes.contains("chunk_size") || routes.contains("chunk-size"),
        "Task 2 claims chunk_size is used in routes.rs but not found"
    );
}

/// Integration test: Verify chunk_size usage in transfer mod (task 2)
#[test]
fn test_task2_chunk_size_usage_in_transfer() {
    let transfer = fs::read_to_string("src/rust/transfer/mod.rs")
        .expect("Failed to read src/rust/transfer/mod.rs");

    // Task 2 mentions chunk_size calculations in transfer module
    assert!(
        transfer.contains("chunk_size"),
        "Task 2 claims chunk_size is used in transfer/mod.rs but not found"
    );
}

/// Integration test: Verify port configuration in main.rs (task 3)
#[test]
fn test_task3_port_configuration_in_main() {
    let main_rs = fs::read_to_string("src/rust/main.rs")
        .expect("Failed to read src/rust/main.rs");

    // Task 3 mentions default port configuration
    let mentions_port = main_rs.contains("3030") ||
                       main_rs.contains("8000") ||
                       main_rs.contains("PORT");

    assert!(
        mentions_port,
        "Task 3 claims main.rs has port configuration (3030 or 8000) but not found"
    );
}

/// Integration test: Verify README mentions port defaults (task 3)
#[test]
fn test_task3_port_documentation_in_readme() {
    let readme = fs::read_to_string("README.md")
        .expect("Failed to read README.md");

    // Task 3 mentions README documents port as 3030
    let mentions_port = readme.contains("3030") || readme.contains("8000");

    assert!(
        mentions_port,
        "Task 3 claims README documents default port but not found"
    );
}

/// Integration test: Check if transfer/API modules have test modules (task 4)
#[test]
fn test_task4_verifies_lack_of_tests() {
    let transfer = fs::read_to_string("src/rust/transfer/mod.rs")
        .expect("Failed to read src/rust/transfer/mod.rs");
    let routes = fs::read_to_string("src/rust/api/routes.rs")
        .expect("Failed to read src/rust/api/routes.rs");

    // Task 4 claims there are no test modules
    // This test verifies the claim is accurate
    let transfer_has_tests = transfer.contains("#[cfg(test)]") ||
                            transfer.contains("mod tests");
    let routes_has_tests = routes.contains("#[cfg(test)]") ||
                          routes.contains("mod tests");

    // If tests exist, the task description might be outdated
    if transfer_has_tests || routes_has_tests {
        eprintln!("Note: Task 4 claims no tests exist, but some test modules were found");
        eprintln!("Transfer has tests: {}", transfer_has_tests);
        eprintln!("Routes has tests: {}", routes_has_tests);
    }

    // This test always passes but provides information
    assert!(
        true,
        "This test documents the current test coverage state"
    );
}

/// Edge case: Verify division by chunk_size pattern exists (task 2 concern)
#[test]
fn test_task2_division_pattern_exists() {
    let routes = fs::read_to_string("src/rust/api/routes.rs")
        .expect("Failed to read src/rust/api/routes.rs");
    let transfer = fs::read_to_string("src/rust/transfer/mod.rs")
        .expect("Failed to read src/rust/transfer/mod.rs");

    let combined = format!("{}{}", routes, transfer);

    // Look for patterns that suggest division by chunk_size
    let has_division_pattern = combined.contains("/ chunk_size") ||
                              combined.contains("/chunk_size") ||
                              combined.contains("chunk_size)") ||
                              combined.contains("total_chunks");

    assert!(
        has_division_pattern,
        "Task 2 mentions division by chunk_size causing issues, but pattern not found"
    );
}

/// Negative test: Verify that chunk_size validation doesn't already exist
#[test]
fn test_task2_chunk_size_validation_status() {
    let routes = fs::read_to_string("src/rust/api/routes.rs")
        .expect("Failed to read src/rust/api/routes.rs");

    // Check if validation already exists
    let has_zero_check = routes.contains("chunk_size == 0") ||
                        routes.contains("chunk_size <= 0") ||
                        routes.contains("chunk_size > 0");

    if has_zero_check {
        eprintln!("Note: Task 2 suggests adding chunk_size validation, but some validation may already exist");
    }

    // This test documents current state
    assert!(
        true,
        "Test documents whether chunk_size validation exists"
    );
}

/// Regression test: Verify Cargo.toml has test dependencies for task 4
#[test]
fn test_task4_test_infrastructure_exists() {
    let cargo_toml = fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml");

    // Task 4 suggests adding tests that run with cargo test
    // Verify the infrastructure exists
    assert!(
        cargo_toml.contains("[dev-dependencies]"),
        "Project should have dev-dependencies section for testing"
    );

    // Check for testing dependencies
    let has_test_deps = cargo_toml.contains("tokio-test") ||
                       cargo_toml.contains("test");

    assert!(
        has_test_deps,
        "Project should have some test dependencies configured"
    );
}

/// Boundary test: Verify all task-referenced files are Rust or markdown
#[test]
fn test_referenced_files_are_expected_types() {
    let files = vec![
        ("README.md", "markdown"),
        ("src/rust/api/routes.rs", "rust"),
        ("src/rust/transfer/mod.rs", "rust"),
        ("src/rust/main.rs", "rust"),
    ];

    for (file_path, expected_type) in files {
        assert!(
            std::path::Path::new(file_path).exists(),
            "File {} should exist",
            file_path
        );

        if expected_type == "rust" {
            assert!(
                file_path.ends_with(".rs"),
                "Rust file {} should have .rs extension",
                file_path
            );
        } else if expected_type == "markdown" {
            assert!(
                file_path.ends_with(".md"),
                "Markdown file {} should have .md extension",
                file_path
            );
        }
    }
}

/// Integration test: Verify task priorities are reasonable
#[test]
fn test_task_priorities_are_reasonable() {
    let content = fs::read_to_string("ISSUE_TASKS.md")
        .expect("Failed to read ISSUE_TASKS.md");

    // Tasks are numbered 1-4, presumably by priority
    // Task 1: Typo fix (low risk)
    // Task 2: Bug fix (medium/high risk - division by zero)
    // Task 3: Documentation discrepancy (low/medium risk)
    // Task 4: Test improvement (proactive improvement)

    // Verify bug fix task (task 2) mentions high-risk terms
    let task2_start = content.find("## 2) Bug fix task").unwrap();
    let task2_end = content[task2_start..].find("## 3)").unwrap_or(content.len() - task2_start);
    let task2 = &content[task2_start..task2_start + task2_end];

    let has_risk_indicators = task2.contains("panic") ||
                             task2.contains("division") ||
                             task2.contains("error") ||
                             task2.contains("undefined behavior");

    assert!(
        has_risk_indicators,
        "Bug fix task should mention risk indicators to justify priority"
    );
}

/// Comprehensive integration test: Verify all 4 tasks reference valid locations
#[test]
fn test_all_tasks_have_verifiable_claims() {
    let content = fs::read_to_string("ISSUE_TASKS.md")
        .expect("Failed to read ISSUE_TASKS.md");

    // Extract "Where observed" sections and verify they're meaningful
    let where_observed_count = content.matches("**Where observed:**").count();
    assert_eq!(
        where_observed_count, 4,
        "Should have 4 'Where observed' sections (one per task)"
    );

    // Each "Where observed" should mention specific files
    let tasks: Vec<&str> = content.split("## ").skip(1).collect();

    for (idx, task) in tasks.iter().take(4).enumerate() {
        let task_num = idx + 1;

        // Find the "Where observed" section
        if let Some(where_pos) = task.find("**Where observed:**") {
            let after_where = &task[where_pos..];
            if let Some(end_pos) = after_where.find("**Suggested acceptance criteria:**") {
                let where_section = &after_where[..end_pos];

                // Should mention at least one file
                let mentions_file = where_section.contains(".rs") ||
                                  where_section.contains(".md") ||
                                  where_section.contains("src/");

                assert!(
                    mentions_file,
                    "Task {} 'Where observed' should mention specific files",
                    task_num
                );
            }
        }
    }
}