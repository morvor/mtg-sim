//! Structure-coverage seed tests: triggered-ability structures shared by many cards
//! (see `docs/STRUCTURE_COVERAGE.md`). Each test checks a real card's ability does what
//! its text says.

use super::structure_seed_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn raucous_theater_enters_tapped_and_surveils_1() {
    cr!("603.6a", "701.25a", "614.1d");
    // "This land enters tapped. When this land enters, surveil 1."
    supported("Raucous Theater");
    let mut t = TestGame::new(2);
    let land = t.enter(P0, "Raucous Theater");
    assert!(t.obj_now(land).tapped);
    t.resolve_all();
    assert_eq!(surveils(&t), vec![1]);
}

#[test]
fn seedship_broodtender_mills_three_on_entering() {
    cr!("603.6a", "701.17a");
    // "When this creature enters, mill three cards."
    supported("Seedship Broodtender");
    let mut t = TestGame::new(2);
    t.enter(P0, "Seedship Broodtender");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn owl_familiar_draws_then_discards() {
    cr!("603.6a", "121.1", "701.9a");
    // "When this creature enters, draw a card, then discard a card."
    supported("Owl Familiar");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    let library = t.library_size(P0);
    t.enter(P0, "Owl Familiar");
    t.resolve_all();
    assert_eq!(t.library_size(P0), library - 1);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn guardian_gladewalker_puts_a_counter_on_target_creature() {
    cr!("603.6a", "122.6");
    // "When this creature enters, put a +1/+1 counter on target creature."
    supported("Guardian Gladewalker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Guardian Gladewalker");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn merfolk_skydiver_targets_only_a_creature_you_control() {
    cr!("603.6a", "115.1", "122.6");
    // "When this creature enters, put a +1/+1 counter on target creature you control."
    supported("Merfolk Skydiver");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    // The opponent's creature isn't a legal target: the answer falls back to a legal one.
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.enter(P0, "Merfolk Skydiver");
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    let skydiver = t.named_on_battlefield("Merfolk Skydiver")[0];
    assert_eq!(
        t.counters(mine, "+1/+1") + t.counters(skydiver, "+1/+1"),
        1,
        "the counter goes on a creature P0 controls"
    );
}

#[test]
fn trail_of_crumbs_creates_a_food() {
    cr!("603.6a", "111.10b");
    // "When this enchantment enters, create a Food token."
    supported("Trail of Crumbs");
    let mut t = TestGame::new(2);
    t.enter(P0, "Trail of Crumbs");
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Food"), 1);
}

#[test]
fn rapacious_dragon_creates_two_treasures() {
    cr!("603.6a", "111.10a");
    // "When this creature enters, create two Treasure tokens."
    supported("Rapacious Dragon");
    let mut t = TestGame::new(2);
    t.enter(P0, "Rapacious Dragon");
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Treasure"), 2);
}

#[test]
fn loxodon_eavesdropper_investigates() {
    cr!("603.6a", "701.16a", "111.10f");
    // "When this creature enters, investigate."
    supported("Loxodon Eavesdropper");
    let mut t = TestGame::new(2);
    t.enter(P0, "Loxodon Eavesdropper");
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Clue"), 1);
}

#[test]
fn lilianas_specter_makes_each_opponent_discard() {
    cr!("603.6a", "701.9a");
    // "When this creature enters, each opponent discards a card."
    supported("Liliana's Specter");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.enter(P0, "Liliana's Specter");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn tithebearer_giant_draws_and_loses_life() {
    cr!("603.6a", "121.1", "119.3");
    // "When this creature enters, you draw a card and you lose 1 life."
    supported("Tithebearer Giant");
    let mut t = TestGame::new(2);
    t.enter(P0, "Tithebearer Giant");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn merrow_harbinger_tutors_a_merfolk_to_the_top() {
    cr!("603.6a", "701.23a");
    // "When this creature enters, you may search your library for a Merfolk card, reveal
    // it, then shuffle and put that card on top."
    supported("Merrow Harbinger");
    let mut t = TestGame::new(2);
    let merfolk = t.library_top(P0, "Silvergill Adept");
    t.g.players[P0.idx()].library.rotate_left(1);
    assert_ne!(t.g.players[P0.idx()].library.last(), Some(&merfolk));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(merfolk)]);
    t.enter(P0, "Merrow Harbinger");
    t.resolve_all();
    let top = *t.g.players[P0.idx()].library.last().unwrap();
    assert_eq!(t.obj_now(top).chars.name, "Silvergill Adept");
}

#[test]
fn vizier_of_the_scorpion_amasses_zombies_1() {
    cr!("603.6a", "701.47a");
    // "When this creature enters, amass Zombies 1."
    supported("Vizier of the Scorpion");
    let mut t = TestGame::new(2);
    t.enter(P0, "Vizier of the Scorpion");
    t.resolve_all();
    let army = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| t.obj_now(*id).chars.has_subtype("Army"))
        .expect("an Army token");
    assert!(t.obj_now(army).chars.has_subtype("Zombie"));
    assert_eq!(t.pt(army), (1, 1));
}

#[test]
fn doomed_dissenter_makes_a_zombie_when_it_dies() {
    cr!("603.6c", "700.4");
    // "When this creature dies, create a 2/2 black Zombie creature token."
    supported("Doomed Dissenter");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Doomed Dissenter");
    t.g.destroy(d, None);
    t.resolve_all();
    let z = subtype_ids(&t, P0, "Zombie");
    assert_eq!(z.len(), 1);
    let z = z[0];
    assert_eq!(t.pt(z), (2, 2));
}

#[test]
fn tibalts_rager_deals_1_damage_when_it_dies() {
    cr!("603.6c", "603.10a");
    // "When this creature dies, it deals 1 damage to any target."
    supported("Tibalt's Rager");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Tibalt's Rager");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.destroy(r, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn lifecreed_duo_gains_life_when_another_creature_you_control_enters() {
    cr!("603.6a", "119.3");
    // "Whenever another creature you control enters, you gain 1 life."
    supported("Lifecreed Duo");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lifecreed Duo");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Not for an opponent's creature.
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn youthful_valkyrie_grows_when_another_angel_enters() {
    cr!("603.6a", "122.6");
    // "Whenever another Angel you control enters, put a +1/+1 counter on this creature."
    supported("Youthful Valkyrie");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Youthful Valkyrie");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(v, "+1/+1"), 0);
    t.enter(P0, "Serra Angel");
    t.resolve_all();
    assert_eq!(t.counters(v, "+1/+1"), 1);
}

#[test]
fn sea_dasher_octopus_draws_on_combat_damage_to_a_player() {
    cr!("603.2", "510.3a");
    // "Whenever this creature deals combat damage to a player, draw a card."
    supported("Sea-Dasher Octopus");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Sea-Dasher Octopus");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(o, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn chilling_apparition_makes_the_damaged_player_discard() {
    cr!("603.2", "510.3a", "701.9a");
    // "Whenever this creature deals combat damage to a player, that player discards a card."
    supported("Chilling Apparition");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Chilling Apparition");
    t.hand(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
