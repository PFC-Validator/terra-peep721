use cosmwasm_std::{
    BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, QuerierWrapper,
    Response, StdError, StdResult, Uint128,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::ContractError;
use crate::extension::{MetaDataPersonalization, MetaPersonalize};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse, CustomMsg, Cw721Execute, Cw721ReceiveMsg, Expiration};
use terra_cosmwasm::TerraQuerier;
use terraswap::querier::query_balance;

use crate::msg::{BuyMsg, ExecuteMsg, InstantiateMsg, MintMsg};
use crate::state::{
    Approval, ChangeDynamics, Cw721Contract, NftListing, NftTraitSummary, TokenInfo,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-peep721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Length of a serialized compressed public key
const ECDSA_COMPRESSED_PUBKEY_LEN: usize = 33;
/// Length of a serialized uncompressed public key
const ECDSA_UNCOMPRESSED_PUBKEY_LEN: usize = 65;
static DECIMAL_FRACTION: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

impl<'a, T, C> Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone + MetaDataPersonalization,
    C: CustomMsg,
{
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response<C>> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let info = ContractInfoResponse {
            name: msg.name,
            symbol: msg.symbol,
        };
        let public_key = base64::decode(&msg.public_key).unwrap();

        #[cfg(not(feature = "backtraces"))]
        Self::check_pubkey(&public_key).map_err(|e| cosmwasm_std::StdError::ParseErr {
            target_type: "public key".to_string(),
            msg: format!("Parsing Public Key: {:?}", &e),
        })?;

        #[cfg(feature = "backtraces")]
        Self::check_pubkey(&public_key).map_err(|e| cosmwasm_std::StdError::ParseErr {
            target_type: "public key".to_string(),
            msg: format!("Parsing Public Key: {:?}", &e),
            backtrace: Default::default(),
        })?;
        self.contract_info.save(deps.storage, &info)?;
        let minter = deps.api.addr_validate(&msg.minter)?;
        self.minter.save(deps.storage, &minter)?;
        self.public_key.save(deps.storage, &msg.public_key)?;
        self.mint_amount.save(deps.storage, &msg.mint_amount)?;
        self.change_amount.save(deps.storage, &msg.change_amount)?;
        self.change_multiplier
            .save(deps.storage, &msg.change_multiplier)?;
        self.max_issuance.save(deps.storage, &msg.max_issuance)?;
        Ok(Response::default())
    }
    fn check_pubkey(data: &[u8]) -> Result<(), ContractError> {
        let ok = match data.first() {
            Some(0x02) | Some(0x03) => data.len() == ECDSA_COMPRESSED_PUBKEY_LEN,
            Some(0x04) => data.len() == ECDSA_UNCOMPRESSED_PUBKEY_LEN,
            _ => false,
        };
        if ok {
            Ok(())
        } else {
            Err(ContractError::InvalidSecp256k1PubkeyFormat {})
        }
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<T>,
    ) -> Result<Response<C>, ContractError> {
        match msg {
            ExecuteMsg::Mint(msg) => self.mint(deps, env, info, msg),
            ExecuteMsg::Burn { token_id } => self.burn(deps, env, info, token_id),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => self.approve(deps, env, info, spender, token_id, expires),
            ExecuteMsg::Revoke { spender, token_id } => {
                self.revoke(deps, env, info, spender, token_id)
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                self.approve_all(deps, env, info, operator, expires)
            }
            ExecuteMsg::RevokeAll { operator } => self.revoke_all(deps, env, info, operator),
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => self.transfer_nft(deps, env, info, recipient, token_id),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => self.send_nft(deps, env, info, contract, token_id, msg),
            ExecuteMsg::Buy(msg) => self.buy(deps, env, info, msg),
            ExecuteMsg::SetPublicKey { public_key } => {
                self.set_public_key(deps, env, info, public_key)
            }

            ExecuteMsg::SetMintAmount { mint_amount } => {
                self.set_mint_amount(deps, env, info, mint_amount)
            }
            ExecuteMsg::SetChangeAmount { change_amount } => {
                self.set_change_amount(deps, env, info, change_amount)
            }
            ExecuteMsg::SetChangeTimesMultiplier { change_multiplier } => {
                self.set_change_multiplier(deps, env, info, change_multiplier)
            }
            ExecuteMsg::SetImagePrefix { prefix } => self.set_image_prefix(deps, env, info, prefix),
            ExecuteMsg::SetTokenStatus { token_id, status } => {
                self.set_status(deps, env, info, token_id, status)
            }
            ExecuteMsg::SetTokenNameDescription {
                description,
                name,
                token_id,
            } => self.set_name_description(deps, env, info, token_id, name, description),
            ExecuteMsg::SetNftContractInfo {
                description,
                src,
                banner_src,
                twitter,
                github,
                discord,
                telegram,
                listing,
            } => self.set_nft_contract_info(
                deps,
                env,
                info,
                description,
                src,
                banner_src,
                twitter,
                github,
                discord,
                telegram,
                listing,
            ),
            ExecuteMsg::SetNftContractTraitInfo { trait_map } => {
                self.set_nft_trait_map(deps, env, info, trait_map)
            }
            ExecuteMsg::SetNftContractKeybaseVerification { message } => {
                self.set_nft_keybase_verification(deps, env, info, message)
            }
            ExecuteMsg::Sweep { denom } => self.sweep(deps, env, info, denom),
        }
    }
}

