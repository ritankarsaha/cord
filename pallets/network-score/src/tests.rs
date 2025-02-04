// This file is part of CORD – https://cord.network

// Copyright (C) Dhiway Networks Pvt. Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// CORD is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// CORD is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with CORD. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use crate::mock::*;
use codec::Encode;
use cord_utilities::mock::{mock_origin::DoubleOrigin, SubjectId};
use frame_support::{assert_err, assert_ok, BoundedVec};
use frame_system::RawOrigin;
use pallet_chain_space::SpaceCodeOf;
use sp_runtime::{traits::Hash, AccountId32};
use sp_std::prelude::*;

pub fn generate_rating_id<T: Config>(digest: &RatingEntryHashOf<T>) -> RatingEntryIdOf {
	Ss58Identifier::create_identifier(&(digest).encode()[..], IdentifierType::Rating).unwrap()
}

pub fn generate_space_id<T: Config>(digest: &SpaceCodeOf<T>) -> SpaceIdOf {
	Ss58Identifier::create_identifier(&(digest).encode()[..], IdentifierType::Space).unwrap()
}

pub(crate) const DID_00: SubjectId = SubjectId(AccountId32::new([1u8; 32]));
pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);

#[test]
fn check_successful_rating_creation() {
	let creator = DID_00;
	let author = ACCOUNT_00;

	let message_id = BoundedVec::try_from([72u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id,
		provider_id,
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);

	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);

	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		// Author Transaction

		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest,
		));

		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry,
			entry_digest,
			message_id,
			authorization_id,
		));
	});
}

#[test]
fn register_rating_with_invalid_data_should_fail() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();

	let invalid_message_id = BoundedVec::try_from([0u8; 0].to_vec()).unwrap(); // Invalid message ID (empty)
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let invalid_entry = RatingInputEntryOf::<Test> {
		entity_id,
		provider_id,
		total_encoded_rating: 0u64, // Invalid rating (0 value)
		count_of_txn: 0u64,         // Invalid transaction count (0)
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&invalid_entry.encode()[..]].concat()[..]);

	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);

	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Create space
		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest
		));

		// Approve space
		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		// Try registering rating with invalid data
		assert_err!(
			Score::register_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				invalid_entry.clone(),
				entry_digest,
				invalid_message_id.clone(),
				authorization_id.clone()
			),
			Error::<Test>::InvalidRatingValue
		);
	});
}

#[test]
fn revise_rating_with_invalid_values_should_fail() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();

	let message_id = BoundedVec::try_from([72u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id,
		provider_id,
		total_encoded_rating: 250u64, // Initially valid rating
		count_of_txn: 7u64,           // Initially valid transaction count
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);

	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);

	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	let id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[
			&entry_digest.encode()[..],
			&entry.entity_id.encode()[..],
			&message_id.encode()[..],
			&space_id.encode()[..],
			&creator.clone().encode()[..],
		]
		.concat()[..],
	);

	let rating_identifier =
		Ss58Identifier::create_identifier(&(id_digest).encode()[..], IdentifierType::Rating)
			.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest,
		));

		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		// Modify the entry for revision with invalid values
		let mut revised_entry = entry.clone();
		revised_entry.total_encoded_rating = 0u64; // Invalid rating
		revised_entry.count_of_txn = 0u64; // Invalid transaction count

		let revised_entry_digest = <Test as frame_system::Config>::Hashing::hash(
			&[&revised_entry.encode()[..]].concat()[..],
		);

		assert_err!(
			Score::revise_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				revised_entry.clone(),
				revised_entry_digest,
				message_id.clone(),
				rating_identifier.clone(),
				authorization_id.clone(),
			),
			Error::<Test>::InvalidRatingValue
		);
	});
}

#[test]
fn check_duplicate_message_id() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();

	let message_id = BoundedVec::try_from([72u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id,
		provider_id,
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);

	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);

	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		// Author Transaction

		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest,
		));

		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		// Register the rating entry once
		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		//error as if found the same message id
		assert_err!(
			Score::register_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				entry,
				entry_digest,
				message_id,
				authorization_id,
			),
			Error::<Test>::MessageIdAlreadyExists
		);
	});
}

