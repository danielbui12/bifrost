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

//! Tests for the module.

#![cfg(test)]
use crate::{
	impls::on_charge_transaction::PaymentInfo, mock::*, BlockNumberFor, BoundedVec, Config,
	DispatchError::BadOrigin, Error, UserDefaultFeeCurrency,
};
use bifrost_asset_registry::AssetMetadata;
use bifrost_asset_registry::CurrencyMetadatas;
use bifrost_primitives::{
	AccountFeeCurrency, BalanceCmp, CurrencyId, TryConvertFrom, BNC, DOT, KSM, MANTA, VBNC, VDOT,
	WETH,
};
use frame_support::{
	assert_noop, assert_ok, assert_storage_noop,
	dispatch::{DispatchInfo, PostDispatchInfo},
	pallet_prelude::ValidateUnsigned,
	traits::fungibles::Mutate,
	weights::Weight,
};
use orml_traits::MultiCurrency;
use pallet_traits::evm::InspectEvmAccounts;
use pallet_transaction_payment::OnChargeTransaction;
use sp_arithmetic::FixedU128;
use sp_core::{H256, U256};
use sp_runtime::{transaction_validity::TransactionSource, AccountId32};
use std::cmp::Ordering::{Greater, Less};
use zenlink_protocol::AssetId;

// some common variables
pub const CHARLIE: AccountId32 = AccountId32::new([0u8; 32]);

pub const ALICE: AccountId = AccountId::new([1; 32]);
pub const BOB: AccountId = AccountId::new([2; 32]);
pub const DICK: AccountId32 = AccountId32::new([3u8; 32]);

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info() -> DispatchInfo {
	// pays_fee: Pays::Yes -- class: DispatchClass::Normal
	DispatchInfo {
		weight: Weight::default(),
		..Default::default()
	}
}

fn post_info() -> PostDispatchInfo {
	PostDispatchInfo {
		actual_weight: Some(Weight::default()),
		pays_fee: Default::default(),
	}
}

