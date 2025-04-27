use aes_gcm_siv::{
    aead::{self, consts::U12, generic_array::GenericArray, Aead, KeyInit, OsRng},
    AeadCore, Aes128GcmSiv, Key,
};
use scrypt::{password_hash::SaltString, scrypt, Params};
use std::{fs, path::PathBuf, str};

use crate::{append_to_file, clear_file_content, create_file, hash, write_to_file};

pub use super::models::RecordOperationConfig;

/// User
/// Data about a user is not exposed to the outside world
/// Only methods to interact with the user data are exposed
///
/// # Fields
/// * `0` - The records
/// * `1` - The path to the user data
/// * `2` - The username
#[derive(Debug, Clone, PartialEq)]
pub struct User(Vec<Record>, PathBuf, Username);

/// ReadOnlyRecords is a read-only version of the records
/// It is used to return records to the user
/// The user can only read the records and not modify them
/// This is to prevent the user from modifying the records directly
/// and to ensure that the records are always encrypted
///
/// # Fields
/// * `0` - Vector of domain-password pairs
#[derive(Debug, Clone, PartialEq)]
pub struct ReadOnlyRecords(Vec<(String, String)>);

/// A domain-password pair
///
/// # Fields
///
/// * `domain` - The domain
/// * `password` - The password
#[derive(Debug, Clone, PartialEq)]
struct DomainPasswordPair {
    domain: String,
    password: String,
}

/// CipherConfig is a configuration for the cipher
///
/// # Fields
/// * `key` - The key
/// * `salt` - The salt
/// * `nonce` - The nonce
/// * `ciphertext` - The ciphertext
#[derive(Debug, Clone, PartialEq)]
struct CipherConfig {
    key: Key<Aes128GcmSiv>,
    salt: Vec<u8>,                // 22 bytes
    nonce: GenericArray<u8, U12>, // 12 bytes
    ciphertext: Vec<u8>,
}

/// DerivedKey is a derived key from a master password
///
/// # Fields
/// * `key` - The key
/// * `salt` - The salt
#[derive(Debug, Clone, PartialEq)]
struct DerivedKey {
    pub key: [u8; 16],
    pub salt: Vec<u8>,
}

/// Record represents an encrypted domain-password pair
///
/// # Fields
/// * `cypher` - The cipher configuration
/// * `offset` - The offset in the file
#[derive(Debug, Clone, PartialEq)]
struct Record {
    cypher: CipherConfig,
    offset: u32,
}

/// Username represents the username of a user
///
/// # Fields
/// * `0` - The username
#[derive(Debug, Clone, PartialEq)]
struct Username(String);

impl CipherConfig {
    /// Creates a new `CipherConfig`
    ///
    /// # Arguments
    /// * `key` - The key
    /// * `salt` - The salt
    /// * `nonce` - The nonce
    /// * `ciphertext` - The ciphertext
    ///
    /// # Returns
    /// A new `CipherConfig`
    fn new(
        key: Key<Aes128GcmSiv>,
        salt: Vec<u8>,
        nonce: GenericArray<u8, U12>,
        ciphertext: Vec<u8>,
    ) -> Self {
        CipherConfig {
            key,
            salt,
            nonce,
            ciphertext,
        }
    }

    /// Marshals the domain and password into a string
    /// escaping spaces and backslashes in the process
    /// Domain and password are separated by a space after escaping
    ///
    /// # Arguments
    /// * `domain` - The domain
    /// * `password` - The password
    ///
    /// # Returns
    /// The marshalled string
    fn marshal(domain: &str, password: &str) -> String {
        let marshal_domain = domain.to_string().replace("\\", "\\\\").replace(" ", "\\s");
        let marshal_password = password
            .to_string()
            .replace("\\", "\\\\")
            .replace(" ", "\\s");
        format!("{} {}", marshal_domain, marshal_password)
    }

