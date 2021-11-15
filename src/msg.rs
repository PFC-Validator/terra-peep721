use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::extension::MetaDataPersonalization;
use crate::state::{NftListing, NftTraitSummary};
use crate::BuyExtension;
use cosmwasm_std::Binary;
use cw721::Expiration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,

    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    pub minter: String,
    /// public key that can sign buy messages
    pub public_key: String,
    /// minimum amount of uluna to buy via BUY message
    pub mint_amount: u64,
    /// minimum amount of uusd to execute a change message
    pub change_amount: u64,
    /// price change multiplier
    pub change_multiplier: u64,
    /// max amount of tokens to issue
    pub max_issuance: u64,
}

/// This is like Cw721ExecuteMsg but we add a Mint command for an owner
/// to make this stand-alone. You will likely want to remove mint and
/// use other control logic in any contract that inherits this.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T>
where
    T: MetaDataPersonalization,
{
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },

    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Allow a buyer to mint a NFT directly
    Buy(BuyMsg),
    /// Owner function: Sends coins in the contract to admin
    Sweep { denom: String },
    /// Owner function: change public key
    SetPublicKey { public_key: String },
    /// Owner function: change mint price (uluna)
    SetMintAmount { mint_amount: u64 },
    /// Owner function: change change #times multipler)
    SetChangeTimesMultiplier { change_multiplier: u64 },
    /// Owner function: change change name price (uusd)
    SetChangeAmount { change_amount: u64 },
    /// User message: allow owner to change status field of NFT
    SetTokenStatus { status: String, token_id: String },
    /// User message: allow owner to change name & description field of NFT
    SetTokenNameDescription {
        description: Option<String>,
        name: Option<String>,
        token_id: String,
    },
    /// Owner message: change prefix for images. defaults to ipfs://
    SetImagePrefix { prefix: String },
    /// Owner message: Set information about the NFT Collection
    SetNftContractInfo {
        description: Option<String>,
        src: Option<String>,
        banner_src: Option<String>,
        twitter: Option<String>,
        github: Option<String>,
        discord: Option<String>,
        telegram: Option<String>,
        listing: Vec<NftListing>,
    },
    /// Owner message: Set information about the NFT Traits
    SetNftContractTraitInfo {
        trait_map: Vec<(String, Vec<NftTraitSummary>)>,
    },
    /// Owner message: Set keybase verification string
    SetNftContractKeybaseVerification { message: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg<T> {
    /// Unique ID of the NFT
    pub token_id: String,
    /// The owner of the newly minter NFT
    pub owner: String,
    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,
    /// Any custom extension used by this contract
    pub extension: T,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct BuyMsg
//<T>
where
//  T: MetaDataPersonalization,
{
    /// Unique ID of the NFT. This is generated via attributes
    //  pub token_id: String,
    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// signature that proves the request was initiated by a trusted party
    pub signature: String,
    /// attributes should be a json string
    pub attributes: String,
    /// other attributes that can come from the purchaser
    pub buy_metadata: BuyExtension,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return the owner of the given token, error if token does not exist
    /// Return type: OwnerOfResponse
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
    /// List all operators that can access all of the owner's tokens
    /// Return type: `ApprovedForAllResponse`
    ApprovedForAll {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Total number of tokens issued
    NumTokens {},

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract: `ContractInfoResponse`
    ContractInfo {},
    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract: `NftInfoResponse`
    NftInfo { token_id: String },
    /// How many changes has occurred to this token
    ChangeDynamics { token_id: String },
    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract: `NftInfoResponse`
    ImageInfo { img_uri: String },
    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients: `AllNftInfo`
    AllNftInfo {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    /// Return type: TokensResponse.
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    /// Return type: TokensResponse.
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllImgTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    /// Return type: TokensResponse.
    RangeTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Return the minter
    Minter {},
    /// Return the public key that is being used to validate messages with signatures
    PublicKey {},
    /// Return the mint amount
    MintAmount {},
    /// Return the change amount and multiplier
    ChangeDetails {},
    /// Return the total supply
    TotalSupply {},
    /// Return the prefix for the images. defaults to ipfs://
    ImagePrefix {},
    /// Returns top-level NFT metadata about the contract: `NFTContractInfoResponse`
    NftContractInfo {},
    /// Returns top-level NFT metadata about the trait maps:
    NftContractTraitMap {},
    /// Returns top-level NFT metadata about the keybase signature:
    NftContractKeybaseVerification {},
}

/// Shows who can mint these tokens
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MinterResponse {
    pub minter: String,
}
