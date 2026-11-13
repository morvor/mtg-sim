//! Rulings batch P177 — putting other players' creatures onto the battlefield under your
//! control: Minion's Return, Unholy Indenture, Unhallowed Pact, Beacon of Unrest,
//! Necroskitter, Athreos, Come Back Wrong, Sorin, The Darkness Crystal, Endless Whispers,
//! Bribery and Lord of the Void — and what happens when their controller leaves the game.

use crate::r_p177_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 controls the Aura `aura` on `p`'s Grizzly Bears, which then dies. Returns the Bears'
/// card id (follow it with `current`).
fn aura_then_die(t: &mut TestGame, aura: &str, p: PlayerId) -> ObjectId {
    supported(aura);
    let bears = t.battlefield(p, "Grizzly Bears");
    attach_new(t, P0, aura, bears);
    destroy(t, bears);
    t.resolve_all();
    bears
}

#[test]
fn a_creature_returned_by_an_aura_is_exiled_when_its_controller_leaves() {
    cr!("800.4a");
    ruling!(
        "Minion's Return",
        "If you leave the game, the creature you control from Minion’s Return is exiled if you don’t own it."
    );
    ruling!(
        "Unholy Indenture",
        "If you leave the game, the creature you control from Unholy Indenture’s effect is exiled."
    );
    for aura in ["Minion's Return", "Unholy Indenture"] {
        let mut t = ffa(3);
        let bears = aura_then_die(&mut t, aura, P1);
        assert!(t.on_battlefield(bears), "{aura}");
        assert_eq!(controller(&mut t, bears), P0, "{aura}");
        concede(&mut t, P0);
        assert_eq!(t.zone(bears), Zone::Exile, "{aura}");
    }
    // A creature P0 owns leaves the game with P0 instead.
    let mut t = ffa(3);
    let bears = aura_then_die(&mut t, "Minion's Return", P0);
    assert!(t.on_battlefield(bears));
    concede(&mut t, P0);
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn beacon_of_unrests_permanent_is_exiled_when_its_controller_leaves() {
    cr!("800.4a");
    ruling!(
        "Beacon of Unrest",
        "If you leave the game, the permanent you control from Beacon of Unrest is exiled."
    );
    let mut t = ffa(3);
    let bears = t.graveyard(P2, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Beacon of Unrest", &[obj(bears)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P0);
    concede(&mut t, P0);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn unholy_indenture_returns_a_new_object_under_your_control() {
    cr!("400.7", "303.4c", "301.5c");
    ruling!(
        "Unholy Indenture",
        "Once Unholy Indenture returns the creature, it’s considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Unholy Indenture",
        "You control the returned creature, regardless of who controlled it before it died."
    );
    supported("Unholy Indenture");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let indenture = attach_new(&mut t, P0, "Unholy Indenture", bears);
    let strength = attach_new(&mut t, P1, "Holy Strength", bears);
    let splitter = attach_new(&mut t, P1, "Bonesplitter", bears);
    crate::r_s13_common::add(&mut t, obj(bears), "+1/+1", 2);
    destroy(&mut t, bears);
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_eq!(controller(&mut t, back), P0);
    // Only Indenture's own counter; no Auras came back (including Indenture); the
    // Equipment is unattached.
    assert_eq!(t.counters(back, "+1/+1"), 1);
    assert!(!t.on_battlefield(indenture));
    assert!(!t.on_battlefield(strength));
    assert!(t.on_battlefield(splitter));
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(back), (3, 3));
}

#[test]
fn unhallowed_pact_does_nothing_if_the_card_already_left_the_graveyard() {
    cr!("603.3b", "608.2b", "400.7");
    ruling!(
        "Unhallowed Pact",
        "The card will return to the battlefield under your control only if it’s still in the graveyard when Unhallowed Pact’s ability resolves."
    );
    supported("Unhallowed Pact");
    // P1's Strangleroot Geist (undying) dies on P0's turn with P0's Pact on it: P0's
    // trigger goes on the stack first, P1's undying trigger resolves first.
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P1, "Strangleroot Geist");
    attach_new(&mut t, P0, "Unhallowed Pact", geist);
    destroy(&mut t, geist);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let back = t.g.current(geist);
    assert!(t.on_battlefield(back));
    assert_eq!(controller(&mut t, back), P1);
    assert_eq!(t.counters(back, "+1/+1"), 1);
}

#[test]
fn necroskitter_returns_any_creature_an_opponent_controlled_with_a_minus_counter() {
    cr!("603.10a", "122.1");
    ruling!(
        "Necroskitter",
        "It doesn't matter how many -1/-1 counters were on the creature when it was put into a graveyard from the battlefield, as long as it had at least one."
    );
    ruling!(
        "Necroskitter",
        "The card returns to the battlefield without any -1/-1 counters on it"
    );
    supported("Necroskitter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necroskitter");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s13_common::add(&mut t, obj(giant), "-1/-1", 2);
    assert_eq!(t.pt(giant), (1, 1));
    t.answer_yes(P0, true);
    destroy(&mut t, giant);
    t.resolve_all();
    let back = t.g.current(giant);
    assert!(t.on_battlefield(back));
    assert_eq!(controller(&mut t, back), P0);
    assert_eq!(t.counters(back, "-1/-1"), 0);
    assert_eq!(t.pt(back), (3, 3));
}

#[test]
fn necroskitter_only_cares_who_controlled_the_creature_not_whose_graveyard() {
    cr!("603.10a");
    ruling!(
        "Necroskitter",
        "It doesn't matter whose graveyard the creature is put into. It matters only that an opponent controlled it when it left the battlefield."
    );
    // P0 owns the Bears, P1 controls it: it goes to P0's graveyard and returns.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necroskitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    crate::r_s13_common::add(&mut t, obj(bears), "-1/-1", 1);
    t.answer_yes(P0, true);
    destroy(&mut t, bears);
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_eq!(controller(&mut t, back), P0);
    // P1 owns the Hill Giant, P0 controls it: no trigger.
    let giant = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, giant, P0);
    crate::r_s13_common::add(&mut t, obj(giant), "-1/-1", 1);
    destroy(&mut t, giant);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn athreos_returns_the_creature_without_its_coin_counter() {
    cr!("400.7", "122.2");
    ruling!(
        "Athreos, Shroud-Veiled",
        "The creature returns to the battlefield without a coin counter on it."
    );
    supported("Athreos, Shroud-Veiled");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Athreos, Shroud-Veiled");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s13_common::add(&mut t, obj(bears), "coin", 1);
    destroy(&mut t, bears);
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_eq!(controller(&mut t, back), P0);
    assert_eq!(t.counters(back, "coin"), 0);
}

#[test]
fn with_two_athreos_the_last_player_in_apnap_order_gets_the_creature() {
    cr!("101.4", "603.3b", "400.7");
    ruling!(
        "Athreos, Shroud-Veiled",
        "if both players control an Athreos when a creature with a coin counter on it dies, the nonactive player (the one whose turn it isn't) will return the creature under their control."
    );
    // Two players: P0's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Athreos, Shroud-Veiled");
    t.battlefield(P1, "Athreos, Shroud-Veiled");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s13_common::add(&mut t, obj(bears), "coin", 1);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P1);
    // Three players, P0's turn; P1 and P2 control an Athreos: the last in turn order
    // after the active player (P2, to P0's right) gets it.
    let mut t = ffa(3);
    t.battlefield(P1, "Athreos, Shroud-Veiled");
    t.battlefield(P2, "Athreos, Shroud-Veiled");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s13_common::add(&mut t, obj(bears), "coin", 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P2);
}

