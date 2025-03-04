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
pub mod types;
pub mod weights;
pub use weights::WeightInfo;

use bifrost_primitives::{CurrencyId, FarmingInfo, PoolId, VtokenMintingInterface};
pub use frame_support::weights::Weight;
use frame_support::{dispatch::DispatchResultWithPostInfo, traits::Get, PalletId};
use frame_system::pallet_prelude::BlockNumberFor;
use orml_traits::MultiCurrency;
pub use pallet::*;
use sp_runtime::{
	traits::{AccountIdConversion, BlockNumberProvider, Saturating, Zero},
	BoundedVec,
};
use sp_std::vec::Vec;
pub use types::*;
pub use RoundIndex;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod migrations;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type CurrencyIdOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

pub type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{RoundInfo, TokenInfo};
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{Perbill, Permill},
	};
	use frame_system::pallet_prelude::*;

	pub type RoundIndex = u32;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type MultiCurrency: MultiCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;

		type EnsureConfirmAsGovernance: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		type WeightInfo: WeightInfo;

		/// The interface to call Farming module functions.
		type FarmingInfo: FarmingInfo<BalanceOf<Self>, CurrencyIdOf<Self>, AccountIdOf<Self>>;

		/// The interface to call VtokenMinting module functions.
		type VtokenMintingInterface: VtokenMintingInterface<
			AccountIdOf<Self>,
			CurrencyIdOf<Self>,
			BalanceOf<Self>,
		>;

		#[pallet::constant]
		type BenefitReceivingAccount: Get<Self::AccountId>;

		/// Max token length 500
		#[pallet::constant]
		type MaxTokenLen: Get<u32>;

		/// Max farming poolid length
		#[pallet::constant]
		type MaxFarmingPoolIdLen: Get<u32>;

		/// ModuleID for creating sub account
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The number of blocks per round, as defined in the runtime.
		///
		/// This value is set to 1500 in the runtime configuration.
		#[pallet::constant]
		type BlocksPerRound: Get<u32>;

		/// The current block number provider.
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	/// Current Round Information
	#[pallet::storage]
	pub(crate) type Round<T: Config> = StorageValue<_, RoundInfo<BlockNumberFor<T>>, OptionQuery>;

	/// The tokenInfo for each currency
	#[pallet::storage]
	pub(crate) type TokenStatus<T: Config> = StorageMap<
		_,
		Twox64Concat,
		CurrencyIdOf<T>,
		TokenInfo<BalanceOf<T>, BlockNumberFor<T>>,
		OptionQuery,
	>;

	/// All token sets
	#[pallet::storage]
	pub(crate) type TokenList<T: Config> =
		StorageValue<_, BoundedVec<CurrencyIdOf<T>, T::MaxTokenLen>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new staking round has started.
		///
		/// - `current`: The index of the current round.
		/// - `first`: The block number at which this round started.
		/// - `length`: The length of the round in blocks.
		NewRound {
			current: RoundIndex,
			first: BlockNumberFor<T>,
			length: u32,
		},
		/// Configuration of a token has been changed.
		///
		/// - `token`: The identifier of the token whose configuration changed.
		/// - `exec_delay`: The delay in blocks before the changes take effect.
		/// - `system_stakable_farming_rate`: The farming rate applied to system-stakable tokens.
		/// - `add_or_sub`: Whether to add or subtract from the stakable farming rate.
		/// - `system_stakable_base`: The base value of system-stakable assets.
		/// - `farming_poolids`: List of pool IDs related to the token.
		/// - `lptoken_rates`: List of rates for liquidity provider (LP) tokens.
		TokenConfigChanged {
			token: CurrencyIdOf<T>,
			exec_delay: BlockNumberFor<T>,
			system_stakable_farming_rate: Permill,
			add_or_sub: bool,
			system_stakable_base: BalanceOf<T>,
			farming_poolids: BoundedVec<PoolId, ConstU32<32>>,
			lptoken_rates: BoundedVec<Perbill, ConstU32<32>>,
		},
		/// A deposit operation has failed.
		///
		/// - `token`: The identifier of the token being deposited.
		/// - `amount`: The amount of the token to be deposited.
		/// - `farming_staking_amount`: The amount staked in the farming pool.
		/// - `system_stakable_amount`: The amount staked in the system-stakable pool.
		/// - `system_shadow_amount`: The amount shadow-staked in the system.
		/// - `pending_redeem_amount`: The amount pending redemption.
		DepositFailed {
			token: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			farming_staking_amount: BalanceOf<T>,
			system_stakable_amount: BalanceOf<T>,
			system_shadow_amount: BalanceOf<T>,
			pending_redeem_amount: BalanceOf<T>,
		},

		/// Minting operation succeeded.
		///
		/// - `token`: The identifier of the token being minted.
		/// - `amount`: The amount of the token to be minted.
		/// - `farming_staking_amount`: The amount staked in the farming pool.
		/// - `system_stakable_amount`: The amount staked in the system-stakable pool.
		/// - `system_shadow_amount`: The amount shadow-staked in the system.
		/// - `pending_redeem_amount`: The amount pending redemption.
		MintSuccess {
			token: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			farming_staking_amount: BalanceOf<T>,
			system_stakable_amount: BalanceOf<T>,
			system_shadow_amount: BalanceOf<T>,
			pending_redeem_amount: BalanceOf<T>,
		},
		/// Minting operation failed.
		///
		/// # Parameters
		/// (Same as MintSuccess)
		MintFailed {
			token: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			farming_staking_amount: BalanceOf<T>,
			system_stakable_amount: BalanceOf<T>,
			system_shadow_amount: BalanceOf<T>,
			pending_redeem_amount: BalanceOf<T>,
		},
		/// Withdrawal operation succeeded.
		///
		/// # Parameters
		/// (Same as MintSuccess)
		WithdrawSuccess {
			token: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			farming_staking_amount: BalanceOf<T>,
			system_stakable_amount: BalanceOf<T>,
			system_shadow_amount: BalanceOf<T>,
			pending_redeem_amount: BalanceOf<T>,
		},
		/// Withdrawal operation failed.
		///
		/// # Parameters
		/// (Same as MintSuccess)
		WithdrawFailed {
			token: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			farming_staking_amount: BalanceOf<T>,
			system_stakable_amount: BalanceOf<T>,
			system_shadow_amount: BalanceOf<T>,
			pending_redeem_amount: BalanceOf<T>,
		},

		/// A redemption operation has succeeded.
		///
		/// # Parameters
		/// (Same as MintSuccess)
		Redeemed {
			token: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			farming_staking_amount: BalanceOf<T>,
			system_stakable_amount: BalanceOf<T>,
			system_shadow_amount: BalanceOf<T>,
			pending_redeem_amount: BalanceOf<T>,
		},
		/// A redemption operation has failed.
		///
		/// # Parameters
		/// (Same as MintSuccess)
		RedeemFailed {
			token: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			farming_staking_amount: BalanceOf<T>,
			system_stakable_amount: BalanceOf<T>,
			system_shadow_amount: BalanceOf<T>,
			pending_redeem_amount: BalanceOf<T>,
		},
		/// The specified token could not be found.
		///
		/// - `token`: The identifier of the token that was not found.
		VtokenNotFound { token: CurrencyIdOf<T> },
		/// Token information has been refreshed.
		///
		/// - `token`: The identifier of the token whose information was refreshed.
		TokenInfoRefreshed { token: CurrencyIdOf<T> },
		/// A payout has been made.
		///
		/// - `token`: The identifier of the token involved in the payout.
		/// - `vtoken`: The identifier of the vtoken involved.
		/// - `from`: The account from which the payout originated.
		/// - `to`: The account to which the payout was made.
		/// - `amount`: The total amount of the payout.
		/// - `free`: The amount of free balance after the payout.
		/// - `vfree`: The amount of vtoken free balance after the payout.
		/// - `shadow`: The shadow balance after the payout.
		Payout {
			token: CurrencyIdOf<T>,
			vtoken: CurrencyIdOf<T>,
			from: AccountIdOf<T>,
			to: AccountIdOf<T>,
			amount: BalanceOf<T>,
			free: BalanceOf<T>,
			vfree: BalanceOf<T>,
			shadow: BalanceOf<T>,
		},
		/// payout error
		PayoutFailed { token: CurrencyIdOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid token config params
		InvalidTokenConfig,
		/// exceed max token len
		ExceedMaxTokenLen,
		/// exceed max poolid len
		ExceedMaxFarmingPoolidLen,
		/// Token info not found
		TokenInfoNotFound,
		/// payout error
		PayoutFailed,
		/// Error converting Vec to BoundedVec.
		ConversionError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: BlockNumberFor<T>) -> Weight {
			// Get token list
			let token_list = TokenList::<T>::get();
			let current_block_number = T::BlockNumberProvider::current_block_number();

			//Get round info, if can't find it in the storage, a new one will be created.
			let mut round = if let Some(round) = <Round<T>>::get() {
				round
			} else {
				// BlocksPerRound == 1500 , 5 hours
				RoundInfo::new(0u32, 0u32.into(), T::BlocksPerRound::get())
			};

			// New round start
			// Current blockNumber -  BlockNumber of Round Start >= Length of Round
			if round.should_update(current_block_number) {
				// Mutate round
				// Set current round index -= 1
				// BlockNumber of Round Start = Current blockNumber
				round.update(current_block_number);
				<Round<T>>::put(round);

				// Iterate through the token list
				for i in token_list.clone().into_iter() {
					// Query the token info for each token in the token list
					if let Some(mut token_info) = TokenStatus::<T>::get(i) {
						// Check token_info.current_config != token_info.new_config
						if token_info.check_config_change() {
							// Update token_info.current_config , set token_info.current_config =
							// token_info.new_config
							token_info.update_config();
							<TokenStatus<T>>::insert(&i, token_info.clone());
						}
					}
				}
				// Trigger event after update round
				Self::deposit_event(Event::NewRound {
					current: round.current,
					first: round.first,
					length: round.length,
				});
			}

			// Get pallet account , eCSrvbA5gGNR17nzbZNJxo7G9mYziLiJcujnWXCNB2CUakX
			let pallet_account: AccountIdOf<T> = T::PalletId::get().into_account_truncating();
			// Iterate through the token list
			for i in token_list.into_iter() {
				// Query the token info for each token in the token list
				if let Some(token_info) = TokenStatus::<T>::get(i) {
					// Current blockNumber -  BlockNumber of Round Start ==
					// token_info.current_config.exec_delay ===> true
					if round.check_delay(current_block_number, token_info.current_config.exec_delay)
					{
						Self::process_token_info(pallet_account.clone(), token_info, i).ok();

						if let Err(_) = Self::do_payout(i) {
							log::error!("System staking auto payout failed, token: {:?}", i);
							Self::deposit_event(Event::PayoutFailed { token: i });
						}
					}
				}
			}

			T::WeightInfo::on_initialize()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Update token config，take effect when next round begins
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::token_config())]
		pub fn token_config(
			origin: OriginFor<T>,
			token: CurrencyIdOf<T>,
			exec_delay: Option<BlockNumberFor<T>>,
			system_stakable_farming_rate: Option<Permill>,
			add_or_sub: Option<bool>,
			system_stakable_base: Option<BalanceOf<T>>,
			farming_poolids: Option<Vec<PoolId>>,
			lptoken_rates: Option<Vec<Perbill>>, // TODO, can be > 1
		) -> DispatchResultWithPostInfo {
			T::EnsureConfirmAsGovernance::ensure_origin(origin)?; // Motion

			// If it exists, get token info, if not, create a new token info
			let mut new_token = false;
			let mut token_info = if let Some(state) = <TokenStatus<T>>::get(&token) {
				state
			} else {
				new_token = true;
				<TokenInfo<BalanceOf<T>, BlockNumberFor<T>>>::default()
			};

			// Set token_info.new_config
			token_info.new_config = token_info.current_config.clone();

			// Set token_info.new_config.exec_delay = exec_delay
			if let Some(exec_delay) = exec_delay {
				ensure!(!exec_delay.is_zero(), Error::<T>::InvalidTokenConfig);
				token_info.new_config.exec_delay = exec_delay;
			}

			// Set token_info.new_config.system_stakable_farming_rate = system_stakable_farming_rate
			if let Some(system_stakable_farming_rate) = system_stakable_farming_rate {
				token_info.new_config.system_stakable_farming_rate = system_stakable_farming_rate;
			}

			// Set token_info.new_config.system_stakable_base = system_stakable_base
			if let Some(system_stakable_base) = system_stakable_base {
				token_info.new_config.system_stakable_base = system_stakable_base;
			}

			// Set token_info.new_config.add_or_sub = add_or_sub
			if let Some(add_or_sub) = add_or_sub {
				token_info.new_config.add_or_sub = add_or_sub;
			}

			// Set token_info.new_config.farming_poolids = farming_poolids
			if let Some(farming_poolids) = farming_poolids.clone() {
				ensure!(!farming_poolids.is_empty(), Error::<T>::InvalidTokenConfig);
				ensure!(
					farming_poolids.len() as u32 <= T::MaxFarmingPoolIdLen::get(),
					Error::<T>::ExceedMaxFarmingPoolidLen
				);
				token_info.new_config.farming_poolids =
					BoundedVec::try_from(farming_poolids.clone())
						.map_err(|_| Error::<T>::ConversionError)?;
			}

			// Set token_info.new_config.lptoken_rates = lptoken_rates
			if let Some(lptoken_rates) = lptoken_rates.clone() {
				ensure!(!lptoken_rates.is_empty(), Error::<T>::InvalidTokenConfig);
				ensure!(
					lptoken_rates.len() as u32 <= T::MaxFarmingPoolIdLen::get(),
					Error::<T>::ExceedMaxFarmingPoolidLen
				);
				token_info.new_config.lptoken_rates = BoundedVec::try_from(lptoken_rates.clone())
					.map_err(|_| Error::<T>::ConversionError)?;
			}

			// Update token info
			<TokenStatus<T>>::insert(&token, token_info.clone());

			// If it is a new token, add it to the token list
			if new_token {
				let mut token_list = TokenList::<T>::get();
				token_list
					.try_push(token)
					.map_err(|_| Error::<T>::ExceedMaxTokenLen)?;
				<TokenList<T>>::put(token_list);
			}

			Self::deposit_event(Event::TokenConfigChanged {
				token,
				exec_delay: token_info.new_config.exec_delay,
				system_stakable_farming_rate: token_info.new_config.system_stakable_farming_rate,
				add_or_sub: token_info.new_config.add_or_sub,
				system_stakable_base: token_info.new_config.system_stakable_base,
				farming_poolids: token_info.new_config.farming_poolids.clone(),
				lptoken_rates: token_info.new_config.lptoken_rates.clone(),
			});

			Ok(().into())
		}

		/// Update token config，take effect when next round begins
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::delete_token())]
		pub fn delete_token(
			origin: OriginFor<T>,
			token: CurrencyIdOf<T>,
		) -> DispatchResultWithPostInfo {
			T::EnsureConfirmAsGovernance::ensure_origin(origin)?; // Motion

			// Remove token info
			<TokenStatus<T>>::remove(&token);

			// Remove token from token list
			let mut token_list = TokenList::<T>::get();
			token_list.retain(|&x| x != token);
			<TokenList<T>>::put(token_list);

			Ok(().into())
		}

		/// refresh token info，query farming pallet, and update TokenInfo, change to new
		/// config，ignore exec_delay, execute immediately
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::refresh_token_info())]
		pub fn refresh_token_info(
			origin: OriginFor<T>,
			token: CurrencyIdOf<T>,
		) -> DispatchResultWithPostInfo {
			T::EnsureConfirmAsGovernance::ensure_origin(origin)?;

			// Get token info
			let mut token_info =
				<TokenStatus<T>>::get(&token).ok_or(Error::<T>::TokenInfoNotFound)?;

			// Check current_config != new_config
			if token_info.check_config_change() {
				// Set current_config = new_config
				token_info.update_config();
			}

			// Get pallet account
			let pallet_account: AccountIdOf<T> = T::PalletId::get().into_account_truncating();
			//
			Pallet::<T>::process_token_info(pallet_account, token_info, token)?;

			Self::deposit_event(Event::TokenInfoRefreshed { token });

			Ok(().into())
		}

		/// payout to receiving account
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::payout())]
		pub fn payout(origin: OriginFor<T>, token: CurrencyIdOf<T>) -> DispatchResultWithPostInfo {
			T::EnsureConfirmAsGovernance::ensure_origin(origin)?;

			Self::do_payout(token)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn process_token_info(
		account: AccountIdOf<T>,
		mut token_info: TokenInfo<BalanceOf<T>, BlockNumberFor<T>>,
		token_id: CurrencyIdOf<T>,
	) -> DispatchResultWithPostInfo {
		// Query farming info
		let mut farming_staking_amount = BalanceOf::<T>::zero();
		for i in 0..token_info.current_config.farming_poolids.len() {
			farming_staking_amount = farming_staking_amount
				+ token_info.current_config.lptoken_rates[i].mul_floor(
					// TODO: get_token_shares
					T::FarmingInfo::get_token_shares(
						token_info.current_config.farming_poolids[i],
						token_id,
					),
				);
		}
		// Set token_info.farming_staking_amount
		token_info.farming_staking_amount = farming_staking_amount;

		// Check amount, and call vtoken minting pallet
		let stakable_amount = if token_info.current_config.add_or_sub {
			// system_stakable_farming_rate * farming_staking_amount + system_stakable_base
			token_info
				.current_config
				.system_stakable_farming_rate
				.mul_floor(token_info.farming_staking_amount)
				.saturating_add(token_info.current_config.system_stakable_base)
		} else {
			// system_stakable_farming_rate * farming_staking_amount - system_stakable_base
			token_info
				.current_config
				.system_stakable_farming_rate
				.mul_floor(token_info.farming_staking_amount)
				.saturating_sub(token_info.current_config.system_stakable_base)
		};
		// Set token_info.system_stakable_amount
		token_info.system_stakable_amount = stakable_amount;

		// Set token_info
		<TokenStatus<T>>::insert(&token_id, token_info.clone());

		// Check stakable_amount > (system_shadow_amount - pending_redeem_amount) ===> mint vksm ,
		// update system_shadow_amount+=mint_amount
		if stakable_amount
			> token_info
				.system_shadow_amount
				.saturating_sub(token_info.pending_redeem_amount)
		{
			// mint_amount = stakable_amount - (system_shadow_amount - pending_redeem_amount)
			let mint_amount = stakable_amount.saturating_sub(
				token_info
					.system_shadow_amount
					.saturating_sub(token_info.pending_redeem_amount),
			);

			// Deposit mint_amount ksm to pallet_account
			T::MultiCurrency::deposit(token_id, &account, mint_amount)?;
			// Change ksm mint to vksm
			T::VtokenMintingInterface::mint(
				account.clone(),
				token_id,
				mint_amount,
				BoundedVec::default(),
				None,
			)?;

			//Update system_shadow_amount += mint_amount
			token_info.system_shadow_amount =
				token_info.system_shadow_amount.saturating_add(mint_amount);

			// Trigger event after update system_shadow_amount
			Self::deposit_event(Event::MintSuccess {
				token: token_id,
				amount: mint_amount,
				farming_staking_amount: token_info.farming_staking_amount,
				system_stakable_amount: token_info.system_stakable_amount,
				system_shadow_amount: token_info.system_shadow_amount,
				pending_redeem_amount: token_info.pending_redeem_amount,
			});
		// Check stakable_amount < (system_shadow_amount - pending_redeem_amount) ===> redeem vksm ,
		// update pending_redeem_amount += token_amount
		} else if stakable_amount
			< token_info
				.system_shadow_amount
				.saturating_sub(token_info.pending_redeem_amount)
		{
			// redeem_amount = system_shadow_amount - pending_redeem_amount - stakable_amount
			let redeem_amount = token_info
				.system_shadow_amount
				.saturating_sub(token_info.pending_redeem_amount)
				.saturating_sub(stakable_amount);

			// token_id convert to vtoken_id
			if let Ok(vtoken_id) = token_id.to_vtoken() {
				// Calculate how many ksm can be received by vksm through VtokenMintingInterface
				// ===> vredeem_amount(vksm amount)
				let vredeem_amount =
					T::VtokenMintingInterface::get_v_currency_amount_by_currency_amount(
						token_id,
						vtoken_id,
						redeem_amount,
					)?;
				if vredeem_amount != BalanceOf::<T>::zero() {
					// redeem vksm ===> vTokenMinting redeem_inner on_redeemed , update
					// pending_redeem_amount += token_amount
					T::VtokenMintingInterface::redeem(account, vtoken_id, vredeem_amount)?;

					//Update token_info.pending_redeem_amount
					let new_token_info = if let Some(state) = <TokenStatus<T>>::get(&token_id) {
						state
					} else {
						<TokenInfo<BalanceOf<T>, BlockNumberFor<T>>>::default()
					};
					token_info.pending_redeem_amount = new_token_info.pending_redeem_amount;
				}
			}
		}

		// Update token_info
		<TokenStatus<T>>::insert(&token_id, token_info.clone());
		Ok(().into())
	}

	fn do_payout(token: CurrencyIdOf<T>) -> DispatchResultWithPostInfo {
		let token_info = <TokenStatus<T>>::get(&token).ok_or(Error::<T>::TokenInfoNotFound)?;

		// token_id convert to vtoken_id
		let vtoken_id = token
			.to_vtoken()
			.map_err(|_| Error::<T>::TokenInfoNotFound)?;

		let pallet_account: AccountIdOf<T> = T::PalletId::get().into_account_truncating();

		// Calculate the revenue generated by vtoken
		let vfree_amount = T::MultiCurrency::free_balance(vtoken_id, &pallet_account);
		let free_amount = T::VtokenMintingInterface::get_currency_amount_by_v_currency_amount(
			token,
			vtoken_id,
			vfree_amount,
		)?;
		let token_amount = free_amount.saturating_sub(token_info.system_shadow_amount);

		// Calculate the number of benefits converted to vtoken
		let vtoken_amount = T::VtokenMintingInterface::get_v_currency_amount_by_currency_amount(
			token,
			vtoken_id,
			token_amount,
		)?;

		// Transfer vtoken(benefits) to BenefitReceivingAccount
		T::MultiCurrency::transfer(
			vtoken_id,
			&pallet_account,
			&T::BenefitReceivingAccount::get(),
			vtoken_amount,
		)
		.map_err(|_| Error::<T>::PayoutFailed)?;

		Self::deposit_event(Event::Payout {
			token,
			vtoken: vtoken_id,
			from: pallet_account,
			to: T::BenefitReceivingAccount::get(),
			amount: vtoken_amount,
			vfree: vfree_amount,
			free: free_amount,
			shadow: token_info.system_shadow_amount,
		});

		Ok(().into())
	}

	// vTokenMinting on_initialize_update_ledger , update pending_redeem_amount -= token_amount ,
	// update system_shadow_amount -= token_amount
	pub fn on_redeem_success(
		token_id: CurrencyIdOf<T>,
		to: AccountIdOf<T>,
		token_amount: BalanceOf<T>,
	) -> Weight {
		//Get pallet account
		let pallet_account: AccountIdOf<T> = T::PalletId::get().into_account_truncating();
		if pallet_account != to {
			return Weight::zero();
		}

		//Get token info
		let mut token_info = if let Some(state) = <TokenStatus<T>>::get(&token_id) {
			state
		} else {
			<TokenInfo<BalanceOf<T>, BlockNumberFor<T>>>::default()
		};

		// pending_redeem_amount -= token_amount
		token_info.pending_redeem_amount = token_info
			.pending_redeem_amount
			.saturating_sub(token_amount);

		// Destroy token
		match T::MultiCurrency::withdraw(token_id, &to, token_amount) {
			Ok(_) => {
				Self::deposit_event(Event::WithdrawSuccess {
					token: token_id,
					amount: token_amount,
					farming_staking_amount: token_info.farming_staking_amount,
					system_stakable_amount: token_info.system_stakable_amount,
					system_shadow_amount: token_info.system_shadow_amount,
					pending_redeem_amount: token_info.pending_redeem_amount,
				});
				token_info.system_shadow_amount =
					token_info.system_shadow_amount.saturating_sub(token_amount);
			}
			Err(error) => {
				log::warn!("{:?} withdraw error: {:?}", &token_id, error);
				Self::deposit_event(Event::WithdrawFailed {
					token: token_id,
					amount: token_amount,
					farming_staking_amount: token_info.farming_staking_amount,
					system_stakable_amount: token_info.system_stakable_amount,
					system_shadow_amount: token_info.system_shadow_amount,
					pending_redeem_amount: token_info.pending_redeem_amount,
				});
			}
		}
		<TokenStatus<T>>::insert(&token_id, token_info);
		T::WeightInfo::on_redeem_success()
	}

	// Slp refund_currency_due_unbond
	pub fn on_refund(
		token_id: CurrencyIdOf<T>,
		to: AccountIdOf<T>,
		token_amount: BalanceOf<T>,
	) -> Weight {
		Self::on_redeem_success(token_id, to, token_amount)
	}

	// vTokenMinting redeem_inner , update pending_redeem_amount += token_amount
	pub fn on_redeemed(
		address: AccountIdOf<T>,
		token_id: CurrencyIdOf<T>,
		token_amount: BalanceOf<T>,
		_vtoken_amount: BalanceOf<T>,
		_fee: BalanceOf<T>,
	) -> Weight {
		//Get pallet account
		let pallet_account: AccountIdOf<T> = T::PalletId::get().into_account_truncating();
		if pallet_account != address {
			return Weight::zero();
		}

		//Get token info
		let mut token_info = if let Some(state) = <TokenStatus<T>>::get(&token_id) {
			state
		} else {
			<TokenInfo<BalanceOf<T>, BlockNumberFor<T>>>::default()
		};

		// pending_redeem_amount += token_amount
		token_info.pending_redeem_amount = token_info
			.pending_redeem_amount
			.saturating_add(token_amount);

		<TokenStatus<T>>::insert(&token_id, token_info.clone());

		Self::deposit_event(Event::Redeemed {
			token: token_id,
			amount: token_amount,
			farming_staking_amount: token_info.farming_staking_amount,
			system_stakable_amount: token_info.system_stakable_amount,
			system_shadow_amount: token_info.system_shadow_amount,
			pending_redeem_amount: token_info.pending_redeem_amount,
		});
		T::WeightInfo::on_redeemed()
	}
}
