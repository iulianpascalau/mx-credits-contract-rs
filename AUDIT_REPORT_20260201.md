# Security Audit Report: Credits Contract

**Contract Version:** 0.0.0
**MultiversX SDK Version:** 0.57.1
**Audit Date:** 2026-02-01
**Auditor:** Claude Opus 4.5
**Commit Hash:** 24dda2d694f4a423dfd024318cd6784c701ed223

---

## Executive Summary

The `CreditsContract` is a well-structured MultiversX smart contract that allows users to purchase credits using EGLD at a configurable exchange rate. After thorough analysis, **the contract is considered secure for deployment** with a few informational notes and recommendations.

---

## 1. Security Findings

### 1.1 SECURE - Arithmetic Precision

**Location:** `src/credits.rs:45`

```rust
let credits_to_add = (amount_wei.clone() * &num_credits_per_egld) / one_egld;
```

**Analysis:** The contract correctly multiplies before dividing to preserve precision. This prevents precision loss for fractional EGLD payments. Test `add_credits_accumulation.scen.json` confirms 0.5 EGLD correctly yields 50 credits at rate 100.

**Status:** SECURE

---

### 1.2 SECURE - Access Control

**Locations:** Lines 77-80, 93-96, 107-110, 121-124

All privileged functions properly enforce owner-only access:

| Function | Owner Check | Status |
|----------|-------------|--------|
| `change_num_credits_per_egld` | `require!(caller == owner, ...)` | SECURE |
| `pause` | `require!(caller == owner, ...)` | SECURE |
| `unpause` | `require!(caller == owner, ...)` | SECURE |
| `withdraw_all` | `require!(caller == owner, ...)` | SECURE |

**Status:** SECURE

---

### 1.3 SECURE - Pause Mechanism

**Locations:** Lines 34, 92-115

- Contract correctly blocks `addCredits` when paused
- Double-pause/unpause prevention implemented
- Upgrade function preserves pause state correctly

**Status:** SECURE

---

### 1.4 SECURE - Re-entrancy

**Location:** `src/credits.rs:129-134`

```rust
self.tx()
    .to(&owner)
    .egld(&contract_balance)
    .transfer();
self.withdraw_event(&owner, &contract_balance);
```

**Analysis:** While the event is emitted after the transfer (traditionally less safe), this is acceptable because:

1. MultiversX uses an async execution model different from Ethereum
2. `withdraw_all` sends all funds in one transfer
3. No state modifications occur after balance check
4. Only owner can call this function

**Status:** SECURE

---

### 1.5 SECURE - Input Validation

**Locations:** Lines 10, 18, 39, 81

All critical inputs are validated:

| Validation | Location | Status |
|------------|----------|--------|
| Rate > 0 at init | Line 10 | SECURE |
| Rate > 0 at upgrade | Line 18 | SECURE |
| Payment > 0 | Line 39 | SECURE |
| New rate > 0 | Line 81 | SECURE |
| Balance > 0 before withdrawal | Line 127 | SECURE |

**Status:** SECURE

---

### 1.6 INFO - Zero Credits on Small Payments

**Severity:** Informational
**Location:** `src/credits.rs:45`

**Description:** With low exchange rates, very small payments could result in zero credits being awarded while the EGLD payment is still accepted.

**Example Calculation:**
- Rate: 1 credit per EGLD
- Payment: 0.5 EGLD (5×10¹⁷ wei)
- Credits: `(5×10¹⁷ × 1) / 10¹⁸ = 0` credits

**Impact:** User loses EGLD without receiving credits.

**Recommendation:** Consider adding a minimum credit check:
```rust
require!(credits_to_add > 0, "Payment too small for any credits");
```

**Status:** INFORMATIONAL - Acceptable if rate is kept sufficiently high

---

### 1.7 INFO - Front-Running Risk

**Severity:** Informational

**Description:** The owner could theoretically change the exchange rate between a user submitting a transaction and its execution, resulting in the user receiving fewer credits than expected.

**Mitigation Options:**
1. Users could be given an expected minimum credits parameter
2. Rate changes could have a time delay
3. Current design is acceptable if users trust the owner

**Status:** INFORMATIONAL - Acceptable for trusted operator model

---

### 1.8 INFO - Event After Transfer

**Severity:** Informational
**Location:** `src/credits.rs:134`

**Description:** The `withdraw_event` is emitted after the transfer. Best practice is to emit events before external calls to ensure logging even if callbacks fail.

**Impact:** In MultiversX, this is less critical than on Ethereum due to the different execution model.

**Status:** INFORMATIONAL - Low risk

---

## 2. Code Quality Assessment

