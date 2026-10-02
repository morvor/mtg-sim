//! Rulings batch P226 — transforming permanents (CR 701.27, 712): effects keep applying
//! to a permanent that transforms (CR 712.18), "transform" instructions that are part of
//! a resolving ability (CR 608.2), intervening "if" clauses (CR 603.4), abilities that
//! already transformed their source (CR 701.27f), and planeswalkers that transform.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, create_token, destroy};
use crate::r_s04_common::untapped_lands;
use crate::r_s06_common::{activate_containing, damage};
use crate::r_s08_common::{is_tapped, mana_value};
use crate::r_s13_common::add;
use crate::r_s17_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const OKO: &str = "Oko, Lorwyn Liege // Oko, Shadowmoor Scion";
const SQUIRE: &str = "Bloodsworn Squire // Bloodsworn Knight";
const HIDETSUGU: &str = "Hidetsugu Consumes All // Vessel of the All-Consuming";
const CAPTIVE: &str = "Wolfbitten Captive // Krallenhorde Killer";
const WEDDING: &str = "Wedding Announcement // Wedding Festivity";
const DOCENT: &str = "Docent of Perfection // Final Iteration";
const THING: &str = "Thing in the Ice // Awoken Horror";
const TORMENTOR: &str = "Elusive Tormentor // Insidious Mist";
const CHALICE: &str = "Chalice of Life // Chalice of Death";
const SEIZER: &str = "Soul Seizer // Ghastly Haunting";

/// `p` casts Lightning Bolt at `target` (with a Mountain for it); it's left on the stack.
fn cast_bolt(t: &mut TestGame, p: PlayerId, target: impl Into<Entity>) -> ObjectId {
    t.lands(p, "Mountain", 1);
    let b = t.hand(p, "Lightning Bolt");
    t.cast(p, b).target(target).go()
}

#[test]
fn okos_creature_types_effect_outlasts_the_turn_and_oko_himself() {
    cr!("611.2a", "712.18");
    ruling!(
        "Oko, Lorwyn Liege // Oko, Shadowmoor Scion",
        "The effect of Oko, Lorwyn Liege's first loyalty ability lasts indefinitely. It doesn't expire during the cleanup step or when Oko leaves the battlefield or transforms."
    );
    // (Oko, Shadowmoor Scion's −1 doesn't compile; the +2 does.)
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, OKO);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!t.obj_now(bears).chars.has_subtype("Elf"));
    // "+2: Up to one target creature gains all creature types."
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, oko, "+2").unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).chars.has_subtype("Elf"));
    // Through the cleanup step, Oko transforming, and Oko leaving the battlefield.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).chars.has_subtype("Elf"));
    transform(&mut t, oko);
    assert_eq!(name_of(&t, oko), "Oko, Shadowmoor Scion");
    assert!(t.obj_now(bears).chars.has_subtype("Sliver"));
    destroy(&mut t, oko);
    assert!(!t.on_battlefield(oko));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.obj_now(bears).chars.has_subtype("Elf"));
    assert!(t.obj_now(bears).chars.has_subtype("Goat"));
}

#[test]
fn bloodsworn_knight_keeps_the_squires_indestructibility() {
    cr!("712.18", "611.2a");
    ruling!(
        "Bloodsworn Squire // Bloodsworn Knight",
        "The effect that makes Bloodsworn Squire indestructible until end of turn continues to apply after it transforms into Bloodsworn Knight."
    );
    supported(SQUIRE);
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, SQUIRE);
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let discard = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    // "{1}{B}, Discard a card: This creature gains indestructible until end of turn. Tap
    // it. Then if there are four or more creature cards in your graveyard, transform
    // this creature."
    t.answer_choose(P0, &[Entity::Object(discard)]);
    activate_containing(&mut t, P0, squire, "Discard").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, squire), "Bloodsworn Knight");
    assert!(t.obj_now(squire).has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.pt(squire), (4, 4));
    destroy(&mut t, squire);
    assert!(t.on_battlefield(squire));
    // The effect ends at end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(squire).has_keyword(KeywordKind::Indestructible));
}

