use x25519_dalek::PublicKey;

pub struct CryptoContext {
    opposite_key: PublicKey,
    self_key: PublicKey,
}

impl CryptoContext {
    pub fn new(opposite_key: PublicKey, self_key: PublicKey) -> Self {
        Self {
            opposite_key,
            self_key,
        }
    }
}
