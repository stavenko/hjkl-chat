# Manual Testing Task: Bootstrap and Testing Verification

## Summary

Verify the bootstrap and testing enabler implementation by running the application and collecting evidence for all acceptance criteria.

## Prerequisites

- Docker and docker-compose installed
- Rust toolchain installed
- Chromium browser installed (for frontend verification)

## What to do

### 1. Build the Project

Run the following commands and log the complete output (stdout and stderr) to the report:

```bash
cargo check --workspace
```

Record: SUCCESS or FAILURE

---

```bash
cargo build -p backend
```

Record: SUCCESS or FAILURE

---

```bash
cargo build -p frontend
```

Record: SUCCESS or FAILURE

---

### 2. Start Test Docker Environment

```bash
cd docker/test && docker-compose up -d
```

Log the complete output. Wait 5 seconds for services to initialize.

Record: SUCCESS or FAILURE

---

### 3. Verify MinIO is Running

```bash
curl -s http://localhost:9000/minio/health/live
```

Log the response. Expected: `{"status":"true"}`

Record: SUCCESS or FAILURE

---

### 4. Verify MailHog is Running

```bash
curl -s http://localhost:8025/api/v2/status
```

Log the response. Expected: `{"Status":"OK"}`

Record: SUCCESS or FAILURE

---

### 5. Run Integration Tests

```bash
cargo test
```

Log the complete output including test results.

Record: SUCCESS or FAILURE (all tests passing or ignored counts as success)

---

### 6. Start Local Docker Environment

```bash
cd docker/local && docker-compose up -d
```

Log the complete output. Wait 10 seconds for services to initialize.

Record: SUCCESS or FAILURE

---

### 7. Verify Backend Endpoint

```bash
curl -s http://localhost:8080/health
```

Log the response. Expected: HTTP 200 with response body.

Record: SUCCESS or FAILURE

---

### 8. Verify Frontend Page Renders

```bash
chromium --headless --dump-dom http://localhost:3000 2>&1
```

Log the complete HTML output. Verify it contains valid HTML structure.

Record: SUCCESS or FAILURE

---

### 9. Verify Project Structure

Run these commands and log output:

```bash
ls -la /project/
ls -la /project/backend/src/
ls -la /project/frontend/src/
ls -la /project/common/keyword-extractor/src/
```

Verify:
- No `mod.rs` files exist in any directory
- Module files use the `<name>.rs` pattern (not inline modules)
- Structure matches PROJECT-STRUCTURE.md

Record: SUCCESS or FAILURE

---

### 10. Verify .gitignore

```bash
cat /project/.gitignore
```

Verify it contains:
- `target/`
- IDE files (`.idea/`, `.vscode/`, `*.swp`, etc.)
- OS files (`*.DS_Store`, `Thumbs.db`, etc.)
- Temp test artifacts (`*.db`, `*.sqlite`, `tmp/`, etc.)

Record: SUCCESS or FAILURE

---

### 11. Stop Docker Services

```bash
cd docker/local && docker-compose down
cd docker/test && docker-compose down
```

Log the output.

---

## Report Format

Write the report to `/project/.ralph-wiggum/reports/00-manual-testing-bootstrap.md`

For each acceptance criterion, include:
- The criterion description
- Command(s) run
- Full output (stdout and stderr)
- **PASS** or **FAIL** conclusion

At the end, include a summary table:

| Criterion | Result |
|-----------|--------|
| cargo check --workspace succeeds | PASS/FAIL |
| cargo build -p backend succeeds | PASS/FAIL |
| cargo build -p frontend succeeds | PASS/FAIL |
| docker-compose test starts MinIO and MailHog | PASS/FAIL |
| docker-compose local starts all services | PASS/FAIL |
| cargo test runs at least one integration test | PASS/FAIL |
| Project structure matches PROJECT-STRUCTURE.md | PASS/FAIL |
| Module conventions follow RUST-COMMON-SPEC.md | PASS/FAIL |
| .gitignore exists and covers required artifacts | PASS/FAIL |

## Files to Reference

- User Story: `/project/user-stories/00-bootstrap-and-testing.md`
- Implementation Task: `/project/.ralph-wiggum/tasks/00-task-bootstrap.md`
- Test Task: `/project/.ralph-wiggum/tasks/00-test-bootstrap.md`
- Implementation Report: `/project/.ralph-wiggum/reports/00-task-bootstrap.md`
- Test Report: `/project/.ralph-wiggum/reports/00-test-bootstrap.md`
- Spec: `/project/specs/PROJECT-STRUCTURE.md`
- Spec: `/project/specs/RUST-COMMON-SPEC.md`