pub mod content_box;

use snow::params::NoiseParams;

lazy_static::lazy_static! {
    static ref PARAMS: NoiseParams = "Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s".parse().unwrap();
}
