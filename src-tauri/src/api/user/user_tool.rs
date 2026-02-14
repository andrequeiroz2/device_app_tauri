use scrypt::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Scrypt,
};
use crate::api::error::map_password_hash_error;
use tracing::{instrument, error};

#[instrument(skip(password))]
pub fn get_password_hash(password: &String) -> Result<String, String> {

    let salt = SaltString::generate(&mut OsRng);
    
    match Scrypt.hash_password(password.as_bytes(), &salt){
        Ok(hash) => Ok(hash.to_string()),
        Err(err)=> {
            error!(error = %err, "get_password_hash failed");
            Err(map_password_hash_error(&err))
        }
    }
}