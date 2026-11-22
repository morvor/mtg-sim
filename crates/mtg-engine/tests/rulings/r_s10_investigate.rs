//! Rulings batch S10 — investigate (CR 701.16): "Create a Clue token. (It's an artifact
//! with '{2}, Sacrifice this token: Draw a card.')"

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s02_common::{can_activate, create_token, destroy};
use crate::r_s04_common::add_mana;
use crate::r_s05_common::{enter, tokens_with_subtype};
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::resolved;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn clues(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Clue"))
        .map(|o| o.id)
        .collect()
}

/// Activates a Clue's "{2}, Sacrifice this token: Draw a card." with mana from the pool.
fn crack(t: &mut TestGame, p: PlayerId, clue: ObjectId) {
    add_mana(t, p, ManaType::C, 2);
    activate_containing(t, p, clue, "Draw a card").expect("activate the Clue");
    t.resolve();
}

#[test]
fn clue_is_only_an_artifact_type() {
    cr!("205.3g", "205.3m", "702.73a");
    ruling!(
        "Alquist Proft, Master Sleuth",
        "Clue is an artifact type. Even though it appears on some cards with other permanent types, it's never a creature type, a land type, or anything but an artifact type."
    );
    supported("Alquist Proft, Master Sleuth");
    supported("Tireless Tracker");
    supported("Changeling Outcast");
    assert_eq!(subtype_kinds("Clue"), vec![SubtypeKind::Artifact]);
    assert!(!is_creature_type("Clue"));
    // A changeling is every creature type, but not a Clue: Tireless Tracker ("Whenever
    // you sacrifice a Clue, put a +1/+1 counter on this creature") doesn't trigger when
    // it's sacrificed, and it can't pay Alquist Proft's "Sacrifice a Clue".
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Tireless Tracker");
    let alquist = t.battlefield(P0, "Alquist Proft, Master Sleuth");
    let outcast = t.battlefield(P0, "Changeling Outcast");
    assert!(t.obj_now(outcast).chars.has_subtype("Elf"));
    assert!(!t.obj_now(outcast).chars.has_subtype("Clue"));
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    assert!(!can_activate(&mut t, P0, alquist));
    t.g.sacrifice(outcast, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(tracker, counters::PLUS1), 0);
}

#[test]
fn a_clue_that_becomes_a_creature_is_still_only_an_artifact_clue() {
    cr!("205.3g", "613.1d");
    ruling!(
        "Tangletrove Kelp",
        "Clue is an artifact type. Even though it appears on some cards with other permanent types, it’s never a creature type, a land type, or anything but an artifact type."
    );
    supported("Tangletrove Kelp");
    // Tangletrove Kelp: "At the beginning of each combat, other Clues you control become
    // 6/6 Plant creatures in addition to their other types until end of turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tangletrove Kelp");
    let clue = create_token(&mut t, P0, "Clue");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(clue);
    assert!(o.is(CardType::Creature) && o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Plant"));
    assert!(o.chars.has_subtype("Clue"));
    assert_eq!(t.pt(clue), (6, 6));
    // It's still a Clue once the effect ends and it's no longer a creature.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(clue).is(CardType::Creature));
    assert!(t.obj_now(clue).chars.has_subtype("Clue"));
    assert!(!t.obj_now(clue).chars.has_subtype("Plant"));
}

/// With one Clue, `p` can activate Alquist Proft's "{X}{W}{U}{U}, {T}, Sacrifice a Clue"
/// until that Clue is sacrificed for its own ability.
fn one_clue_one_cost() {
    supported("Alquist Proft, Master Sleuth");
    let mut t = TestGame::new(2);
    let alquist = t.battlefield(P0, "Alquist Proft, Master Sleuth");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    let clue = create_token(&mut t, P0, "Clue");
    assert!(can_activate(&mut t, P0, alquist));
    crack(&mut t, P0, clue);
    assert!(!t.on_battlefield(clue));
    assert!(!can_activate(&mut t, P0, alquist));
}

#[test]
fn a_clue_cant_be_sacrificed_to_pay_two_costs() {
    cr!("118.3", "701.16a");
    ruling!(
        "Alquist Proft, Master Sleuth",
        "You can't sacrifice a Clue to pay multiple costs. For example, you can't sacrifice a Clue token to activate its own ability and also to activate Alquist Proft, Master Sleuth's ability."
    );
    one_clue_one_cost();
}

#[test]
fn a_clue_cant_be_sacrificed_to_pay_two_costs_curly() {
    cr!("118.3", "701.16a");
    ruling!(
        "Merchant of Truth",
        "You can’t sacrifice a Clue to pay multiple costs. For example, you can’t sacrifice a Clue token to activate its own ability and also to activate Alquist Proft, Master Sleuth’s ability."
    );
    one_clue_one_cost();
}

#[test]
fn a_nontoken_clue_artifact_is_a_clue() {
    cr!("205.3g", "701.16a");
    ruling!(
        "Alquist Proft, Master Sleuth",
        "If an effect refers to a Clue, it means any Clue artifact, not just a Clue artifact token. For example, you can sacrifice Wrench to pay for Alquist Proft, Master Sleuth's activated ability."
    );
    supported("Wrench");
    // Wrench is an "Artifact — Clue Equipment" card. Alquist Proft: "{X}{W}{U}{U}, {T},
    // Sacrifice a Clue: You draw X cards and gain X life."
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Tireless Tracker");
    let alquist = t.battlefield(P0, "Alquist Proft, Master Sleuth");
    let wrench = t.battlefield(P0, "Wrench");
    assert!(!t.obj_now(wrench).is_token());
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 1);
    assert!(can_activate(&mut t, P0, alquist));
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    activate_containing(&mut t, P0, alquist, "Sacrifice a Clue").expect("activate Alquist");
    assert!(t.in_graveyard(P0, "Wrench"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 21);
    // Tireless Tracker's "Whenever you sacrifice a Clue" triggered too.
    assert_eq!(t.counters(tracker, counters::PLUS1), 1);
}

#[test]
fn a_nontoken_clue_artifact_is_a_clue_curly() {
    cr!("205.3g", "613.1f");
    ruling!(
        "Merchant of Truth",
        "If an effect refers to a Clue, it means any Clue artifact, not just a Clue artifact token."
    );
    supported("Merchant of Truth");
    // Merchant of Truth: "Clues you control have exalted." Wrench is a Clue.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Merchant of Truth");
    let wrench = t.battlefield(P0, "Wrench");
    let opponents = t.battlefield(P1, "Wrench");
    assert!(t.obj_now(wrench).has_keyword(KeywordKind::Exalted));
    assert!(!t.obj_now(opponents).has_keyword(KeywordKind::Exalted));
}

/// P0 casts the spell `name` targeting P1's Grizzly Bears; the Bears are destroyed in
/// response, so it doesn't resolve and P0 doesn't investigate.
fn fizzle_no_clue(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, name);
    let card = t.hand(P0, name);
    let spell = t.cast(P0, card).target(bears).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(t.in_graveyard(P0, name));
    assert!(clues(&t, P0).is_empty());
    // With a legal target it investigates.
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, name);
    let card = t.hand(P0, name);
    t.cast(P0, card).target(bears).go();
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Clue").len(), 1);
}

