Internet-Draft: draft-radr-http-434-private-payment-proof-required-00  
Intended status: Informational  
Expires: May 2026  

# The 434 (Private Payment Proof Required) HTTP Status Code

Author: Radr / ShadowPay  

---

## Abstract

This document defines the HTTP status code `434 Private Payment Proof Required`.

The 434 status code is used by servers that require a privacy preserving payment
proof before processing a request. It is intended for payment systems that use
zero knowledge proofs, encrypted amounts, and commitment based accounting.

The document describes semantics, typical use cases, and guidance for clients
and servers. It also requests registration of the 434 status code in the HTTP
Status Code registry.

---

## Status of This Memo

This Internet Draft is submitted in full conformance with the provisions of
BCP 78 and BCP 79.

Internet Drafts are working documents of the Internet Engineering Task Force
(IETF). Other groups may also distribute working documents as Internet Drafts.

An Internet Draft is valid for a maximum of six months and may be updated,
replaced, or obsoleted by other documents at any time. It is inappropriate to
use Internet Drafts as reference material or to cite them other than as work in
progress.

This Internet Draft will expire in May 2026.

---

## Copyright Notice

Copyright (c) 2025 IETF Trust and the persons identified as the document
authors. All rights reserved.

---

## 1. Introduction

Modern payment systems often use cryptographic protocols that separate payment
proofs from traditional authentication and authorization.

In privacy preserving designs, the server may require a proof that a payment or
entitlement exists while avoiding exposure of payer identity or unencrypted
amounts.

HTTP status code `402 Payment Required` signals that payment is needed but does
not provide semantics for systems that rely on private payment proofs. There is
no standard HTTP status code that says the request failed specifically because
a private payment proof is missing.

This document defines a new HTTP status code in the 4xx class named
`434 Private Payment Proof Required`. The code is intended for systems that
rely on zero knowledge proofs or similar schemes to enforce payment policies
while preserving privacy.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this
document are to be interpreted as described in RFC 2119 and RFC 8174.

---

## 2. Terminology

This document uses the terminology defined in RFC 9110 for HTTP semantics.

The following additional terms are used:

- Private payment system  
  A payment system that uses techniques such as zero knowledge proofs,
  encrypted amounts, commitment schemes, and nullifiers to hide payer identity
  and raw amounts while still allowing verification.

- Payment proof  
  A cryptographic proof that demonstrates that a valid payment, entitlement, or
  balance exists according to the rules of a private payment system.

- Proof presentation  
  The act of attaching a payment proof to an HTTP request using headers or
  message content so that the server can verify it.

---

## 3. The 434 Private Payment Proof Required Status Code

### 3.1. Semantics

The `434 Private Payment Proof Required` status code indicates that:

1. The server understood the request.  
2. The target resource is protected by a payment requirement that is enforced
   by a private payment system.  
3. The request did not include a valid payment proof in the format expected by
   that system.  
4. The client is expected to obtain or generate such a proof and then retry
   the request.

The status code does not specify a particular payment protocol or proof system.
It only communicates that a private payment proof is required and missing.

Servers generating a `434` response SHOULD provide a representation containing
information that enables the client to understand how to satisfy the payment
requirement. This representation SHOULD be machine readable and SHOULD include
fields such as:

- A description of the payment scheme and version.  
- An identifier of the invoice, entitlement, or payment session.  
- An indication that the payment amount is encrypted or otherwise not exposed.  
- Optional non sensitive metadata that helps the client decide how to proceed.

### 3.2. Use with 402 Payment Required

The 434 status code complements, but does not replace, the existing
`402 Payment Required` status code.

A server MAY use both codes in the following way:

- Use `402 Payment Required` when there is no payment session or invoice yet
  and the client must initiate a payment.  
- Use `434 Private Payment Proof Required` when a payment context already
  exists and the missing element is a cryptographic payment proof.

A typical flow is:

1. Client requests a protected resource without any payment context.  
2. Server responds with `402 Payment Required` and a description of how to
   create or fund an invoice.  
3. Client completes payment using the relevant payment system.  
4. Client generates a payment proof bound to that invoice.  
5. Client retries the original request, attaching the proof.  
6. If the proof is missing or malformed, server responds with `434`.  
7. If the proof is valid, server processes the request and returns a success
   status code (for example `200 OK` or `201 Created`).