    /// Unmarshals the domain and password from a string
    /// unescaping spaces and backslashes in the process
    ///
    /// # Arguments
    /// * `data` - The data to unmarshal
    ///
    /// # Returns
    /// A tuple of the domain and password
    fn unmarshal(data: &str) -> (String, String) {
        let parts: Vec<&str> = data.split_whitespace().collect();
        let domain = parts[0].replace("\\s", " ").replace("\\\\", "\\");
        let password = parts[1].replace("\\s", " ").replace("\\\\", "\\");
        (domain, password)
    }

    /// Writes the cipher configuration to a buffer modifying the buffer
    ///
    /// # Arguments
    /// * `buffer` - The buffer to write to
    fn write(&self, buffer: &mut Vec<u8>) {
        // this is needed to get the length of the ciphertext
        // so that we can read it back from the file
        let ciphertext_len: u32 = self.ciphertext.len() as u32;
        let mut data: Vec<u8> = self.salt.clone();

        data.append(&mut self.nonce.to_vec());
        data.append(&mut ciphertext_len.to_be_bytes().to_vec());
        data.append(&mut self.ciphertext.clone());

        buffer.append(&mut data);
    }

    /// Encrypts the domain and password using the master password
    ///
    /// # Arguments
    /// * `domain` - The domain
    /// * `password` - The password
    /// * `master_password` - The master password
    ///
    /// # Returns
    /// A new `CipherConfig` or an error
    fn encrypt_data(
        domain: &str,
        password: &str,
        master_password: &str,
    ) -> Result<Self, aead::Error> {
        let derived_key = DerivedKey::derive_key(master_password, None);
        let salt = derived_key.salt;
        let key = Key::<Aes128GcmSiv>::clone_from_slice(&derived_key.key);
        let cipher = Aes128GcmSiv::new(&key);
        let nonce = Aes128GcmSiv::generate_nonce(&mut OsRng);
        let data = CipherConfig::marshal(domain, password);

        let ciphertext = cipher.encrypt(&nonce, data.as_bytes())?;
        Ok(CipherConfig::new(key, salt, nonce, ciphertext))
    }

    /// Decrypts the domain and password
    ///
    /// # Returns
    /// A new `DomainPasswordPair` or an error if decryption fails
    fn decrypt_data(&self) -> Result<DomainPasswordPair, aead::Error> {
        let cipher = Aes128GcmSiv::new(&self.key);
        let plaintext = cipher.decrypt(&self.nonce, self.ciphertext.as_ref())?;
        let (domain, password) = CipherConfig::unmarshal(str::from_utf8(&plaintext).unwrap());
        Ok(DomainPasswordPair { domain, password })
    }
}

impl DerivedKey {
    /// Creates a new `DerivedKey`
    ///
    /// # Arguments
    /// * `key` - The key
    /// * `salt` - The salt
    ///
    /// # Returns
    /// A new `DerivedKey`
    fn new(key: [u8; 16], salt: Vec<u8>) -> Self {
        DerivedKey { key, salt }
    }

    /// Derives a key from a master password
    /// If a salt is provided, it means that the key is being derived
    /// from an existing salt. If salt is None, a new salt is generated
    ///
    /// # Arguments
    /// * `data` - The data to derive the key from
    /// * `salt` - The salt
    ///
    /// # Returns
    /// A new `DerivedKey`
    fn derive_key(data: &str, salt: Option<Vec<u8>>) -> Self {
        let salt = match salt {
            Some(salt) => salt,
            None => SaltString::generate(&mut OsRng)
                .as_str()
                .as_bytes()
                .to_vec(),
        };
        let salt_copy = salt.clone();
        let mut derived_key = [0u8; 16];
        scrypt(
            &data.as_bytes(),
            &salt,
            &Params::new(14 as u8, 8 as u32, 1 as u32, 16 as usize).unwrap(),
            &mut derived_key,
        )
        .unwrap();

        DerivedKey::new(derived_key, salt_copy)
    }
}

impl Record {
    /// Creates a new `Record`
    ///
    /// # Arguments
    /// * `cypher` - The cipher configuration
    /// * `offset` - The offset
    ///
    /// # Returns
    /// A new `Record`
    fn new(cypher: CipherConfig, offset: u32) -> Self {
        Record { cypher, offset }
    }

