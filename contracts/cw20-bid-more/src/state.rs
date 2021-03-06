use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::balance::Balance;
use cosmwasm_std::{
    Binary, BlockInfo, CanonicalAddr, Order, ReadonlyStorage, StdError, StdResult, Storage,
};
use cosmwasm_storage::{bucket, bucket_read, prefixed_read, Bucket, ReadonlyBucket};
use cw20::{Expiration, Cw20Coin};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Auction {
    pub winner: CanonicalAddr,
    pub source: CanonicalAddr,
    pub expires: Expiration,
    /// Balance in cw20 token
    pub balance: Cw20Coin,
}

impl Auction {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(&block)
    }
}

pub const PREFIX_AUCTION: &[u8] = b"auction";

/// Returns a bucket with all swaps (query by id)
pub fn auction<S: Storage>(storage: &mut S) -> Bucket<S, Auction> {
    bucket(PREFIX_AUCTION, storage)
}

/// Returns a bucket with all swaps (query by id)
/// (read-only version for queries)
pub fn auction_read<S: ReadonlyStorage>(storage: &S) -> ReadonlyBucket<S, Auction> {
    bucket_read(PREFIX_AUCTION, storage)
}

/// This returns the list of ids for all active swaps
pub fn all_auction_ids<S: ReadonlyStorage>(
    storage: &S,
    start: Option<Vec<u8>>,
    limit: usize,
) -> StdResult<Vec<String>> {
    prefixed_read(PREFIX_AUCTION, storage)
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|(k, _)| String::from_utf8(k).map_err(|_| StdError::invalid_utf8("Parsing auction id")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::{Binary, Uint128};

    #[test]
    fn test_no_swap_ids() {
        let storage = MockStorage::new();
        let ids = all_auction_ids(&storage, None, 10).unwrap();
        assert_eq!(0, ids.len());
    }

    fn dummy_swap() -> Auction {
        Auction {
            winner: CanonicalAddr(Binary(b"recip".to_vec())),
            source: CanonicalAddr(Binary(b"source".to_vec())),
            expires: Expiration::default(),
            balance: Cw20Coin{
                address:CanonicalAddr(Binary(b"address".to_vec())),
                amount: Uint128(0),
            }
        }
    }

    #[test]
    fn test_all_swap_ids() {
        let mut storage = MockStorage::new();
        auction(&mut storage)
            .save("lazy".as_bytes(), &dummy_swap())
            .unwrap();
        auction(&mut storage)
            .save("assign".as_bytes(), &dummy_swap())
            .unwrap();
        auction(&mut storage)
            .save("zen".as_bytes(), &dummy_swap())
            .unwrap();

        let ids = all_auction_ids(&storage, None, 10).unwrap();
        assert_eq!(3, ids.len());
        assert_eq!(
            vec!["assign".to_string(), "lazy".to_string(), "zen".to_string()],
            ids
        )
    }
}
