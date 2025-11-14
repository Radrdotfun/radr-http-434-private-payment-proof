**HTTP 434 Private Payment Proof Required

Specification and reference examples for the proposed HTTP status code:

434 Private Payment Proof Required

Goal:

Let HTTP APIs say: this request must include a private payment proof.

This repo contains:

Internet Draft style spec for 434

A short spec for implementers

A ShadowPay specific profile (Solana, ZK, escrow)

Minimal Express, FastAPI, and Axum examples

Status

Work in progress proposal, not an IETF standard.

The main spec is written in Internet Draft format so it can be turned into a draft and submitted if needed.

Layout
Specs

specs/draft-radr-http-434-private-payment-proof-required-00.md
Internet Draft style document that defines 434 and requests IANA registration.

specs/http-434-private-payment-proof-required.md
Short spec for general readers and API designers.

specs/shadowpay-http-434-profile.md
How ShadowPay uses 434 on Solana, including headers, JSON shapes, and how it maps to proofs and escrow.

Docs

docs/architecture-overview.md
Where 434 sits relative to 402, ZK proofs, and Solana escrow.

docs/client-behavior-guide.md
How clients and SDKs should react when they see 434 and follow up codes like 422 and 409.

docs/server-implementation-guide.md
How to add 434 and proof checks to your backend using middleware or filters.

docs/faq.md
Answers for “why not just use 402”, “what about 403”, and similar questions.

Examples

examples/node-express-http-434-middleware.js
Express middleware that:

returns 434 when proof headers are missing

does basic structural checks

maps errors into 422, 409, 423, 428

examples/fastapi-http-434-dependency.py
FastAPI dependency that:

enforces 434 at the dependency level

validates headers

raises HTTPException with 422, 409, 423, 428 as needed

examples/axum-http-434-middleware.rs
Axum plus tower layer that:

protects /v1/protected

keeps a small in memory nullifier set

returns 434 and other 4xx codes based on verification

Quick start
1. Read the short spec

Start with:

specs/http-434-private-payment-proof-required.md

specs/shadowpay-http-434-profile.md if you care about ShadowPay.

These two files explain when to send 434, what to return in the body, and which other status codes to use.

2. Run an example

Node and Express:

cd examples
node node-express-http-434-middleware.js
# GET http://localhost:3000/v1/public    -> 200
# GET http://localhost:3000/v1/protected -> 434 (no ShadowPay headers)


FastAPI:

cd examples
uvicorn fastapi-http-434-dependency:app --reload
# GET /v1/public    -> 200
# GET /v1/protected -> 434 (no ShadowPay headers)


Axum:

cd examples
cargo run --bin axum-http-434-middleware
# GET /v1/public    -> 200
# GET /v1/protected -> 434 (no ShadowPay headers)


Each example also exposes /v1/demo-invoice which returns an invoice id used in demo verification.

3. Integrate in your backend

At a minimum your server should:

Mark some routes as payment protected.

Check for required proof fields (for example X-ShadowPay-*).

Return 434 with a JSON body when proof is missing.

Verify proofs, merkle roots, and nullifiers using you**# HTTP 434 Private Payment Proof Required

Specification and reference examples for the proposed HTTP status code:

`434 Private Payment Proof Required`

Goal:

> Let HTTP APIs say: this request must include a private payment proof.

This repo contains:

- Internet Draft style spec for 434  
- A short spec for implementers  
- A ShadowPay specific profile (Solana, ZK, escrow)  
- Minimal Express, FastAPI, and Axum examples

---

## Status

Work in progress proposal, not an IETF standard.

The main spec is written in Internet Draft format so it can be turned into a draft and submitted if needed.

---

## Layout

### Specs

- `specs/draft-radr-http-434-private-payment-proof-required-00.md`  
  Internet Draft style document that defines 434 and requests IANA registration.

- `specs/http-434-private-payment-proof-required.md`  
  Short spec for general readers and API designers.

- `specs/shadowpay-http-434-profile.md`  
  How ShadowPay uses 434 on Solana, including headers, JSON shapes, and how it maps to proofs and escrow.

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
  Express middleware that:
  - returns 434 when proof headers are missing  
  - does basic structural checks  
  - maps errors into 422, 409, 423, 428  

- `examples/fastapi-http-434-dependency.py`  
  FastAPI dependency that:
  - enforces 434 at the dependency level  
  - validates headers  
  - raises HTTPException with 422, 409, 423, 428 as needed  

- `examples/axum-http-434-middleware.rs`  
  Axum plus tower layer that:
  - protects `/v1/protected`  
  - keeps a small in memory nullifier set  
  - returns 434 and other 4xx codes based on verification

---

## Quick start

### 1. Read the short spec

Start with:

- `specs/http-434-private-payment-proof-required.md`  
- `specs/shadowpay-http-434-profile.md` if you care about ShadowPay.

These two files explain when to send 434, what to return in the body, and which other status codes to use.

### 2. Run an example

Node and Express:

    cd examples
    node node-express-http-434-middleware.js
    # GET http://localhost:3000/v1/public    -> 200
    # GET http://localhost:3000/v1/protected -> 434 (no ShadowPay headers)

FastAPI:

    cd examples
    uvicorn fastapi-http-434-dependency:app --reload
    # GET /v1/public    -> 200
    # GET /v1/protected -> 434 (no ShadowPay headers)

Axum:

    cd examples
    cargo run --bin axum-http-434-middleware
    # GET /v1/public    -> 200
    # GET /v1/protected -> 434 (no ShadowPay headers)

Each example also exposes `/v1/demo-invoice` which returns an invoice id used in demo verification.

### 3. Integrate in your backend

At a minimum your server should:

1. Mark some routes as payment protected.  
2. Check for required proof fields (for example `X-ShadowPay-*`).  
3. Return `434` with a JSON body when proof is missing.  
4. Verify proofs, merkle roots, and nullifiers using your payment system.  
5. Map failures into status codes such as `422`, `409`, `423`, `425`, `428`.  
6. Only run business logic when verification passes.

See `docs/server-implementation-guide.md` and `docs/client-behavior-guide.md` for detailed flow.

---

## Relationship to ShadowPay

The 434 spec is payment system agnostic.

The ShadowPay profile is the first concrete implementation:

- Uses Groth16 proofs and merkle roots  
- Uses nullifiers to prevent double spend  
- Anchors proofs to Solana escrow PDAs and invoices  
- Uses HTTP 434 as the signal that a private payment proof must be attached

If you only care about ShadowPay:

- `specs/shadowpay-http-434-profile.md`  
- `docs/architecture-overview.md`  
- the `examples/` directory

are the main files to read.

---

## Contributing

Contributions are welcome, in particular:

- Spec clarifications  
- More examples in other languages or frameworks  
- Feedback on status code mapping and error taxonomy

Open an issue or pull request. Keep changes tight and technical.

---

## License

MIT, see `LICENSE`.
