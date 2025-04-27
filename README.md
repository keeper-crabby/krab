# Krab 🦀 - A Simple Terminal Password Manager

[![Build Status](https://github.com/keeper-crabby/krab/actions/workflows/rust.yml/badge.svg)](https://github.com/keeper-crabby/krab/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<!-- Optional: Add crates.io badge if published -->
<!-- [![Crates.io](https://img.shields.io/crates/v/krab.svg)](https://crates.io/crates/krab) -->

Krab is a lightweight, secure, and easy-to-use password manager designed to run entirely within your terminal. Built with Rust, it prioritizes security and simplicity for managing your sensitive credentials without leaving the command line.

<!-- ## Screenshot -->
<!-- (Leave this space for your screenshot) -->
<!-- Example: -->
<!-- ``` -->
<!-- [Screenshot of Krab's main interface] -->
<!-- ``` -->
<!-- Or use an image tag: -->
<!-- ![Krab Screenshot](path/to/your/screenshot.png) -->

## ✨ Features

*   **Secure Encryption:** Your password database is encrypted using strong, modern cryptography.
*   **Intuitive TUI:** A clean and navigable Terminal User Interface built with [`ratatui`](https://ratatui.rs/).
*   **Password Generation:** Generate strong, random passwords.
*   **Fuzzy filtering:** Quickly find the credentials you need.
*   **Cross-Platform:** Runs on Linux, macOS, and Windows thanks to Rust and `crossterm`.
*   **Single File Database:** Your entire encrypted vault is stored in a single file.
*   **Master Password Protection:** Access is controlled by a single, strong master password.

## 🔐 Security

Security is paramount for a password manager. Krab employs the following:

*   **Encryption Algorithm:** The password database file is encrypted using **AES-256-GCM**. AES-GCM is an Authenticated Encryption with Associated Data (AEAD) scheme, which provides both confidentiality (data is secret) and integrity (data cannot be tampered with undetected).
*   **Key Derivation:** The encryption key used for the database is derived from your master password using **scrypt**. `scrypt` is a password-based key derivation function (KDF) specifically designed to be computationally and memory-intensive, making large-scale, custom hardware attacks (like those using GPUs or ASICs) significantly more costly and difficult compared to older KDFs.
*   **Master Password:** Your master password is **never** stored directly. It is only used temporarily in memory during runtime to derive the encryption key via `scrypt`. **Choose a strong, unique master password!**
*   **Dependencies:** Cryptographic operations rely on established Rust crates (`aes-gcm`, `scrypt`).

**Disclaimer:** While care has been taken to use secure practices, this software has not undergone a formal security audit. Use at your own risk. Always ensure you have backups of your encrypted database file.

## 🚀 Installation

### From Source (Recommended)

Ensure you have the Rust toolchain installed (`rustup`).

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/keeper-crabby/krab.git
    cd krab
    ```
2.  **Build the release binary:**
    ```bash
    cargo build --release
    ```
3.  **Run Krab:**
    The executable will be located at `target/release/krab`. You can copy this binary to a location in your system's `PATH` (e.g., `~/.local/bin` or `/usr/local/bin`) for easier access.
    ```bash
    ./target/release/krab
    # Or, if you moved it to your PATH:
    krab
    ```

<!-- ### From Crates.io (Coming Soon?) -->
<!-- Once published, you'll be able to install Krab directly using cargo: -->
<!-- ```bash -->
<!-- cargo install krab -->
<!-- ``` -->

<!-- ### Pre-built Binaries (Optional) -->
<!-- Check the [Releases](https://github.com/keeper-crabby/krab/releases) page for pre-compiled binaries for your platform (if available). -->

## 💻 Usage

1.  **Start Krab:**
    ```bash
    krab
    ```

2.  **Initial Screen (Login or Register):**
    *   Upon starting, Krab will present options to either **Login** or **Register** a new user. Use your arrow keys or specified keys to select an option and press `Enter`.

3.  **Registering a New User:**
    *   If you choose **Register**, you will be prompted for the following:
        *   **Username:** Choose a unique username for your account.
        *   **Master Password:** Create a strong, unique master password. **Remember this password!** It's the key to your encrypted secrets.
        *   **Confirm Master Password:** Re-enter the master password to ensure accuracy.
        *   **Initial Secret:** You must add at least one secret to create the vault:
            *   **Domain/Service:** Enter the name of the service or website (e.g., `github.com`, `My Email`).
            *   **Password:** Enter the corresponding password for that service.
    *   Upon successful registration, Krab will create an encrypted database file specifically for this user, typically located in your configuration directory (see Data Storage section).

4.  **Logging In:**
    *   If you choose **Login**, you will be prompted for:
        *   **Username:** Enter the username you registered with.
        *   **Master Password:** Enter the master password associated with that username.
    *   Krab will attempt to locate the user's database file, decrypt it using the provided master password, and load your secrets.

5.  **Home View (After Login):**
    *   Once logged in, you'll see your stored secrets listed in rows.
    *   **Navigation & Actions:** Use the following keys to interact with your secrets:
        *   `j` or `Down Arrow`: Move selection down.
        *   `k` or `Up Arrow`: Move selection up.
        *   `h` or `Left Arrow`: Move left if scrollable
        *   `l` or `Right Arrow`: Move right if scrollable
        *   `q`: Quit Krab.
        *   `a`: Add a new secret entry.
        *   `d`: Delete the currently selected secret.
        *   `e`: Edit the currently selected secret.
        *   `c`: Copy the password of the selected secret to the clipboard.
        *   `f`: Enter filtering mode. Type to **fuzzy find** secrets based on the domain/service name. Press `Esc` to return to **normal** mode.
        *   `Enter`: Toggle the visibility of the selected secret's password (show/hide).

## 💾 Data Storage

*   Your encrypted password database is stored as a single file.
*   Krab uses [directories](https://crates.io/crates/directories) library to store the data across Windows, macOS and Linux.
The library provides multiple options for choosing the directory which is needed.
Krab uses the **ProjectDirs** directory provided by the library. **Directories** then handles the cross platform convrsion.
For more infomration check out the library page but for short:
    *   **Linux:** `$XDG_CONFIG_HOME/krab` or `~/.config/krab`
    *   **macOS:** `~/Library/Application Support/krab`
    *   **Windows:** `%APPDATA%\krab\config`
    *   *(Verify this path based on the `directories` crate usage)*
*   **Backup:** It is crucial to **back up this encrypted database file** regularly to a secure location (e.g., an encrypted USB drive, secure cloud storage). Losing this file means losing your passwords.

## 💾 Data Storage

Krab securely stores each user's encrypted secrets in a dedicated database file. The location of the directory containing these files follows standard conventions for each operating system, determined using the [`directories`](https://crates.io/crates/directories) crate.

*   **Mechanism:** Krab uses `directories::ProjectDirs::from("", "", "krab")` to identify the appropriate project-specific directory. Using empty strings for the `qualifier` and `organization` simplifies the generated path structure.

*   **Target Directory Method:** Krab retrieves the base storage location using the `data_dir()` method from the generated `ProjectDirs` object. This method typically points to user-specific application data locations (often synced across devices on Windows/macOS, unlike local data).

*   **Resulting Base Directory Examples:** Based on `ProjectDirs::from("", "", "krab")` and the use of `data_dir()`, the standard base directories are:
    *   **Linux:** `$XDG_DATA_HOME/krab` or `$HOME/.local/share/krab` (The specific base depends on the `XDG_DATA_HOME` environment variable).
    *   **Windows:** `{FOLDERID_RoamingAppData}\krab\data` (e.g., `C:\Users\YourUser\AppData\Roaming\krab\data`). Note the `\data` suffix added by the `data_dir()` method on Windows.
    *   **macOS:** `$HOME/Library/Application Support/krab` (e.g., `/Users/YourUser/Library/Application Support/krab`).

*   **User Database Files:** Within this determined base directory (`data_dir()`), Krab stores a separate encrypted file for each registered user.
    *   The filename convention is `sha256([username])` 

*   **Crucial Backup:** It is **absolutely essential** to regularly **back up these individual user database files** located within the application's data directory identified above. Store backups securely (e.g., encrypted external drive, secure cloud storage). **Losing these files means losing all the passwords stored under that specific username.**

## 🛠️ Technology Stack

*   **Language:** [Rust](https://www.rust-lang.org/)
*   **Terminal UI (TUI):** [`ratatui`](https://ratatui.rs/) (using the [`crossterm`](https://github.com/crossterm-rs/crossterm) backend)
*   **Encryption:** [`aes-gcm`](https://crates.io/crates/aes-gcm) crate
*   **Key Derivation:** [`scrypt`](https://crates.io/crates/scrypt) crate
*   **Directory Paths:** [`directories`](https://crates.io/crates/directories)

## ✅ Testing

Krab includes a suite of tests to ensure its core functionality works as expected. To run these tests correctly, you need to specify a temporary directory where test data (like temporary user database files) can be created.

1.  **Set Environment Variable:** Before running the tests, you **must** set the `KRAB_TEMP_DIR` environment variable to point to a directory where test files can be temporarily stored. This directory should ideally be empty or designated specifically for these test artifacts.

    *   **Linux/macOS (bash/zsh):**
        ```bash
        export KRAB_TEMP_DIR="/tmp/krab_test_data"
        ```

    *   **Windows (Command Prompt):**
        ```cmd
        set KRAB_TEMP_DIR="C:\temp\krab_test_data"
        mkdir %KRAB_TEMP_DIR%
        ```

    *   **Windows (PowerShell):**
        ```powershell
        $env:KRAB_TEMP_DIR = "C:\temp\krab_test_data"
        New-Item -ItemType Directory -Force -Path $env:KRAB_TEMP_DIR
        ```

2.  **Run Tests:** Once the environment variable is set, navigate to the project's root directory in your terminal and run the standard Rust test command:
    ```bash
    cargo test
    ```

3.  **Cleanup (Important):** The tests aim to clean up after themselves, but under certain conditions (like test failures or interruptions), some temporary user files might remain in the directory specified by `KRAB_TEMP_DIR`. **After running tests, it's recommended to check this directory and manually delete any leftover files** to ensure a clean state for subsequent test runs or to free up space. You can simply delete the entire directory if you created it solely for testing.

## 🤝 Contributing

Contributions are welcome! Whether it's bug reports, feature requests, or code contributions:

1.  **Check Issues:** Look for existing issues or open a new one to discuss your idea or bug report.
2.  **Fork the Repository:** Create your own fork of `krab`.
3.  **Create a Branch:** `git checkout -b feature/your-new-feature` or `bugfix/issue-number`.
4.  **Write Code:** Implement your changes or fix the bug.
5.  **Add Tests:** Ensure your changes are covered by tests.
6.  **Commit Changes:** Write clear and concise commit messages.
7.  **Push to Your Fork:** `git push origin feature/your-new-feature`.
8. **Open a Pull Request:** Submit a PR against the `main` branch of the original `keeper-crabby/krab` repository.

Please adhere to the project's code style and provide clear descriptions for your changes.

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.