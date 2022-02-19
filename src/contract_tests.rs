#![cfg(test)]

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, Coin, CosmosMsg, Decimal, DepsMut, Empty, Response, StdError, WasmMsg,
};
use std::str::FromStr;

use cw721::{
    ApprovedForAllResponse, ContractInfoResponse, Cw721Query, Cw721ReceiveMsg, Expiration,
    NftInfoResponse, OwnerOfResponse,
};

use crate::extension::{MetaDataPersonalization, Metadata, Trait};
use crate::msg::BuyMsg;
use crate::state::{NftListing, NftTraitSummary};
use crate::{
    BuyExtension, ContractError, Cw721Contract, ExecuteMsg, Extension, InstantiateMsg, MintMsg,
    QueryMsg,
};

const MINTER: &str = "merlin";
const CONTRACT_NAME: &str = "Magic Power";
const SYMBOL: &str = "MGK";
//const PUBLIC_KEY: &str = "AiMzHaA2bvnDXfHzkjMM+vkSE/p0ymBtAFKUnUtQAeXe";
const PUBLIC_KEY: &str = "AlRu+P0GWx+4eYLCOzNk45QiDjheKvHJUTDHT5dFtHUc";

fn setup_contract(deps: DepsMut<'_>) -> Cw721Contract<'static, Extension, Empty> {
    let contract = Cw721Contract::default();
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        minter: String::from(MINTER),
        public_key: String::from(PUBLIC_KEY),
        mint_amount: 3_000_000u64,
        change_amount: 1_000_000u64,
        change_multiplier: 2u64,
        max_issuance: 3u64,
    };
    let info = mock_info("creator", &[]);
    let res = contract.instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    contract
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let contract = Cw721Contract::<Extension, Empty>::default();

    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        minter: String::from(MINTER),
        public_key: String::from(PUBLIC_KEY),
        mint_amount: 3_000_000u64,
        change_amount: 1_000_000u64,
        change_multiplier: 10,
        max_issuance: 3u64,
    };
    let info = mock_info("creator", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = contract
        .instantiate(deps.as_mut(), mock_env(), info, msg)
        .unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = contract.minter(deps.as_ref()).unwrap();
    assert_eq!(MINTER, res.minter);
    let info = contract.contract_info(deps.as_ref()).unwrap();
    assert_eq!(
        info,
        ContractInfoResponse {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
        }
    );

    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(0, count.count);

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(0, tokens.tokens.len());
}

#[test]
fn minting() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    let token_id = "petrify".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let mint_msg = ExecuteMsg::<Extension>::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("medusa"),
        token_uri: Some(token_uri.clone()),
        extension: extension.clone(),
    });

    // random cannot mint
    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    match err {
        ContractError::Unauthorized { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    //assert_eq!(err, ContractError::Unauthorized {});

    // minter can mint
    let allowed = mock_info(MINTER, &[]);
    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg)
        .unwrap();

    // ensure num tokens increases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(1, count.count);

    // unknown nft returns error
    let _ = contract
        .nft_info(deps.as_ref(), "unknown".to_string())
        .unwrap_err();

    // this nft info is correct
    let info = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
    assert_eq!(
        info,
        NftInfoResponse::<Extension> {
            token_uri: Some(token_uri),
            extension: extension.clone(),
        }
    );

    // owner info is correct
    let owner = contract
        .owner_of(deps.as_ref(), mock_env(), token_id.clone(), true)
        .unwrap();
    assert_eq!(
        owner,
        OwnerOfResponse {
            owner: String::from("medusa"),
            approvals: vec![],
        }
    );

    // Cannot mint same token_id again
    let mint_msg2 = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("hercules"),
        token_uri: Some("Qasdasd".to_string()),
        extension: extension.clone(),
    });

    let allowed = mock_info(MINTER, &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg2)
        .unwrap_err();
    match err {
        ContractError::Claimed { .. } => {}
        ContractError::ImageClaimed { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    //assert_eq!(err, ContractError::Claimed {});

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id], tokens.tokens);
}
#[test]
fn burning() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    let token_id = "petrify".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: MINTER.to_string(),
        token_uri: Some(token_uri),
        extension,
    });

    let burn_msg = ExecuteMsg::Burn { token_id };

    // mint some NFT
    let allowed = mock_info(MINTER, &[]);
    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed.clone(), mint_msg)
        .unwrap();

    // random not allowed to burn
    let random = mock_info("random", &[]);
    let err = contract.execute(deps.as_mut(), mock_env(), random, burn_msg.clone());

    match err {
        Err(ContractError::Unauthorized {}) => {}
        Err(e) => {
            assert!(false, "Unexpected Error {:?}", e)
        }
        Ok(_) => {
            assert!(false, "Unexpected Ok")
        }
    }

    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed, burn_msg)
        .unwrap();

    // ensure num tokens decreases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(0, count.count);

    // trying to get nft returns error
    let _ = contract
        .nft_info(deps.as_ref(), "petrify".to_string())
        .unwrap_err();

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert!(tokens.tokens.is_empty());
}