    /// Reads a record from bytes and returns the record,
    /// the remaining bytes and the current offset
    ///
    /// # Arguments
    /// * `bytes` - The bytes to read from
    /// * `master_password` - The master password
    /// * `offset` - The offset
    ///
    /// # Returns
    /// A tuple of the record, the remaining bytes and the current offset
    fn read_from_bytes(
        bytes: Vec<u8>,
        master_password: &str,
        offset: u32,
    ) -> Result<(Self, Vec<u8>, u32), aead::Error> {
        let salt = bytes[0..22].to_vec();
        let nonce = GenericArray::clone_from_slice(&bytes[22..34]);
        let ciphertext_len = u32::from_be_bytes(bytes[34..38].try_into().unwrap());
        let ciphertext = bytes[38..(38 + ciphertext_len as usize)].to_vec();
        let derived_key = DerivedKey::derive_key(master_password, Some(salt.clone()));
        let key = Key::<Aes128GcmSiv>::clone_from_slice(&derived_key.key);
        let cipher_config = CipherConfig::new(key, salt, nonce, ciphertext);
        let current_offset = 38 + ciphertext_len as usize + offset as usize;

        Ok((
            Record::new(cipher_config, offset),
            bytes[(38 + ciphertext_len as usize)..].to_vec(),
            current_offset as u32,
        ))
    }

    /// Read user data from file
    ///
    /// # Arguments
    ///
    /// * `p` - Path to the directory where the file (users data) is stored
    /// * `username` - The username of the user
    /// * `master_password` - The master password of the user
    ///
    /// # Returns
    /// * `Result<Vec<Self>, String>` - A vector of records or an error message
    fn read_user(p: &PathBuf, username: &str, master_password: &str) -> Result<Vec<Self>, String> {
        let hash = hash(username.to_string());
        let file_path = p.join(hash.as_str());
        let mut data: Vec<Record> = Vec::new();
        let mut offset = 0;
        if file_path.exists() {
            let mut bytes = fs::read(file_path).unwrap();
            let mut run = true;
            while run {
                let res = Record::read_from_bytes(bytes, master_password, offset);
                if res.is_err() {
                    return Err("Could not read user".to_string());
                }
                let (cipher, remaining, next_offset) = res.unwrap();
                data.push(cipher);
                bytes = remaining;
                if bytes.len() == 0 {
                    run = false;
                }

                offset = next_offset;
            }
        } else {
            return Err("User not found".to_string());
        }
        Ok(data)
    }

    /// Decrypts the data
    ///
    /// # Returns
    /// The decrypted data or an error if decryption fails
    fn data(&self) -> Result<DomainPasswordPair, aead::Error> {
        self.cypher.decrypt_data()
    }
}

impl User {
    /// Creates a new `User` instance
    /// Does not create a new user in the file system
    ///
    /// # Arguments
    /// * `path` - The path to the user data
    /// * `username` - The username
    /// * `master_password` - The master password
    ///
    /// # Returns
    /// A new `User` and `ReadOnlyRecords` or an error message
    pub fn from(
        path: &PathBuf,
        username: &str,
        master_password: &str,
    ) -> Result<(Self, ReadOnlyRecords), String> {
        let records = Record::read_user(path, username, master_password);
        let records = match records {
            Ok(r) => r,
            Err(e) => return Err(e),
        };
        let mut read_only_records = vec![];

        for record in records.iter() {
            let decrypted = record.cypher.decrypt_data();
            match decrypted {
                Ok(decrypted) => {
                    read_only_records
                        .push((decrypted.domain.to_string(), decrypted.password.to_string()));
                }
                Err(_) => return Err("Could not decrypt data".to_string()),
            }
        }

        let path = path.join(hash(username.to_string()));

        Ok((
            User(records, path, Username(username.to_string())),
            ReadOnlyRecords(read_only_records),
        ))
    }

