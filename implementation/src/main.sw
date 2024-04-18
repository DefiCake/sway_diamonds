contract;

#[namespace(implementation)]
storage {
    number: u64 = 0
}

abi ContractImplementation {
    fn double(value: u64) -> u64;

    #[storage(read,write)]
    fn set_number(value: u64);

    #[storage(read)]
    fn get_number() -> u64;
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
}
