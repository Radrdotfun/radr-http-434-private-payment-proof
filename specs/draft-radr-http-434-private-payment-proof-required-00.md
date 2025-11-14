Internet Draft                                             Radr / ShadowPay
Intended status: Informational                                  November 2025
Expires: May 2026


           The 434 (Private Payment Proof Required) HTTP Status Code
          draft-radr-http-434-private-payment-proof-required-00


**Abstract**

   This document defines the HTTP status code 434 (Private Payment Proof
   Required).  The 434 status code is used by servers that require a
   privacy preserving payment proof before processing a request.

   The status code is suitable for payment systems that use zero
   knowledge proofs, encrypted amounts, and commitment based accounting.
   The document describes semantics, typical use cases, and guidance for
   clients and servers.  It also requests registration of the 434 status
   code in the HTTP Status Code registry.


**Status of This Memo
**
   This Internet Draft is submitted in full conformance with the
   provisions of BCP 78 and BCP 79.

   Internet Drafts are working documents of the Internet Engineering
   Task Force (IETF).  Other groups may also distribute working
   documents as Internet Drafts.

   An Internet Draft is valid for a maximum of six months and may be
   updated, replaced, or obsoleted by other documents at any time.  It
   is inappropriate to use Internet Drafts as reference material or to
   cite them other than as work in progress.

   This Internet Draft will expire in May 2026.


**Copyright Notice
**
   Copyright (c) 2025 IETF Trust and the persons identified as the
   document authors.  All rights reserved.


1.  **Introduction**

   Modern payment systems often use cryptographic protocols that
   separate payment proofs from traditional authentication and
   authorization.  In privacy preserving designs the server may require
   a proof that a payment or entitlement exists while avoiding exposure
   of payer identity or unencrypted amounts.

   HTTP status code 402 (Payment Required) indicates that a payment is
   required but provides no semantics for systems that use private
   payment proofs.  Application designers need a way to signal that a
   request is blocked specifically because a private payment proof is
   missing.

   This document defines a new HTTP status code in the 4xx class named
   "434 Private Payment Proof Required".  The code is intended for
   systems that rely on zero knowledge proofs or similar schemes to
   enforce payment policies.

   The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT",
   "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and
   "OPTIONAL" in this document are to be interpreted as described in
   RFC 2119 and RFC 8174.


2.  **Terminology**

   This document uses the terminology defined in RFC 9110 for HTTP
   semantics.

   The following additional terms are used:

   Private payment system
      A payment system that uses techniques such as zero knowledge
      proofs, encrypted amounts, commitment schemes, and nullifiers to
      hide payer identity and raw amounts while still allowing
      verification.

   Payment proof
      A cryptographic proof that demonstrates that a valid payment,
      entitlement, or balance exists according to the rules of a private
      payment system.

   Client
      An HTTP user agent that understands this status code and knows how
      to construct or attach a private payment proof.

   Server
      An HTTP origin server or intermediary that enforces a private
      payment requirement.


3.  **The 434 Private Payment Proof Required Status Code**

3.1.  Semantics

   The 434 (Private Payment Proof Required) status code indicates that:

   *  The server understood the request and the request is syntactically
      correct.

   *  The requested resource is protected by a payment requirement that
      is enforced using a private payment system.

   *  The server did not receive a valid private payment proof attached
      to the request.

   *  The client is expected to supply such a proof before the request
      can be processed successfully.

   The server is unwilling to process the request further until a
   suitable private payment proof is presented.  The server may include
   information in the response content that describes the required proof
   and how the client should obtain or construct it.


3.2.  Use Cases

   The 434 status code is intended for use in systems where payment is
   enforced using cryptographic proofs rather than traditional account
   checks.  Examples include:

   *  An API endpoint that charges per request and that verifies a zero
      knowledge proof over an escrow balance.

   *  A subscription service that requires a proof that the current
      billing period has been paid without revealing subscriber
      identity.

   *  A content delivery endpoint that is unlocked only when a payment
      proof shows that a related invoice has been paid.

   *  A withdrawal endpoint that requires a proof of entitlement before
      funds are released from escrow.


