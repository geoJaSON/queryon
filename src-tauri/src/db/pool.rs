//! Builds a `deadpool` pool for a profile. A rustls connector is always
//! installed; `tokio_postgres` only negotiates TLS when the configured
//! `ssl_mode` asks for it, so one uniform pool type covers all three modes.
//!
//! The cert verifier accepts any server certificate — this matches libpq's
//! `require` semantics (encrypt, do not verify CA). Verifying modes
//! (`verify-ca`/`verify-full`) are intentionally not exposed.

use std::sync::Arc;

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use tokio_postgres::config::SslMode as PgSslMode;
use tokio_postgres::Config as PgConfig;
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::error::{AppError, AppResult};
use crate::models::{ConnectionProfile, SslMode};

#[derive(Debug)]
struct NoVerify;

impl ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

fn tls_connector() -> MakeRustlsConnect {
    let cfg = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerify))
        .with_no_client_auth();
    MakeRustlsConnect::new(cfg)
}

pub fn build_pool(p: &ConnectionProfile, password: &str) -> AppResult<Pool> {
    let mut cfg = PgConfig::new();
    cfg.host(&p.host)
        .port(p.port)
        .dbname(&p.database)
        .user(&p.username)
        .application_name("queryon");
    if !password.is_empty() {
        cfg.password(password);
    }
    cfg.ssl_mode(match p.ssl_mode {
        SslMode::Disable => PgSslMode::Disable,
        SslMode::Prefer => PgSslMode::Prefer,
        SslMode::Require => PgSslMode::Require,
    });

    let mgr = Manager::from_config(
        cfg,
        tls_connector(),
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );
    Pool::builder(mgr)
        .max_size(8)
        .build()
        .map_err(|e| AppError::Pool(e.to_string()))
}
