contract;

use std::{
    call_frames::first_param,
    constants::ZERO_B256,
    execution::run_external,
    hash::Hash,
    hash::sha256,
};

configurable {
    TARGET: ContractId = ContractId::from(ZERO_B256),
    INITIAL_OWNER: Option<Identity> = None
}

// Inspired by EIP-2535: Diamonds, Multifacet proxy
#[namespace(diamonds)]
storage {
    facets: StorageMap<u64, ContractId> = StorageMap {},
    owner: Option<Identity> = None
}

abi Diamonds {
    #[storage(read, write)]
    fn set_facet_for_selector(method_selector: u64, facet: ContractId);

    #[storage(read)]
    fn _proxy_owner() -> Option<Identity>;
}

impl Diamonds for Contract {
    #[storage(read, write)]
    fn set_facet_for_selector(method_selector: u64, facet: ContractId) {
        storage.facets.insert(method_selector, facet);
    }

    #[storage(read)]
    fn _proxy_owner() -> Option<Identity> {
        match storage.owner.read() {
            Some(value) => Some(value),
            None => INITIAL_OWNER,
        }
    }
}

#[fallback, storage(read)]
fn fallback() {
    let method_selector = first_param();

    let _ = storage.facets.get(method_selector).try_read();

    run_external(TARGET)
}
