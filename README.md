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
- **public_key**:" the public key of the attribute signer. This does not have to be the same as the minter. (It shouldn't be)
- **mint_amount**: the price to mint a token. in uluna (1 million uluna = 1 luna)
- **max_issuance**: maximum number of tokens that can be minted. This is not alterable in the current implementation.



It has a `buy` function, that allows anyone to perform a 'mint' like transaction. The attributes used in NFT generation are pre-generated, and signed by the owner of the NFT, and passed to the contract. *note* the funds are deposited into the contract account directly. There will be a method to transfer them to the owner coming at a later date.
 * **set_public_key**. Allows the NFT owner to set the public key to verify the signatures
 * **set_mint_amount**. Allows the NFT owner to change the price to perform a mint. It is currently hard coded to uLuna.

It has a few useful contract level functions, that can be queried:
   * **set_nft_contract_info**. This allows the NFT owner to set various details about the NFT, like discord/twitter links, a description of the project, and listing details to marketplaces.
   * **set_nft_contract_trait_info**. For Collectables, rarity is a big thing. This allows the NFT owner to set the attributes, and rarity levels of each. (no more filling in excel spreadsheets)*
   * **set_nft_contract_keybase_verification**. This allows the NFT owner to verify their ownership of the NFT collection via a keybase signature. This allows people to verify the authenticity of the collection, and it's not a 2nd rate clone
   * **set_image_prefix**. Allows the NFT owner to change the image prefix, to where the image URLs live. If they want to switch IPFS providers for example. This may not be compatible with caching strategies used by current listing providers.

There are also queries to to query non-changeabel items :
* **total_supply**. Maximum amount of tokens that can be minted.

## TODO
- [x] Add a `set_sign` function to allow the owner to set the public verification key
- [x] Add a `set_price` function to allow the owner to set auction price
- [ ] Add a 'sweep' function to allow the owner to transfer all funds to another account
