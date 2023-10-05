use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
use cw_storage_plus::Item;

use cw721::Expiration;
use cw721_base::Cw721Contract;

use crate::msg::Cw2981QueryMsg;

pub type Cw721TimeLimited<'a> = Cw721Contract<'a, Extension, Empty, Empty, Cw2981QueryMsg>;

// we define new Metadata for this contract
// this is just a extension of cw721_base::Metadata
pub type Extension = Option<Metadata>;

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    /// This is how much the minter takes as a cut when sold
    /// royalties are owed on this token if it is Some
    pub royalty_percentage: Option<u64>,
    /// The payment address, may be different to or the same
    /// as the minter addr
    /// question: how do we validate this?
    pub royalty_payment_address: Option<String>,
    // the expiration time of the token
    pub expires: Option<Expiration>,
}

#[cw_serde]
#[derive(Default)]
pub struct Config {
    pub royalty_percentage: Option<u64>,
    pub royalty_payment_address: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
// Some collection may want to have the creator different from the minter
pub const CREATOR: Item<Option<String>> = Item::new("creator");