#[test]
fn norns_inquisitor_triggers_in_either_direction_if_the_result_is_phyrexian() {
    cr!("701.27e", "603.2");
    ruling!(
        "Norn's Inquisitor",
        "The last ability of Norn's Inquisitor will trigger if a permanent you control transforms in either direction, going from front face up to back face up or vice versa, as long as it's a Phyrexian after doing so."
    );
    supported("Norn's Inquisitor");
    // "When this creature enters, incubate 2." / "Whenever a permanent you control
    // transforms into a Phyrexian, put a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    t.enter(P0, "Norn's Inquisitor");
    t.resolve_all();
    let incubator = with_subtype(&t, P0, "Incubator")[0];
    assert_eq!(t.counters(incubator, counters::PLUS1), 2);
    // Front to back: the Incubator becomes a Phyrexian artifact creature.
    transform(&mut t, incubator);
    assert!(t.obj_now(incubator).chars.has_subtype("Phyrexian"));
    t.resolve_all();
    assert_eq!(t.counters(incubator, counters::PLUS1), 3);
    // Back to front: it's an Incubator again, not a Phyrexian — no trigger.
    transform(&mut t, incubator);
    assert!(!t.obj_now(incubator).chars.has_subtype("Phyrexian"));
    assert_eq!(t.stack_len(), 0);
    // Back to front into a Phyrexian: The True Scriptures transforms into Sheoldred.
    let saga = enter_transformed(&mut t, P0, "Sheoldred // The True Scriptures");
    t.resolve_all();
    assert_eq!(name_of(&t, saga), "The True Scriptures");
    assert_eq!(face(&t, saga), FaceState::Back);
    transform(&mut t, saga);
    assert_eq!(name_of(&t, saga), "Sheoldred");
    t.resolve_all();
    assert_eq!(t.counters(saga, counters::PLUS1), 1);
}

#[test]
fn a_transformed_cards_mana_value_is_its_front_faces() {
    cr!("202.3b", "712.8e");
    ruling!(
        "Hidetsugu Consumes All // Vessel of the All-Consuming",
        "The mana value of a transforming double-faced card with its back face up is the mana value of the front face."
    );
    // (Vessel of the All-Consuming's last ability doesn't compile; its mana value
    // doesn't depend on it.) The front face costs {1}{B}{R}; the back face has no mana
    // cost.
    let mut t = TestGame::new(2);
    let vessel = enter_transformed(&mut t, P0, HIDETSUGU);
    assert_eq!(name_of(&t, vessel), "Vessel of the All-Consuming");
    assert!(t.obj_now(vessel).chars.mana_cost.is_none());
    assert_eq!(mana_value(&t, vessel), 3);
}

