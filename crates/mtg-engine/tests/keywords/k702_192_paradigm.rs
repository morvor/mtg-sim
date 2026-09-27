//! CR 702.192 Paradigm (`src/kw/paradigm.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Germination Practicum ({3}{G}{G} sorcery — Lesson): "Put two +1/+1 counters on each
/// creature you control. Paradigm".
const PRACTICUM: &str = "Germination Practicum";

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// Advances to player 0's next precombat main phase (on their next turn).
fn to_next_main(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
}

/// Advances to player 0's next precombat main phase and resolves what triggers there.
fn next_main(t: &mut TestGame) {
    to_next_main(t);
    t.resolve_all();
}

#[test]
fn paradigm_cards_compile() {
    assert_supported(&[
        PRACTICUM,
        "Decorum Dissertation",
        "Restoration Seminar",
    ]);
}

#[test]
fn the_spell_is_exiled_and_copied_at_each_first_main_phase() {
    cr!("702.192a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, PRACTICUM);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
    // "Exile this spell."
    assert!(t.in_exile(PRACTICUM));
    assert!(!t.in_graveyard(P0, PRACTICUM));
    // Nothing more this turn.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
    // At the beginning of each of its controller's precombat main phases: a copy is
    // created in exile and may be cast without paying its mana cost.
    next_main(&mut t);
    assert_eq!(plus1(&t, bears), 4);
    // The copy resolved and ceased to exist; the card is still the one exiled card.
    assert_eq!(exiled_count(&t, PRACTICUM), 1);
    // Not during the opponent's turn; again on the next turn of its controller.
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(plus1(&t, bears), 4);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 6);
}

#[test]
fn only_the_first_time_a_spell_with_that_name_resolves() {
    cr!("702.192a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 10);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, PRACTICUM);
    t.cast(P0, c).go();
    t.resolve_all();
    let c2 = t.hand(P0, PRACTICUM);
    t.cast(P0, c2).go();
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 4);
    assert_eq!(exiled_count(&t, PRACTICUM), 2);
    // One delayed triggered ability: one copy per turn.
    next_main(&mut t);
    assert_eq!(plus1(&t, bears), 6);
}

#[test]
fn a_countered_spell_doesnt_resolve() {
    cr!("702.192a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, PRACTICUM);
    let spell = t.cast(P0, c).go();
    run(
        &mut t,
        P1,
        None,
        Effect::CounterSpell {
            what: Sel::Target(0),
        },
        &[Entity::Object(spell)],
    );
    assert!(t.in_graveyard(P0, PRACTICUM));
    next_main(&mut t);
    assert_eq!(plus1(&t, bears), 0);
}

#[test]
fn the_card_leaving_exile_doesnt_matter() {
    cr!("702.192a");
    ruling!(
        "Decorum Dissertation",
        "Even if the card leaves exile, the delayed triggered ability will still trigger during each of your first main phases, and the copy will still be created."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, PRACTICUM);
    t.cast(P0, c).go();
    t.resolve_all();
    let card = t.g.find_in_zone(Zone::Exile, PRACTICUM)[0];
    run(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Library),
        },
        &[Entity::Object(card)],
    );
    assert_eq!(exiled_count(&t, PRACTICUM), 0);
    next_main(&mut t);
    assert_eq!(plus1(&t, bears), 4);
}

#[test]
fn the_copy_need_not_be_cast() {
    cr!("702.192a", "707.10a");
    ruling!(
        "Decorum Dissertation",
        "you can choose not to cast the copy. In that case, the copy will cease to exist the next time state-based actions are checked"
    );
    // Decorum Dissertation: "Target player draws two cards and loses 2 life. Paradigm".
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let c = t.hand(P0, "Decorum Dissertation");
    t.cast(P0, c).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    to_next_main(&mut t);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(exiled_count(&t, "Decorum Dissertation"), 1);
    // It triggers again on the next turn; this time the copy is cast.
    to_next_main(&mut t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(exiled_count(&t, "Decorum Dissertation"), 1);
}
