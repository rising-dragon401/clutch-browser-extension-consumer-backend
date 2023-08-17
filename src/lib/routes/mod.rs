use ethers::{signers::{LocalWallet, Signer}, types::{H256, U256}};

pub mod account_api;
pub mod account_guardians_api;
pub mod api;
pub mod guardian_api;
pub mod guardian_settings_api;
pub mod nomination_api;
pub mod verification_api;
pub mod transaction_api;


async fn sign_message(msg: Vec<u8>, wallet: LocalWallet) -> anyhow::Result<Vec<u8>> {
    let signature = wallet.sign_message(msg).await?;
    let mut signature_for_eth_sign = [
        H256(U256::from(signature.r).try_into().unwrap()).to_fixed_bytes(),
        H256(U256::from(signature.s).try_into().unwrap()).to_fixed_bytes(),
    ]
    .concat();
    signature_for_eth_sign.extend_from_slice(&[(signature.v as u8)]);
    Ok(signature_for_eth_sign)
}