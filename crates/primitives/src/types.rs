use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct EncryptedTransaction {
    /// Encrypted transaction data.
    pub encrypted_data: Vec<String>,

    /// Nonce for decrypting the encrypted transaction.
    pub nonce: String,

    /// t for calculating time-lock puzzle.
    pub t: u64,

    /// g for calculating time-lock puzzle.
    pub g: String,

    /// n for calculating time-lock puzzle.
    pub n: String,

    /// decryption key for decrypting a encrypted transaction without calculating time-lock puzzle.
    pub decryption_key: Option<String>,
}
