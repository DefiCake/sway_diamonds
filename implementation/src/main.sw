contract;

use std::constants::ZERO_B256;

#[namespace(implementation)]
storage {
    number: u64 = 0,
    bits: b256 = ZERO_B256,
    bits2: b256 = ZERO_B256,
}

abi ContractImplementation {
    fn double(value: u64) -> u64;

    #[storage(read,write)]
    fn set_number(value: u64);

    #[storage(read)]
    fn get_number() -> u64;

    #[storage(read,write)]
    fn set_bits(value: b256);

    #[storage(read)]
    fn get_bits() -> b256;

    #[storage(read,write)]
    fn set_bits2(value: b256);

    #[storage(read)]
    fn get_bits2() -> b256;

    fn test_function() -> b256;
    fn test_function_2() -> b256;
}

impl ContractImplementation for Contract {
    fn double(value: u64) -> u64 {
        value * 2
    }

    #[storage(read,write)]
    fn set_number(value: u64) {
        storage.number.write(value);
    }

    #[storage(read)]
    fn get_number() -> u64 {
        storage.number.read()
    }

    #[storage(read,write)]
    fn set_bits(value: b256) {
        storage.bits.write(value);
    }

    #[storage(read)]
    fn get_bits() -> b256 {
        storage.bits.read()
    }

    #[storage(read,write)]
    fn set_bits2(value: b256) {
        storage.bits2.write(value);
    }

    #[storage(read)]
    fn get_bits2() -> b256 {
        storage.bits2.read()
    }

    fn test_function() -> b256 {
        0x00000000000000000000000059F2f1fCfE2474fD5F0b9BA1E73ca90b143Eb8d0
    }

    fn test_function_2() -> b256 {
        0x0000000000000000000000001111111111111111111111111111111111111111
    }
}
