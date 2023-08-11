use ethers::types::H160;

pub fn convert_to_hex(addr: H160) -> String {
    format!("0x{}", hex::encode(addr))
}