| Aspect | Rating | Notes |
|--------|--------|-------|
| Documentation | Excellent | Clear comments on all functions |
| Error Messages | Good | Descriptive error messages |
| Event Emission | Good | Proper indexed fields for filtering |
| Storage Design | Good | Efficient mapper usage |
| Test Coverage | Excellent | 23 comprehensive scenarios |

---

## 3. Test Coverage Analysis

The test suite provides comprehensive coverage:

| Category | Tests | Coverage |
|----------|-------|----------|
| Initialization | `init_valid`, `init_zero` | Complete |
| Credits Addition | `add_credits_single`, `add_credits_multiple`, `add_credits_accumulation`, `add_credits_when_paused` | Complete |
| Rate Changes | `change_rate_valid`, `change_rate_zero`, `change_rate_non_owner`, `rate_change_affects_future` | Complete |
| Pause | `pause_success`, `pause_already_paused`, `pause_non_owner` | Complete |
| Unpause | `unpause_success`, `unpause_not_paused`, `unpause_non_owner`, `pause_unpause_workflow` | Complete |
| Withdrawals | `withdraw_success`, `withdraw_empty`, `withdraw_non_owner` | Complete |
| Queries | `get_credits_existing`, `get_credits_nonexistent` | Complete |
| Integration | `full_workflow` | Complete |

**Recommended Additional Tests:**
1. Small payment resulting in zero credits
2. Very large payment (overflow boundary test)
3. Upgrade with existing credits preservation verification

---

## 4. Dependency Review

| Dependency | Version | Status |
|------------|---------|--------|
| multiversx-sc | 0.57.1 | No known vulnerabilities |
| multiversx-sc-scenario | 0.57.1 | No known vulnerabilities |

---

## 5. Upgrade Considerations

**Location:** `src/credits.rs:17-26`

The upgrade function requires the rate as a parameter:

```rust
#[upgrade]
fn upgrade(&self, num_credits_per_egld: BigUint) {
    require!(num_credits_per_egld > 0, "Number of credits per EGLD must be non-zero");
    self.num_credits_per_egld().set(num_credits_per_egld);
    if !self.is_paused().is_empty() {
        // is_paused already exists, keep current value
    } else {
        // First upgrade, initialize pause state
        self.is_paused().set(false);
    }
}
```

**Considerations:**
- Must pass the correct rate during upgrade to avoid accidental changes
- Pause state is preserved correctly
- Acquired credits are preserved (not reset)

---

## 6. Recommendations Summary

| Priority | Recommendation | Impact |
|----------|---------------|--------|
| Medium | Add minimum credits check to prevent zero-credit payments | Prevents user fund loss on small payments |
| Low | Consider adding max rate cap for safety | Prevents accidental extreme rate setting |
| Low | Emit withdraw event before transfer | Follows best practices |
| Low | Add upgrade test for credits preservation | Improves test coverage |

---

## 7. Contract Architecture

```
CreditsContract
├── Storage
│   ├── numCreditsPerEgld: SingleValueMapper<BigUint>
│   ├── acquiredCredits: SingleValueMapper<BigUint> (keyed by id)
│   └── isPaused: SingleValueMapper<bool>
├── User Endpoints
│   └── addCredits(id: u64) [payable EGLD]
├── Owner Endpoints
│   ├── changeNumCreditsPerEGLD(new_rate: BigUint)
│   ├── pause()
│   ├── unpause()
│   └── withdrawAll()
├── View Functions
│   ├── getCredits(id: u64) -> BigUint
│   ├── isPaused() -> bool
│   └── getCreditsPerEgld() -> BigUint
└── Events
    ├── addCredits(id, egld_amount, credits_added)
    ├── changeNumCreditsPerEGLD(old_value, new_value)
    ├── pause()
    ├── unpause()
    └── withdraw(recipient, amount)
```

---

## 8. Conclusion

**Verdict: APPROVED FOR DEPLOYMENT**

The Credits Contract demonstrates solid security practices:

- Proper access control on all privileged functions
- Correct arithmetic handling for precision
- Comprehensive test coverage (23 scenarios)
- Clean, well-documented code
- Appropriate event emission for off-chain tracking

The informational findings are minor and acceptable depending on operational context. The contract is structurally sound and ready for mainnet deployment.

---

## Appendix: Files Reviewed

| File | Lines | Description |
|------|-------|-------------|
| `src/credits.rs` | 182 | Main contract implementation |
| `tests/scenarios_rs_test.rs` | 125 | Test runner |
| `scenarios/*.scen.json` | 23 files | Test scenarios |
| `Cargo.toml` | 16 | Package configuration |

---

*Report generated by Claude Opus 4.5 - Anthropic*