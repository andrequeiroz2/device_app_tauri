use rsa::{pkcs8::EncodePrivateKey, pkcs8::EncodePublicKey, RsaPrivateKey};
use rsa::rand_core::OsRng;
use std::fs;
use tauri::{AppHandle, Manager};
use tracing::{info, error};
use jwt_lib::components::key::{JwtPath, Jwtkey};
use super::auth_model::{AuthKeys, KeyPairPaths};
use password_hash::{PasswordHash, PasswordVerifier};
use scrypt::Scrypt;

pub fn ensure_keys(app_handle: &AppHandle) -> tauri::Result<KeyPairPaths> {

    let mut base_dir = app_handle.path().app_config_dir()?;
    base_dir.push("keys");

    if !base_dir.exists() {
        fs::create_dir_all(&base_dir)?;
    }

    let priv_path = base_dir.join("private_key.pem");
    let pub_path = base_dir.join("public_key.pem");

    if !priv_path.exists() || !pub_path.exists() {
        info!("Keys not found, generating new RSA keypair");
            let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048)
            .map_err(|e| tauri::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        let public_key = private_key.to_public_key();

        let priv_pem = private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| tauri::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        let pub_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| tauri::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        fs::write(&priv_path, priv_pem.as_bytes())?;
        fs::write(&pub_path, pub_pem.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&priv_path, fs::Permissions::from_mode(0o600))?;
        }
    }

    Ok(KeyPairPaths {
        private_key: priv_path,
        public_key: pub_path,
    })
}

/// Carrega as chaves em memória (strings PEM) para uso em state compartilhado.
pub fn load_keys_to_memory(paths: &KeyPairPaths) -> tauri::Result<AuthKeys> {
    let private_key_pem = fs::read_to_string(&paths.private_key)?;
    let public_key_pem = fs::read_to_string(&paths.public_key)?;

    Ok(AuthKeys {
        private_key_pem,
        public_key_pem,
    })
}

pub fn setup_auth_keys(private_key_path: &str, public_key_path: &str) -> Result<(), String> {
    JwtPath::set_private_key_path(private_key_path)
        .map_err(|e| format!("Set private key path error: {}", e))?;
    JwtPath::set_public_key_path(public_key_path)
        .map_err(|e| format!("Set public key path error: {}", e))?;
    Jwtkey::set_private_key().map_err(|e| format!("Set private key error: {}", e))?;
    Jwtkey::set_public_key().map_err(|e| format!("Set public key error: {}", e))?;
    Ok(())
}

/// Verifica se o `password` em texto plano corresponde ao `password_hash` scrypt armazenado.
/// Retorna Ok em caso de sucesso; em falha, loga detalhes e devolve mensagem amigável.
pub fn verify_password(password: &str, password_hash: &str) -> Result<(), String> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|err| {
        error!(
            error = %err,
            "verify_password: failed to parse hash"
        );
        "Incorrect password".to_string()
    })?;

    info!("verify_password: parsed_hash loaded");

    Scrypt
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            error!(
                "verify_password: invalid password"
            );
            "Incorrect password".to_string()
        })
}


