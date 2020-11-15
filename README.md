# Bid More

This is an example collection of two contracts designed to showcase
increasing price auction with cw20 tokens

**Warning** None of these contracts have been audited and no liability is
assumed for the use of any of this code. They are provided to turbo-start
your projects.

## Contracts

| Name                                               | Description                                  |
| -------------------------------------------------- | -------------------------------------------- |
| [`cw20-bid-more`](contracts/cw20-bid-more)         | Increasing price auction accepting only cw20 |
| [`cw20-base`](contracts/cw20-base)                 | CW20 (ERC20 equivalent) token implementation |

## Running this contract

You will need Rust 1.44.1+ with wasm32-unknown-unknown target installed.

Once you are happy with the content, you can compile it to wasm on each contracts directory via:

```
cargo wasm
```

Or for a production-ready (compressed) build, run the following from the repository root:

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.3
```

The optimized contracts are generated in the artifacts/ directory.

## Licenses

This repo contains two license, [Apache 2.0](./LICENSE-APACHE) and
[AGPL 3.0](./LICENSE-AGPL.md). All crates in this repo may be licensed
as one or the other. Please check the `NOTICE` in each crate or the 
relevant `Cargo.toml` file for clarity.

All *specifications* will always be Apache-2.0. All contracts that are
meant to be *building blocks* will also be Apache-2.0. This is along
the lines of Open Zepellin or other public references.

Contracts that are "ready to deploy" may be licensed under AGPL 3.0 to 
encourage anyone using them to contribute back any improvements they
make. This is common practice for actual projects running on Ethereum,
like Uniswap or Maker DAO.
