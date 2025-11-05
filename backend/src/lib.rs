use directories::{BaseDirs, ProjectDirs};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    str,
};

mod models;
pub mod user;

const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()-_=+[]{}|;:,.<>?";

const DEFAULT_LENGTH: usize = 16;

const DB_DIR: &str = "krab";
const RELEASE_SUFFIX: &str = "release";
const CONFIG_FILE: &str = "config.json";

/// Configuration for password generation options
///
/// # Fields
/// * `include_uppercase` - Does the password include uppercase characters
/// * `include_numbers` - Does the password include numbers
/// * `include_special` - Does the password include special characters
/// * `length` - Length of the password
///
/// # Implements
/// * `Default`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasswordConfig {
    pub include_uppercase: bool,
    pub include_numbers: bool,
    pub include_special: bool,
    pub length: usize,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            include_uppercase: true,
            include_numbers: true,
            include_special: true,
            length: DEFAULT_LENGTH,
        }
    }
}

/// Configuration for the application
///
/// # Fields
/// * `password_config` - Configuration for password generation options
///
/// # Methods
/// * `get_config_path` - Gets the path to the configuration file
/// * `load` - Loads password configuration from file, or returns default if file doesn't exist
/// * `save` - Saves password configuration to file
///
/// # Implements
/// * `Default`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub password_config: PasswordConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            password_config: PasswordConfig::default(),
        }
    }
}

impl Config {
    /// Gets the path to the configuration file
    fn get_config_path() -> Result<PathBuf, io::Error> {
        if let Some(base_dirs) = BaseDirs::new() {
            let config_dir = base_dirs.config_dir().join(DB_DIR);
            if !config_dir.exists() {
                fs::create_dir_all(&config_dir)?;
            }
            Ok(config_dir.join(CONFIG_FILE))
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not get config directory",
            ))
        }
    }

    /// Loads password configuration from file, or returns default if file doesn't exist
    pub fn load() -> Result<Self, io::Error> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // Create default config file
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        match serde_json::from_str(&contents) {
            Ok(config) => Ok(config),
            Err(_) => {
                // If config file is corrupted, return default and save it
                let default_config = Self::default();
                default_config.save()?;
                Ok(default_config)
            }
        }
    }

    /// Saves password configuration to file
    pub fn save(&self) -> Result<(), io::Error> {
        let config_path = Self::get_config_path()?;
        let contents = serde_json::to_string_pretty(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e),
            )
        })?;

        let mut file = File::create(config_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}

/// Initializes the project directories and returns the path to the data directory
/// If the environment variable KRAB_DIR is set, the data directory will be created
/// in the specified directory. Otherwise, the data directory will be created in the
/// default directory.
///
/// # Returns
/// A `Result` containing the path to the data directory if successful, otherwise an
/// `io::Error` is returned.
pub fn init() -> Result<PathBuf, io::Error> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", DB_DIR) {
        let sub_dir = env::var("KRAB_DIR").unwrap_or(RELEASE_SUFFIX.to_string());
        let proj_dirs = proj_dirs.data_dir().join(sub_dir);
        if !proj_dirs.is_dir() {
            let res = create_if_not_exists(&proj_dirs);
            assert!(res.is_ok());
        }
        Ok(proj_dirs.to_path_buf())
    } else {
        panic!("Could not get project directories");
    }
}

/// Checks if a user exists in the database
///
/// # Arguments
/// * `username` - The username of the user
/// * `path` - The path to the data directory
///
/// # Returns
///
/// `true` if the user exists, otherwise `false`
pub fn check_user(username: &str, path: PathBuf) -> bool {
    let hashed_username = hash(username.to_string());
    match path.join(hashed_username).exists() {
        true => true,
        false => false,
    }
}