#[test]
fn transferring_nft() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "melt".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/melt".to_string();
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("venus"),
        token_uri: Some(token_uri),
        extension: extension.clone(),
    });

    let minter = mock_info(MINTER, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // random cannot transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: token_id.clone(),
    };

    let err = contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap_err();
    match err {
        ContractError::Unauthorized { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    //assert_eq!(err, ContractError::Unauthorized {});

    // owner can
    let random = mock_info("venus", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: token_id.clone(),
    };

    let res = contract
        .execute(deps.as_mut(), mock_env(), random.clone(), transfer_msg)
        .unwrap();

    match contract.change_dynamics.load(&deps.storage, &token_id) {
        Ok(dynamics) => {
            assert_eq!(dynamics.owner, "random".to_string());
            assert!(dynamics.unique_owners.contains(&random.sender));
            assert_eq!(dynamics.unique_owners.len(), 2);
            assert_eq!(dynamics.transfer_count, 1);
        }
        Err(e) => {
            assert!(false, "Unexpected Error {:?}", e)
        }
    }

    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", "venus")
            .add_attribute("recipient", "random")
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn sending_nft() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "melt".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/melt".to_string();
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("venus"),
        token_uri: Some(token_uri),
        extension: extension.clone(),
    });

    let minter = mock_info(MINTER, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    let msg = to_binary("You now have the melting power").unwrap();
    let target = String::from("another_contract");
    let send_msg = ExecuteMsg::SendNft {
        contract: target.clone(),
        token_id: token_id.clone(),
        msg: msg.clone(),
    };

    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, send_msg.clone())
        .unwrap_err();
    match err {
        ContractError::Unauthorized { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    //    assert_eq!(err, ContractError::Unauthorized {});

    // but owner can
    let random = mock_info("venus", &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), random, send_msg)
        .unwrap();

    let payload = Cw721ReceiveMsg {
        sender: String::from("venus"),
        token_id: token_id.clone(),
        msg,
    };
    let expected = payload.into_cosmos_msg(target.clone()).unwrap();
    // ensure expected serializes as we think it should
    match &expected {
        CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => {
            assert_eq!(contract_addr, &target)
        }
        m => panic!("Unexpected message type: {:?}", m),
    }
    // and make sure this is the request sent by the contract
    assert_eq!(
        res,
        Response::new()
            .add_message(expected)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", "venus")
            .add_attribute("recipient", "another_contract")
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn approving_revoking() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "grow".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/grow".to_string();
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("demeter"),
        token_uri: Some(token_uri),
        extension: extension.clone(),
    });

    let minter = mock_info(MINTER, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // Give random transferring power
    let approve_msg = ExecuteMsg::Approve {
        spender: String::from("random"),
        token_id: token_id.clone(),
        expires: None,
    };
    let owner = mock_info("demeter", &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), owner, approve_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", "demeter")
            .add_attribute("spender", "random")
            .add_attribute("token_id", token_id.clone())
    );

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id.clone(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap();

    // Approvals are removed / cleared
    let query_msg = QueryMsg::OwnerOf {
        token_id: token_id.clone(),
        include_expired: None,
    };
    let res: OwnerOfResponse = from_binary(
        &contract
            .query(deps.as_ref(), mock_env(), query_msg.clone())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![],
        }
    );

    // Approve, revoke, and check for empty, to test revoke
    let approve_msg = ExecuteMsg::Approve {
        spender: String::from("random"),
        token_id: token_id.clone(),
        expires: None,
    };
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_msg)
        .unwrap();

    let revoke_msg = ExecuteMsg::Revoke {
        spender: String::from("random"),
        token_id,
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_msg)
        .unwrap();

    // Approvals are now removed / cleared
    let res: OwnerOfResponse = from_binary(
        &contract
            .query(deps.as_ref(), mock_env(), query_msg)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![],
        }
    );
}

