//! "[Effect] unless [a player] [does something]" (CR 118.12a): the player named may
//! perform the action (a cost paid as the spell or ability resolves, CR 118.12); if they
//! don't, the effect happens. Punishers whose player chooses among several actions
//! ("unless they sacrifice a nonland permanent of their choice or discard a card"),
//! actions other than paying mana ("has ~ deal 6 damage to them", "discards their hand",
//! "exiles all cards from their graveyard", "returns a land they control"), payments with
//! amounts ("life equal to its toughness", "mana equal to [value]", "{1} or 1 life",
//! "{1} and 1 life"), and "any player" payments (each player in turn may pay).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn yes_no_askers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect()
}

// --- Several actions to choose among ------------------------------------------------

/// Torment of Scarabs on P1, with a card in hand and a nonland permanent.
fn scarabs_game() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Torment of Scarabs");
    t.g.attach(curse, Entity::Player(P1));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P1, "Grizzly Bears");
    t.set_step(P0, Step::End);
    (t, bears, card)
}

#[test]
fn torment_of_scarabs_player_chooses_sacrifice_discard_or_life() {
    cr!("118.12a", "118.12");
    ruling!(
        "Torment of Scarabs",
        "That player can always choose to lose 3 life, even if they have cards to discard or nonland permanents to sacrifice."
    );
    assert_supported(&["Torment of Scarabs"]);
    // Sacrifices the creature (the first action offered).
    let (mut t, bears, _) = scarabs_game();
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 1);
    // Declines to sacrifice, discards the card.
    let (mut t, bears, card) = scarabs_game();
    t.answer_yes(P1, false);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(card)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
    // Does neither and loses 3 life, though they could have done either.
    let (mut t, bears, _) = scarabs_game();
    t.answer_yes(P1, false);
    t.answer_yes(P1, false);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn torment_of_scarabs_lands_cant_be_sacrificed_and_an_empty_hand_cant_discard() {
    cr!("118.12a", "118.3");
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Torment of Scarabs");
    t.g.attach(curse, Entity::Player(P1));
    let forest = t.battlefield(P1, "Forest");
    t.set_step(P0, Step::End);
    // Only lands and no cards in hand: neither action is offered.
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    let from = t.asked().len();
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(yes_no_askers(&t, from).is_empty());
    assert!(t.on_battlefield(forest));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn nicol_bolas_each_opponent_chooses_in_turn_order() {
    cr!("118.12a", "101.4");
    ruling!(
        "Nicol Bolas, the Deceiver",
        "each opponent in turn order makes their choice for Nicol Bolas's first ability, then all of the actions occur simultaneously"
    );
    assert_supported(&["Nicol Bolas, the Deceiver"]);
    let mut t = TestGame::new(3);
    let bolas = t.battlefield(P0, "Nicol Bolas, the Deceiver");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    let p2_card = t.hand(P2, "Grizzly Bears");
    // P1 sacrifices the Bears; P2 declines to sacrifice (it has nothing to) and discards.
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(p1_bears)]);
    t.answer_yes(P2, true);
    t.answer_choose(P2, &[Entity::Object(p2_card)]);
    let from = t.asked().len();
    t.activate(P0, bolas, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(p1_bears));
    assert!(t.in_graveyard(P2, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 20);
    // P1 decided before P2.
    let askers = yes_no_askers(&t, from);
    assert_eq!(askers.first(), Some(&P1));
    assert!(askers.contains(&P2));
}

#[test]
fn indulgent_tormentor_opponent_cant_make_an_impossible_choice() {
    cr!("118.12a", "118.3");
    ruling!(
        "Indulgent Tormentor",
        "That player can’t make an impossible choice, such as sacrificing a creature while they control no creatures."
    );
    assert_supported(&["Indulgent Tormentor"]);
    // No creatures: only paying 3 life is offered; P1 declines and P0 draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Indulgent Tormentor");
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, false);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(yes_no_askers(&t, from), vec![P1]);
    assert_eq!(t.hand_size(P0), hand + 1);
    // With a creature, P1 sacrifices it and P0 draws nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Indulgent Tormentor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn erosion_pays_mana_or_life() {
    cr!("118.12a");
    assert_supported(&["Erosion"]);
    let setup = || {
        let mut t = TestGame::new(2);
        let erosion = t.battlefield(P0, "Erosion");
        let land = t.battlefield(P1, "Forest");
        t.g.attach(erosion, Entity::Object(land));
        t.set_step(P0, Step::End);
        (t, land)
    };
    // Declines {1} and pays 1 life instead: the land stays.
    let (mut t, land) = setup();
    t.answer_yes(P1, false);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(land));
    assert_eq!(t.life(P1), 19);
    // Pays neither: the land is destroyed.
    let (mut t, land) = setup();
    t.answer_yes(P1, false);
    t.answer_yes(P1, false);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(land));
    assert_eq!(t.life(P1), 20);
}

