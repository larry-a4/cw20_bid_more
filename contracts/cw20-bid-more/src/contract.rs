use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, Binary, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, InitResponse, Querier, StdError, StdResult, Storage, WasmMsg,
};
use cw0::calc_range_start_string;
use cw2::set_contract_version;
use cw20::{Cw20Coin, Cw20CoinHuman, Cw20HandleMsg, Cw20ReceiveMsg};

use crate::balance::Balance;
use crate::msg::{
    is_valid_name, BalanceHuman, CreateMsg, DetailsResponse, HandleMsg, InitMsg, ListResponse,
    QueryMsg, ReceiveMsg,
};
use crate::state::{all_auction_ids, auction, auction_read, Auction};

// Version info, for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-bid-more";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    set_contract_version(&mut deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // No setup
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive(msg) => try_receive(deps, env, msg),
    }
}

pub fn try_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    wrapper: Cw20ReceiveMsg,
) -> StdResult<HandleResponse> {
    let msg: ReceiveMsg = match wrapper.msg {
        Some(bin) => from_binary(&bin),
        None => Err(StdError::parse_err("ReceiveMsg", "no data")),
    }?;
    let token = Cw20Coin {
        address: deps.api.canonical_address(&env.message.sender)?,
        amount: wrapper.amount,
    };
    match msg {
        ReceiveMsg::Create(create) => try_create(deps, env, create, token, wrapper.sender),
        ReceiveMsg::Bid(bid) => try_bid(deps, env, token, bid.id, wrapper.sender),
    }
}

pub fn try_create<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: CreateMsg,
    balance: Cw20Coin,
    sender: HumanAddr,
) -> StdResult<HandleResponse> {
    if !is_valid_name(&msg.id) {
        return Err(StdError::generic_err("Invalid auction id"));
    }

/*    // must have zero token balance
    if !balance.is_empty() {
        return Err(StdError::generic_err(
            "Create auction with zero token only",
        ));
    }
*/
    if msg.expires.is_expired(&env.block) {
        return Err(StdError::generic_err("Expired auction"));
    }

    let my_auction = Auction {
        winner: deps.api.canonical_address(&sender)?,
        source: deps.api.canonical_address(&sender)?,
        expires: msg.expires,
        balance: balance,
    };

    // Try to store it, fail if the id already exists
    auction(&mut deps.storage).update(msg.id.as_bytes(), |existing| match existing {
        None => Ok(my_auction),
        Some(_) => Err(StdError::generic_err("Auction already exists")),
    })?;

    let mut res = HandleResponse::default();
    res.log = vec![
        log("action", "create"),
        log("id", msg.id),
    ];
    Ok(res)
}

pub fn try_bid<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token: Cw20Coin,
    id: String,
    sender: HumanAddr,
) -> StdResult<HandleResponse> {
    let my_auction = auction_read(&deps.storage).load(id.as_bytes())?;

    // Anyone can try to bid, as long as the auction has not expired
    if my_auction.is_expired(&env.block) {
        return Err(StdError::generic_err("Auction has already expired"));
    }
    // the bidder must use the same cw20 token as the current balance
    if my_auction.balance.address != token.address {
        return Err(StdError::generic_err("Must use the same token address"))
    }
    // new bid price must be higher than current bid price
    if my_auction.balance.amount >= token.amount {
        return Err(StdError::generic_err("Bid price not high enough"));
    }

    let current_winner = deps.api.human_address(&my_auction.winner)?;
    let messages = send_tokens(&deps.api, &env.contract.address, &current_winner, my_auction.balance.clone())?;

    let auction_to_save = Auction {
        winner: deps.api.canonical_address(&sender)?,
        balance: token,
        ..my_auction
    };

    // Try to store it, fail if the id does not exist
    auction(&mut deps.storage).update(id.as_bytes(), |existing| match existing {
        None => Err(StdError::generic_err("Auction does not exist")),
        Some(_) => Ok(auction_to_save),
    })?;

    // delete action from storage
    // auction(&mut deps.storage).remove(id.as_bytes());

    Ok(HandleResponse {
        messages: messages,
        log: vec![log("action", "bid"), log("id", id), log("by", sender)],
        data: None,
    })
}

fn parse_hex_32(data: &str) -> StdResult<Vec<u8>> {
    match hex::decode(data) {
        Ok(bin) => {
            if bin.len() == 32 {
                Ok(bin)
            } else {
                Err(StdError::generic_err("Hash must be 64 characters"))
            }
        }
        Err(e) => Err(StdError::generic_err(format!(
            "Error parsing hash: {}",
            e.to_string()
        ))),
    }
}

