mod tests {
    use std::mem::size_of;

    use fuel_core_types::fuel_vm::SecretKey;
    use fuel_tx::{Address, GasCosts, GasCostsValues};
    use fuels::{
        accounts::{provider::Provider, wallet::WalletUnlocked, Account},
        prelude::{abigen, setup_test_provider, AssetId, Contract},
        programs::{call_response::FuelCallResponse, contract::LoadConfiguration},
        test_helpers::{setup_custom_assets_coins, AssetConfig, Config}, 
        types::{transaction::TxPolicies, tx_status::TxStatus, errors::Error as FuelError},
    };

    abigen!(
        Contract(
            name = "MyContract",
            abi = "implementation/out/debug/implementation-abi.json",
        ),
        Contract(name = "Proxy", abi = "proxy/out/debug/proxy-abi.json",)
    );

    const CONTRACT_BINARY: &str = "implementation/out/debug/implementation.bin";
    const PROXY_BINARY: &str = "proxy/out/debug/proxy.bin";

    pub const DEFAULT_COIN_AMOUNT: u64 = 1_000_000_000;

    async fn create_wallet(provider: Option<Provider>, fund_with_wallet: Option<WalletUnlocked>) -> WalletUnlocked {
        const SIZE_SECRET_KEY: usize = size_of::<SecretKey>();
        const PADDING_BYTES: usize = SIZE_SECRET_KEY - size_of::<u64>();
        let mut secret_key: [u8; SIZE_SECRET_KEY] = [0; SIZE_SECRET_KEY];
        secret_key[PADDING_BYTES..].copy_from_slice(&(8320147306839812359u64).to_be_bytes());

        let wallet = WalletUnlocked::new_random(provider);

        if let Some(funding_wallet) = fund_with_wallet {
            funding_wallet
                .transfer(wallet.address().into(), 100, Default::default(), Default::default())
                .await
                .unwrap();
        }
        
        wallet
    }

    async fn setup_env() -> (
        MyContract<WalletUnlocked>,
        MyContract<WalletUnlocked>,
        Proxy<WalletUnlocked>,
        WalletUnlocked,
    ) {
        let mut wallet = create_wallet(None, None).await;
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Generate coins for wallet
        let asset_configs = vec![AssetConfig {
            id: coin.1,
            num_coins: 1,
            coin_amount: coin.0,
        }];

        let all_coins = setup_custom_assets_coins(wallet.address(), &asset_configs[..]);
        let mut node_config = Config::default();
        node_config.chain_conf.consensus_parameters.gas_costs = GasCosts::new(GasCostsValues::free());

        let provider = setup_test_provider(all_coins.clone(), vec![], Some(node_config), None)
            .await
            .expect("Could not instantiate provider");

        wallet.set_provider(provider.clone());

        let implementation_configuration = LoadConfiguration::default();

        let implementation_contract_id =
            Contract::load_from(CONTRACT_BINARY, implementation_configuration)
                .unwrap()
                .deploy(&wallet.clone(), Default::default())
                .await
                .unwrap();

        let proxy_configuration = LoadConfiguration::default().with_configurables(
            ProxyConfigurables::default()
                .with_TARGET(implementation_contract_id.clone().into())
                .with_INITIAL_OWNER(Some(wallet.clone().address().into())),
        );

        let proxy_contract_id = Contract::load_from(PROXY_BINARY, proxy_configuration)
            .unwrap()
            .deploy(&wallet.clone(), Default::default())
            .await
            .unwrap();

        let implementation = MyContract::new(implementation_contract_id.clone(), wallet.clone());
        let proxy = MyContract::new(proxy_contract_id.clone(), wallet.clone());
        let proxy_admin = Proxy::new(proxy_contract_id, wallet.clone());
        (proxy, implementation, proxy_admin, wallet)
    }

    #[tokio::test]
    async fn test_function() {
        let (proxy, implementation, _, _) = setup_env().await;

        let value = 5u64;
        let result = implementation.methods().double(value).call().await.unwrap();
        assert_eq!(value * 2, result.value);

        let proxy_result = proxy
            .methods()
            .double(value)
            .with_contract_ids(&[implementation.contract_id().clone().into()])
            .call()
            .await
            .unwrap();
        assert_eq!(value * 2, proxy_result.value);
    }