fn ini_meta_data() {
	let metadata = AssetMetadata {
		name: b"Polkadot DOT".to_vec(),
		symbol: b"DOT".to_vec(),
		decimals: 10,
		minimal_balance: 1000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	assert_ok!(AssetRegistry::register_vtoken_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		0
	));

	assert_eq!(CurrencyMetadatas::<Test>::get(DOT), Some(metadata.clone()));

	let metadata = AssetMetadata {
		name: b"Moonbeam Native Token".to_vec(),
		symbol: b"GLMR".to_vec(),
		decimals: 18,
		minimal_balance: 1000000000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"Tether USD".to_vec(),
		symbol: b"USDT".to_vec(),
		decimals: 6,
		minimal_balance: 1000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"Astar".to_vec(),
		symbol: b"ASTR".to_vec(),
		decimals: 18,
		minimal_balance: 10000000000000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"Filecoin Network Token".to_vec(),
		symbol: b"FIL".to_vec(),
		decimals: 18,
		minimal_balance: 1000000000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"USD Coin".to_vec(),
		symbol: b"USDC".to_vec(),
		decimals: 6,
		minimal_balance: 1000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"interBTC".to_vec(),
		symbol: b"IBTC".to_vec(),
		decimals: 8,
		minimal_balance: 100,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"Interlay".to_vec(),
		symbol: b"INTR".to_vec(),
		decimals: 10,
		minimal_balance: 10000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"Manta Network".to_vec(),
		symbol: b"MANTA".to_vec(),
		decimals: 18,
		minimal_balance: 1000000000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"bncs-20 inscription token BNCS".to_vec(),
		symbol: b"BNCS".to_vec(),
		decimals: 12,
		minimal_balance: 10000000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"PINK".to_vec(),
		symbol: b"PINK".to_vec(),
		decimals: 10,
		minimal_balance: 100000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"DED".to_vec(),
		symbol: b"DED".to_vec(),
		decimals: 10,
		minimal_balance: 1,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"Pendulum".to_vec(),
		symbol: b"PEN".to_vec(),
		decimals: 12,
		minimal_balance: 100000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	let metadata = AssetMetadata {
		name: b"Wrapped ETH".to_vec(),
		symbol: b"WETH".to_vec(),
		decimals: 18,
		minimal_balance: 15000000000000,
	};

	assert_ok!(AssetRegistry::register_token_metadata(
		RuntimeOrigin::signed(CouncilAccount::get()),
		Box::new(metadata.clone())
	));

	assert_eq!(CurrencyMetadatas::<Test>::get(WETH), Some(metadata.clone()));
}

fn basic_setup() {
	// Deposit some money in Alice, Bob and Charlie's accounts.
	// Alice
	assert_ok!(Currencies::deposit(BNC, &ALICE, 1000 * 10u128.pow(12)));
	assert_ok!(Currencies::deposit(DOT, &ALICE, 1000 * 10u128.pow(10)));
	assert_ok!(Currencies::deposit(VDOT, &ALICE, 1000 * 10u128.pow(10)));
	assert_ok!(Currencies::deposit(KSM, &ALICE, 1000 * 10u128.pow(12)));

	assert_ok!(Currencies::deposit(BNC, &DICK, 1000 * 10u128.pow(12)));
	assert_ok!(Currencies::deposit(DOT, &DICK, 1000 * 10u128.pow(10)));
	assert_ok!(Currencies::deposit(VDOT, &DICK, 1000 * 10u128.pow(10)));
	assert_ok!(Currencies::deposit(KSM, &DICK, 1000 * 10u128.pow(12)));

	// create DEX pair
	let para_id: u32 = 2001;
	let bnc_asset_id: AssetId = AssetId::try_convert_from(BNC, para_id).unwrap();
	let dot_asset_id: AssetId = AssetId::try_convert_from(DOT, para_id).unwrap();
	let vdot_asset_id: AssetId = AssetId::try_convert_from(VDOT, para_id).unwrap();
	let ksm_asset_id: AssetId = AssetId::try_convert_from(KSM, para_id).unwrap();

	assert_ok!(ZenlinkProtocol::create_pair(
		RuntimeOrigin::root(),
		bnc_asset_id,
		dot_asset_id,
		DICK
	));

	assert_ok!(ZenlinkProtocol::create_pair(
		RuntimeOrigin::root(),
		bnc_asset_id,
		vdot_asset_id,
		DICK
	));

	assert_ok!(ZenlinkProtocol::create_pair(
		RuntimeOrigin::root(),
		bnc_asset_id,
		ksm_asset_id,
		DICK
	));

	let deadline: BlockNumberFor<Test> =
		<frame_system::Pallet<Test>>::block_number() + BlockNumberFor::<Test>::from(100u32);

	// pool 0 2
	assert_ok!(ZenlinkProtocol::add_liquidity(
		RuntimeOrigin::signed(DICK),
		bnc_asset_id,
		dot_asset_id,
		100 * 10u128.pow(12),
		100 * 10u128.pow(10),
		1,
		1,
		deadline
	));

	assert_ok!(ZenlinkProtocol::add_liquidity(
		RuntimeOrigin::signed(DICK),
		bnc_asset_id,
		vdot_asset_id,
		100 * 10u128.pow(12),
		100 * 10u128.pow(10),
		1,
		1,
		deadline
	));

	assert_ok!(ZenlinkProtocol::add_liquidity(
		RuntimeOrigin::signed(DICK),
		bnc_asset_id,
		ksm_asset_id,
		100 * 10u128.pow(12),
		100 * 10u128.pow(12),
		1,
		1,
		deadline
	));
}

#[test]
fn set_user_default_fee_currency_should_work() {
	new_test_ext().execute_with(|| {
		let origin_signed_alice = RuntimeOrigin::signed(ALICE);
		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			origin_signed_alice.clone(),
			Some(BNC)
		));

		let alice_default_currency = UserDefaultFeeCurrency::<Test>::get(ALICE).unwrap();
		assert_eq!(alice_default_currency, BNC);

		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			origin_signed_alice.clone(),
			None
		));
		assert_eq!(UserDefaultFeeCurrency::<Test>::get(ALICE).is_none(), true);
	});
}

#[test]
fn set_user_default_fee_currency_should_fail_with_error_currency() {
	new_test_ext().execute_with(|| {
		let origin_signed_alice = RuntimeOrigin::signed(ALICE);
		assert_noop!(
			FlexibleFee::set_user_default_fee_currency(origin_signed_alice.clone(), Some(VBNC)),
			Error::<Test>::CurrencyNotSupport
		);

		assert_eq!(UserDefaultFeeCurrency::<Test>::get(ALICE).is_none(), true);
	});
}

