# Architecture overview for HTTP 434 Private Payment Proof Required

This document gives a high level view of where HTTP status code `434 Private Payment Proof Required` fits in a system that uses private payment proofs.

The focus is:

- How 434 relates to 402 Payment Required  
- How 434 fits into a typical request flow  
- How to layer private payment logic without polluting business code  
- How protocol specific profiles, such as ShadowPay, plug into the core model

---

## 1. Layers in a private payment aware architecture

A typical architecture that uses HTTP 434 can be viewed in four layers.

1. HTTP and application layer  
   - REST or GraphQL endpoints  
   - HTTP status codes and JSON responses  
   - Route level flags that say whether a resource is payment protected

2. Payment integration layer  
   - Code that knows how to talk to a private payment system  
   - Mapping between application resources and payment contexts  
   - Handling of invoices, subscriptions, and entitlements

3. Proof and verification layer  
   - Logic for generating and verifying private payment proofs  
   - Management of commitment trees, nullifiers, or similar structures  
   - Local verification or calls to an external verifier

4. Ledger or settlement layer  
   - Underlying ledger such as a blockchain or traditional payment rail  
   - Escrow accounts or balances that back the proofs  
   - Optional settlement and withdrawal flows

HTTP 434 lives at layer 1. It is the signal from the HTTP layer that a private payment proof is required. The actual proof system and ledger details belong to layers 3 and 4 and are defined by protocol specific profiles.

---

## 2. Relationship between 402 and 434

In a private payment system you can separate two phases:

1. Creating and funding a payment context  
2. Presenting a private payment proof to access a resource

The two HTTP status codes map naturally to these phases:

- `402 Payment Required`  
  Means there is no suitable payment context yet. The client must create or fund one.

- `434 Private Payment Proof Required`  
  Means a suitable payment context exists but a private payment proof is missing or unusable for this request.

A typical combined flow:

1. Client calls a payment protected endpoint with no prior setup.  
2. Server responds with `402` and describes how to create or fund a payment context.  
3. Client uses a payment protocol to create and fund that context.  
4. Client generates a private payment proof tied to that context.  
5. Client retries the original request and attaches the proof.  
6. Server either accepts the proof and returns `2xx` or rejects it with a more specific status such as `422` or `409`.

Some deployments skip 402 entirely and use 434 alone when payment contexts are established out of band.

---

## 3. Request lifecycle with HTTP 434

For a single HTTP request to a payment protected resource, the lifecycle usually looks like this:

1. **Routing**  
   - Request is matched to a handler.  
   - Route metadata marks it as payment protected.

2. **Proof inspection**  
   - Middleware or filters inspect headers and optionally the body for proof material.  
   - If proof is missing, server returns `434 Private Payment Proof Required` with a response body describing what is needed.  
   - If proof is present, the request moves to verification.

3. **Proof verification**  
   - Server validates syntax and encoding of proof fields.  
   - Server calls verification logic or an external verifier for cryptographic checks.  
   - Server checks replay protection (for example nullifier reuse).  
   - Server validates any additional conditions such as expiry or usage limits.

4. **Decision and mapping**  
   - If verification passes, the request is allowed to reach business logic.  
   - If verification fails, the server maps the error into an HTTP status code such as:
     - `422 Unprocessable Content` for invalid or malformed proofs  
     - `409 Conflict` for double spend conditions  
     - `423 Locked` for locked funds  
     - `425 Too Early` for timelock conditions  
     - `428 Precondition Required` for missing funding or similar

5. **Business logic**  
   - Handler executes application specific code with the knowledge that payment has been privately proven.  
   - Handler may also log which payment context was used.

HTTP 434 is the explicit gate at step 2 when a proof is required but not yet provided.

---

## 4. Status code taxonomy in a private payment system

To keep behavior predictable, it is useful to reserve certain status codes for specific classes of payment related errors:

- `402 Payment Required`  
  No payment context yet. Client must initiate payment.

- `434 Private Payment Proof Required`  
  Payment context exists or is assumed to exist. Client must present a private payment proof.

- `422 Unprocessable Content`  
  Proof was present but could not be processed or verified. For example wrong format or cryptographic failure.

- `409 Conflict`  
  Proof indicates a state conflict, such as reuse of a nullifier or already consumed entitlement.

- `423 Locked`  
  Funds or entitlements exist but cannot be used yet due to a lock or hold.

- `425 Too Early`  
  Time based condition not yet satisfied. For example a timelock or subscription window.

- `428 Precondition Required`  
  A prerequisite step such as funding an escrow account has not been completed.

This mapping keeps payment specific concerns visible at the HTTP level while allowing the underlying payment protocol to evolve.

---

## 5. Integration points in typical backends

To integrate HTTP 434 into an existing backend, most systems need to change three areas.

1. **Configuration**  
   - Define which routes are payment protected.  
   - Attach configuration such as required payment scheme, currency, pricing model, or context mapping.

2. **Middleware or filters**  
   - Implement a reusable component that:
     - Checks for proof fields.  
     - Returns `434` when proof is missing.  
     - Invokes verification logic when proof is present.  
     - Maps verification results to HTTP status codes.

3. **Handler context**  
   - Attach verification results such as the payment context id to the request context.  
   - Let business logic trust that payment has been proven without handling cryptography or ledger interaction directly.

This separation allows teams to maintain payment logic independently from application features.

---

## 6. Example profile: ShadowPay on Solana

The core 434 status code is protocol neutral. Concrete payment systems define their own profiles that specify how proofs and contexts are carried over HTTP.

ShadowPay is one such profile for private payments on Solana. In the ShadowPay profile:

- Proofs are carried using headers such as:
  - `X-ShadowPay-Proof`  
  - `X-ShadowPay-Nullifier`  
  - `X-ShadowPay-Merkle-Root`  
  - `X-ShadowPay-Invoice-Id`  
  - `X-ShadowPay-Escrow-Account`  
  - `X-ShadowPay-Scheme`

- A `434` response includes a JSON payload with fields like:
  - `proof_type`, `payment_scheme`, `invoice_id`, `currency`, `amount`, and optional metadata.

- Verification logic ties proofs to Solana escrow accounts and uses nullifiers to prevent replay.

For details of the ShadowPay profile, see:

- `specs/shadowpay-http-434-profile.md`  
- The example implementations in the `examples/` directory

These documents show how to apply the generic architecture described above to a concrete zero knowledge payment system while remaining compatible with the core semantics of HTTP 434.
