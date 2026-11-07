//! The worked examples of CR 603 (handling triggered abilities), played out as tests. The
//! second CR 603.7a example and the CR 603.7c example are cited by the tests in
//! `r600_delayed_triggers.rs`.

use crate::r600_common::{act, mana_cost, CB};
use crate::r600_examples_common::*;
use mtg_engine::ability::*;
use mtg_engine::game::GameResult;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_land_destruction_spell_triggers_once_for_each_land_put_into_a_graveyard() {
    cr!("603.2c");
    cr_example!("603.2c", "destroys all lands");
    supported("Dingus Egg");
    supported("Armageddon");
    let mut t = TestGame::new(2);
    // Dingus Egg: "Whenever a land is put into a graveyard from the battlefield, this
    // artifact deals 2 damage to that land's controller."
    t.battlefield(P0, "Dingus Egg");
    t.lands(P0, "Plains", 4);
    t.lands(P1, "Island", 3);
    let geddon = t.hand(P0, "Armageddon");
    t.cast(P0, geddon).go();
    t.resolve();
    // Seven lands went to the graveyard during one resolution: seven triggers.
    assert_eq!(t.stack_len(), 7);
    t.resolve_all();
    assert_eq!(t.life(P0), 20 - 4 * 2);
    assert_eq!(t.life(P1), 20 - 3 * 2);
}