#[test]
fn set_default_fee_currency_list_should_work() {
	new_test_ext().execute_with(|| {
		let asset_order_list_vec: BoundedVec<
			CurrencyId,
			<Test as Config>::MaxFeeCurrencyOrderListLen,
		> = BoundedVec::try_from(vec![KSM, VDOT, DOT, BNC]).unwrap();
		assert_noop!(
			FlexibleFee::set_default_fee_currency_list(
				RuntimeOrigin::signed(CHARLIE),
				asset_order_list_vec.clone()
			),
			BadOrigin
		);

		assert_ok!(FlexibleFee::set_default_fee_currency_list(
			RuntimeOrigin::root(),
			asset_order_list_vec.clone()
		));

		assert_eq!(
			crate::UniversalFeeCurrencyOrderList::<Test>::get(),
			asset_order_list_vec
		);
	});
}

#[test]
fn ensure_can_swap() {
	new_test_ext().execute_with(|| {
		basic_setup();
		assert_ok!(FlexibleFee::ensure_can_swap(&ALICE, BNC, DOT, 10));
	})
}

#[test]
fn withdraw_fee() {
	new_test_ext().execute_with(|| {
		basic_setup();
		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			RuntimeOrigin::signed(ALICE),
			Some(BNC)
		));
		assert_eq!(
			FlexibleFee::withdraw_fee(
				&ALICE,
				&BALANCE_TRANSFER_CALL,
				&info(),
				100 * 10u128.pow(12),
				0
			)
			.unwrap(),
			Some(PaymentInfo::Native(100 * 10u128.pow(12)))
		);
		assert_eq!(Currencies::free_balance(BNC, &ALICE), 900 * 10u128.pow(12));

		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			RuntimeOrigin::signed(ALICE),
			Some(DOT)
		));
		assert_eq!(
			FlexibleFee::withdraw_fee(
				&ALICE,
				&BALANCE_TRANSFER_CALL,
				&info(),
				100 * 10u128.pow(12),
				0
			)
			.unwrap(),
			Some(PaymentInfo::NonNative(
				4 * 10u128.pow(10),
				DOT,
				FixedU128::from_inner(200_000_000_000_000_000),
				FixedU128::from(5)
			))
		);
		assert_eq!(Currencies::free_balance(DOT, &ALICE), 996 * 10u128.pow(10));
	})
}

#[test]
fn withdraw_fee_with_universal_fee_currency() {
	new_test_ext().execute_with(|| {
		basic_setup();
		let fee = 100 * 10u128.pow(12);
		let info = info();
		assert_ok!(FlexibleFee::set_default_fee_currency_list(
			RuntimeOrigin::root(),
			BoundedVec::try_from(vec![BNC, DOT, MANTA]).unwrap()
		));

		assert_eq!(
			FlexibleFee::withdraw_fee(&ALICE, &BALANCE_TRANSFER_CALL, &info, fee, 0).unwrap(),
			Some(PaymentInfo::Native(fee))
		);
		assert_eq!(Currencies::free_balance(BNC, &ALICE), 900 * 10u128.pow(12));

		Currencies::set_balance(BNC, &ALICE, 0u128);
		assert_eq!(
			FlexibleFee::withdraw_fee(&ALICE, &BALANCE_TRANSFER_CALL, &info, fee, 0).unwrap(),
			Some(PaymentInfo::NonNative(
				4 * 10u128.pow(10),
				DOT,
				FixedU128::from_inner(200_000_000_000_000_000),
				FixedU128::from(5)
			))
		);
		assert_eq!(Currencies::free_balance(DOT, &ALICE), 996 * 10u128.pow(10));
	})
}

#[test]
fn withdraw_extra_fee() {
	new_test_ext().execute_with(|| {
		basic_setup();
		let fee = 100 * 10u128.pow(12);
		let info = info();
		assert_ok!(FlexibleFee::set_default_fee_currency_list(
			RuntimeOrigin::root(),
			BoundedVec::try_from(vec![BNC, DOT, MANTA]).unwrap()
		));

		assert_ok!(FlexibleFee::set_extra_fee(
			RuntimeOrigin::root(),
			BoundedVec::try_from(vec![10, 0]).unwrap(),
			Some((DOT, 1_000_000_000, DICK))
		));

		assert_eq!(
			FlexibleFee::withdraw_fee(&ALICE, &BALANCE_TRANSFER_CALL, &info, fee, 0).unwrap(),
			Some(PaymentInfo::Native(fee))
		);
		assert_eq!(Currencies::free_balance(BNC, &ALICE), 899899598695987);
	})
}

