//! # QUIC Utilities
//!
//! Common utilities for QUIC server and client setup, including certificate generation.

use quinn::{Endpoint, ServerConfig, ClientConfig};
use rustls::{Certificate, PrivateKey, ServerConfig as TlsServerConfig, ClientConfig as TlsClientConfig};
use std::sync::Arc;

/// Generates a self-signed certificate for QUIC/TLS.
///
/// QUIC requires TLS 1.3 encryption, so we need a certificate.
/// This function creates a self-signed certificate for development/testing.
/// In production, use a certificate from a trusted CA.
///
/// # Returns
/// Tuple of (Certificate, PrivateKey) in DER format
///
/// # Note
/// Self-signed certificates will show security warnings in browsers/clients.
/// This is expected for development use.
pub fn generate_self_signed_cert() -> Result<(Certificate, PrivateKey), Box<dyn std::error::Error>> {
    use rcgen::{CertificateParams, KeyPair, DistinguishedName, DnType, Certificate as RcgenCert};
    
    let mut params = CertificateParams::new(vec!["localhost".to_string()]);
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(DnType::CommonName, "Simple Torrent QUIC");
    
    let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)?;
    let cert = RcgenCert::from_params(params)?;
    
    let cert_der = cert.serialize_der()?;
    let key_der = key_pair.serialize_der();
    
    Ok((
        Certificate(cert_der),
        PrivateKey(key_der),
    ))
}

/// Creates a QUIC server configuration with a self-signed certificate.
///
/// # Returns
/// `ServerConfig` ready to use for creating a QUIC endpoint
pub fn create_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    // Generate self-signed certificate
    let (cert, key) = generate_self_signed_cert()?;
    
    // Configure TLS server settings
    let mut tls_config = TlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    
    // Set ALPN with multiple protocols for compatibility (fallback mechanism)
    // Order matters: most preferred first, fallbacks follow
    // This ensures ALPN negotiation succeeds even if there are parsing issues
    tls_config.alpn_protocols = vec![
        b"h3".to_vec(),           // Primary: HTTP/3 (QUIC)
        b"h2".to_vec(),           // Fallback 1: HTTP/2
        b"http/1.1".to_vec(),     // Fallback 2: HTTP/1.1
        b"doq".to_vec(),          // Fallback 3: DNS over QUIC
    ];
    
    // Convert rustls ServerConfig to quinn's crypto trait (quinn 0.10)
    use quinn::crypto::ServerConfig as CryptoServerConfig;
    let crypto: Arc<dyn CryptoServerConfig> = Arc::new(tls_config);
    
    // Create QUIC server configuration
    let mut server_config = ServerConfig::with_crypto(crypto);
    
    // Configure transport settings
    // Allow up to 100 concurrent bidirectional streams per connection
    let transport = Arc::get_mut(&mut server_config.transport).unwrap();
    transport.max_concurrent_bidi_streams(100u32.into());
    
    Ok(server_config)
}

/// Creates a QUIC client configuration that accepts self-signed certificates.
///
/// # Returns
/// `ClientConfig` ready to use for creating a QUIC client endpoint
pub fn create_client_config() -> Result<ClientConfig, Box<dyn std::error::Error>> {
    // For development: create a client config that accepts any certificate
    // WARNING: This is insecure and should only be used for development
    use rustls::client::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    
    struct AcceptAllVerifier;
    impl ServerCertVerifier for AcceptAllVerifier {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::Certificate,
            _intermediates: &[rustls::Certificate],
            _server_name: &rustls::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }
        
        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::Certificate,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        
        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::Certificate,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        
        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            vec![
                rustls::SignatureScheme::RSA_PKCS1_SHA256,
                rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
                rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
                rustls::SignatureScheme::RSA_PKCS1_SHA384,
                rustls::SignatureScheme::RSA_PKCS1_SHA512,
                rustls::SignatureScheme::RSA_PSS_SHA256,
                rustls::SignatureScheme::RSA_PSS_SHA384,
                rustls::SignatureScheme::RSA_PSS_SHA512,
            ]
        }
    }
    
    let mut tls_config = TlsClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();
    
    // Set ALPN with multiple protocols for compatibility (fallback mechanism)
    // Order matters: most preferred first, fallbacks follow
    // This ensures ALPN negotiation succeeds even if there are parsing issues
    tls_config.alpn_protocols = vec![
        b"h3".to_vec(),           // Primary: HTTP/3 (QUIC)
        b"h2".to_vec(),           // Fallback 1: HTTP/2
        b"http/1.1".to_vec(),     // Fallback 2: HTTP/1.1
        b"doq".to_vec(),          // Fallback 3: DNS over QUIC
    ];
    
    // FALLBACK SHUNT: Multiple ALPN protocols provide automatic fallback
    // The TLS handshake will negotiate the first common protocol
    // This bypasses parsing issues by offering multiple options
    
    // Disable certificate validation for development
    tls_config.dangerous().set_certificate_verifier(Arc::new(AcceptAllVerifier));
    
    // Convert rustls ClientConfig to quinn's crypto trait (quinn 0.10)
    use quinn::crypto::ClientConfig as CryptoClientConfig;
    let crypto: Arc<dyn CryptoClientConfig> = Arc::new(tls_config);
    
    let client_config = ClientConfig::new(crypto);
    
    Ok(client_config)
}

