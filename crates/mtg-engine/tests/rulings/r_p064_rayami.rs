//! Rulings batch P064 — Rayami, First of the Fallen: its replacement effect on dying
//! creatures (CR 614.6, 616.1, 700.4) and the keywords it gets from exiled creature cards
//! with blood counters (only their abilities that function in exile, 113.6).

use crate::r_p064_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts the real creature card `name` into exile with a blood counter on it.
fn exiled_with_blood(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.exile(p, name);
    t.g.objects[id.0 as usize].counters.insert("blood".into(), 1);
    t.g.recompute();
    id
}

/// The exiled card the object (followed across zone changes) became, and its blood
/// counters.
fn blood_in_exile(t: &TestGame, id: ObjectId) -> Option<u32> {
    (t.zone(id) == Zone::Exile).then(|| t.counters(id, "blood"))
}

#[test]
fn rayami_exiles_a_dying_vehicle_but_doesnt_gain_its_abilities() {
    cr!("614.6", "700.4", "301.7b");
    ruling!(
        "Rayami, First of the Fallen",
        "If a nontoken creature that isn't normally a creature (such as a Vehicle) would die, Rayami exiles it with a blood counter on it, although Rayami won't gain any of its abilities."
    );
    supported("Rayami, First of the Fallen");
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rayami, First of the Fallen");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(crate::r_s20_common::creature_now(&mut t, copter));
    crate::r_s02_common::destroy(&mut t, copter);
    assert_eq!(blood_in_exile(&t, copter), Some(1));
    assert!(
        !flies(&mut t, r),
        "the exiled Copter isn't a creature card"
    );
}

#[test]
fn rayami_ignores_conditional_keywords_of_exiled_cards() {
    cr!("113.6", "611.3a");
    ruling!(
        "Rayami, First of the Fallen",
        "If an exiled creature card has one of the listed keywords only if certain conditions are met"
    );
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rayami, First of the Fallen");
    exiled_with_blood(&mut t, P1, "Kitesail Corsair");
    assert!(!flies(&mut t, r), "Kitesail Corsair has flying only while attacking");
    exiled_with_blood(&mut t, P1, "Wind Drake");
    assert!(flies(&mut t, r));
}

#[test]
fn rayami_gains_the_specific_variants() {
    cr!("702.16a", "702.16b");
    ruling!(
        "Rayami, First of the Fallen",
        "If an exiled creature card has one or more variants of the listed keywords (for example, protection from black or hexproof from white), Rayami gains those specific variants."
    );
    supported("White Knight");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rayami, First of the Fallen");
    exiled_with_blood(&mut t, P0, "White Knight");
    assert!(has(&mut t, r, KeywordKind::FirstStrike));
    assert!(has(&mut t, r, KeywordKind::Protection));
    let murder = crate::r_s04_common::spell_targets(&mut t, P1, "Murder");
    assert!(!murder.contains(&obj(r)), "protection from black");
    let shock = crate::r_s04_common::spell_targets(&mut t, P1, "Shock");
    assert!(shock.contains(&obj(r)), "only from black");
}

/// Picks Rest in Peace's replacement effect when the order is asked.
fn rest_in_peace_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| o.contains("Rest in Peace"))
            .map(Answer::Index),
        _ => None,
    }
}

#[test]
fn rayami_exile_competing_with_another_replacement_effect() {
    cr!("616.1", "616.1e", "614.6");
    ruling!(
        "Rayami, First of the Fallen",
        "If multiple replacement effects try to change where a dying creature would go, the controller of the creature chooses one to apply."
    );
    supported("Rest in Peace");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rayami, First of the Fallen");
    t.battlefield(P0, "Rest in Peace");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s03_common::respond(&mut t, P1, rest_in_peace_first);
    let from = t.asked().len();
    crate::r_s02_common::destroy(&mut t, bears);
    assert!(
        t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseReplacement { .. })),
        "the creature's controller chose"
    );
    assert_eq!(
        blood_in_exile(&t, bears),
        Some(0),
        "exiled by Rest in Peace: no blood counter"
    );
}

#[test]
fn creatures_dying_with_rayami_are_exiled_with_blood_counters() {
    cr!("614.6", "603.10a");
    ruling!(
        "Rayami, First of the Fallen",
        "Nontoken creatures that would die at the same time that Rayami leaves the battlefield are exiled with blood counters."
    );
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rayami, First of the Fallen");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(blood_in_exile(&t, bears), Some(1));
    assert_eq!(blood_in_exile(&t, r), Some(1), "Rayami itself, too");
}

#[test]
fn rayami_looks_at_every_exiled_creature_card_with_a_blood_counter() {
    cr!("613.1f", "122.1");
    ruling!(
        "Rayami, First of the Fallen",
        "Rayami looks at all exiled creature cards with blood counters, not only those exiled with its ability."
    );
    // A card exiled by another player's Rayami.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Rayami, First of the Fallen");
    t.battlefield(P1, "Rayami, First of the Fallen");
    let angel = t.battlefield(P1, "Serra Angel");
    crate::r_s02_common::destroy(&mut t, angel);
    assert_eq!(blood_in_exile(&t, angel), Some(1));
    assert!(flies(&mut t, mine));
    assert!(has(&mut t, mine, KeywordKind::Vigilance));
}
