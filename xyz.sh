contract="terra1s5cg8ganpdflvst0mcf09pe9srgl72nerh7vlp"
echo "Num Tokens"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"num_tokens":\{\}\}'
echo "\nMinter"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"minter":\{\}\}'
echo "\nAll Tokens"
curl https://bombay-lcd.terra.dev/wasm/contracts/${contract}/store?query_msg='\{"all_tokens":\{\}\}'

