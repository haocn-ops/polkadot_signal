#![cfg(test)]

use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_identity_works() {
	new_test_ext().execute_with(|| {
		let identity_key = vec![1u8; 32];
		let signed_prekey = vec![2u8; 32];
		let prekey_signature = vec![3u8; 64];

		assert_ok!(SignalKeys::register_identity(
			RuntimeOrigin::signed(1),
			identity_key.clone(),
			signed_prekey.clone(),
			prekey_signature.clone()
		));

		let bundle = SignalKeys::get_identity(&1).unwrap();
		assert_eq!(bundle.identity_key.to_vec(), identity_key);
		assert_eq!(bundle.signed_prekey.to_vec(), signed_prekey);
	});
}

#[test]
fn register_identity_twice_updates() {
	new_test_ext().execute_with(|| {
		assert_ok!(SignalKeys::register_identity(
			RuntimeOrigin::signed(1),
			vec![1u8; 32],
			vec![2u8; 32],
			vec![3u8; 64]
		));

		let new_identity_key = vec![9u8; 32];
		assert_ok!(SignalKeys::register_identity(
			RuntimeOrigin::signed(1),
			new_identity_key.clone(),
			vec![2u8; 32],
			vec![3u8; 64]
		));

		let bundle = SignalKeys::get_identity(&1).unwrap();
		assert_eq!(bundle.identity_key.to_vec(), new_identity_key);
	});
}

#[test]
fn register_identity_fails_with_large_key() {
	new_test_ext().execute_with(|| {
		let large_key = vec![0u8; 300];
		assert_noop!(
			SignalKeys::register_identity(
				RuntimeOrigin::signed(1),
				large_key,
				vec![2u8; 32],
				vec![3u8; 64]
			),
			Error::<Test>::IdentityKeyTooLarge
		);
	});
}

#[test]
fn add_one_time_prekeys_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(SignalKeys::register_identity(
			RuntimeOrigin::signed(1),
			vec![1u8; 32],
			vec![2u8; 32],
			vec![3u8; 64]
		));

		let prekeys: Vec<Vec<u8>> = (0..5).map(|i| vec![i as u8; 32]).collect();
		assert_ok!(SignalKeys::add_one_time_prekeys(
			RuntimeOrigin::signed(1),
			prekeys
		));

		assert_eq!(SignalKeys::get_remaining_prekey_count(&1), 5);
	});
}

#[test]
fn add_one_time_prekeys_fails_without_identity() {
	new_test_ext().execute_with(|| {
		let prekeys: Vec<Vec<u8>> = vec![vec![0u8; 32]];
		assert_noop!(
			SignalKeys::add_one_time_prekeys(RuntimeOrigin::signed(1), prekeys),
			Error::<Test>::IdentityNotFound
		);
	});
}

#[test]
fn get_one_time_prekey_consumes_key() {
	new_test_ext().execute_with(|| {
		assert_ok!(SignalKeys::register_identity(
			RuntimeOrigin::signed(1),
			vec![1u8; 32],
			vec![2u8; 32],
			vec![3u8; 64]
		));

		let prekeys: Vec<Vec<u8>> = vec![vec![10u8; 32], vec![11u8; 32]];
		assert_ok!(SignalKeys::add_one_time_prekeys(
			RuntimeOrigin::signed(1),
			prekeys
		));

		assert_eq!(SignalKeys::get_remaining_prekey_count(&1), 2);

		let first = SignalKeys::get_one_time_prekey(&1);
		assert!(first.is_some());
		assert_eq!(SignalKeys::get_remaining_prekey_count(&1), 1);

		let second = SignalKeys::get_one_time_prekey(&1);
		assert!(second.is_some());
		assert_eq!(SignalKeys::get_remaining_prekey_count(&1), 0);

		let third = SignalKeys::get_one_time_prekey(&1);
		assert!(third.is_none());
	});
}

#[test]
fn remove_identity_clears_all_keys() {
	new_test_ext().execute_with(|| {
		assert_ok!(SignalKeys::register_identity(
			RuntimeOrigin::signed(1),
			vec![1u8; 32],
			vec![2u8; 32],
			vec![3u8; 64]
		));

		let prekeys: Vec<Vec<u8>> = vec![vec![0u8; 32], vec![1u8; 32]];
		assert_ok!(SignalKeys::add_one_time_prekeys(
			RuntimeOrigin::signed(1),
			prekeys
		));

		assert!(SignalKeys::get_identity(&1).is_some());
		assert_eq!(SignalKeys::get_remaining_prekey_count(&1), 2);

		assert_ok!(SignalKeys::remove_identity(RuntimeOrigin::signed(1)));

		assert!(SignalKeys::get_identity(&1).is_none());
		assert_eq!(SignalKeys::get_remaining_prekey_count(&1), 0);
	});
}

#[test]
fn remove_identity_fails_if_not_registered() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SignalKeys::remove_identity(RuntimeOrigin::signed(1)),
			Error::<Test>::IdentityNotFound
		);
	});
}

#[test]
fn key_bundle_count_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(SignalKeys::key_bundle_count(), 0);

		assert_ok!(SignalKeys::register_identity(
			RuntimeOrigin::signed(1),
			vec![1u8; 32],
			vec![2u8; 32],
			vec![3u8; 64]
		));
		assert_eq!(SignalKeys::key_bundle_count(), 1);

		assert_ok!(SignalKeys::register_identity(
			RuntimeOrigin::signed(2),
			vec![1u8; 32],
			vec![2u8; 32],
			vec![3u8; 64]
		));
		assert_eq!(SignalKeys::key_bundle_count(), 2);

		assert_ok!(SignalKeys::remove_identity(RuntimeOrigin::signed(1)));
		assert_eq!(SignalKeys::key_bundle_count(), 1);
	});
}
