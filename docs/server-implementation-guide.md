# Server implementation guide for HTTP 434 with ShadowPay

This document explains how to implement `434 Private Payment Proof Required` on the server side in a ShadowPay aware application.

The scope:

- How to wire 434 into your routing and middleware.  
- How to verify ShadowPay proofs and map errors to HTTP codes.  
- How to keep business logic free from cryptographic details.

---

## 1. Core responsibilities

A server that supports 434 with ShadowPay has the following responsibilities:

1. Identify which routes are protected by ShadowPay.  
2. Inspect incoming requests for ShadowPay proof headers or body fields.  
3. If proof is missing, return `434 Private Payment Proof Required`.  
4. If proof is present, verify it using the ShadowPay verifier and on chain state.  
5. Map verification failures to appropriate HTTP status codes.  
6. Allow the request to reach business logic only when proof is valid.

Payment logic should live in a dedicated middleware, filter, or interceptor, not inside application handlers.

---

## 2. Minimal decision tree

At a high level, the decision tree for a protected route looks like this:

1. Check whether the request targets a ShadowPay protected resource.  
2. If not protected, pass through to normal application handling.  
3. If protected:

   - If required ShadowPay headers are missing, return `434` with a JSON body that describes what is required.  
   - If headers are present, run the ShadowPay proof verifier.  
   - If verification passes, continue to business logic.  
   - If verification fails, return an error status that reflects the reason.

Example pseudocode:

    if not is_shadowpay_protected_route(request.path, request.method):
        return next_handler(request)

    if not has_shadowpay_headers(request.headers):
        return response_434_requirement()

    result = verify_shadowpay_proof(request.headers)

    if result == OK:
        return next_handler(request)
    if result == INVALID_PROOF:
        return response_422_invalid_proof()
    if result == DOUBLE_SPEND:
        return response_409_conflict()
    if result == ESCROW_LOCKED:
        return response_423_locked()
    if result == TIMING_CONDITION:
        return response_425_too_early()
    if result == PRECONDITION_MISSING:
        return response_428_precondition_required()

---

## 3. Required inputs for verification

For ShadowPay, the verifier module needs at least the following values from the request:

- `X-ShadowPay-Proof`  
- `X-ShadowPay-Nullifier`  
- `X-ShadowPay-Merkle-Root`  
- `X-ShadowPay-Invoice-Id`  
- `X-ShadowPay-Escrow-Account` (if escrow is used)  
- `X-ShadowPay-Scheme`  

You can read these from headers and pass them into a verifier function. Optionally you can allow JSON body fields if your API prefers that format.

Example extraction (language agnostic):

    proof        = headers["X-ShadowPay-Proof"]
    nullifier    = headers["X-ShadowPay-Nullifier"]
    merkle_root  = headers["X-ShadowPay-Merkle-Root"]
    invoice_id   = headers["X-ShadowPay-Invoice-Id"]
    escrow_acc   = headers.get("X-ShadowPay-Escrow-Account")
    scheme       = headers.get("X-ShadowPay-Scheme", "shadowpay_v1")

If any mandatory field is missing, treat this as proof not supplied and return 434.

---

## 4. Verification pipeline

A robust ShadowPay verification pipeline on the server side follows these steps:

1. **Syntax validation**

   - Check that all required headers are present.  
   - Check basic formats (for example base64 for proof, hex for merkle root).

2. **Invoice context**

   - Look up `invoice_id` in your ShadowPay invoice store or in ShadowPay API.  
   - Confirm that the invoice exists and is in a state that allows usage (not cancelled, not fully refunded).

3. **Merkle root validation**

   - Confirm that `merkle_root` belongs to the accepted set of ShadowPay roots for the relevant epoch or configuration.  
   - Reject if root is unknown or too old according to your policy.

4. **Proof verification**

   - Use ShadowPay verifier (for example a library or service) to check the Groth16 proof using the given merkle root and verifying key.  
   - Reject on any cryptographic failure.

5. **Nullifier check**

   - Query a persistent store to see whether `nullifier` has been seen before.  
   - If already seen, treat as double spend and reject.  
   - If not seen, atomically mark it as used before returning success.

6. **On chain checks**

   - If `escrow_acc` is present, query Solana or your indexer to confirm that the escrow PDA has sufficient balance and correct token type.  
   - Check time based conditions such as subscription validity windows or timelocks if your scheme uses them.

7. **Return result**

   - Return a structured result such as `OK`, `INVALID_PROOF`, `DOUBLE_SPEND`, `ESCROW_LOCKED`, `TIMING_CONDITION`, or `PRECONDITION_MISSING` to the middleware.

This design keeps the verifier logic self contained and testable.

---

## 5. Mapping verification results to HTTP status codes

After verification, the middleware should map the result into HTTP status codes in a consistent way.

Recommended mapping:

- `OK`  
  - Continue to the protected handler.  
  - Handler returns `2xx` on success.

