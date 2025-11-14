# FAQ for HTTP 434 Private Payment Proof Required

This document answers common questions about the status code `434 Private Payment Proof Required` and how it is used with ShadowPay.

---

## 1. How is 434 different from 402 Payment Required?

`402 Payment Required` says that some kind of payment is needed but does not specify how or in what format.

`434 Private Payment Proof Required` says:

- A payment requirement exists.  
- The server expects a **private payment proof** that satisfies a specific protocol, for example ShadowPay with Groth16 and encrypted amounts.  
- The request was rejected because this proof was not provided.

In short:

- `402` is about the existence of a payment obligation.  
- `434` is about the absence of a required private payment proof.

---

## 2. Can I use 434 without 402?

Yes.

Possible patterns:

- You use **only 434** if invoices and payments are created entirely out of band. For example a merchant dashboard or a separate billing service issues invoices, and the API only cares about proof presentation.  
- You use **402 and 434 together** if your API controls both invoice creation and proof based access.

A common combined pattern:

1. First request from the client hits a protected endpoint.  
2. Server responds with `402` and an invoice object.  
3. Client pays and later submits a proof.  
4. Server responds with `434` if the proof is missing from the follow up request.  
5. Server responds with `2xx` if proof is valid.

---

## 3. What happens if a client does not know about 434?

A client that does not know about 434 will treat it as a generic 4xx client error.

Typical behavior:

- HTTP libraries expose `status = 434` without special handling.  
- Frameworks treat it as a failed request.  
- No automatic payment flow is triggered.

This is acceptable because HTTP status codes in the 4xx range are explicitly extensible. Only ShadowPay aware clients and SDKs will add special behavior for 434.

---

## 4. How does 434 relate to identity and 403 Forbidden?

434 is not an identity or permission error in the usual sense. It is specifically about **payment proofs**.

You should:

- Use `401 Unauthorized` for authentication failures.  
- Use `403 Forbidden` for authorization or permission failures that are not tied to payment.  
- Use `434 Private Payment Proof Required` when the only missing condition is a valid private payment proof.

It is possible for an API to require both identity and payment. In that case:

- Identity checks happen first with `401` or `403`.  
- Payment proof requirements are enforced separately with `434`.

---

## 5. Why use a new code instead of reusing 402?

Reusing 402 to mean both "you need to pay" and "you need to submit a private proof" would blur two separate responsibilities.

Reasons to define 434:

- It creates a **clean separation** between payment creation and proof presentation.  
- It allows gateways and middlewares to have simple logic such as "if status is 434 call ShadowPay".  
- It keeps the meaning of 402 consistent with existing uses and documentation.  
- It makes private payment flows visible and self describing at the HTTP level.

---

## 6. Can 434 be used without ShadowPay?

Yes.

The core specification for 434 is payment system neutral. Any protocol that uses private payment proofs can adopt it, for example:

- A ZK rollup settlement system.  
- A privacy preserving subscription protocol.  
- A mixer based entitlement scheme.

The ShadowPay profile documents one specific usage for Solana and ShadowPay. Other protocols can define their own profiles while reusing 434 as the shared status code.

---

## 7. Is 434 safe for proxies, CDNs, and gateways?

Yes.

HTTP status codes are explicitly extensible. Intermediaries that do not understand 434 will:

- Forward it unchanged to clients.  
- Or treat it as a generic client error in the 4xx range.

There is no requirement that all participants understand every 4xx code. ShadowPay aware components add extra semantics on top, but standard HTTP behavior remains valid.

---

## 8. How should I log and monitor 434?

Recommended practices:

- Log each occurrence of 434 with:
  - The request path and method.  
  - A non sensitive subset of the response body, for example `invoice_id` and `payment_scheme`.  
- Count 434 occurrences per endpoint to detect misconfiguration or unexpected volume.  
- Track ratios:
  - 434 count versus successful paid access.  
  - 422 and 409 counts to detect proof failures and possible double spend attempts.

Avoid logging full proofs, nullifiers, or secrets. Treat ShadowPay headers and body fields as sensitive.

---

## 9. What should a client do if it keeps receiving 434 after attaching a proof?

Repeated 434 responses after attaching proof usually indicate one of:

- The proof is not being attached correctly to the retried request.  
- The server and client disagree on expected headers or JSON shape.  
- The server ignores the proof headers due to missing middleware integration.  
- A proxy or gateway is stripping or altering ShadowPay headers.

Client side actions:

- Log the full set of request headers excluding secrets.  
- Confirm that `X-ShadowPay-*` headers are present in the retried request.  
- Confirm that the target path and method match what the server expects.  
- Surface an error rather than looping.

Server side actions:

- Confirm that ShadowPay middleware or filters run before business logic on the relevant routes.  
- Confirm that any reverse proxy is configured to forward custom headers.

---

## 10. How does 434 interact with rate limits or 429 Too Many Requests?

434 and 429 solve different problems:

- `429 Too Many Requests` indicates that a client exceeded a rate limit.  
- `434 Private Payment Proof Required` indicates that a payment proof is missing.

A system can combine both:

- 429 for global or per user API rate limits.  
- 434 for access to high value endpoints that require private payment.

If rate limiting is implemented using a private payment system with rate limited nullifiers, the system may use both codes in different contexts, but they should not be conflated.

---

## 11. Does using 434 change how HTTP caches behave?

In practice, no.

- 4xx responses are usually not cached by default.  
- 434 should be treated like any other client error status for caching purposes.  
- You can still use `Cache-Control` headers if you want to control intermediate caching behavior explicitly.

Since payment proofs and entitlements are highly dynamic, most ShadowPay integrations will simply not cache 434 responses.

---

## 12. Short summary

- Use `434` when a private payment proof is required and missing.  
- Keep `402` for starting payment sessions.  
- Continue using `401` and `403` for identity and permission issues.  
- Adopt a ShadowPay aware client or SDK to handle 434 automatically.  
- Use standard 4xx codes such as `409` and `422` to classify proof errors.

This keeps private payment flows explicit and understandable for both humans and tooling while staying compatible with existing HTTP infrastructure.