// TODO pull this into some sort of trait extension??
impl<'a, T, C> Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone + MetaDataPersonalization,
    C: CustomMsg,
{
    pub fn mint(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: MintMsg<T>,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;
        let max_issuance = self.max_issuance.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        let count = self.token_count(deps.storage)?;
        if count >= max_issuance {
            return Err(ContractError::MaxIssued {});
        }

        // create the token
        let token = TokenInfo {
            owner: deps.api.addr_validate(&msg.owner)?,
            approvals: vec![],
            token_uri: msg.token_uri.clone(),
            extension: msg.extension.clone(), /*
                                              change_count: 0,
                                              unique_owners: vec![],
                                              transfer_count: 0,
                                              block_number: 0,
                                              price_ceiling: Default::default()
                                              */
        };
        if let Some(token_uri) = msg.token_uri.clone() {
            if let Ok(_x) = self.tokens_uri.load(deps.storage, &token_uri) {
                return Err(ContractError::Claimed {});
            }
        } else {
            return Err(ContractError::TokenMissing {});
        }
        if let Some(image_uri) = msg.extension.get_image_raw() {
            if let Ok(_x) = self.image_uri.load(deps.storage, &image_uri) {
                return Err(ContractError::ImageClaimed {});
            }
        } else {
            return Err(ContractError::ImageMissing {});
        }

        self.tokens
            .update(deps.storage, &msg.token_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(token),
            })?;
        if let Some(token_uri) = msg.token_uri.clone() {
            self.tokens_uri
                .update(deps.storage, &token_uri, |old| match old {
                    Some(_) => Err(ContractError::Claimed {}),
                    None => Ok(token_uri.clone()),
                })?;
        }
        if let Some(image_uri) = msg.extension.get_image_raw() {
            self.image_uri
                .update(deps.storage, &image_uri, |old| match old {
                    Some(_) => Err(ContractError::ImageClaimed {}),
                    None => Ok(msg.token_id.clone()),
                })?;
        }

        self.increment_tokens(deps.storage)?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("token_id", msg.token_id))
    }
    fn burn(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        let token = self.tokens.load(deps.storage, &token_id)?;
        self.check_can_send(deps.as_ref(), &env, &info, &token)?;

        self.tokens.remove(deps.storage, &token_id)?;
        if let Some(image) = &token.extension.get_image_raw() {
            self.image_uri.remove(deps.storage, image)?;
        }
        if let Some(token_uri) = &token.token_uri {
            self.tokens_uri.remove(deps.storage, token_uri)?;
        }

        self.decrement_tokens(deps.storage)?;
        let total = self.max_issuance.load(deps.storage)?;
        if total > 0 {
            let new_total = total - 1;
            self.max_issuance.save(deps.storage, &new_total)?;
        }

        Ok(Response::new()
            .add_attribute("action", "burn")
            .add_attribute("sender", info.sender)
            .add_attribute("token_id", token_id))
    }
}

