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

use crate::state::Cw721Contract;
//use cosmwasm_std::Order;
//use cw_storage_plus::{IndexedMap, MultiIndex};
// This is a simple type to let us handle empty extensions
pub type Extension = extension::Metadata;
pub type BuyExtension = extension::BuyMetaData;

//use crate::state::TokenInfo;
#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    //use crate::extension::MetaDataPersonalization;

    //use crate::state::{token_owner_idx_change_dynamics, ChangeDynamicsIndexes};
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
        /*
                let mut tract = Cw721Contract::<Extension, Empty>::default();
                let change_dynamics_key = "change_dynamics";
                let change_dynamics_owner_key = "change_dynamics__owner";
                let change_dynamics_indexes = ChangeDynamicsIndexes {
                    owner: MultiIndex::new(
                        token_owner_idx_change_dynamics,
                        change_dynamics_key,
                        change_dynamics_owner_key,
                    ),
                };
                tract.change_dynamics = IndexedMap::new(change_dynamics_key, change_dynamics_indexes);
        */
        Ok(Response::default())
    }
}

/*
if false {
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
            let t = tract
                .tokens
                .range(deps.storage, None, None, Order::Ascending)
                .map(|f| match f {
                    Ok(token_pair) => Ok(token_pair.1),
                    Err(e) => Err(e),
                })
                .collect::<Vec<StdResult<TokenInfo<Extension>>>>();
            let mut count = 0;
            let mut errors = 0;
            for token_result in t {
                if let Ok(token) = token_result {
                    // let token_id = token.token_uri.unwrap_or_default();
                    let token_name = token.extension.get_name().unwrap_or_default();
                    let img = token.extension.get_image_raw();
                    if let Some(img_str) = img {
                        let _x = tract.image_uri.save(deps.storage, &img_str, &token_name)?;
                        count += 1;
                    } else {
                        errors += 1;
                    }
                } else {
                    errors += 1;
                }
            }
            Ok(Response::new()
                .add_attribute("count", format!("{}", count))
                .add_attribute("errors", format!("{}", errors)))
        } else {
            Ok(Response::default())
        }
 */
