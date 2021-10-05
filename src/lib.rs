use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Empty;
pub use cw721_base::{ContractError, InstantiateMsg, MintMsg, MinterResponse, QueryMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Trait {
    pub key: String,
    pub value: String,
}
impl Trait {
    pub fn create(key: &str, value: &str) -> Self {
        Trait {
            key: key.into(),
            value: value.into(),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Extension {
    pub metadata_uri: String,
    pub traits: Vec<Trait>,
}

pub type TerraPeepNft<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;

pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    // This is a simple type to let us handle empty extensions

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        TerraPeepNft::default().instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        TerraPeepNft::default().execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        TerraPeepNft::default().query(deps, env, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721::Cw721Query;

    const CREATOR: &str = "terra1d85ncnvn822u5lul9kf8430dd3chyjd3ka2f98";

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies(&[]);
        let contract = TerraPeepNft::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "TerraPeeps".to_string(),
            symbol: "PEEPS".to_string(),
            minter: CREATOR.to_string(),
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id = "PeepDEV";
        let body = Trait::create("body", "body/bar");
        let face = Trait::create("face", "face/id-123");
        let color = Trait::create("color", "color/red");
        let traits = vec![body, face, color];

        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "terra1d85ncnvn822u5lul9kf8430dd3chyjd3ka2f98".to_string(),
            name: "Terra Peep #123".to_string(),
            description: Some("Created with love by people like you".into()),
            image: Some("https://cloudflare-ipfs.com/ipfs/QmPcVMgezRnSq33FoZvZiqYuRL4vGT54bP6nT4QVkwoMv7".into()),
            extension: Extension {
                metadata_uri: "https://cloudflare-ipfs.com/ipfs/QmPcVMgezRnSq33FoZvZiqYuRL4vGT54bP6nT4QVkwoMv7".into(),
                traits
            },
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        assert_eq!(res.name, mint_msg.name);
        assert_eq!(res.description, mint_msg.description.unwrap());
        assert_eq!(res.image, mint_msg.image);
        assert_eq!(res.extension, mint_msg.extension);
    }
}
