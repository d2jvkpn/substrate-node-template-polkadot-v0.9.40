use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn create_claim_works() {
  new_test_ext().execute_with(|| {
  	let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

  	assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
  });
}
