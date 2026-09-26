//! CR 702.148 Cleave.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const CLEAVE: CastMethod = CastMethod::Keyword(KeywordKind::Cleave);

/// The candidates offered for the first target chosen since decision `from`.
fn offered(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn cleave_is_an_alternative_cost_that_removes_the_bracketed_text() {
    cr!("702.148", "702.148a");
    assert_supported("Fierce Retribution");
    ruling!(
        "Lantern Flare",
        "A cleave cost is an alternative cost that's paid instead of the spell's mana cost. Casting a spell for its cleave cost doesn't change the spell's mana value."
    );
    // Fierce Retribution: {1}{W} "Destroy target [attacking] creature." Cleave {5}{W}.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Fierce Retribution");
    // Cast normally, it can target only an attacking creature.
    add_mana(&mut t, P0, ManaType::W, 2);
    let from = t.asked().len();
    assert!(t.cast(P0, card).target(bears).try_go().is_err());
    assert!(!offered(&t, from).contains(&Entity::Object(bears)));
    t.clear_answers();
    // For its cleave cost, any creature. (The failed attempt was undone: the {W}{W} is
    // still in the pool.)
    assert_eq!(pool(&t, P0), 2);
    add_mana(&mut t, P0, ManaType::C, 4);
    let spell = t.cast(P0, card).method(CLEAVE).target(bears).go();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn cast_normally_the_bracketed_text_applies() {
    cr!("702.148a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(giant, Entity::Player(P0))]);
    let card = t.hand(P0, "Fierce Retribution");
    add_mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, card).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn path_of_peril_destroys_all_creatures_when_cleaved() {
    cr!("702.148a");
    assert_supported("Path of Peril");
    for (cleave, survivors) in [(false, 1), (true, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Hill Giant");
        let card = t.hand(P0, "Path of Peril");
        add_mana(&mut t, P0, ManaType::W, 1);
        add_mana(&mut t, P0, ManaType::B, 2);
        add_mana(&mut t, P0, ManaType::C, 4);
        let c = t.cast(P0, card);
        if cleave {
            c.method(CLEAVE).go();
        } else {
            c.go();
        }
        t.resolve_all();
        let left = t.g.permanents().filter(|o| o.is_creature()).count();
        assert_eq!(left, survivors, "cleave {cleave}");
    }
}

#[test]
fn a_cleaved_spell_doesnt_have_the_bracketed_text_on_the_stack() {
    cr!("702.148a", "702.148b");
    ruling!(
        "Lantern Flare",
        "If you cast a spell for its cleave cost, that spell doesn't have any of the text in square brackets while it's on the stack."
    );
    ruling!(
        "Lantern Flare",
        "If you're paying the cleave cost, you choose the value of X."
    );
    // Lantern Flare: {1}{W} "~ deals X damage to target creature or planeswalker and you
    // gain X life. [X is the number of creatures you control.]" Cleave {X}{R}{W}.
    assert_supported("Lantern Flare");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Lantern Flare");
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    let spell = t.cast(P0, card).method(CLEAVE).x(3).target(giant).go();
    let text = t.obj(spell).chars.rules_text.to_string();
    assert!(!text.contains('['), "{text}");
    assert!(!text.contains("X is the number of creatures you control"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 23);
    // Cast normally, X is the number of creatures you control.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Lantern Flare");
    add_mana(&mut t, P0, ManaType::W, 2);
    let spell = t.cast(P0, card).target(giant).go();
    assert!(t.obj(spell).chars.rules_text.contains('['));
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.g.obj(giant).damage, 2);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn a_copy_of_a_cleaved_spell_is_cleaved_too() {
    cr!("702.148b", "707.10");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Fierce Retribution");
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 5);
    let spell = t.cast(P0, card).method(CLEAVE).target(a).go();
    // Reverberate: "Copy target instant or sorcery spell. You may choose new targets for
    // the copy." The copy's new target needn't be attacking either.
    let rev = t.hand(P0, "Reverberate");
    add_mana(&mut t, P0, ManaType::R, 2);
    t.cast(P0, rev).target(spell).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn wash_away_counters_only_spells_not_cast_from_hand_unless_cleaved() {
    cr!("702.148a");
    assert_supported("Wash Away");
    ruling!(
        "Wash Away",
        "Copies of spells are never cast from a player's hand, so Wash Away can counter copies of spells even if its cleave cost wasn't paid"
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    // P1 casts a spell from their hand: Wash Away can't target it without its cleave cost.
    let bears = t.hand(P1, "Grizzly Bears");
    add_mana(&mut t, P1, ManaType::G, 2);
    let spell = t.cast(P1, bears).go();
    let wash = t.hand(P0, "Wash Away");
    add_mana(&mut t, P0, ManaType::U, 1);
    let from = t.asked().len();
    assert!(t.cast(P0, wash).target(spell).try_go().is_err());
    assert!(!offered(&t, from).contains(&Entity::Object(spell)));
    t.clear_answers();
    add_mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, wash).method(CLEAVE).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));

    // A copy of a spell cast from a hand can be countered by it cast normally.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P1, ManaType::R, 1);
    let original = t.cast(P1, bolt).target(a).go();
    let twin = t.hand(P1, "Twincast");
    add_mana(&mut t, P1, ManaType::U, 2);
    t.cast(P1, twin).target(original).go();
    t.answer_yes(P1, false);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, original);
    let wash = t.hand(P0, "Wash Away");
    add_mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, wash).target(copy).go();
    t.resolve();
    assert!(!t.g.stack.contains(&copy));
    assert!(t.g.stack.contains(&original));
}

#[test]
fn dig_up_finds_any_card_when_cleaved() {
    cr!("702.148a");
    assert_supported("Dig Up");
    ruling!(
        "Dig Up",
        "If you paid the cleave cost, you must put a card into your hand as Dig Up resolves"
    );
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let card = t.hand(P0, "Dig Up");
    add_mana(&mut t, P0, ManaType::B, 2);
    add_mana(&mut t, P0, ManaType::G, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, card).method(CLEAVE).go();
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn inspired_idea_reduces_your_maximum_hand_size_unless_cleaved() {
    cr!("702.148a");
    assert_supported("Inspired Idea");
    for (cleave, max) in [(false, Some(4)), (true, Some(7))] {
        let mut t = TestGame::new(2);
        let card = t.hand(P0, "Inspired Idea");
        add_mana(&mut t, P0, ManaType::U, 2);
        add_mana(&mut t, P0, ManaType::C, 3);
        let hand = t.hand_size(P0);
        let c = t.cast(P0, card);
        if cleave {
            c.method(CLEAVE).go();
        } else {
            c.go();
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand - 1 + 3);
        assert_eq!(t.player(P0).max_hand_size, max, "cleave {cleave}");
    }
}

#[test]
fn alchemists_gambit_loses_the_game_only_if_not_cleaved() {
    cr!("702.148a");
    assert_supported("Alchemist's Gambit");
    ruling!(
        "Alchemist's Gambit",
        "(If you cast Alchemist's Gambit by paying its cleave cost, the delayed triggered ability is never created at all.)"
    );
    for cleave in [false, true] {
        let mut t = TestGame::new(2);
        let card = t.hand(P0, "Alchemist's Gambit");
        add_mana(&mut t, P0, ManaType::U, 2);
        add_mana(&mut t, P0, ManaType::R, 2);
        add_mana(&mut t, P0, ManaType::C, 3);
        let c = t.cast(P0, card);
        if cleave {
            c.method(CLEAVE).go();
        } else {
            c.go();
        }
        t.resolve_all();
        assert!(t.in_exile("Alchemist's Gambit"));
        let turn = t.g.turn.number;
        // The extra turn is P0's next turn; at its end step P0 loses unless cleaved.
        t.advance_to(P0, Step::Upkeep);
        assert_eq!(t.g.turn.number, turn + 1);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        t.settle();
        assert_eq!(t.has_lost(P0), !cleave, "cleave {cleave}");
    }
}

#[test]
fn a_spell_cast_without_paying_its_mana_cost_isnt_cleaved() {
    cr!("702.148a");
    ruling!(
        "Lantern Flare",
        "If an effect allows you to “cast a spell without paying its mana cost,” you can't cast that spell for its cleave cost."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(giant, Entity::Player(P0))]);
    let card = t.hand(P0, "Fierce Retribution");
    // Cast without paying its mana cost, it's "Destroy target attacking creature": the
    // Bears aren't a legal target, the attacking Hill Giant is.
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(giant)]);
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::CastCard {
            who: mtg_engine::ability::PlayerRef::You,
            what: mtg_engine::ability::Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(card)],
    );
    let candidates = offered(&t, from);
    assert!(candidates.contains(&Entity::Object(giant)));
    assert!(!candidates.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(bears));
}
