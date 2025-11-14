# HTTP 434 Private Payment Proof Required

Specification and reference examples for the proposed HTTP status code:

`434 Private Payment Proof Required`

Goal:

> Let HTTP APIs clearly say: you must attach a private payment proof for this request to succeed.

This repo contains:

- The Internet Draft style text for 434  
- A plain spec for implementers  
- A ShadowPay specific profile for Solana payments  
- Minimal Express, FastAPI, and Axum examples

---

## Status

Work in progress proposal, not an IETF standard.

The main spec is written in Internet Draft style so it can be turned into a proper draft. Names and structure may change while iterating.

---

## Layout

### Specs

- `specs/draft-radr-http-434-private-payment-proof-required-00.md`  
  Internet Draft style document that defines 434 and requests IANA registration.

- `specs/http-434-private-payment-proof-required.md`  
  Short spec for general readers and API designers.

- `specs/shadowpay-http-434-profile.md`  
  How ShadowPay uses 434 on Solana, including headers, JSON shapes, and mapping to ZK proofs and escrow.

### Docs

- `docs/architecture-overview.md`  
  Where 434 sits relative to 402, ZK proofs, and Solana escrow.

- `docs/client-behavior-guide.md`  
  How clients and SDKs should react when they see 434 and follow up codes like 422 and 409.

- `docs/server-implementation-guide.md`  
  How to add 434 and proof checks to your backend using middleware or filters.

- `docs/faq.md`  
  Answers for “why not just use 402”, “what about 403”, and similar questions.

### Examples

- `examples/node-express-http-434-middleware.js`  
  Express middleware that returns 434 when proof headers are missing, runs basic checks, and maps errors into 422, 409, 423, 428.

- `examples/fastapi-http-434-dependency.py`  
  FastAPI dependency that enforces 434 and surfaces verification errors as HTTPException values.

- `examples/axum-http-434-middleware.rs`  
  Axum and tower layer that protects `/v1/protected` and keeps a small in memory nullifier set.

---

## Quick start

### 1. Read the short spec

Start with:

- `specs/http-434-private-payment-proof-required.md`  
- then `specs/shadowpay-http-434-profile.md` if you care about ShadowPay.

These two files are enough to understand how to use 434 and what to put in responses.

### 2. Run an example server

Node and Express:

```bash
cd examples
node node-express-http-434-middleware.js
# GET /v1/public    -> 200
# GET /v1/protected -> 434 without ShadowPay headers