    /// Creates a new user and writes the user data to the file system
    ///
    /// # Arguments
    /// * `user` - The user configuration
    ///
    /// # Returns
    /// An error message if the user could not be created
    pub fn new(user: &RecordOperationConfig) -> Result<(), String> {
        let hashed_username = hash(user.username.to_string());
        let res = create_file(&user.path, hashed_username.as_str());
        let file_path = match res {
            Ok(path) => path,
            Err(_) => return Err("Could not create file.".to_string()),
        };

        let cipher =
            CipherConfig::encrypt_data(&user.domain, &user.password, &user.master_password);
        let cipher = match cipher {
            Ok(cipher) => cipher,
            Err(_) => return Err("Could not encrypt data.".to_string()),
        };
        let mut buffer = vec![];
        cipher.write(&mut buffer);
        match write_to_file(&file_path, buffer) {
            Ok(_) => Ok(()),
            Err(_) => Err("Could not write to file.".to_string()),
        }
    }

    /// Returns the username of the user
    ///
    /// # Returns
    /// The username
    pub fn username(&self) -> String {
        self.2.clone().0
    }

    /// Returns the path to the user data
    ///
    /// # Returns
    /// The path to the user data
    fn path(&self) -> PathBuf {
        self.1.clone()
    }

    /// Adds a new record to the user data
    /// The record is encrypted before being added
    ///
    /// # Arguments
    /// * `record` - The record configuration
    ///
    /// # Returns
    /// The read-only records or an error message
    pub fn add_record(&mut self, record: RecordOperationConfig) -> Result<ReadOnlyRecords, String> {
        let (integrity, ro_records) =
            self.check_integrity(&record.username, &record.master_password, &record.path);

        let mut ro_records = match ro_records {
            Some(ro_records) => ro_records,
            None => return Err("Could not read user".to_string()),
        };

        if !integrity {
            return Err("Integrity check failed".to_string());
        }

        if ro_records.0.iter().find(|r| r.0 == record.domain).is_some() {
            return Err("Record already exists".to_string());
        }

        ro_records.add_record(&record.domain, &record.password);
        let cipher =
            CipherConfig::encrypt_data(&record.domain, &record.password, &record.master_password);
        let cipher = match cipher {
            Ok(cipher) => cipher,
            Err(_) => return Err("Could not create user.".to_string()),
        };

        let offset = self.last_offset();
        let record = Record::new(cipher, offset);
        let mut buffer = vec![];
        record.cypher.write(&mut buffer);
        append_to_file(&self.path(), buffer).unwrap();
        self.0.push(record);

        Ok(ro_records)
    }

    /// Removes a record from the user data
    /// The record is removed by domain
    ///
    /// # Arguments
    /// * `record` - The record configuration
    ///
    /// # Returns
    /// The read-only records or an error message
    pub fn remove_record(
        &mut self,
        record: RecordOperationConfig,
    ) -> Result<ReadOnlyRecords, String> {
        let (integrity, ro_records) =
            self.check_integrity(&record.username, &record.master_password, &record.path);

        let mut ro_records = match ro_records {
            Some(ro_records) => ro_records,
            None => return Err("Could not read user".to_string()),
        };

        if !integrity {
            return Err("Integrity check failed".to_string());
        }

        let mut new_records = vec![];
        let mut found = false;
        for r in self.0.iter() {
            let data = match r.data() {
                Ok(data) => data,
                Err(_) => return Err("Could not read data".to_string()),
            };

            if data.domain != record.domain {
                new_records.push(r.clone());
                ro_records.remove_record(&record.domain);
            } else {
                found = true;
            }
        }

        if !found {
            return Err("Record not found".to_string());
        }

        // TODO: calibrate offsets or remove them

        self.remove_records_from_file();
        let path = self.path();
        let mut buffer = vec![];

        for record in new_records.iter() {
            record.cypher.write(&mut buffer);
        }

        write_to_file(&path, buffer).unwrap();
        self.0 = new_records;

        Ok(ro_records)
    }