// --- Actions other than paying -------------------------------------------------------

#[test]
fn mogis_deals_damage_if_the_player_cant_sacrifice() {
    cr!("118.12a", "118.3");
    ruling!(
        "Mogis, God of Slaughter",
        "If the player can't sacrifice a creature (usually because they don't control one), Mogis will deal 2 damage to them."
    );
    assert_supported(&["Mogis, God of Slaughter"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mogis, God of Slaughter");
    t.set_step(P0, Step::End);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // With a creature, P1 may sacrifice it instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mogis, God of Slaughter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::End);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn lava_blister_controller_may_take_six_damage() {
    cr!("118.12a");
    assert_supported(&["Lava Blister"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let land = t.battlefield(P1, "Wasteland");
    let blister = t.hand(P0, "Lava Blister");
    t.answer_yes(P1, true);
    t.cast(P0, blister).target(land).go();
    t.resolve_all();
    assert!(t.on_battlefield(land));
    assert_eq!(t.life(P1), 14);
    // Declined: the land is destroyed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let land = t.battlefield(P1, "Wasteland");
    let blister = t.hand(P0, "Lava Blister");
    t.answer_yes(P1, false);
    t.cast(P0, blister).target(land).go();
    t.resolve_all();
    assert!(!t.on_battlefield(land));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn combustion_man_deals_damage_equal_to_his_own_power() {
    cr!("118.12a");
    assert_supported(&["Combustion Man"]);
    let mut t = TestGame::new(2);
    let man = t.battlefield(P0, "Combustion Man");
    let wall = t.battlefield(P1, "Wall of Stone");
    let (power, _) = t.pt(man);
    t.answer_targets(P0, &[Entity::Object(wall)]);
    t.answer_yes(P1, true);
    t.attack(&[(man, Entity::Player(P1))], &[]);
    assert!(t.on_battlefield(wall));
    // Dealt damage equal to Combustion Man's power (not the Wall's, 0), then combat damage.
    assert_eq!(t.life(P1), 20 - power - power);
}

#[test]
fn perplex_can_be_stopped_by_discarding_an_empty_hand() {
    cr!("118.12a");
    ruling!(
        "Perplex",
        "If the spell’s controller has no cards in their hand, that player can still choose to discard their hand and prevent the spell from being countered."
    );
    assert_supported(&["Perplex"]);
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let perplex = t.hand(P0, "Perplex");
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert_eq!(t.hand_size(P1), 0);
    t.answer_yes(P1, true);
    t.cast(P0, perplex).target(bolt).go();
    t.resolve_all();
    // Not countered: the Bolt resolved.
    assert_eq!(t.life(P0), 17);
}

#[test]
fn grip_of_amnesia_exile_graveyard_or_countered() {
    cr!("118.12a");
    assert_supported(&["Grip of Amnesia"]);
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Island", 2);
    t.graveyard(P1, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    let grip = t.hand(P0, "Grip of Amnesia");
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.answer_yes(P1, true);
    t.cast(P0, grip).target(bolt).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.graveyard_size(P1), 1, "only the resolved Bolt");
}

#[test]
fn quickling_must_be_sacrificed_without_another_creature() {
    cr!("118.12a", "118.3");
    ruling!(
        "Quickling",
        "If you control no other creatures when the enters-the-battlefield ability resolves, you must sacrifice Quickling."
    );
    assert_supported(&["Quickling"]);
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let q = t.enter(P0, "Quickling");
    t.resolve_all();
    assert!(!t.on_battlefield(q));
    // With another creature, it may be returned instead.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let q = t.enter(P0, "Quickling");
    t.resolve_all();
    assert!(t.on_battlefield(q));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn harvest_wurm_returns_a_basic_land_card_or_is_sacrificed() {
    cr!("118.12a", "118.3");
    ruling!(
        "Harvest Wurm",
        "If you can’t perform the other action, then you must sacrifice the creature."
    );
    assert_supported(&["Harvest Wurm"]);
    // Only a nonbasic land in the graveyard: sacrificed.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Wasteland");
    t.answer_yes(P0, true);
    let w = t.enter(P0, "Harvest Wurm");
    t.resolve_all();
    assert!(!t.on_battlefield(w));
    // A basic land card: returned to hand, the Wurm stays.
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    let w = t.enter(P0, "Harvest Wurm");
    t.resolve_all();
    assert!(t.on_battlefield(w));
    assert!(t.in_hand(P0, "Forest"));
}

#[test]
fn junk_golem_removes_a_counter_from_itself_or_is_sacrificed() {
    cr!("118.12a");
    assert_supported(&["Junk Golem", "Magmatic Sprinter"]);
    let mut t = TestGame::new(2);
    let golem = t.enter(P0, "Junk Golem");
    t.resolve_all();
    assert_eq!(t.counters(golem, "+1/+1"), 3);
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(golem));
    assert_eq!(t.counters(golem, "+1/+1"), 2);
    // Without counters it's sacrificed.
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Junk Golem");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(golem));
}

#[test]
fn unnatural_hunger_another_creature_isnt_the_enchanted_one() {
    cr!("118.12a");
    assert_supported(&["Unnatural Hunger"]);
    let mut t = TestGame::new(2);
    let aura = t.battlefield(P0, "Unnatural Hunger");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.g.attach(aura, Entity::Object(ogre));
    t.set_step(P0, Step::End);
    // P1's only creature is the enchanted one: it can't be sacrificed for this.
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(ogre)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(ogre));
    // Damage equal to the enchanted creature's power (2) to its controller.
    assert_eq!(t.life(P1), 18);
    // Another creature: P1 sacrifices it and takes no damage.
    let mut t = TestGame::new(2);
    let aura = t.battlefield(P0, "Unnatural Hunger");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.attach(aura, Entity::Object(ogre));
    t.set_step(P0, Step::End);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(ogre));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn curse_artifact_only_its_controller_can_sacrifice_that_artifact() {
    cr!("118.12a", "118.3", "701.21a");
    assert_supported(&["Curse Artifact"]);
    let setup = || {
        let mut t = TestGame::new(2);
        let curse = t.battlefield(P0, "Curse Artifact");
        let ring = t.battlefield(P1, "Sol Ring");
        t.g.attach(curse, Entity::Object(ring));
        t.set_step(P0, Step::End);
        (t, ring)
    };
    // P1 sacrifices the enchanted artifact: no damage.
    let (mut t, ring) = setup();
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert_eq!(t.life(P1), 20);
    // P0 gains control of it with the ability on the stack: P1 can't sacrifice a
    // permanent it doesn't control, so it isn't offered and P1 takes 2 damage.
    let (mut t, ring) = setup();
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let o = &mut t.g.objects[ring.0 as usize];
    o.base_controller = P0;
    o.controller = P0;
    t.g.recompute();
    t.answer_yes(P1, true);
    let from = t.asked().len();
    t.resolve_all();
    assert!(yes_no_askers(&t, from).is_empty());
    assert!(t.on_battlefield(ring));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn drake_familiar_may_return_an_opponents_enchantment() {
    cr!("118.12a", "118.3");
    ruling!(
        "Drake Familiar",
        "The ability lets you return any enchantment on the battlefield, including an opponent’s enchantment."
    );
    assert_supported(&["Drake Familiar"]);
    // P1's Glorious Anthem, which has shroud (the action isn't targeted): returned to
    // P1's hand, and the Drake stays.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sterling Grove");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(anthem)]);
    let d = t.enter(P0, "Drake Familiar");
    t.resolve_all();
    assert!(t.on_battlefield(d));
    assert!(t.in_hand(P1, "Glorious Anthem"));
    // No enchantment on the battlefield (one in hand doesn't count): sacrificed.
    let mut t = TestGame::new(2);
    t.hand(P0, "Glorious Anthem");
    t.answer_yes(P0, true);
    let d = t.enter(P0, "Drake Familiar");
    t.resolve_all();
    assert!(!t.on_battlefield(d));
    assert!(t.in_hand(P0, "Glorious Anthem"));
}

// --- Payments with amounts -------------------------------------------------------------

#[test]
fn essence_vortex_pays_life_equal_to_toughness_or_no_regeneration() {
    cr!("118.12a", "119.4", "701.19c");
    assert_supported(&["Essence Vortex"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 1);
    let wall = t.battlefield(P1, "Wall of Stone");
    let vortex = t.hand(P0, "Essence Vortex");
    t.answer_yes(P1, true);
    t.cast(P0, vortex).target(wall).go();
    t.resolve_all();
    assert!(t.on_battlefield(wall));
    assert_eq!(t.life(P1), 12);
    // With less life than its toughness, P1 can't pay (CR 119.4): destroyed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 1);
    let wall = t.battlefield(P1, "Wall of Stone");
    t.g.player_mut(P1).life = 7;
    let vortex = t.hand(P0, "Essence Vortex");
    t.answer_yes(P1, true);
    t.cast(P0, vortex).target(wall).go();
    t.resolve_all();
    assert!(!t.on_battlefield(wall));
    assert_eq!(t.life(P1), 7);
    // Not paid: destroyed, and it can't be regenerated.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 1);
    let boa = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve();
    let vortex = t.hand(P0, "Essence Vortex");
    t.answer_yes(P1, false);
    t.cast(P0, vortex).target(boa).go();
    t.resolve_all();
    assert!(!t.on_battlefield(boa));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn repulsive_mutation_zero_mana_may_still_be_declined() {
    cr!("118.12a", "118.3a");
    ruling!(
        "Repulsive Mutation",
        "That player can choose not to pay 0 mana; if they do, the spell will be countered."
    );
    assert_supported(&["Repulsive Mutation"]);
    // P0's only creature is gone when Repulsive Mutation resolves: the amount is 0. P1 is
    // still asked, and declining counters the Bolt; paying 0 lets it resolve.
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P1, "Mountain", 1);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Forest", 1);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let bolt = t.hand(P1, "Lightning Bolt");
        let mutation = t.hand(P0, "Repulsive Mutation");
        let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
        t.cast(P0, mutation).x(0).target(bears).target(bolt).go();
        t.g.destroy(bears, None);
        t.answer_yes(P1, pays);
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(yes_no_askers(&t, from), vec![P1], "P1 chooses whether to pay 0");
        assert_eq!(t.life(P0), if pays { 17 } else { 20 }, "paid: {pays}");
    }
    // The Bears still there: {2}, which P1 (no mana left) can't pay.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Forest", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    let mutation = t.hand(P0, "Repulsive Mutation");
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.answer_yes(P1, true);
    t.cast(P0, mutation).x(0).target(bears).target(bolt).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "the Bolt was countered: P1 couldn't pay {{2}}");
}

#[test]
fn mundungu_needs_both_mana_and_life() {
    cr!("118.12a", "118.3");
    assert_supported(&["Mundungu"]);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mundungu");
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.answer_yes(P1, true);
    t.activate(P0, m, 0, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    // P1 paid {1} and 1 life: the Bolt resolved.
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 17);
    // No mana left after the Bolt: the whole cost can't be paid (CR 118.3), countered.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mundungu");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.answer_yes(P1, true);
    t.activate(P0, m, 0, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "no life paid");
    assert_eq!(t.life(P0), 20, "countered");
}

// --- Any player may pay ------------------------------------------------------------------

#[test]
fn soul_strings_each_player_gets_the_option_to_pay() {
    cr!("118.12a", "107.3");
    ruling!("Soul Strings", "Each player gets the option to pay when this spell resolves.");
    assert_supported(&["Soul Strings", "Rhystic Tutor", "Nakaya Shade"]);
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 5);
    t.lands(P2, "Swamp", 2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Gray Ogre");
    let strings = t.hand(P0, "Soul Strings");
    // X = 2: P0 and P1 decline (P1 can't pay), P2 pays {2}.
    t.answer_yes(P0, false);
    t.answer_yes(P2, true);
    let from = t.asked().len();
    t.cast(P0, strings)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    let askers = yes_no_askers(&t, from);
    assert!(askers.contains(&P0) && askers.contains(&P2));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Gray Ogre"));
    // Nobody pays: both cards return.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Gray Ogre");
    let strings = t.hand(P0, "Soul Strings");
    t.answer_yes(P0, false);
    t.answer_yes(P1, false);
    t.cast(P0, strings)
        .x(1)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Gray Ogre"));
}

#[test]
fn nakaya_shade_any_player_may_pay_to_stop_the_pump() {
    cr!("118.12a");
    ruling!("Nakaya Shade", "Each player gets the option to pay when the ability resolves.");
    let mut t = TestGame::new(2);
    let shade = t.battlefield(P0, "Nakaya Shade");
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Island", 2);
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.activate(P0, shade, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(shade), (1, 1));
    t.answer_yes(P0, false);
    t.answer_yes(P1, false);
    t.lands(P0, "Swamp", 1);
    t.activate(P0, shade, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(shade), (2, 2));
}

#[test]
fn pias_revolution_target_opponent_chooses_damage() {
    cr!("118.12a");
    ruling!(
        "Pia's Revolution",
        "The target opponent chooses whether to have Pia’s Revolution deal 3 damage to them as its ability resolves."
    );
    assert_supported(&["Pia's Revolution"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pia's Revolution");
    let ring = t.battlefield(P0, "Sol Ring");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, true);
    t.g.destroy(ring, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Sol Ring"));
    let ring = t.battlefield(P0, "Sol Ring");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, false);
    t.g.destroy(ring, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P0, "Sol Ring"));
}
