//! Builds a `deadpool` pool for a profile. A rustls connector is always
//! installed; `tokio_postgres` only negotiates TLS when the configured
//! `ssl_mode` asks for it, so one uniform pool type covers all three modes.
//!
//! The cert verifier accepts any server certificate — this matches libpq's
//! `require` semantics (encrypt, do not verify CA). Verifying modes
//! (`verify-ca`/`verify-full`) are intentionally not exposed.

use std::sync::{Arc, Mutex};

use deadpool_postgres::{Connect, Manager, ManagerConfig, Pool, RecyclingMethod};
use futures_util::future::BoxFuture;
use futures_util::StreamExt;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use tokio::task::JoinHandle;
use tokio_postgres::config::SslMode as PgSslMode;
use tokio_postgres::{AsyncMessage, Client as PgClient, Config as PgConfig};
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::error::{AppError, AppResult};
use crate::models::{ConnectionProfile, SslMode};

/// Per-connection collector for server NOTICE/WARNING messages. All pool
/// connections of one profile share a sink; `run_sql` drains it per statement.
pub type NoticeSink = Arc<Mutex<Vec<String>>>;

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

pub fn tls_connector() -> MakeRustlsConnect {
    // rustls 0.23 with `default-features = false` does NOT install a
    // process-level default CryptoProvider, so `ClientConfig::builder()`
    // would panic. Pass the `ring` provider explicitly instead of relying
    // on global install ordering.
    let cfg = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("ring provider supports the safe default protocol versions")
    .dangerous()
    .with_custom_certificate_verifier(Arc::new(NoVerify))
    .with_no_client_auth();
    MakeRustlsConnect::new(cfg)
}

/// Like deadpool's default connector, but polls the connection for async
/// messages so NOTICE/WARNING (e.g. RAISE NOTICE) reach the UI instead of
/// being dropped.
struct NoticeConnect {
    sink: NoticeSink,
}

impl Connect for NoticeConnect {
    fn connect(
        &self,
        pg_config: &PgConfig,
    ) -> BoxFuture<'_, Result<(PgClient, JoinHandle<()>), tokio_postgres::Error>> {
        let sink = self.sink.clone();
        let pg_config = pg_config.clone();
        Box::pin(async move {
            let (client, mut connection) = pg_config.connect(tls_connector()).await?;
            let task = tokio::spawn(async move {
                let mut messages =
                    futures_util::stream::poll_fn(move |cx| connection.poll_message(cx));
                while let Some(msg) = messages.next().await {
                    match msg {
                        Ok(AsyncMessage::Notice(n)) => {
                            sink.lock()
                                .unwrap()
                                .push(format!("{}: {}", n.severity(), n.message()));
                        }
                        Ok(_) => {}
                        Err(e) => {
                            tracing::warn!("postgres connection error: {e}");
                            break;
                        }
                    }
                }
            });
            Ok((client, task))
        })
    }
}

pub fn build_pool(p: &ConnectionProfile, password: &str) -> AppResult<(Pool, NoticeSink)> {
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

    let sink: NoticeSink = Arc::new(Mutex::new(Vec::new()));
    let mgr = Manager::from_connect(
        cfg,
        NoticeConnect { sink: sink.clone() },
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );
    let pool = Pool::builder(mgr)
        .max_size(8)
        .build()
        .map_err(|e| AppError::Pool(e.to_string()))?;
    Ok((pool, sink))
}