#[test]
fn correct_and_deposit_fee_should_work() {
	new_test_ext().execute_with(|| {
		basic_setup();

		let corrected_fee = 5 * 10u128.pow(12);
		let tip = 0;

		assert_eq!(Currencies::free_balance(BNC, &ALICE), 1000 * 10u128.pow(12));

		let already_withdrawn = Some(PaymentInfo::Native(10 * 10u128.pow(12)));
		assert_ok!(FlexibleFee::correct_and_deposit_fee(
			&ALICE,
			&info(),
			&post_info(),
			corrected_fee,
			tip,
			already_withdrawn
		));
		assert_eq!(Currencies::free_balance(BNC, &ALICE), 1005 * 10u128.pow(12));

		let corrected_fee = 10 * 10u128.pow(12);
		let tip = 0;
		assert_eq!(Currencies::free_balance(DOT, &ALICE), 1000 * 10u128.pow(10));

		let already_withdrawn = Some(PaymentInfo::NonNative(
			1 * 10u128.pow(10),
			DOT,
			FixedU128::from_inner(200_000_000_000_000_000),
			FixedU128::from(5),
		));
		assert_ok!(FlexibleFee::correct_and_deposit_fee(
			&ALICE,
			&info(),
			&post_info(),
			corrected_fee,
			tip,
			already_withdrawn
		));
		assert_eq!(Currencies::free_balance(DOT, &ALICE), 10006 * 10u128.pow(9));
	});
}

#[test]
fn correct_and_deposit_fee_with_tip() {
	new_test_ext().execute_with(|| {
		basic_setup();

		let corrected_fee = 5 * 10u128.pow(12);
		let tip = 5 * 10u128.pow(12);

		assert_eq!(Currencies::free_balance(BNC, &ALICE), 1000 * 10u128.pow(12));

		let already_withdrawn = Some(PaymentInfo::Native(10 * 10u128.pow(12)));
		assert_ok!(FlexibleFee::correct_and_deposit_fee(
			&ALICE,
			&info(),
			&post_info(),
			corrected_fee,
			tip,
			already_withdrawn
		));
		assert_eq!(Currencies::free_balance(BNC, &ALICE), 1005 * 10u128.pow(12));

		let corrected_fee = 10 * 10u128.pow(12);
		let tip = 10 * 10u128.pow(12);
		assert_eq!(Currencies::free_balance(DOT, &ALICE), 1000 * 10u128.pow(10));

		let already_withdrawn = Some(PaymentInfo::NonNative(
			1 * 10u128.pow(10),
			DOT,
			FixedU128::from_inner(200_000_000_000_000_000),
			FixedU128::from(5),
		));
		assert_ok!(FlexibleFee::correct_and_deposit_fee(
			&ALICE,
			&info(),
			&post_info(),
			corrected_fee,
			tip,
			already_withdrawn
		));
		assert_eq!(Currencies::free_balance(DOT, &ALICE), 10006 * 10u128.pow(9));
	});
}

#[test]
fn get_currency_asset_id_should_work() {
	new_test_ext().execute_with(|| {
		// BNC
		let asset_id = FlexibleFee::get_currency_asset_id(BNC).unwrap();
		let bnc_asset_id = AssetId {
			chain_id: 2001,
			asset_type: 0,
			asset_index: 0,
		};
		assert_eq!(asset_id, bnc_asset_id);

		// KSM
		let asset_id = FlexibleFee::get_currency_asset_id(KSM).unwrap();
		let ksm_asset_id = AssetId {
			chain_id: 2001,
			asset_type: 2,
			asset_index: 516,
		};
		assert_eq!(asset_id, ksm_asset_id);
	});
}

