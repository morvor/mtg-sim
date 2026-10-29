//! Rulings batch P052 — Bard, King of Dale's token doubling ("If one or more tokens would
//! be created under your control, twice that many of those tokens are created instead"):
//! several kinds of tokens, later instructions, "as enters" abilities applied to each
//! token, amass, and several such effects (CR 111.1, 614.1a, 616.1, 701.47a).

use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn bard(t: &mut TestGame) -> ObjectId {
    supported("Bard, King of Dale");
    t.battlefield(P0, "Bard, King of Dale")
}

#[test]
fn every_kind_of_token_an_effect_creates_is_doubled() {
    cr!("614.1a", "616.1", "616.1e", "111.1");
    ruling!(
        "Bard, King of Dale",
        "If an effect creates more than one kind of token, it'll create twice as many of each kind. For example, if you control Bilbo, Fellow Conspirator and Bard, King of Dale and you create a Food token, you'll create two Food tokens and two Treasure tokens (regardless of the order you chose to apply the replacement effects)."
    );
    supported("Bilbo, Fellow Conspirator");
    supported("Bake into a Pie");
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        bard(&mut t);
        t.battlefield(P0, "Bilbo, Fellow Conspirator");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let pie = t.hand(P0, "Bake into a Pie");
        fill_pool(&mut t, P0, 4);
        t.answer(
            P0,
            DecisionKind::Replacement,
            mtg_engine::decision::Answer::Index(pick),
        );
        t.cast(P0, pie).target(obj(bears)).go();
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Food").len(), 2, "pick {pick}");
        assert_eq!(with_subtype(&t, P0, "Treasure").len(), 2, "pick {pick}");
    }
}

#[test]
fn a_later_instruction_applies_to_every_doubled_token() {
    cr!("614.1a", "111.1", "603.7c");
    ruling!(
        "Bard, King of Dale",
        "If the effect creating the tokens instructs you to do something with those tokens at a later time, like exiling them at the end of combat, you'll do that for all the tokens."
    );
    supported("Heat Shimmer");
    let mut t = TestGame::new(2);
    bard(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    let shimmer = t.hand(P0, "Heat Shimmer");
    fill_pool(&mut t, P0, 3);
    t.cast(P0, shimmer).target(obj(giant)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 3);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // Both tokens were exiled; the card remains.
    assert_eq!(t.named_on_battlefield("Hill Giant"), vec![giant]);
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn each_doubled_token_applies_its_as_enters_abilities_separately() {
    cr!("614.1a", "614.1c", "702.128a");
    ruling!(
        "Bard, King of Dale",
        "If the token you create has any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities, first determine how many tokens are being created, then apply those abilities individually for each one. For example, if a token with \"You may have [this permanent] enter as a copy of any creature on the battlefield\" would be created (such as an embalmed Vizier of Many Faces), the resulting two tokens can each copy a different creature."
    );
    let mut t = TestGame::new(2);
    bard(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let vizier = t.graveyard(P0, "Vizier of Many Faces");
    fill_pool(&mut t, P0, 5);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, vizier, 0, &[]).unwrap();
    t.resolve_all();
    let mut names: Vec<String> = tokens(&t, P0)
        .iter()
        .map(|id| t.obj_now(*id).chars.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Hill Giant"]);
    for id in tokens(&t, P0) {
        assert!(t.obj_now(id).chars.has_subtype("Zombie"));
    }
}

#[test]
fn amassing_with_bard_creates_two_armies_and_one_dies() {
    cr!("701.47a", "614.1a", "704.5f");
    ruling!(
        "Bard, King of Dale",
        "If you control Bard, King of Dale and create an Army token because you amass and didn't already control an Army, you will create two Army tokens. However, since you can only put the counters on one of the tokens, the other remains a 0/0 creature and will be put into its owner's graveyard the next time state-based actions are checked."
    );
    supported("Relentless Advance");
    let mut t = TestGame::new(2);
    bard(&mut t);
    let advance = t.hand(P0, "Relentless Advance");
    fill_pool(&mut t, P0, 4);
    t.cast(P0, advance).go();
    t.g.settle();
    t.g.resolve_top();
    // Before state-based actions: two Armies, one with the three counters.
    let armies = with_subtype(&t, P0, "Army");
    assert_eq!(armies.len(), 2);
    let mut counters: Vec<u32> = armies.iter().map(|a| t.counters(*a, "+1/+1")).collect();
    counters.sort();
    assert_eq!(counters, vec![0, 3]);
    t.settle();
    let armies = with_subtype(&t, P0, "Army");
    assert_eq!(armies.len(), 1);
    assert_eq!(t.pt(armies[0]), (3, 3));
}

#[test]
fn two_bards_create_four_times_as_many_tokens() {
    cr!("616.1f", "614.1a", "707.9b");
    ruling!(
        "Bard, King of Dale",
        "Similarly, the effect of the last ability on multiple Bard, King of Dale is cumulative. If you control two, you'll create four times the number of tokens."
    );
    supported("Spark Double");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    let b = bard(&mut t);
    // Spark Double: a nonlegendary copy of Bard.
    t.answer_choose(P0, &[obj(b)]);
    t.enter(P0, "Spark Double");
    t.settle();
    assert_eq!(t.named_on_battlefield("Bard, King of Dale").len(), 2);
    let alarm = t.hand(P0, "Raise the Alarm");
    fill_pool(&mut t, P0, 2);
    t.cast(P0, alarm).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 8);
}
