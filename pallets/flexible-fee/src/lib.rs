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

pub use crate::pallet::*;
use bifrost_asset_registry::{AssetMetadata, CurrencyIdMapping, TokenInfo};
use bifrost_primitives::{
	traits::XcmDestWeightAndFeeHandler, AssetHubChainId, Balance, BalanceCmp, CurrencyId,
	DerivativeIndex, OraclePriceProvider, Price, TryConvertFrom, XcmOperationType, BNC, VBNC,
};
use bifrost_xcm_interface::calls::{PolkadotXcmCall, RelaychainCall};
use core::convert::Into;
use cumulus_primitives_core::ParaId;
use frame_support::{
	dispatch::PostDispatchInfo,
	pallet_prelude::{ValidateUnsigned, *},
	storage::with_transaction,
	traits::{
		fungibles::Inspect,
		tokens::{Fortitude, Preservation},
		Get,
	},
	transactional,
	weights::WeightMeter,
	PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrency;
use pallet_traits::evm::InspectEvmAccounts;
use polkadot_parachain_primitives::primitives::Sibling;
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_core::{H160, H256, U256};
use sp_runtime::{
	traits::{AccountIdConversion, One},
	BoundedVec, ModuleError, TransactionOutcome,
};
use sp_std::{boxed::Box, cmp::Ordering, vec, vec::Vec};
pub use weights::WeightInfo;
use xcm::{prelude::Unlimited, v4::prelude::*};
use zenlink_protocol::{AssetId, ExportZenlink};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod impls;
pub mod migrations;
mod mock;
mod mock_price;
mod tests;
pub mod weights;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
pub type CurrencyIdOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;
pub type RawCallName = BoundedVec<u8, ConstU32<32>>;

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TargetChain {
	AssetHub,
	RelayChain,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use bifrost_primitives::{Balance, EvmPermit, OraclePriceProvider};
	use frame_support::traits::fungibles::Inspect;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// Event
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
		/// Handler for both NativeCurrency and MultiCurrency
		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>
			+ Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
		/// xcm transfer interface
		type XcmRouter: SendXcm;
		/// Zenlink interface
		type DexOperator: ExportZenlink<Self::AccountId, AssetId>;
		/// The oracle price feeder
		type OraclePriceProvider: OraclePriceProvider;
		/// The only origin that can set universal fee currency order list
		type ControlOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Get the weight and fee for executing Xcm.
		type XcmWeightAndFeeHandler: XcmDestWeightAndFeeHandler<CurrencyId, Balance>;
		/// EVM Accounts info
		type InspectEvmAccounts: InspectEvmAccounts<Self::AccountId, H160>;
		type EvmPermit: EvmPermit;
		/// Get TreasuryAccount
		#[pallet::constant]
		type TreasuryAccount: Get<Self::AccountId>;
		/// Maximum number of CurrencyId's to support handling fees.
		#[pallet::constant]
		type MaxFeeCurrencyOrderListLen: Get<u32>;
		/// When this number is reached, the DOT is sent to AssetHub
		#[pallet::constant]
		type MinAssetHubExecutionFee: Get<BalanceOf<Self>>;
		/// When this number is reached, the DOT is sent to Relaychain
		#[pallet::constant]
		type MinRelaychainExecutionFee: Get<BalanceOf<Self>>;
		/// The currency id of the RelayChain
		#[pallet::constant]
		type RelaychainCurrencyId: Get<CurrencyId>;
		#[pallet::constant]
		type ParachainId: Get<ParaId>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		// asset registry to get asset metadata
		type AssetIdMaps: CurrencyIdMapping<CurrencyId, AssetMetadata<BalanceOf<Self>>>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(_n: BlockNumberFor<T>, limit: Weight) -> Weight {
			let mut weight = Weight::default();

			if WeightMeter::with_limit(limit)
				.try_consume(T::DbWeight::get().reads_writes(4, 2))
				.is_err()
			{
				return weight;
			}

			weight += T::DbWeight::get().reads_writes(4, 2);

			if Self::handle_fee().is_err() {
				return weight;
			}

			weight += T::DbWeight::get().reads_writes(1, 2);
			weight
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transfer to another chain
		TransferTo {
			from: T::AccountId,
			target_chain: TargetChain,
			amount: BalanceOf<T>,
		},
		/// Set user default fee currency
		SetDefaultFeeCurrency {
			who: T::AccountId,
			currency_id: Option<CurrencyId>,
		},
		/// Set universal fee currency order list
		SetFeeCurrencyList {
			currency_list: BoundedVec<CurrencyId, T::MaxFeeCurrencyOrderListLen>,
		},
		/// Set extra fee by call
		SetExtraFee {
			/// The raw call name to be set as the extra fee call.
			raw_call_name: RawCallName,
			/// currency_id, fee_amount, receiver
			fee_info: Option<(CurrencyId, BalanceOf<T>, T::AccountId)>,
		},
	}

	/// The current storage version, we set to 2 our new version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

	/// Universal fee currency order list for all users
	#[pallet::storage]
	pub type UniversalFeeCurrencyOrderList<T: Config> =
		StorageValue<_, BoundedVec<CurrencyId, T::MaxFeeCurrencyOrderListLen>, ValueQuery>;

	/// User default fee currency, if set, will be used as the first fee currency, and then use the
	/// universal fee currency order list
	#[pallet::storage]
	pub type UserDefaultFeeCurrency<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, CurrencyId, OptionQuery>;

	/// Extra fee by call
	#[pallet::storage]
	pub type ExtraFeeByCall<T: Config> = StorageMap<
		_,
		Twox64Concat,
		RawCallName,
		(CurrencyId, BalanceOf<T>, T::AccountId),
		OptionQuery,
	>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// The account does not have enough balance to perform the operation.
		NotEnoughBalance,
		/// An error occurred during currency conversion.
		ConversionError,
		/// No weight or fee information is available for the requested operation.
		WeightAndFeeNotExist,
		/// The message cannot be weighed, possibly due to insufficient information.
		UnweighableMessage,
		/// The XCM execution has failed.
		XcmExecutionFailed,
		/// The specified currency is not supported by the system.
		CurrencyNotSupport,
		/// The maximum number of currencies that can be handled has been reached.
		MaxCurrenciesReached,
		/// EVM permit expired.
		EvmPermitExpired,
		/// EVM permit is invalid.
		EvmPermitInvalid,
		/// EVM permit call failed.
		EvmPermitCallExecutionError,
		/// EVM permit call failed.
		EvmPermitRunnerError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set user default fee currency
		/// Parameters:
		/// - `maybe_fee_currency`: The currency id to be set as the default fee currency.
		///  If `None`, the user default fee currency will be removed.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::set_user_default_fee_currency())]
		pub fn set_user_default_fee_currency(
			origin: OriginFor<T>,
			currency_id: Option<CurrencyId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if let Some(fee_currency) = &currency_id {
				// VBNC is not supported.
				ensure!(fee_currency != &VBNC, Error::<T>::CurrencyNotSupport);

				UserDefaultFeeCurrency::<T>::insert(&who, fee_currency);
			} else {
				UserDefaultFeeCurrency::<T>::remove(&who);
			}
			Self::deposit_event(Event::<T>::SetDefaultFeeCurrency { who, currency_id });
			Ok(())
		}

		/// Set universal fee currency order list
		/// Parameters:
		/// - `default_list`: The currency id list to be set as the universal fee currency order
		///   list.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::set_default_fee_currency_list())]
		pub fn set_default_fee_currency_list(
			origin: OriginFor<T>,
			currency_list: BoundedVec<CurrencyId, T::MaxFeeCurrencyOrderListLen>,
		) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;
			UniversalFeeCurrencyOrderList::<T>::put(currency_list.clone());
			Self::deposit_event(Event::<T>::SetFeeCurrencyList { currency_list });
			Ok(())
		}

		/// Set universal fee currency order list
		/// Parameters:
		/// - `raw_call_name`: The raw call name to be set as the extra fee call.
		/// - `fee_info`: The currency id, fee amount and receiver to be set as the extra fee.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::set_user_default_fee_currency())]
		pub fn set_extra_fee(
			origin: OriginFor<T>,
			raw_call_name: RawCallName,
			fee_info: Option<(CurrencyId, BalanceOf<T>, T::AccountId)>,
		) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;
			match fee_info.clone() {
				Some(fee_info) => ExtraFeeByCall::<T>::insert(&raw_call_name, fee_info),
				None => ExtraFeeByCall::<T>::remove(&raw_call_name),
			};
			Self::deposit_event(Event::<T>::SetExtraFee {
				raw_call_name,
				fee_info,
			});
			Ok(())
		}

		/// Dispatch EVM permit.
		/// The main purpose of this function is to allow EVM accounts to pay for the transaction
		/// fee in non-native currency by allowing them to self-dispatch pre-signed permit.
		/// The EVM fee is paid in the currency set for the account.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::EvmPermit::dispatch_weight(*gas_limit))]
		pub fn dispatch_permit(
			origin: OriginFor<T>,
			from: H160,
			to: H160,
			value: U256,
			data: Vec<u8>,
			gas_limit: u64,
			deadline: U256,
			v: u8,
			r: H256,
			s: H256,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;

			// dispatch permit should never return error.
			// validate_unsigned should prevent the transaction getting to this point in case of
			// invalid permit. In case of any error, we call error handler ( which should pause
			// this transaction) and return ok.
			if T::EvmPermit::validate_permit(
				from,
				to,
				data.clone(),
				value,
				gas_limit,
				deadline,
				v,
				r,
				s,
			)
			.is_err()
			{
				T::EvmPermit::on_dispatch_permit_error();
				return Ok(PostDispatchInfo::default());
			};

			let (gas_price, _) = T::EvmPermit::gas_price();

			let result = T::EvmPermit::dispatch_permit(
				from,
				to,
				data,
				value,
				gas_limit,
				gas_price,
				None,
				None,
				vec![],
			)
			.unwrap_or_else(|e| {
				// In case of runner error, account has not been charged, so we need to call error
				// handler to pause dispatch error
				if e.error == Error::<T>::EvmPermitRunnerError.into() {
					T::EvmPermit::on_dispatch_permit_error();
				}
				e.post_info
			});

			Ok(result)
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::dispatch_permit {
					from,
					to,
					value,
					data,
					gas_limit,
					deadline,
					v,
					r,
					s,
				} => {
					// We need to wrap this as separate tx, and since we also "dry-run" the
					// dispatch, we need to rollback the changes if any
					let result = with_transaction::<(), DispatchError, _>(|| {
						// First verify signature
						let result = T::EvmPermit::validate_permit(
							*from,
							*to,
							data.clone(),
							*value,
							*gas_limit,
							*deadline,
							*v,
							*r,
							*s,
						);
						if let Some(error_res) = result.err() {
							return TransactionOutcome::Rollback(Err(error_res));
						}

						let (gas_price, _) = T::EvmPermit::gas_price();

						let result = T::EvmPermit::dispatch_permit(
							*from,
							*to,
							data.clone(),
							*value,
							*gas_limit,
							gas_price,
							None,
							None,
							vec![],
						);
						match result {
							Ok(_post_info) => TransactionOutcome::Rollback(Ok(())),
							Err(e) => TransactionOutcome::Rollback(Err(e.error)),
						}
					});
					let nonce = T::EvmPermit::permit_nonce(*from);
					match result {
						Ok(()) => ValidTransaction::with_tag_prefix("EvmPermit")
							.and_provides((nonce, from))
							.priority(0)
							.longevity(64)
							.propagate(true)
							.build(),
						Err(e) => {
							let error_number = match e {
								DispatchError::Module(ModuleError { error, .. }) => error[0],
								_ => 0, /* this case should never happen because an Error is
								         * always converted to DispatchError::Module(ModuleError) */
							};
							InvalidTransaction::Custom(error_number).into()
						}
					}
				}
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	#[transactional]
	fn handle_fee() -> DispatchResult {
		let fee_receiver = Self::get_fee_receiver(1);
		let fee_receiver_balance =
			T::MultiCurrency::free_balance(T::RelaychainCurrencyId::get(), &fee_receiver);
		if fee_receiver_balance >= T::MinAssetHubExecutionFee::get() {
			T::MultiCurrency::withdraw(
				T::RelaychainCurrencyId::get(),
				&fee_receiver,
				fee_receiver_balance,
			)?;

			let asset: Asset = Asset {
				id: AssetId(Location::here()),
				fun: Fungible(UniqueSaturatedInto::<u128>::unique_saturated_into(
					fee_receiver_balance,
				)),
			};

			let remote_call = RelaychainCall::XcmPallet(PolkadotXcmCall::LimitedTeleportAssets(
				Box::new(Location::new(0, [Parachain(AssetHubChainId::get())]).into()),
				Box::new(
					Location::new(
						0,
						[AccountId32 {
							network: None,
							id: Sibling::from(T::ParachainId::get()).into_account_truncating(),
						}],
					)
					.into(),
				),
				Box::new(asset.into()),
				0,
				Unlimited,
			))
			.encode()
			.into();

			let (require_weight_at_most, xcm_fee) =
				T::XcmWeightAndFeeHandler::get_operation_weight_and_fee(
					T::RelaychainCurrencyId::get(),
					XcmOperationType::TeleportAssets,
				)
				.ok_or(Error::<T>::WeightAndFeeNotExist)?;

			let fee: Asset = Asset {
				id: AssetId(Location::here()),
				fun: Fungible(xcm_fee),
			};

			let remote_xcm = Xcm(vec![
				WithdrawAsset(fee.clone().into()),
				BuyExecution {
					fees: fee.clone(),
					weight_limit: Unlimited,
				},
				Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most,
					call: remote_call,
				},
				DepositAsset {
					assets: All.into(),
					beneficiary: Location::new(0, [Parachain(T::ParachainId::get().into())]),
				},
			]);
			let (ticket, _) =
				T::XcmRouter::validate(&mut Some(Location::parent()), &mut Some(remote_xcm))
					.map_err(|_| Error::<T>::UnweighableMessage)?;
			T::XcmRouter::deliver(ticket).map_err(|_| Error::<T>::XcmExecutionFailed)?;

			Self::deposit_event(Event::TransferTo {
				from: fee_receiver,
				target_chain: TargetChain::AssetHub,
				amount: fee_receiver_balance,
			});
		}

		let fee_receiver = Self::get_fee_receiver(0);
		let fee_receiver_balance =
			T::MultiCurrency::free_balance(T::RelaychainCurrencyId::get(), &fee_receiver);
		if fee_receiver_balance >= T::MinRelaychainExecutionFee::get() {
			T::MultiCurrency::withdraw(
				T::RelaychainCurrencyId::get(),
				&fee_receiver,
				fee_receiver_balance,
			)?;

			Self::deposit_event(Event::TransferTo {
				from: fee_receiver,
				target_chain: TargetChain::RelayChain,
				amount: fee_receiver_balance,
			});
		}

		Ok(())
	}

	fn get_fee_receiver(index: DerivativeIndex) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating(index)
	}

	/// Get user fee charge assets order
	fn get_fee_currency_list(account_id: &T::AccountId) -> Vec<CurrencyId> {
		// Get universal fee currency order list
		let mut fee_currency_list: Vec<CurrencyId> = UniversalFeeCurrencyOrderList::<T>::get()
			.into_iter()
			.collect();

		// Get user default fee currency
		if let Some(default_fee_currency) = UserDefaultFeeCurrency::<T>::get(&account_id) {
			if let Some(index) = fee_currency_list
				.iter()
				.position(|&c| c == default_fee_currency)
			{
				fee_currency_list.remove(index);
			}
			let first_fee_currency_index = 0;
			fee_currency_list.insert(first_fee_currency_index, default_fee_currency);
		};

		if fee_currency_list.is_empty() {
			vec![BNC]
		} else {
			fee_currency_list
		}
	}

	fn get_fee_currency_and_fee_amount(
		who: &T::AccountId,
		fee_amount: Balance,
	) -> Result<(CurrencyId, Balance, Price, Price), Error<T>> {
		let fee_currency_list = Self::get_fee_currency_list(who);
		// charge the fee by the order of the above order list.
		// first to check whether the user has the asset. If no, pass it. If yes, try to make
		// transaction in the DEX in exchange for BNC
		for currency_id in fee_currency_list {
			// If it is mainnet currency
			if currency_id == BNC {
				if T::MultiCurrency::ensure_can_withdraw(currency_id, who, fee_amount).is_ok() {
					return Ok((currency_id, fee_amount, Price::one(), Price::one()));
				}
			} else {
				let (fee_amount, price_in, price_out) =
					T::OraclePriceProvider::get_oracle_amount_by_currency_and_amount_in(
						&BNC,
						fee_amount,
						&currency_id,
					)
					.ok_or(Error::<T>::ConversionError)?;
				if T::MultiCurrency::ensure_can_withdraw(currency_id, who, fee_amount).is_ok() {
					return Ok((currency_id, fee_amount, price_in, price_out));
				}
			}
		}
		Err(Error::<T>::NotEnoughBalance)
	}

	fn charge_extra_fee(
		who: &T::AccountId,
		extra_fee_currency: CurrencyId,
		extra_fee_amount: Balance,
		extra_fee_receiver: &T::AccountId,
	) -> Result<(), Error<T>> {
		let fee_currency_list = Self::get_fee_currency_list(who);
		// charge the fee by the order of the above order list.
		// first to check whether the user has the asset. If no, pass it. If yes, try to make
		// transaction in the DEX in exchange for BNC
		let mut fee_info = None;
		for currency_id in fee_currency_list {
			// If it is mainnet currency
			if currency_id == extra_fee_currency {
				if T::MultiCurrency::ensure_can_withdraw(extra_fee_currency, who, extra_fee_amount)
					.is_ok()
				{
					fee_info = Some((currency_id, extra_fee_amount));
					break;
				}
			} else {
				match Self::ensure_can_swap(who, currency_id, extra_fee_currency, extra_fee_amount)
				{
					Ok(amount_in) => {
						fee_info = Some((currency_id, amount_in));
						break;
					}
					Err(_) => {}
				}
			}
		}

		match fee_info {
			Some((fee_currency, fee_amount)) => {
				if fee_currency == extra_fee_currency {
					T::MultiCurrency::transfer(fee_currency, who, extra_fee_receiver, fee_amount)
						.map_err(|_| Error::<T>::NotEnoughBalance)?;
					Ok(())
				} else {
					let from_asset_id = Self::get_currency_asset_id(fee_currency)?;
					let to_asset_id = Self::get_currency_asset_id(extra_fee_currency)?;
					let path = vec![from_asset_id, to_asset_id];

					T::DexOperator::inner_swap_assets_for_exact_assets(
						who,
						extra_fee_amount,
						fee_amount,
						&path,
						extra_fee_receiver,
					)
					.map_err(|_| Error::<T>::NotEnoughBalance)?;
					Ok(())
				}
			}
			None => Err(Error::<T>::ConversionError),
		}
	}

	/// This function is for runtime-api to call
	pub fn cal_fee_token_and_amount(
		who: &T::AccountId,
		fee: Balance,
		_utx: &<T as frame_system::Config>::RuntimeCall,
	) -> Result<(CurrencyId, Balance), Error<T>> {
		let (fee_currency, fee_amount, _, _) = Self::get_fee_currency_and_fee_amount(who, fee)
			.map_err(|_| Error::<T>::NotEnoughBalance)?;
		Ok((fee_currency, fee_amount))
	}

	fn get_currency_asset_id(currency_id: CurrencyId) -> Result<AssetId, Error<T>> {
		let asset_id: AssetId =
			AssetId::try_convert_from(currency_id, T::ParachainId::get().into())
				.map_err(|_| Error::<T>::ConversionError)?;
		Ok(asset_id)
	}

	fn ensure_can_swap(
		who: &T::AccountId,
		from_currency: CurrencyId,
		to_currency: CurrencyId,
		amount_out: Balance,
	) -> Result<Balance, Error<T>> {
		// If it is other assets, go to exchange fee amount.
		let from_asset_id =
			Self::get_currency_asset_id(from_currency).map_err(|_| Error::<T>::ConversionError)?;

		let to_asset_id =
			Self::get_currency_asset_id(to_currency).map_err(|_| Error::<T>::ConversionError)?;

		let path = vec![from_asset_id, to_asset_id];
		match T::DexOperator::get_amount_in_by_path(amount_out, &path) {
			Ok(amounts) => {
				let amount_in = amounts[0];
				T::MultiCurrency::ensure_can_withdraw(from_currency, who, amount_in)
					.map_err(|_| Error::<T>::NotEnoughBalance)?;
				Ok(amount_in)
			}
			Err(_) => Err(Error::<T>::NotEnoughBalance)?,
		}
	}
}