#[test]
fn get_fee_currency_should_work_with_default_currency() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		let origin_signed_alice = RuntimeOrigin::signed(ALICE);
		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			origin_signed_alice.clone(),
			Some(BNC)
		));

		assert_ok!(Currencies::deposit(BNC, &ALICE, 100u128.pow(12))); // BNC
		assert_ok!(Currencies::deposit(DOT, &ALICE, 100u128.pow(10))); // DOT
		assert_ok!(Currencies::deposit(VDOT, &ALICE, 100u128.pow(10))); // vDOT
		assert_ok!(Currencies::deposit(KSM, &ALICE, 100u128.pow(12))); // KSM CurrencyNotSupport
		assert_ok!(Currencies::deposit(WETH, &ALICE, 100u128.pow(18))); // ETH

		let currency = FlexibleFee::get_fee_currency(&ALICE, 10u128.pow(18).into()).unwrap();
		assert_eq!(currency, BNC);
	});
}

#[test]
fn get_fee_currency_should_work_with_default_currency_poor() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		let origin_signed_alice = RuntimeOrigin::signed(ALICE);
		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			origin_signed_alice.clone(),
			Some(BNC)
		));

		assert_ok!(Currencies::deposit(BNC, &ALICE, 1u128.pow(12))); // BNC
		assert_ok!(Currencies::deposit(DOT, &ALICE, 100u128.pow(10))); // DOT
		assert_ok!(Currencies::deposit(VDOT, &ALICE, 100u128.pow(10))); // vDOT
		assert_ok!(Currencies::deposit(KSM, &ALICE, 100u128.pow(12))); // KSM CurrencyNotSupport
		assert_ok!(Currencies::deposit(WETH, &ALICE, 100u128.pow(18))); // ETH

		let currency = FlexibleFee::get_fee_currency(&ALICE, 10u128.pow(18).into()).unwrap();
		assert_eq!(currency, WETH);
	});
}

#[test]
fn get_fee_currency_should_work_with_weth() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		assert_ok!(Currencies::deposit(BNC, &ALICE, 100u128.pow(12))); // BNC
		assert_ok!(Currencies::deposit(DOT, &ALICE, 100u128.pow(10))); // DOT
		assert_ok!(Currencies::deposit(VDOT, &ALICE, 100u128.pow(10))); // vDOT
		assert_ok!(Currencies::deposit(KSM, &ALICE, 100u128.pow(12))); // KSM CurrencyNotSupport
		assert_ok!(Currencies::deposit(WETH, &ALICE, 100u128.pow(18))); // ETH

		let currency = FlexibleFee::get_fee_currency(&ALICE, 10u128.pow(18).into()).unwrap();
		assert_eq!(currency, WETH);
	});
}

#[test]
fn get_fee_currency_should_work_with_weth_poor() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		assert_ok!(Currencies::deposit(BNC, &ALICE, 100u128.pow(12))); // BNC
		assert_ok!(Currencies::deposit(DOT, &ALICE, 100u128.pow(10))); // DOT
		assert_ok!(Currencies::deposit(VDOT, &ALICE, 100u128.pow(10))); // vDOT
		assert_ok!(Currencies::deposit(KSM, &ALICE, 100u128.pow(12))); // KSM CurrencyNotSupport
		assert_ok!(Currencies::deposit(WETH, &ALICE, 1u128.pow(18))); // ETH

		let asset_order_list_vec: BoundedVec<
			CurrencyId,
			<Test as Config>::MaxFeeCurrencyOrderListLen,
		> = BoundedVec::try_from(vec![VDOT, DOT, BNC]).unwrap();

		assert_ok!(FlexibleFee::set_default_fee_currency_list(
			RuntimeOrigin::root(),
			asset_order_list_vec.clone()
		));

		let currency = FlexibleFee::get_fee_currency(&ALICE, 10u128.pow(18).into()).unwrap();
		assert_eq!(currency, VDOT);
	});
}

