# Test Suite for ISSUE_TASKS.md

This directory contains comprehensive validation tests for the `ISSUE_TASKS.md` documentation file.

## Running the Tests

To run the tests, use:

```bash
cargo test --test issue_tasks_validator
```

Or to run all tests:

```bash
cargo test
```

## Test Coverage

The test suite (`issue_tasks_validator.rs`) includes 25 comprehensive tests covering:

### Structure and Format Tests
- ✓ File existence and readability
- ✓ Non-empty content validation
- ✓ Proper markdown header structure
- ✓ All 4 required task sections present
- ✓ Consistent task numbering (1-4)
- ✓ Proper task separation with blank lines

### Content Validation Tests
- ✓ Each task has required fields: Task, Why, Where observed, Suggested acceptance criteria
- ✓ Referenced files exist (README.md, src/rust/api/routes.rs, src/rust/transfer/mod.rs, src/rust/main.rs)
- ✓ Task-specific content validation for all 4 tasks
- ✓ Acceptance criteria include bullet points
- ✓ Task descriptions are substantial (not empty placeholders)

### Quality and Edge Case Tests
- ✓ No placeholder content (TODO, FIXME, TBD)
- ✓ No trailing whitespace on lines
- ✓ Document ends with newline
- ✓ File paths use forward slashes (not backslashes)
- ✓ No common typos

### Task-Specific Validation
- ✓ Task 1 (Typo fix): Contains _nerolink._tcp and _neurolink._tcp references
- ✓ Task 2 (Bug fix): Mentions chunk_size, division-by-zero, validation, 400 error
- ✓ Task 3 (Documentation): References port defaults (3030, 8000)
- ✓ Task 4 (Tests): Mentions transfer lifecycle, edge cases, cargo test

### Regression and Boundary Tests
- ✓ Bug fix task specifically mentions validation error
- ✓ Each task has minimum content length (200+ chars)

## Test Philosophy

These tests ensure that:
1. The documentation maintains consistent structure
2. All referenced files actually exist in the codebase
3. Each task follows the established format
4. The content is meaningful and free of placeholders
5. Common documentation issues (trailing spaces, typos) are caught
6. Task-specific requirements are validated

## Adding New Tests

When adding new tasks to ISSUE_TASKS.md:
1. Update the count in `test_issue_tasks_has_all_required_tasks()`
2. Add task-specific content validation following the pattern of existing tests
3. Verify referenced files exist
4. Ensure acceptance criteria are specific and testable