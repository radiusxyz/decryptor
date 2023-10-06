pub mod rpc;

fn decrypt(encrypted_transaction: EncryptedTransaction) {
    let base = 10; // Expression base (e.g. 10 == decimal / 16 == hex)
    let lambda = 2048; // N's bits (ex. RSA-2048 => lambda = 2048)

    let decrypt_function = SequencerPoseidonEncryption::new();
    let delay_function = VDF::new(lambda, base);

    let symmetric_key = encrypted_transaction.decryption_key.unwrap_or_else(|| {
        delay_function.evaluate(
            encrypted_transaction.t as u64,
            encrypted_transaction.g.clone(),
            encrypted_transaction.n.clone(),
        )
    });

    let symmetric_key = SequencerPoseidonEncryption::calculate_secret_key(symmetric_key.as_bytes());

    let decrypted_invoke_tx = decrypt_function.decrypt(
        encrypted_transaction.encrypted_data.clone(),
        &symmetric_key,
        encrypted_transaction.nonce,
    );
    let decrypted_invoke_tx = String::from_utf8(decrypted_invoke_tx).unwrap();

    decrypted_invoke_tx.trim_end_matches('\0').to_string()
}