#[test]
fn an_investigate_spell_with_illegal_targets_creates_no_clue() {
    cr!("608.2b", "701.16a");
    ruling!(
        "Toxin Analysis",
        "Some spells and abilities that investigate may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. You won't create any Clue tokens."
    );
    // Toxin Analysis: "Target creature gains deathtouch and lifelink until end of turn.
    // Investigate."
    fizzle_no_clue("Toxin Analysis");
}

#[test]
fn an_investigate_spell_with_illegal_targets_creates_no_clue_curly() {
    cr!("608.2b", "701.16a");
    ruling!(
        "Merchant of Truth",
        "Some spells and abilities that investigate may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won’t resolve. You won’t create any Clue tokens."
    );
    // Press for Answers: "Tap target creature. ... Investigate."
    fizzle_no_clue("Press for Answers");
}

#[test]
fn an_investigate_spell_needs_legal_targets() {
    cr!("601.2c", "608.2b");
    ruling!(
        "Jace's Scrutiny",
        "You can't cast a spell without choosing legal targets. If all of those targets become illegal, the spell doesn't resolve and you won't investigate."
    );
    // Jace's Scrutiny: "Target creature gets -4/-0 until end of turn. Investigate."
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Jace's Scrutiny");
    let card = t.hand(P0, "Jace's Scrutiny");
    assert!(t.cast(P0, card).try_go().is_err());
    assert!(t.in_hand(P0, "Jace's Scrutiny"));
    fizzle_no_clue("Jace's Scrutiny");
}

/// Tireless Tracker's "Whenever you sacrifice a Clue" triggers when the Clue is
/// sacrificed to pay for Breya's Apprentice ("{T}, Sacrifice an artifact: ...").
fn sacrificed_for_another_cost(clue: impl FnOnce(&mut TestGame) -> ObjectId) {
    supported("Breya's Apprentice");
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Tireless Tracker");
    let apprentice = t.battlefield(P0, "Breya's Apprentice");
    let clue = clue(&mut t);
    // The mode "Target creature gets +2/+0 until end of turn."
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_choose(P0, &[Entity::Object(clue)]);
    t.answer_targets(P0, &[Entity::Object(tracker)]);
    activate_containing(&mut t, P0, apprentice, "Sacrifice an artifact").expect("activate");
    assert!(!t.on_battlefield(clue));
    t.resolve_all();
    // Tireless Tracker (3/2): a +1/+1 counter, and +2/+0 from the Apprentice's ability.
    assert_eq!(t.counters(tracker, counters::PLUS1), 1);
    assert_eq!(t.pt(tracker), (6, 3));
}

#[test]
fn whenever_you_sacrifice_a_clue_triggers_for_any_sacrifice() {
    cr!("701.16a", "603.2", "701.21a");
    ruling!(
        "Tireless Tracker",
        "Some abilities trigger \"whenever you sacrifice a Clue\". Those abilities trigger whenever you sacrifice a Clue for any reason, not just to activate a Clue's activated ability."
    );
    sacrificed_for_another_cost(|t| create_token(t, P0, "Clue"));
}

