//! Rulings batch P159 — pump abilities and combat: becoming unblockable or gaining menace
//! after blocks (CR 509.1h), blocking restrictions combined (CR 509.1b), attacking without
//! defender (hellbent), sacrificing combatants (CR 510.1, 704.5g), "can't block" targets
//! (CR 608.2b), attack triggers that act before blockers (CR 508.2), lands that become
//! creatures (CR 302.6), tokens put onto the battlefield attacking (CR 508.4), and
//! Immolation Shaman's "activates an ability" trigger (CR 602.1, 605.1a).

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::{activate_containing, attach_new};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn opp() -> Entity {
    Entity::Player(P1)
}

fn blocked(t: &TestGame, id: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(t.g.current(id)))
}

fn attacking(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_attacking(t.g.current(id))
}

#[test]
fn becoming_unblockable_or_menace_after_blocks_doesnt_unblock() {
    cr!("509.1h", "506.4");
    ruling!(
        "Kaseto, Orochi Archmage",
        "Once a creature has been legally blocked, activating Kaseto’s ability targeting that creature won’t cause it to become unblocked. It will give it +2/+2 if it’s a Snake, though."
    );
    ruling!(
        "Immolation Shaman",
        "Activating Immolation Shaman's second ability after one creature has blocked it won't cause Immolation Shaman to become unblocked."
    );
    supported("Kaseto, Orochi Archmage");
    supported("Immolation Shaman");
    // Kaseto targets a blocked Ambush Viper (a Snake).
    let mut t = TestGame::new(2);
    let kaseto = t.battlefield(P0, "Kaseto, Orochi Archmage");
    let viper = t.battlefield(P0, "Ambush Viper");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    to_blockers(&mut t, &[(viper, opp())], &[(bears, viper)]);
    assert!(blocked(&t, viper));
    t.answer_targets(P0, &[obj(viper)]);
    activate_containing(&mut t, P0, kaseto, "can't be blocked").expect("activate");
    t.resolve_all();
    assert!(blocked(&t, viper));
    assert_eq!(t.pt(viper), (4, 3));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Immolation Shaman gains menace after one creature blocked it.
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P0, "Immolation Shaman");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    to_blockers(&mut t, &[(shaman, opp())], &[(bears, shaman)]);
    activate_containing(&mut t, P0, shaman, "menace").expect("activate");
    t.resolve_all();
    assert!(blocked(&t, shaman));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn safewright_cavalry_with_menace_cant_be_blocked() {
    cr!("509.1b", "702.111b");
    ruling!(
        "Safewright Cavalry",
        "If Safewright Cavalry gains menace and still has its first ability, it can't be blocked at all."
    );
    supported("Safewright Cavalry");
    let mut t = TestGame::new(2);
    let cav = t.battlefield(P0, "Safewright Cavalry");
    attach_new(&mut t, P0, "Madcap Skills", cav);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    // Menace forbids one blocker and the Cavalry's own ability forbids two or more: a
    // single blocker (legal without menace) is rejected.
    to_blockers(&mut t, &[(cav, opp())], &[(a, cav)]);
    assert!(!blocked(&t, cav));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn slithering_shade_keeps_attacking_after_a_card_is_put_into_hand() {
    cr!("506.4", "702.3b", "508.1c");
    ruling!(
        "Slithering Shade",
        "If you declare Slithering Shade as an attacker, then a card is put into your hand, the Shade won't be removed from combat."
    );
    supported("Slithering Shade");
    let mut t = TestGame::new(2);
    let shade = t.battlefield(P0, "Slithering Shade");
    t.lands(P0, "Swamp", 2);
    attack_with(&mut t, &[(shade, opp())]);
    assert!(attacking(&t, shade));
    t.g.draw_cards(P0, 1);
    for _ in 0..2 {
        activate_containing(&mut t, P0, shade, "+1/+1").expect("activate");
        t.resolve_all();
    }
    assert!(attacking(&t, shade));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn sacrificing_a_blocked_attacker_during_declare_blockers() {
    cr!("510.1", "506.4", "704.5g");
    ruling!(
        "Vampire Aristocrat",
        "If you sacrifice an attacking or blocking creature during the declare blockers step, it won't deal combat damage."
    );
    ruling!(
        "Bloodthrone Vampire",
        "If you sacrifice an attacking or blocking creature during the declare blockers step, it won't deal combat damage."
    );
    for card in ["Vampire Aristocrat", "Bloodthrone Vampire"] {
        supported(card);
        let mut t = TestGame::new(2);
        let v = t.battlefield(P0, card);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Wall of Wood");
        to_blockers(&mut t, &[(bears, opp())], &[(wall, bears)]);
        t.answer_choose(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, v, "Sacrifice").expect("activate");
        t.resolve_all();
        let (p, q) = t.pt(v);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(crate::r_s29_common::damage_marked(&t, wall), 0, "{card}");
        assert_eq!(t.pt(v), (p, q), "{card}");
    }
    // Waiting until the combat damage step: a creature dealt lethal damage is already gone.
    ruling!(
        "Vampire Aristocrat",
        "If you wait until the combat damage step, but that creature has been dealt lethal damage, it'll be destroyed before you get a chance to sacrifice it."
    );
    ruling!(
        "Bloodthrone Vampire",
        "If you wait until the combat damage step, but the creature you wish to sacrifice is dealt lethal damage, it'll be destroyed before you get a chance to sacrifice it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vampire Aristocrat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(bears, opp())], &[(giant, bears)]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn radhas_firebrand_target_legality_and_lasting_effect() {
    cr!("608.2b", "115.1", "611.2c");
    ruling!(
        "Radha's Firebrand",
        "Increasing the target creature's power in response to Radha's Firebrand's first ability can cause it to be an illegal target for the ability. However, once the ability has resolved, it won't be able to block even if its power becomes greater than Radha's Firebrand's power."
    );
    supported("Radha's Firebrand");
    for respond in [true, false] {
        let mut t = TestGame::new(2);
        let f = t.battlefield(P0, "Radha's Firebrand");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        attack_with(&mut t, &[(f, opp())]);
        assert_eq!(triggers_on_stack(&t, "can't block"), 1);
        let pump = |t: &mut TestGame| {
            give_mana_for(t, P1, "Giant Growth");
            let g = t.hand(P1, "Giant Growth");
            t.cast(P1, g).target(bears).go();
        };
        if respond {
            pump(&mut t);
            t.resolve_all();
        } else {
            t.resolve_all();
            pump(&mut t);
            t.resolve_all();
        }
        assert_eq!(t.pt(bears), (5, 5));
        block_and_finish(&mut t, P1, &[(bears, f)]);
        assert_eq!(!t.on_battlefield(f), respond, "blocked: {respond}");
        assert_eq!(t.life(P1), if respond { 20 } else { 17 });
    }
}

#[test]
fn surtland_flinger_uses_the_sacrificed_power_before_blocks() {
    cr!("603.12", "608.2h", "508.2");
    ruling!(
        "Surtland Flinger",
        "Use the power of the creature as it was sacrificed to determine how much damage is dealt. Damage will be dealt before blockers are declared."
    );
    supported("Surtland Flinger");
    let mut t = TestGame::new(2);
    let fl = t.battlefield(P0, "Surtland Flinger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p044_common::cast_spell(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[opp()]);
    attack_with(&mut t, &[(fl, opp())]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn a_land_animated_the_turn_it_entered_cant_tap_or_attack() {
    cr!("302.6", "508.1a");
    ruling!(
        "Kamahl, Fist of Krosa",
        "Notably, if you turn a land into a creature on the turn it entered the battlefield, you won’t be able to tap it for mana."
    );
    let mut t = TestGame::new(2);
    let kamahl = t.battlefield(P0, "Kamahl, Fist of Krosa");
    t.lands(P0, "Forest", 1);
    let land = crate::r_s20_common::entered_this_turn(&mut t, P0, "Forest");
    t.answer_targets(P0, &[obj(land)]);
    activate_containing(&mut t, P0, kamahl, "becomes").expect("activate");
    t.resolve_all();
    assert!(crate::r_s20_common::creature_now(&mut t, land));
    assert!(!crate::r_s20_common::tap_for_mana(&mut t, P0, land, "Add"));
    // An older land that becomes a creature can.
    let mut t = TestGame::new(2);
    let kamahl = t.battlefield(P0, "Kamahl, Fist of Krosa");
    t.lands(P0, "Forest", 1);
    let old = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[obj(old)]);
    activate_containing(&mut t, P0, kamahl, "becomes").expect("activate");
    t.resolve_all();
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, old, "Add"));
}

#[test]
fn immolation_shaman_triggers_on_nonmana_abilities_of_permanents_only() {
    cr!("602.1", "605.1a", "603.3b", "702.29a");
    ruling!(
        "Immolation Shaman",
        "An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    ruling!(
        "Immolation Shaman",
        "Immolation Shaman's first ability doesn't trigger when an opponent activates an ability of a card in hand (such as cycling)"
    );
    ruling!(
        "Immolation Shaman",
        "Immolation Shaman's first ability resolves before the ability that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Immolation Shaman");
    t.set_step(P1, Step::PrecombatMain);
    // P1 taps Llanowar Elves for mana: no trigger.
    let elves = t.battlefield(P1, "Llanowar Elves");
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P1, elves, "Add"));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // P1 cycles a creature card in hand: no trigger.
    let beast = t.hand(P1, "Yoked Plowbeast");
    t.lands(P1, "Forest", 6);
    activate_containing(&mut t, P1, beast, "Cycling").expect("cycle");
    t.settle();
    assert_eq!(t.stack_len(), 1, "only the cycling ability");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // P1 activates Elder of Laurels' ability ({3}{G}, not a mana ability): the trigger goes
    // on top and resolves first.
    let elder = t.battlefield(P1, "Elder of Laurels");
    t.answer_targets(P1, &[obj(elder)]);
    activate_containing(&mut t, P1, elder, "+X/+X").expect("activate");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.pt(elder), (2, 3));
    t.resolve_all();
    assert_eq!(t.pt(elder), (4, 5));
}

