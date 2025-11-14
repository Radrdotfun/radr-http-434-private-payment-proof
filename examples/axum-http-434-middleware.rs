// examples/axum-http-434-middleware.rs

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::future::BoxFuture;
use serde::Serialize;
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower::{Layer, Service};

#[derive(Clone, Default)]
struct ShadowPayState {
    used_nullifiers: Arc<Mutex<HashSet<String>>>,
}

#[derive(Clone)]
struct ShadowPayLayer {
    state: ShadowPayState,
}

impl ShadowPayLayer {
    fn new() -> Self {
        Self {
            state: ShadowPayState::default(),
        }
    }
}

impl<S> Layer<S> for ShadowPayLayer {
    type Service = ShadowPayMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ShadowPayMiddleware {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
struct ShadowPayMiddleware<S> {
    inner: S,
    state: ShadowPayState,
}

#[derive(Debug)]
enum VerifyError {
    MissingHeaders,
    InvalidProof(String),
    DoubleSpend(String),
    EscrowLocked,
    PreconditionMissing(String),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    status: u16,
    title: &'a str,
    detail: String,
}

fn has_shadowpay_headers(headers: &HeaderMap) -> bool {
    headers.contains_key("X-ShadowPay-Proof")
        && headers.contains_key("X-ShadowPay-Nullifier")
        && headers.contains_key("X-ShadowPay-Merkle-Root")
        && headers.contains_key("X-ShadowPay-Invoice-Id")
}

fn looks_like_base64(s: &str) -> bool {
    let clean = s.trim();
    if clean.is_empty() {
        return false;
    }
    base64::decode(clean).is_ok()
}

fn looks_like_hex32(s: &str) -> bool {
    let clean = s.trim();
    clean.len() == 64 && clean.chars().all(|c| c.is_ascii_hexdigit())
}

// Demo verifier that does real structural checks and a nullifier set.
fn verify_shadowpay(headers: &HeaderMap, state: &ShadowPayState) -> Result<(), VerifyError> {
    let proof = headers
        .get("X-ShadowPay-Proof")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let nullifier = headers
        .get("X-ShadowPay-Nullifier")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let merkle_root = headers
        .get("X-ShadowPay-Merkle-Root")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let invoice_id = headers
        .get("X-ShadowPay-Invoice-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let escrow_account = headers
        .get("X-ShadowPay-Escrow-Account")
        .and_then(|v| v.to_str().ok());
    let _scheme = headers
        .get("X-ShadowPay-Scheme")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("shadowpay_v1");

    if proof.is_empty() || nullifier.is_empty() || merkle_root.is_empty() || invoice_id.is_empty() {
        return Err(VerifyError::MissingHeaders);
    }

    if invoice_id != "inv_demo_1" {
        return Err(VerifyError::PreconditionMissing(
            "Unknown or inactive invoice id".into(),
        ));
    }

    if !looks_like_base64(proof) {
        return Err(VerifyError::InvalidProof("Proof is not valid base64".into()));
    }

    if !looks_like_hex32(merkle_root) {
        return Err(VerifyError::InvalidProof(
            "Merkle root must be 32 byte hex".into(),
        ));
    }

    if nullifier.len() < 16 {
        return Err(VerifyError::InvalidProof(
            "Nullifier looks too short".into(),
        ));
    }

    if let Some(acc) = escrow_account {
        if acc == "LOCKED_ESCROW_FOR_DEMO" {
            return Err(VerifyError::EscrowLocked);
        }
    }

    let mut guard = state
        .used_nullifiers
        .lock()
        .expect("nullifier mutex poisoned");

    if guard.contains(nullifier) {
        return Err(VerifyError::DoubleSpend(
            "Nullifier already used".into(),
        ));
    }

    guard.insert(nullifier.to_string());

    Ok(())
}

impl<S> Service<Request<Body>> for ShadowPayMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response, S::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let state = self.state.clone();

        BoxFuture::from(async move {
            let path = req.uri().path().to_string();
            let is_protected = path.starts_with("/v1/protected");

            if !is_protected {
                return inner.call(req).await;
            }

            let headers = req.headers();

            if !has_shadowpay_headers(headers) {
                let body = ErrorBody {
                    status: 434,
                    title: "Private Payment Proof Required",
                    detail: "ShadowPay proof headers are required on this endpoint.".into(),
                };
                let resp = (StatusCode::from_u16(434).unwrap(), Json(body)).into_response();
                return Ok(resp);
            }

            match verify_shadowpay(headers, &state) {
                Ok(()) => inner.call(req).await,
                Err(VerifyError::MissingHeaders) => {
                    let body = ErrorBody {
                        status: 434,
                        title: "Private Payment Proof Required",
                        detail: "ShadowPay proof headers are incomplete.".into(),
                    };
                    let resp = (StatusCode::from_u16(434).unwrap(), Json(body)).into_response();
                    Ok(resp)
                }
                Err(VerifyError::InvalidProof(msg)) => {
                    let body = ErrorBody {
                        status: 422,
                        title: "Invalid ShadowPay Proof",
                        detail: msg,
                    };
                    let resp = (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response();
                    Ok(resp)
                }
                Err(VerifyError::DoubleSpend(msg)) => {
                    let body = ErrorBody {
                        status: 409,
                        title: "ShadowPay Nullifier Conflict",
                        detail: msg,
                    };
                    let resp = (StatusCode::CONFLICT, Json(body)).into_response();
                    Ok(resp)
                }
                Err(VerifyError::EscrowLocked) => {
                    let body = ErrorBody {
                        status: 423,
                        title: "ShadowPay Escrow Locked",
                        detail: "Escrow account is locked for this demo.".into(),
                    };
                    let resp = (StatusCode::LOCKED, Json(body)).into_response();
                    Ok(resp)
                }
                Err(VerifyError::PreconditionMissing(msg)) => {
                    let body = ErrorBody {
                        status: 428,
                        title: "ShadowPay Precondition Required",
                        detail: msg,
                    };
                    let resp = (StatusCode::PRECONDITION_REQUIRED, Json(body)).into_response();
                    Ok(resp)
                }
            }
        })
    }
}

async fn public_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "data": "public ok"
    }))
}

async fn protected_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "data": "protected ok"
    }))
}

async fn demo_invoice_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "invoice_id": "inv_demo_1",
        "currency": "USDC",
        "scheme": "shadowpay_v1",
        "note": "Use this invoice id when testing HTTP 434 with ShadowPay in Axum."
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v1/public", get(public_handler))
        .route("/v1/protected", get(protected_handler))
        .route("/v1/demo-invoice", get(demo_invoice_handler))
        .layer(ShadowPayLayer::new());

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("ShadowPay 434 Axum demo on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
