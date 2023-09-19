use crate::rpc::{async_trait, RpcParameter};
use encryptor::SequencerPoseidonEncryption;
use primitives::types::EncryptedTransaction;
use vdf::VDF;

#[async_trait]
impl RpcParameter for EncryptedTransaction {
    type Output = String;

    fn method_name() -> &'static str {
        "decrypt_transaction"
    }

    async fn handler(self) -> Self::Output {
        let base = 10; // Expression base (e.g. 10 == decimal / 16 == hex)
        let lambda = 2048; // N's bits (ex. RSA-2048 => lambda = 2048)

        let decrypt_function = SequencerPoseidonEncryption::new();
        let delay_function = VDF::new(lambda, base);

        let symmetric_key = self
            .decryption_key
            .unwrap_or_else(|| delay_function.evaluate(self.t, self.g.clone(), self.n.clone()));

        let symmetric_key =
            SequencerPoseidonEncryption::calculate_secret_key(symmetric_key.as_bytes());

        let decrypted_invoke_tx =
            decrypt_function.decrypt(self.encrypted_data.clone(), &symmetric_key, self.nonce);
        let decrypted_invoke_tx = String::from_utf8(decrypted_invoke_tx).unwrap();

        decrypted_invoke_tx.trim_end_matches('\0').to_string()
    }
}
