#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::execute::{approve, approve_all, burn, revoke, revoke_all, send_nft, transfer_nft};
use crate::msg::{Cw2981QueryMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::{check_royalties, contract_info, query_royalties_info};
use crate::state::{Config, Cw721TimeLimited, CONFIG, CREATOR};

use cw721_base::{ContractError, InstantiateMsg as Cw721InstantiateMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-time-limited";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // create InstantiateMsg for cw721-base
    let time_limited_init = Cw721InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        minter: msg.minter,
    };
    let res =
        Cw721TimeLimited::default().instantiate(deps.branch(), env, info, time_limited_init)?;
    // Explicitly set contract name and version, otherwise set to cw721-base info
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;

    // validate royalty_percentage to be between 0 and 100
    if let Some(royalty_percentage) = msg.royalty_percentage {
        if royalty_percentage > 100 {
            return Err(ContractError::Std(StdError::generic_err(
                "Royalty percentage cannot be greater than 100",
            )));
        }
    }

    // set royalty_percentage and royalty_payment_address
    CONFIG.save(
        deps.storage,
        &Config {
            royalty_percentage: msg.royalty_percentage,
            royalty_payment_address: msg.royalty_payment_address,
        },
    )?;

    // set creator
    CREATOR.save(deps.storage, &msg.creator)?;

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {
            token_id,
            owner,
            token_uri,
            extension,
        } => {
            let mut extension = extension.unwrap_or_default();

            // return error if royalty is set
            if extension.royalty_percentage.is_some() || extension.royalty_payment_address.is_some()
            {
                return Err(ContractError::Std(StdError::generic_err(
                    "Cannot set royalty information in mint message",
                )));
            }

            // override royalty information with config
            let config = CONFIG.load(deps.storage)?;
            extension.royalty_percentage = config.royalty_percentage;
            extension.royalty_payment_address = config.royalty_payment_address;

            let mint_msg = ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                extension: Some(extension),
            };

            Cw721TimeLimited::default().execute(deps, env, info, mint_msg)
        }
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => approve(deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke { spender, token_id } => revoke(deps, env, info, spender, token_id),
        ExecuteMsg::ApproveAll { operator, expires } => {
            approve_all(deps, env, info, operator, expires)
        }
        ExecuteMsg::RevokeAll { operator } => revoke_all(deps, env, info, operator),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Burn { token_id } => burn(deps, env, info, token_id),
        _ => Cw721TimeLimited::default().execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Extension { msg } => match msg {
            Cw2981QueryMsg::RoyaltyInfo {
                token_id,
                sale_price,
            } => to_binary(&query_royalties_info(deps, token_id, sale_price)?),
            Cw2981QueryMsg::CheckRoyalties {} => to_binary(&check_royalties(deps)?),
        },
        QueryMsg::ContractInfo {} => to_binary(&contract_info(deps)?),
        _ => Cw721TimeLimited::default().query(deps, env, msg),
    }
}