#[test]
fn mila_puts_loyalty_on_every_planeswalker_you_control() {
    cr!("508.1b", "603.2");
    ruling!(
        "Mila, Crafty Companion // Lukka, Wayward Bonder",
        "When Mila’s first ability resolves, you will put a loyalty counter on all planeswalkers you control, even ones that aren’t being attacked."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mila, Crafty Companion // Lukka, Wayward Bonder");
    let a = t.battlefield(P0, "Jace Beleren");
    let b = t.battlefield(P0, "Chandra, Torch of Defiance");
    let (la, lb) = (t.counters(a, "loyalty"), t.counters(b, "loyalty"));
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, obj(a))]);
    t.resolve_all();
    assert_eq!(
        (t.counters(a, "loyalty"), t.counters(b, "loyalty")),
        (la + 1, lb + 1)
    );
}

#[test]
fn krothuss_token_can_attack_a_different_player() {
    cr!("508.4");
    ruling!(
        "Runo Stromkirk // Krothuss, Lord of the Deep",
        "You choose which opponent or opposing planeswalker the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Krothuss, Lord of the Deep is attacking."
    );
    let mut t = TestGame::new(3);
    let k = t.battlefield(P0, "Runo Stromkirk // Krothuss, Lord of the Deep");
    crate::r_s17_common::transform(&mut t, k);
    let k = t.g.current(k);
    assert_eq!(t.obj_now(k).chars.name, "Krothuss, Lord of the Deep");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    let choose_p2 = |_: &mtg_engine::game::Game, d: &Decision| match d {
        Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&Entity::Player(P2)) =>
        {
            Some(Answer::Entities(vec![Entity::Player(P2)]))
        }
        Decision::ChooseOption { options, .. } if options.iter().any(|o| o.contains("2")) => {
            Some(Answer::Index(
                options.iter().position(|o| o.contains("2")).unwrap(),
            ))
        }
        _ => None,
    };
    crate::r_s03_common::respond(&mut t, P0, choose_p2);
    attack_with(&mut t, &[(k, opp()), (bears, opp())]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1, "{:?}", asked_since(&t, from));
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(toks[0]), Some(Entity::Player(P2)));
    assert_eq!(c.attack_target(t.g.current(k)), Some(opp()));
}
