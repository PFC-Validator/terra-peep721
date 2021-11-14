mod contract_tests;
mod error;
mod execute;
mod extension;
//mod identity_digest;
pub mod msg;
mod query;
//mod secp256k1;
pub mod state;

pub use crate::error::ContractError;
pub use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, MintMsg, MinterResponse, QueryMsg};

use crate::state::{image_uri_idx_string, Cw721Contract, TokenIndexString};
use cw_storage_plus::{IndexedMap, MultiIndex};

// This is a simple type to let us handle empty extensions
pub type Extension = extension::Metadata;
pub type BuyExtension = extension::BuyMetaData;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult};

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        let tract = Cw721Contract::<Extension, Empty>::default();
        tract.instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<Extension>,
    ) -> Result<Response, ContractError> {
        let tract = Cw721Contract::<Extension, Empty>::default();
        tract.execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        let tract = Cw721Contract::<Extension, Empty>::default();
        tract.query(deps, env, msg)
    }
    #[entry_point]
    pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
        let mut tract = Cw721Contract::<Extension, Empty>::default();

        // set the new version
        //   cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let image_uri_key = "image_uri";
        let image_uri_owner_key = "image_uri__owner";
        let image_indexes = TokenIndexString {
            owner: MultiIndex::new(image_uri_idx_string, image_uri_key, image_uri_owner_key),
        };
        let image_uri = IndexedMap::new(image_uri_key, image_indexes);
        tract.image_uri = image_uri;

        Ok(Response::default())
    }
}