#[test]
fn get_fee_currency_should_work_with_universal_fee_currency() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		let origin_signed_alice = RuntimeOrigin::signed(ALICE);
		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			origin_signed_alice.clone(),
			Some(BNC)
		));

		assert_ok!(Currencies::deposit(BNC, &ALICE, 1u128.pow(12))); // BNC
		assert_ok!(Currencies::deposit(DOT, &ALICE, 100u128.pow(10))); // DOT
		assert_ok!(Currencies::deposit(VDOT, &ALICE, 100u128.pow(10))); // vDOT
		assert_ok!(Currencies::deposit(KSM, &ALICE, 100u128.pow(12))); // KSM CurrencyNotSupport
		assert_ok!(Currencies::deposit(WETH, &ALICE, 1u128.pow(18))); // ETH

		let asset_order_list_vec: BoundedVec<
			CurrencyId,
			<Test as Config>::MaxFeeCurrencyOrderListLen,
		> = BoundedVec::try_from(vec![VDOT, DOT, BNC]).unwrap();

		assert_ok!(FlexibleFee::set_default_fee_currency_list(
			RuntimeOrigin::root(),
			asset_order_list_vec.clone()
		));

		let currency = FlexibleFee::get_fee_currency(&ALICE, 10u128.pow(18).into()).unwrap();
		assert_eq!(currency, VDOT);
	});
}

#[test]
fn get_fee_currency_should_work_with_universal_fee_currency_poor() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		assert_ok!(Currencies::deposit(BNC, &ALICE, 1u128.pow(12))); // BNC
		assert_ok!(Currencies::deposit(DOT, &ALICE, 100u128.pow(10))); // DOT
		assert_ok!(Currencies::deposit(VDOT, &ALICE, 1u128.pow(10))); // vDOT
		assert_ok!(Currencies::deposit(KSM, &ALICE, 100u128.pow(12))); // KSM CurrencyNotSupport
		assert_ok!(Currencies::deposit(WETH, &ALICE, 1u128.pow(18))); // ETH

		let asset_order_list_vec: BoundedVec<
			CurrencyId,
			<Test as Config>::MaxFeeCurrencyOrderListLen,
		> = BoundedVec::try_from(vec![VDOT, DOT, BNC]).unwrap();

		assert_ok!(FlexibleFee::set_default_fee_currency_list(
			RuntimeOrigin::root(),
			asset_order_list_vec.clone()
		));

		let currency = FlexibleFee::get_fee_currency(&ALICE, 10u128.pow(18).into()).unwrap();
		assert_eq!(currency, DOT);
	});
}

#[test]
fn get_fee_currency_should_work_with_all_currency_poor() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		let origin_signed_alice = RuntimeOrigin::signed(ALICE);
		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			origin_signed_alice.clone(),
			Some(BNC)
		));

		assert_ok!(Currencies::deposit(BNC, &ALICE, 7u128.pow(12))); // BNC
		assert_ok!(Currencies::deposit(DOT, &ALICE, 5u128.pow(10))); // DOT
		assert_ok!(Currencies::deposit(VDOT, &ALICE, 4u128.pow(10))); // vDOT
		assert_ok!(Currencies::deposit(KSM, &ALICE, 3u128.pow(12))); // KSM CurrencyNotSupport
		assert_ok!(Currencies::deposit(WETH, &ALICE, 2u128.pow(18))); // ETH

		let asset_order_list_vec: BoundedVec<
			CurrencyId,
			<Test as Config>::MaxFeeCurrencyOrderListLen,
		> = BoundedVec::try_from(vec![VDOT, DOT, BNC]).unwrap();

		assert_ok!(FlexibleFee::set_default_fee_currency_list(
			RuntimeOrigin::root(),
			asset_order_list_vec.clone()
		));

		let currency = FlexibleFee::get_fee_currency(&ALICE, 10u128.pow(18).into()).unwrap();
		assert_eq!(currency, BNC);
	});
}

#[test]
fn cmp_with_precision_should_work_with_weth() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		assert_ok!(Currencies::deposit(WETH, &ALICE, 10u128.pow(18) - 1)); // ETH

		let ordering =
			FlexibleFee::cmp_with_precision(&ALICE, &WETH, 10u128.pow(18), 18u32).unwrap();
		assert_eq!(ordering, Less);
	});
}

#[test]
fn cmp_with_precision_should_work_with_dot() {
	new_test_ext().execute_with(|| {
		ini_meta_data();

		assert_ok!(Currencies::deposit(DOT, &ALICE, 10u128.pow(11) + 1)); // DOT

		let ordering =
			FlexibleFee::cmp_with_precision(&ALICE, &DOT, 10u128.pow(18), 18u32).unwrap();
		assert_eq!(ordering, Greater);
	});
}

