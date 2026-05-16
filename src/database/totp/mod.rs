use crate::database::totp::{ totp::TotpStoreOperations, code::TotpUsedCodeOperations };

pub mod totp;
pub mod code;

pub struct TotpOperations {
    pub store: TotpStoreOperations,
    pub code: TotpUsedCodeOperations,
}
