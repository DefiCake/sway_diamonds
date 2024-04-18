```
forc --version
forc 0.54.0
```

```
fuel-core --version
fuel-core 0.23.0
```

To run, from the root:

`cd implementation && forc build && cd ../proxy && forc build && cd .. && cargo test`

# How it works

The proxy has its own storage under `#[namespace(diamonds)]`, which holds:

- The owner of the proxy (who can do upgrades)
- A map of function selectors to a corresponding implementation (or as it is called in the sway std lib: a target)

When a function is invoked in the proxy, two things can happen:

- The function is one of the `_proxy` administration functions, that allow to upgrade the contract or transfer ownership
- The fallback is invoked, which checks the function selector and locates which "facet" of the diamond holds the logic to run it, then uses `run_external` against that implementation like a `DELEGATECALL` does in the EVM

## A typical workflow

1. The proxy contract is deployed by an EOA and optionally calls `_proxy_transfer_ownership` to a multisig or some other entity.
2. An implementation is deployed with the actual business logic of the contract.
3. The owner of the proxy calls `_proxy_set_facet_for_selector(fn_selector, implementation_contract_id)` and points (one by one) the function selectors that the implementation has to the implementation contract ID
4. When an user calls the proxy (say, `set_number`), the proxy will invoke the `fallback` function and locate which function of the `implementation` the user wants to run, then run the logic that the implementation contains in the `proxy` context (i.e. a delegatecall).
5. Afterwards, the owners can set new business logic and new storage variables by deploying another implementation contract and calling `_proxy_set_facet_for_selector(new_function_selector, new_implementation_contract_id)`