#[test]
fn whenever_you_sacrifice_a_clue_triggers_for_any_sacrifice_curly() {
    cr!("701.16a", "603.2", "701.21a");
    ruling!(
        "Merchant of Truth",
        "Some abilities trigger “whenever you sacrifice a Clue”. Those abilities trigger whenever you sacrifice a Clue for any reason, not just to activate a Clue’s activated ability."
    );
    // The Clue is one Merchant of Truth made: "Whenever a nontoken creature you control
    // dies, investigate."
    sacrificed_for_another_cost(|t| {
        t.battlefield(P0, "Merchant of Truth");
        let bears = t.battlefield(P0, "Grizzly Bears");
        destroy(t, bears);
        t.resolve_all();
        let clues = tokens_with_subtype(t, P0, "Clue");
        assert_eq!(clues.len(), 1);
        clues[0]
    });
}

#[test]
fn the_clue_token_is_named_clue_token() {
    cr!("111.4", "111.10f", "701.16a");
    ruling!(
        "Floodhound",
        "The token is named Clue Token and has the artifact subtype Clue. Clue isn't a creature type."
    );
    supported("Floodhound");
    // Floodhound: "{3}, {T}: Investigate."
    let mut t = TestGame::new(2);
    let hound = t.battlefield(P0, "Floodhound");
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, hound, "Investigate").expect("activate Floodhound");
    t.resolve();
    let clues = tokens_with_subtype(&t, P0, "Clue");
    assert_eq!(clues.len(), 1);
    let o = t.obj_now(clues[0]);
    assert_eq!(o.chars.name, "Clue Token");
    assert!(o.is(CardType::Artifact));
    assert!(!o.is(CardType::Creature));
    assert!(!is_creature_type("Clue"));
    // Xenograft ("As this enchantment enters, choose a creature type.") doesn't offer it.
    let from = t.asked().len();
    enter(&mut t, P0, "Xenograft");
    let options: Vec<String> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .expect("a creature type was chosen");
    assert!(options.iter().any(|o| o == "Crab"));
    assert!(!options.iter().any(|o| o == "Clue"));
}

#[test]
fn clue_tokens_are_normal_artifacts() {
    cr!("111.10f", "115.1");
    ruling!(
        "Funnel-Web Recluse",
        "The tokens are normal artifacts. For example, one can be sacrificed to activate the ability of Breya's Apprentice and one can be the target of Break Ties."
    );
    supported("Funnel-Web Recluse");
    supported("Break Ties");
    // Funnel-Web Recluse: "Morbid — When this creature enters, if a creature died this
    // turn, investigate."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    enter(&mut t, P0, "Funnel-Web Recluse");
    t.resolve_all();
    enter(&mut t, P0, "Funnel-Web Recluse");
    t.resolve_all();
    let clues = tokens_with_subtype(&t, P0, "Clue");
    assert_eq!(clues.len(), 2);
    // One is sacrificed for Breya's Apprentice.
    let apprentice = t.battlefield(P0, "Breya's Apprentice");
    let recluse = t.named_on_battlefield("Funnel-Web Recluse")[0];
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_choose(P0, &[Entity::Object(clues[0])]);
    t.answer_targets(P0, &[Entity::Object(recluse)]);
    activate_containing(&mut t, P0, apprentice, "Sacrifice an artifact").expect("activate");
    t.resolve_all();
    assert!(!t.on_battlefield(clues[0]));
    // The other is the target of Break Ties's "Destroy target artifact."
    give_mana_for(&mut t, P1, "Break Ties");
    let ties = t.hand(P1, "Break Ties");
    t.cast(P1, ties).modes(&[0]).target(clues[1]).go();
    t.resolve();
    assert!(!t.on_battlefield(clues[1]));
}

#[test]
fn a_clue_sacrificed_for_its_own_ability_cant_pay_another_sacrifice_cost() {
    cr!("118.3", "701.16a");
    ruling!(
        "Floodhound",
        "You can't sacrifice a Clue to activate its own ability and also to activate another ability that requires sacrificing a Clue (or any artifact) as a cost"
    );
    supported("Alquist Proft, Master Sleuth");
    // Floodhound investigates; Alquist Proft ("{X}{W}{U}{U}, {T}, Sacrifice a Clue: ...")
    // could sacrifice that Clue, until it's sacrificed for its own ability.
    let mut t = TestGame::new(2);
    let hound = t.battlefield(P0, "Floodhound");
    let alquist = t.battlefield(P0, "Alquist Proft, Master Sleuth");
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, hound, "Investigate").expect("activate Floodhound");
    t.resolve();
    let clue = tokens_with_subtype(&t, P0, "Clue")[0];
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    assert!(can_activate(&mut t, P0, alquist));
    crack(&mut t, P0, clue);
    assert!(!t.on_battlefield(clue));
    assert!(!can_activate(&mut t, P0, alquist));
}
