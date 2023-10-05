#[cfg(test)]
use crate::contract::{execute, instantiate, query};
use crate::msg::{
    CheckRoyaltiesResponse, Cw2981QueryMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    RoyaltiesInfoResponse,
};
use crate::query::{check_royalties, query_royalties_info};
use crate::state::{Cw721TimeLimited, Metadata};

use cosmwasm_std::{from_binary, to_binary, Uint128};

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cw721::{Cw721Query, Expiration};

const CREATOR: &str = "minter";

#[test]
fn use_metadata_extension() {
    let mut deps = mock_dependencies();
    let contract = Cw721TimeLimited::default();

    let info = mock_info(CREATOR, &[]);
    // let royalty_percentage = 101
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(50),
        royalty_payment_address: Some("john".to_string()),
        creator: Some("creator".to_string()),
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let expected_extension = Some(Metadata {
        description: Some("Spaceship with Warp Drive".into()),
        name: Some("Starship USS Enterprise".to_string()),
        royalty_percentage: Some(50),
        royalty_payment_address: Some("john".to_string()),
        ..Metadata::default()
    });

    let token_id = "Enterprise";

    let exec_msg = ExecuteMsg::Mint {
        token_id: token_id.to_string(),
        owner: "john".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        }),
    };
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
    assert_eq!(
        res.token_uri,
        Some("https://starships.example.com/Starship/Enterprise.json".into())
    );
    assert_eq!(res.extension, expected_extension);
}

#[test]
fn validate_royalty_information() {
    let mut deps = mock_dependencies();
    let _contract = Cw721TimeLimited::default();

    let info = mock_info(CREATOR, &[]);
    // let royalty_percentage = 101
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(101),
        royalty_payment_address: Some("john".to_string()),
        creator: Some("creator".to_string()),
    };
    // instantiate will fail
    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg);
    assert!(res.is_err());
}

#[test]
fn not_allow_setting_royalty_when_minting() {
    let mut deps = mock_dependencies();
    let _contract = Cw721TimeLimited::default();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(50),
        royalty_payment_address: Some("john".to_string()),
        creator: Some("creator".to_string()),
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";

    let exec_msg = ExecuteMsg::Mint {
        token_id: token_id.to_string(),
        owner: "john".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            royalty_percentage: Some(50),
            royalty_payment_address: Some("john".to_string()),
            ..Metadata::default()
        }),
    };
    let res = execute(deps.as_mut(), mock_env(), info, exec_msg);
    assert!(res.is_err());
}

#[test]
fn check_royalties_response() {
    let mut deps = mock_dependencies();
    let _contract = Cw721TimeLimited::default();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(50),
        royalty_payment_address: Some("john".to_string()),
        creator: Some("creator".to_string()),
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";

    let exec_msg = ExecuteMsg::Mint {
        token_id: token_id.to_string(),
        owner: "john".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        }),
    };
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    let expected = CheckRoyaltiesResponse {
        royalty_payments: true,
    };
    let res = check_royalties(deps.as_ref()).unwrap();
    assert_eq!(res, expected);

    // also check the longhand way
    let query_msg = QueryMsg::Extension {
        msg: Cw2981QueryMsg::CheckRoyalties {},
    };
    let query_res: CheckRoyaltiesResponse =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(query_res, expected);
}

#[test]
fn check_token_royalties() {
    let mut deps = mock_dependencies();

    let royalty_payment_address = "jeanluc".to_string();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(10),
        royalty_payment_address: Some(royalty_payment_address.clone()),
        creator: Some("creator".to_string()),
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";

    let exec_msg = ExecuteMsg::Mint {
        token_id: token_id.to_string(),
        owner: "jeanluc".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        }),
    };
    execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

    let expected = RoyaltiesInfoResponse {
        address: royalty_payment_address.clone(),
        royalty_amount: Uint128::new(10),
    };
    let res = query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
    assert_eq!(res, expected);

    // also check the longhand way
    let query_msg = QueryMsg::Extension {
        msg: Cw2981QueryMsg::RoyaltyInfo {
            token_id: token_id.to_string(),
            sale_price: Uint128::new(100),
        },
    };
    let query_res: RoyaltiesInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(query_res, expected);

    // check for rounding down
    // which is the default behaviour
    let voyager_token_id = "Voyager";

    let voyager_exec_msg = ExecuteMsg::Mint {
        token_id: voyager_token_id.to_string(),
        owner: "janeway".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Voyager.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Voyager".to_string()),
            ..Metadata::default()
        }),
    };
    execute(deps.as_mut(), mock_env(), info, voyager_exec_msg).unwrap();

    // 43 x 0.10 (i.e., 10%) should be 4.3
    // we expect this to be rounded down to 1
    let voyager_expected = RoyaltiesInfoResponse {
        address: royalty_payment_address,
        royalty_amount: Uint128::new(4),
    };

    let res = query_royalties_info(
        deps.as_ref(),
        voyager_token_id.to_string(),
        Uint128::new(43),
    )
    .unwrap();
    assert_eq!(res, voyager_expected);
}

