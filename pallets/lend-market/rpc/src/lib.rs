// This file is part of Bifrost.

// Copyright (C) Liebi Technologies PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::sync::Arc;

pub use lend_market_rpc_runtime_api::LendMarketApi as LendMarketRuntimeApi;

use bifrost_primitives::{CurrencyId, Liquidity, Rate, Ratio, Shortfall};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::{ErrorCode, ErrorObject},
};
use parity_scale_codec::Codec;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{traits::Block as BlockT, FixedU128};

#[rpc(client, server)]
pub trait LendMarketApi<BlockHash, AccountId, Balance>
where
	Balance: Codec + Copy + TryFrom<NumberOrHex>,
{
	#[method(name = "lend_market_getCollateralLiquidity")]
	fn get_account_liquidity(
		&self,
		account: AccountId,
		at: Option<BlockHash>,
	) -> RpcResult<(Liquidity, Shortfall, Liquidity, Shortfall)>;
	#[method(name = "lend_market_getMarketStatus")]
	fn get_market_status(
		&self,
		asset_id: CurrencyId,
		at: Option<BlockHash>,
	) -> RpcResult<(Rate, Rate, Rate, Ratio, NumberOrHex, NumberOrHex, FixedU128)>;
	#[method(name = "lend_market_getLiquidationThresholdLiquidity")]
	fn get_liquidation_threshold_liquidity(
		&self,
		account: AccountId,
		at: Option<BlockHash>,
	) -> RpcResult<(Liquidity, Shortfall, Liquidity, Shortfall)>;
}

/// A struct that implements the [`LendMarketApi`].
pub struct LendMarket<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> LendMarket<C, B> {
	/// Create new `LendMarket` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	RuntimeError,
	AccountLiquidityError,
	MarketStatusError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::AccountLiquidityError => 2,
			Error::MarketStatusError => 3,
		}
	}
}

#[async_trait]
impl<C, Block, AccountId, Balance> LendMarketApiServer<<Block as BlockT>::Hash, AccountId, Balance>
	for LendMarket<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: LendMarketRuntimeApi<Block, AccountId, Balance>,
	AccountId: Codec,
	Balance: Codec + Copy + TryFrom<NumberOrHex> + Into<NumberOrHex> + std::fmt::Display,
{
	fn get_account_liquidity(
		&self,
		account: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(Liquidity, Shortfall, Liquidity, Shortfall)> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		);
		api.get_account_liquidity(at, account)
			.map_err(runtime_error_into_rpc_error)?
			.map_err(account_liquidity_error_into_rpc_error)
	}

	fn get_market_status(
		&self,
		asset_id: CurrencyId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(Rate, Rate, Rate, Ratio, NumberOrHex, NumberOrHex, FixedU128)> {
		let api = self.client.runtime_api();
		let at: <Block as BlockT>::Hash = at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		);
		let (
			borrow_rate,
			supply_rate,
			exchange_rate,
			util,
			total_borrows,
			total_reserves,
			borrow_index,
		) = api.get_market_status(at, asset_id)
			.map_err(runtime_error_into_rpc_error)?
			.map_err(market_status_error_into_rpc_error)?;
		Ok((
			borrow_rate,
			supply_rate,
			exchange_rate,
			util,
			try_into_rpc_balance(total_borrows)?,
			try_into_rpc_balance(total_reserves)?,
			borrow_index,
		))
	}

	fn get_liquidation_threshold_liquidity(
		&self,
		account: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(Liquidity, Shortfall, Liquidity, Shortfall)> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		);
		api.get_liquidation_threshold_liquidity(at, account)
			.map_err(runtime_error_into_rpc_error)?
			.map_err(account_liquidity_error_into_rpc_error)
	}
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_error(err: impl std::fmt::Debug) -> ErrorObject<'static> {
	ErrorObject::owned(
		Error::RuntimeError.into(),
		"Runtime trapped",
		Some(format!("{:?}", err)),
	)
}

/// Converts an account liquidity error into an RPC error.
fn account_liquidity_error_into_rpc_error(err: impl std::fmt::Debug) -> ErrorObject<'static> {
	ErrorObject::owned(
		Error::AccountLiquidityError.into(),
		"Not able to get account liquidity",
		Some(format!("{:?}", err)),
	)
}

/// Converts an market status error into an RPC error.
fn market_status_error_into_rpc_error(err: impl std::fmt::Debug) -> ErrorObject<'static> {
	ErrorObject::owned(
		Error::MarketStatusError.into(),
		"Not able to get market status",
		Some(format!("{:?}", err)),
	)
}

fn try_into_rpc_balance<T: std::fmt::Display + Copy + TryInto<NumberOrHex>>(
	value: T,
) -> RpcResult<NumberOrHex> {
	value.try_into().map_err(|_| {
		ErrorObject::owned(
			ErrorCode::InvalidParams.code(),
			format!("{} doesn't fit in NumberOrHex representation", value),
			None::<()>,
		)
	})
}
