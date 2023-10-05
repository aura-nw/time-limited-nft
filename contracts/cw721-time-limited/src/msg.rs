use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CustomMsg, Empty, Uint128};

use crate::state::Extension;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    pub minter: String,
    pub royalty_percentage: Option<u64>,
    pub royalty_payment_address: Option<String>,
    pub creator: Option<String>,
}

pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
pub type QueryMsg = cw721_base::QueryMsg<Cw2981QueryMsg>;

#[cw_serde]
pub enum Cw2981QueryMsg {
    /// Should be called on sale to see if royalties are owed
    /// by the marketplace selling the NFT, if CheckRoyalties
    /// returns true
    /// See https://eips.ethereum.org/EIPS/eip-2981
    RoyaltyInfo {
        token_id: String,
        // the denom of this sale must also be the denom returned by RoyaltiesInfoResponse
        // this was originally implemented as a Coin
        // however that would mean you couldn't buy using CW20s
        // as CW20 is just mapping of addr -> balance
        sale_price: Uint128,
    },
    /// Called against contract to determine if this NFT
    /// implements royalties. Should return a boolean as part of
    /// CheckRoyaltiesResponse - default can simply be true
    /// if royalties are implemented at token level
    /// (i.e. always check on sale)
    CheckRoyalties {},
}

impl Default for Cw2981QueryMsg {
    fn default() -> Self {
        Cw2981QueryMsg::CheckRoyalties {}
    }
}

impl CustomMsg for Cw2981QueryMsg {}

#[cw_serde]
pub struct RoyaltiesInfoResponse {
    pub address: String,
    // Note that this must be the same denom as that passed in to RoyaltyInfo
    // rounding up or down is at the discretion of the implementer
    pub royalty_amount: Uint128,
}

/// Shows if the contract implements royalties
/// if royalty_payments is true, marketplaces should pay them
#[cw_serde]
pub struct CheckRoyaltiesResponse {
    pub royalty_payments: bool,
}

// We define a new ContractInfoResponse to add the creator field
#[cw_serde]
pub struct ContractInfoResponse {
    pub name: String,
    pub symbol: String,
    pub creator: Option<String>,
}
