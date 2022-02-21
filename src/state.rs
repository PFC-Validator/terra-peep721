use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{Addr, BlockInfo, Decimal, StdResult, Storage, Uint128};

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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
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
/*
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
*/
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
    pub change_amount: Item<'a, u64>,
    pub change_multiplier: Item<'a, u64>,
    pub max_issuance: Item<'a, u64>,
    /// Stored as (granter, operator) giving operator full control over granter's account
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,
    pub tokens_uri: IndexedMap<'a, &'a str, String, TokenIndexString<'a>>,
    pub image_uri: IndexedMap<'a, &'a str, String, TokenIndexString<'a>>,
    pub image_prefix: Item<'a, String>,
    pub change_dynamics: IndexedMap<'a, &'a str, ChangeDynamics, ChangeDynamicsIndexes<'a>>,
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
            "image_uri",
            "image_uri__owner",
            "change_amount",
            "change_multiplier",
            "change_dynamics",
            "change_dynamics__owner",
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
        image_uri_key: &'a str,
        image_uri_owner_key: &'a str,
        change_amount: &'a str,
        change_multiplier: &'a str,
        change_dynamics_key: &'a str,
        change_dynamics_owner_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        let uri_indexes = TokenIndexString {
            owner: MultiIndex::new(token_owner_idx_string, tokens_uri_key, tokens_uri_owner_key),
        };
        let image_indexes = TokenIndexString {
            owner: MultiIndex::new(image_uri_idx_string, image_uri_key, image_uri_owner_key),
        };
        let change_dynamics_indexes = ChangeDynamicsIndexes {
            owner: MultiIndex::new(
                token_owner_idx_change_dynamics,
                change_dynamics_key,
                change_dynamics_owner_key,
            ),
        };

        Self {
            contract_info: Item::new(contract_key),
            minter: Item::new(minter_key),
            token_count: Item::new(token_count_key),
            public_key: Item::new(public_key),
            mint_amount: Item::new(mint_amount),
            change_amount: Item::new(change_amount),
            change_multiplier: Item::new(change_multiplier),
            max_issuance: Item::new(max_issuance),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            tokens_uri: IndexedMap::new(tokens_uri_key, uri_indexes),
            image_uri: IndexedMap::new(image_uri_key, image_indexes),
            image_prefix: Item::new(image_prefix),
            nft_contract_info: Item::new(nft_contract_info_key),
            trait_map: Item::new(trait_map_key),
            keybase_message: Item::new(keybase_message_key),
            change_dynamics: IndexedMap::new(change_dynamics_key, change_dynamics_indexes),
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
    pub fn change_amount(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.change_amount.may_load(storage)?.unwrap_or_default())
    }
    pub fn change_multiplier(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self
            .change_multiplier
            .may_load(storage)?
            .unwrap_or_default())
    }
    pub fn max_issuance(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.max_issuance.may_load(storage)?.unwrap_or_default())
    }

    pub fn change_details(&self, storage: &dyn Storage) -> StdResult<ChangeDetail> {
        let amount = self.change_amount.may_load(storage)?.unwrap_or_default();
        let multiplier = self
            .change_multiplier
            .may_load(storage)?
            .unwrap_or_default();
        Ok(ChangeDetail {
            change_amount: amount,
            change_multiplier: multiplier,
        })
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
    pub fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? - 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }
}

/// History of token changes
/// this is a separate struct as it happened after v1.. and migrations are challenging
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ChangeDynamics {
    pub owner: Addr,
    pub token_id: String,
    /// how many changes have occurred to this token
    pub change_count: u64,
    pub unique_owners: Vec<Addr>,
    pub transfer_count: u64,
    pub block_number: u64,
    pub price_ceiling: Uint128,
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ChangeDetail {
    /// amount the change will cost (multiplied by # of times changes have occurred)
    pub change_amount: u64,
    /// change multiplier
    pub change_multiplier: u64,
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

pub struct ChangeDynamicsIndexes<'a> {
    // pk goes to second tuple element
    pub owner: MultiIndex<'a, (Addr, Vec<u8>), ChangeDynamics>,
}

impl<'a> IndexList<ChangeDynamics> for ChangeDynamicsIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<ChangeDynamics>> + '_> {
        let v: Vec<&dyn Index<ChangeDynamics>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}
pub fn token_owner_idx_change_dynamics(d: &ChangeDynamics, k: Vec<u8>) -> (Addr, Vec<u8>) {
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

// not sure if these should just be a single function
#[allow(clippy::ptr_arg)]
pub fn token_owner_idx_string(d: &String, k: Vec<u8>) -> (String, Vec<u8>) {
    (d.clone(), k)
}
#[allow(clippy::ptr_arg)]
pub fn image_uri_idx_string(d: &String, k: Vec<u8>) -> (String, Vec<u8>) {
    (d.clone(), k)
}
