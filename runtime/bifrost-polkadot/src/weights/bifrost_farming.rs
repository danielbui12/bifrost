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
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for bifrost_farming
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-09-14, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bifrost-jenkins`, CPU: `Intel(R) Xeon(R) CPU E5-26xx v4`
//! WASM-EXECUTION: Compiled, CHAIN: Some("bifrost-kusama-local"), DB CACHE: 1024

// Executed Command:
// target/release/bifrost
// benchmark
// pallet
// --chain=bifrost-kusama-local
// --steps=50
// --repeat=20
// --pallet=bifrost_farming
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./runtime/bifrost-kusama/src/weights/bifrost_farming.rs
// --template=./weight-template/runtime-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions for bifrost_farming.
pub struct BifrostWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> bifrost_farming::WeightInfo for BifrostWeight<T> {
	// Storage: Farming PoolInfos (r:1 w:0)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming GaugePoolInfos (r:1 w:0)
	// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming BoostPoolInfos (r:1 w:0)
	// Proof Skipped: Farming BoostPoolInfos (max_values: Some(1), max_size: None, mode: Measured)
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `113`
		//  Estimated: `3578`
		// Minimum execution time: 21_942 nanoseconds.
		Weight::from_parts(22_864_000, 3578)
			.saturating_add(T::DbWeight::get().reads(3))
	}
	// Storage: Farming PoolNextId (r:1 w:1)
	// Proof Skipped: Farming PoolNextId (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Farming GaugePoolNextId (r:1 w:1)
	// Proof Skipped: Farming GaugePoolNextId (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Farming GaugePoolInfos (r:0 w:1)
	// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming PoolInfos (r:0 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	fn create_farming_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `1594`
		// Minimum execution time: 52_521 nanoseconds.
		Weight::from_parts(53_437_000, 1594)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Tokens Accounts (r:2 w:2)
	// Proof: Tokens Accounts (max_values: None, max_size: Some(118), added: 2593, mode: MaxEncodedLen)
	// Storage: AssetRegistry CurrencyMetadatas (r:1 w:0)
	// Proof Skipped: AssetRegistry CurrencyMetadatas (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	fn deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1752`
		//  Estimated: `6176`
		// Minimum execution time: 164_962 nanoseconds.
		Weight::from_parts(169_239_000, 6176)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	fn withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `510`
		//  Estimated: `3975`
		// Minimum execution time: 77_186 nanoseconds.
		Weight::from_parts(78_796_000, 3975)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming GaugeInfos (r:1 w:0)
	// Proof Skipped: Farming GaugeInfos (max_values: None, max_size: None, mode: Measured)
	fn claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `547`
		//  Estimated: `4012`
		// Minimum execution time: 76_561 nanoseconds.
		Weight::from_parts(77_635_000, 4012)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Farming PoolInfos (r:1 w:0)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	fn withdraw_claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `510`
		//  Estimated: `3975`
		// Minimum execution time: 52_808 nanoseconds.
		Weight::from_parts(53_519_000, 3975)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming GaugePoolNextId (r:1 w:1)
	// Proof Skipped: Farming GaugePoolNextId (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Farming GaugePoolInfos (r:0 w:1)
	// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	fn reset_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `436`
		//  Estimated: `3901`
		// Minimum execution time: 58_270 nanoseconds.
		Weight::from_parts(59_721_000, 3901)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming RetireLimit (r:1 w:0)
	// Proof Skipped: Farming RetireLimit (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:0)
	// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming GaugePoolInfos (r:1 w:1)
	// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	fn force_retire_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `627`
		//  Estimated: `4092`
		// Minimum execution time: 71_326 nanoseconds.
		Weight::from_parts(72_463_000, 4092)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	fn kill_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `380`
		//  Estimated: `3845`
		// Minimum execution time: 50_342 nanoseconds.
		Weight::from_parts(51_828_000, 3845)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming GaugePoolInfos (r:1 w:1)
	// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	fn edit_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `513`
		//  Estimated: `3978`
		// Minimum execution time: 55_700 nanoseconds.
		Weight::from_parts(57_535_000, 3978)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	fn close_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `417`
		//  Estimated: `3882`
		// Minimum execution time: 45_159 nanoseconds.
		Weight::from_parts(46_221_000, 3882)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Tokens Accounts (r:2 w:2)
	// Proof: Tokens Accounts (max_values: None, max_size: Some(118), added: 2593, mode: MaxEncodedLen)
	// Storage: AssetRegistry CurrencyMetadatas (r:1 w:0)
	// Proof Skipped: AssetRegistry CurrencyMetadatas (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn charge() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2059`
		//  Estimated: `6176`
		// Minimum execution time: 162_037 nanoseconds.
		Weight::from_parts(167_329_000, 6176)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Farming RetireLimit (r:1 w:0)
	// Proof Skipped: Farming RetireLimit (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Farming GaugeInfos (r:2 w:1)
	// Proof Skipped: Farming GaugeInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming GaugePoolInfos (r:1 w:1)
	// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming PoolInfos (r:1 w:0)
	// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:0)
	// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	fn force_gauge_claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `855`
		//  Estimated: `6795`
		// Minimum execution time: 99_268 nanoseconds.
		Weight::from_parts(100_563_000, 6795)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Farming RetireLimit (r:1 w:1)
	// Proof Skipped: Farming RetireLimit (max_values: Some(1), max_size: None, mode: Measured)
	fn set_retire_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `1594`
		// Minimum execution time: 28_852 nanoseconds.
		Weight::from_parts(29_866_000, 1594)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming BoostWhitelist (r:0 w:1)
	// Proof Skipped: Farming BoostWhitelist (max_values: None, max_size: None, mode: Measured)
	fn add_boost_pool_whitelist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 11_356 nanoseconds.
		Weight::from_parts(11_723_000, 0)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming BoostNextRoundWhitelist (r:0 w:1)
	// Proof Skipped: Farming BoostNextRoundWhitelist (max_values: None, max_size: None, mode: Measured)
	fn set_next_round_whitelist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `145`
		// Minimum execution time: 16_252 nanoseconds.
		Weight::from_parts(19_916_000, 145)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming BoostPoolInfos (r:1 w:1)
	// Proof Skipped: Farming BoostPoolInfos (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Farming UserBoostInfos (r:1 w:1)
	// Proof Skipped: Farming UserBoostInfos (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming BoostWhitelist (r:1 w:0)
	// Proof Skipped: Farming BoostWhitelist (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming BoostVotingPools (r:1 w:1)
	// Proof Skipped: Farming BoostVotingPools (max_values: None, max_size: None, mode: Measured)
	fn vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `3610`
		// Minimum execution time: 50_384 nanoseconds.
		Weight::from_parts(52_399_000, 3610)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Farming BoostPoolInfos (r:1 w:1)
	// Proof Skipped: Farming BoostPoolInfos (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Farming BoostNextRoundWhitelist (r:1 w:0)
	// Proof Skipped: Farming BoostNextRoundWhitelist (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming BoostWhitelist (r:2 w:0)
	// Proof Skipped: Farming BoostWhitelist (max_values: None, max_size: None, mode: Measured)
	// Storage: Farming BoostVotingPools (r:1 w:0)
	// Proof Skipped: Farming BoostVotingPools (max_values: None, max_size: None, mode: Measured)
	fn start_boost_round() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149`
		//  Estimated: `6089`
		// Minimum execution time: 63_053 nanoseconds.
		Weight::from_parts(64_194_000, 6089)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming BoostPoolInfos (r:1 w:1)
	// Proof Skipped: Farming BoostPoolInfos (max_values: Some(1), max_size: None, mode: Measured)
	fn end_boost_round() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `195`
		//  Estimated: `1680`
		// Minimum execution time: 42_148 nanoseconds.
		Weight::from_parts(43_580_000, 1680)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Tokens Accounts (r:2 w:2)
	// Proof: Tokens Accounts (max_values: None, max_size: Some(118), added: 2593, mode: MaxEncodedLen)
	// Storage: AssetRegistry CurrencyMetadatas (r:1 w:0)
	// Proof Skipped: AssetRegistry CurrencyMetadatas (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn charge_boost() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1477`
		//  Estimated: `6176`
		// Minimum execution time: 126_868 nanoseconds.
		Weight::from_parts(131_856_000, 6176)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Farming::UserFarmingPool` (r:1 w:0)
	/// Proof: `Farming::UserFarmingPool` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Farming::SharesAndWithdrawnRewards` (r:2 w:0)
	/// Proof: `Farming::SharesAndWithdrawnRewards` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Farming::PoolInfos` (r:1 w:0)
	/// Proof: `Farming::PoolInfos` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `BbBNC::UserPositions` (r:1 w:0)
	/// Proof: `BbBNC::UserPositions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn refresh() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `604`
		//  Estimated: `6544`
		// Minimum execution time: 16_472_000 picoseconds.
		Weight::from_parts(17_023_000, 0)
			.saturating_add(Weight::from_parts(0, 6544))
			.saturating_add(T::DbWeight::get().reads(5))
	}
}
