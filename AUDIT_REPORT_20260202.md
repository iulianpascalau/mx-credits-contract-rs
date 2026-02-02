# Security Audit Report: Migration Functionality

**Contract:** mx-credits-contract-rs
**Audit Date:** 2026-02-02
**Auditor:** Claude (AI-assisted review)
**Scope:** Migration endpoint and related functionality
**Branch:** add-migrate-functionality
**Commit Hash:** e0082239bc8f171a29f7f424ec609242dd3bee37

---

## Executive Summary

This audit focused on the new `migrate` endpoint introduced to handle the transition from the old `requests` contract to the new `credits` contract. The migration is necessary because the storage mapper was renamed, which would leave existing users' credits inaccessible after a contract upgrade.

Two critical issues were identified and fixed during the audit.

---

## Findings

### 1. CRITICAL: Storage Mapper Key Mismatch

**Severity:** Critical
**Status:** Fixed
**Location:** `src/credits.rs:159`

**Description:**
The old contract stored user data using the storage key `"acquiredRequests"`:

```rust
// Old contract (requests/src/lib.rs:175-176)
#[storage_mapper("acquiredRequests")]
fn acquired_requests(&self, id: &u64) -> SingleValueMapper<BigUint>;
```

However, the migration function was reading from a different storage key `"requests"`:

```rust
// New contract - BEFORE fix
#[storage_mapper("requests")]
fn old_requests(&self, id: &u64) -> SingleValueMapper<BigUint>;
```

**Impact:**
The migration would read from `str:requests|u64:<id>` but actual data exists at `str:acquiredRequests|u64:<id>`. Migration would find no data and complete without migrating any credits, resulting in permanent loss of user credits.

**Fix Applied:**
```rust
// New contract - AFTER fix
#[storage_mapper("acquiredRequests")]
fn old_requests(&self, id: &u64) -> SingleValueMapper<BigUint>;
```

---

### 2. HIGH: Non-Contiguous ID Handling

**Severity:** High
**Status:** Fixed
**Location:** `src/credits.rs:146-155`

**Description:**
The original implementation used early exit logic that stopped iteration upon encountering the first empty ID:

```rust
// BEFORE fix
for id in start..=end {
    let old_val = self.old_requests(&id).get();
    if old_val > BigUint::zero() {
        self.acquired_credits(&id).update(|credits| *credits += old_val.clone());
        self.old_requests(&id).clear();
    } else {
        self.migration_finished_event(&id);
        break;  // Stops on first empty ID
    }
}
```

The old contract allows users to specify any arbitrary `id` when calling `addRequests`. IDs are not guaranteed to be sequential.

**Impact:**
If users chose non-contiguous IDs (e.g., 1, 5, 10), migration would stop at the first gap, leaving subsequent users' credits unmigrated.

**Example:**
- User A: ID 1 = 100 credits
- User B: ID 5 = 200 credits
- User C: ID 10 = 50 credits

Calling `migrate(1, 100)` would only migrate User A, then stop at ID 2 (empty).

**Fix Applied:**
```rust
// AFTER fix
for id in start..=end {
    let old_val = self.old_requests(&id).get();
    if old_val > BigUint::zero() {
        self.acquired_credits(&id).update(|credits| *credits += old_val.clone());
        self.old_requests(&id).clear();
    }
    // Continue checking all IDs in range - no early exit
}
```

The `migrationFinished` event was also removed as it no longer serves a purpose.

---

### 3. LOW: Missing Range Validation

**Severity:** Low
**Status:** Not Fixed (Accepted)
**Location:** `src/credits.rs:140`

**Description:**
No validation that `start <= end`. If `start > end`, the range `start..=end` produces an empty iterator and nothing happens.

**Impact:**
No security impact. Owner would simply need to call again with correct parameters.

**Recommendation:**
Consider adding validation for better UX:
```rust
require!(start <= end, "Invalid range: start must be <= end");
```

---

### 4. MEDIUM: No Migration Progress Tracking

**Severity:** Medium
**Status:** Not Fixed (Accepted)
**Location:** `src/credits.rs:137-156`

**Description:**
There is no on-chain storage tracking which IDs have been migrated. For large migrations that require multiple batches due to gas limits, the owner must track progress off-chain.

**Impact:**
Operational complexity for the contract owner. No security impact as double-migration is prevented by clearing old values.

**Recommendation:**
For production use, maintain an off-chain record of migrated ranges or query `addRequests` events from the old contract to determine the set of IDs to migrate.

---

## Positive Findings

| Finding | Status |
|---------|--------|
| Owner-only access control on `migrate` | Correct |
| Pause requirement before migration | Correct |
| Double-migration prevention via `clear()` | Correct |
| Additive migration (`+=`) preserves post-upgrade credits | Correct |
| Contract must be paused during migration | Correct |

---

## Test Coverage

The migration scenario test (`scenarios/migration_test.scen.json`) covers:

1. Deploying old contract and adding requests for multiple users
2. Contract upgrade to new credits contract
3. Adding new credits post-upgrade (pre-migration)
4. Pausing contract before migration
5. Executing migration
6. Verifying migrated credits = old requests + new credits

**Note:** The test uses a `setState` workaround (step 10) due to a framework bug that doesn't properly preserve storage during upgrade simulation. This is documented with a TODO comment. The storage keys in this workaround were updated to use the correct `acquiredRequests` key.

---

## Files Modified

| File | Changes |
|------|---------|
| `src/credits.rs` | Fixed storage key, removed early exit logic, removed `migrationFinished` event |
| `scenarios/migration_test.scen.json` | Updated storage keys from `requests` to `acquiredRequests` |

---

## Recommendations for Production Migration

1. **Determine ID Range:** Before calling `migrate`, analyze historical `addRequests` events from the old contract to identify all IDs that have credits. This ensures the migration range covers all users.

2. **Gas Considerations:** If there are many IDs to migrate, consider batching:
   - Call `migrate(1, 1000)`, then `migrate(1001, 2000)`, etc.
   - Monitor gas usage and adjust batch size accordingly

3. **Verification:** After migration, verify a sample of known accounts to confirm credits were migrated correctly.

4. **Pause Duration:** Keep the contract paused only for the duration of migration to minimize disruption to users.

---

## Conclusion

Two critical/high severity issues were identified and fixed. The migration functionality is now safe for production use, provided the owner carefully determines the correct ID range to migrate.

All 24 tests pass after the fixes were applied.
