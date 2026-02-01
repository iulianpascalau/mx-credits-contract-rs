This document details the specifications and audit results for the `CreditsContract` smart contract designed for the MultiversX blockchain.

## 1. Contract Overview

**Name:** Credits Contract  
**Purpose:** To allow users to purchase "acquired credits" using EGLD based on a configurable exchange rate.  
**Platform:** MultiversX (Rust Framework)  
**Roles:**
*   **Owner:** Has exclusive rights to change the exchange rate, pause/unpause the contract, and withdraw funds.
*   **User:** Can purchase credits by sending EGLD to the `addCredits` endpoint.

## 2. Technical Specifications

### 2.1 Storage Mappers
*   `numCreditsPerEgld` (`SingleValueMapper<BigUint>`): Stores the current exchange rate (credits per 1 EGLD).
*   `acquiredCredits` (`SingleValueMapper<BigUint>`): Map storing the total credits purchased for a specific `id` (u64).
*   `isPaused` (`SingleValueMapper<bool>`): Boolean flag indicating if the contract is currently paused.

### 2.2 Endpoints

#### `init`
*   **Input:** `num_credits_per_egld` (BigUint)
*   **Logic:**
    *   Sets the initial exchange rate.
    *   Initializes `isPaused` to `false`.
    *   **Constraint:** `num_credits_per_egld` must be > 0.

#### `upgrade`
*   **Input:** `num_credits_per_egld` (BigUint)
*   **Logic:**
    *   Updates the exchange rate.
    *   Preserves the existing `isPaused` state if set; otherwise defaults to `false`.
    *   **Constraint:** `num_credits_per_egld` must be > 0.

#### `addCredits` (Payable)
*   **Input:** `id` (u64)
*   **Payment:** EGLD
*   **Logic:**
    *   Checks if contract is paused.
    *   Calculates credits: `(payment_wei * rate) / 10^18`.
    *   Updates `acquiredCredits` for the given `id`.
    *   Emits `addCredits` event.
    *   **Constraints:**
        *   Contract must not be paused.
        *   Payment > 0.

#### `changeNumCreditsPerEGLD`
*   **Input:** `new_num_credits_per_egld` (BigUint)
*   **Logic:**
    *   Updates the exchange rate.
    *   Emits `changeNumCreditsPerEGLD` event.
    *   **Constraints:**
        *   Caller must be Owner.
        *   New rate must be > 0.

#### `pause` / `unpause`
*   **Logic:** Toggles the `isPaused` storage.
*   **Constraints:**
    *   Caller must be Owner.
    *   Cannot pause if already paused (and vice versa).

#### `withdrawAll`
*   **Logic:** Sends entire EGLD balance to the Owner.
*   **Constraints:**
    *   Caller must be Owner.
    *   Balance > 0.

#### `getCredits` (View)
*   **Input:** `id` (u64)
*   **Output:** Returns the count of credits for the ID.

## 3. Security Audit & Findings

### 3.1 Arithmetic Precision
*   **Status:** **SECURE** (Fixed)
*   **Details:** The contract originally calculated credits as `(amount / 10^18) * rate`, which caused significant precision loss for fractional EGLD amounts. This was patched to `(amount * rate) / 10^18`.
*   **Recommendation:** No further action.

### 3.2 Access Control
*   **Status:** **SECURE**
*   **Details:** Critical functions (`changeNumCreditsPerEGLD`, `pause`, `unpause`, `withdrawAll`) properly enforce `require!(caller == owner)`.

### 3.3 Re-entrancy
*   **Status:** **SECURE**
*   **Details:** The only external call is the EGLD transfer in `withdrawAll` to the owner. Since this is the final step and state variables are not modified after the transfer, re-entrancy risks are minimal.

### 3.4 Logic Constraints
*   **Status:** **SECURE**
*   **Details:**
    *   `init` and `upgrade` enforce non-zero rates to prevent division/logic errors.
    *   `addCredits` enforces non-zero payments.
    *   `pause` mechanism correctly blocks deposits.

### 3.5 Upgrade Safety
*   **Status:** **NOTE**
*   **Details:** The `upgrade` function resets the rate. Owners must be careful to pass the correct (current or new) rate during upgrades to avoid accidental rate changes. It safely handles the `isPaused` flag initialization for older versions of the contract state.

## 4. Recommendations and Improvements

1.  **Rate Limiting / Caps (Optional):** Currently, there is no upper limit on the number of credits a single ID can accumulate. If `credits` represent a resource liability, consider adding a cap.
2.  **Event Data:** The `addCredits` event indexes `id` and `egld_amount`. This is good practice. The `credits_added` is also included, allowing full off-chain reconstruction of the state.

## 5. Conclusion
The `CreditsContract` contains robust logic with appropriate access controls and safeguards. The identification and repair of the arithmetic precision bug was the primary critical finding, which has been resolved and verified via tests.

**Verdict:** The contract is structurally sound and ready for deployment.
