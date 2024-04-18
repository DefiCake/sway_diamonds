contract;

use std::{
    call_frames::first_param,
    constants::ZERO_B256,
    execution::run_external,
    hash::Hash,
    hash::sha256,
};

configurable {
    INITIAL_OWNER: Option<Identity> = None,
}

// Inspired by EIP-2535: Diamonds, Multifacet proxy
#[namespace(diamonds)]
storage {
    facets: StorageMap<u64, ContractId> = StorageMap {},
    owner: Option<Identity> = None,
}

abi Diamonds {
    #[storage(read, write)]
    fn _proxy_set_facet_for_selector(method_selector: u64, facet: ContractId);

    #[storage(read, write)]
    fn _proxy_remove_selector(method_selector: u64);

    #[storage(read)]
    fn _proxy_owner() -> Option<Identity>;

    #[storage(read, write)]
    fn _proxy_transfer_ownership(new_owner: Identity);

    #[storage(read, write)]
    fn _proxy_revoke_ownership();
}

impl Diamonds for Contract {
    #[storage(read)]
    fn _proxy_owner() -> Option<Identity> {
        match storage.owner.read() {
            Some(value) => Some(value),
            None => INITIAL_OWNER,
        }
    }

    #[storage(read, write)]
    fn _proxy_set_facet_for_selector(method_selector: u64, facet: ContractId) {
        _proxy_check_ownership();

        storage.facets.insert(method_selector, facet);
    }

    #[storage(read, write)]
    fn _proxy_remove_selector(method_selector: u64) {
        _proxy_check_ownership();

        storage.facets.remove(method_selector);
    }

    #[storage(read, write)]
    fn _proxy_transfer_ownership(new_owner: Identity) {
        _proxy_check_ownership();

        storage.owner.write(Some(new_owner));
    }

    #[storage(read, write)]
    fn _proxy_revoke_ownership() {
        _proxy_check_ownership();

        storage
            .owner
            .write(Some(Identity::Address(Address::from(ZERO_B256))));
    }
}

#[fallback, storage(read)]
fn fallback() {
    let method_selector = first_param();

    match storage.facets.get(method_selector).try_read() {
        None => revert(0), // Cannot use require (log): https://github.com/FuelLabs/sway/issues/5850
        Some(facet) => run_external(facet),
    };
}

#[storage(read)]
fn _proxy_check_ownership() {
    let current_owner: Identity = match storage.owner.read() {
        Some(value) => Some(value),
        None => INITIAL_OWNER,
    }.unwrap();

    let sender = msg_sender().unwrap();

    require(sender == current_owner, DiamondsProxyError::Auth);
}

pub enum DiamondsProxyError {
    Auth: (),
}
