//! Changes to the costs of activated abilities (CR 601.2f, 602.2b, 118.7): "Activated
//! abilities of creatures you control cost {2} less", "Equip abilities you activate of
//! other Equipment cost {1} less", "During your turn, ... abilities your opponents
//! activate cost {1} more", "The first activated ability of an artifact you activate each
//! turn costs {2} less", "This ability costs {X} less to activate, where X is the power of
//! the creature it targets", channel abilities that cost less for each legendary creature,
//! and exiling a spell from the stack as a cost.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
}

fn untapped(t: &TestGame, lands: &[ObjectId]) -> usize {
    lands.iter().filter(|l| !t.obj_now(**l).tapped).count()
}

#[test]
fn biomancers_familiar_reduces_creature_abilities_to_no_less_than_one_mana() {
    cr!("601.2f", "602.2b", "118.7a");
    // (Its other ability, about adapting, isn't supported; this one is.)
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Biomancer's Familiar");
    let rogue = t.battlefield(P0, "Spike Rogue");
    counters(&mut t, rogue, "+1/+1", 2);
    // {2} less {2} would be {0}: it stays at {1}.
    assert!(t.activate(P0, rogue, 1, &[]).is_err());
    let forest = t.lands(P0, "Forest", 1);
    t.activate(P0, rogue, 1, &[]).unwrap();
    assert_eq!(untapped(&t, &forest), 0);
    t.resolve();
    assert_eq!(t.counters(rogue, "+1/+1"), 2);
}

#[test]
fn bladehold_war_whip_reduces_other_equipments_equip_costs() {
    cr!("601.2f", "602.2b", "702.6a");
    compiles("Bladehold War-Whip");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let whip = t.battlefield(P0, "Bladehold War-Whip");
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Bonesplitter's equip {1} costs {0}.
    t.activate(P0, splitter, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(
        t.obj_now(splitter).attached_to,
        Some(Entity::Object(bears))
    );
    // The War-Whip's own equip {3}{R}{W}... isn't reduced: no mana, no equip.
    assert!(t.activate(P0, whip, 0, &[Entity::Object(bears)]).is_err());
}

#[test]
fn tithe_taker_taxes_opponents_during_your_turn() {
    cr!("601.2f", "602.2b");
    compiles("Tithe Taker");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Tithe Taker");
    let herbalist = t.battlefield(P1, "Royal Herbalist");
    t.library_top(P1, "Shock");
    let plains = t.lands(P1, "Plains", 2);
    // {2} + {1} during P0's turn.
    assert!(t.activate(P1, herbalist, 0, &[]).is_err());
    assert_eq!(untapped(&t, &plains), 2);
    t.lands(P1, "Plains", 1);
    t.activate(P1, herbalist, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 21);
    // Not during P1's own turn.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Tithe Taker");
    let herbalist = t.battlefield(P1, "Royal Herbalist");
    t.library_top(P1, "Shock");
    t.lands(P1, "Plains", 2);
    t.activate(P1, herbalist, 0, &[]).unwrap();
}

#[test]
fn tezzeret_reduces_only_the_first_artifact_ability_each_turn() {
    cr!("601.2f", "602.2b", "118.7a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Tezzeret, Betrayer of Flesh");
    let a = t.battlefield(P0, "Mind Stone");
    let b = t.battlefield(P0, "Mind Stone");
    // Mind Stone: "{1}, {T}, Sacrifice this artifact: Draw a card." costs {0} first.
    let hand = t.hand_size(P0);
    t.activate(P0, a, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    // The second one costs {1}, which can only come from Mind Stone itself... tapping
    // it for mana would leave it unable to pay {T}.
    assert!(t.activate(P0, b, 1, &[]).is_err());
    assert_eq!(t.zone(b), Zone::Battlefield);
}

#[test]
fn belt_of_giant_strength_costs_less_by_the_targets_power() {
    cr!("601.2c", "601.2f", "602.2b");
    compiles("Belt of Giant Strength");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let belt = t.battlefield(P0, "Belt of Giant Strength");
    let wurm = t.battlefield(P0, "Craw Wurm"); // power 6
    let lands = t.lands(P0, "Forest", 4);
    // Equip {10} less {6}.
    t.activate(P0, belt, 0, &[Entity::Object(wurm)]).unwrap();
    assert_eq!(untapped(&t, &lands), 0);
    t.resolve();
    assert_eq!(t.pt(wurm), (10, 10));
}

#[test]
fn otawara_channel_costs_less_for_each_legendary_creature() {
    cr!("601.2f", "602.2b", "113.6j");
    compiles("Otawara, Soaring City");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let otawara = t.hand(P0, "Otawara, Soaring City");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    let islands = t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // {3}{U} less {2}: two Islands.
    t.activate(P0, otawara, 1, &[Entity::Object(bears)]).unwrap();
    assert_eq!(untapped(&t, &islands), 0);
    assert!(t.in_graveyard(P0, "Otawara, Soaring City"));
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn nivmagus_elemental_exiles_a_spell_you_control() {
    cr!("602.2b", "118.3");
    compiles("Nivmagus Elemental");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let niv = t.battlefield(P0, "Nivmagus Elemental");
    // No spell on the stack: it can't be activated.
    assert!(t.activate(P0, niv, 0, &[]).is_err());
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.answer_choose(P0, &[Entity::Object(spell)]);
    t.activate(P0, niv, 0, &[]).unwrap();
    assert!(t.in_exile("Shock"));
    t.resolve();
    assert_eq!(t.counters(niv, "+1/+1"), 2);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
}