Some deployments MAY choose to use only `434` if invoices and payments are
created entirely out of band.

---

## 4. Client and Server Requirements

### 4.1. Client behavior

A client that understands 434 and participates in a private payment system
SHOULD:

1. Detect responses with HTTP status code 434.  
2. Parse the response body into a structured form that includes at least:
   - payment scheme identifier  
   - invoice or session identifier  
   - optional metadata  

3. Decide according to policy or user consent whether it is acceptable to pay
   for the requested resource.  
4. If payment is acceptable, use the relevant payment system to:
   - ensure that a valid payment exists for the given invoice or session  
   - generate a private payment proof bound to that context  

5. Retry the original request, attaching the proof using protocol specific
   headers or message fields.  
6. Interpret any follow up response, including the case where proof
   verification fails.

Clients SHOULD avoid automatically paying for arbitrary resources without
explicit configuration or user consent.

A client that does not understand 434 will treat it as a generic 4xx status
code and is not required to perform any special behavior.

### 4.2. Server behavior

A server that implements 434 MUST:

1. Only emit 434 when the sole remaining reason for request failure is the
   absence or invalidity of a private payment proof.  
2. Clearly indicate in the response body that a private payment proof is
   required and provide enough information to identify the relevant payment
   scheme and context.  
3. Verify any proofs provided in follow up requests using the rules of the
   underlying payment system.  
4. Map proof verification failures into appropriate HTTP status codes,
   which MAY include:
   - `422 Unprocessable Content` for malformed or cryptographically invalid
     proofs.  
   - `409 Conflict` for reused nullifiers or other double spend indicators.  
   - `423 Locked` when funds exist but are temporarily locked.  
   - `425 Too Early` when time based conditions are not yet satisfied.  
   - `428 Precondition Required` when a required payment precondition such as
     escrow funding has not been met.

A server MAY use 434 in combination with other access control mechanisms such
as authentication and authorization. In such cases:

- Authentication failures SHOULD continue to use codes such as `401`.  
- Authorization failures that are not related to payment SHOULD continue to
  use codes such as `403`.  
- 434 SHOULD be reserved for cases where lack of a valid private payment proof
  is the relevant condition.

---

## 5. IANA Considerations

This document requests that IANA register the following HTTP status code in the
"HTTP Status Code Registry":

- Code: 434  
- Short Description: Private Payment Proof Required  
- Reference: this document

---

## 6. Security Considerations

The 434 status code itself does not introduce new security mechanisms. It
standardizes how existing private payment systems interact with HTTP.

However, deployments must consider the following:

- Privacy leakage  
  Servers SHOULD avoid including clear text payer identities or amounts in
  434 responses. Fields that describe the payment requirement SHOULD be
  limited to non sensitive identifiers and metadata.

- Proof reuse and replay  
  Private payment systems that use nullifiers or similar constructs MUST
  ensure that replayed proofs are detectable and rejected. Servers SHOULD map
  such events to a distinct status code such as `409 Conflict`.

- Denial of service  
  Proof verification is often more expensive than normal request processing.
  Servers SHOULD implement rate limiting, resource accounting, or other
  controls to avoid being overloaded by proof verification attempts.

- Mixed content  
  When using 434 in web contexts, implementers SHOULD ensure that payment
  flows and proof submission use secure transport. HTTP over TLS is strongly
  RECOMMENDED.

The use of 434 does not weaken or replace existing security requirements for
authentication, authorization, or transport.

---

## 7. References

### 7.1. Normative References

- RFC 2119  
  Key words for use in RFCs to indicate requirement levels.

- RFC 8174  
  Clarification of the usage of key words for requirement levels.

- RFC 9110  
  HTTP Semantics.

### 7.2. Informative References

- RFC 7231  
  Hypertext Transfer Protocol version 1.1 semantics and content.

- Relevant private payment system specifications as deployed by specific
  implementations.

---

## 8. Acknowledgements

This draft was inspired by work on private payments and zero knowledge proof
based payment rails, including deployment experience with ShadowPay on Solana.

The authors thank reviewers and implementers who provided feedback on early
versions of this document.