    /// Modifies a record in the user data
    /// The record is modified by domain
    ///
    /// # Arguments
    /// * `record` - The record configuration
    ///
    /// # Returns
    /// The read-only records or an error message
    pub fn modify_record(
        &mut self,
        record: RecordOperationConfig,
    ) -> Result<ReadOnlyRecords, String> {
        let (integrity, ro_records) =
            self.check_integrity(&record.username, &record.master_password, &record.path);

        let mut ro_records = match ro_records {
            Some(ro_records) => ro_records,
            None => return Err("Could not read user".to_string()),
        };

        if !integrity {
            return Err("Integrity check failed".to_string());
        }

        let mut new_records = vec![];
        let mut found = false;
        for r in self.0.iter() {
            let data = match r.data() {
                Ok(data) => data,
                Err(_) => return Err("Could not read data".to_string()),
            };

            if data.domain != record.domain {
                new_records.push(r.clone());
                ro_records.remove_record(&record.domain);
            } else {
                found = true;
            }
        }

        if !found {
            return Err("Record not found".to_string());
        }

        ro_records.add_record(&record.domain, &record.password);

        let cipher =
            CipherConfig::encrypt_data(&record.domain, &record.password, &record.master_password);
        let cipher = match cipher {
            Ok(cipher) => cipher,
            Err(_) => return Err("Could not create user.".to_string()),
        };

        let record = Record::new(cipher, self.last_offset());

        new_records.push(record);

        let mut buffer = vec![];
        for record in new_records.iter() {
            record.cypher.write(&mut buffer);
        }

        write_to_file(&self.path(), buffer).unwrap();
        self.0 = new_records;

        Ok(ro_records)
    }

    /// Returns the last offset in the file
    ///
    /// # Returns
    /// The last offset
    fn last_offset(&self) -> u32 {
        let mut offset = 0;
        for record in self.0.iter() {
            if record.offset > offset {
                offset = record.offset;
            }
        }

        offset
    }

    /// Checks the integrity of the user data
    /// The integrity is checked by decrypting the data
    /// If the data cannot be decrypted, the integrity check fails
    ///
    /// # Arguments
    /// * `username` - The username of the user
    /// * `master_password` - The master password of the user
    /// * `path` - The path to the user data
    ///
    /// # Returns
    /// A tuple of the integrity status and the read-only records if the integrity check passes
    fn check_integrity(
        &self,
        username: &str,
        master_password: &str,
        path: &PathBuf,
    ) -> (bool, Option<ReadOnlyRecords>) {
        let records = Record::read_user(path, username, master_password);

        match records {
            Ok(r) => {
                let mut read_only_records = vec![];
                for record in r.iter() {
                    let decrypted = record.cypher.decrypt_data();
                    match decrypted {
                        Ok(decrypted) => {
                            read_only_records.push((
                                decrypted.domain.to_string(),
                                decrypted.password.to_string(),
                            ));
                        }
                        Err(_) => return (false, None),
                    }
                }
                (true, Some(ReadOnlyRecords(read_only_records)))
            }
            Err(_) => (false, None),
        }
    }

    /// Removes all records from the file
    ///
    /// # Returns
    /// Nothing, panics if the records could not be removed (TODO: handle error)
    fn remove_records_from_file(&mut self) {
        let path = self.path();
        match clear_file_content(&path) {
            Ok(_) => {}
            Err(_) => panic!("Could not clear file content"),
        }
    }
}

impl ReadOnlyRecords {
    /// Returns the records
    ///
    /// # Returns
    /// A vector of domain-password pairs
    pub fn records(&self) -> Vec<(String, String)> {
        self.0.clone()
    }

    /// Adds a new record to the read-only records
    ///
    /// # Arguments
    /// * `domain` - The domain
    /// * `password` - The password
    fn add_record(&mut self, domain: &String, password: &String) {
        self.0.push((domain.clone(), password.clone()));
    }

