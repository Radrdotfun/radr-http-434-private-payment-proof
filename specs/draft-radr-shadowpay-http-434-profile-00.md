Internet-Draft: draft-radr-shadowpay-http-434-profile-00  
Intended status: Informational  
Expires: May 2026  

# ShadowPay Profile for HTTP 434 (Private Payment Proof Required)

Author: Radr  

---

## Abstract

This document describes how the ShadowPay protocol profiles the HTTP status code
434 (Private Payment Proof Required) for use with private payments on the Solana
blockchain.

It defines how ShadowPay binds 434 to Groth16 proofs, merkle roots, nullifiers,
and Solana escrow accounts, and how clients and servers are expected to use
headers and JSON fields when implementing ShadowPay aware HTTP APIs.

This profile is non normative with respect to the 434 status code itself. The
normative definition of 434 is provided by the core 434 specification.

---

## 1. Introduction

ShadowPay is a private payment protocol that uses zero knowledge proofs to
authorize access to HTTP resources while hiding payer identity and clear text
amounts.

The HTTP status code 434 (Private Payment Proof Required) provides a generic way
for HTTP servers to signal that a private payment proof is required. This
document specifies how ShadowPay uses 434:

- which headers are used to carry ShadowPay proof material  
- how 434 responses are structured for ShadowPay aware clients  
- how verification results map into other HTTP status codes  

The intent is to allow ShadowPay integrations to behave consistently across
languages and frameworks while remaining compatible with generic 434 behavior.

---

## 2. ShadowPay specific semantics of 434

In ShadowPay, a response with status code 434 (Private Payment Proof Required)
means:

1. The server requires a ShadowPay payment proof that is valid for a specific
   ShadowPay context (for example invoice or subscription).  
2. The proof MUST be generated using ShadowPay circuits and verifying keys and
   MUST conform to ShadowPay rules for nullifiers and merkle roots.  
3. The proof MUST NOT reveal the payer Solana address or clear text payment
   amount.  
4. The server will not process the request further until such a proof is
   presented and verified.

Servers that use ShadowPay MAY also emit 434 in cases where a proof is present
but structurally incomplete, as long as the response representation makes this
clear.

---

## 3. ShadowPay HTTP headers

ShadowPay uses the following HTTP header fields by convention when attaching
proofs to requests.

X-ShadowPay-Proof  
: Base64 encoded Groth16 proof bytes.

X-ShadowPay-Nullifier  
: Encoded nullifier value that is unique within the ShadowPay domain.

X-ShadowPay-Merkle-Root  
: Hex encoded merkle root used during proof generation.

X-ShadowPay-Invoice-Id  
: ShadowPay invoice or payment session identifier.

X-ShadowPay-Escrow-Account  
: Base58 encoded Solana escrow program derived address or account address.

X-ShadowPay-Scheme  
: ShadowPay scheme or version identifier, such as "shadowpay_v1".

Implementations MAY move some of this information into the request body if that
fits their API design better, but the header form above is the recommended
default.

---

## 4. ShadowPay 434 response format

ShadowPay servers SHOULD return structured JSON when sending a 434 status code.

A typical response body is:

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

Field semantics:

status  
: HTTP status code. Always 434 in this context.

title  
: Short human readable label for this status.

detail  
: Human readable description that can be logged or shown in error messages.

proof_type  
: Expected proof system. For ShadowPay this is typically "groth16".

payment_scheme  
: Logical name of the ShadowPay protocol variant and version.

invoice_id  
: ShadowPay invoice or session identifier that proofs must bind to.

currency  
: Logical currency such as "USDC" or "SOL".

amount  
: Typically the literal string "encrypted" or omitted, to avoid exposing clear
text amounts.

escrow_account  
: Solana escrow account or program derived address associated with this payment
context.

metadata  
: Optional non sensitive context that helps the client decide how to respond.

Servers MAY omit fields that are not relevant and MAY add additional
ShadowPay specific fields.

---

## 5. Server side verification steps

A ShadowPay aware server that receives a request with ShadowPay proof headers
SHOULD perform at least the following steps before treating the request as paid.