fn send_tokens<A: Api>(
    api: &A,
    from: &HumanAddr,
    to: &HumanAddr,
    coin: Cw20Coin,
) -> StdResult<Vec<CosmosMsg>> {
    if coin.is_empty() {
        Ok(vec![])
    } else {
        let msg = Cw20HandleMsg::Transfer {
            recipient: to.into(),
            amount: coin.amount,
        };
        let exec = WasmMsg::Execute {
            contract_addr: api.human_address(&coin.address)?,
            msg: to_binary(&msg)?,
            send: vec![],
        };
        Ok(vec![exec.into()])
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::List { start_after, limit } => to_binary(&query_list(deps, start_after, limit)?),
        QueryMsg::Details { id } => to_binary(&query_details(deps, id)?),
    }
}

fn query_details<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    id: String,
) -> StdResult<DetailsResponse> {
    let my_auction = auction_read(&deps.storage).load(id.as_bytes())?;

    // Convert balance to human balance
    let balance_human = Cw20CoinHuman {
        address: deps.api.human_address(&my_auction.balance.address)?,
        amount: my_auction.balance.amount,
    };

    let details = DetailsResponse {
        id,
        winner: deps.api.human_address(&my_auction.winner)?,
        source: deps.api.human_address(&my_auction.source)?,
        expires: my_auction.expires,
        balance: balance_human,
    };
    Ok(details)
}

// Settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn query_list<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ListResponse> {
    let start = calc_range_start_string(start_after);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    Ok(ListResponse {
        auctions: all_auction_ids(&deps.storage, start, limit)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, from_binary, Coin, CosmosMsg, StdError, Uint128};
    use cw20::Expiration;

    const CANONICAL_LENGTH: usize = 20;

    fn mock_env_height<U: Into<HumanAddr>>(sender: U, sent: &[Coin], height: u64) -> Env {
        let mut env = mock_env(sender, sent);
        env.block.height = height;
        env
    }

    #[test]
    fn test_init() {
        let mut deps = mock_dependencies(CANONICAL_LENGTH, &[]);

        // Init an empty contract
        let init_msg = InitMsg {};
        let env = mock_env("anyone", &[]);
        let res = init(&mut deps, env, init_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    /*    #[test]
    fn test_create() {
        let mut deps = mock_dependencies(CANONICAL_LENGTH, &[]);

        let env = mock_env("anyone", &[]);
        init(&mut deps, env, InitMsg {}).unwrap();

        let sender = HumanAddr::from("sender0001");
        let balance = coins(100, "tokens");

        // Cannot create, invalid ids
        let env = mock_env(&sender, &balance);
        for id in vec!["sh", "atomic_swap_id_too_long"] {
            let create = CreateMsg {
                id: id.to_string(),
                expires: Expiration::AtHeight(123456),
            };
            let res = handle(&mut deps, env.clone(), HandleMsg::Receive(create.clone()));
            match res {
                Ok(_) => panic!("expected error"),
                Err(StdError::GenericErr { msg, .. }) => {
                    assert_eq!(msg, "Invalid auction id".to_string())
                }
                Err(e) => panic!("unexpected error: {:?}", e),
            }
        }

        // Cannot create, expired
        let env = mock_env(&sender, &balance);
        let create = CreateMsg {
            id: "swap0001".to_string(),
            expires: Expiration::AtTime(1),
        };
        let res = handle(&mut deps, env, HandleMsg::Create(create.clone()));
        match res {
            Ok(_) => panic!("expected error"),
            Err(StdError::GenericErr { msg, .. }) => {
                assert_eq!(msg, "Expired auction".to_string())
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }

        // Can create, all valid
        let env = mock_env(&sender, &balance);
        let create = CreateMsg {
            id: "swap0001".to_string(),
            expires: Expiration::AtHeight(123456),
        };
        let res = handle(&mut deps, env, HandleMsg::Create(create.clone())).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(log("action", "create"), res.log[0]);

        // Cannot re-create (modify), already existing
        let new_balance = coins(1, "tokens");
        let env = mock_env(&sender, &new_balance);
        let create = CreateMsg {
            id: "swap0001".to_string(),
            expires: Expiration::AtHeight(123456),
        };
        let res = handle(&mut deps, env, HandleMsg::Create(create.clone()));
        match res {
            Ok(_) => panic!("expected error"),
            Err(StdError::GenericErr { msg, .. }) => {
                assert_eq!(msg, "Auction already exists".to_string())
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }
*/
}