#[test]
fn final_iteration_doesnt_see_the_spell_that_transformed_docent() {
    cr!("603.2", "603.2e", "712.18");
    ruling!(
        "Docent of Perfection // Final Iteration",
        "When Docent of Perfection transforms into Final Iteration, the instant or sorcery spell that’s on the stack doesn’t cause Final Iteration’s triggered ability to trigger."
    );
    supported(DOCENT);
    let mut t = TestGame::new(2);
    let docent = t.battlefield(P0, DOCENT);
    create_token(&mut t, P0, "Wizard");
    create_token(&mut t, P0, "Wizard");
    // The first spell: a third Wizard, and Docent transforms.
    cast_bolt(&mut t, P0, P1);
    t.resolve_all();
    assert_eq!(name_of(&t, docent), "Final Iteration");
    assert_eq!(with_subtype(&t, P0, "Wizard").len(), 3);
    // The next spell triggers Final Iteration.
    cast_bolt(&mut t, P0, P1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Wizard").len(), 4);
}

#[test]
fn awoken_horrors_trigger_resolves_before_the_spell_that_transformed_thing_in_the_ice() {
    cr!("603.3", "405.5", "608.2b");
    ruling!(
        "Thing in the Ice // Awoken Horror",
        "When Thing in the Ice’s triggered ability transforms it, Awoken Horror’s ability will trigger and resolve before the spell that caused Thing in the Ice’s last ability to trigger."
    );
    supported(THING);
    let mut t = TestGame::new(2);
    let thing = t.battlefield(P0, THING);
    add(&mut t, thing, "ice", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Lightning Bolt at the Bears: Thing transforms, its trigger returns the Bears to
    // P1's hand, and the Bolt has no legal target when it tries to resolve.
    let bolt = cast_bolt(&mut t, P0, bears);
    t.resolve();
    assert_eq!(name_of(&t, thing), "Awoken Horror");
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn chalice_of_life_checks_your_life_total_only_as_its_ability_resolves() {
    cr!("608.2c", "701.27a");
    ruling!(
        "Chalice of Life // Chalice of Death",
        "You check your life total to see if Chalice of Life transforms only when its ability resolves."
    );
    supported(CHALICE);
    // "{T}: You gain 1 life. Then if you have at least 10 life more than your starting
    // life total, transform this artifact."
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P0, CHALICE);
    // A high life total on its own doesn't transform it.
    t.g.players[0].life = 40;
    t.settle();
    assert_eq!(name_of(&t, chalice), "Chalice of Life");
    // At 28 life, the ability makes it 29: no.
    t.g.players[0].life = 28;
    activate_containing(&mut t, P0, chalice, "gain 1 life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 29);
    assert_eq!(name_of(&t, chalice), "Chalice of Life");
    // 29 at activation; P0 is at 32 by the time it resolves (33): yes.
    t.g.objects[chalice.0 as usize].tapped = false;
    activate_containing(&mut t, P0, chalice, "gain 1 life").unwrap();
    t.g.players[0].life = 32;
    t.resolve_all();
    assert_eq!(t.life(P0), 33);
    assert_eq!(name_of(&t, chalice), "Chalice of Death");
}

#[test]
fn wolfbitten_captive_and_krallenhorde_killer_each_activate_once_per_turn() {
    cr!("602.5b", "712.18", "701.27a");
    ruling!(
        "Wolfbitten Captive // Krallenhorde Killer",
        "You can activate Wolfbitten Captive's ability, let it transform into Krallenhorde Killer, then activate Krallenhorde Killer's ability, all on the same turn, and vice versa."
    );
    supported(CAPTIVE);
    // Wolfbitten Captive "{1}{G}: +2/+2 ... Activate only once each turn." //
    // Krallenhorde Killer "{3}{G}: +4/+4 ... Activate only once each turn."
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, CAPTIVE);
    t.lands(P0, "Forest", 6);
    activate_containing(&mut t, P0, wolf, "+2/+2").unwrap();
    t.resolve_all();
    assert!(activate_containing(&mut t, P0, wolf, "+2/+2").is_err());
    transform(&mut t, wolf);
    assert_eq!(name_of(&t, wolf), "Krallenhorde Killer");
    activate_containing(&mut t, P0, wolf, "+4/+4").unwrap();
    t.resolve_all();
    // 2/2, +2/+2 and +4/+4.
    assert_eq!(t.pt(wolf), (8, 8));
    assert!(activate_containing(&mut t, P0, wolf, "+4/+4").is_err());
    // And the other way around.
    let mut t = TestGame::new(2);
    let killer = enter_transformed(&mut t, P0, CAPTIVE);
    t.lands(P0, "Forest", 6);
    activate_containing(&mut t, P0, killer, "+4/+4").unwrap();
    t.resolve_all();
    transform(&mut t, killer);
    assert_eq!(name_of(&t, killer), "Wolfbitten Captive");
    activate_containing(&mut t, P0, killer, "+2/+2").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(killer), (7, 7));
}

#[test]
fn wedding_announcement_transforms_only_as_its_trigger_resolves_and_keeps_its_counters() {
    cr!("608.2c", "712.18", "122.2");
    ruling!(
        "Wedding Announcement // Wedding Festivity",
        "Transforming is part of the triggered ability that puts invitation counters on Wedding Announcement. If some other effect causes Wedding Announcement to have three or more invitation counters on it, it won't transform until the next time its triggered ability resolves."
    );
    ruling!(
        "Wedding Announcement // Wedding Festivity",
        "Wedding Announcement keeps its invitation counters as it transforms into Wedding Festivity."
    );
    supported(WEDDING);
    let mut t = TestGame::new(2);
    let wedding = t.battlefield(P0, WEDDING);
    add(&mut t, wedding, "invitation", 3);
    t.settle();
    assert_eq!(name_of(&t, wedding), "Wedding Announcement");
    // At the end step: a fourth counter, a Human (no attack this turn), and it
    // transforms with its four counters.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Human").len(), 1);
    assert_eq!(name_of(&t, wedding), "Wedding Festivity");
    assert_eq!(t.counters(wedding, "invitation"), 4);
    // "Creatures you control get +1/+1."
    let human = with_subtype(&t, P0, "Human")[0];
    assert_eq!(t.pt(human), (2, 2));
}

#[test]
fn the_wedding_draws_a_card_after_an_attack_with_two_creatures() {
    cr!("508.1", "608.2c");
    // "If you attacked with two or more creatures this turn, draw a card. Otherwise,
    // create a 1/1 white Human creature token." (One counter: no transformation.)
    let mut t = TestGame::new(2);
    let wedding = t.battlefield(P0, WEDDING);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(with_subtype(&t, P0, "Human").is_empty());
    assert_eq!(t.counters(wedding, "invitation"), 1);
    assert_eq!(name_of(&t, wedding), "Wedding Announcement");
}

#[test]
fn only_the_first_elusive_tormentor_activation_to_resolve_transforms_it() {
    cr!("701.27f", "602.2");
    ruling!(
        "Elusive Tormentor // Insidious Mist",
        "You can activate Elusive Tormentor's ability multiple times to discard multiple cards. Only the first instance of the ability to resolve will cause it to transform."
    );
    supported(TORMENTOR);
    let mut t = TestGame::new(2);
    let tormentor = t.battlefield(P0, TORMENTOR);
    t.lands(P0, "Swamp", 2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    // "{1}, Discard a card: Transform this creature." — twice.
    activate_containing(&mut t, P0, tormentor, "Discard").unwrap();
    activate_containing(&mut t, P0, tormentor, "Discard").unwrap();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(name_of(&t, tormentor), "Insidious Mist");
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(name_of(&t, tormentor), "Insidious Mist");
}

#[test]
fn elusive_tormentor_attacks_turns_to_mist_and_back_and_deals_four() {
    cr!("506.4", "509.1h", "712.18", "510.1");
    ruling!(
        "Elusive Tormentor // Insidious Mist",
        "You can, all within one turn, attack with Elusive Tormentor, transform it into Insidious Mist before blockers are chosen, transform it back as its triggered ability resolves, and have it deal 4 combat damage."
    );
    let mut t = TestGame::new(2);
    let tormentor = t.battlefield(P0, TORMENTOR);
    t.lands(P0, "Swamp", 4);
    t.hand(P0, "Grizzly Bears");
    // P1's untapped creature can't block Insidious Mist.
    t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(tormentor, Entity::Player(P1))]);
    activate_containing(&mut t, P0, tormentor, "Discard").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, tormentor), "Insidious Mist");
    // "Whenever this creature attacks and isn't blocked, you may pay {2}{B}. If you do,
    // transform it."
    t.answer_yes(P0, true);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(name_of(&t, tormentor), "Elusive Tormentor");
    assert_eq!(t.life(P1), 16);
}