#[test]
fn approving_all_revoking_all() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a couple tokens (from the same owner)
    let token_id1 = "grow1".to_string();
    let token_uri1 = "https://www.merriam-webster.com/dictionary/grow1".to_string();

    let token_id2 = "grow2".to_string();
    let token_uri2 = "https://www.merriam-webster.com/dictionary/grow2".to_string();
    let mut extension = Metadata {
        token_uri: token_uri1.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let mint_msg1 = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: String::from("demeter"),
        token_uri: Some(token_uri1),
        extension: extension.clone(),
    });
    extension.image = Some("QmReTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string());

    let minter = mock_info(MINTER, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg1)
        .unwrap();

    let mint_msg2 = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: String::from("demeter"),
        token_uri: Some(token_uri2),
        extension: extension.clone(),
    });

    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg2)
        .unwrap();

    // paginate the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, Some(1)).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id1.clone()], tokens.tokens);
    let tokens = contract
        .all_tokens(deps.as_ref(), Some(token_id1.clone()), Some(3))
        .unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id2.clone()], tokens.tokens);

    // demeter gives random full (operator) power over her tokens
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("random"),
        expires: None,
    };
    let owner = mock_info("demeter", &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), owner, approve_all_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", "demeter")
            .add_attribute("operator", "random")
    );

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id1,
    };
    contract
        .execute(deps.as_mut(), mock_env(), random.clone(), transfer_msg)
        .unwrap();

    // random can now send
    let inner_msg = WasmMsg::Execute {
        contract_addr: "another_contract".into(),
        msg: to_binary("You now also have the growing power").unwrap(),
        funds: vec![],
    };
    let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

    let send_msg = ExecuteMsg::SendNft {
        contract: String::from("another_contract"),
        token_id: token_id2,
        msg: to_binary(&msg).unwrap(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random, send_msg)
        .unwrap();

    // Approve_all, revoke_all, and check for empty, to test revoke_all
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("operator"),
        expires: None,
    };
    // person is now the owner of the tokens
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner, approve_all_msg)
        .unwrap();

    let res = contract
        .all_approvals(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovedForAllResponse {
            operators: vec![cw721::Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }]
        }
    );

    // second approval
    let buddy_expires = Expiration::AtHeight(1234567);
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("buddy"),
        expires: Some(buddy_expires),
    };
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_all_msg)
        .unwrap();

    // and paginate queries
    let res = contract
        .all_approvals(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            None,
            Some(1),
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovedForAllResponse {
            operators: vec![cw721::Approval {
                spender: String::from("buddy"),
                expires: buddy_expires,
            }]
        }
    );
    let res = contract
        .all_approvals(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            Some(String::from("buddy")),
            Some(2),
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovedForAllResponse {
            operators: vec![cw721::Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }]
        }
    );

    let revoke_all_msg = ExecuteMsg::RevokeAll {
        operator: String::from("operator"),
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_all_msg)
        .unwrap();

    // Approvals are removed / cleared without affecting others
    let res = contract
        .all_approvals(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            false,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovedForAllResponse {
            operators: vec![cw721::Approval {
                spender: String::from("buddy"),
                expires: buddy_expires,
            }]
        }
    );

    // ensure the filter works (nothing should be here
    let mut late_env = mock_env();
    late_env.block.height = 1234568; //expired
    let res = contract
        .all_approvals(
            deps.as_ref(),
            late_env,
            String::from("person"),
            false,
            None,
            None,
        )
        .unwrap();
    assert_eq!(0, res.operators.len());
}

