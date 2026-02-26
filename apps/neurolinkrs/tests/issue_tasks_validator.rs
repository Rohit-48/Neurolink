use std::fs;
use std::path::{Path, PathBuf};

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn issue_tasks_path() -> PathBuf {
    project_root().join("ISSUE_TASKS.md")
}


/// Test that ISSUE_TASKS.md file exists and is readable
#[test]
fn test_issue_tasks_file_exists() {
    let file_path = "ISSUE_TASKS.md";
    assert!(
        issue_tasks_path().exists(),
        "ISSUE_TASKS.md file should exist in the project root"
    );
}

/// Test that ISSUE_TASKS.md is not empty
#[test]
fn test_issue_tasks_not_empty() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");
    assert!(
        !content.trim().is_empty(),
        "ISSUE_TASKS.md should not be empty"
    );
}

/// Test that ISSUE_TASKS.md has proper markdown structure with header
#[test]
fn test_issue_tasks_has_header() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");
    assert!(
        content.starts_with("# Proposed Fix Tasks"),
        "ISSUE_TASKS.md should start with '# Proposed Fix Tasks' header"
    );
}

/// Test that ISSUE_TASKS.md contains expected task sections
#[test]
fn test_issue_tasks_has_all_required_tasks() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    // Check for all 4 expected task sections
    assert!(content.contains("## 1) Typo fix task"), "Should contain task 1");
    assert!(content.contains("## 2) Bug fix task"), "Should contain task 2");
    assert!(content.contains("## 3) Code comment / documentation discrepancy task"), "Should contain task 3");
    assert!(content.contains("## 4) Test improvement task"), "Should contain task 4");
}

/// Test that each task has required fields
#[test]
fn test_each_task_has_required_fields() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    // Split content into task sections
    let tasks: Vec<&str> = content.split("## ").skip(1).collect();
    assert!(tasks.len() >= 4, "Should have at least 4 tasks");

    for (idx, task) in tasks.iter().enumerate() {
        let task_num = idx + 1;
        assert!(
            task.contains("**Task:**"),
            "Task {} should have a **Task:** field",
            task_num
        );
        assert!(
            task.contains("**Why:**"),
            "Task {} should have a **Why:** field",
            task_num
        );
        assert!(
            task.contains("**Where observed:**"),
            "Task {} should have a **Where observed:** field",
            task_num
        );
        assert!(
            task.contains("**Suggested acceptance criteria:**"),
            "Task {} should have a **Suggested acceptance criteria:** field",
            task_num
        );
    }
}

/// Test that referenced files in ISSUE_TASKS.md exist
#[test]
fn test_referenced_files_exist() {
    let expected_files = vec![
        "README.md",
        "src/rust/api/routes.rs",
        "src/rust/transfer/mod.rs",
        "src/rust/main.rs",
    ];

    for file in expected_files {
        assert!(
            project_root().join(file).exists(),
            "Referenced file {} should exist",
            file
        );
    }
}

/// Test that task 1 (typo fix) has correct content structure
#[test]
fn test_task1_typo_fix_content() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    assert!(content.contains("_nerolink._tcp"));
    assert!(content.contains("_neurolink._tcp"));
    assert!(content.contains("README.md"));
}

/// Test that task 2 (bug fix) has correct content structure
#[test]
fn test_task2_bug_fix_content() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    assert!(content.contains("chunk_size"));
    assert!(content.contains("division-by-zero"));
    assert!(content.contains("src/rust/api/routes.rs"));
    assert!(content.contains("src/rust/transfer/mod.rs"));
}

/// Test that task 3 (documentation discrepancy) has correct content structure
#[test]
fn test_task3_documentation_discrepancy_content() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    assert!(content.contains("default port"));
    assert!(content.contains("3030"));
    assert!(content.contains("8000"));
    assert!(content.contains("src/rust/main.rs"));
}

/// Test that task 4 (test improvement) has correct content structure
#[test]
fn test_task4_test_improvement_content() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    assert!(content.contains("transfer lifecycle"));
    assert!(content.contains("edge cases"));
    assert!(content.contains("cargo test"));
}

/// Test that each task has acceptance criteria with bullet points
#[test]
fn test_tasks_have_acceptance_criteria_bullets() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    let tasks: Vec<&str> = content.split("## ").skip(1).collect();

    for (idx, task) in tasks.iter().enumerate() {
        let task_num = idx + 1;

        // After acceptance criteria, there should be bullet points (lines starting with "  -")
        if let Some(criteria_pos) = task.find("**Suggested acceptance criteria:**") {
            let after_criteria = &task[criteria_pos..];
            assert!(
                after_criteria.contains("  -"),
                "Task {} should have bullet points in acceptance criteria",
                task_num
            );
        }
    }
}

