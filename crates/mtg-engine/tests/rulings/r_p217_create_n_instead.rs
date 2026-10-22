//! Rulings batch P217 — "Create [tokens]. If [condition], create N of those tokens
//! instead." (CR 608.2c, 614.1a): the cards the "create N of those tokens instead"
//! pattern compiles, one test per wording. The same tokens are created, only more of them,
//! and only when the condition holds as the spell or ability resolves.

use crate::r_s01_common::{give_mana_for, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::tokens_with_subtype;
use crate::r_s06_common::activate_containing;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts `name` from hand (with lands named `land`) (kicked or not) and resolves it; returns P0's tokens of
/// `subtype`.
fn cast_kicked(name: &str, land: &str, kick: bool, subtype: &str) -> Vec<ObjectId> {
    supported(name);
    let mut t = TestGame::new(2);
    t.lands(P0, land, 13);
    let card = t.hand(P0, name);
    t.cast(P0, card).kicked(kick).go();
    t.resolve_all();
    let toks = tokens_with_subtype(&t, P0, subtype);
    for tok in &toks {
        assert_eq!(t.pt(*tok), (1, 1));
    }
    toks
}

#[test]
fn saproling_migration_creates_four_instead_of_two_when_kicked() {
    cr!("608.2c", "614.1a", "702.33d");
    // "Create two 1/1 green Saproling creature tokens. If this spell was kicked, create
    // four of those tokens instead."
    assert_eq!(cast_kicked("Saproling Migration", "Forest", false, "Saproling").len(), 2);
    assert_eq!(cast_kicked("Saproling Migration", "Forest", true, "Saproling").len(), 4);
}

#[test]
fn conquerors_pledge_creates_twelve_instead_of_six_when_kicked() {
    cr!("608.2c", "614.1a", "702.33d");
    assert_eq!(cast_kicked("Conqueror's Pledge", "Plains", false, "Kor").len(), 6);
    let kor = cast_kicked("Conqueror's Pledge", "Plains", true, "Kor");
    assert_eq!(kor.len(), 12);
}

#[test]
fn increasing_devotion_creates_ten_when_cast_from_a_graveyard() {
    cr!("608.2c", "614.1a", "702.34a");
    supported("Increasing Devotion");
    // From hand: five Humans.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Increasing Devotion");
    let card = t.hand(P0, "Increasing Devotion");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Human").len(), 5);
    // With flashback from the graveyard: ten.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 9);
    let card = t.graveyard(P0, "Increasing Devotion");
    t.cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Human").len(), 10);
}

#[test]
fn gather_the_townsfolk_creates_five_with_five_or_less_life() {
    cr!("608.2c", "614.1a");
    supported("Gather the Townsfolk");
    for (life, n) in [(6, 2), (5, 5)] {
        let mut t = TestGame::new(2);
        t.g.player_mut(P0).life = life;
        give_mana_for(&mut t, P0, "Gather the Townsfolk");
        let card = t.hand(P0, "Gather the Townsfolk");
        t.cast(P0, card).go();
        t.resolve_all();
        assert_eq!(tokens_with_subtype(&t, P0, "Human").len(), n);
    }
}

#[test]
fn skeletal_swarming_creates_two_tapped_skeletons_if_a_creature_died() {
    cr!("608.2c", "614.1a");
    supported("Skeletal Swarming");
    for died in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Skeletal Swarming");
        if died {
            let bears = t.battlefield(P1, "Grizzly Bears");
            destroy(&mut t, bears);
        }
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "create a tapped"), 1);
        t.resolve_all();
        let skeletons = tokens_with_subtype(&t, P0, "Skeleton");
        assert_eq!(skeletons.len(), if died { 2 } else { 1 });
        assert!(skeletons.iter().all(|s| t.obj_now(*s).tapped));
    }
}

#[test]
fn safana_creates_three_treasures_if_youve_completed_a_dungeon() {
    cr!("608.2c", "614.1a", "309.7");
    supported("Safana, Calimport Cutthroat");
    for completed in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Safana, Calimport Cutthroat");
        t.g.initiative = Some(P0);
        t.g.player_mut(P0).dungeons_completed = completed;
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let n = tokens_with_subtype(&t, P0, "Treasure").len();
        assert_eq!(n, if completed > 0 { 3 } else { 1 });
    }
}

#[test]
fn throne_of_empires_creates_five_soldiers_with_both_crown_and_scepter() {
    cr!("608.2c", "614.1a");
    supported("Throne of Empires");
    for (crown, scepter, n) in [(false, false, 1), (true, false, 1), (true, true, 5)] {
        let mut t = TestGame::new(2);
        let throne = t.battlefield(P0, "Throne of Empires");
        if crown {
            t.battlefield(P0, "Crown of Empires");
        }
        if scepter {
            t.battlefield(P0, "Scepter of Empires");
        }
        // The opponent's Scepter doesn't count ("if you control").
        if !scepter && crown {
            t.battlefield(P1, "Scepter of Empires");
        }
        add_mana(&mut t, P0, ManaType::C, 1);
        activate_containing(&mut t, P0, throne, "Create").expect("activate");
        t.resolve_all();
        assert_eq!(tokens_with_subtype(&t, P0, "Soldier").len(), n);
    }
}

#[test]
fn starnheim_unleashed_creates_x_angels_when_foretold() {
    cr!("608.2c", "614.1a", "702.143a", "107.3a");
    supported("Starnheim Unleashed");
    // Cast from hand: one 4/4 Angel.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Starnheim Unleashed");
    let card = t.hand(P0, "Starnheim Unleashed");
    t.cast(P0, card).go();
    t.resolve_all();
    let angels = tokens_with_subtype(&t, P0, "Angel");
    assert_eq!(angels.len(), 1);
    assert_eq!(t.pt(angels[0]), (4, 4));
    // Foretold, then cast on a later turn for {X}{X}{W} with X = 3: three Angels.
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Starnheim Unleashed");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.take_action(
        P0,
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::Foretell {
            card,
        }),
    );
    t.g.flush_events();
    let card = t.g.current(card);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 7);
    t.cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Foretell))
        .x(3)
        .go();
    t.resolve_all();
    let angels = tokens_with_subtype(&t, P0, "Angel");
    assert_eq!(angels.len(), 3);
    assert!(angels.iter().all(|a| t.pt(*a) == (4, 4)));
}