- `MISSING_PROOF` or missing headers  
  - Return `434 Private Payment Proof Required`.  
  - Include a JSON body with fields such as `proof_type`, `payment_scheme`, and `invoice_id` if available.

- `INVALID_PROOF`  
  - Return `422 Unprocessable Content`.  
  - Include a `detail` field such as `"ShadowPay proof failed verification"`.

- `DOUBLE_SPEND`  
  - Return `409 Conflict`.  
  - Include a `detail` field indicating that the nullifier or invoice was already used.

- `ESCROW_LOCKED`  
  - Return `423 Locked`.  
  - Indicate that escrow state does not allow access yet.

- `TIMING_CONDITION`  
  - Return `425 Too Early`.  
  - Provide optional fields that indicate when retry may be possible.

- `PRECONDITION_MISSING`  
  - Return `428 Precondition Required`.  
  - Describe which pre payment step is missing, for example "escrow funding required".

These codes make it easier for clients and operators to understand exactly what failed.

---

## 6. Example JSON responses

### 6.1 434 Private Payment Proof Required

    HTTP/1.1 434 Private Payment Proof Required
    Content-Type: application/json

    {
      "status": 434,
      "title": "Private Payment Proof Required",
      "detail": "This endpoint requires a valid ShadowPay payment proof.",
      "proof_type": "groth16",
      "payment_scheme": "shadowpay_v1",
      "invoice_id": "inv_abc123",
      "currency": "USDC",
      "amount": "encrypted"
    }

### 6.2 422 Invalid proof

    HTTP/1.1 422 Unprocessable Content
    Content-Type: application/json

    {
      "status": 422,
      "title": "Invalid ShadowPay Proof",
      "detail": "The attached ShadowPay payment proof did not verify."
    }

### 6.3 409 Nullifier conflict

    HTTP/1.1 409 Conflict
    Content-Type: application/json

    {
      "status": 409,
      "title": "ShadowPay Nullifier Conflict",
      "detail": "This ShadowPay nullifier has already been used."
    }

You can add more structured fields if needed, but avoid exposing raw proofs or secrets.

---

## 7. Integration patterns by framework

Common patterns for integrating ShadowPay proof checks:

- **Express (Node)**  
  - Implement `requireShadowPayProof` as a middleware.  
  - Apply it to protected routes or routers.  

- **FastAPI or other Python frameworks**  
  - Implement a dependency function that raises HTTPException for 434 and other codes.  
  - Add it to the `dependencies` of protected endpoints.

- **Axum or other Rust frameworks**  
  - Implement a tower layer that intercepts requests and returns responses for 434 and other codes.  
  - Apply the layer to the router that contains protected routes.

- **Reverse proxies or API gateways**  
  - Implement a plugin or filter that checks for ShadowPay headers and returns 434 directly at the edge.  
  - Upstream services then only see requests that already carry verified proof context.

In all patterns, the idea is the same: keep the ShadowPay gate in a reusable component that can be enabled per route.

---

## 8. Logging and observability

Server side logging should focus on:

- Counts of 434 responses per route.  
- Counts and distribution of 422, 409, 423, 425, and 428 responses.  
- High level reasons for verification failures (for example invalid proof, unknown root, double spend).

Recommendations:

- Do log:
  - Status code.  
  - Route path and method.  
  - Invoice id and payment scheme.  
  - A short error code or reason string.

- Do not log:
  - Raw proofs.  
  - Private keys or secret material.  
  - Full nullifier or merkle root values unless needed for debugging, and even then treat them as sensitive.

Metrics to consider:

- `shadowpay_434_total` by route.  
- `shadowpay_proof_verification_failures_total` by reason.  
- Average verification latency.

These metrics help you detect misconfigurations, version skew between client and server, and potential fraud.

---

## 9. Testing strategy

To ensure correct behavior:

- Unit test the verifier module with:
  - Valid proofs, expecting `OK`.  
  - Corrupted proofs, expecting `INVALID_PROOF`.  
  - Reused nullifiers, expecting `DOUBLE_SPEND`.

- Unit test the middleware with:
  - Requests missing headers, expecting 434.  
  - Requests with dummy valid inputs wired to a stub verifier, expecting pass through to handler.  
  - Requests with verifier errors, expecting correct status mapping.

- Integration test end to end flows with a ShadowPay test environment:
  - Create invoices.  
  - Generate proofs.  
  - Call protected endpoints and confirm 2xx results.

---

## 10. Summary

A correct server side implementation for HTTP 434 with ShadowPay:

- Uses middleware or filters to enforce proof requirements on protected routes.  
- Returns `434` when proof is missing.  
- Verifies ShadowPay proofs with a dedicated module.  
- Maps verification results into `2xx`, `422`, `409`, `423`, `425`, or `428` as appropriate.  
- Keeps business logic independent from payment and proof details.

This pattern keeps private payments maintainable and transparent for both operators and clients.