/// Test that the document has consistent formatting
#[test]
fn test_consistent_task_numbering() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    // Check that tasks are numbered 1, 2, 3, 4
    for i in 1..=4 {
        let expected = format!("## {}) ", i);
        assert!(
            content.contains(&expected),
            "Should contain task numbered '{}'",
            expected
        );
    }
}

/// Test that no tasks have TODO or placeholder text
#[test]
fn test_no_placeholder_content() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    let placeholder_patterns = vec!["TODO", "FIXME", "XXX", "[TBD]", "placeholder"];

    for pattern in placeholder_patterns {
        assert!(
            !content.contains(pattern),
            "Document should not contain placeholder text: {}",
            pattern
        );
    }
}

/// Test that task descriptions are not empty
#[test]
fn test_task_descriptions_not_empty() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    let tasks: Vec<&str> = content.split("## ").skip(1).collect();

    for (idx, task) in tasks.iter().enumerate() {
        let task_num = idx + 1;

        // Find the Task: field content
        if let Some(task_start) = task.find("**Task:**") {
            let after_task = &task[task_start + 9..];
            if let Some(next_section) = after_task.find("\n\n") {
                let task_desc = after_task[..next_section].trim();
                assert!(
                    task_desc.len() > 10,
                    "Task {} description should be substantial (more than 10 chars)",
                    task_num
                );
            }
        }
    }
}

/// Test that referenced code paths use forward slashes (not backslashes)
#[test]
fn test_file_paths_use_forward_slashes() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    // Check that any file path patterns don't use backslashes
    let lines_with_src: Vec<&str> = content.lines()
        .filter(|line| line.contains("src/"))
        .collect();

    for line in lines_with_src {
        if line.contains("src") {
            assert!(
                !line.contains("src\\"),
                "File paths should use forward slashes, not backslashes: {}",
                line
            );
        }
    }
}

/// Edge case: Test that document ends with a newline
#[test]
fn test_document_ends_with_newline() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    assert!(
        content.ends_with('\n'),
        "ISSUE_TASKS.md should end with a newline character"
    );
}

/// Edge case: Test that there are no trailing spaces at end of lines
#[test]
fn test_no_trailing_spaces() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    let lines_with_trailing_spaces: Vec<(usize, &str)> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.ends_with(' ') || line.ends_with('\t'))
        .collect();

    assert!(
        lines_with_trailing_spaces.is_empty(),
        "Lines should not have trailing whitespace. Found at lines: {:?}",
        lines_with_trailing_spaces.iter().map(|(n, _)| n + 1).collect::<Vec<_>>()
    );
}

/// Negative test: Verify document doesn't contain common typos
#[test]
fn test_no_common_typos() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    let common_typos = vec![
        "teh ",
        "recieve",
        "seperate",
        "occured",
        "untill",
    ];

    for typo in common_typos {
        assert!(
            !content.to_lowercase().contains(typo),
            "Document contains typo: {}",
            typo
        );
    }
}

/// Regression test: Ensure specific bug fix task mentions validation error
#[test]
fn test_bug_fix_mentions_validation() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    // Task 2 should mention validation error
    let task2_start = content.find("## 2) Bug fix task").unwrap();
    let task2_end = content[task2_start..].find("## 3)").unwrap_or(content.len() - task2_start);
    let task2 = &content[task2_start..task2_start + task2_end];

    assert!(
        task2.contains("validation"),
        "Bug fix task should mention validation"
    );
    assert!(
        task2.contains("400"),
        "Bug fix task should mention 400 error code"
    );
}

/// Boundary test: Verify each task has minimum required content length
#[test]
fn test_tasks_have_minimum_content_length() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    let tasks: Vec<&str> = content.split("## ").skip(1).collect();

    for (idx, task) in tasks.iter().enumerate() {
        let task_num = idx + 1;
        assert!(
            task.len() > 200,
            "Task {} should have substantial content (at least 200 chars), has {}",
            task_num,
            task.len()
        );
    }
}

/// Test that tasks are properly separated by blank lines
#[test]
fn test_tasks_separated_by_blank_lines() {
    let content = fs::read_to_string(issue_tasks_path())
        .expect("Failed to read ISSUE_TASKS.md");

    // Check that each task section (except first) is preceded by blank line
    for i in 2..=4 {
        let task_marker = format!("\n## {}) ", i);
        assert!(
            content.contains(&task_marker),
            "Task {} should be preceded by a newline",
            i
        );
    }
}