    /// Removes a record from the read-only records
    /// The record is removed by domain
    ///
    /// # Arguments
    /// * `domain` - The domain
    fn remove_record(&mut self, domain: &String) {
        let mut new_records = vec![];
        for record in self.0.iter() {
            if record.0 != *domain {
                new_records.push(record.clone());
            }
        }

        self.0 = new_records;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;
    use std::{env, fs};

    fn random_number() -> u32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(10000000..99999999)
    }

    fn generate_random_username() -> String {
        format!("krab-{}", random_number())
    }

    fn setup_user_data(domain: &str) -> Result<RecordOperationConfig, String> {
        let username = generate_random_username();
        let username = username.as_str().to_owned();
        let master_password = "password";
        let password = "password";
        let path = PathBuf::from(env::var("KRAB_TEMP_DIR").unwrap());
        let user =
            RecordOperationConfig::new(username.as_str(), master_password, domain, password, &path);
        match User::new(&user) {
            Ok(_) => Ok(user.clone()),
            Err(e) => Err(e),
        }
    }

    fn create_user(config: &RecordOperationConfig) -> Result<(User, ReadOnlyRecords), String> {
        User::from(&config.path, &config.username, &config.master_password)
    }

    #[test]
    fn test_derive_key() {
        let data = "krab";
        let derived_key = DerivedKey::derive_key(data, None);
        let key = derived_key.key;
        let salt = derived_key.salt;
        assert_eq!(key.len(), 16);
        assert_eq!(salt.len(), 22);
    }

    #[test]
    fn test_marshalling() {
        let domain = "example.com";
        let password = "password";
        let data = CipherConfig::marshal(domain, password);
        let (d, p) = CipherConfig::unmarshal(data.as_str());
        assert_eq!(d, domain);
        assert_eq!(p, password);

        let domain = "example.com with spaces";
        let password = "password with spaces";
        let data = CipherConfig::marshal(domain, password);
        let (d, p) = CipherConfig::unmarshal(data.as_str());
        assert_eq!(d, domain);
        assert_eq!(p, password);

        let domain = "example.com with \\";
        let password = "password with \\";
        let data = CipherConfig::marshal(domain, password);
        let (d, p) = CipherConfig::unmarshal(data.as_str());
        assert_eq!(d, domain);
        assert_eq!(p, password);

        let domain = "example.com with \\ and    spacessss";
        let password = "password with \\ and    spacessss";
        let data = CipherConfig::marshal(domain, password);
        let (d, p) = CipherConfig::unmarshal(data.as_str());
        assert_eq!(d, domain);
        assert_eq!(p, password);
    }

    #[test]
    fn test_cipher_config() {
        let domain = "example.com";
        let password = "password";
        let data = CipherConfig::marshal(domain, password);
        let master_password = "password";
        let cipher = CipherConfig::encrypt_data(domain, password, master_password).unwrap();
        let decrypted = cipher.decrypt_data().unwrap();
        let decrypted = format!("{} {}", decrypted.domain, decrypted.password);
        assert_eq!(decrypted, data);

        let domain = "example.com with  spaces and \\";
        let password = "password with  spaces and \\";
        let data = CipherConfig::marshal(domain, password);
        let marshalled =
            "example.com\\swith\\s\\sspaces\\sand\\s\\\\ password\\swith\\s\\sspaces\\sand\\s\\\\"
                .to_string();
        assert_eq!(data, marshalled);
    }

    #[test]
    fn test_create_user_success() {
        let user_data = setup_user_data("example.com").unwrap();
        let user = create_user(&user_data);

        // delete the file (user)
        let hashed_username = hash(user_data.username.to_string());
        let file_path = user_data.path.join(hashed_username.as_str());
        fs::remove_file(file_path).unwrap();

        assert_eq!(user.is_ok(), true);
    }

