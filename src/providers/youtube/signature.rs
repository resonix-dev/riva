#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("signature deciphering is not yet implemented")]
    Unavailable,
}

#[derive(Debug, Default)]
pub struct SignatureDecipher;

impl SignatureDecipher {
    pub fn decipher(
        &self,
        _player_js_url: &str,
        _scrambled: &str,
    ) -> Result<String, SignatureError> {
        Err(SignatureError::Unavailable)
    }
}

pub struct SignatureResolver<'a> {
    pub js_url: &'a str,
    pub decipher: &'a SignatureDecipher,
}

impl<'a> SignatureResolver<'a> {
    pub fn apply(&self, scrambled: &str) -> Result<String, SignatureError> {
        self.decipher.decipher(self.js_url, scrambled)
    }
}
