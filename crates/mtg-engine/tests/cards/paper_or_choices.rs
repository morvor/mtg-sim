//! Implicit choices of older Oracle wordings (cards4-paper-effects, family `or-choices`):
//! a keyword list joined by "or" and two instructions joined by "or" are the controller's
//! choice as the ability resolves (CR 608.2d).

use crate::combat2_common::supported;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(k)
}

fn landwalk(t: &TestGame, id: ObjectId) -> bool {
    has(t, id, KeywordKind::Landwalk)
}

#[test]
fn vhati_il_dal_sets_base_power_or_base_toughness_as_it_resolves() {
    cr!("608.2d", "613.4b");
    ruling!(
        "Vhati il-Dal",
        "You don’t choose whether to affect the targeted creature’s power or its toughness until the ability resolves."
    );
    supported("Vhati il-Dal");
    for (choice, pt) in [(0, (1, 2)), (1, (2, 1))] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let vhati = t.battlefield(P0, "Vhati il-Dal");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.activate(P0, vhati, 0, &[Entity::Object(bears)]).unwrap();
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.resolve();
        assert_eq!(t.pt(bears), pt, "choice {choice}");
        // Until end of turn only.
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.pt(bears), (2, 2));
    }
}

#[test]
fn urborg_removes_first_strike_or_swampwalk_not_both() {
    cr!("608.2d");
    ruling!(
        "Urborg",
        "You choose whether the targeted creature loses first strike or swampwalk as the ability resolves. If the creature has both abilities, it will lose just one of them."
    );
    supported("Urborg");
    for choice in [0, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let urborg = t.battlefield(P0, "Urborg");
        let knight = t.battlefield(P1, "White Knight");
        // Give the knight swampwalk too, so it has both.
        let wraith = t.battlefield(P1, "Bog Wraith");
        assert!(landwalk(&t, wraith));
        // Urborg's second ability (the first is its mana ability).
        t.activate(P0, urborg, 1, &[Entity::Object(wraith)]).unwrap();
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.resolve();
        // Bog Wraith has swampwalk only: the swampwalk option removes it, the first
        // strike option does nothing.
        assert_eq!(landwalk(&t, wraith), choice == 0, "choice {choice}");
        assert!(has(&t, knight, KeywordKind::FirstStrike));
    }
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let urborg = t.battlefield(P0, "Urborg");
    let knight = t.battlefield(P1, "White Knight");
    t.activate(P0, urborg, 1, &[Entity::Object(knight)]).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve();
    assert!(!has(&t, knight, KeywordKind::FirstStrike));
    // Protection from black is untouched.
    assert!(has(&t, knight, KeywordKind::Protection));
    t.advance_to(P1, Step::Upkeep);
    assert!(has(&t, knight, KeywordKind::FirstStrike));
}

#[test]
fn flowstone_sculpture_puts_a_counter_or_gains_one_keyword_for_good() {
    cr!("608.2d", "611.2a");
    supported("Flowstone Sculpture");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = t.battlefield(P0, "Flowstone Sculpture");
    t.lands(P0, "Mountain", 4);
    let base = t.pt(s);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    // First activation: the counter.
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.activate(P0, s, 0, &[]).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve();
    assert_eq!(t.counters(s, "+1/+1"), 1);
    assert_eq!(t.pt(s), (base.0 + 1, base.1 + 1));
    assert!(!has(&t, s, KeywordKind::Trample));
    // Second activation: a keyword, chosen from the list (trample).
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.activate(P0, s, 0, &[]).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.resolve();
    assert_eq!(t.counters(s, "+1/+1"), 1);
    assert!(has(&t, s, KeywordKind::Trample));
    assert!(!has(&t, s, KeywordKind::Flying));
    assert!(!has(&t, s, KeywordKind::FirstStrike));
    // It lasts indefinitely.
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert!(has(&t, s, KeywordKind::Trample));
}

#[test]
fn natures_blessing_targets_once_for_either_instruction() {
    cr!("608.2d", "115.1");
    ruling!(
        "Nature's Blessing",
        "This ability has no duration. The affected creature will have banding, first strike, or trample until the game ends"
    );
    supported("Nature's Blessing");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let nb = t.battlefield(P0, "Nature's Blessing");
    t.lands(P0, "Savannah", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let discard = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(discard)]);
    t.activate(P0, nb, 0, &[Entity::Object(bears)]).unwrap();
    // The keyword branch, then banding.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert!(has(&t, bears, KeywordKind::Banding));
    assert!(!has(&t, bears, KeywordKind::Trample));
    t.advance_to(P1, Step::Upkeep);
    assert!(has(&t, bears, KeywordKind::Banding));
}