// TODO pull this into some sort of trait extension??
impl<'a, T, C> Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone + MetaDataPersonalization,
    C: CustomMsg,
{
    pub fn buy(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: BuyMsg, //<T>,
    ) -> Result<Response<C>, ContractError> {
        // TODO
        // set amount & public sig on init/admin
        //   let _minter = self.minter.load(deps.storage)?;
        let max_issuance = self.max_issuance.load(deps.storage)?;
        let count = self.token_count(deps.storage)?;
        if count >= max_issuance {
            return Err(ContractError::MaxIssued {});
        }
        let public_key_str = self.public_key.load(deps.storage)?;
        let public_key = base64::decode(&public_key_str).unwrap();
        Self::check_pubkey(&public_key)?;

        let minimum_amount = self.mint_amount.load(deps.storage)?;
        if let Some(coins) = info.funds.first() {
            if coins.denom != "uluna" || coins.amount < Uint128::from(minimum_amount) {
                return Err(ContractError::Funds {});
            }
        } else {
            return Err(ContractError::Funds {});
        }
        let hash_message = format!("{}/{}", info.sender, msg.attributes);
        //println!("{}", hash_message);
        let hash = Sha256::digest(hash_message.as_bytes());

        let signature = base64::decode(&msg.signature).unwrap();

        let result = deps
            .api
            .secp256k1_verify(&hash, &signature, public_key.as_ref())?;

        if result {
            let mut extension_copy: T = serde_json_wasm::from_str(&msg.attributes)?;
            let token_uri = extension_copy.get_token_uri();
            if let Some(token_id) = msg.buy_metadata.perform_mint(&mut extension_copy) {
                extension_copy.set_status("Alive and curious");
                // create the token
                let token = TokenInfo {
                    owner: info.sender.clone(),
                    approvals: vec![],
                    token_uri: Some(token_uri.clone()),
                    extension: extension_copy.clone(),
                };
                if let Ok(_x) = self.tokens_uri.load(deps.storage, &token_uri) {
                    return Err(ContractError::Claimed {});
                }
                if let Some(image_uri) = extension_copy.get_image_raw() {
                    if let Ok(_x) = self.image_uri.load(deps.storage, &image_uri) {
                        return Err(ContractError::ImageClaimed {});
                    }
                } else {
                    return Err(ContractError::ImageMissing {});
                }

                self.tokens
                    .update(deps.storage, &token_id, |old| match old {
                        Some(_) => Err(ContractError::Claimed {}),
                        None => Ok(token),
                    })?;
                self.tokens_uri
                    .update(deps.storage, &token_uri, |old| match old {
                        Some(_) => Err(ContractError::Claimed {}),
                        None => Ok(token_id.clone()),
                    })?;
                // note.. we checked this above
                let image_uri = extension_copy.get_image_raw().unwrap_or_default();
                self.image_uri
                    .update(deps.storage, &image_uri, |old| match old {
                        Some(_) => Err(ContractError::ImageClaimed {}),
                        None => Ok(token_id.clone()),
                    })?;

                self.increment_tokens(deps.storage)?;

                Ok(Response::new()
                    .add_attribute("action", "mint")
                    .add_attribute("minter", info.sender)
                    .add_attribute("token_id", token_id))
            } else {
                Err(ContractError::BadTokenId {})
            }
        } else {
            Err(ContractError::BadSignature {})
        }
    }
    pub fn set_public_key(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        public_key: String,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        self.public_key.save(deps.storage, &public_key)?;
        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("public_key", public_key))
    }

    pub fn set_mint_amount(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        mint_amount: u64,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        self.mint_amount.save(deps.storage, &mint_amount)?;
        let mint_amount_string = format!("{}", mint_amount);
        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("mint_amount", mint_amount_string))
    }

    pub fn set_change_amount(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        change_amount: u64,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        self.change_amount.save(deps.storage, &change_amount)?;
        let change_amount_string = format!("{}", change_amount);
        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("change_amount", change_amount_string))
    }

    pub fn set_change_multiplier(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        change_multiplier: u64,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        self.change_multiplier
            .save(deps.storage, &change_multiplier)?;
        let change_multiplier_string = format!("{}", change_multiplier);
        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("change_multiplier", change_multiplier_string))
    }

    pub fn set_image_prefix(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        prefix: String,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        self.image_prefix.save(deps.storage, &prefix)?;
        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("image_prefix", prefix))
    }

    pub fn set_nft_keybase_verification(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: String,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        self.keybase_message
            .save(deps.storage, &Some(message.clone()))?;
        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("nft_keybase_verification", message))
    }

    pub fn set_nft_trait_map(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        trait_map: Vec<(String, Vec<NftTraitSummary>)>,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        match serde_json_wasm::to_string(&trait_map) {
            Ok(json) => {
                self.trait_map.save(deps.storage, &trait_map.clone())?;
                Ok(Response::new()
                    .add_attribute("action", "approve")
                    .add_attribute("sender", info.sender)
                    .add_attribute("nft_trait_map", json))
            }
            Err(e) => Err(ContractError::JsonSerError(e)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_nft_contract_info(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        description: Option<String>,
        src: Option<String>,
        banner_src: Option<String>,
        twitter: Option<String>,
        github: Option<String>,
        discord: Option<String>,
        telegram: Option<String>,
        listing: Vec<NftListing>,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }
        let nft_contract_info = crate::state::NftContractInfo {
            description,
            src,
            banner_src,
            twitter,
            github,
            discord,
            telegram,
            listing,
        };
        match serde_json_wasm::to_string(&nft_contract_info) {
            Ok(json) => {
                self.nft_contract_info
                    .save(deps.storage, &nft_contract_info)?;

                Ok(Response::new()
                    .add_attribute("action", "approve")
                    .add_attribute("sender", info.sender)
                    .add_attribute("nft_contract_info", &json))
            }
            Err(e) => Err(ContractError::JsonSerError(e)),
        }
    }
    pub fn sweep(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        denom: String,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }

        let amount = query_balance(&deps.querier, env.contract.address, denom.clone())?;
        if amount.is_zero() {
            return Err(ContractError::NoFunds {});
        }
        let tax_amount =
            Self::compute_tax(&deps.querier, amount, denom.clone())? + Uint128::new(1u128);
        if tax_amount > amount {
            return Err(ContractError::FundsTooSmall {});
        }
        Ok(Response::new()
            .add_attribute("sweep", denom.clone())
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    denom,
                    amount: amount - tax_amount,
                }],
            })))
    }
    fn compute_tax(querier: &QuerierWrapper, amount: Uint128, denom: String) -> StdResult<Uint128> {
        if denom == "uluna" {
            return Ok(Uint128::zero());
        }

        let terra_querier = TerraQuerier::new(querier);
        let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
        let tax_cap: Uint128 = (terra_querier.query_tax_cap(denom)?).cap;
        Ok(std::cmp::min(
            amount.checked_sub(amount.multiply_ratio(
                DECIMAL_FRACTION,
                DECIMAL_FRACTION * tax_rate + DECIMAL_FRACTION,
            ))?,
            tax_cap,
        ))
    }
}