    #[tokio::test]
    async fn test_initial_ownership() {
        let (_, _, proxy, wallet) = setup_env().await;


        let owner = proxy
            .methods()
            ._proxy_owner()
            .call()
            .await
            .unwrap();

        assert_eq!(owner.value, Some(wallet.address().into()));
    }

    #[tokio::test]
    async fn test_transfer_ownership() {
        let (_, _, proxy, wallet) = setup_env().await;

        let provider = wallet.provider().clone().unwrap().to_owned();
        let first_owner = create_wallet(Some(provider.clone()), Some(wallet.clone())).await;
        
        let call_result: FuelCallResponse<_> = proxy
            .with_account(wallet.clone())
            .unwrap()
            .methods()
            ._proxy_transfer_ownership(first_owner.address().into())
            .call()
            .await
            .unwrap();

        assert!(
            matches!(
                provider.tx_status(&call_result.tx_id.unwrap()).await.unwrap(), 
                TxStatus::Success { .. }
            )
        );

        let owner = proxy
            .methods()
            ._proxy_owner()
            .call()
            .await
            .unwrap();

        assert_eq!(owner.value, Some(first_owner.address().into()));

        // Transfer a second time
        let second_owner = create_wallet(Some(provider.clone()), Some(wallet.clone())).await;
        
        let call_result: FuelCallResponse<_> = proxy
            .with_account(first_owner.clone())
            .unwrap()
            .methods()
            ._proxy_transfer_ownership(second_owner.address().into())
            .with_tx_policies(TxPolicies::default().with_gas_price(0).with_max_fee(0))
            .call()
            .await
            .unwrap();

        assert!(
            matches!(
                &provider.tx_status(&call_result.tx_id.unwrap()).await.unwrap(), 
                TxStatus::Success { .. }
            )
        );

        let owner = proxy
            .methods()
            ._proxy_owner()
            .call()
            .await
            .unwrap();

        assert_eq!(owner.value, Some(second_owner.address().into()));
    }
    

    #[tokio::test]
    async fn test_transfer_ownership_auth() {
        let (_, _, proxy, wallet) = setup_env().await;

        let provider = wallet.provider().clone().unwrap().to_owned();
        let mallory = create_wallet(Some(provider.clone()), Some(wallet.clone())).await;
        
        let call_result: Result<FuelCallResponse<()>, FuelError> = proxy
            .with_account(mallory.clone())
            .unwrap()
            .methods()
            ._proxy_transfer_ownership(mallory.address().into())
            .call()
            .await;
            
        match call_result.unwrap_err() {
            FuelError::RevertTransactionError { reason, .. } => {
                assert_eq!(&reason, "Auth");
            },
            _ => panic!("Wrong transaction error"),
            
        };
    }
    

    #[tokio::test]
    async fn test_revoke_ownership() {
        let (_, _, proxy, wallet) = setup_env().await;

        let provider = wallet.provider().clone().unwrap().to_owned();
        
        let call_result: FuelCallResponse<_> = proxy
            .with_account(wallet.clone())
            .unwrap()
            .methods()
            ._proxy_revoke_ownership()
            .call()
            .await
            .unwrap();

        assert!(
            matches!(
                provider.tx_status(&call_result.tx_id.unwrap()).await.unwrap(), 
                TxStatus::Success { .. }
            )
        );

        let owner = proxy
            .methods()
            ._proxy_owner()
            .call()
            .await
            .unwrap();

        assert_eq!(owner.value, Some(fuels::types::Identity::Address(Address::zeroed())));
    }
    
    #[tokio::test]
    async fn test_revoke_ownership_auth() {
        let (_, _, proxy, wallet) = setup_env().await;

        let provider = wallet.provider().clone().unwrap().to_owned();
        let mallory = create_wallet(Some(provider.clone()), Some(wallet.clone())).await;
        
        let call_result: Result<FuelCallResponse<()>, FuelError> = proxy
            .with_account(mallory.clone())
            .unwrap()
            .methods()
            ._proxy_revoke_ownership()
            .call()
            .await;
            
        match call_result.unwrap_err() {
            FuelError::RevertTransactionError { reason, .. } => {
                assert_eq!(&reason, "Auth");
            },
            _ => panic!("Wrong transaction error"),
            
        };
    }
}
