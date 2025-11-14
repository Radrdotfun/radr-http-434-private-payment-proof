# Architecture overview for HTTP 434 in ShadowPay

This document explains how HTTP status code `434 Private Payment Proof Required` fits into the ShadowPay payment architecture and how it works together with `402 Payment Required` and other status codes.

The goal is to give API and backend engineers a mental model of where 434 sits in the full stack:

- HTTP layer  
- ShadowPay API and SDK  
- ZK proof system  
- Solana escrow and settlement

---

## 1. Layers in the ShadowPay architecture

ShadowPay payment flows with HTTP 434 can be viewed as four cooperating layers.

1. HTTP and application layer  
   - Your REST or GraphQL API endpoints  
   - HTTP status codes and JSON responses  
   - Route level configuration for payment gating

2. ShadowPay integration layer  
   - ShadowPay server client (for example `@shadowpay/server`)  
   - ShadowPay schemas for invoices and escrows  
   - Mapping between your business resources and ShadowPay payment entities

3. Proof and verification layer  
   - Groth16 circuits and verifying keys  
   - Merkle tree commitments and nullifiers  
   - Local proof generation in the client or agent  
   - Server side proof verification

4. Solana and on chain state  
   - Escrow PDAs and merchant accounts  
   - Token transfers and deposits  
   - Optional settlement and withdrawal flows

HTTP 434 sits at layer 1 and gives a standard signal from layer 1 down into layers 2 and 3.

---

## 2. High level flow with 402 and 434

A typical ShadowPay integration for a paywalled endpoint looks like this.

1. Client calls an API route that is ShadowPay protected.  
2. Server checks for a valid ShadowPay proof on the request.  
3. If no invoice or payment context exists, server may respond with `402 Payment Required` and include invoice information.  
4. Client uses ShadowPay SDK to create or fund an invoice and possibly deposit funds into an escrow PDA on Solana.  
5. Client uses ShadowPay SDK to generate a private payment proof for that invoice or escrow.  
6. Client retries the original request and attaches the proof using ShadowPay headers or body fields.  
7. If the proof is missing, server responds with `434 Private Payment Proof Required`.  
8. If the proof is present but invalid, server responds with a more specific code such as `422` or `409`.  
9. If the proof is valid, server processes the request and returns `2xx` success.

You can think of 402 as the signal "start a payment session" and 434 as the signal "present the private payment proof".

Some systems will use only 434 if invoices and payments are created entirely out of band.

---

## 3. Where 434 lives in a request lifecycle

For a given HTTP request, the lifecycle in a ShadowPay aware service usually looks like this:

1. Routing  
   - Incoming request is routed to a handler.  
   - Handler or middleware marks the route as ShadowPay protected.

2. Proof inspection  
   - Middleware inspects `X-ShadowPay-*` headers and optionally JSON body.  
   - If no proof and no waiver, middleware returns `434` with a description of what is required.  
   - If proof fields exist, request proceeds to verification.

3. Proof verification  
   - Server side integration calls a ShadowPay verifier module or service.  
   - Verifier decodes proof, nullifier, merkle root, invoice id, and scheme.  
   - Verifier checks merkle root, verifies the Groth16 proof, checks nullifier uniqueness, and validates invoice state.  
   - Verifier returns a structured result such as `OK`, `INVALID`, `DOUBLE_SPEND`, `TIMLOCK`, `ESCROW_LOCKED`.

4. Decision  
   - If result is `OK`, the request is allowed to proceed to business logic.  
   - If result is not `OK`, the server maps the result into an HTTP status code:
     - `INVALID` -> `422 Unprocessable Content`  
     - `DOUBLE_SPEND` -> `409 Conflict`  
     - `TIMLOCK` -> `425 Too Early`  
     - `ESCROW_LOCKED` -> `423 Locked`  
     - Other conditions -> `428 Precondition Required` or application specific errors.

5. Business logic  
   - Handler sees that payment is proven and can safely perform actions such as:
     - Serving model outputs or content  
     - Enqueuing jobs or tasks  
     - Writing records  
     - Emitting webhooks

434 is therefore a gate at step 2 in this lifecycle.

---

## 4. HTTP status code mapping in ShadowPay

The ShadowPay profile uses the following status codes in a payment aware architecture.

- `402 Payment Required`  
  No invoice or payment session yet. Client must create or fund an invoice.

- `434 Private Payment Proof Required`  
  Payment system exists, but a private payment proof is missing from the request.

- `422 Unprocessable Content`  
  Proof is malformed, incomplete, or fails cryptographic verification.

- `409 Conflict`  
  Nullifier or invoice has already been used, indicating replay or double spend.

- `423 Locked`  
  Funds exist but cannot be used yet due to escrow or lock conditions.

- `425 Too Early`  
  Time based condition such as timelock or subscription window has not been reached.

- `428 Precondition Required`  
  Required pre payment step is missing, such as initial escrow funding.

This HTTP level taxonomy mirrors the internal ShadowPay state machine and makes private payment behavior visible to clients in a standard way.

---

## 5. Integration points in a typical backend

To add ShadowPay and 434 into an existing backend, you usually modify three main areas.

1. Configuration  
   - Mark which routes require ShadowPay payment.  
   - Configure required schemes and currencies, for example `shadowpay_v1` and `USDC`.  
   - Optionally configure mappings such as resource to invoice template.

2. Middleware or filters  
   - Add a middleware that checks for proof headers on protected routes.  
   - Return `434` with a JSON body when proof is missing.  
   - Call a verifier module when proof is present.

3. Business logic hooks  
   - Attach the verified invoice id and proof context to the request or context object.  
   - Use that context for logging, metering, and audit.  
   - Avoid mixing payment concerns into core business logic.

This keeps payment concerns contained and allows you to adopt ShadowPay incrementally.

---

## 6. Interaction with Solana and on chain state

HTTP 434 itself is chain agnostic, but in the ShadowPay profile it is tightly aligned with Solana on chain state.

- Escrow PDAs hold user funds for invoices or subscriptions.  
- ShadowPay circuits commit to balances and deposits that live on Solana.  
- Nullifiers are derived from commitments that are linked to on chain state, without exposing address level details.  
- Verifier modules may query Solana RPC or an indexer to confirm that escrow accounts are funded or settled.

The architecture keeps the boundaries clear:

- HTTP 434 indicates missing proof at the API surface.  
- ShadowPay verifier checks cryptographic proofs and nullifiers.  
- Solana state is used to anchor those proofs in an objective ledger.

---

## 7. Summary

In the ShadowPay architecture:

- `402` starts payment sessions when needed.  
- `434` is the precise signal that a private payment proof must be attached.  
- `409`, `422`, `423`, `425`, and `428` classify specific failure modes.  
- ShadowPay SDKs and verifiers connect HTTP semantics to ZK proofs and Solana state.

This structure lets you plug ShadowPay into existing HTTP APIs with minimal changes while gaining a clear and standards friendly story for private payments on Solana.
