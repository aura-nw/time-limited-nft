use crate::msg::{CheckRoyaltiesResponse, ContractInfoResponse, RoyaltiesInfoResponse};
use crate::state::{Cw721TimeLimited, CREATOR};
use cosmwasm_std::{Decimal, Deps, StdResult, Uint128};

/// NOTE: default behaviour here is to round down
/// EIP2981 specifies that the rounding behaviour is at the discretion of the implementer
pub fn query_royalties_info(
    deps: Deps,
    token_id: String,
    sale_price: Uint128,
) -> StdResult<RoyaltiesInfoResponse> {
    let contract = Cw721TimeLimited::default();
    let token_info = contract.tokens.load(deps.storage, &token_id)?;

    let royalty_percentage = match token_info.extension {
        Some(ref ext) => match ext.royalty_percentage {
            Some(percentage) => Decimal::percent(percentage),
            None => Decimal::percent(0),
        },
        None => Decimal::percent(0),
    };
    let royalty_from_sale_price = sale_price * royalty_percentage;

    let royalty_address = match token_info.extension {
        Some(ext) => match ext.royalty_payment_address {
            Some(addr) => addr,
            None => String::from(""),
        },
        None => String::from(""),
    };

    Ok(RoyaltiesInfoResponse {
        address: royalty_address,
        royalty_amount: royalty_from_sale_price,
    })
}

/// As our default implementation here specifies royalties at token level
/// and not at contract level, it is therefore logically true that
/// on sale, every token managed by this contract should be checked
/// to see if royalties are owed, and to whom. If you are importing
/// this logic, you may want a custom implementation here
pub fn check_royalties(_deps: Deps) -> StdResult<CheckRoyaltiesResponse> {
    Ok(CheckRoyaltiesResponse {
        royalty_payments: true,
    })
}

/// Add the creator to the contract info
pub fn contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    // load default main info
    let default_info = Cw721TimeLimited::default()
        .contract_info
        .load(deps.storage)?;
    // load creator
    let creator = CREATOR.load(deps.storage)?;

    Ok(ContractInfoResponse {
        name: default_info.name,
        symbol: default_info.symbol,
        creator,
    })
}