impl<T: Config> BalanceCmp<T::AccountId> for Pallet<T> {
	type Error = Error<T>;

	/// Compares the balance of a specific `currency` for a given `account` against an `amount`
	/// while considering different currency precisions.
	///
	/// # Parameters
	/// - `account`: The account ID whose balance will be checked.
	/// - `currency`: The currency ID to be compared.
	/// - `amount`: The amount to compare against the account's balance, with the precision
	///   specified by `amount_precision`.
	/// - `amount_precision`: The precision of the `amount` specified. If greater than 18, the
	///   precision of the `currency` will be adjusted accordingly.
	///
	/// # Returns
	/// - `Ok(std::cmp::Ordering)`: Returns the ordering result (`Less`, `Equal`, `Greater`) based
	///   on the comparison between the adjusted balance and the adjusted amount.
	/// - `Err(Error<T>)`: Returns an error if the currency is not supported.
	fn cmp_with_precision(
		account: &T::AccountId,
		currency: &CurrencyId,
		amount: u128,
		amount_precision: u32,
	) -> Result<Ordering, Error<T>> {
		// Get the reducible balance for the specified account and currency.
		let mut balance = T::MultiCurrency::reducible_balance(
			*currency,
			account,
			Preservation::Preserve,
			Fortitude::Polite,
		);

		// Define the standard precision as 18 decimal places.
		let standard_precision: u32 = amount_precision.max(18);

		// Adjust the amount to the standard precision.
		let precision_offset = standard_precision.saturating_sub(amount_precision);
		let adjust_precision = 10u128.pow(precision_offset);
		let amount = amount.saturating_mul(adjust_precision);

		// Adjust the balance based on currency type.
		let decimals = currency
			.decimals()
			.or_else(|| {
				T::AssetIdMaps::get_currency_metadata(*currency)
					.map(|metadata| metadata.decimals.into())
			})
			.ok_or(Error::<T>::CurrencyNotSupport)?;
		let balance_precision_offset = standard_precision.saturating_sub(decimals.into());

		// Apply precision adjustment to balance.
		balance = balance.saturating_mul(10u128.pow(balance_precision_offset));

		// Compare the adjusted balance with the input amount.
		Ok(balance.cmp(&amount))
	}
}
