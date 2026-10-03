//! Rulings batch P225 — abilities of a permanent that transform it do so only if it hasn't
//! transformed since the ability was put on the stack (or a delayed ability was created)
//! (CR 701.27f); another permanent's ability may transform it any number of times; and
//! "activate only once each turn" follows the object across transformations (CR 602.5b,
//! 712.18).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, create_token};
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::*;
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const AANG: &str = "Aang, at the Crossroads // Aang, Destined Savior";
const STRANGER: &str = "Kindly Stranger // Demon-Possessed Witch";
const SLEUTH: &str = "Daring Sleuth // Bearer of Overwhelming Truths";
const SENTRY: &str = "Thraben Sentry // Thraben Militia";
const SURVIVOR: &str = "Bereaved Survivor // Dauntless Avenger";
const BANDIT: &str = "Geier Reach Bandit // Vildin-Pack Alpha";
const CAPTIVE: &str = "Wolfbitten Captive // Krallenhorde Killer";

/// Two Grizzly Bears `p` controls die at the same time; then triggers are put on the stack.
fn two_bears_die(t: &mut TestGame, p: PlayerId) {
    let a = t.battlefield(p, "Grizzly Bears");
    let b = t.battlefield(p, "Grizzly Bears");
    t.g.destroy_all(vec![a, b], None, false);
    t.settle();
}

#[test]
fn aangs_second_delayed_trigger_doesnt_transform_him_back() {
    cr!("701.27f", "603.7");
    ruling!(
        "Aang, at the Crossroads // Aang, Destined Savior",
        "Aang, at the Crossroads's delayed triggered ability won't cause it to transform back"
    );
    supported(AANG);
    let mut t = TestGame::new(2);
    let aang = t.battlefield(P0, AANG);
    two_bears_die(&mut t, P0);
    t.resolve_all();
    assert_eq!(face(&t, aang), FaceState::Front);
    // The next upkeep: two delayed triggers, one transformation.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(crate::r_s04_common::on_stack(&t, "delayed trigger"), 2);
    t.resolve_all();
    assert_eq!(name_of(&t, aang), "Aang, Destined Savior");
}

#[test]
fn self_transforming_abilities_only_transform_once() {
    cr!("701.27f");
    ruling!(
        "Kindly Stranger // Demon-Possessed Witch",
        "Activating Kindly Stranger's ability twice won't cause it to transform back"
    );
    ruling!(
        "Daring Sleuth // Bearer of Overwhelming Truths",
        "only the first one to resolve will cause it to transform"
    );
    ruling!(
        "Thraben Sentry // Thraben Militia",
        "Only the first one to resolve will cause it to transform."
    );
    ruling!(
        "Bereaved Survivor // Dauntless Avenger",
        "Multiple creatures dying will cause Bereaved Survivor to transform only once."
    );
    // Kindly Stranger: two activations ({2}{B} each, delirium from four card types).
    supported(STRANGER);
    let mut t = TestGame::new(2);
    let stranger = t.battlefield(P0, STRANGER);
    for c in ["Forest", "Grizzly Bears", "Lightning Bolt", "Divination"] {
        t.graveyard(P0, c);
    }
    t.lands(P0, "Swamp", 6);
    activate_containing(&mut t, P0, stranger, "Transform").unwrap();
    activate_containing(&mut t, P0, stranger, "Transform").unwrap();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(name_of(&t, stranger), "Demon-Possessed Witch");
    // Daring Sleuth: two Clues sacrificed, two triggers.
    supported(SLEUTH);
    let mut t = TestGame::new(2);
    let sleuth = t.battlefield(P0, SLEUTH);
    let c1 = create_token(&mut t, P0, "Clue");
    let c2 = create_token(&mut t, P0, "Clue");
    t.lands(P0, "Wastes", 4);
    activate_containing(&mut t, P0, c1, "Sacrifice").unwrap();
    t.settle();
    activate_containing(&mut t, P0, c2, "Sacrifice").unwrap();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "transform"), 2);
    t.resolve_all();
    assert_eq!(name_of(&t, sleuth), "Bearer of Overwhelming Truths");
    // Thraben Sentry ("you may transform") and Bereaved Survivor: two creatures die.
    for (name, back) in [(SENTRY, "Thraben Militia"), (SURVIVOR, "Dauntless Avenger")] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        two_bears_die(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, "transform"), 2, "{name}");
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(name_of(&t, c), back, "{name}");
    }
}

#[test]
fn three_vildin_pack_alphas_can_transform_a_werewolf_three_times() {
    cr!("701.27f", "701.27a");
    ruling!(
        "Geier Reach Bandit // Vildin-Pack Alpha",
        "you may transform it, transform it again, and transform it a third time"
    );
    supported(BANDIT);
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        let a = t.battlefield(P0, BANDIT);
        transform(&mut t, a);
        assert_eq!(name_of(&t, a), "Vildin-Pack Alpha");
    }
    let bandit = t.enter(P0, BANDIT);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "transform it"), 3);
    // After each resolution, the face changes.
    for back in [true, false, true] {
        t.answer_yes(P0, true);
        t.resolve();
        assert_eq!(face(&t, bandit) == FaceState::Back, back);
    }
    assert_eq!(name_of(&t, bandit), "Vildin-Pack Alpha");
}

#[test]
fn wolfbitten_captives_once_each_turn_ability_survives_two_transformations() {
    cr!("602.5b", "712.18");
    ruling!(
        "Wolfbitten Captive // Krallenhorde Killer",
        "you won't be able to activate that face's ability again that turn"
    );
    supported(CAPTIVE);
    let mut t = TestGame::new(2);
    let captive = t.battlefield(P0, CAPTIVE);
    t.lands(P0, "Forest", 4);
    activate_containing(&mut t, P0, captive, "+2/+2").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(captive), (3, 3));
    assert!(!can_activate(&mut t, P0, captive));
    transform(&mut t, captive);
    transform(&mut t, captive);
    assert_eq!(name_of(&t, captive), "Wolfbitten Captive");
    assert!(!can_activate(&mut t, P0, captive));
}