/// Creates a hash of the input data
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// The hashed data as a string
pub fn hash(data: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Helper function to check if a password contains uppercase characters
///
/// # Arguments
/// * `password` - The password to check
///
/// # Returns
/// `true` if the password contains uppercase characters, otherwise `false`
fn contains_uppercase(password: &str) -> bool {
    password.chars().any(|c| UPPERCASE.contains(c))
}

/// Helper function to check if a password contains numbers
///
/// # Arguments
/// * `password` - The password to check
///
/// # Returns
/// `true` if the password contains numbers, otherwise `false`
fn contains_numbers(password: &str) -> bool {
    password.chars().any(|c| NUMBERS.contains(c))
}

/// Helper function to check if a password contains special characters
///
/// # Arguments
/// * `password` - The password to check
///
/// # Returns
/// `true` if the password contains special characters, otherwise `false`
fn contains_special(password: &str) -> bool {
    password.chars().any(|c| SPECIAL.contains(c))
}

/// Helper function to validate if a password meets the configuration requirements
///
/// # Arguments
/// * `password` - The password to validate
/// * `config` - The password configuration
///
/// # Returns
/// `true` if the password meets the requirements, otherwise `false`
fn validate_password(password: &str, config: &PasswordConfig) -> bool {
    // Always contains lowercase (always included in charset)
    let has_lowercase = password.chars().any(|c| LOWERCASE.contains(c));

    let has_uppercase = if config.include_uppercase {
        contains_uppercase(password)
    } else {
        true // Not required, so consider it valid
    };

    let has_numbers = if config.include_numbers {
        contains_numbers(password)
    } else {
        true // Not required, so consider it valid
    };

    let has_special = if config.include_special {
        contains_special(password)
    } else {
        true // Not required, so consider it valid
    };

    has_lowercase && has_uppercase && has_numbers && has_special
}

/// Generates a random password with the specified configuration
///
/// # Returns
///
/// A randomly generated password as a string that meets all the configuration requirements
pub fn generate_password() -> String {
    let config = Config::load().unwrap_or_default();
    let config = config.password_config;
    let mut rng = rand::thread_rng();

    // Build charset based on configuration
    let mut charset = LOWERCASE.to_string();
    if config.include_uppercase {
        charset.push_str(UPPERCASE);
    }
    if config.include_numbers {
        charset.push_str(NUMBERS);
    }
    if config.include_special {
        charset.push_str(SPECIAL);
    }

    let charset: Vec<char> = charset.chars().collect();

    // Generate password until it meets all requirements
    loop {
        let mut password = String::new();
        for _ in 0..config.length {
            let idx = rng.gen_range(0..charset.len());
            password.push(charset[idx]);
        }

        // Validate the generated password
        if validate_password(&password, &config) {
            return password;
        }
        // If validation fails, loop continues to generate a new password
    }
}

/// Creates a new file in the specified directory
///
/// # Arguments
/// * `p` - The path to the directory
/// * `file_name` - The name of the file to create
///
/// # Returns
/// The path to the newly created file if successful, otherwise an `io::Error` is returned
pub fn create_file(p: &PathBuf, file_name: &str) -> io::Result<PathBuf> {
    let file_path = p.join(file_name);
    if !file_path.exists() {
        File::create(file_path.as_path())?;
        return Ok(file_path);
    } else {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "File already exists",
        ));
    }
}

/// Clears the content of a file
///
/// # Arguments
/// * `p` - The path to the file
///
/// # Returns
/// An `io::Result` indicating success or failure
pub fn clear_file_content(p: &PathBuf) -> io::Result<()> {
    if !p.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "File does not exist",
        ));
    }
    File::create(p)?;
    Ok(())
}

/// Writes data to a file
///
/// # Arguments
/// * `p` - The path to the file
/// * `data` - The data to write to the file
///
/// # Returns
/// An `io::Result` indicating success or failure
pub fn write_to_file(p: &PathBuf, data: Vec<u8>) -> io::Result<()> {
    if !p.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "File does not exist",
        ));
    }
    let mut f = File::create(p)?;
    f.write_all(&data)?;
    Ok(())
}

/// Appends data to a file
///
/// # Arguments
/// * `p` - The path to the file
/// * `data` - The data to append to the file
///
/// # Returns
/// An `io::Result` indicating success or failure
pub fn append_to_file(p: &PathBuf, data: Vec<u8>) -> io::Result<()> {
    if !p.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "File does not exist",
        ));
    }
    let mut f = OpenOptions::new().append(true).open(p)?;
    f.write_all(&data)?;
    Ok(())
}

/// Creates a parent directory if it does not exist
///
/// # Arguments
/// * `p` - The path to the directory
///
/// # Returns
/// An `io::Result` indicating success or failure
fn create_parent_dir(p: &Path) -> io::Result<()> {
    match p.parent() {
        Some(parent) => {
            fs::create_dir_all(parent)?;
        }
        None => {}
    }
    Ok(())
}

/// Creates a directory if it does not exist
///
/// # Arguments
///
/// * `p` - The path to the directory
///
/// # Returns
///
/// An `io::Result` indicating success or failure
fn create_if_not_exists(p: &Path) -> io::Result<()> {
    if !p.exists() {
        create_parent_dir(p)?;
        fs::create_dir(p)?;
    }
    Ok(())
}