#[test]
fn revise_rating_with_entry_entity_mismatch_should_fail() {
	let creator = DID_00;
	let author = ACCOUNT_00;

	let message_id = BoundedVec::try_from([72u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id: entity_id.clone(),
		provider_id,
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);

	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);

	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	let id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[
			&entry_digest.encode()[..],
			&entity_id.encode()[..],
			&message_id.encode()[..],
			&space_id.encode()[..],
			&creator.clone().encode()[..],
		]
		.concat()[..],
	);

	let identifier =
		Ss58Identifier::create_identifier(&(id_digest).encode()[..], IdentifierType::Rating);

	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		// Author Transaction

		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest,
		));

		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		//error
		let mut mismatched_entry = entry.clone();
		mismatched_entry.entity_id = BoundedVec::try_from([80u8; 10].to_vec()).unwrap();
		let mismatched_entry_digest =
			<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);
		assert_err!(
			Score::revise_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				mismatched_entry.clone(),
				mismatched_entry_digest,
				message_id.clone(),
				identifier.unwrap(),
				authorization_id.clone(),
			),
			Error::<Test>::EntityMismatch
		);
	});
}

#[test]
fn register_rating_with_existing_rating_identifier_should_fail() {
	// Define test parameters
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();
	let message_id = BoundedVec::try_from([72u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id,
		provider_id,
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);
	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);
	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Create a space
		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest
		));
		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		// Register the rating entry once
		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		// Remove message_id and provider_did from entries
		<MessageIdentifiers<Test>>::remove(message_id.clone(), creator.clone());
		// Attempt to register another rating entry with the same identifier
		assert_err!(
			Score::register_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				entry.clone(),
				entry_digest,
				message_id.clone(),
				authorization_id.clone(),
			),
			Error::<Test>::RatingIdentifierAlreadyAdded
		);
	});
}

#[test]
fn revoke_rating_with_existing_rating_identifier_should_fail() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();
	let message_id = BoundedVec::try_from([82u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id,
		provider_id,
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);
	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);
	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	let id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[
			&entry_digest.encode()[..],
			&entry.entity_id.encode()[..],
			&message_id.encode()[..],
			&space_id.encode()[..],
			&creator.clone().encode()[..],
		]
		.concat()[..],
	);

	let identifier =
		Ss58Identifier::create_identifier(&(id_digest).encode()[..], IdentifierType::Rating);

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest
		));
		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		<MessageIdentifiers<Test>>::remove(message_id.clone(), creator.clone());

		assert_err!(
			Score::revoke_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				identifier.unwrap(),
				message_id.clone(),
				entry_digest,
				authorization_id.clone(),
			),
			Error::<Test>::RatingIdentifierAlreadyAdded
		);
	});
}

#[test]
fn revise_rating_with_existing_rating_identifier_should_fail() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();
	let message_id = BoundedVec::try_from([82u8; 10].to_vec()).unwrap();
	let message_id_revise: BoundedVec<u8, MaxEncodedValueLength> =
		BoundedVec::try_from([75u8; 10].to_vec()).unwrap();
	let message_id_revoke = BoundedVec::try_from([84u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id: entity_id.clone(),
		provider_id: provider_id.clone(),
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);

	let entry_revise = RatingInputEntryOf::<Test> {
		entity_id: entity_id.clone(),
		provider_id: provider_id.clone(),
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_revise_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry_revise.encode()[..]].concat()[..]);

	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);
	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();
	let id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[
			&entry_digest.encode()[..],
			&entry.entity_id.encode()[..],
			&message_id.encode()[..],
			&space_id.encode()[..],
			&creator.clone().encode()[..],
		]
		.concat()[..],
	);

	let identifier =
		Ss58Identifier::create_identifier(&(id_digest).encode()[..], IdentifierType::Rating)
			.unwrap();
	let id_digest_revise = <Test as frame_system::Config>::Hashing::hash(
		&[
			&entry_revise_digest.encode()[..],
			&entry_revise.entity_id.encode()[..],
			&message_id_revoke.encode()[..],
			&space_id.encode()[..],
			&creator.clone().encode()[..],
		]
		.concat()[..],
	);
	let identifier_add =
		Ss58Identifier::create_identifier(&(id_digest_revise).encode()[..], IdentifierType::Rating)
			.unwrap();
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Create a space
		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest
		));
		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 5u64));

		// Register the rating entry once
		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		// Revoke the rating to create a debit entry
		assert_ok!(Score::revoke_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			identifier.clone(),
			message_id_revoke.clone(),
			entry_digest,
			authorization_id.clone()
		));

		// Revise rating to create a new rating for the entity
		assert_ok!(Score::revise_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry_revise.clone(),
			entry_revise_digest,
			message_id_revise.clone(),
			identifier_add.clone(),
			authorization_id.clone(),
		));

		// // Remove the messgae_id from list to reach `RatingIdentifierAlreadyAdded` block.
		<MessageIdentifiers<Test>>::remove(message_id_revise.clone(), creator.clone());

		// // Error when revising rating again with same rating identifier
		assert_err!(
			Score::revise_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				entry_revise.clone(),
				entry_revise_digest,
				message_id_revise.clone(),
				identifier_add.clone(),
				authorization_id.clone(),
			),
			Error::<Test>::RatingIdentifierAlreadyAdded
		);
	})
}

