/** use frame_support::{assert_ok, assert_noop};
use primitives::{H256, Blake2Hasher};

impl_outer_origin! {
	pub enum Origin for SocialBanTest {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct SocialBanTest;

impl system::Trait for SocialBanTest {
	type Origin = Origin;
	type ScopeId = u64;
	type AccountId = u64;
	type Event = ();
}

impl super::Trait for SocialBanTest {
	type Event = ();
}

type SocialBan = super::Module<SocialBanTest>;

fn build_ext() -> TestExternalities<Blake2Hasher> {
	let mut t = system::GenesisConfig::<SocialBanTest>::default().build_storage().unwrap().0;
	t.into()
}

#[test]
fn account_ban_should_work() {
	with_externalities(
		&mut build_ext(), ||
		{ assert!(true) }
    )
}

#[test]
fn account_unblock_should_work() {
	with_externalities(
		&mut build_ext(), ||
		{ assert!(true) }
    )
} **/