#[test]
fn check_token_without_royalties() {
    let mut deps = mock_dependencies();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: None,
        royalty_payment_address: None,
        creator: Some("creator".to_string()),
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";

    let exec_msg = ExecuteMsg::Mint {
        token_id: token_id.to_string(),
        owner: "jeanluc".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        }),
    };
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    let expected = RoyaltiesInfoResponse {
        address: "".to_string(),
        royalty_amount: Uint128::new(0),
    };
    let res = query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
    assert_eq!(res, expected);

    // also check the longhand way
    let query_msg = QueryMsg::Extension {
        msg: Cw2981QueryMsg::RoyaltyInfo {
            token_id: token_id.to_string(),
            sale_price: Uint128::new(100),
        },
    };
    let query_res: RoyaltiesInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(query_res, expected);
}

#[test]
fn check_token_without_extension() {
    let mut deps = mock_dependencies();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: None,
        royalty_payment_address: None,
        creator: Some("creator".to_string()),
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";

    let exec_msg = ExecuteMsg::Mint {
        token_id: token_id.to_string(),
        owner: "jeanluc".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: None,
    };
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    let expected = RoyaltiesInfoResponse {
        address: "".to_string(),
        royalty_amount: Uint128::new(0),
    };
    let res = query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
    assert_eq!(res, expected);

    // also check the longhand way
    let query_msg = QueryMsg::Extension {
        msg: Cw2981QueryMsg::RoyaltyInfo {
            token_id: token_id.to_string(),
            sale_price: Uint128::new(100),
        },
    };
    let query_res: RoyaltiesInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(query_res, expected);
}

#[test]
fn check_token_expires() {
    let mut deps = mock_dependencies();
    let contract = Cw721TimeLimited::default();

    let royalty_payment_address = "jeanluc".to_string();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(10),
        royalty_payment_address: Some(royalty_payment_address.clone()),
        creator: Some("creator".to_string()),
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";

    let exec_msg = ExecuteMsg::Mint {
        token_id: token_id.to_string(),
        owner: "jeanluc".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            expires: Some(Expiration::AtHeight(mock_env().block.height + 100)),
            ..Metadata::default()
        }),
    };
    execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

    let expected = RoyaltiesInfoResponse {
        address: royalty_payment_address.clone(),
        royalty_amount: Uint128::new(10),
    };
    let res = query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
    assert_eq!(res, expected);

    // query token_info
    let token_info = contract
        .nft_info(deps.as_ref(), token_id.to_string())
        .unwrap();
    assert_eq!(
        token_info.extension.unwrap().expires,
        Some(Expiration::AtHeight(mock_env().block.height + 100))
    );

    // check that we can still transfer the token
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: "picard1".to_string(),
        token_id: token_id.to_string(),
    };
    let new_info = mock_info("jeanluc", &[]);
    execute(deps.as_mut(), mock_env(), new_info, transfer_msg).unwrap();

    // query token_info
    let token_info = contract
        .owner_of(deps.as_ref(), mock_env(), token_id.to_string(), false)
        .unwrap();
    assert_eq!(token_info.owner, "picard1".to_string());

    // check that CREATOR cannot burn the token
    let burn_msg = ExecuteMsg::Burn {
        token_id: token_id.to_string(),
    };
    let new_info = mock_info(CREATOR, &[]);
    let res = execute(deps.as_mut(), mock_env(), new_info, burn_msg);
    assert!(res.is_err());

    // increase height to 101
    let mut new_env = mock_env();
    new_env.block.height += 101;

    // check that "picard" cannot transfer the token
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: "riker".to_string(),
        token_id: token_id.to_string(),
    };
    let new_info = mock_info("picard1", &[]);
    let res = execute(deps.as_mut(), new_env.clone(), new_info, transfer_msg);
    assert!(res.is_err());

    // check that "picard" cannot send the token
    let send_msg = ExecuteMsg::SendNft {
        contract: "riker".to_string(),
        token_id: token_id.to_string(),
        msg: to_binary("hello").unwrap(),
    };
    let new_info = mock_info("picard1", &[]);
    let res = execute(deps.as_mut(), new_env.clone(), new_info, send_msg);
    assert!(res.is_err());

    // check that CREATOR can burn the token
    let burn_msg = ExecuteMsg::Burn {
        token_id: token_id.to_string(),
    };
    let new_info = mock_info(CREATOR, &[]);
    let res = execute(deps.as_mut(), new_env.clone(), new_info, burn_msg);
    assert!(res.is_ok());
}