4.  **Response Content**

   A server that sends a 434 response SHOULD provide a representation
   that explains what is required in order for the request to succeed.

   JSON is a common representation format.  The following is an example
   response body:

   {
     "status": 434,
     "title": "Private Payment Proof Required",
     "detail": "This endpoint requires a valid private payment proof.",
     "proof_type": "groth16",
     "payment_scheme": "shadowpay_v1",
     "invoice_id": "inv_abc123",
     "currency": "USDC",
     "amount": "encrypted",
     "metadata": {
       "resource": "/v1/chat",
       "plan": "pro",
       "interval": "monthly"
     }
   }

   Fields such as "proof_type", "payment_scheme" and "invoice_id" are
   application specific hints.  Servers SHOULD avoid including clear
   text payer identity or exact amounts in responses that are likely to
   be logged or exposed.


5.  **Relationship to Other Status Codes**

   The 434 code is intended to complement status code 402 (Payment
   Required) rather than replace it.

   A server MAY respond with 402 when no payment session or invoice is
   associated with the request and the client needs to initiate a
   payment process.

   A server MAY respond with 434 when a payment system is already in
   place and the missing element is a private payment proof.

   Systems that use 434 will commonly also use:

   *  409 (Conflict) when a nullifier or unique payment reference has
      already been used.

   *  422 (Unprocessable Content) when an attached proof is malformed or
      fails cryptographic verification.

   *  423 (Locked) when funds exist but are locked under conditions that
      prevent use.

   *  425 (Too Early) when a time based condition such as a timelock is
      not yet satisfied.

   *  428 (Precondition Required) when a required pre payment step is
      missing.

   Clients that do not understand 434 will treat it as a generic client
   error in the 4xx class, which is acceptable under HTTP semantics.


6.  **Client Behavior**

   A client that understands 434 and that supports the relevant private
   payment system SHOULD, upon receiving a 434 response:

   1.  Inspect the response content for information about the required
       proof, such as invoice identifiers, payment schemes, or proof
       types.

   2.  Ensure that an appropriate payment has been made or that
       sufficient funds exist in the corresponding system.

   3.  Construct a private payment proof that satisfies the server
       policy.

   4.  Repeat the original request and attach the proof in a manner
       defined by the payment system and by the application protocol.

   Clients SHOULD avoid blindly retrying requests that result in 434
   without user input or suitable application logic.


7.  **Security Considerations**

   Applications that use 434 rely on the security of the underlying
   private payment system.  Implementers MUST ensure that proof
   verification uses correct parameters and verifying keys and that
   uniqueness conditions such as nullifiers are enforced.

   Servers SHOULD avoid including sensitive payment data in 434
   responses or logs.  In particular, servers SHOULD NOT log full proofs
   or cryptographic secrets.

   Clients MUST NOT send private keys or other sensitive secrets when
   responding to a 434 status.


8.  **Privacy Considerations**

   The purpose of 434 is to support systems that can enforce payment
   without exposing payer identity or amounts.  This can improve privacy
   relative to traditional account based models.

   However, response content may still contain identifiers that can be
   used for correlation.  Implementers SHOULD consider minimising the
   information included in 434 responses and SHOULD treat logs involving
   payment proofs as sensitive.


9.  **IANA Considerations**

   IANA is requested to register the following in the "HTTP Status
   Codes" registry.

   Value: 434

   Description: Private Payment Proof Required

   Reference: This document


10. ** References**

10.1.  Normative References

   [RFC2119]  Bradner, S., "Key words for use in RFCs to Indicate
              Requirement Levels", BCP 14, RFC 2119, March 1997.

   [RFC8174]  Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC
              2119 Key Words", BCP 14, RFC 8174, May 2017.

   [RFC9110]  Fielding, R., Nottingham, M., and J. Reschke, "HTTP
              Semantics", RFC 9110, June 2022.


Authors' Addresses

   ShadowPay Engineering
   Radr
   Email: Hello@radr.fun
