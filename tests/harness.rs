mod tests {
    use std::mem::size_of;

    use fuel_core_types::fuel_vm::SecretKey;
    use fuels::{
        accounts::wallet::WalletUnlocked,
        prelude::{abigen, setup_test_provider, AssetId, Contract},
        test_helpers::{setup_custom_assets_coins, AssetConfig},
        programs::contract::LoadConfiguration,
    };

    abigen!(Contract(
        name = "MyContract",
        abi = "out/debug/my-fuel-project-abi.json",
    ));

    const CONTRACT_BINARY: &str = "out/debug/my-fuel-project.bin";
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

    #[tokio::test]
    async fn test_function() {
        let mut wallet = create_wallet();
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Generate coins for wallet
        let asset_configs = vec![AssetConfig {
                id: coin.1,
                num_coins: 1,
                coin_amount: coin.0,
        }];

        let all_coins = setup_custom_assets_coins(wallet.address(), &asset_configs[..]);
  
        let provider = setup_test_provider(
            all_coins.clone(),
            vec![],
            None,
            None,
        )
        .await
        .expect("Could not instantiate provider");

        wallet.set_provider(provider.clone());

        let load_configuration = LoadConfiguration::default();

        let test_contract_id =
            Contract::load_from(CONTRACT_BINARY, load_configuration)
                .unwrap()
                .deploy(&wallet.clone(), Default::default())
                .await
                .unwrap();


        let contract = MyContract::new(test_contract_id.clone(), wallet.clone());

        let value = 0u64;
        let result = contract
            .methods()
            .match_with_constants(value)
            .call()
            .await
            .unwrap();

        assert_eq!(value, result.value);

    }
}
