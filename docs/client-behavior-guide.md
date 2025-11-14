# Client behavior for HTTP 434 with ShadowPay

This document describes how HTTP clients and SDKs should behave when they receive `434 Private Payment Proof Required` from a ShadowPay aware server.

The goal is:

- Make behavior predictable for all ShadowPay clients.  
- Keep cryptographic and on chain details inside the ShadowPay SDK.  
- Let application developers work only with HTTP status codes and high level SDK calls.

---

## 1. Detection and parsing

A ShadowPay aware client MUST detect HTTP status code `434` and SHOULD parse the response body.

Typical detection patterns:

- JavaScript: `if (response.status === 434)`  
- Python requests: `if resp.status_code == 434`  
- Rust reqwest: `if resp.status() == 434`  
- Go net/http: `if resp.StatusCode == 434`  

After detection, the client SHOULD parse the response body as JSON and map it into a structure such as:

    interface ShadowPayRequirement {
      status: number;            // should be 434
      title?: string;
      detail?: string;
      proof_type?: string;
      payment_scheme?: string;
      invoice_id?: string;
      currency?: string;
      amount?: string;
      escrow_account?: string;
      metadata?: Record<string, unknown>;
    }

Clients SHOULD tolerate missing optional fields and default `payment_scheme` to `shadowpay_v1` if it is not present.

---

## 2. Decision: whether to pay and prove

Receiving `434` does not mean the client must always pay. The client or SDK MUST decide according to application rules whether it is appropriate to satisfy the requirement.

Inputs into this decision:

- Application configuration or policy.  
- User consent settings.  
- Allow lists or deny lists of hosts, paths, and methods.  
- Spending limits such as maximum invoice amount or per day caps.  

Typical patterns:

- Interactive application: prompt the user before starting the ShadowPay flow.  
- Headless agent: obey a configuration file that specifies which endpoints can trigger ShadowPay payments.  
- Backend service: use static configuration and environment variables to decide which resources are allowed to be paid.

If the client decides not to satisfy the 434 requirement it SHOULD:

- Not retry automatically.  
- Surface an error that includes the status code and the `detail` field from the response body.

---

## 3. Integration with the ShadowPay SDK

A ShadowPay client SHOULD delegate most work to a ShadowPay SDK.

High level algorithm:

1. Detect status `434`.  
2. Parse the JSON response into a `ShadowPayRequirement`.  
3. Pass the requirement and original request context into a helper provided by the SDK.

For example in TypeScript:

    const requirement = await response.json() as ShadowPayRequirement;

    const result = await shadowpay.handleRequirement({
      requirement,
      originalRequest: {
        method,
        url,
        headers,
        body
      }
    });

    if (!result.shouldRetry) {
      throw new Error("ShadowPay requirement not satisfied");
    }

    const retriedResponse = await fetch(url, result.updatedRequestInit);

The SDK is responsible for:

- Locating or creating a ShadowPay invoice that matches `invoice_id` or the metadata.  
- Ensuring that a payment exists for this invoice and scheme.  
- Generating a Groth16 proof, a merkle root, and a nullifier.  
- Constructing updated headers and body fields for the retry.

This keeps application code focused on control flow rather than cryptography.

---

## 4. Attaching ShadowPay proof material

When retrying the original request after satisfying the payment requirement, the client MUST attach ShadowPay proof material using the headers defined in the ShadowPay profile.

Typical header set:

- `X-ShadowPay-Proof`  
- `X-ShadowPay-Nullifier`  
- `X-ShadowPay-Merkle-Root`  
- `X-ShadowPay-Invoice-Id`  
- `X-ShadowPay-Escrow-Account` (if escrow is used)  
- `X-ShadowPay-Scheme`  

Example in JavaScript:

    const proof = await shadowpay.generateProof({
      invoiceId: requirement.invoice_id,
      scheme: requirement.payment_scheme ?? "shadowpay_v1"
    });

    const retriedResponse = await fetch(originalUrl, {
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

The precise API will differ by SDK and language. The pattern is fixed:

434 received -> generate proof with ShadowPay -> retry with proof attached.

---

## 5. Handling follow up responses

After the retry with proof, the client MUST inspect the new response status code and act accordingly.

Suggested mapping:

- `2xx`  
  Treat as success. Return the response body to the caller. The payment and proof are considered accepted.

- `434` again  
  Treat as a configuration or flow error. The server still believes that a proof is missing or unsuitable. The client SHOULD log this and surface an error instead of looping indefinitely.

- `422 Unprocessable Content`  
  The proof was malformed or failed cryptographic verification. The client SHOULD:
  - Log the error code and any diagnostic reason field in the response.  
  - Optionally attempt a single regeneration of the proof if a transient mismatch is suspected.  
  - Fail fast if repeated 422 responses occur.

- `409 Conflict`  
  The nullifier or invoice was already used. This strongly suggests a replay, double spend attempt, or misuse of the same proof. The client MUST NOT retry with the same proof and SHOULD surface a hard error.

- `423 Locked`  
  Funds are locked in escrow or are otherwise temporarily unavailable. The client MAY implement a backoff and retry strategy if the response body communicates a clear unlock condition, otherwise it SHOULD surface an error.

- `425 Too Early`  
  A time based condition is not yet satisfied, for example a timelock or subscription epoch. The client MAY:
  - Use metadata in the body to schedule a retry after a given time.  
  - Or simply return an error to the caller.

- `428 Precondition Required`  
  A pre payment step is missing, such as initial funding of escrow. The client SHOULD treat this as a signal that the ShadowPay configuration is incomplete and surface an error.

The ShadowPay SDK MAY provide high level helpers that map these status codes into typed errors or result variants for easier handling.

---

## 6. Error handling and observability

ShadowPay aware clients SHOULD:

- Log occurrences of 434 and the endpoint URLs involved.  
- Log 409, 422, 423, 425, and 428 responses with enough context to debug but without printing full proofs or secrets.  
- Track metrics such as:
  - Count of 434 responses per endpoint.  
  - Ratio of 434 to successful payment based calls.  
  - Count of 422 proof failures for a given SDK version.

Such metrics help operators detect integration issues, mismatched versions, or misuse of the protocol.

---

## 7. Behavior for clients without ShadowPay support

Clients that do not implement ShadowPay specific logic will see 434 as a generic 4xx client error.

For those clients:

- The correct behavior is to surface an error and not retry.  
- Application developers who want private payment support must adopt a ShadowPay aware SDK or implement the patterns described here.

This is acceptable because HTTP status codes are explicitly extensible in the 4xx range and unknown codes are treated as general client errors.

---

## 8. Summary

For ShadowPay aware clients:

- 434 is the signal that a private payment proof is required.  
- The ShadowPay SDK should own invoice lookup, payment checks, and proof generation.  
- The client logic is:
  - detect 434  
  - decide whether to pay  
  - call ShadowPay SDK  
  - retry with proof  
  - interpret follow up status codes

This keeps private payment behavior consistent across languages and platforms while exposing a simple and standard interface at the HTTP level.
