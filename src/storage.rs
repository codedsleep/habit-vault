use crate::encryption::{Encryption, EncryptedData, generate_salt};
use crate::habit::HabitData;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct SecureStorage {
    data_path: PathBuf,
}

impl SecureStorage {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not find data directory")?
            .join("rust-gtk-habits");
        
        fs::create_dir_all(&data_dir)?;
        
        Ok(Self {
            data_path: data_dir.join("habits.encrypted"),
        })
    }
    
    pub fn save(&self, data: &HabitData, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_vec(data)?;
        
        let salt = generate_salt();
        let encryption = Encryption::new(password, &salt)?;
        let mut encrypted_data = encryption.encrypt(&json_data)?;
        encrypted_data.salt = salt.to_vec();
        
        let encrypted_json = serde_json::to_vec(&encrypted_data)?;
        fs::write(&self.data_path, encrypted_json)?;
        
        Ok(())
    }
    
    pub fn load(&self, password: &str) -> Result<HabitData, Box<dyn std::error::Error>> {
        if !self.data_path.exists() {
            return Ok(HabitData::new());
        }
        
        let encrypted_json = fs::read(&self.data_path)?;
        let encrypted_data: EncryptedData = serde_json::from_slice(&encrypted_json)?;
        
        let encryption = Encryption::new(password, &encrypted_data.salt)?;
        let decrypted_data = encryption.decrypt(&encrypted_data)?;
        
        let habit_data: HabitData = serde_json::from_slice(&decrypted_data)?;
        Ok(habit_data)
    }
    
    pub fn exists(&self) -> bool {
        self.data_path.exists()
    }
    
    pub fn export_backup(&self, current_password: &str, backup_password: &str, backup_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let habit_data = self.load(current_password)?;
        let json_data = serde_json::to_vec(&habit_data)?;
        
        let salt = generate_salt();
        let encryption = Encryption::new(backup_password, &salt)?;
        let mut encrypted_data = encryption.encrypt(&json_data)?;
        encrypted_data.salt = salt.to_vec();
        
        let encrypted_json = serde_json::to_vec(&encrypted_data)?;
        fs::write(backup_path, encrypted_json)?;
        
        Ok(())
    }
    
    pub fn import_backup(&self, backup_path: &std::path::Path, backup_password: &str, new_password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_json = fs::read(backup_path)?;
        let encrypted_data: EncryptedData = serde_json::from_slice(&encrypted_json)?;
        
        let encryption = Encryption::new(backup_password, &encrypted_data.salt)?;
        let decrypted_data = encryption.decrypt(&encrypted_data)?;
        
        let habit_data: HabitData = serde_json::from_slice(&decrypted_data)?;
        self.save(&habit_data, new_password)?;
        
        Ok(())
    }
    
    pub fn delete_all_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.data_path.exists() {
            fs::remove_file(&self.data_path)?;
        }
        Ok(())
    }
}