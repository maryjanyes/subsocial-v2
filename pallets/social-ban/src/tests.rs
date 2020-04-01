#![cfg(test)]

pub use super::*;

use sp_core::H256;
use frame_support::{impl_outer_origin, assert_ok, assert_noop, parameter_types, weights::Weight, dispatch::DispatchResult};
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill};
use pallet_space_owners as SpaceOwners;

impl_outer_origin! {
  pub enum Origin for SocialBanTest {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct SocialBanTest;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for SocialBanTest {
  type Origin = Origin;
  type Call = ();
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = u64;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type AccountData = u64;
  type Event = ();
  type Version = ();
  type ModuleToIndex = ();
  type OnKilledAccount = ();
  type OnNewAccount = ();
  type BlockHashCount = BlockHashCount;
  type MaximumBlockWeight = MaximumBlockWeight;
  type MaximumBlockLength = MaximumBlockLength;
  type AvailableBlockRatio = AvailableBlockRatio;
}

impl pallet_space_owners::Trait for SocialBanTest { }

parameter_types! {
  pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Trait for SocialBanTest {
  type Moment = u64;
  type OnTimestampSet = ();
  type MinimumPeriod = MinimumPeriod;
}

impl Trait for SocialBanTest {
	type Event = ();
	type SpaceOwnersSharedModule = SpaceOwnersModule;
}

type SocialBanTestModule = Module<SocialBanTest>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
fn new_test_ext() -> sp_io::TestExternalities {
  system::GenesisConfig::default().build_storage::<SocialBanTest>().unwrap().into()
}

#[test]
fn test1() {
	new_test_ext().execute_with(|| {
		assert_ok!(SocialBanTestModule::block_account(
			Origin::signed(1),
			10,
			Origin::signed(2),
		));
	});
}

pub type SpaceOwnersModule = pallet_space_owners::Module<SocialBanTest>;
