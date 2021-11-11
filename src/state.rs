use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{Addr, BlockInfo, Decimal, StdResult, Storage};

use crate::extension::MetaDataPersonalization;
use cw721::{ContractInfoResponse, CustomMsg, Cw721, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NftListing {
    pub label: String,
    pub listing_uri: String,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NftTraitSummary {
    pub label: String,
    pub value: Decimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NftContractInfo {
    pub description: Option<String>,
    pub src: Option<String>,
    pub banner_src: Option<String>,
    pub twitter: Option<String>,
    pub github: Option<String>,
    pub discord: Option<String>,
    pub telegram: Option<String>,
    pub listing: Vec<NftListing>,
}
impl Default for NftContractInfo {
    fn default() -> Self {
        NftContractInfo {
            description: None,
            src: None,
            banner_src: None,
            twitter: None,
            github: None,
            discord: None,
            telegram: None,
            listing: vec![],
        }
    }
}

pub struct Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub contract_info: Item<'a, ContractInfoResponse>,
    pub nft_contract_info: Item<'a, NftContractInfo>,
    pub trait_map: Item<'a, Vec<(String, Vec<NftTraitSummary>)>>,
    pub keybase_message: Item<'a, Option<String>>,
    pub minter: Item<'a, Addr>,
    pub token_count: Item<'a, u64>,
    pub public_key: Item<'a, String>,
    pub mint_amount: Item<'a, u64>,
    pub max_issuance: Item<'a, u64>,
    /// Stored as (granter, operator) giving operator full control over granter's account
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,
    pub tokens_uri: IndexedMap<'a, &'a str, String, TokenIndexString<'a>>,
    pub image_prefix: Item<'a, String>,

    pub(crate) _custom_response: PhantomData<C>,
}

// This is a signal, the implementations are in other files
impl<'a, T, C> Cw721<T, C> for Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone + MetaDataPersonalization,
    C: CustomMsg,
{
}

impl<T, C> Default for Cw721Contract<'static, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Self::new(
            "nft_info",
            "minter",
            "num_tokens",
            "operators",
            "tokens",
            "tokens__owner",
            "tokens_uri",
            "tokens_uri__owner",
            "public_key",
            "mint_amount",
            "max_issuance",
            "image_prefix",
            "nft_contract_info",
            "trait_map",
            "keybase_message",
        )
    }
}

impl<'a, T, C> Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    #[allow(clippy::too_many_arguments)]
    fn new(
        contract_key: &'a str,
        minter_key: &'a str,
        token_count_key: &'a str,
        operator_key: &'a str,
        tokens_key: &'a str,
        tokens_owner_key: &'a str,
        tokens_uri_key: &'a str,
        tokens_uri_owner_key: &'a str,
        public_key: &'a str,
        mint_amount: &'a str,
        max_issuance: &'a str,
        image_prefix: &'a str,
        nft_contract_info_key: &'a str,
        trait_map_key: &'a str,
        keybase_message_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        let uri_indexes = TokenIndexString {
            owner: MultiIndex::new(token_owner_idx_string, tokens_uri_key, tokens_uri_owner_key),
        };
        Self {
            contract_info: Item::new(contract_key),
            minter: Item::new(minter_key),
            token_count: Item::new(token_count_key),
            public_key: Item::new(public_key),
            mint_amount: Item::new(mint_amount),
            max_issuance: Item::new(max_issuance),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            tokens_uri: IndexedMap::new(tokens_uri_key, uri_indexes),
            image_prefix: Item::new(image_prefix),
            nft_contract_info: Item::new(nft_contract_info_key),
            trait_map: Item::new(trait_map_key),
            keybase_message: Item::new(keybase_message_key),
            _custom_response: PhantomData,
        }
    }

    pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.token_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn public_key(&self, storage: &dyn Storage) -> StdResult<String> {
        Ok(self.public_key.may_load(storage)?.unwrap_or_default())
    }

    pub fn mint_amount(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.mint_amount.may_load(storage)?.unwrap_or_default())
    }

    pub fn max_issuance(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.max_issuance.may_load(storage)?.unwrap_or_default())
    }

    pub fn image_prefix(&self, storage: &dyn Storage) -> StdResult<String> {
        Ok(self.image_prefix.may_load(storage)?.unwrap_or_default())
    }

    pub fn nft_contract_info(&self, storage: &dyn Storage) -> StdResult<NftContractInfo> {
        Ok(self
            .nft_contract_info
            .may_load(storage)?
            .unwrap_or_default())
    }
    pub fn nft_contract_trait_map(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<Vec<(String, Vec<NftTraitSummary>)>> {
        Ok(self.trait_map.may_load(storage)?.unwrap_or_default())
    }
    pub fn nft_contract_keybase_verification(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<Option<String>> {
        Ok(self.keybase_message.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? + 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,

    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    /// You can add any custom metadata here when you extend cw721-base
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub struct TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    // pk goes to second tuple element
    pub owner: MultiIndex<'a, (Addr, Vec<u8>), TokenInfo<T>>,
}

impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx<T>(d: &TokenInfo<T>, k: Vec<u8>) -> (Addr, Vec<u8>) {
    (d.owner.clone(), k)
}

pub struct TokenIndexString<'a> {
    // pk goes to second tuple element
    pub owner: MultiIndex<'a, (String, Vec<u8>), String>,
}

impl<'a> IndexList<String> for TokenIndexString<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<String>> + '_> {
        let v: Vec<&dyn Index<String>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}
#[allow(clippy::ptr_arg)]
pub fn token_owner_idx_string(d: &String, k: Vec<u8>) -> (String, Vec<u8>) {
    (d.clone(), k)
}