#[test]
fn query_tokens_by_owner() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());
    let minter = mock_info(MINTER, &[]);

    // Mint a couple tokens (from the same owner)
    let token_id1 = "grow1".to_string();
    let demeter = String::from("Demeter");
    let token_id2 = "grow2".to_string();
    let ceres = String::from("Ceres");
    let token_id3 = "sing".to_string();
    let mut extension = Metadata {
        token_uri: token_id1.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: demeter.clone(),
        token_uri: Some("QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv1".to_string()),
        extension: extension.clone(),
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    extension.image = Some("QmRdTSx4v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string());
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: ceres.clone(),
        token_uri: Some("QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv2".to_string()),
        extension: extension.clone(),
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();
    extension.image = Some("QmRdTSx5v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string());

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id3.clone(),
        owner: demeter.clone(),
        token_uri: Some("QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv3".to_string()),
        extension: extension.clone(),
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // get all tokens in order:
    let expected = vec![token_id1.clone(), token_id2.clone(), token_id3.clone()];
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(&expected, &tokens.tokens);
    // paginate
    let tokens = contract.all_tokens(deps.as_ref(), None, Some(2)).unwrap();
    assert_eq!(&expected[..2], &tokens.tokens[..]);
    let tokens = contract
        .all_tokens(deps.as_ref(), Some(expected[1].clone()), None)
        .unwrap();
    assert_eq!(&expected[2..], &tokens.tokens[..]);

    // get by owner
    let by_ceres = vec![token_id2];
    let by_demeter = vec![token_id1, token_id3];
    // all tokens by owner
    let tokens = contract
        .tokens(deps.as_ref(), demeter.clone(), None, None)
        .unwrap();
    assert_eq!(&by_demeter, &tokens.tokens);
    let tokens = contract.tokens(deps.as_ref(), ceres, None, None).unwrap();
    assert_eq!(&by_ceres, &tokens.tokens);

    // paginate for demeter
    let tokens = contract
        .tokens(deps.as_ref(), demeter.clone(), None, Some(1))
        .unwrap();
    assert_eq!(&by_demeter[..1], &tokens.tokens[..]);
    let tokens = contract
        .tokens(deps.as_ref(), demeter, Some(by_demeter[0].clone()), Some(3))
        .unwrap();
    assert_eq!(&by_demeter[1..], &tokens.tokens[..]);
}

#[test]
fn query_tokens_by_image() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());
    let minter = mock_info(MINTER, &[]);

    // Mint a couple tokens (from the same owner)
    let token_id1 = "grow1".to_string();
    let demeter = String::from("Demeter");
    let token_id2 = "grow2".to_string();
    let ceres = String::from("Ceres");
    let token_id3 = "sing".to_string();
    let mut extension = Metadata {
        token_uri: token_id1.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: demeter.clone(),
        token_uri: Some("QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv1".to_string()),
        extension: extension.clone(),
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    extension.image = Some("QmRdTSx4v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string());
    extension.token_uri = "QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv2".to_string();
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: ceres.clone(),
        token_uri: Some("QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv2".to_string()),
        extension: extension.clone(),
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();
    extension.image = Some("QmRdTSx5v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string());
    extension.token_uri = "QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv3".to_string();
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id3.clone(),
        owner: demeter.clone(),
        token_uri: Some("QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv3".to_string()),
        extension: extension.clone(),
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    let result = contract
        .image_info(
            deps.as_ref(),
            "QmRdTSx4v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string(),
        )
        .unwrap();
    assert_eq!(
        result.extension.token_uri,
        "QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv2"
    );
    assert_eq!(
        result.token_uri.unwrap(),
        "QmcTLCzFDJDf4hSKHn75MTVifwWNobiB4ZvXtYMTxUVcv2"
    );
}

#[test]
fn buying() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let attributes: Vec<Trait> = vec![
        Trait {
            display_type: None,
            trait_type: "gender".to_string(),
            value: "male".to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "name".to_string(),
            value: "Jim Morrisson".to_string(),
        },
    ];
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();
    //   println!("{}", &json_string);
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "TODO".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });

    // no money
    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    match err {
        ContractError::Funds { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }

    // not enough money
    let random = mock_info("random", &[Coin::new(2_999_999u128, "uluna")]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    match err {
        ContractError::Funds { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    //wrong type of money
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uuusd")]);

    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    match err {
        ContractError::Funds { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    //bad signature
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);

    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    match err {
        ContractError::CryptoVerify { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "TODO".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });

    //bad signature
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);

    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    match err {
        ContractError::CryptoVerify { .. } => {}
        _ => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        // signature is supposed to be generated from account sending the message + extension
        signature: "fJCPvlqjmTZxuKT1Uu8FqEmEuvEfNcKv3rdZ267qVFpU3S7AzIXG7ic1d9NxfUdGBdRpwW/yfdoN7nM34dW8LQ==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });
    //println!("EXEC:{}", serde_json_wasm::to_string(&mint_msg).unwrap());
    //good signature, the token_id not so much.
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);

    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Ok(resp) => {
            println!("{:?}", resp)
        }
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    //assert_eq!(err, ContractError::Unauthorized {});

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec!["James Dean"], tokens.tokens);
}

#[test]
fn max_issued() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let attributes: Vec<Trait> = vec![
        Trait {
            display_type: None,
            trait_type: "gender".to_string(),
            value: "male".to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "name".to_string(),
            value: "Jim Morrisson".to_string(),
        },
    ];
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "fJCPvlqjmTZxuKT1Uu8FqEmEuvEfNcKv3rdZ267qVFpU3S7AzIXG7ic1d9NxfUdGBdRpwW/yfdoN7nM34dW8LQ==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });
    //good signature
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {}
    }
    //    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();

    let buy_msg = BuyExtension {
        male_name: "Jimmy Sparks".to_string(),
        female_name: "Florence O'Niel".to_string(),
    };
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "fJCPvlqjmTZxuKT1Uu8FqEmEuvEfNcKv3rdZ267qVFpU3S7AzIXG7ic1d9NxfUdGBdRpwW/yfdoN7nM34dW8LQ==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });
    //good signature, but should have been claimed
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);

    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    match err {
        ContractError::Claimed {} => {}
        _ => {
            assert!(
                false,
                "Unexpected Error. should have been claimed. Token URI duplicate {:?}",
                err
            )
        }
    }

    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();

    assert_eq!(tokens.tokens.len(), 1);
    let buy_msg = BuyExtension {
        male_name: "Peter Walker".to_string(),
        female_name: "Lady Ga Ga".to_string(),
    };

    let json_string = r#"{"token_uri":"https://www.merriam-webster.com/dictionary/token2","image":"QASD","image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"male"},{"display_type":null,"trait_type":"name","value":"James T. Kirk"}],"background_color":null,"animation_url":null,"youtube_url":null,"current_status":null}"#;
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "Ahl64fqr8orUI+Mli/h179wRnF8YRkde/GgUa2dR5vJ1gsT1Ay72wP1Q9DrS/xhjZazDNsUkuHCU1oM2s+LFhA==".to_string(),
        attributes:String::from( json_string),
        buy_metadata: buy_msg.clone(),
    });

    //good signature,  token #2
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);

    let contract_response = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_response {
        Ok(_resp) => {}
        _ => {
            assert!(false, "Unexpected Error. token#2 {:?}", err)
        }
    }
    let buy_msg = BuyExtension {
        male_name: "Evan Green".to_string(),
        female_name: "Agatha Tokra".to_string(),
    };

    //let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();

    let json_string = r#"{"token_uri":"https://www.merriam-webster.com/dictionary/token3","image":"XYZ","image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"female"},{"display_type":null,"trait_type":"name","value":"James T. Kirk"}],"background_color":null,"animation_url":null,"youtube_url":null,"current_status":null}"#;

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        // signature is supposed to be generated from account sending the message + extension
        signature: "QaCk+WUonQMwN/UtuKsHKogEhA5pUheCD4z0hZJ16n4HmAjCd/kuSSmsCGOsaCWfyCN6nVQPSRLr2dGDePClrA==".to_string(),
        attributes: String::from(json_string),
        buy_metadata: buy_msg.clone(),
    });
    //good signature, the token_id not so much.
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Ok(_resp) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    let buy_msg = BuyExtension {
        male_name: "Adam Smith".to_string(),
        female_name: "Natalie Parker".to_string(),
    };

    //    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();

    let json_string = r#"{"token_uri":"https://www.merriam-webster.com/dictionary/token4","image":"A123","image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"female"},{"display_type":null,"trait_type":"name","value":"James T. Kirk"}],"background_color":null,"animation_url":null,"youtube_url":null}"#;

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "3/18ROI5T4oFpy+2bIps5QFBVWwBPIqq+ZNgBmGbqnMFuyeGnftjk5QCX+akUPSHmYnr2AX2nnBOr4WDbkOduA==".to_string(),
        attributes: String::from(json_string),
        buy_metadata: buy_msg.clone(),
    });

    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    match err {
        ContractError::MaxIssued {} => {}
        _ => {
            assert!(
                false,
                "Unexpected Error. We've claimed more than we have {:?}",
                err
            )
        }
    }
    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    println!("{}", tokens.tokens.join(","));
    assert_eq!(3, tokens.tokens.len());

    assert_eq!(
        true,
        tokens
            .tokens
            .iter()
            .find(|x| x.clone() == &String::from("James Dean"))
            .is_some()
    );
}

