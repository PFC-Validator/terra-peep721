//use cosmwasm_crypto::CryptoError;
use cosmwasm_std::{StdError, VerificationError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Funds Missing or insufficient")]
    Funds {},
    #[error("Signature doesn't match")]
    BADSIG {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},
    // #[error(transparent)]
    //CryptoError(#[from] cosmwasm_crypto::CryptoError),
    //#[error(transparent)]
    //K256Error(#[from] k256::ecdsa::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json_wasm::de::Error),
    #[error(transparent)]
    CryptoVerify(#[from] VerificationError),
    #[error("Token ID Can't be set?")]
    BadTokenID {},
    #[error("Invalid Secp256k1 Pubkey Format")]
    InvalidSecp256k1PubkeyFormat {},
    // #[error("Crypto {0}")]
    // Crypto(CryptoError),
    #[error("Invalid Secp256k1 Hash Format")]
    InvalidSecp256k1HashFormat {},
    #[error("Invalid Secp256k1 Signature Format")]
    InvalidSecp256k1SignatureFormat {},
}
