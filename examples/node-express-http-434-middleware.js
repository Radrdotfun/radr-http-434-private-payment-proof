// examples/node-express-http-434-middleware.js

const express = require("express");
const app = express();

app.use(express.json());

// Super simple demo state.
// In production this would come from your DB and on chain checks.
const usedNullifiers = new Set();
const invoices = new Map([
  [
    "inv_demo_1",
    {
      currency: "USDC",
      scheme: "shadowpay_v1",
      active: true
    }
  ]
]);

function hasShadowPayHeaders(req) {
  const h = req.headers;
  return (
    typeof h["x-shadowpay-proof"] === "string" &&
    typeof h["x-shadowpay-nullifier"] === "string" &&
    typeof h["x-shadowpay-merkle-root"] === "string" &&
    typeof h["x-shadowpay-invoice-id"] === "string"
  );
}

function looksLikeBase64(str) {
  if (!str || typeof str !== "string") return false;
  try {
    const clean = str.replace(/\s+/g, "");
    return Buffer.from(clean, "base64").toString("base64") === clean;
  } catch {
    return false;
  }
}

function looksLikeHex32(str) {
  return typeof str === "string" && /^[0-9a-f]{64}$/i.test(str);
}

// Replace the middle section with real ShadowPay ZK verification.
async function verifyShadowPayProof(req) {
  const proof = req.header("X-ShadowPay-Proof");
  const nullifier = req.header("X-ShadowPay-Nullifier");
  const merkleRoot = req.header("X-ShadowPay-Merkle-Root");
  const invoiceId = req.header("X-ShadowPay-Invoice-Id");
  const escrowAccount = req.header("X-ShadowPay-Escrow-Account") || null;
  const scheme = req.header("X-ShadowPay-Scheme") || "shadowpay_v1";

  const invoice = invoices.get(invoiceId);
  if (!invoice || !invoice.active) {
    return { ok: false, code: 428, msg: "Unknown or inactive invoice" };
  }

  if (invoice.scheme && invoice.scheme !== scheme) {
    return { ok: false, code: 422, msg: "Scheme does not match invoice" };
  }

  if (!looksLikeBase64(proof)) {
    return { ok: false, code: 422, msg: "Proof is not valid base64" };
  }

  if (!looksLikeHex32(merkleRoot)) {
    return { ok: false, code: 422, msg: "Merkle root must be 32 byte hex" };
  }

  if (!nullifier || nullifier.length < 16) {
    return { ok: false, code: 422, msg: "Nullifier looks invalid" };
  }

  if (usedNullifiers.has(nullifier)) {
    return { ok: false, code: 409, msg: "Nullifier already used" };
  }

  if (escrowAccount === "LOCKED_ESCROW_FOR_DEMO") {
    return { ok: false, code: 423, msg: "Escrow is locked" };
  }

  // Real implementation: verify Groth16 proof against ShadowPay verifier here.
  // For the example we treat a structurally valid payload as OK.
  usedNullifiers.add(nullifier);

  return {
    ok: true,
    ctx: {
      invoiceId,
      nullifier,
      merkleRoot,
      escrowAccount,
      scheme
    }
  };
}

function shadowpayGuard() {
  return async function (req, res, next) {
    if (!hasShadowPayHeaders(req)) {
      return res.status(434).json({
        status: 434,
        title: "Private Payment Proof Required",
        detail: "ShadowPay proof headers are required on this endpoint.",
        proof_type: "groth16",
        payment_scheme: "shadowpay_v1",
        example_invoice_id: "inv_demo_1"
      });
    }

    let result;
    try {
      result = await verifyShadowPayProof(req);
    } catch (err) {
      console.error("[shadowpay] verifier crashed:", err);
      return res.status(422).json({
        status: 422,
        title: "ShadowPay verification error",
        detail: "Could not verify ShadowPay proof"
      });
    }

    if (!result.ok) {
      const code = result.code || 422;
      const msg = result.msg || "ShadowPay proof rejected";

      if (code === 409) {
        return res.status(409).json({
          status: 409,
          title: "ShadowPay nullifier conflict",
          detail: msg
        });
      }

      if (code === 423) {
        return res.status(423).json({
          status: 423,
          title: "ShadowPay escrow locked",
          detail: msg
        });
      }

      if (code === 428) {
        return res.status(428).json({
          status: 428,
          title: "ShadowPay precondition required",
          detail: msg
        });
      }

      return res.status(422).json({
        status: 422,
        title: "Invalid ShadowPay proof",
        detail: msg
      });
    }

    req.shadowpay = result.ctx;
    return next();
  };
}

// Demo routes

app.get("/v1/public", (req, res) => {
  res.json({ data: "public ok" });
});

app.get("/v1/protected", shadowpayGuard(), (req, res) => {
  res.json({
    data: "protected ok",
    shadowpay: req.shadowpay
  });
});

app.get("/v1/demo-invoice", (req, res) => {
  res.json({
    invoice_id: "inv_demo_1",
    currency: "USDC",
    scheme: "shadowpay_v1",
    note: "Use this invoice id when testing HTTP 434 with ShadowPay"
  });
});

if (require.main === module) {
  const port = Number(process.env.PORT) || 3000;
  app.listen(port, () => {
    console.log(`ShadowPay 434 demo on http://localhost:${port}`);
  });
}

module.exports = {
  app,
  shadowpayGuard,
  verifyShadowPayProof
};
