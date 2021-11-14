# TerraPeep721
## TL;DR
This is a working CW721 contract for the [Terra](https://terra.money/) blockchain, and is in active use by the [TerraPeeps](https://terrapeeps.com) project.

It serves TerraPeeps needs. It may serve yours.

If you think this was useful, feel free to delegate to the [PFC](https://station.terra.money/validator/terravaloper12g4nkvsjjnl0t7fvq3hdcw7y8dc9fq69nyeu9q) validator. It will help defray the costs.

[PFC](https://twitter.com/PFC_Validator) - As Terra is all about Pink Fluffy Characters right... feel free to drop me a line


## Details

It extends the standard CW721 in several ways:

To instantiate you need to specify:
- **name**: standard cw721 name
- **symbol**: standard cw721 symbol
- **minter**: the admin account of the NFT.
- **public_key**: the public key of the attribute signer. This does not have to be the same as the minter. (It shouldn't be)
- **mint_amount**: the price to mint a token. in uluna (1 million uluna = 1 luna)
- **max_issuance**: maximum number of tokens that can be minted. This is not alterable in the current implementation.



It has a `buy` function, that allows anyone to perform a 'mint' like transaction. The attributes used in NFT generation are pre-generated, and signed by the owner of the NFT, and passed to the contract. *note* the funds are deposited into the contract account directly. There will be a method to transfer them to the owner coming at a later date.
 * **set_public_key**. Allows the NFT owner to set the public key to verify the signatures
 * **set_mint_amount**. Allows the NFT owner to change the price to perform a mint. It is currently hard coded to uLuna.

It has a few useful contract level functions, that can be queried:
   * **set_nft_contract_info**. This allows the NFT owner to set various details about the NFT, like discord/twitter links, a description of the project, and listing details to marketplaces.
   * **set_nft_contract_trait_info**. For Collectables, rarity is a big thing. This allows the NFT owner to set the attributes, and rarity levels of each. (no more filling in excel spreadsheets)*
   * **set_nft_contract_keybase_verification**. This allows the NFT owner to verify their ownership of the NFT collection via a keybase signature. This allows people to verify the authenticity of the collection, and it's not a 2nd rate clone
   * **set_image_prefix**. Allows the NFT owner to change the image prefix, to where the image URLs live. If they want to switch from IPFS to something else (and back later). This may not be compatible with caching strategies used by current listing providers. It's probably best not to use it

There are also queries to query immutable items: (note it is immutable in this level, there is nothing stopping someone forking this and changing it)
* **total_supply**. Maximum amount of tokens that can be minted.

### How to build
* `cargo build` command will build the contract as normal rustaceans expect
* `cargo unit-test` will run the tests.
* `cargo optimize` or `cargo optimize-w32` will optimize the contract for potential upload onto the chain. Yes. you can build on windows, and it's great!

### How to upload it to the chain
```sh
terrad tx wasm store artifacts/terra_peep721.wasm --chain-id bombay-12 --from your-wallet --fees 391868uusd --gas auto -y -b sync --gas-adjustment 1.2

```
this will upload the contract to the chain, and provide a `code id`. This can be used to either instantiate the contract, or migrate the existing contract
to instantiate it.
```shell
minter=terra1d85ncnvn822u5lul9kf8430dd3chyjd3ka2f98
code_id=12345
json='{"name":"your name","symbol":"your symbol","minter":"${minter}","public_key":"A8O7tqWAvsKW9XA7p2W8YZdIZmmadf9qoQmRiZq8xpvl","mint_amount":2000000,"max_issuance":10000}'
terrad tx wasm instantiate ${code_id} ${json} --chain-id bombay-12 --gas auto --from your-wallet --fees  5627uluna --admin ${minter}

```
the output of  instantiate should provide you with a contract address. this is used in your applications.
see [finder](https://finder.terra.money/testnet/tx/CB141B83A90D04EC71EFFF0D0148F7D4D63C5B0FC487413FB76267CD73EF49BD) for an example.
to upgrade, you will need to migrate the contract to the new `code_id`

for example:
```shell
terrad tx wasm migrate terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav 18416 '{}'  --from terrapeep --chain-id bombay-12 --fees 2640uluna
```
you should probably set the default image source, NFT contract info, and a keybase signed message.
I use 'TerraPeeps Bombay Contract is at terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav'.

As long as you have the contract address signed in the message, it should be sufficient.

```shell
terrad tx wasm execute terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav  '{"set_image_prefix":{"prefix":"ipfs://"}}' --from terrapeep --chain-id bombay-12 --fees 2640uluna
contract_info='{"set_nft_contract_info":{"listing": [{"label":"knowhere","listing_uri":"https://knowhere.art/collection/terra1t0l0sz0efnr7cm3hxked7nn2x7xx5syw02k8tc"}],                   "src":"https://example.com/logo",                   "banner_src": "https://example.com/banner",                   "descriptions": "Peeps are the social NFT",                   "discord":"https://discord.gg/rF5T86hVMG",                   "github": null,                   "telegram":null,                   "twitter":"https://twitter.com/TerraPeep"}}'

terrad tx wasm execute terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav  ${contract_info} --from terrapeep --chain-id bombay-12 --fees 2640uluna
terrad tx wasm execute terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav  '{"set_nft_contract_keybase_verification":{"message":"BEGIN KEYBASE SALTPACK ENCRYPTED MESSAGE. keDIDMQWYvVR58B FTfTeDQNI531bT7 WTI8PEuoKWxELk1 MFJeAHVqx4s0efe gkGGapAZTDrPVYw CGLVhvxPExCUKp1 NTfmSyoZAbFRuqg XWKMsHhyJgzViNo iyk60GT3AUeZIrc 9ibie85pwi4MD0v Sbld52zopYnIxVj lhxHhC9T2VGUyNO lQZu2KvmMa9IrmI MrNLyEtcT9Ra1Pd ZQ02Qin64JkMXWj Hi8k6TCngHKlgvl sWGQayfpGPzUNkq XYqEpwXsqc60dXK FRqXsoSBRj7KNJ8 3TuSFxFv3B8ycY4 GPtLPhprzFyXbTl KLwRLNnExQdF7bw kaFXYKE41hE0diV J42bZj5tluOfsHu xhfpMdb54tWiVFl Pdear6Jp05S7ahI jHD74OtZLTeTMZo UEZwJpUmuzHpAmG NeBgjCqgv41GmKW 1eGIPAJnGctWbpX I73Z32AwtkpDxtO Pz. END KEYBASE SALTPACK ENCRYPTED MESSAGE."}}' --from terrapeep --chain-id bombay-12 --fees 2640uluna

```

### how to generate a 'buy' message
the "1-step buy(tm)" is a simple way to buy NFTs.
It allows the end-user to mint the NFT directly, avoiding the need for the minter to 'pre-mint' (which can be expensive), and 'post-mint' (which leaves the end-user in a potential rug-pull situation).

The buy message requires a signed message. The signature is generated by the minter, and consists of the end-user's wallet address, and the attributes of the NFT.
The buyer's wallet address is used to avoid replication. you could also add a 'expiry time', or other constraints to it.
The only downside is that the end-user can 'see' the NFT attributes, and can potentially be tricked 'refreshing' until they find one they like. you could obfuscate the attributes (which may be a good idea just for compression). but it's really hard to do encryption on a chain..

an example of this message is: [here](https://finder.terra.money/mainnet/tx/896C873F69D72F07E06F852058133E7F2C883C65DC51117EEE59638605A4DCE9)

this is a example of what is signed. check out the test suite for more examples.

let json = r#"random/{"token_uri":"https://www.merriam-webster.com/dictionary/petrify","image":null,"image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"male"},{"display_type":null,"trait_type":"name","value":"Jim Morrisson"}],"background_color":null,"animation_url":null,"youtube_url":null,"current_status":null}"#;

## TODO
- [x] Add a `set_sign` function to allow the owner to set the public verification key
- [x] Add a `set_price` function to allow the owner to set auction price
- [x] Add a 'sweep' function to allow the owner to transfer all funds to another account

