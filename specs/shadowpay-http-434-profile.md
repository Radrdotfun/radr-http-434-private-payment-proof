# ShadowPay profile for HTTP 434 Private Payment Proof Required

This document describes how the ShadowPay protocol uses HTTP status code `434 Private Payment Proof Required` for private payment gating on Solana.

It is a concrete profile of the general 434 specification focused on ShadowPay proofs, headers, JSON shapes, and on chain behavior.

---

## 1. ShadowPay specific meaning

In ShadowPay, `434 Private Payment Proof Required` means:

- The server requires a ShadowPay payment proof that is valid for a specific invoice, resource, or entitlement.  
- The proof MUST be a valid Groth16 zero knowledge proof generated using ShadowPay circuits and verifying keys.  
- The proof MUST bind to a ShadowPay merkle root and a nullifier that is unique within the ShadowPay domain.  
- The proof MUST NOT reveal the payer Solana address or clear text payment amount.

The server will not process the request further until such a proof is presented and verified.

---

## 2. ShadowPay headers

ShadowPay uses the following HTTP headers by convention when attaching proofs to requests.

| Header name                  | Description                                               |
|-----------------------------|-----------------------------------------------------------|
| `X-ShadowPay-Proof`         | Base64 encoded Groth16 proof bytes                       |
| `X-ShadowPay-Nullifier`     | Base58 or hex encoded nullifier                          |
| `X-ShadowPay-Merkle-Root`   | Hex encoded merkle root used during proof generation     |
| `X-ShadowPay-Invoice-Id`    | ShadowPay invoice or payment session identifier          |
| `X-ShadowPay-Escrow-Account`| Base58 encoded Solana escrow PDA or account address      |
| `X-ShadowPay-Scheme`        | ShadowPay scheme or version identifier, for example `shadowpay_v1` |

Implementations MAY move some of this information into the JSON body if that fits their API design better, but these headers are the recommended default.

---

## 3. ShadowPay 434 JSON example

ShadowPay servers SHOULD return structured JSON content with the 434 status code.

Example HTTP response:

    HTTP/1.1 434 Private Payment Proof Required
    Content-Type: application/json

Example JSON body:

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

Field notes:

- `proof_type` indicates the proof system used by ShadowPay, currently `groth16`.  
- `payment_scheme` identifies the ShadowPay scheme and version, such as `shadowpay_v1`.  
- `invoice_id` is a ShadowPay invoice or session identifier that proofs must bind to.  
- `amount` is typically `"encrypted"` to avoid exposing clear text amounts.  
- `escrow_account` is the on chain PDA that holds funds for this payment context.  

Servers MAY omit fields that are not relevant. Servers MAY add ShadowPay specific metadata as needed.

---

## 4. Server verification steps

A ShadowPay aware server that receives a request with ShadowPay proof headers MUST perform at least the following steps.

1. Parse proof fields  
   - Decode `X-ShadowPay-Proof` from base64.  
   - Decode `X-ShadowPay-Nullifier` from base58 or hex.  
   - Decode `X-ShadowPay-Merkle-Root` from hex.  
   - Read `X-ShadowPay-Invoice-Id`, `X-ShadowPay-Escrow-Account`, and `X-ShadowPay-Scheme`.

2. Validate invoice context  
   - Confirm that `X-ShadowPay-Invoice-Id` refers to a known ShadowPay invoice or payment session.  
   - Confirm that the invoice is in a state that allows proof based access, for example not cancelled or fully refunded.

3. Verify merkle root  
   - Check that `X-ShadowPay-Merkle-Root` is in the set of accepted ShadowPay merkle roots for the relevant epoch or configuration.

4. Verify proof  
   - Use the ShadowPay verifying key and merkle root to verify the Groth16 proof.  
   - Reject if proof verification fails.

5. Check nullifier  
   - Look up `X-ShadowPay-Nullifier` in a persistent store.  
   - Reject if it has already been used in a prior payment or settlement.  
   - Atomically record the nullifier as used if verification succeeds.

6. Check on chain state where required  
   - If `X-ShadowPay-Escrow-Account` is present, confirm that the Solana escrow PDA has sufficient funds according to ShadowPay rules.  
   - Verify any additional constraints such as timelocks or subscription windows.

7. Return appropriate status  
   - On success, process the request and return a `2xx` status.  
   - On failure, map to one of the error codes shown below.

Recommended error mappings:

- Use `422 Unprocessable Content` when the proof is malformed or fails cryptographic verification.  
- Use `409 Conflict` when the nullifier has already been used or when the invoice has already been settled.  
- Use `423 Locked` when escrow or account state prevents immediate use of funds.  
- Use `425 Too Early` when a timelock or epoch boundary has not yet been reached.  
- Use `428 Precondition Required` when required pre payment steps such as escrow funding have not been completed.

---

## 5. ShadowPay client behavior

A ShadowPay client or SDK SHOULD implement the following steps when it receives a `434` response from a ShadowPay aware endpoint.

1. Detect `status === 434`.  
2. Parse the JSON body into a `ShadowPayRequirement` structure that contains `invoice_id`, `payment_scheme`, `currency`, and any metadata.  
3. Ensure that the user or application has opted into paying for this resource.  
4. Use the ShadowPay SDK to:

   - Locate or create a ShadowPay invoice with the given `invoice_id`.  
   - Confirm that payment has been made into the correct escrow account if required.  
   - Generate a Groth16 proof that binds the payment to the resource context.  
   - Obtain the merkle root and nullifier that will be used for verification.

5. Retry the original HTTP request and attach the proof using:

   - `X-ShadowPay-Proof`  
   - `X-ShadowPay-Nullifier`  
   - `X-ShadowPay-Merkle-Root`  
   - `X-ShadowPay-Invoice-Id`  
   - `X-ShadowPay-Escrow-Account` if applicable  
   - `X-ShadowPay-Scheme`

6. Handle the follow up response:

   - On `2xx`, return the successful result to the caller.  
   - On `409`, mark the payment as potentially double used and surface an error.  
   - On `422`, treat this as a proof generation or configuration error and notify the developer or user.  
   - On `423`, `425`, or `428`, inspect the body and update client behavior according to the described condition.

The ShadowPay SDK should hide most cryptographic details and on chain operations from application code. Application developers interact mainly with the 434 status code and ShadowPay high level methods.

---

## 6. Compatibility

- Servers that do not know ShadowPay but see 434 from an upstream will forward it as a normal 4xx code.  
- Clients that do not implement ShadowPay will treat 434 as a generic client error.  
- ShadowPay specific behavior is only enabled in clients and servers that follow this profile and the core ShadowPay protocol.
****