#[test]
fn set_mint_amount() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let attributes: Vec<Trait> = vec![
        Trait {
            display_type: None,
            trait_type: "gender".to_string(),
            value: "male".to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "name".to_string(),
            value: "Jim Morrisson".to_string(),
        },
    ];
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "fJCPvlqjmTZxuKT1Uu8FqEmEuvEfNcKv3rdZ267qVFpU3S7AzIXG7ic1d9NxfUdGBdRpwW/yfdoN7nM34dW8LQ==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });
    //good signature
    let random = mock_info("random", &[Coin::new(2_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Err(ContractError::Funds { .. }) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {
            assert!(false, "Contract should not have worked")
        }
    }
    let mint_price_msg = ExecuteMsg::<Extension>::SetMintAmount {
        mint_amount: 2_000_000,
    };
    let random = mock_info("random", &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_price_msg.clone());
    match contract_exec {
        Err(ContractError::Unauthorized { .. }) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {
            assert!(false, "Contract should not have worked")
        }
    }

    let random = mock_info(MINTER, &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_price_msg.clone());
    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {}
    }
    let random = mock_info("random", &[Coin::new(1_999_999u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());
    match contract_exec {
        Err(ContractError::Funds { .. }) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {
            assert!(false, "Contract should not have worked")
        }
    }
    let random = mock_info("random", &[Coin::new(2_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());
    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {}
    }
}

#[test]
fn set_public_key() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let attributes: Vec<Trait> = vec![
        Trait {
            display_type: None,
            trait_type: "gender".to_string(),
            value: "male".to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "name".to_string(),
            value: "Jim Morrisson".to_string(),
        },
    ];
    let mut extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "fJCPvlqjmTZxuKT1Uu8FqEmEuvEfNcKv3rdZ267qVFpU3S7AzIXG7ic1d9NxfUdGBdRpwW/yfdoN7nM34dW8LQ==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });
    //good signature
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {}
    }
    // Phrase words - fiction artefact enjoy bicycle agent jungle another mesh item slam voice motion reflect code jewel tunnel glory hobby access that asthma ethics volcano cargo
    // Public Key: AqNQdMoVoy8Ub5/sh2q6UYk1Di1BTpm7hoL83wQe0nZL
    // signature: gMyokP8J9N51ouzcq8nZ6SAR6zWYrXWqo1jtzFcrL718sxvFDKyOp2uqqNxSeitbiEU7jpj7To1rdDxVZPV2IA==
    let set_pubkey_msg = ExecuteMsg::<Extension>::SetPublicKey {
        public_key: "AqNQdMoVoy8Ub5/sh2q6UYk1Di1BTpm7hoL83wQe0nZL".to_string(),
    };
    let random = mock_info("random", &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, set_pubkey_msg.clone());
    match contract_exec {
        Err(ContractError::Unauthorized { .. }) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {
            assert!(false, "Contract should not have worked")
        }
    }

    let random = mock_info(MINTER, &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, set_pubkey_msg.clone());
    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {}
    }

    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());
    match contract_exec {
        Err(ContractError::BadSignature {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {
            assert!(false, "Should have failed")
        }
    }
    extension.token_uri = "https://example.com".to_string();
    extension.image = Some("QWESDSD".to_string());

    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();
    println!("{}", json_string);

    // this fails as we actually 'bought' the NFT in the first message. but it gets past the signature check so it works
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "gJrUgYcsF4M0VkWxTc1r/HtYK3q7FYKxDcGrCyAlY6U8bogSqAidoV2RRWfIkNsqKTIdP6+SbBxFXoafBf2PSg==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });

    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());
    match contract_exec {
        Err(ContractError::Claimed {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {}
    }
}

#[test]
fn set_status() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let attributes: Vec<Trait> = vec![
        Trait {
            display_type: None,
            trait_type: "gender".to_string(),
            value: "male".to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "name".to_string(),
            value: "Jim Morrisson".to_string(),
        },
    ];
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "fJCPvlqjmTZxuKT1Uu8FqEmEuvEfNcKv3rdZ267qVFpU3S7AzIXG7ic1d9NxfUdGBdRpwW/yfdoN7nM34dW8LQ==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });
    //good signature
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(t) => {
            let token_id_opt = t
                .attributes
                .iter()
                .find(|t| t.key == "token_id")
                .map(|f| f.value.clone());
            if let Some(token_id) = token_id_opt {
                assert_eq!(token_id, "James Dean");

                let res = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
                let status = res.extension.get_status();
                assert!(status.is_some(), "Should have seen a status");
                assert_eq!(status.unwrap(), "Alive and curious");
                let set_status_msg = ExecuteMsg::<Extension>::SetTokenStatus {
                    status: "Peeping the life fantastic".to_string(),
                    token_id: token_id.clone(),
                };
                let random = mock_info("random", &[]);
                let contract_exec =
                    contract.execute(deps.as_mut(), mock_env(), random, set_status_msg.clone());
                match contract_exec {
                    Err(err) => {
                        assert!(false, "Unexpected Error {:?}", err)
                    }
                    _ => {}
                }
                let res = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
                let status = res.extension.get_status();
                assert!(status.is_some(), "Should have seen a status");
                assert_eq!(status.unwrap(), "Peeping the life fantastic");
                let random2 = mock_info("random2", &[]);
                let set_status_msg2 = ExecuteMsg::<Extension>::SetTokenStatus {
                    status: "Failing the life fantastic".to_string(),
                    token_id: token_id.clone(),
                };
                let contract_exec =
                    contract.execute(deps.as_mut(), mock_env(), random2, set_status_msg2.clone());
                match contract_exec {
                    Err(ContractError::Unauthorized {}) => {}
                    Err(err) => {
                        assert!(false, "Unexpected Error {:?}", err)
                    }
                    _ => {
                        assert!(false, "Contract should not have worked")
                    }
                }
                let res = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
                let status = res.extension.get_status();
                assert!(status.is_some(), "Should have seen a status");
                assert_eq!(status.unwrap(), "Peeping the life fantastic");
                let admin_msg = mock_info(MINTER, &[]);
                let set_status_msg2 = ExecuteMsg::<Extension>::SetTokenStatus {
                    status: "Admin doing the override".to_string(),
                    token_id: token_id.clone(),
                };

                let contract_exec = contract.execute(
                    deps.as_mut(),
                    mock_env(),
                    admin_msg,
                    set_status_msg2.clone(),
                );
                match contract_exec {
                    Err(ContractError::Unauthorized {}) => {}
                    Err(err) => {
                        assert!(false, "Unexpected Error {:?}", err)
                    }
                    _ => {
                        assert!(false, "Contract should not have worked")
                    }
                }
                let res = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
                let status = res.extension.get_status();
                assert!(status.is_some(), "Should have seen a status");
                assert_eq!(status.unwrap(), "Peeping the life fantastic");
            } else {
                assert!(false, "Token ID not found")
            }
        }
    }
}