#[test]
fn come_back_wrong_returns_the_creature_in_the_same_resolution() {
    cr!("608.2c");
    ruling!(
        "Come Back Wrong",
        "Players can't take actions in between the time you destroy the target creature and the time it returns to the battlefield under your control."
    );
    supported("Come Back Wrong");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_targeting(&mut t, P0, "Come Back Wrong", &[obj(bears)]);
    t.g.resolve_top();
    // Without anything else happening, it's back under P0's control.
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P0);
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn sorins_ultimate_can_target_a_token_but_it_doesnt_return() {
    cr!("111.7", "704.5d");
    ruling!(
        "Sorin, Lord of Innistrad",
        "You can target a token creature with Sorin, Lord of Innistrad's third ability, but it won't return to the battlefield."
    );
    supported("Sorin, Lord of Innistrad");
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin, Lord of Innistrad");
    set_loyalty(&mut t, sorin, 6);
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(token), obj(bears)]);
    t.activate(P0, sorin, 2, &[]).unwrap();
    t.resolve_all();
    assert!(!t.g.is_live(t.g.current(token)) || !t.on_battlefield(token));
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P0);
}

#[test]
fn the_darkness_crystal_stops_dies_triggers_for_opponents_creatures() {
    cr!("614.1a", "700.4");
    ruling!(
        "The Darkness Crystal",
        "While you control The Darkness Crystal, abilities that trigger whenever a nontoken creature an opponent control dies won't trigger"
    );
    supported("The Darkness Crystal");
    // Blood Artist: "Whenever this creature or another creature dies, target player loses
    // 1 life and you gain 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Darkness Crystal");
    t.battlefield(P1, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 0, "no trigger");
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn the_darkness_crystal_reduces_only_generic_mana() {
    cr!("601.2f", "202.3");
    ruling!(
        "The Darkness Crystal",
        "The cost reduction applies only to generic mana in the total cost of black spells you cast."
    );
    ruling!(
        "The Darkness Crystal",
        "The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Darkness Crystal");
    // Distress ({B}{B}) still costs {B}{B}.
    t.lands(P0, "Swamp", 1);
    let distress = t.hand(P0, "Distress");
    assert!(!crate::r_s21_common::castable(&mut t, P0, distress));
    // Doomfall ({2}{B}) costs {1}{B}; its mana value is still 3.
    t.lands(P0, "Swamp", 1);
    t.battlefield(P1, "Grizzly Bears");
    let doomfall = t.hand(P0, "Doomfall");
    let spell = t
        .cast(P0, doomfall)
        .modes(&[0])
        .target(Entity::Player(P1))
        .go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
    assert_eq!(t.obj_now(spell).chars.mana_value(), 3);
}

#[test]
fn endless_whispers_each_creatures_controller_chooses_the_opponent() {
    cr!("603.3d", "113.8");
    ruling!(
        "Endless Whispers",
        "The creatures have the triggered ability, so each creature’s controller chooses one of their opponents to target."
    );
    supported("Endless Whispers");
    let mut t = ffa(3);
    t.battlefield(P0, "Endless Whispers");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1 controls the trigger and chooses P2.
    t.answer_targets(P1, &[Entity::Player(P2)]);
    destroy(&mut t, bears);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P2);
}

#[test]
fn bribery_you_control_the_creatures_enters_abilities() {
    cr!("603.3a", "110.2");
    ruling!(
        "Bribery",
        "You put the creature onto the battlefield, so you control it and any \"enters\" abilities it has."
    );
    supported("Bribery");
    // Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let visionary = t.library_top(P1, "Elvish Visionary");
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.answer_choose(P0, &[obj(visionary)]);
    cast_resolve(&mut t, P0, "Bribery", &[Entity::Player(P1)]);
    assert!(t.on_battlefield(visionary));
    assert_eq!(controller(&mut t, visionary), P0);
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn lord_of_the_void_must_put_a_creature_onto_the_battlefield() {
    cr!("510.2", "609.3");
    ruling!(
        "Lord of the Void",
        "Lord of the Void’s ability isn’t optional."
    );
    supported("Lord of the Void");
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Lord of the Void");
    crate::r_s01_common::stack_library(&mut t, P1, &["Island", "Grizzly Bears"]);
    // P0 tries to choose nothing.
    t.answer_choose(P0, &[]);
    t.attack(&[(lord, Entity::Player(P1))], &[]);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(controller(&mut t, bears[0]), P0);
}
