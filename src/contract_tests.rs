#![cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Coin, CosmosMsg, DepsMut, Empty, Response, WasmMsg};

use cw721::{
    ApprovedForAllResponse, ContractInfoResponse, Cw721Query, Cw721ReceiveMsg, Expiration,
    NftInfoResponse, OwnerOfResponse,
};

use crate::extension::{Metadata, Trait};
use crate::msg::BuyMsg;
use crate::{
    BuyExtension, ContractError, Cw721Contract, ExecuteMsg, Extension, InstantiateMsg, MintMsg,
    QueryMsg,
};

const MINTER: &str = "merlin";
const CONTRACT_NAME: &str = "Magic Power";
const SYMBOL: &str = "MGK";
const PUBLIC_KEY: &str = "AiMzHaA2bvnDXfHzkjMM+vkSE/p0ymBtAFKUnUtQAeXe";

fn setup_contract(deps: DepsMut<'_>) -> Cw721Contract<'static, Extension, Empty> {
    let contract = Cw721Contract::default();
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        minter: String::from(MINTER),
        public_key: String::from(PUBLIC_KEY),
        mint_amount: 3_000_000u64,
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
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
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
        token_uri: None,
        extension: extension.clone(),
    });

    let allowed = mock_info(MINTER, &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg2)
        .unwrap_err();
    match err {
        ContractError::Claimed { .. } => {}
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
fn transferring_nft() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "melt".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/melt".to_string();
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
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
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap();

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
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
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
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
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
    let extension = Metadata {
        token_uri: token_uri1.clone(),
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };
    let mint_msg1 = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: String::from("demeter"),
        token_uri: Some(token_uri1),
        extension: extension.clone(),
    });

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
    let extension = Metadata {
        token_uri: token_id1.clone(),
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: demeter.clone(),
        token_uri: None,
        extension: extension.clone(),
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: ceres.clone(),
        token_uri: None,
        extension: extension.clone(),
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id3.clone(),
        owner: demeter.clone(),
        token_uri: None,
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
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
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
        signature: "/+vT0JBx74/qKwMjuhKzoe80IvYSneamPtkE2anJEPstJjltkO4pz1k+m4wwc8QmsqB3Szp+RLJR2VbE66pnSg==".to_string(),
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
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();
    //println!("json_string:\n{}", json_string);
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "/+vT0JBx74/qKwMjuhKzoe80IvYSneamPtkE2anJEPstJjltkO4pz1k+m4wwc8QmsqB3Szp+RLJR2VbE66pnSg==".to_string(),
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
        signature: "/+vT0JBx74/qKwMjuhKzoe80IvYSneamPtkE2anJEPstJjltkO4pz1k+m4wwc8QmsqB3Szp+RLJR2VbE66pnSg==".to_string(),
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

    // eprintln!("#2 - should only be 1: {}", tokens.tokens.join(","));
    assert_eq!(tokens.tokens.len(), 1);
    let buy_msg = BuyExtension {
        male_name: "Peter Walker".to_string(),
        female_name: "Lady Ga Ga".to_string(),
    };

    let json_string = r#"{"token_uri":"https://www.merriam-webster.com/dictionary/token2","image":null,"image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"male"},{"display_type":null,"trait_type":"name","value":"James T. Kirk"}],"background_color":null,"animation_url":null,"youtube_url":null}"#;
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "DZGWXyGtJq53sv1haH15sdi2xJP3m9FuD5hTYaBAdldoMD1H3wMuqWIzaE/KoMsXe7i3e/lW+54uKm8iV5lPlA==".to_string(),
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

    let json_string = r#"{"token_uri":"https://www.merriam-webster.com/dictionary/token3","image":null,"image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"female"},{"display_type":null,"trait_type":"name","value":"James T. Kirk"}],"background_color":null,"animation_url":null,"youtube_url":null}"#;

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        // signature is supposed to be generated from account sending the message + extension
        signature: "PV0jYV0ddgM/h3zsnarH3/471GntTbGpqcv1dO5qYMkgAI/Vh6faK+x5mRXi89R7xC78azguIxmQdBV6ArZHpQ==".to_string(),
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

    let json_string = r#"{"token_uri":"https://www.merriam-webster.com/dictionary/token4","image":null,"image_data":null,"external_url":null,"description":null,"name":null,"attributes":[{"display_type":null,"trait_type":"gender","value":"female"},{"display_type":null,"trait_type":"name","value":"James T. Kirk"}],"background_color":null,"animation_url":null,"youtube_url":null}"#;

    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "a2jc8HamP06+S3sYfD7TluUQ8kjkOvHiHtEh+75tKCY9GcCEkUWFLRg9hpMiQQBem3PqVS2RK8fHkTl/poiiEA==".to_string(),
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
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "/+vT0JBx74/qKwMjuhKzoe80IvYSneamPtkE2anJEPstJjltkO4pz1k+m4wwc8QmsqB3Szp+RLJR2VbE66pnSg==".to_string(),
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
    let extension = Metadata {
        token_uri: token_uri.clone(),
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(attributes),
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };
    let buy_msg = BuyExtension {
        male_name: "James Dean".to_string(),
        female_name: "Norma Rae".to_string(),
    };
    let json = serde_json_wasm::to_string(&extension);
    assert!(json.is_ok(), "JSON unpacking failed");

    let json_string = json.unwrap();
    println!("{}", json_string);
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "/+vT0JBx74/qKwMjuhKzoe80IvYSneamPtkE2anJEPstJjltkO4pz1k+m4wwc8QmsqB3Szp+RLJR2VbE66pnSg==".to_string(),
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
    // signature: btGp+2hXDFeiosEJsw2kp5r2KYlvyNcpQE6YsNh/Ln5wbU3ZkEN4szt7j75dR35pxD+RyBK1qIU4vDLMtjG8Yw==
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
    // this fails as we actually 'bought' the NFT in the first message. but it gets past the signature check so it works
    let mint_msg = ExecuteMsg::<Extension>::Buy(BuyMsg {
        signature: "btGp+2hXDFeiosEJsw2kp5r2KYlvyNcpQE6YsNh/Ln5wbU3ZkEN4szt7j75dR35pxD+RyBK1qIU4vDLMtjG8Yw==".to_string(),
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