#[test]
fn set_nft_contract_info() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    let nft_info_msg = ExecuteMsg::SetNftContractInfo {
        description: None,
        src: None,
        banner_src: Some(String::from("URL to banner")),
        twitter: Some("@twitter".to_string()),
        github: None,
        discord: None,
        telegram: None,
        listing: vec![
            NftListing {
                label: "XYZ".to_string(),
                listing_uri: "SomeURL".to_string(),
            },
            NftListing {
                label: "ABC".to_string(),
                listing_uri: "Some Other URL".to_string(),
            },
        ],
    };
    let random = mock_info("random", &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, nft_info_msg.clone());

    match contract_exec {
        Err(ContractError::Unauthorized {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            assert!(false, "Should not have worked");
        }
    }
    let random = mock_info(MINTER, &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, nft_info_msg.clone());
    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            let res = contract.nft_contract_info(&deps.storage).unwrap();
            assert_eq!(res.description, None);
            assert_eq!(res.listing.len(), 2);
            assert_eq!(res.listing[0].label, "XYZ");
            assert_eq!(res.listing[0].listing_uri, "SomeURL");
            assert_eq!(res.twitter, Some("@twitter".to_string()));
        }
    }
}

#[test]
fn set_nft_contract_keybase_verification() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    let nft_keybase_msg = ExecuteMsg::SetNftContractKeybaseVerification {
        message: "This can really be anything. We don't verify it. but the aim is for the NFT owner to use keybase to sign it so others can verify the owner".to_string(),
    };
    let random = mock_info("random", &[]);
    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, nft_keybase_msg.clone());

    match contract_exec {
        Err(ContractError::Unauthorized {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            assert!(false, "Should not have worked");
        }
    }
    let random = mock_info(MINTER, &[]);
    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, nft_keybase_msg.clone());
    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            let res = contract
                .nft_contract_keybase_verification(&deps.storage)
                .unwrap();
            assert_eq!(res.is_some(), true);
        }
    }
}

