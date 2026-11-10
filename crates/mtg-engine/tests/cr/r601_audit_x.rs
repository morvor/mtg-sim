//! CR 107.3 / 601.2b audit: X is announced as the spell is proposed, before its targets;
//! it is the announced value only while the spell is on the stack (CR 107.3a, 202.3e) and
//! 0 everywhere else (CR 107.3g), with the exception for the permanent's enters abilities
//! (CR 107.3m); X in a mana cost must be 0 when that cost isn't paid (CR 107.3b), but an
//! X in an additional cost is still chosen; and the values of X a player may announce are
//! bounded by what they could pay for it (mana, life, cards), not by the mana alone.

use crate::r601_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

#[test]
fn x_is_announced_before_targets_and_is_the_spells_x_while_on_the_stack() {
    cr!("601.2b", "601.2c", "107.3a", "202.3e");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let blaze = t.hand(P0, "Blaze");
    let spell = t.cast(P0, blaze).x(3).target(P1).go();
    let labels = decision_labels(&t, P0);
    let x = labels.iter().position(|l| *l == "x").expect("X was asked");
    let targets = labels
        .iter()
        .position(|l| *l == "targets")
        .expect("targets were asked");
    assert!(x < targets, "X is announced before the targets: {labels:?}");
    assert_eq!(x_of(&t, spell), Some(3));
    assert_eq!(t.g.mana_value_of(spell), 4);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn x_is_zero_in_every_zone_but_the_stack_and_the_permanent_enters_with_the_announced_x() {
    cr!("107.3g", "202.3e", "107.3m", "107.3a");
    ruling!(
        "Walking Ballista",
        "A casting cost of {X}{X} means that you pay twice X. If you want X to be 3, you pay {6} to cast Walking Ballista."
    );
    supported("Endless One");
    supported("Walking Ballista");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let one = t.hand(P0, "Endless One");
    assert_eq!(t.g.mana_value_of(one), 0, "X is 0 in the hand");
    let spell = t.cast(P0, one).x(3).go();
    assert_eq!(t.g.mana_value_of(spell), 3, "X is 3 on the stack");
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve();
    let perm = t.g.current(one);
    assert_eq!(t.counters(perm, counters::PLUS1), 3);
    assert_eq!(t.g.mana_value_of(perm), 0, "X is 0 on the battlefield");
    assert_eq!(t.pt(perm), (3, 3));
    // {X}{X}: twice X is paid; the permanent enters with X counters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 5);
    let ballista = t.hand(P0, "Walking Ballista");
    let spell = t.cast(P0, ballista).x(2).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve();
    let perm = t.g.current(ballista);
    assert_eq!(t.counters(perm, counters::PLUS1), 2);
    assert_eq!(t.g.mana_value_of(perm), 0);
}

#[test]
fn x_must_be_zero_for_a_spell_cast_without_paying_its_mana_cost() {
    cr!("107.3b", "601.2b");
    ruling!(
        "Omniscience",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.lands(P0, "Wastes", 4);
    let one = t.hand(P0, "Endless One");
    // The player would like X = 3; the only legal value is 0.
    let spell = t.cast(P0, one).method(CastMethod::Free).x(3).go();
    assert_eq!(x_of(&t, spell), Some(0));
    assert_eq!(t.g.mana_value_of(spell), 0);
    assert_eq!(tapped_lands(&t, P0), 0);
    t.resolve();
    // It entered with no counters, a 0/0, and died.
    assert!(t.in_graveyard(P0, "Endless One"));
}

#[test]
fn x_in_an_additional_cost_is_still_chosen_when_the_mana_cost_isnt_paid() {
    cr!("107.3b", "118.8", "601.2b");
    ruling!(
        "Toxic Deluge",
        "If you cast Toxic Deluge without paying its mana cost, you'll still choose a value for X and pay X life. This is because it doesn't have {X} in its mana cost."
    );
    supported("Toxic Deluge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let deluge = t.hand(P0, "Toxic Deluge");
    let spell = t.cast(P0, deluge).method(CastMethod::Free).x(3).go();
    assert_eq!(x_of(&t, spell), Some(3));
    assert_eq!(t.life(P0), 17, "X life is paid");
    t.resolve();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn an_x_paid_as_life_is_bounded_by_the_life_total_not_by_the_mana_available() {
    cr!("107.3a", "601.2b", "119.4", "118.3");
    // Toxic Deluge {2}{B}, "As an additional cost to cast this spell, pay X life." With
    // three Swamps and 20 life, X may be anything up to 20.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let deluge = t.hand(P0, "Toxic Deluge");
    t.cast(P0, deluge).x(5).go();
    assert_eq!(offered_x_max(&t, P0), Some(20));
    assert_eq!(t.life(P0), 15);
    assert_eq!(untapped_lands(&t, P0), 0);
    // At 30 life, up to 30 (the bound is the life total, not a constant).
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 30;
    t.lands(P0, "Swamp", 3);
    let deluge = t.hand(P0, "Toxic Deluge");
    t.cast(P0, deluge).x(25).go();
    assert_eq!(offered_x_max(&t, P0), Some(30));
    assert_eq!(t.life(P0), 5);
    // At 4 life, X can't be 5: that casting is illegal and nothing is paid.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 4;
    t.lands(P0, "Swamp", 3);
    let deluge = t.hand(P0, "Toxic Deluge");
    assert!(t.cast(P0, deluge).x(5).try_go().is_err());
    assert_eq!(offered_x_max(&t, P0), Some(4));
    assert_eq!(t.life(P0), 4);
    assert_eq!(untapped_lands(&t, P0), 3);
    assert!(t.in_hand(P0, "Toxic Deluge"));
}

#[test]
fn an_x_paid_with_cards_isnt_bounded_by_the_mana_available() {
    cr!("107.3a", "601.2b", "601.2h");
    // Conflagrate's flashback cost: {R}{R}, Discard X cards. With two Mountains and four
    // cards in hand, X may be 4: the mana pays only {R}{R}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    for _ in 0..4 {
        t.hand(P0, "Grizzly Bears");
    }
    let card = t.graveyard(P0, "Conflagrate");
    let spell = t.cast(P0, card).method(FLASHBACK).x(4).go();
    assert_eq!(offered_x_max(&t, P0), Some(4));
    assert_eq!(x_of(&t, spell), Some(4));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn the_greatest_x_offered_accounts_for_cost_increases() {
    cr!("601.2b", "601.2f", "107.3a");
    // Blaze {X}{R} under Thalia with four Mountains: {X}{R} + {1} leaves X at most 2.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 4);
    let blaze = t.hand(P0, "Blaze");
    // X = 3 can't be paid: the casting is illegal and reversed.
    assert!(t.cast(P0, blaze).x(3).target(P1).try_go().is_err());
    assert_eq!(offered_x_max(&t, P0), Some(2));
    assert_eq!(untapped_lands(&t, P0), 4);
    // The default is the greatest value: 2.
    let blaze = t.g.current(blaze);
    t.answer(P0, DecisionKind::X, Answer::Default);
    let spell = t.cast(P0, blaze).target(P1).go();
    assert_eq!(x_of(&t, spell), Some(2));
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn the_x_of_a_kicker_cost_is_announced_as_the_spell_is_cast() {
    cr!("601.2b", "702.33a", "107.3a");
    supported("Verdeloth the Ancient");
    // Verdeloth the Ancient {4}{G}{G}, kicker {X}: kicked with X = 2 it costs {6}{G}{G} and
    // makes two Saprolings.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 8);
    let verdeloth = t.hand(P0, "Verdeloth the Ancient");
    let spell = t.cast(P0, verdeloth).kicked(true).x(2).go();
    assert_eq!(x_of(&t, spell), Some(2));
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(
        t.g.mana_value_of(spell),
        6,
        "kicker doesn't change the mana value"
    );
    t.resolve_all();
    assert_eq!(permanents_with_subtype(&t, "Saproling"), 2);
    // Not kicked: X isn't asked and the Treefolk costs {4}{G}{G}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let verdeloth = t.hand(P0, "Verdeloth the Ancient");
    t.cast(P0, verdeloth).kicked(false).go();
    assert_eq!(x_asked(&t, P0), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(permanents_with_subtype(&t, "Saproling"), 0);
}

#[test]
fn a_copy_of_an_x_spell_cast_without_paying_its_mana_cost_has_x_zero() {
    cr!("707.12", "107.3b", "601.2b");
    ruling!(
        "Isochron Scepter",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Isochron Scepter");
    supported("Volcanic Geyser");
    // Volcanic Geyser ({X}{R}{R}) imprinted on Isochron Scepter: the copy is cast without
    // paying its mana cost, so X is 0 and it deals no damage.
    let mut t = TestGame::new(2);
    let geyser = t.hand(P0, "Volcanic Geyser");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(geyser)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(geyser), Zone::Exile);
    t.lands(P0, "Mountain", 4);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    // The player would like X = 2; it must be 0.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(t.activate(P0, scepter, 0, &[]).is_ok());
    t.resolve_all();
    let cast_copies =
        t.g.history
            .spells_cast
            .iter()
            .filter(|(p, s)| *p == P0 && t.g.obj(*s).chars.name == "Volcanic Geyser")
            .count();
    assert_eq!(cast_copies, 1, "the copy was cast");
    assert_eq!(t.life(P1), 20, "X is 0");
    // Only the Scepter's {2} was paid.
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn x_may_be_announced_higher_than_the_mana_available_when_convoke_pays_for_it() {
    cr!("601.2b", "702.51b", "107.3a");
    ruling!(
        "Chord of Calling",
        "When using convoke to cast a spell with {X} in its mana cost, first choose the value for X."
    );
    // Chord of Calling {X}{G}{G}{G} with four green creatures and no lands: X = 1 makes the
    // total {1}{G}{G}{G}, which the four creatures convoke.
    let mut t = TestGame::new(2);
    let bears: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let chord = t.hand(P0, "Chord of Calling");
    choose_objects(&mut t, P0, &bears);
    let spell = t.cast(P0, chord).x(1).go();
    assert_eq!(x_of(&t, spell), Some(1));
    assert!(bears.iter().all(|b| is_tapped(&t, *b)));
    assert_eq!(t.g.mana_value_of(spell), 4);
    // With three creatures, X = 1 can't be paid: the casting is reversed.
    let mut t = TestGame::new(2);
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let chord = t.hand(P0, "Chord of Calling");
    choose_objects(&mut t, P0, &bears);
    assert!(t.cast(P0, chord).x(1).try_go().is_err());
    assert!(bears.iter().all(|b| !is_tapped(&t, *b)));
    assert!(t.in_hand(P0, "Chord of Calling"));
}

#[test]
fn an_unannounced_x_of_an_activation_cost_paid_with_counters_is_the_least_payable_value() {
    cr!("602.2b", "107.3a", "118.3");
    interp!("default-numbers");
    supported("Moxite Refinery");
    // Moxite Refinery: "{2}, {T}, Remove X counters from an artifact or creature you
    // control: Choose one — Put X charge counters on target artifact; or put X +1/+1
    // counters on target creature." With no value announced, X is the least value the
    // cost can be paid at: 0 (removing no counters is payable whatever the permanents
    // carry), so the Bears keep their two counters and the Bonesplitter gets none. (A
    // payability check that found X = 0 unpayable would announce 1 instead.)
    let mut t = TestGame::new(2);
    let refinery = t.battlefield(P0, "Moxite Refinery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rock = t.battlefield(P0, "Bonesplitter");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    t.lands(P0, "Plains", 2);
    t.answer(P0, DecisionKind::X, Answer::Default);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, refinery, 0, &[Entity::Object(rock)])
        .unwrap();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.counters(rock, "charge"), 0);
    // Announced, X = 2 removes the Bears' two counters and puts two charge counters.
    let mut t = TestGame::new(2);
    let refinery = t.battlefield(P0, "Moxite Refinery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rock = t.battlefield(P0, "Bonesplitter");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    t.lands(P0, "Plains", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, refinery, 0, &[Entity::Object(rock)])
        .unwrap();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    t.resolve();
    assert_eq!(t.counters(rock, "charge"), 2);
}
