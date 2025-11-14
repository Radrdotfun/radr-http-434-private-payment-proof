# examples/fastapi-http-434-dependency.py

from fastapi import FastAPI, Header, HTTPException, Depends
from typing import Optional, Dict, Any
from pydantic import BaseModel

app = FastAPI(title="ShadowPay HTTP 434 demo")

# In memory demo state.
used_nullifiers: set[str] = set()
invoices: Dict[str, Dict[str, Any]] = {
    "inv_demo_1": {
        "currency": "USDC",
        "scheme": "shadowpay_v1",
        "active": True,
    }
}


class ShadowPayContext(BaseModel):
    invoice_id: str
    nullifier: str
    merkle_root: str
    escrow_account: Optional[str]
    scheme: str


def looks_like_base64(value: Optional[str]) -> bool:
    if not value or not isinstance(value, str):
        return False
    import base64

    try:
        clean = "".join(value.split())
        decoded = base64.b64decode(clean, validate=True)
        return base64.b64encode(decoded).decode() == clean
    except Exception:
        return False


def looks_like_hex32(value: Optional[str]) -> bool:
    if not value or not isinstance(value, str):
        return False
    import re

    return bool(re.fullmatch(r"[0-9a-fA-F]{64}", value))


async def verify_shadowpay_dependency(
    x_shadowpay_proof: Optional[str] = Header(default=None, alias="X-ShadowPay-Proof"),
    x_shadowpay_nullifier: Optional[str] = Header(
        default=None, alias="X-ShadowPay-Nullifier"
    ),
    x_shadowpay_merkle_root: Optional[str] = Header(
        default=None, alias="X-ShadowPay-Merkle-Root"
    ),
    x_shadowpay_invoice_id: Optional[str] = Header(
        default=None, alias="X-ShadowPay-Invoice-Id"
    ),
    x_shadowpay_escrow_account: Optional[str] = Header(
        default=None, alias="X-ShadowPay-Escrow-Account"
    ),
    x_shadowpay_scheme: Optional[str] = Header(
        default="shadowpay_v1", alias="X-ShadowPay-Scheme"
    ),
) -> ShadowPayContext:
    # If any required header is missing, return 434.
    if not all(
        [
            x_shadowpay_proof,
            x_shadowpay_nullifier,
            x_shadowpay_merkle_root,
            x_shadowpay_invoice_id,
        ]
    ):
        raise HTTPException(
            status_code=434,
            detail={
                "status": 434,
                "title": "Private Payment Proof Required",
                "detail": "This endpoint requires a valid ShadowPay payment proof.",
                "proof_type": "groth16",
                "payment_scheme": "shadowpay_v1",
                "example_invoice_id": "inv_demo_1",
            },
        )

    invoice = invoices.get(x_shadowpay_invoice_id)
    if not invoice or not invoice.get("active"):
        raise HTTPException(
            status_code=428,
            detail={
                "status": 428,
                "title": "ShadowPay Precondition Required",
                "detail": "Unknown or inactive ShadowPay invoice id.",
            },
        )

    if invoice.get("scheme") and invoice["scheme"] != x_shadowpay_scheme:
        raise HTTPException(
            status_code=422,
            detail={
                "status": 422,
                "title": "Invalid ShadowPay Proof",
                "detail": "Scheme does not match invoice.",
            },
        )

    if not looks_like_base64(x_shadowpay_proof):
        raise HTTPException(
            status_code=422,
            detail={
                "status": 422,
                "title": "Invalid ShadowPay Proof",
                "detail": "Proof is not valid base64.",
            },
        )

    if not looks_like_hex32(x_shadowpay_merkle_root):
        raise HTTPException(
            status_code=422,
            detail={
                "status": 422,
                "title": "Invalid ShadowPay Proof",
                "detail": "Merkle root must be 32 byte hex.",
            },
        )

    if not x_shadowpay_nullifier or len(x_shadowpay_nullifier) < 16:
        raise HTTPException(
            status_code=422,
            detail={
                "status": 422,
                "title": "Invalid ShadowPay Proof",
                "detail": "Nullifier looks invalid.",
            },
        )

    if x_shadowpay_nullifier in used_nullifiers:
        raise HTTPException(
            status_code=409,
            detail={
                "status": 409,
                "title": "ShadowPay Nullifier Conflict",
                "detail": "This ShadowPay nullifier has already been used.",
            },
        )

    if x_shadowpay_escrow_account == "LOCKED_ESCROW_FOR_DEMO":
        raise HTTPException(
            status_code=423,
            detail={
                "status": 423,
                "title": "ShadowPay Escrow Locked",
                "detail": "Escrow account is locked for this demo.",
            },
        )

    # Real implementation would verify Groth16 proof here.
    # For this example we treat structurally valid payload as OK.
    used_nullifiers.add(x_shadowpay_nullifier)

    return ShadowPayContext(
        invoice_id=x_shadowpay_invoice_id,
        nullifier=x_shadowpay_nullifier,
        merkle_root=x_shadowpay_merkle_root,
        escrow_account=x_shadowpay_escrow_account,
        scheme=x_shadowpay_scheme or "shadowpay_v1",
    )


@app.get("/v1/public")
async def public_endpoint():
    return {"data": "public ok"}


@app.get("/v1/protected")
async def protected_endpoint(shadowpay: ShadowPayContext = Depends(verify_shadowpay_dependency)):
    return {
        "data": "protected ok",
        "shadowpay": shadowpay.dict(),
    }


@app.get("/v1/demo-invoice")
async def demo_invoice():
    return {
        "invoice_id": "inv_demo_1",
        "currency": "USDC",
        "scheme": "shadowpay_v1",
        "note": "Use this invoice id when testing HTTP 434 with ShadowPay in FastAPI.",
    }
