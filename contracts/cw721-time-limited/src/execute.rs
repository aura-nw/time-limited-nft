use crate::state::{Cw721TimeLimited, Extension};
use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response};
use cw721::{Cw721ReceiveMsg, Expiration};
use cw721_base::{
    state::{Approval, TokenInfo},
    ContractError,
};
use cw_ownable::OwnershipError;

pub fn _transfer_nft(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    recipient: &str,
    token_id: &str,
) -> Result<TokenInfo<Extension>, ContractError> {
    let mut token = Cw721TimeLimited::default()
        .tokens
        .load(deps.storage, token_id)?;
    // ensure we have permissions
    check_can_send(deps.as_ref(), env, info, &token)?;
    // set owner and remove existing approvals
    token.owner = deps.api.addr_validate(recipient)?;
    token.approvals = vec![];
    Cw721TimeLimited::default()
        .tokens
        .save(deps.storage, token_id, &token)?;
    Ok(token)
}

#[allow(clippy::too_many_arguments)]
pub fn _update_approvals(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    spender: &str,
    token_id: &str,
    // if add == false, remove. if add == true, remove then set with this expiration
    add: bool,
    expires: Option<Expiration>,
) -> Result<TokenInfo<Extension>, ContractError> {
    let mut token = Cw721TimeLimited::default()
        .tokens
        .load(deps.storage, token_id)?;
    // ensure we have permissions
    check_can_approve(deps.as_ref(), env, info, &token)?;

    // update the approval list (remove any for the same spender before adding)
    let spender_addr = deps.api.addr_validate(spender)?;
    token.approvals.retain(|apr| apr.spender != spender_addr);

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

    Cw721TimeLimited::default()
        .tokens
        .save(deps.storage, token_id, &token)?;

    Ok(token)
}

/// returns true iff the sender can execute approve or reject on the contract
pub fn check_can_approve(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo<Extension>,
) -> Result<(), ContractError> {
    // if the token is expired, just minter can send
    match &token.extension {
        Some(metadata) => match metadata.expires {
            Some(expires) => {
                if expires.is_expired(&env.block) {
                    cw_ownable::assert_owner(deps.storage, &info.sender)?;
                } else if token.owner == info.sender {
                    return Ok(());
                }
            }
            None => {
                if token.owner == info.sender {
                    return Ok(());
                }
            }
        },
        None => {
            if token.owner == info.sender {
                return Ok(());
            }
        }
    }

    // operator can approve
    let op = Cw721TimeLimited::default()
        .operators
        .may_load(deps.storage, (&token.owner, &info.sender))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(ContractError::Ownership(OwnershipError::NotOwner))
            } else {
                Ok(())
            }
        }
        None => Err(ContractError::Ownership(OwnershipError::NotOwner)),
    }
}

/// returns true iff the sender can transfer ownership of the token
pub fn check_can_send(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo<Extension>,
) -> Result<(), ContractError> {
    // if the token is expired, just minter can send
    match &token.extension {
        Some(metadata) => match metadata.expires {
            Some(expires) => {
                if expires.is_expired(&env.block) {
                    if cw_ownable::assert_owner(deps.storage, &info.sender).is_ok() {
                        return Ok(());
                    } else {
                        return Err(ContractError::Ownership(OwnershipError::NotOwner));
                    }
                } else if token.owner == info.sender {
                    return Ok(());
                }
            }
            None => {
                if token.owner == info.sender {
                    return Ok(());
                }
            }
        },
        None => {
            if token.owner == info.sender {
                return Ok(());
            }
        }
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
    let op = Cw721TimeLimited::default()
        .operators
        .may_load(deps.storage, (&token.owner, &info.sender))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(ContractError::Ownership(OwnershipError::NotOwner))
            } else {
                Ok(())
            }
        }
        None => Err(ContractError::Ownership(OwnershipError::NotOwner)),
    }
}

pub fn approve(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<Response<Empty>, ContractError> {
    _update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id))
}

pub fn revoke(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> Result<Response<Empty>, ContractError> {
    _update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

    Ok(Response::new()
        .add_attribute("action", "revoke")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id))
}

pub fn approve_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    expires: Option<Expiration>,
) -> Result<Response<Empty>, ContractError> {
    // reject expired data as invalid
    let expires = expires.unwrap_or_default();
    if expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // set the operator for us
    let operator_addr = deps.api.addr_validate(&operator)?;
    Cw721TimeLimited::default().operators.save(
        deps.storage,
        (&info.sender, &operator_addr),
        &expires,
    )?;

    Ok(Response::new()
        .add_attribute("action", "approve_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

pub fn revoke_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response<Empty>, ContractError> {
    let operator_addr = deps.api.addr_validate(&operator)?;
    Cw721TimeLimited::default()
        .operators
        .remove(deps.storage, (&info.sender, &operator_addr));

    Ok(Response::new()
        .add_attribute("action", "revoke_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

pub fn transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response<Empty>, ContractError> {
    _transfer_nft(deps, &env, &info, &recipient, &token_id)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id))
}

pub fn send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response<Empty>, ContractError> {
    // Transfer token
    _transfer_nft(deps, &env, &info, &contract, &token_id)?;

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

pub fn burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response<Empty>, ContractError> {
    let token = Cw721TimeLimited::default()
        .tokens
        .load(deps.storage, &token_id)?;
    check_can_send(deps.as_ref(), &env, &info, &token)?;

    Cw721TimeLimited::default()
        .tokens
        .remove(deps.storage, &token_id)?;
    Cw721TimeLimited::default().decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}