#[test]
fn becomes_tapped_triggers_only_when_a_permanent_on_the_battlefield_becomes_tapped() {
    cr!("603.2e");
    cr_example!("603.2e", "becomes tapped");
    supported("Selesnya Guildgate");
    let mut t = TestGame::new(2);
    let watcher = custom_card(
        "Tap Watcher",
        "Enchantment",
        None,
        "Whenever a land becomes tapped, you gain 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    // Selesnya Guildgate enters the battlefield tapped: it never changed from untapped
    // to tapped.
    let gate = t.hand(P0, "Selesnya Guildgate");
    t.play_land(P0, gate).unwrap();
    t.settle();
    let gate = t.g.current(gate);
    assert!(t.obj(gate).tapped);
    assert_eq!(t.stack_len(), 0);
    // Untapped and then tapped on the battlefield: that's the trigger event.
    t.g.untap(gate);
    t.g.tap(gate);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn a_damage_trigger_doesnt_trigger_if_all_the_damage_is_prevented() {
    cr!("603.2g", "615.1");
    cr_example!("603.2g", "all the damage is prevented");
    let mut t = TestGame::new(2);
    let hurt = || {
        custom_card(
            "Hurt Watcher",
            "Creature — Wall",
            Some((0, 4)),
            "Whenever this creature is dealt damage, you gain 1 life.",
        )
    };
    let shielded = t.custom(P0, hurt(), Zone::Battlefield);
    let open = t.custom(P0, hurt(), Zone::Battlefield);
    let shield = custom_costed(
        "Shield",
        "Instant",
        "{0}",
        "Prevent all damage that would be dealt to target creature this turn.",
    );
    let s = t.custom(P0, shield, Zone::Hand(P0));
    t.cast(P0, s).target(shielded).go();
    t.resolve();
    t.lands(P1, "Mountain", 2);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(shielded).go();
    t.resolve_all();
    assert_eq!(t.obj(shielded).damage, 0);
    assert_eq!(t.life(P0), 20);
    // Damage that isn't prevented triggers it.
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(open).go();
    t.resolve_all();
    assert_eq!(t.obj(open).damage, 2);
    assert_eq!(t.life(P0), 21);
}

/// A game where P0 controls Felidar Sovereign with `life` life, at the start of P0's
/// upkeep (any trigger put on the stack).
fn sovereign_upkeep(life: i32) -> TestGame {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Felidar Sovereign");
    t.g.players[0].life = life;
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t
}

#[test]
fn felidar_sovereign_checks_life_as_the_upkeep_begins_and_again_on_resolution() {
    cr!("603.4", "608.2a");
    cr_example!("603.4", "Felidar Sovereign");
    supported("Felidar Sovereign");
    // 39 life: it doesn't trigger at all.
    let t = sovereign_upkeep(39);
    assert_eq!(t.stack_len(), 0);
    // 40 life: it triggers. At 39 life as it resolves, it's removed and does nothing.
    let mut t = sovereign_upkeep(40);
    assert_eq!(t.stack_len(), 1);
    t.g.players[0].life = 39;
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert!(t.result.is_none());
    // Still 40 as it resolves: that player wins the game.
    let mut t = sovereign_upkeep(40);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn a_land_played_while_all_lands_are_creatures_triggers_creature_enters_abilities() {
    cr!("603.6b", "603.6a");
    cr_example!("603.6b", "All lands are creatures");
    supported("Nature's Revolt");
    let mut t = TestGame::new(2);
    // Nature's Revolt: "All lands are 2/2 creatures that are still lands."
    t.battlefield(P0, "Nature's Revolt");
    let watcher = custom_card(
        "Creature Watcher",
        "Enchantment",
        None,
        "Whenever a creature enters, you gain 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert!(t.obj_now(forest).is_creature());
}

#[test]
fn a_creature_entering_while_all_creatures_lose_all_abilities_doesnt_trigger_its_etb() {
    cr!("603.6b", "603.6a");
    cr_example!("603.6b", "All creatures lose all abilities");
    supported("Elvish Visionary");
    let mut t = TestGame::new(2);
    let blank = custom_card(
        "Blank Slate",
        "Enchantment",
        None,
        "All creatures lose all abilities.",
    );
    t.custom(P0, blank, Zone::Battlefield);
    t.lands(P0, "Forest", 2);
    // Elvish Visionary: "When this creature enters, draw a card."
    let elf = t.hand(P0, "Elvish Visionary");
    let hand = t.hand_size(P0);
    t.cast(P0, elf).go();
    t.resolve();
    assert!(t.on_battlefield(elf));
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn a_leaves_the_battlefield_delayed_trigger_created_after_the_creature_left_never_triggers() {
    cr!("603.7a", "603.7");
    cr_example!("603.7a", "the delayed ability never triggers");
    // "{0}: When this creature leaves the battlefield, you gain 5 life." (the effect's
    // delayed triggered ability).
    let def = || {
        CB::new("Farewell Bear")
            .creature(2, 2)
            .ability(act(
                mana_cost("{0}"),
                Body::effect(Effect::DelayedTrigger {
                    trigger: TriggerCond::LeavesBattlefield(Filter::Source),
                    body: Box::new(Body::effect(crate::r600_common::gain(5))),
                    once: true,
                }),
            ))
            .build()
    };
    // The creature leaves before the ability creating the effect resolves.
    let mut t = TestGame::new(2);
    let bear = t.custom(P0, def(), Zone::Battlefield);
    t.activate(P0, bear, 0, &[]).unwrap();
    t.g.destroy(bear, None);
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert_eq!(t.life(P0), 20);
    // Even if the card comes back as a new object which then leaves.
    let gy = t.g.current(bear);
    t.g.move_object(
        gy,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    let back = t.g.current(gy);
    t.g.destroy(back, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Created first, it triggers when the creature leaves.
    let mut t = TestGame::new(2);
    let bear = t.custom(P0, def(), Zone::Battlefield);
    t.activate(P0, bear, 0, &[]).unwrap();
    t.resolve();
    t.g.destroy(bear, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn whenever_you_have_no_cards_in_hand_triggers_once_and_during_a_hand_wheel() {
    cr!("603.8");
    cr_example!("603.8", "Whenever you have no cards in hand, draw a card");
    supported("Tolarian Winds");
    let mut t = TestGame::new(2);
    let muse = custom_card(
        "Empty Hand Muse",
        "Enchantment",
        None,
        "Whenever you have no cards in hand, draw a card.",
    );
    t.custom(P0, muse, Zone::Battlefield);
    // The player plays the last card from their hand.
    let forest = t.hand(P0, "Forest");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.play_land(P0, forest).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // It won't trigger again until it has left the stack.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.stack_len(), 0);

    // Tolarian Winds: "Discard all the cards in your hand, then draw that many cards." The
    // hand is momentarily empty during its resolution.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        custom_card(
            "Empty Hand Muse",
            "Enchantment",
            None,
            "Whenever you have no cards in hand, draw a card.",
        ),
        Zone::Battlefield,
    );
    t.lands(P0, "Island", 2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    let winds = t.hand(P0, "Tolarian Winds");
    t.cast(P0, winds).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn a_dies_trigger_on_an_artifact_destroyed_with_the_creatures_triggers_for_each() {
    cr!("603.10a", "603.10");
    cr_example!("603.10a", "The artifact’s ability triggers twice");
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    let relic = custom_card(
        "Mourning Relic",
        "Artifact",
        None,
        "Whenever a creature dies, you gain 1 life.",
    );
    let relic = t.custom(P0, relic, Zone::Battlefield);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 6);
    // Akroma's Vengeance: "Destroy all artifacts, creatures, and enchantments."
    let wrath = t.hand(P0, "Akroma's Vengeance");
    t.cast(P0, wrath).go();
    t.resolve();
    assert!(!t.on_battlefield(relic));
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn rowen_reveals_the_first_draw_and_its_linked_trigger_draws_for_a_basic_land() {
    cr!("603.11", "607.2h");
    cr_example!("603.11", "Whenever you reveal a basic land card this way");
    supported("Rowen");
    // One paragraph: a static ability followed by the triggered ability linked to it.
    let abs = &mtg_engine::card::card("Rowen").faces[0].chars.abilities;
    assert_eq!(abs.len(), 2);
    assert!(matches!(abs[0].kind, AbilityKind::Static(_)));
    assert!(matches!(abs[1].kind, AbilityKind::Triggered(_)));
    assert_ne!(abs[0].link, 0);
    assert_eq!(abs[0].link, abs[1].link);

    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rowen");
    let extra = t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // The revealed basic land drew another card (only the first draw is revealed).
    assert_eq!(t.zone(extra), Zone::Hand(P0));
    assert_eq!(t.stack_len(), 0);
    // A later basic land drawn this turn isn't revealed by the static ability.
    t.library_top(P0, "Island");
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn heart_piercer_manticores_reflexive_trigger_needs_its_own_sacrifice() {
    cr!("603.12");
    cr_example!("603.12", "Heart-Piercer Manticore");
    supported("Heart-Piercer Manticore");
    supported("Altar's Reap");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let manticore = t.hand(P0, "Heart-Piercer Manticore");
    t.cast(P0, manticore).go();
    t.resolve();
    // The enters trigger: sacrifice Hill Giant (power 3); "When you do" deals 3 damage.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 17);
    // Sacrificing a creature for another reason doesn't trigger it.
    t.lands(P0, "Swamp", 2);
    let reap = t.hand(P0, "Altar's Reap");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, reap).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
