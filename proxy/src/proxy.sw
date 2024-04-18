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

    #[storage(read,write)]
    fn _proxy_transfer_ownership(new_owner: Identity);

    #[storage(read,write)]
    fn _proxy_revoke_ownership();
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

    #[storage(read,write)]
    fn _proxy_transfer_ownership(new_owner: Identity) {
        let current_owner: Identity = 
            match storage.owner.read() {
                Some(value) => Some(value),
                None => INITIAL_OWNER,
            }.unwrap();

        let sender = msg_sender().unwrap();

        require(sender == current_owner, DiamondsProxyError::Auth);

        storage.owner.write(Some(new_owner));
    }

    #[storage(read,write)]
    fn _proxy_revoke_ownership() {
        let current_owner: Identity = 
            match storage.owner.read() {
                Some(value) => Some(value),
                None => INITIAL_OWNER,
            }.unwrap();

        let sender = msg_sender().unwrap();

        require(sender == current_owner, DiamondsProxyError::Auth);

        storage.owner.write(Some(Identity::Address(Address::from(ZERO_B256))));
    }
}

#[fallback, storage(read)]
fn fallback() {
    let method_selector = first_param();

    let _ = storage.facets.get(method_selector).try_read();

    run_external(TARGET)
}


pub enum DiamondsProxyError {
    Auth: (),
    Auth2: (),
}