    #[test]
    fn test_create_user_fail_already_exists() {
        // setup_user_data function not used here because we want to test
        // the case where the user already exists thus we need to try to create
        // a user with the same username twice (setup_user_data creates a new user each time
        // with a unique username)

        let username = generate_random_username();
        let username = username.as_str();
        let master_password = "password";
        let domain = "example.com";
        let password = "password";
        let path = PathBuf::from(env::var("KRAB_TEMP_DIR").unwrap());
        let config = RecordOperationConfig::new(username, master_password, domain, password, &path);
        let _ = User::new(&config);

        let config = RecordOperationConfig::new(username, master_password, domain, password, &path);
        let res = User::new(&config);

        // delete the file (user)
        let hashed_username = hash(username.to_string());
        let file_path = path.join(hashed_username.as_str());
        fs::remove_file(file_path).unwrap();

        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn test_integrity_success() {
        let user_data = setup_user_data("example.com").unwrap();
        let (user, _) = create_user(&user_data).unwrap();

        let (integrity, _) = user.check_integrity(
            &user_data.username,
            &user_data.master_password,
            &user_data.path,
        );

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(integrity, true);
    }

    #[test]
    fn test_integrity_fail() {
        let user_data = setup_user_data("example.com").unwrap();
        let (user, _) = create_user(&user_data).unwrap();

        let (integrity, _) =
            user.check_integrity(&user_data.username, "wrong_password", &user_data.path);

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(integrity, false);
    }

    #[test]
    fn test_read_record_success() {
        let user_data = setup_user_data("example.com").unwrap();
        let (user, records) = create_user(&user_data).unwrap();

        let records = records.records();
        let (domain, password) = records.first().unwrap();

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(domain, "example.com");
        assert_eq!(password, "password");

        let user_data = setup_user_data("example2. com").unwrap();
        let (user, records) = create_user(&user_data).unwrap();

        let records = records.records();
        let (domain, password) = records.first().unwrap();

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(domain, "example2. com");
        assert_eq!(password, "password");
    }

    #[test]
    #[should_panic]
    fn test_read_record_fail() {
        let user_data = setup_user_data("example.com").unwrap();
        let try_user = User::from(&user_data.path, &user_data.username, "wrong_password");

        // delete the file (user)
        let hashed_username = hash(user_data.username);
        let file_path = user_data.path.join(hashed_username.as_str());
        fs::remove_file(file_path).unwrap();

        // this should panic
        let _ = try_user.unwrap();
    }

    #[test]
    fn test_add_record_success() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, _) = create_user(&user_data).unwrap();

