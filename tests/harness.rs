mod tests {
    use std::mem::size_of;

    use fuel_core_types::fuel_vm::SecretKey;
    use fuels::{
        accounts::wallet::WalletUnlocked,
        prelude::{abigen, setup_test_provider, AssetId, Contract},
        programs::contract::LoadConfiguration,
        test_helpers::{setup_custom_assets_coins, AssetConfig},
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

    fn create_wallet() -> WalletUnlocked {
        const SIZE_SECRET_KEY: usize = size_of::<SecretKey>();
        const PADDING_BYTES: usize = SIZE_SECRET_KEY - size_of::<u64>();
        let mut secret_key: [u8; SIZE_SECRET_KEY] = [0; SIZE_SECRET_KEY];
        secret_key[PADDING_BYTES..].copy_from_slice(&(8320147306839812359u64).to_be_bytes());

        let wallet = WalletUnlocked::new_from_private_key(
            SecretKey::try_from(secret_key.as_slice()).unwrap(),
            None,
        );
        wallet
    }

    async fn setup_env() -> (
        MyContract<WalletUnlocked>,
        MyContract<WalletUnlocked>,
        Proxy<WalletUnlocked>,
        WalletUnlocked,
    ) {
        let mut wallet = create_wallet();
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Generate coins for wallet
        let asset_configs = vec![AssetConfig {
            id: coin.1,
            num_coins: 1,
            coin_amount: coin.0,
        }];

        let all_coins = setup_custom_assets_coins(wallet.address(), &asset_configs[..]);

        let provider = setup_test_provider(all_coins.clone(), vec![], None, None)
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

    
}