#[test]
fn set_nft_trait_map() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    let nft_trait_map_msg = ExecuteMsg::SetNftContractTraitInfo {
        trait_map: vec![
            (
                "Attribute1".to_string(),
                vec![
                    NftTraitSummary {
                        label: "A".to_string(),
                        value: Decimal::from_str("0.90").unwrap(),
                    },
                    NftTraitSummary {
                        label: "B".to_string(),
                        value: Decimal::from_str("0.10").unwrap(),
                    },
                ],
            ),
            (
                "Attribute2".to_string(),
                vec![
                    NftTraitSummary {
                        label: "m".to_string(),
                        value: Decimal::from_str("0.40").unwrap(),
                    },
                    NftTraitSummary {
                        label: "n".to_string(),
                        value: Decimal::from_str("0.10").unwrap(),
                    },
                    NftTraitSummary {
                        label: "0".to_string(),
                        value: Decimal::from_str("0.10").unwrap(),
                    },
                ],
            ),
        ],
    };
    let json_to_match = r#"[["Attribute1",[{"label":"A","value":"0.9"},{"label":"B","value":"0.1"}]],["Attribute2",[{"label":"m","value":"0.4"},{"label":"n","value":"0.1"},{"label":"0","value":"0.1"}]]]"#;

    let random = mock_info("random", &[]);
    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, nft_trait_map_msg.clone());

    match contract_exec {
        Err(ContractError::Unauthorized {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            assert!(false, "Should not have worked");
        }
    }
    let random = mock_info(MINTER, &[]);
    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, nft_trait_map_msg.clone());
    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            let res = contract.nft_contract_trait_map(&deps.storage).unwrap();
            let json = serde_json_wasm::to_string(&res).unwrap();
            assert_eq!(json, json_to_match);
        }
    }
}

