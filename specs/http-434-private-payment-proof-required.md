# HTTP 434 Private Payment Proof Required

This document describes the HTTP status code `434 Private Payment Proof Required` for general readers and implementers.

It is a simplified companion to the Internet Draft `draft-radr-http-434-private-payment-proof-required-00`.

## Summary

`434 Private Payment Proof Required` indicates that:

- The server understood the request.  
- The resource is protected by a payment requirement that uses a private payment system.  
- The request did not include a valid payment proof.  
- The client is expected to submit such a proof and then retry.

Typical usage is in systems that use zero knowledge proofs and encrypted accounting to authorize access to HTTP resources.

## When to use 434 instead of 402

Use:

- `402 Payment Required` when there is no invoice or payment session yet and the client must start a payment flow.  
- `434 Private Payment Proof Required` when a payment system already exists and the missing element is a cryptographic payment proof.

Many applications will:

1. Respond with `402` once, to create or expose an invoice.  
2. Later respond with `434` when the client calls a protected endpoint without attaching the required proof.

Some systems may only use `434` if invoices and payments are managed entirely outside the HTTP interaction.

## Example response

```http
HTTP/1.1 434 Private Payment Proof Required
Content-Type: application/json

{
  "status": 434,
  "title": "Private Payment Proof Required",
  "detail": "This endpoint requires a valid private payment proof.",
  "proof_type": "groth16",
  "payment_scheme": "shadowpay_v1",
  "invoice_id": "inv_abc123",
  "currency": "USDC",
  "amount": "encrypted"
}
