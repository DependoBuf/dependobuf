use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, de::DeserializeOwned};
use std::error::Error;
use std::fmt;
use serde_json::Value;
use sled;

mod helper;
use helper::get_json_hash;

#[derive(Debug)]
pub enum DbError {
    SerializationError(String),
    DeserializationError(String),
    NotFound,
    DatabaseError(String),
    AlreadyExists(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DbError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            DbError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            DbError::NotFound => write!(f, "Item not found"),
            DbError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            DbError::AlreadyExists(msg) => write!(f, "Item already exists: {}", msg),
        }
    }
}

impl Error for DbError {}

impl From<sled::Error> for DbError {
    fn from(err: sled::Error) -> Self {
        DbError::DatabaseError(err.to_string())
    }
}

pub struct Database {
    db: sled::Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, DbError> {
        let db = sled::open(path).map_err(|e| DbError::DatabaseError(e.to_string()))?;
        
        Ok(Database { db })
    }

    fn generate_id(&self, dependencies_hash: &str) -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        
        format!("{}_{}", dependencies_hash, rand_string)
    }

    fn extract_dependencies_hash(&self, id: &str) -> Result<String, DbError> {
        let parts: Vec<&str> = id.split('_').collect();
        if parts.is_empty() {
            return Err(DbError::DeserializationError("Invalid ID format".to_string()));
        }
        Ok(parts[0].to_string())
    }
    
    pub fn insert<T: Serialize>(&self, value: &T) -> Result<String, DbError> {
        let json = serde_json::to_string(value)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        
        self.insert_json(json)
    }

    fn insert_json(&self, json: String) -> Result<String, DbError> {   
        let value: Value = serde_json::from_str(&json)
            .map_err(|e| DbError::DeserializationError(format!("JSON parsing error: {}", e)))?;

        let obj = match value.as_object() {
            Some(obj) => obj,
            None => return Err(DbError::DeserializationError(
                "Invalid JSON structure: Not an object".to_string()
            )),
        };

        if obj.len() != 2 || !obj.contains_key("body") || !obj.contains_key("dependencies") {
            return Err(DbError::DeserializationError(
                "Invalid JSON structure: Message must contains exactly 2 keys: 'body' and 'dependencies'".to_string()
            ));
        }

        let body = &value["body"];
        let dependencies = &value["dependencies"];

        let body_json = serde_json::to_string(body)
            .map_err(|e| DbError::SerializationError(format!("Failed to serialize body: {}", e)))?;
        let dependencies_json = serde_json::to_string(dependencies)
            .map_err(|e| DbError::SerializationError(format!("Failed to serialize dependencies: {}", e)))?;

        let deps_hash = get_json_hash(&dependencies_json);
        let id = self.generate_id(&deps_hash);

        if self.db.contains_key(id.as_bytes())? {
            return Err(DbError::AlreadyExists(format!("Item with ID {} already exists", id)));
        }
        
        // TODO: make parallel
        self.db.insert(deps_hash.as_bytes(), dependencies_json.as_bytes())?;
        self.db.insert(id.as_bytes(), body_json.as_bytes())?;
        self.db.flush()?;

        Ok(id)
    }
    
    pub fn get<T: DeserializeOwned>(&self, id: &str) -> Result<T, DbError> {
        let json = self.get_json(id)?;
        
        serde_json::from_str(&json)
            .map_err(|e| DbError::DeserializationError(e.to_string()))
    }

    fn get_json(&self, id: &str) -> Result<String, DbError> {
        let body_json = self.get_by_key(id)?;

        let deps_hash = self.extract_dependencies_hash(id)?;
        let deps_json = self.get_by_key(&deps_hash)?;

        let complete_json = format!(
            "{{ \"body\": {}, \"dependencies\": {} }}", 
            body_json,
            deps_json
        );

        Ok(complete_json)
    }

    fn get_by_key(&self, id: &str) -> Result<String, DbError> {
        match self.db.get(id.as_bytes())? {
            Some(bytes) => {
                let json = String::from_utf8(bytes.to_vec())
                    .map_err(|e| DbError::DeserializationError(format!("UTF-8 error: {}", e)))?;
                Ok(json)
            }
            None => Err(DbError::NotFound),
        }
    }
    
    pub fn update<T: Serialize>(&self, id: &str, value: &T) -> Result<(), DbError> {
        let json = serde_json::to_string(value)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        
        self.update_json(id, json)?;
        
        Ok(())
    }

    fn update_json(&self, id: &str, json: String) -> Result<(), DbError> {
        let current_body_json = self.get_by_key(id)?;
        let deps_hash = self.extract_dependencies_hash(id)?;
        let current_deps_json = self.get_by_key(&deps_hash)?;

        let value: Value = serde_json::from_str(&json)
            .map_err(|e| DbError::DeserializationError(format!("JSON parsing error: {}", e)))?;

        let obj = match value.as_object() {
            Some(obj) => obj,
            None => return Err(DbError::DeserializationError(
                "Invalid JSON structure: Not an object".to_string()
            )),
        };

        if obj.len() != 2 || !obj.contains_key("body") || !obj.contains_key("dependencies") {
            return Err(DbError::DeserializationError(
                "Invalid JSON structure: Message must contains exactly 2 keys: 'body' and 'dependencies'".to_string()
            ));
        }

        let new_body = &value["body"];
        let new_dependencies = &value["dependencies"];

        let new_body_json = serde_json::to_string(new_body)
            .map_err(|e| DbError::SerializationError(format!("Failed to serialize body: {}", e)))?;
        let new_dependencies_json = serde_json::to_string(new_dependencies)
            .map_err(|e| DbError::SerializationError(format!("Failed to serialize dependencies: {}", e)))?;

        let body_changed = current_body_json != new_body_json;
        let deps_changed = current_deps_json != new_dependencies_json;

        if !body_changed && !deps_changed {
            return Ok(());
        }

        if body_changed {
            self.db.insert(id.as_bytes(), new_body_json.as_bytes())?;
        }

        if deps_changed {
            self.db.insert(deps_hash.as_bytes(), new_dependencies_json.as_bytes())?;
        }

        self.db.flush()?;
        Ok(())
    }
    
    pub fn delete(&self, id: &str) -> Result<(), DbError> {
        self.delete_json(id)
    }

    pub fn delete_json(&self, id: &str) -> Result<(), DbError> {
        if !self.db.contains_key(id.as_bytes())? {
            return Err(DbError::NotFound);

        }

        self.db.remove(id.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }
}
