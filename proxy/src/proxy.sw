contract;

use std::{
    execution::run_external,
    constants::ZERO_B256,
    call_frames::first_param,
    hash::Hash,
    hash::sha256,
};


configurable {
    TARGET: ContractId = ContractId::from(ZERO_B256)
}

// Inspired by EIP-2535: Diamonds, Multifacet proxy
#[namespace(diamonds)]
storage {
    facets: StorageMap<u64, ContractId> = StorageMap {},
}

abi Diamonds {
    #[storage(read,write)]
    fn set_facet_for_selector(method_selector: u64, facet: ContractId);
}

impl Diamonds for Contract {
    #[storage(read,write)]
    fn set_facet_for_selector(method_selector: u64, facet: ContractId) {
        storage.facets.insert(method_selector, facet);
    }
}

#[fallback, storage(read)]
fn fallback() {
    let method_selector = first_param();

    let _ = storage.facets.get(method_selector).try_read();

    run_external(TARGET)
}