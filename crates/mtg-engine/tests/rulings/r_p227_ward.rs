//! Rulings batch P227 — ward (CR 702.21): several instances of ward, granted ward that
//! has already triggered, ward costs that depend on a card in hand or on the source's
//! power, and Maha, Its Feathers Night's base toughness.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Ward triggers on the stack.
fn ward_triggers(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_deref().is_some_and(|si| {
                matches!(&si.kind, StackKind::Triggered { ability, .. }
                    if ability.text.starts_with("Ward"))
            })
        })
        .count()
}

/// Instances of ward `id` has.
fn wards(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Ward)
        .count()
}

/// P1 casts Shock targeting `target` (with `extra` more lands); the ward triggers are put
/// on the stack and the number of them is returned.
fn p1_shocks(t: &mut TestGame, target: ObjectId, extra: usize) -> usize {
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(t, P1, "Shock");
    t.lands(P1, "Wastes", extra);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Object(target)).go();
    t.settle();
    ward_triggers(t)
}

/// P0's Xenograft, with Bird chosen: each creature P0 controls is a Bird.
fn xenograft_bird(t: &mut TestGame) {
    use mtg_engine::decision::{Answer, Decision};
    // The creature types offered, to find Bird's index.
    let mut probe = TestGame::new(2);
    probe.enter(P0, "Xenograft");
    let options = probe
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .unwrap();
    let bird = options.iter().position(|o| o == "Bird").unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(bird));
    let x = t.enter(P0, "Xenograft");
    assert_eq!(
        t.obj_now(x).choices.creature_type.as_deref(),
        Some("Bird")
    );
}

#[test]
fn radagast_grants_one_ward_to_a_beast_bird() {
    cr!("702.21a", "613.1f");
    ruling!(
        "Radagast, Wizard of Wilds",
        "A creature that is both a Beast and a Bird still only gets one instance of ward {1} from Radagast, Wizard of Wilds's first ability."
    );
    supported("Radagast, Wizard of Wilds");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Radagast, Wizard of Wilds");
    let beast = t.battlefield(P0, "Anurid Barkripper"); // a Frog Beast
    assert_eq!(wards(&t, beast), 1);
    xenograft_bird(&mut t);
    assert!(t.obj_now(beast).chars.has_subtype("Bird"));
    assert!(t.obj_now(beast).chars.has_subtype("Beast"));
    assert_eq!(wards(&t, beast), 1);
    assert_eq!(p1_shocks(&mut t, beast, 0), 1);
    t.resolve_all();
    // Unpaid: Shock is countered.
    assert!(t.on_battlefield(beast));
    assert_eq!(t.obj_now(beast).damage, 0);
}

#[test]
fn radagast_as_a_bird_has_two_wards() {
    cr!("702.21a", "603.2", "613.1f");
    ruling!(
        "Radagast, Wizard of Wilds",
        "If Radagast, Wizard of Wilds is a Beast or a Bird, it will have two instances of ward {1}."
    );
    supported("Radagast, Wizard of Wilds");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Radagast, Wizard of Wilds");
    assert_eq!(wards(&t, r), 1);
    xenograft_bird(&mut t);
    assert_eq!(wards(&t, r), 2);
    // Both trigger; P1 can pay one {1} but not both: Shock is countered.
    assert_eq!(p1_shocks(&mut t, r, 1), 2);
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.obj_now(r).damage, 0);
}

/// P0's Winter, Cursed Rider made an artifact by P0's Liquimetal Coating.
fn artifact_winter() -> (TestGame, ObjectId) {
    supported("Winter, Cursed Rider");
    supported("Liquimetal Coating");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let winter = t.battlefield(P0, "Winter, Cursed Rider");
    assert_eq!(wards(&t, winter), 1);
    let coating = t.battlefield(P0, "Liquimetal Coating");
    t.activate(P0, coating, 0, &[Entity::Object(winter)])
        .expect("Liquimetal Coating");
    t.resolve_all();
    assert!(t.obj_now(winter).chars.is(CardType::Artifact));
    (t, winter)
}

#[test]
fn winter_as_an_artifact_has_two_wards_that_trigger_separately() {
    cr!("702.21a", "603.2", "613.1f");
    ruling!(
        "Winter, Cursed Rider",
        "If an effect causes Winter to become an artifact, its second ability will cause it to gain a second instance of ward."
    );
    ruling!(
        "Winter, Cursed Rider",
        "Multiple instances of ward each trigger separately."
    );
    // P1 pays 2 life for each: Shock resolves.
    let (mut t, winter) = artifact_winter();
    assert_eq!(wards(&t, winter), 2);
    assert_eq!(p1_shocks(&mut t, winter, 0), 2);
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(winter));
    // P1 pays only once: Shock is countered.
    let (mut t, winter) = artifact_winter();
    assert_eq!(p1_shocks(&mut t, winter, 0), 2);
    t.answer_yes(P1, true);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(winter));
    assert!(t.in_graveyard(P1, "Shock"));
}

#[test]
fn saruman_ward_cant_be_paid_without_such_a_card() {
    cr!("702.21a", "118.3", "701.9a");
    ruling!(
        "Saruman of Many Colors",
        "If you don't have an enchantment, instant, or sorcery card in your hand, you won't be able to pay Saruman of Many Colors's ward cost."
    );
    // The ward ability compiles (only the second-spell trigger isn't supported).
    let c = card("Saruman of Many Colors");
    assert!(c
        .unsupported_text()
        .iter()
        .all(|u| u.starts_with("Whenever you cast your second spell")));
    // P1 holds only a creature card: Shock is countered.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Saruman of Many Colors");
    t.hand(P1, "Grizzly Bears");
    assert_eq!(p1_shocks(&mut t, s, 0), 1);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(s).damage, 0);
    // With an instant card in hand, P1 discards it and Shock resolves.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Saruman of Many Colors");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Giant Growth");
    assert_eq!(p1_shocks(&mut t, s, 0), 1);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Giant Growth"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(s).damage, 2);
}

