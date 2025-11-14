# HTTP 434 Private Payment Proof Required

This document describes the HTTP status code `434 Private Payment Proof Required` for general readers and implementers.

It is a simplified companion to the Internet Draft:

`draft-radr-http-434-private-payment-proof-required-00`

---

## 1. Summary

`434 Private Payment Proof Required` indicates that:

- The server understood the request.  
- The target resource is protected by a payment requirement that uses a private payment system (for example ShadowPay).  
- The request did not include a valid private payment proof.  
- The client is expected to obtain or generate such a proof and then retry the request.

This status code is intended for systems that use techniques such as zero knowledge proofs, encrypted amounts, and commitment based accounting to authorize access to HTTP resources.

---

## 2. When to use 434 vs 402

Use these status codes with the following guidance:

- Use `402 Payment Required` when there is no payment session or invoice yet and the client must start a payment flow.  
- Use `434 Private Payment Proof Required` when a payment system already exists and the missing element is a cryptographic payment proof.

A common pattern is:

1. Client calls a protected endpoint.  
2. Server responds with `402` and an invoice description.  
3. Client pays the invoice using a payment system such as ShadowPay.  
4. Client generates a private payment proof bound to that invoice.  
5. Client retries the request, attaching the proof.  
6. If the proof is missing, server responds with `434`.  
7. If the proof is valid, server processes the request and returns `2xx`.

Some deployments will only use `434` if invoice creation and payment happen outside the HTTP interaction.

---

## 3. Example response

### 3.1 HTTP level

    HTTP/1.1 434 Private Payment Proof Required
    Content-Type: application/json

### 3.2 JSON body

    {
      "status": 434,
      "title": "Private Payment Proof Required",
      "detail": "This endpoint requires a valid private payment proof.",
      "proof_type": "groth16",
      "payment_scheme": "shadowpay_v1",
      "invoice_id": "inv_abc123",
      "currency": "USDC",
      "amount": "encrypted",
      "metadata": {
        "resource": "/v1/chat",
        "plan": "pro",
        "interval": "monthly"
      }
    }

Fields:

- `status`  
  The HTTP status code. Always `434` in this context.

- `title`  
  Short human readable label for this status.

- `detail`  
  Human readable description that can be logged or shown in error messages.

- `proof_type`  
  Expected proof system. For example `groth16`.

- `payment_scheme`  
  Logical name of the payment protocol and version. For example `shadowpay_v1`.

- `invoice_id`  
  Identifier of the invoice or payment session that the proof should bind to.

- `currency`  
  Logical currency such as `USDC` or `SOL`.

- `amount`  
  Typically the literal string `"encrypted"` or omitted to avoid leaking clear text amounts.

- `metadata`  
  Optional object with non sensitive context that helps the client decide how to respond.

Servers MAY omit fields that are not relevant. Servers MAY add additional fields that are specific to a given payment system.

---

## 4. Client behavior

A client that understands 434 SHOULD:

1. Detect responses with HTTP status `434`.  
2. Parse the response body into a structured type such as `PrivatePaymentRequirement`.  
3. Decide, according to application policy or user consent, whether it is acceptable to pay for the requested resource.  
4. Use an appropriate payment system (for example ShadowPay) to ensure that a payment exists for the referenced invoice or scheme.  
5. Generate a private payment proof and attach it to a retry of the original request using headers or body fields defined by that payment system.  
6. Inspect the follow up response and handle success or new errors appropriately.

Clients SHOULD avoid automatic payment for arbitrary endpoints without explicit configuration or policy.

---

## 5. Recommended follow up status codes

Implementations that use 434 will usually map proof validation results into more specific status codes.

Common mappings:

- `409 Conflict`  
  Reused nullifier or payment reference, likely double spend or replay.

- `422 Unprocessable Content`  
  Malformed, incomplete, or cryptographically invalid proof.

- `423 Locked`  
  Funds exist but are locked in escrow or under conditions that prevent use.

- `425 Too Early`  
  Timelock or time based condition not yet satisfied.

- `428 Precondition Required`  
  Required pre payment step is missing, such as escrow funding.

These codes are not required for all deployments but provide a useful taxonomy when implementing private payment flows.

---

## 6. Intended audience

This document is intended for:

- HTTP API designers who want to add private payment gating to endpoints.  
- Library and framework authors who want to expose helpers for 434 handling.  
- Payment protocol designers who want a standard HTTP signal for private payment requirements.
