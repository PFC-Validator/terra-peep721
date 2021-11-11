contract="terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav"
echo "Num Tokens"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"num_tokens":\{\}\}'
echo "\nMinter"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"minter":\{\}\}'
echo "\nAll Tokens"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"all_tokens":\{\}\}'
echo "\nNFT Contract Info"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"contract_info":\{\}\}'
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"nft_contract_info":\{\}\}'
echo "\n NFT Trait Map"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"nft_contract_trait_map":\{\}\}'
echo "\n NFT Keybase verification"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"nft_contract_keybase_verification":\{\}\}'


curl https://bombay-lcd.terra.dev/wasm/contracts/terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav/store?query_msg='\{"nft_contract_info":\{\}\}'