#[test]
fn reference_identifier_not_debit_test() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();
	let message_id = BoundedVec::try_from([82u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id,
		provider_id,
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);
	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);
	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	let id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[
			&entry_digest.encode()[..],
			&entry.entity_id.encode()[..],
			&message_id.encode()[..],
			&space_id.encode()[..],
			&creator.clone().encode()[..],
		]
		.concat()[..],
	);

	let identifier =
		Ss58Identifier::create_identifier(&(id_digest).encode()[..], IdentifierType::Rating)
			.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest
		));
		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		<MessageIdentifiers<Test>>::remove(message_id.clone(), creator.clone());

		assert_err!(
			Score::revise_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				entry.clone(),
				entry_digest,
				message_id.clone(),
				identifier.clone(),
				authorization_id.clone(),
			),
			Error::<Test>::ReferenceNotDebitIdentifier
		);
	});
}

#[test]
fn rating_identifier_not_found_test() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();
	let message_id = BoundedVec::try_from([82u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();
	let entry = RatingInputEntryOf::<Test> {
		entity_id,
		provider_id,
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};
	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);
	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);
	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	let id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[
			&entry_digest.encode()[..],
			&entry.entity_id.encode()[..],
			&message_id.encode()[..],
			&space_id.encode()[..],
			&creator.clone().encode()[..],
		]
		.concat()[..],
	);

	let identifier =
		Ss58Identifier::create_identifier(&(id_digest).encode()[..], IdentifierType::Rating)
			.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest
		));
		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		<MessageIdentifiers<Test>>::remove(message_id.clone(), creator.clone());
		<RatingEntries<Test>>::remove(identifier.clone());

		assert_err!(
			Score::revoke_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				identifier,
				message_id.clone(),
				entry_digest,
				authorization_id.clone(),
			),
			Error::<Test>::RatingIdentifierNotFound
		);
	});
}

#[test]
fn reference_identifier_not_found_test() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();
	let message_id = BoundedVec::try_from([82u8; 10].to_vec()).unwrap();
	let message_id_revise = BoundedVec::try_from([75u8; 10].to_vec()).unwrap();
	let message_id_revoke = BoundedVec::try_from([84u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();

	let entry = RatingInputEntryOf::<Test> {
		entity_id: entity_id.clone(),
		provider_id: provider_id.clone(),
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};

	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);

	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);
	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	let id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[
			&entry_digest.encode()[..],
			&entry.entity_id.encode()[..],
			&message_id.encode()[..],
			&space_id.encode()[..],
			&creator.encode()[..],
		]
		.concat()[..],
	);
	let identifier =
		Ss58Identifier::create_identifier(&(id_digest).encode()[..], IdentifierType::Rating)
			.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Create a space
		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest
		));
		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 5u64));

		// Register the rating entry once
		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		assert_ok!(Score::revoke_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			identifier.clone(),
			message_id_revoke.clone(),
			entry_digest,
			authorization_id.clone()
		));

		// Removed message id from the MessageIdentifiers storage
		<MessageIdentifiers<Test>>::remove(message_id_revise.clone(), creator.clone());

		// Remove the rating entry manually to simulate a missing reference
		<RatingEntries<Test>>::remove(identifier.clone());

		// Try to revise the rating again (this should now trigger `ReferenceIdentifierNotFound`)

		assert_err!(
			Score::revise_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				entry.clone(),
				entry_digest,
				message_id_revise.clone(),
				identifier.clone(),
				authorization_id.clone(),
			),
			Error::<Test>::ReferenceIdentifierNotFound
		);

		<MessageIdentifiers<Test>>::remove(message_id_revise.clone(), creator.clone());
	});
}

