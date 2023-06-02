use crate::{mock::*, tests::RuntimeEvent::KittiesModule as KM, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn t_create() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		//
		assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
		assert!(KittiesModule::kitties(kitty_id).is_some());
		assert!(KittiesModule::kitty_parents(kitty_id).is_none());

		crate::NextKittyId::<Test>::set(crate::KittyId::max_value());

		assert_noop!(
			KittiesModule::create(RuntimeOrigin::signed(account_id)),
			Error::<Test>::InvalidKittyId,
		);

		//
		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		println!("~~~ Event: {:?}", event);

		match event {
			KM(Event::KittyCreated { who, kitty_id: event_kitty_id, kitty: _ }) => {
				assert_eq!(who, account_id);
				assert_eq!(event_kitty_id, kitty_id);
			},
			_ => panic!("not exepected event"),
		}
	})
}

#[test]
fn t_breed() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		//
		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id),
			Error::<Test>::SameKittyId,
		);

		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1),
			Error::<Test>::InvalidKittyId,
		);

		//
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);

		assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1));

		//
		let breed_kitty_id = 2;
		assert_eq!(KittiesModule::next_kitty_id(), breed_kitty_id + 1);
		assert!(KittiesModule::kitties(breed_kitty_id).is_some());
		assert_eq!(KittiesModule::kitty_parents(breed_kitty_id), Some((kitty_id, kitty_id + 1)));

		//
		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		println!("~~~ Event: {:?}", event);

		match event {
			KM(Event::KittyBreed { who, kitty_id: event_kitty_id, kitty: _ }) => {
				assert_eq!(who, account_id);
				assert_eq!(event_kitty_id, kitty_id + 2);
			},
			_ => panic!("not exepected event"),
		}
	})
}

#[test]
fn t_transfer() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let recipient = 2;

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(recipient), kitty_id, recipient),
			Error::<Test>::NotOwner,
		);

		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(account_id), kitty_id, recipient));
		System::assert_last_event(
			Event::KittyTransfer { who: account_id, recipient, kitty_id }.into(),
		);
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(recipient));

		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(recipient), kitty_id, account_id));
		System::assert_last_event(
			Event::KittyTransfer { who: recipient, recipient: account_id, kitty_id }.into(),
		);
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
	})
}
