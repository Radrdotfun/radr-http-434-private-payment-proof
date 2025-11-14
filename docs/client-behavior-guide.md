# Client behavior for HTTP 434 Private Payment Proof Required

This document describes how HTTP clients and SDKs should behave when they receive
`434 Private Payment Proof Required` responses, with a focus on the ShadowPay
profile.

The goal is:

- Make client behavior predictable and easy to implement  
- Keep cryptographic and on chain details inside ShadowPay SDKs  
- Let application code work in terms of HTTP status codes and high level calls  

---

## 1. Scope

This guide covers:

- Generic behavior for any client that understands HTTP 434  
- ShadowPay specific behavior when 434 is used with the ShadowPay profile  
- Error handling and mapping of follow up responses  

It does not define the 434 status code itself. That is done in the core 434
specification. It also does not define ShadowPay cryptography; that is handled
by ShadowPay protocol documentation and SDKs.

---

## 2. Generic client behavior for HTTP 434

Any client that understands 434, independent of ShadowPay, SHOULD follow this
pattern.

1. Detect `434 Private Payment Proof Required`.  
2. Parse the response body to extract:
   - payment scheme identifier  
   - payment context identifier (for example invoice or session id)  
   - optional metadata about the resource or plan  

3. Decide whether the client is allowed to pay for this resource:
   - check application configuration  
   - check user consent or policy  
   - check spending limits  

4. If paying is allowed:
   - use the relevant private payment system to make sure a valid payment
     exists for the given context  
   - generate a private payment proof linked to that context  

5. Retry the original request, attaching the proof in headers or body fields
   defined by the payment system.

6. Interpret the new response:
   - `2xx` means success and the resource is available  
   - `4xx` codes such as `422`, `409`, `423`, `425`, `428` indicate specific
     proof or payment issues  
   - another `434` usually means a configuration or integration problem  

Clients that do not understand 434 treat it as a generic 4xx error.

---

## 3. ShadowPay specific behavior

When the response is produced by a ShadowPay aware server using the ShadowPay
profile, clients and SDKs SHOULD follow the ShadowPay specific rules in this
section.

### 3.1. Detecting ShadowPay 434 responses

A 434 response can come from any private payment system. To decide whether to
use ShadowPay, a client SHOULD inspect the response body fields.

Common patterns that indicate a ShadowPay profile:

- `payment_scheme` field contains a value such as `shadowpay_v1`  
- Additional fields like `invoice_id`, `currency`, `amount`, and
  `escrow_account` match ShadowPay conventions  
- Documentation for the endpoint states that it uses ShadowPay  

Typical response body:

    {
      "status": 434,
      "title": "Private Payment Proof Required",
      "detail": "This endpoint requires a valid ShadowPay payment proof.",
      "proof_type": "groth16",
      "payment_scheme": "shadowpay_v1",
      "invoice_id": "inv_abc123",
      "currency": "USDC",
      "amount": "encrypted",
      "escrow_account": "EscrowPDA1111111111111111111111111111111",
      "metadata": {
        "resource": "/v1/chat",
        "plan": "pro",
        "interval": "monthly"
      }
    }

If `payment_scheme` matches a ShadowPay scheme that the client supports, the
client SHOULD route handling to a ShadowPay SDK.

### 3.2. Decision to pay

Before engaging ShadowPay, a client SHOULD decide whether payment is allowed.

Inputs into this decision:

- Allow lists of hosts and paths that are allowed to trigger ShadowPay  
- Maximum price or spend per operation or per day  
- User consent or configuration for subscriptions and one shot payments  
- Application specific rules such as required scopes or plans  

A typical pattern:

- Back office or CLI tools ask for explicit user confirmation  
- Agents and services follow a policy configuration file  
- Browser and mobile apps store user settings for each domain  

If payment is not allowed, the client SHOULD:

- Not retry automatically  
- Surface an error containing the 434 status and the `detail` text  

### 3.3. Interaction with ShadowPay SDKs

Once the client decides that paying is acceptable, it SHOULD delegate to a
ShadowPay SDK. ShadowPay SDKs are expected to provide helpers that:

- Look up or create invoices  
- Confirm that payments exist or are pending  
- Generate Groth16 proofs with the correct merkle root and nullifier  
- Return the header values that must be attached to the retry request  

A typical high level flow in pseudocode:

    const requirement = await resp.json(); // 434 response body
    
    const decision = await shadowpay.decide({
      scheme: requirement.payment_scheme,
      invoiceId: requirement.invoice_id,
      currency: requirement.currency,
      metadata: requirement.metadata
    });
    
    if (!decision.shouldPay) {
      throw new Error("ShadowPay payment not allowed for this request");
    }
    
    const proof = await shadowpay.generateProof({
      invoiceId: requirement.invoice_id,
      escrowAccount: requirement.escrow_account,
      scheme: requirement.payment_scheme
    });
    
    // Retry with proof attached
    const retried = await fetch(originalUrl, {
      method: originalMethod,
      headers: {
        ...originalHeaders,
        "X-ShadowPay-Proof": proof.proofBase64,
        "X-ShadowPay-Nullifier": proof.nullifier,
        "X-ShadowPay-Merkle-Root": proof.merkleRoot,
        "X-ShadowPay-Invoice-Id": proof.invoiceId,
        "X-ShadowPay-Escrow-Account": proof.escrowAccount,
        "X-ShadowPay-Scheme": proof.scheme
      },
      body: originalBody
    });