impl<'a, T, C> Cw721Execute<T, C> for Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone + MetaDataPersonalization,
    C: CustomMsg,
{
    type Err = ContractError;

    fn transfer_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

        Ok(Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", recipient)
            .add_attribute("token_id", token_id))
    }

    fn send_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response<C>, ContractError> {
        // Transfer token
        self._transfer_nft(deps, &env, &info, &contract, &token_id)?;

        let send = Cw721ReceiveMsg {
            sender: info.sender.to_string(),
            token_id: token_id.clone(),
            msg,
        };

        // Send message
        Ok(Response::new()
            .add_message(send.into_cosmos_msg(contract.clone())?)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", contract)
            .add_attribute("token_id", token_id))
    }

    fn approve(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn revoke(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

        Ok(Response::new()
            .add_attribute("action", "revoke")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn approve_all(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        operator: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        // set the operator for us
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn revoke_all(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: String,
    ) -> Result<Response<C>, ContractError> {
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .remove(deps.storage, (&info.sender, &operator_addr));

        Ok(Response::new()
            .add_attribute("action", "revoke_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }
}

// helpers
impl<'a, T, C> Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone + MetaDataPersonalization,
    C: CustomMsg,
{
    pub fn _transfer_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        recipient: &str,
        token_id: &str,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        let old_owner = token.owner.clone();
        // ensure we have permissions
        self.check_can_send(deps.as_ref(), env, info, &token)?;
        // set owner and remove existing approvals
        token.owner = deps.api.addr_validate(recipient)?;
        token.approvals = vec![];
        self.tokens.save(deps.storage, token_id, &token)?;
        let mut change_dynamics = match self.change_dynamics.load(deps.storage, token_id) {
            Ok(c) => c,

            Err(e) => match e {
                StdError::NotFound { .. } => ChangeDynamics {
                    owner: old_owner.clone(),
                    token_id: token_id.to_string(),
                    change_count: 0,
                    unique_owners: vec![old_owner],
                    transfer_count: 0,
                    block_number: 0,
                    price_ceiling: Default::default(),
                },
                _ => return Err(e.into()),
            },
        };
        change_dynamics.transfer_count += 1;
        if !change_dynamics.unique_owners.contains(&token.owner) {
            change_dynamics.unique_owners.push(token.owner.clone());
        }
        change_dynamics.owner = token.owner.clone();

        self.change_dynamics
            .save(deps.storage, token_id, &change_dynamics)?;
        Ok(token)
    }

    pub fn _set_status(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        token_id: &str,
        status: &str,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_send(deps.as_ref(), env, info, &token)?;
        // set owner and remove existing approvals
        //  token.owner = deps.api.addr_validate(recipient)?;
        token.extension.set_status(status);
        //   token.approvals = vec![];
        self.tokens.save(deps.storage, token_id, &token)?;

        Ok(token)
    }

    pub fn _set_name_description(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        token_id: &str,
        name: &Option<String>,
        description: &Option<String>,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        let mut old_exists = false;
        // ensure we have permissions
        self.check_can_send(deps.as_ref(), env, info, &token)?;
        let mut change_dynamics = match self.change_dynamics.load(deps.storage, token_id) {
            Ok(c) => {
                old_exists = true;
                c
            }

            Err(e) => match e {
                StdError::NotFound { .. } => ChangeDynamics {
                    owner: token.owner.clone(),
                    token_id: token_id.to_string(),
                    change_count: 0,
                    unique_owners: vec![token.owner.clone()],
                    transfer_count: 0,
                    block_number: 0,
                    price_ceiling: Default::default(),
                },
                _ => return Err(e.into()),
            },
        };

        let change_cost = self.change_amount(deps.storage)?;
        let change_multiplier = self.change_multiplier(deps.storage)?;

        let change_count = change_dynamics.change_count;
        let cost = change_cost * (change_count / change_multiplier);
        if cost > 0 {
            if let Some(coins) = info.funds.first() {
                if coins.denom != "uusd" || coins.amount < Uint128::from(cost) {
                    return Err(ContractError::Funds {});
                }
            } else {
                return Err(ContractError::Funds {});
            }
        }
        change_dynamics.change_count += 1;

        // set owner and remove existing approvals
        if let Some(desc) = description {
            token.extension.set_description(Some(desc.clone()));
        }
        if let Some(nam) = name {
            if nam.is_empty() {
                self.tokens.save(deps.storage, token_id, &token)?;
                self.change_dynamics
                    .save(deps.storage, token_id, &change_dynamics)?;
            } else {
                match self.tokens.load(deps.storage, nam) {
                    Ok(_) => return Err(ContractError::Claimed {}),
                    Err(_) => {
                        token.extension.set_name(Some(nam.clone()));
                        self.tokens.save(deps.storage, nam, &token)?;
                        self.tokens.remove(deps.storage, token_id)?;
                        self.tokens_uri.save(
                            deps.storage,
                            &token.extension.get_token_uri(),
                            nam,
                        )?;

                        //  self.tokens_uri
                        //     .remove(deps.storage, &token.extension.get_token_uri())?;
                        if let Some(img) = token.extension.get_image_raw() {
                            self.image_uri.save(deps.storage, &img, nam)?;
                            //   self.image_uri.remove(deps.storage, &img)?;
                        }
                        if old_exists {
                            self.change_dynamics.remove(deps.storage, token_id)?;
                        }
                        change_dynamics.token_id = nam.to_string();
                        self.change_dynamics
                            .save(deps.storage, nam, &change_dynamics)?;
                    }
                }
            }
        } else {
            // just a description change.. no need to mess with the indexes
            self.tokens.save(deps.storage, token_id, &token)?;
            self.change_dynamics
                .save(deps.storage, token_id, &change_dynamics)?;
        }

        Ok(token)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn _update_approvals(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: &str,
        token_id: &str,
        // if add == false, remove. if add == true, remove then set with this expiration
        add: bool,
        expires: Option<Expiration>,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_approve(deps.as_ref(), env, info, &token)?;

        // update the approval list (remove any for the same spender before adding)
        let spender_addr = deps.api.addr_validate(spender)?;
        token.approvals = token
            .approvals
            .into_iter()
            .filter(|apr| apr.spender != spender_addr)
            .collect();

        // only difference between approve and revoke
        if add {
            // reject expired data as invalid
            let expires = expires.unwrap_or_default();
            if expires.is_expired(&env.block) {
                return Err(ContractError::Expired {});
            }
            let approval = Approval {
                spender: spender_addr,
                expires,
            };
            token.approvals.push(approval);
        }

        self.tokens.save(deps.storage, token_id, &token)?;

        Ok(token)
    }

    /// returns true iff the sender can execute approve or reject on the contract
    pub fn check_can_approve(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        // owner can approve
        if token.owner == info.sender {
            return Ok(());
        }
        // operator can approve
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Unauthorized {})
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Unauthorized {}),
        }
    }

    /// returns true iff the sender can transfer ownership of the token
    fn check_can_send(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        // owner can send
        if token.owner == info.sender {
            return Ok(());
        }

        // any non-expired token approval can send
        if token
            .approvals
            .iter()
            .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
        {
            return Ok(());
        }

        // operator can send
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Unauthorized {})
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Unauthorized {}),
        }
    }

    fn set_status(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
        status: String,
    ) -> Result<Response<C>, ContractError> {
        self._set_status(deps, &env, &info, &token_id, &status)?;

        Ok(Response::new()
            .add_attribute("action", "set_status")
            .add_attribute("sender", info.sender)
            .add_attribute("status", status)
            .add_attribute("token_id", token_id))
    }
    fn set_name_description(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Response<C>, ContractError> {
        self._set_name_description(deps, &env, &info, &token_id, &name, &description)?;

        if let Some(name_in) = name {
            if name_in.is_empty() {
                Ok(Response::new()
                    .add_attribute("action", "change_name")
                    .add_attribute("sender", info.sender)
                    .add_attribute("token_id", token_id)
                    .add_attribute(
                        "description",
                        description.unwrap_or_else(|| "-not changed-".to_string()),
                    ))
            } else {
                Ok(Response::new()
                    .add_attribute("action", "change_name")
                    .add_attribute("sender", info.sender)
                    .add_attribute("old_token_id", token_id)
                    .add_attribute("token_id", name_in)
                    .add_attribute(
                        "description",
                        description.unwrap_or_else(|| "-not changed-".to_string()),
                    ))
            }
        } else {
            Ok(Response::new()
                .add_attribute("action", "change_description")
                .add_attribute("sender", info.sender)
                .add_attribute("token_id", token_id)
                .add_attribute(
                    "description",
                    description.unwrap_or_else(|| "-not changed-".to_string()),
                ))
        }
    }
}
