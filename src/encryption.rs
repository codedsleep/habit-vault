use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

pub struct Encryption {
    cipher: Aes256Gcm,
}

impl Encryption {
    pub fn new(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt).map_err(|e| format!("Salt encoding error: {}", e))?;
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| format!("Password hashing error: {}", e))?;
        
        let hash = password_hash.hash.unwrap();
        let key_bytes = hash.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt(&self, data: &[u8]) -> Result<EncryptedData, Box<dyn std::error::Error>> {
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted_data = self.cipher.encrypt(nonce, data)
            .map_err(|e| format!("Encryption error: {}", e))?;
        
        Ok(EncryptedData {
            data: encrypted_data,
            nonce: nonce_bytes.to_vec(),
            salt: vec![], // Salt will be set by caller
        })
    }
    
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let nonce = Nonce::from_slice(&encrypted.nonce);
        let decrypted_data = self.cipher.decrypt(nonce, encrypted.data.as_ref())
            .map_err(|e| format!("Decryption error: {}", e))?;
        
        Ok(decrypted_data)
    }
}

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    thread_rng().fill_bytes(&mut salt);
    salt
}