Application code SHOULD treat the ShadowPay SDK as the main entry point for all
cryptographic and on chain operations.

---

## 4. Required ShadowPay headers on retry

When retrying a request after satisfying a ShadowPay 434 requirement, the client
MUST attach proof material using the header fields defined by the ShadowPay
profile.

Standard headers:

- `X-ShadowPay-Proof`  
  Base64 encoded Groth16 proof.

- `X-ShadowPay-Nullifier`  
  Encoded nullifier that the verifier will record and check for reuse.

- `X-ShadowPay-Merkle-Root`  
  Hex encoded merkle root used when generating the proof.

- `X-ShadowPay-Invoice-Id`  
  ShadowPay invoice or payment session identifier.

- `X-ShadowPay-Escrow-Account`  
  Optional. Solana escrow account if the scheme uses escrow.

- `X-ShadowPay-Scheme`  
  Logical scheme or version name such as `shadowpay_v1`.

Clients SHOULD treat these headers as sensitive and avoid logging them by
default.

---

## 5. Handling follow up responses

After retrying a request with ShadowPay proof headers, the client must interpret
the follow up status code. The mapping below assumes a ShadowPay aware server.

- `2xx`  
  Proof accepted and request processed. The client can treat this as success.

- `434` again  
  Server still believes proof is missing or unusable. This usually indicates a
  configuration or integration issue, for example:
  - proof headers not attached correctly  
  - reverse proxy stripping custom headers  
  - server has not enabled ShadowPay middleware on this route  

  The client SHOULD log enough information for debugging and surface an error
  rather than looping.

- `422 Unprocessable Content`  
  Proof was present but invalid or malformed. Possible reasons:
  - base64 or encoding errors  
  - merkle root not recognized  
  - Groth16 verification failure  
  - invoice id or scheme mismatch  

  The client MAY attempt a single regeneration if the error appears transient,
  but repeated `422` responses SHOULD be treated as a configuration problem that
  needs developer attention.

- `409 Conflict`  
  Nullifier or payment reference already used. This indicates a replay or double
  spend attempt or a bug in proof reuse. Clients MUST NOT retry with the same
  proof and SHOULD surface a hard error.

- `423 Locked`  
  Funds or entitlements exist but are locked. The client MAY support delayed
  retry strategies if the response body includes information about unlock
  conditions, otherwise it SHOULD treat this as an error.

- `425 Too Early`  
  Time based condition not satisfied, for example a timelock or subscription
  start time. Clients MAY:
  - read a suggested retry time from the response body  
  - schedule a follow up call  
  - or surface an error  

- `428 Precondition Required`  
  Indicates that an upstream precondition such as escrow funding or plan
  activation is missing. Clients SHOULD treat this as a configuration or setup
  issue rather than repeatedly retrying the same request.

ShadowPay SDKs SHOULD expose these as structured error types or result variants
so that application code does not work directly with raw status codes.

---

## 6. Behavior in non ShadowPay contexts

If a client supports multiple private payment systems, it MUST treat 434 as a
generic signal initially and then route to the correct profile based on the
response body and configuration.

If a 434 response does not match any known profile:

- The client SHOULD treat it as an unsupported payment scheme.  
- The client SHOULD surface an error with the status code and `detail` field.  
- The client SHOULD NOT attempt ShadowPay proof generation when the scheme does
  not match a known ShadowPay value.

This prevents accidental mixing of protocols.

---

## 7. Behavior for clients without ShadowPay support

Clients that are not ShadowPay aware simply see `434` as another 4xx error:

- They do not parse or act on `payment_scheme` or `invoice_id`.  
- They do not attempt to generate or attach ShadowPay proofs.  
- They surface the error according to normal application rules.

This is acceptable because HTTP status code space is explicitly extensible and
unknown 4xx codes are treated as client errors.

---

## 8. Summary

For ShadowPay aware clients:

- 434 is the signal that a ShadowPay payment proof is required.  
- The response body tells the client which scheme and context to use.  
- The ShadowPay SDK owns invoices, payments, proofs, and headers.  
- Application code:
  - detects 434  
  - decides whether to pay  
  - calls the SDK  
  - retries with proof  
  - interprets follow up status codes  

This keeps private payment flows explicit and consistent while keeping ShadowPay
specific logic inside SDKs rather than scattered through application code.
