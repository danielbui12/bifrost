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

#![cfg_attr(not(feature = "std"), no_std)]

use bifrost_primitives::{CurrencyId, Liquidity, Rate, Ratio, Shortfall};
use parity_scale_codec::Codec;
use sp_runtime::{DispatchError, FixedU128};

sp_api::decl_runtime_apis! {
	pub trait LendMarketApi<AccountId, Balance> where
		AccountId: Codec,
		Balance: Codec {
		fn get_account_liquidity(account: AccountId) -> Result<(Liquidity, Shortfall, Liquidity, Shortfall), DispatchError>;
		fn get_market_status(asset_id: CurrencyId) -> Result<(Rate, Rate, Rate, Ratio, Balance, Balance, FixedU128), DispatchError>;
		fn get_liquidation_threshold_liquidity(account: AccountId) -> Result<(Liquidity, Shortfall, Liquidity, Shortfall), DispatchError>;
	}
}
