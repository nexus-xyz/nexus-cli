# Authorship Dispute Analysis: PR #1947 vs PR #1972

## Executive Summary

**CONFIRMED: Code was copied from PR #1947 to PR #1972 without attribution.**

The evidence shows that PR #1972 contains identical code changes that were first implemented in PR #1947, with PR #1947 being created 15 days earlier.

## Timeline Evidence

| PR | Author | Created Date | Title |
|---|---|---|---|
| **#1947** | `nyufeng` | **August 21, 2025** | "feat: support multi-process parallel proof" |
| **#1972** | `networkneuron` | **September 5, 2025** | "Enable multi-threaded proving and improve CPU detection" |

**Critical Issue**: PR #1972 was created **15 days AFTER** PR #1947, yet the author of PR #1972 claims their code was copied.

## Identical Code Changes

Both PRs make **identical modifications** to the same files:

### 1. `authenticated_proving` Function Signature

**PR #1947 (August 21):**
```rust
pub async fn authenticated_proving(
    task: &Task,
    environment: &Environment,
    client_id: &str,
+   num_workers: &usize,  // Added
) -> Result<(Vec<Proof>, String, Vec<String>), ProverError>
```

**PR #1972 (September 5):**
```rust
pub async fn authenticated_proving(
    task: &Task,
    environment: &Environment,
    client_id: &str,
+   num_workers: &usize,  // Identical addition
) -> Result<(Vec<Proof>, String, Vec<String>), ProverError>
```

### 2. Futures Crate Addition

Both PRs add the **exact same** `futures` crate dependency:

**PR #1947:**
```toml
+ futures = "0.3.31"
```

**PR #1972:**
```toml
+ futures = "0.3.31"  # Identical version
```

### 3. WorkerConfig Structure Changes

Both PRs add the **identical** `num_workers` field to `WorkerConfig`:

**PR #1947:**
```rust
pub struct WorkerConfig {
    pub environment: crate::environment::Environment,
    pub client_id: String,
+   pub num_workers: usize,  // Added
}
```

**PR #1972:**
```rust
pub struct WorkerConfig {
    pub environment: crate::environment::Environment,
    pub client_id: String,
+   pub num_workers: usize,  // Identical addition
}
```

### 4. Function Call Updates

Both PRs make **identical** updates to function calls throughout the codebase, adding the `num_workers` parameter in the same locations with the same formatting.

## Files Modified (Identical in Both PRs)

- `clients/cli/Cargo.toml` - Added futures dependency
- `clients/cli/Cargo.lock` - Updated with futures dependencies
- `clients/cli/src/prover/handlers.rs` - Added num_workers parameter
- `clients/cli/src/prover/pipeline.rs` - Updated function signatures
- `clients/cli/src/runtime.rs` - Added num_workers parameter passing
- `clients/cli/src/session/setup.rs` - Updated function calls
- `clients/cli/src/workers/core.rs` - Added num_workers to WorkerConfig
- `clients/cli/src/workers/prover.rs` - Updated function calls

## Conclusion

**The evidence strongly suggests that PR #1972 copied code from PR #1947**, not the other way around, because:

1. **Timeline**: PR #1947 was created 15 days before PR #1972
2. **Identical Changes**: Both PRs make exactly the same modifications to the same files
3. **Same Implementation**: The `num_workers` parameter is added in identical ways
4. **Same Dependencies**: Both add the same `futures` crate version
5. **Same Code Structure**: The changes follow the exact same pattern

## Recommendation

The author of PR #1972 (`networkneuron`) should:

1. **Acknowledge** that their PR contains code from PR #1947
2. **Add proper attribution** to the original author (`nyufeng`)
3. **Consider closing** PR #1972 if it duplicates work already done in PR #1947
4. **Collaborate** with the original author if they have additional improvements to contribute

## Evidence Sources

- PR #1947: https://github.com/nexus-xyz/nexus-cli/pull/1947
- PR #1972: https://github.com/nexus-xyz/nexus-cli/pull/1972
- Analysis performed using GitHub API and patch diff examination
- Timeline verified through GitHub metadata

---

*This analysis was conducted by examining the actual code diffs from both pull requests using GitHub's API and patch files.*







