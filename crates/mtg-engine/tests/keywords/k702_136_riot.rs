//! CR 702.136 Riot.

use crate::common_k702_125_139::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn riot_questions(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Riot")))
        .count()
}

#[test]
fn riot_enters_with_a_counter_if_you_choose() {
    cr!("702.136", "702.136a");
    assert_supported_card("Zhur-Taa Goblin");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let g = t.hand(P0, "Zhur-Taa Goblin");
    t.cast(P0, g).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(g));
    assert_eq!(plus1(&t, g), 1);
    assert_eq!(t.pt(g), (3, 3));
    assert!(!has(&t, g, KeywordKind::Haste));
    assert_eq!(riot_questions(&t), 1);
}

#[test]
fn if_you_dont_it_gains_haste_indefinitely() {
    cr!("702.136a");
    ruling!(
        "Zhur-Taa Goblin",
        "If you choose for the creature to gain haste, it gains haste indefinitely. It won’t lose it as the turn ends or as another player gains control of it."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let g = t.hand(P0, "Zhur-Taa Goblin");
    t.cast(P0, g).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(plus1(&t, g), 0);
    assert_eq!(t.pt(g), (2, 2));
    assert!(has(&t, g, KeywordKind::Haste));
    // It can attack the turn it entered.
    let g = t.g.current(g);
    attack_with(&mut t, &[(g, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
    // Still has haste on later turns and under another player's control.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(has(&t, g, KeywordKind::Haste));
    let now = t.g.current(g);
    run(
        &mut t,
        P1,
        mtg_engine::ability::Effect::GainControl {
            what: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::Objects(vec![now])),
            who: mtg_engine::ability::PlayerRef::You,
            duration: mtg_engine::ability::Duration::Permanent,
        },
    );
    assert_eq!(t.obj_now(g).controller, P1);
    assert!(has(&t, g, KeywordKind::Haste));
}

#[test]
fn riot_applies_however_the_permanent_enters() {
    cr!("702.136a");
    ruling!(
        "Zhur-Taa Goblin",
        "Riot is a replacement effect. Players can’t respond to your choice of +1/+1 counter or haste, and they can’t take actions while the creature is on the battlefield without one or the other."
    );
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let g = t.enter(P0, "Zhur-Taa Goblin");
    // The counter is on it as it enters: it was never on the battlefield without it.
    assert_eq!(plus1(&t, g), 1);
    assert_eq!(riot_questions(&t), 1);
}

#[test]
fn each_instance_of_riot_works_separately() {
    cr!("702.136b");
    assert_supported_card("Rhythm of the Wild");
    ruling!(
        "Rhythm of the Wild",
        "If a creature enters the battlefield with two instances of riot, you may choose to have it get two +1/+1 counters, one +1/+1 counter and haste, or two instances of haste."
    );
    // Two counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhythm of the Wild");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let g = t.enter(P0, "Zhur-Taa Goblin");
    assert_eq!(keyword_count(&t, g, KeywordKind::Riot), 2);
    assert_eq!(riot_questions(&t), 2);
    assert_eq!(plus1(&t, g), 2);
    assert!(!has(&t, g, KeywordKind::Haste));
    // A counter and haste.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhythm of the Wild");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let g = t.enter(P0, "Zhur-Taa Goblin");
    assert_eq!(plus1(&t, g), 1);
    assert!(has(&t, g, KeywordKind::Haste));
}

#[test]
fn a_granted_riot_applies_to_creatures_entering_and_what_it_gave_stays() {
    cr!("702.136a");
    ruling!(
        "Rhythm of the Wild",
        "Once a creature with riot has entered the battlefield, it keeps its +1/+1 counter or haste even if it loses riot."
    );
    let mut t = TestGame::new(2);
    let rhythm = t.battlefield(P0, "Rhythm of the Wild");
    t.answer_yes(P0, false);
    let bears = t.enter(P0, "Grizzly Bears");
    assert!(has(&t, bears, KeywordKind::Haste));
    t.g.move_object(
        rhythm,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.recompute();
    assert!(!has(&t, bears, KeywordKind::Riot));
    assert!(has(&t, bears, KeywordKind::Haste));
}

#[test]
fn a_creature_that_cant_have_counters_gains_haste() {
    cr!("702.136a");
    ruling!(
        "Zhur-Taa Goblin",
        "If a creature entering the battlefield has riot but can’t have a +1/+1 counter put onto it, it gains haste."
    );
    let def = custom_card(
        "Counterless Rioter",
        "Creature — Goblin",
        Some((2, 2)),
        "Riot\nThis creature can't have counters put on it.",
    );
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let c = t.custom(P0, def, Zone::Hand(P0));
    t.g.move_object(
        c,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.flush_events();
    let c = t.g.current(c);
    assert_eq!(plus1(&t, c), 0);
    assert!(has(&t, c, KeywordKind::Haste));
}