        let new_domain = "example2.com";
        let new_password = "password2";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            new_domain,
            new_password,
            &user_data.path,
        );
        let res = user.add_record(add_record);

        let (user, records) = User::from(
            &user_data.path,
            &user_data.username,
            &user_data.master_password,
        )
        .unwrap();

        let records = records.records();

        let inserted_record = records.iter().find(|r| r.0 == new_domain);

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(res.is_ok(), true);
        assert_eq!(inserted_record.is_some(), true);
        assert_eq!(records.len(), 2);
        assert_eq!(
            records.iter().find(|r| r.0 == "example.com").is_some(),
            true
        );
        assert_eq!(
            records.iter().find(|r| r.0 == "example2.com").is_some(),
            true
        );
    }

    #[test]
    fn test_add_record_fail() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, records) = create_user(&user_data).unwrap();

        let new_domain = "example2.com";
        let new_password = "password2";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            "wrong_password",
            new_domain,
            new_password,
            &user_data.path,
        );
        let res = user.add_record(add_record);

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(records.records().len(), 1);
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn test_add_record_fail_already_exists() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, records) = create_user(&user_data).unwrap();

        let new_domain = "example.com";
        let new_password = "password2";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            new_domain,
            new_password,
            &user_data.path,
        );
        let res = user.add_record(add_record);

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(records.records().len(), 1);
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn test_remove_record_success() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, _) = create_user(&user_data).unwrap();

        let new_domain = "example2.com";
        let new_password = "password2";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            new_domain,
            new_password,
            &user_data.path,
        );
        let _ = user.add_record(add_record);

        let new_domain = "example3.com";
        let new_password = "password3";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            new_domain,
            new_password,
            &user_data.path,
        );
        let _ = user.add_record(add_record);

        let remove_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            "example2.com",
            "",
            &user_data.path,
        );
        let res = user.remove_record(remove_record);

        let (user, records) = User::from(
            &user_data.path,
            &user_data.username,
            &user_data.master_password,
        )
        .unwrap();

        let records = records.records();
        let domains: Vec<String> = records.iter().map(|r| r.0.clone()).collect();

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(res.is_ok(), true);
        assert_eq!(records.len(), 2);
        assert_eq!(
            domains
                .iter()
                .find(|d| d.as_str() == "example2.com")
                .is_none(),
            true
        );
        assert_eq!(
            domains
                .iter()
                .find(|d| d.as_str() == "example3.com")
                .is_some(),
            true
        );
        assert_eq!(
            domains
                .iter()
                .find(|d| d.as_str() == "example.com")
                .is_some(),
            true
        );
    }

    #[test]
    fn test_remove_record_read_user_success() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, _) = create_user(&user_data).unwrap();

        let new_domain = "example2.com";
        let new_password = "password2";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            new_domain,
            new_password,
            &user_data.path,
        );
        let _ = user.add_record(add_record);

        let new_domain = "example3.com";
        let new_password = "password3";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            new_domain,
            new_password,
            &user_data.path,
        );
        let _ = user.add_record(add_record);

        let remove_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            "example2.com",
            "",
            &user_data.path,
        );
        let res = user.remove_record(remove_record);

        let (user, records) = User::from(
            &user_data.path,
            &user_data.username,
            &user_data.master_password,
        )
        .unwrap();
        let records = records.records();

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(res.is_ok(), true);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_remove_record_fail_not_found() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, _) = create_user(&user_data).unwrap();

        let new_domain = "example2.com";
        let new_password = "password2";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            new_domain,
            new_password,
            &user_data.path,
        );
        let _ = user.add_record(add_record);

        let remove_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            "example3.com",
            "",
            &user_data.path,
        );
        let res = user.remove_record(remove_record);

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn test_remove_record_fail_integrity_check() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, _) = create_user(&user_data).unwrap();

        let new_domain = "example2.com";
        let new_password = "password2";
        let add_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            new_domain,
            new_password,
            &user_data.path,
        );
        let _ = user.add_record(add_record);

        let remove_record = RecordOperationConfig::new(
            &user_data.username,
            "wrong_password",
            "example2.com",
            "",
            &user_data.path,
        );
        let res = user.remove_record(remove_record);

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(res.is_err(), true);
    }

    #[test]
    pub fn test_modify_record_success() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, _) = create_user(&user_data).unwrap();

        let new_password = "password2";
        let modify_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            &user_data.domain,
            new_password,
            &user_data.path,
        );
        let res = user.modify_record(modify_record);

        let (user, records) = User::from(
            &user_data.path,
            &user_data.username,
            &user_data.master_password,
        )
        .unwrap();
        let records = records.records();
        let modified_record = records.iter().find(|r| r.0 == user_data.domain);
        let modified_record = match modified_record {
            Some(record) => record,
            None => panic!("Record not found"),
        };
        let password = modified_record.1.clone();

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(res.is_ok(), true);
        assert_eq!(password, new_password);
        assert_eq!(records.len(), 1);
    }

    #[test]
    pub fn test_modify_integrity_fail() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, _) = create_user(&user_data).unwrap();

        let new_password = "password2";
        let modify_record = RecordOperationConfig::new(
            &user_data.username,
            "wrong_password",
            &user_data.domain,
            new_password,
            &user_data.path,
        );
        let res = user.modify_record(modify_record);

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(res.is_err(), true);
    }

    #[test]
    pub fn test_modify_record_fail_not_found() {
        let user_data = setup_user_data("example.com").unwrap();
        let (mut user, _) = create_user(&user_data).unwrap();

        let new_password = "password2";
        let modify_record = RecordOperationConfig::new(
            &user_data.username,
            &user_data.master_password,
            "example2.com",
            new_password,
            &user_data.path,
        );
        let res = user.modify_record(modify_record);

        // delete the file (user)
        fs::remove_file(user.path()).unwrap();

        assert_eq!(res.is_err(), true);
    }
}
