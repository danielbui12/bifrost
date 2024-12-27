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

use crate::*;
use frame_support::{
	pallet_prelude::StorageVersion,
	traits::{GetStorageVersion, OnRuntimeUpgrade},
};
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "lend-market::migration";

pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Check the storage version
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version < 1 {
			// Transform storage values
			// We transform the storage values from the old into the new format.
			log::info!(target: LOG_TARGET, "Start to migrate LiquidationFreeCollateralsstorage...");
			LiquidationFreeCollaterals::<T>::translate::<Vec<AssetIdOf<T>>, _>(
				|maybe_value: Option<Vec<AssetIdOf<T>>>| {
					let target_bounded_vec: BoundedVec<AssetIdOf<T>, T::MaxLengthLimit>;
					if let Some(value) = maybe_value {
						// If there's a value, try to convert Vec to BoundedVec
						target_bounded_vec = BoundedVec::try_from(value).unwrap();
					} else {
						// If there's no value (None), set the BoundedVec to default (empty)
						target_bounded_vec =
							BoundedVec::<AssetIdOf<T>, T::MaxLengthLimit>::default();
					}
					// Return the new BoundedVec as the migrated value
					Some(target_bounded_vec)
				},
			)
			.unwrap();

			log::info!(target: LOG_TARGET, "Start to migrate MarketBond storage...");
			MarketBond::<T>::translate::<Vec<AssetIdOf<T>>, _>(
				|k: AssetIdOf<T>, value: Vec<AssetIdOf<T>>| {
					log::info!(target: LOG_TARGET, "Migrated to boundedvec for {:?}...", k);

					let target_bounded_vec: BoundedVec<AssetIdOf<T>, T::MaxLengthLimit>;
					if value.len() != 0 {
						target_bounded_vec = BoundedVec::try_from(value).unwrap();
					} else {
						target_bounded_vec =
							BoundedVec::<AssetIdOf<T>, T::MaxLengthLimit>::default();
					}

					Some(target_bounded_vec)
				},
			);

			// Update the storage version
			StorageVersion::new(1).put::<Pallet<T>>();

			// Return the consumed weight
			let liquidation_free_collaterals_count = 1u64;
			let market_bond_count = MarketBond::<T>::iter().count();
			Weight::from(T::DbWeight::get().reads_writes(
				liquidation_free_collaterals_count + market_bond_count as u64 + 1,
				liquidation_free_collaterals_count as u64 + market_bond_count as u64 + 1,
			))
		} else {
			// We don't do anything here.
			Weight::zero()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		let liquidation_free_collaterals_count = LiquidationFreeCollaterals::<T>::get().len();
		let market_bond_count = MarketBond::<T>::iter().count();

		// print out the pre-migrate storage count
		log::info!(target: LOG_TARGET, "LiquidationFreeCollaterals pre-migrate storage count: {:?}", liquidation_free_collaterals_count);
		log::info!(target: LOG_TARGET, "MarketBond pre-migrate storage count: {:?}", market_bond_count);
		Ok((
			liquidation_free_collaterals_count as u64,
			market_bond_count as u64,
		)
			.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(cnt: Vec<u8>) -> Result<(), TryRuntimeError> {
		let new_liquidation_free_collaterals_count = LiquidationFreeCollaterals::<T>::get().len();
		let new_market_bond_count = MarketBond::<T>::iter().count();

		let (old_liquidation_free_collaterals_count, old_market_bond_count): (u64, u64) =
			Decode::decode(&mut cnt.as_slice()).expect(
				"the state parameter should be something that was generated by pre_upgrade",
			);

		// print out the post-migrate storage count
		log::info!(
			target: LOG_TARGET,
			"LiquidationFreeCollaterals post-migrate storage count: {:?}",
			new_liquidation_free_collaterals_count
		);

		log::info!(
			target: LOG_TARGET,
			"MarketBond post-migrate storage count: {:?}",
			new_market_bond_count
		);

		ensure!(
			new_liquidation_free_collaterals_count as u64 == old_liquidation_free_collaterals_count,
			"LiquidationFreeCollaterals Post-migration storage count does not match pre-migration count"
		);

		ensure!(
			new_market_bond_count as u64 == old_market_bond_count,
			"MarketBond Post-migration storage count does not match pre-migration count"
		);

		Ok(())
	}
}