1. Parse proof fields  
   - Decode X-ShadowPay-Proof from base64.  
   - Decode X-ShadowPay-Nullifier and X-ShadowPay-Merkle-Root from their
     respective encodings.  
   - Read X-ShadowPay-Invoice-Id, X-ShadowPay-Escrow-Account, and
     X-ShadowPay-Scheme.

2. Validate invoice context  
   - Confirm that X-ShadowPay-Invoice-Id refers to a known ShadowPay invoice or
     payment session.  
   - Confirm that the invoice is in a state that allows proof based access.

3. Validate merkle root  
   - Check that X-ShadowPay-Merkle-Root is in the set of accepted ShadowPay
     merkle roots for the relevant epoch or configuration.

4. Verify proof  
   - Use the ShadowPay verifying key and merkle root to verify the Groth16
     proof.  
   - Reject if proof verification fails.

5. Check nullifier  
   - Look up X-ShadowPay-Nullifier in a persistent store.  
   - Reject if it has already been used in a prior payment or settlement.  
   - Atomically record the nullifier as used if verification succeeds.

6. Check on chain state  
   - If X-ShadowPay-Escrow-Account is present, confirm that the Solana escrow
     account has sufficient funds according to ShadowPay rules.  
   - Verify any additional constraints such as timelocks or subscription
     windows.

7. Return appropriate status  
   - On success, process the request and return a 2xx status code.  
   - On failure, map to one of the status codes described below.

ShadowPay profiles the mapping from verification results to HTTP status codes as
follows:

- 422 (Unprocessable Content)  
  for malformed, incomplete, or cryptographically invalid proofs.

- 409 (Conflict)  
  for reused nullifiers or payment references, indicating likely double spend or
  replay.

- 423 (Locked)  
  for cases where funds exist but are locked in escrow or subject to conditions
  that prevent use.

- 425 (Too Early)  
  when a time based condition such as a timelock has not been met.

- 428 (Precondition Required)  
  when required pre payment steps such as escrow funding have not been
  completed.

---

## 6. ShadowPay client behavior

A ShadowPay aware client or SDK SHOULD follow this pattern when it receives a
434 response from a ShadowPay aware endpoint.

1. Detect status code 434.  
2. Parse the response body into a structure that contains at least:
   - proof_type  
   - payment_scheme  
   - invoice_id  
   - currency and metadata  

3. Decide whether the application or user has consented to pay for this
   resource.  
4. If payment is allowed, use the ShadowPay client to:
   - locate or create the ShadowPay invoice with the given invoice_id  
   - ensure that payment has been made into the correct escrow account if
     required  
   - generate a Groth16 proof bound to the invoice context  
   - obtain the merkle root and nullifier used for verification  

5. Retry the original HTTP request and attach proof material using:
   - X-ShadowPay-Proof  
   - X-ShadowPay-Nullifier  
   - X-ShadowPay-Merkle-Root  
   - X-ShadowPay-Invoice-Id  
   - X-ShadowPay-Escrow-Account, if applicable  
   - X-ShadowPay-Scheme  

6. Interpret the follow up response:
   - 2xx: treat as success.  
   - 434 again: treat as a configuration or flow error.  
   - 422: treat as proof or configuration error that needs developer action.  
   - 409: treat as a serious error and do not retry with the same proof.  
   - 423, 425, 428: inspect the body and decide whether to retry later or
     surface an error.

The ShadowPay client libraries are expected to encapsulate most cryptographic
and on chain operations so that application code primarily interacts with HTTP
status codes and high level ShadowPay methods.

---

## 7. Security Considerations

The general security considerations for 434 apply. In addition, ShadowPay
deployments should pay attention to:

- correct and complete verification of Groth16 proofs  
- robust storage and checking of nullifiers  
- correct handling of Solana escrow accounts and timelocks  
- protection of ShadowPay headers and bodies against logging or leakage

ShadowPay specific security guidance is described in the ShadowPay protocol
documentation.

---

## 8. Relationship to the core 434 specification

This document profiles the semantics of 434 for one specific payment protocol.
It does not change the normative definition of the 434 status code, which is
provided by the core 434 specification.

Implementations that follow this profile remain compliant with the generic
semantics of 434, and generic HTTP clients and intermediaries can treat 434 as a
normal 4xx status code.

--- 

## 9. Acknowledgements

This document builds on the core 434 status code definition and on implementation
experience with ShadowPay and other private payment systems.
