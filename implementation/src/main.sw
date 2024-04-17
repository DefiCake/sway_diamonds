contract;

abi ContractImplementation {
    fn double(value: u64) -> u64;
}

impl ContractImplementation for Contract {
    fn double(value: u64) -> u64 {
        value * 2
    }
}
