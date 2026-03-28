# CI/CD Workflows Structure

## File Organization

This structure provides better understanding and maintenance of GitHub Actions workflows.

### Main Workflows

| File                      | Responsibility | Description                               |
| ------------------------- | -------------- | ----------------------------------------- |
| `ci.yml`                  | Orchestrator   | Main pipeline entry point                 |
| `format-check.yml`        | Formatting     | Rust code formatting verification         |
| `security-analysis.yml`   | Security       | CodeQL analysis and security              |
| `comprehensive-tests.yml` | Tests          | Unit and integration tests                |
| `build-validation.yml`    | Build          | Release mode build validation             |
| `deployment.yml`          | Deployment     | Conditional production/staging deployment |

### Strict Rules Applied

#### **Direct Push to Master - FORBIDDEN**

- **No workflow runs on direct master push**
- **Master branch protected against direct pushes**

#### **PR Merge to Master**

- Code formatting
- Complete security analysis
- Full tests (unit + integration)
- Build validation
- **Automatic PRODUCTION deployment** after merge

#### **Other Branches (develop, feature/_, hotfix/_)**

- Code formatting
- Complete security analysis
- Full tests (unit + integration)
- Build validation
- **STAGING deployment** (if all validations pass)

### Validation Gates

For non-master branches, no deployment can occur without:

1. **Valid formatting** - `cargo fmt --check`
2. **Validated security** - CodeQL analysis
3. **Successful tests** - Complete `cargo test`
4. **Successful build** - `cargo build --release`

### Deployment Flow

```
Direct Master Push → FORBIDDEN
  ↓
PR to Master → Format → Security → Tests → Build → Merge → Production
  ↓
Push other branches → Format → Security → Tests → Build → Staging
```

### Required Configuration

1. **GitHub Environments**:
   - `production` (protected)
   - `staging` (development)

2. **Master Branch Protection**:
   - Check "Require pull request reviews before merging"
   - Check "Require status checks to pass before merging"
   - Check "Include administrators"
   - Add all workflows as "Required status checks"

3. **Required Permissions**:
   - `actions: read`
   - `contents: read`
   - `security-events: write`

4. **Rust Dependencies**:
   - `rustfmt` for formatting
   - `clippy` for linting
   - `cargo test` for testing

### Maintenance

Each workflow is independent and can be modified separately. The orchestrator (`ci.yml`) serves as the entry point and documentation for the global pipeline.

### Maximum Security

- **Master branch**: Fully protected, no direct push
- **PR required**: All modifications must go through PR
- **Complete validation**: Tests and security before any merge
- **Automated deployment**: Production only after complete validation
