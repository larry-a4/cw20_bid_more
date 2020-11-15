use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, HumanAddr};
use cw20::{Cw20CoinHuman, Cw20ReceiveMsg, Expiration, Cw20Coin};
use crate::balance::Balance;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Create(CreateMsg),
    Bid(BidMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateMsg {
    /// id is a human-readable name for the auction to use later.
    /// 3-20 bytes of utf-8 text
    pub id: String,
    /// You can set expiration at time or at block height the contract is valid at.
    /// After the contract is expired, it can be returned to the original funder.
    pub expires: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidMsg {
    /// id is a human-readable name for the auction to use later.
    /// 3-20 bytes of utf-8 text
    pub id: String,
}

pub fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() < 3 || bytes.len() > 20 {
        return false;
    }
    true
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Show all open auctions. Return type is ListResponse.
    List {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns the details of the named auction, error if not created.
    /// Return type: DetailsResponse.
    Details { id: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ListResponse {
    /// List all open auction ids
    pub auctions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DetailsResponse {
    /// Id of this auction
    pub id: String,
    /// If released, funds go to the recipient
    pub winner: HumanAddr,
    /// If refunded, funds go to the source
    pub source: HumanAddr,
    /// Once an auction is expired, it can be claimed by the highest bidder (via "claim").
    pub expires: Expiration,
    /// Balance in native tokens or cw20 token, with human address
    pub balance: Cw20CoinHuman,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum BalanceHuman {
    Native(Vec<Coin>),
    Cw20(Cw20CoinHuman),
}
