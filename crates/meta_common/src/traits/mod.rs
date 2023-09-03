use ethers::prelude::*;

pub trait ContractCode {
    fn get_byte_code(&self) -> Option<&String>;
    fn get_code_hash(&self) -> Option<&String>;

    fn get_byte_code_and_hash(&self) -> (Bytes, [u8; 32]) {
        match self.get_byte_code() {
            Some(ref byte_code) => {
                let bytes: Bytes = byte_code.parse().unwrap();
                let code_hash: [u8; 32] =
                    self.get_code_hash().as_ref().map_or([0; 32], |code_hash_str| {
                        hex::decode(code_hash_str).map_or([0; 32], |x| {
                            let mut fixed_length_array: [u8; 32] = [0; 32];
                            fixed_length_array.copy_from_slice(&x);
                            fixed_length_array
                        })
                    });
                (bytes, code_hash)
            }
            None => (vec![].into(), [0; 32]),
        }
    }
}
