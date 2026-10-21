//! Instructions after damage is prevented or replaced (CR 615.5, 120.4a): "prevent that
//! damage and an opponent of your choice gains control of it" (part of the replacement),
//! "Whenever damage from a black or red source is prevented this way this turn, you gain
//! that much life." and "When damage is prevented this way, ~ deals that much damage to
//! that source's controller and you draw that many cards." (triggered abilities each time
//! the prevention applies); and "Excess damage is dealt to that creature's controller
//! instead." after a triggered ability's damage.

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn ravenous_tyrannosaurus_deals_excess_damage_to_the_controller() {
    cr!("120.4a");
    ruling!(
        "Ravenous Tyrannosaurus",
        "Excess damage is calculated in a way similar to the way trample is handled"
    );
    assert_supported("Ravenous Tyrannosaurus");
    let mut t = TestGame::new(2);
    let rex = t.battlefield(P0, "Ravenous Tyrannosaurus");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // One damage already marked: 1 is lethal, 5 is excess.
    t.g.obj_mut(bears).damage = 1;
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(rex, Entity::Player(P1))], &[]);
    // 5 excess, then 6 combat damage.
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20 - 5 - 6);
}

#[test]
fn kharn_prevents_damage_and_an_opponent_gains_control_of_it() {
    cr!("615.5", "615.1a");
    ruling!("Khârn the Betrayer", "happens immediately");
    ruling!("Khârn the Betrayer", "won't go onto the stack until after the spell or ability");
    assert_supported("Khârn the Betrayer");
    let mut t = TestGame::new(2);
    let kharn = t.battlefield(P0, "Khârn the Betrayer");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    let hand = t.hand_size(P0);
    t.cast(P0, shock).target(kharn).go();
    t.resolve();
    assert_eq!(t.g.obj(kharn).damage, 0);
    assert_eq!(t.g.obj(kharn).controller, P1);
    // Sigil of Corruption: the trigger went on the stack after the spell resolved.
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn samite_ministration_gains_life_for_black_or_red_damage_prevented() {
    cr!("615.5", "609.7a", "603.2");
    assert_supported("Samite Ministration");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.lands(P0, "Plains", 2);
    let sm = t.hand(P0, "Samite Ministration");
    t.answer_choose(P0, &[Entity::Object(goblin)]);
    t.cast(P0, sm).go();
    t.resolve();
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(goblin, Entity::Player(P0))], &[]);
    // The damage was prevented, and P0 gained 1.
    assert_eq!(t.life(P0), 21);
    // A white source's damage is prevented without life gain.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "Savannah Lions");
    t.lands(P0, "Plains", 2);
    let sm = t.hand(P0, "Samite Ministration");
    t.answer_choose(P0, &[Entity::Object(knight)]);
    t.cast(P0, sm).go();
    t.resolve();
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(knight, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn new_way_forward_redirects_the_prevented_amount_and_draws() {
    cr!("615.5", "609.7a", "603.12");
    ruling!("New Way Forward", "You choose a source as New Way Forward resolves");
    assert_supported("New Way Forward");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let nwf = t.hand(P0, "New Way Forward");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.cast(P0, nwf).go();
    t.resolve();
    let hand = t.hand_size(P0);
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(wurm, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.hand_size(P0), hand + 6);
}