#[test]
fn cmp_with_precision_should_work_with_bnc() {
	new_test_ext().execute_with(|| {
		assert_ok!(Currencies::deposit(BNC, &ALICE, 11u128.pow(12))); // BNC

		let ordering =
			FlexibleFee::cmp_with_precision(&ALICE, &BNC, 10u128.pow(18), 18u32).unwrap();
		assert_eq!(ordering, Greater);
	});
}

#[test]
fn validate_unsigned_should_correctly_call_validate_handler() {
	let alice_evm_address = EVMAccounts::evm_address(&ALICE);
	let other_evm_address = EVMAccounts::evm_address(&BOB);

	new_test_ext().execute_with(|| {
		let r: [u8; 32] = [100; 32];
		let s: [u8; 32] = [200; 32];

		let call = crate::Call::dispatch_permit {
			from: alice_evm_address,
			to: other_evm_address,
			data: b"test".to_vec(),
			value: U256::from(1234),
			gas_limit: 123,
			deadline: U256::from(99999),
			v: 255,
			r: H256::from(r),
			s: H256::from(s),
		};

		assert_storage_noop!({
			let res = FlexibleFee::validate_unsigned(TransactionSource::Local, &call);
			assert_ok!(res);
		});

		let expected = ValidationData {
			source: alice_evm_address,
			target: other_evm_address,
			input: b"test".to_vec(),
			value: U256::from(1234),
			gas_limit: 123,
			deadline: U256::from(99999),
			v: 255,
			r: H256::from(r),
			s: H256::from(s),
		};

		assert_eq!(PermitDispatchHandler::last_validation_call_data(), expected);
	});
}

#[test]
fn validate_unsigned_should_correctly_dry_run_dispatch() {
	let alice_evm_address = EVMAccounts::evm_address(&ALICE);
	let other_evm_address = EVMAccounts::evm_address(&BOB);

	new_test_ext().execute_with(|| {
		let r: [u8; 32] = [100; 32];
		let s: [u8; 32] = [200; 32];

		let call = crate::Call::dispatch_permit {
			from: alice_evm_address,
			to: other_evm_address,
			data: b"test".to_vec(),
			value: U256::from(1234),
			gas_limit: 123,
			deadline: U256::from(99999),
			v: 255,
			r: H256::from(r),
			s: H256::from(s),
		};

		assert_storage_noop!({
			let res = FlexibleFee::validate_unsigned(TransactionSource::Local, &call);
			assert_ok!(res);
		});

		let expected = PermitDispatchData {
			source: alice_evm_address,
			target: other_evm_address,
			input: b"test".to_vec(),
			value: U256::from(1234),
			gas_limit: 123,
			max_fee_per_gas: U256::from(222u128),
			max_priority_fee_per_gas: None,
			nonce: None,
			access_list: vec![],
		};

		assert_eq!(PermitDispatchHandler::last_dispatch_call_data(), expected);
	});
}

#[test]
fn dispatch_should_correctly_call_validate_and_dispatch() {
	let alice_evm_address = EVMAccounts::evm_address(&ALICE);
	let other_evm_address = EVMAccounts::evm_address(&BOB);

	new_test_ext().execute_with(|| {
		let r: [u8; 32] = [50; 32];
		let s: [u8; 32] = [100; 32];

		assert_ok!(FlexibleFee::dispatch_permit(
			RuntimeOrigin::none(),
			alice_evm_address,
			other_evm_address,
			U256::from(1234),
			b"test".to_vec(),
			333,
			U256::from(99999u128),
			128,
			H256::from(r),
			H256::from(s),
		));

		let expected = ValidationData {
			source: alice_evm_address,
			target: other_evm_address,
			input: b"test".to_vec(),
			value: U256::from(1234),
			gas_limit: 333,
			deadline: U256::from(99999u128),
			v: 128,
			r: H256::from(r),
			s: H256::from(s),
		};

		assert_eq!(PermitDispatchHandler::last_validation_call_data(), expected);

		let expected = PermitDispatchData {
			source: alice_evm_address,
			target: other_evm_address,
			input: b"test".to_vec(),
			value: U256::from(1234),
			gas_limit: 333,
			max_fee_per_gas: U256::from(222u128),
			max_priority_fee_per_gas: None,
			nonce: None,
			access_list: vec![],
		};

		assert_eq!(PermitDispatchHandler::last_dispatch_call_data(), expected);
	});
}
