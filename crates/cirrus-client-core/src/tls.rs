use std::{fs::File, io::{BufReader, Read}, path::PathBuf};

use tonic::transport::{Certificate, ClientTlsConfig};

pub fn load_cert(cert_path: &PathBuf, domain_name: &str) -> Result<ClientTlsConfig, anyhow::Error> {
    let pem_file = File::open(cert_path).unwrap();
    let mut pem_file = BufReader::new(pem_file);
    let mut pem = Vec::new();

    pem_file.read_to_end(&mut pem).unwrap();

    let ca = Certificate::from_pem(pem);
    let tls_config = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name(domain_name);

    Ok(tls_config)
}