#[test]
fn long_river_lurker_granted_ward_already_triggered() {
    cr!("702.21a", "603.4", "113.7a");
    ruling!(
        "Long River Lurker",
        "Once a ward ability of another Frog has triggered, causing it to lose ward by removing Long River Lurker or changing its creature types won't affect that ability."
    );
    // The ward lines compile (only the enters trigger isn't supported).
    let c = card("Long River Lurker");
    assert!(c
        .unsupported_text()
        .iter()
        .all(|u| u.starts_with("When ~ enters")));
    supported("Yargle, Goliath of Otaria");
    let mut t = TestGame::new(2);
    let lurker = t.battlefield(P0, "Long River Lurker");
    let frog = t.battlefield(P0, "Yargle, Goliath of Otaria");
    assert_eq!(wards(&t, frog), 1);
    assert_eq!(p1_shocks(&mut t, frog, 0), 1);
    destroy(&mut t, lurker);
    assert_eq!(wards(&t, frog), 0);
    // P1 can't pay {1}: Shock is still countered.
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.obj_now(frog).damage, 0);
}

/// P1 Shocks P0's Raubahn; in response P0 casts Giant Growth on it (power 5), and with
/// `remove`, Raubahn is then destroyed. P1 pays the ward cost.
fn raubahn_ward(remove: bool) -> TestGame {
    // The ward line compiles (only the attack trigger isn't supported).
    let c = card("Raubahn, Bull of Ala Mhigo");
    assert!(c
        .unsupported_text()
        .iter()
        .all(|u| u.starts_with("Whenever ~ attacks")));
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Raubahn, Bull of Ala Mhigo");
    assert_eq!(p1_shocks(&mut t, r, 0), 1);
    give_mana_for(&mut t, P0, "Giant Growth");
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(Entity::Object(r)).go();
    t.resolve();
    assert_eq!(t.pt(r), (5, 5));
    if remove {
        destroy(&mut t, r);
    }
    t.answer_yes(P1, true);
    t.resolve_all();
    t
}

#[test]
fn raubahn_ward_uses_power_as_it_resolves() {
    cr!("702.21a", "702.21b", "608.2h", "113.7a");
    ruling!(
        "Raubahn, Bull of Ala Mhigo",
        "Use Raubahn's power at the time the ward ability resolves to determine how much life must be paid."
    );
    let t = raubahn_ward(false);
    assert_eq!(t.life(P1), 15);
    let r = t.named_on_battlefield("Raubahn, Bull of Ala Mhigo")[0];
    assert_eq!(t.obj_now(r).damage, 2);
    // Gone: its last known power.
    let t = raubahn_ward(true);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn maha_overwrites_earlier_base_toughness_and_later_ones_overwrite_it() {
    cr!("613.4b", "613.7", "611.3a");
    ruling!(
        "Maha, Its Feathers Night",
        "Maha’s last ability overwrites all previous effects that set those creatures’ base toughness to specific values."
    );
    supported("Maha, Its Feathers Night");
    supported("Kenrith's Transformation");
    let transform = |t: &mut TestGame, target: ObjectId| {
        t.set_step(P0, Step::PrecombatMain);
        give_mana_for(t, P0, "Kenrith's Transformation");
        let k = t.hand(P0, "Kenrith's Transformation");
        t.cast(P0, k).target(Entity::Object(target)).go();
        t.resolve_all();
    };
    // Kenrith's Transformation (base 3/3 Elk) first, then Maha: 3/1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    transform(&mut t, bears);
    assert_eq!(t.pt(bears), (3, 3));
    t.battlefield(P0, "Maha, Its Feathers Night");
    assert_eq!(t.pt(bears), (3, 1));
    // Maha first, then Kenrith's Transformation: 3/3.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Maha, Its Feathers Night");
    assert_eq!(t.pt(bears), (2, 1));
    transform(&mut t, bears);
    assert_eq!(t.pt(bears), (3, 3));
}

/// P0's Loot, the Key to Everything's upkeep trigger resolves with `others` on P0's
/// battlefield; returns how many cards it exiled from P0's library.
fn loot_exiles(others: &[&str]) -> usize {
    supported("Loot, the Key to Everything");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loot, the Key to Everything");
    for o in others {
        t.battlefield(P0, o);
    }
    stack_library(&mut t, P0, &["Island"; 6]);
    let before = t.library_size(P0);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    before - t.library_size(P0)
}

#[test]
fn loot_disregards_lands_when_counting_card_types() {
    cr!("205.2a", "608.2h");
    ruling!(
        "Loot, the Key to Everything",
        "Lands are disregarded when calculating the value of X. For example, if you control an artifact land but no other artifacts, artifact won't count toward the value of X."
    );
    // Loot itself doesn't count ("other").
    assert_eq!(loot_exiles(&[]), 0);
    // An artifact land: nothing counts.
    assert_eq!(loot_exiles(&["Darksteel Citadel"]), 0);
    // Plus Grizzly Bears: creature only.
    assert_eq!(loot_exiles(&["Darksteel Citadel", "Grizzly Bears"]), 1);
    // Plus Ornithopter (an artifact creature): artifact and creature.
    assert_eq!(
        loot_exiles(&["Darksteel Citadel", "Grizzly Bears", "Ornithopter"]),
        2
    );
    // Plus an enchantment: three.
    assert_eq!(
        loot_exiles(&["Darksteel Citadel", "Ornithopter", "Xenograft"]),
        3
    );
}