#[test]
fn insidious_mist_stays_a_mist_if_its_controller_doesnt_pay() {
    cr!("603.5", "509.1h");
    // The same, declining to pay: Insidious Mist deals 0 damage.
    let mut t = TestGame::new(2);
    let mist = enter_transformed(&mut t, P0, TORMENTOR);
    t.g.objects[mist.0 as usize].summoning_sick = false;
    t.lands(P0, "Swamp", 3);
    attack_with(&mut t, &[(mist, Entity::Player(P1))]);
    t.answer_yes(P0, false);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(name_of(&t, mist), "Insidious Mist");
    assert_eq!(untapped_lands(&t, P0), 3);
    assert_eq!(t.life(P1), 20);
}

/// P0's Soul Seizer attacks P1 (who controls Grizzly Bears) unblocked; the game stops
/// in the combat damage step with its trigger, targeting the Bears, on the stack.
/// Returns (Soul Seizer, the Bears).
fn soul_seizer_connects(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let seizer = t.battlefield(P0, SEIZER);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(t, &[(seizer, Entity::Player(P1))]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.stack_len(), 1);
    (seizer, bears)
}

#[test]
fn soul_seizer_targets_on_triggering_and_does_nothing_if_the_target_is_gone() {
    cr!("603.3d", "608.2b", "701.27a");
    ruling!(
        "Soul Seizer // Ghastly Haunting",
        "You choose the target for Soul Seizer's triggered ability when that ability triggers and goes on the stack. You choose whether or not to transform it when that ability resolves. If the creature is an illegal target by that time, the ability doesn't resolve and none of its effects happen. You can't have Soul Seizer transform."
    );
    supported(SEIZER);
    // "When this creature deals combat damage to a player, you may transform it. If you
    // do, attach it to target creature that player controls."
    let mut t = TestGame::new(2);
    let (seizer, bears) = soul_seizer_connects(&mut t);
    let from = t.asked().len();
    destroy(&mut t, bears);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(name_of(&t, seizer), "Soul Seizer");
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    // With the target still there, P0 chooses as it resolves: it transforms into
    // Ghastly Haunting attached to the Bears, and P0 controls them.
    let mut t = TestGame::new(2);
    let (seizer, bears) = soul_seizer_connects(&mut t);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(name_of(&t, seizer), "Ghastly Haunting");
    assert_eq!(t.obj_now(seizer).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.obj_now(bears).controller, P0);
    // Choosing not to: no transformation, nothing attached.
    let mut t = TestGame::new(2);
    let (seizer, bears) = soul_seizer_connects(&mut t);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(name_of(&t, seizer), "Soul Seizer");
    assert_eq!(t.obj_now(seizer).attached_to, None);
    assert_eq!(t.obj_now(bears).controller, P1);
}