#[test]
fn revise_rating_with_space_mismatch_should_fail() {
	let creator = DID_00.clone();
	let author = ACCOUNT_00.clone();

	let message_id = BoundedVec::try_from([72u8; 10].to_vec()).unwrap();
	let entity_id = BoundedVec::try_from([73u8; 10].to_vec()).unwrap();
	let provider_id = BoundedVec::try_from([74u8; 10].to_vec()).unwrap();

	// Valid rating entry
	let entry = RatingInputEntryOf::<Test> {
		entity_id: entity_id.clone(),
		provider_id: provider_id.clone(),
		total_encoded_rating: 250u64,
		count_of_txn: 7u64,
		rating_type: RatingTypeOf::Overall,
		provider_did: creator.clone(),
	};

	let entry_digest =
		<Test as frame_system::Config>::Hashing::hash(&[&entry.encode()[..]].concat()[..]);

	// Create "main" space:
	let raw_space = [2u8; 256].to_vec();
	let space_digest = <Test as frame_system::Config>::Hashing::hash(&raw_space.encode()[..]);
	let space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let space_id: SpaceIdOf = generate_space_id::<Test>(&space_id_digest);

	// Create "mismatch" space:
	let mismatch_raw_space = [3u8; 256].to_vec();
	let mismatch_space_digest =
		<Test as frame_system::Config>::Hashing::hash(&mismatch_raw_space.encode()[..]);
	let mismatch_space_id_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&mismatch_space_digest.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let mismatch_space_id: SpaceIdOf = generate_space_id::<Test>(&mismatch_space_id_digest);

	// Auth ID for main space
	let auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let authorization_id: AuthorizationIdOf =
		Ss58Identifier::create_identifier(&auth_digest.encode()[..], IdentifierType::Authorization)
			.unwrap();

	// Auth ID for mismatch space
	let mismatch_auth_digest = <Test as frame_system::Config>::Hashing::hash(
		&[&mismatch_space_id.encode()[..], &creator.encode()[..], &creator.encode()[..]].concat()[..],
	);
	let mismatch_authorization_id: AuthorizationIdOf = Ss58Identifier::create_identifier(
		&mismatch_auth_digest.encode()[..],
		IdentifierType::Authorization,
	)
	.unwrap();

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Create & approve main space
		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			space_digest
		));
		assert_ok!(Space::approve(RawOrigin::Root.into(), space_id, 3u64));

		// Register a 'credit' rating in main space
		assert_ok!(Score::register_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			entry.clone(),
			entry_digest,
			message_id.clone(),
			authorization_id.clone(),
		));

		// Create & approve mismatch space
		assert_ok!(Space::create(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			mismatch_space_digest
		));
		assert_ok!(Space::approve(RawOrigin::Root.into(), mismatch_space_id, 3u64));

		// Revoke the original rating to produce a 'debit' entry
		let revoke_msg_id = BoundedVec::try_from([80u8; 10].to_vec()).unwrap();
		let revoke_digest = <Test as frame_system::Config>::Hashing::hash(&[50u8; 16].to_vec()[..]);
		let orig_rating_id = <MessageIdentifiers<Test>>::get(&message_id, &creator).unwrap();

		assert_ok!(Score::revoke_rating(
			DoubleOrigin(author.clone(), creator.clone()).into(),
			orig_rating_id.clone(),
			revoke_msg_id.clone(),
			revoke_digest,
			authorization_id.clone(),
		));

		// Attempt to revise using the mismatch space
		let mismatch_msg_id = BoundedVec::try_from([88u8; 10].to_vec()).unwrap();
		let mismatch_digest =
			<Test as frame_system::Config>::Hashing::hash(&[90u8; 16].to_vec()[..]);

		// This call should fail with SpaceMismatch, because the rating was made in 'space_id',
		// but we pass 'mismatch_authorization_id' that points to 'mismatch_space_id'.
		assert_err!(
			Score::revise_rating(
				DoubleOrigin(author.clone(), creator.clone()).into(),
				entry.clone(),
				mismatch_digest,
				mismatch_msg_id,
				orig_rating_id.clone(),
				mismatch_authorization_id,
			),
			Error::<Test>::SpaceMismatch
		);
	});
}