#[test]
fn do_sweep() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    let sweep_msg = ExecuteMsg::Sweep {
        denom: "uluna".to_string(),
    };
    let random = mock_info("random", &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, sweep_msg.clone());
    match contract_exec {
        Err(ContractError::Unauthorized {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            assert!(false, "Unexpected OK")
        }
    }

    let minter = mock_info(MINTER, &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), minter, sweep_msg.clone());
    match contract_exec {
        Err(ContractError::NoFunds {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            assert!(false, "Unexpected OK")
        }
    }

    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let attributes: Vec<Trait> = vec![
        Trait {
            display_type: None,
            trait_type: "gender".to_string(),
            value: "male".to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "name".to_string(),
            value: "Jim Morrisson".to_string(),
        },
    ];
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "fJCPvlqjmTZxuKT1Uu8FqEmEuvEfNcKv3rdZ267qVFpU3S7AzIXG7ic1d9NxfUdGBdRpwW/yfdoN7nM34dW8LQ==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });
    //good signature
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {}
    }
    let sweep_msg = ExecuteMsg::Sweep {
        denom: "uusd".to_string(),
    };
    let minter = mock_info(MINTER, &[]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), minter, sweep_msg.clone());
    match contract_exec {
        Err(ContractError::NoFunds {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        Ok(_) => {
            assert!(false, "Unexpected OK")
        }
    }
}

#[test]
fn change_name() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let attributes: Vec<Trait> = vec![
        Trait {
            display_type: None,
            trait_type: "gender".to_string(),
            value: "male".to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "name".to_string(),
            value: "Jim Morrisson".to_string(),
        },
    ];
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: Some("QmRdTSx3v2VFw1xDN8vrYFLdFqjUn8XH8ibaGnsS1GT2mQ".to_string()),
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "fJCPvlqjmTZxuKT1Uu8FqEmEuvEfNcKv3rdZ267qVFpU3S7AzIXG7ic1d9NxfUdGBdRpwW/yfdoN7nM34dW8LQ==".to_string(),
        attributes: json_string.clone(),
        buy_metadata: buy_msg.clone(),
    });

    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
        _ => {}
    }

    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();

    assert_eq!(tokens.tokens.len(), 1);
    let buy_msg = BuyExtension {
        male_name: "Peter Walker".to_string(),
        female_name: "Lady Ga Ga".to_string(),
    };

    let json_string = r#"{"token_uri":"https://www.merriam-webster.com/dictionary/token2","image":"QASD","image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"male"},{"display_type":null,"trait_type":"name","value":"James T. Kirk"}],"background_color":null,"animation_url":null,"youtube_url":null,"current_status":null}"#;
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "Ahl64fqr8orUI+Mli/h179wRnF8YRkde/GgUa2dR5vJ1gsT1Ay72wP1Q9DrS/xhjZazDNsUkuHCU1oM2s+LFhA==".to_string(),
        attributes:String::from( json_string),
        buy_metadata: buy_msg.clone(),
    });

    //good signature,  token #2
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);

    let contract_response = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_response {
        Ok(_resp) => {}
        Err(err) => {
            assert!(false, "Unexpected Error. token#2 {:?}", err)
        }
    }
    let buy_msg = BuyExtension {
        male_name: "Evan Green".to_string(),
        female_name: "Agatha Tokra".to_string(),
    };

    //let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();

    let json_string = r#"{"token_uri":"https://www.merriam-webster.com/dictionary/token3","image":"XYZ","image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"female"},{"display_type":null,"trait_type":"name","value":"James T. Kirk"}],"background_color":null,"animation_url":null,"youtube_url":null,"current_status":null}"#;

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        // signature is supposed to be generated from account sending the message + extension
        signature: "QaCk+WUonQMwN/UtuKsHKogEhA5pUheCD4z0hZJ16n4HmAjCd/kuSSmsCGOsaCWfyCN6nVQPSRLr2dGDePClrA==".to_string(),
        attributes: String::from(json_string),
        buy_metadata: buy_msg.clone(),
    });
    //good signature, the token_id not so much.
    let random = mock_info("random", &[Coin::new(3_000_000u128, "uluna")]);
    let contract_exec = contract.execute(deps.as_mut(), mock_env(), random, mint_msg.clone());

    match contract_exec {
        Ok(_resp) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }

    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: Some("This is a strange description".to_string()),
        name: None,
        token_id: "James Dean".to_string(),
    };
    let random = mock_info("random", &[]);
    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_) => {
            match contract.nft_info(deps.as_ref(), "James Dean".to_string()) {
                Ok(nft_info) => {
                    assert_eq!(
                        nft_info.extension.name.unwrap_or_default(),
                        "James Dean".to_string()
                    );
                    assert_eq!(
                        nft_info.extension.description.unwrap_or_default(),
                        "This is a strange description".to_string()
                    );
                }
                Err(err) => {
                    assert!(false, "Unexpected Error {:?}", err)
                }
            }
            match contract.change_dynamics.load(&deps.storage, "James Dean") {
                Ok(t) => assert_eq!(t.change_count, 1),
                Err(e) => assert!(false, "Unexpected Error {:?}", e),
            }
        }
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }

    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: Some("This is a strange description as well".to_string()),
        name: Some("James Q Kirk".to_string()),
        token_id: "James Dean".to_string(),
    };
    let random = mock_info("random", &[]);
    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_resp) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }

    match contract.nft_info(deps.as_ref(), "James Q Kirk".to_string()) {
        Ok(c) => {
            assert_eq!(c.extension.get_name().unwrap_or_default(), "James Q Kirk");
            assert_eq!(
                c.extension.description.unwrap_or_default(),
                "This is a strange description as well"
            );
        }
        Err(e) => {
            assert!(false, "Unexpected Error {:?}", e)
        }
    }
    match contract.change_dynamics.load(&deps.storage, "James Q Kirk") {
        Ok(t) => assert_eq!(t.change_count, 2),
        Err(e) => assert!(false, "Unexpected Error {:?}", e),
    }

    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: Some("This is a strange description as well".to_string()),
        name: Some("James Q Kirk".to_string()),
        token_id: "James Dean".to_string(),
    };
    let random = mock_info("random", &[]);

    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_resp) => {
            assert!(false, "should not be able to be found")
        }
        Err(ContractError::Std(x)) => match x {
            StdError::NotFound { .. } => {}
            _ => {
                assert!(false, "Unexpected Error {:?}", x)
            }
        },
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }

    match contract.nft_info(deps.as_ref(), "James Dean".to_string()) {
        Ok(_) => {
            assert!(false, "Should have failed")
        }
        Err(StdError::NotFound { .. }) => {}
        Err(e) => {
            assert!(false, "Unexpected Error {:?}", e)
        }
    }
    // should be able to rename to now unused name
    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: None,
        name: Some("James Dean".to_string()),
        token_id: "Agatha Tokra".to_string(),
    };
    let random = mock_info("random", &[]);
    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_resp) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: Some("The Crystal Method".to_string()),
        name: None,
        token_id: "James Q Kirk".to_string(),
    };
    let random = mock_info("random", &[]);

    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_resp) => {
            assert!(false, "should have asked for $$")
        }
        Err(ContractError::Funds {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: Some("Welcome to the Jungle".to_string()),
        name: None,
        token_id: "James Q Kirk".to_string(),
    };
    let random = mock_info("random", &[Coin::new(1_000_000, "uusd")]);

    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_resp) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: Some("Run with the Wolves".to_string()),
        name: None,
        token_id: "James Q Kirk".to_string(),
    };
    let random = mock_info("random", &[Coin::new(1_000_000, "uusd")]);

    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_resp) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    // this one should fail, as it now costs $2
    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: Some("Highway to Hell".to_string()),
        name: None,
        token_id: "James Q Kirk".to_string(),
    };

    let random = mock_info("random", &[Coin::new(1_000_000, "uusd")]);

    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_resp) => {
            assert!(false, "should have asked for more $$")
        }
        Err(ContractError::Funds {}) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }
    // ensure the calc works
    let change_name_msg = ExecuteMsg::SetTokenNameDescription {
        description: Some("Another one bites the dust".to_string()),
        name: None,
        token_id: "James Q Kirk".to_string(),
    };

    let random = mock_info("random", &[Coin::new(2_000_000, "uusd")]);

    let contract_exec =
        contract.execute(deps.as_mut(), mock_env(), random, change_name_msg.clone());
    match contract_exec {
        Ok(_resp) => {}
        Err(err) => {
            assert!(false, "Unexpected Error {:?}", err)
        }
    }

    match contract.nft_info(deps.as_ref(), "James Q Kirk".to_string()) {
        Ok(x) => {
            assert_eq!(
                x.extension.description.unwrap_or_default(),
                "Another one bites the dust"
            )
        }
        Err(e) => {
            assert!(false, "Unexpected Error {:?}", e)
        }
    }

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    println!("{}", tokens.tokens.join(","));
    assert_eq!(3, tokens.tokens.len());

    assert_eq!(
        true,
        tokens
            .tokens
            .iter()
            .find(|x| x.clone() == &String::from("James Q Kirk"))
            .is_some()
    );
}
