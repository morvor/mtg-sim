//! Rulings batch P008 — burn that can hit planeswalkers, with damage prevented, redirected,
//! increased or reduced: the rest of the effect still happens (CR 608.2c, 615), effects
//! that count "players dealt damage this way" see only damage actually dealt (CR 120.3),
//! "damage can't be prevented" stops only prevention effects (CR 615.12), redirected
//! combat damage is still combat damage (CR 614.9, 510.2), prevention shields against
//! simultaneous damage (CR 615.7), and "if it would die this turn, exile it instead"
//! (CR 614.6).

use crate::r_p007_common::*;
use crate::r_p008_common::*;
use crate::r_p113_common::cast_modes;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::attach_new;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

/// P1 casts Healing Salve's second mode: the next 3 damage to `to` this turn is prevented.
fn salve(t: &mut TestGame, to: Entity) {
    cast_modes(t, P1, "Healing Salve", &[1], &[to]);
    t.resolve_all();
}

/// P1 casts Safe Passage: all damage to P1 and P1's creatures this turn is prevented.
fn safe_passage(t: &mut TestGame) {
    cast_new(t, P1, "Safe Passage", &[]);
    t.resolve_all();
}

#[test]
fn goblin_chainwhirler_still_damages_permanents_if_the_player_damage_is_prevented() {
    cr!("608.2c", "615.1a");
    ruling!(
        "Goblin Chainwhirler",
        "If the damage Goblin Chainwhirler would deal to a player is prevented, it still deals 1 damage to that player’s creatures and planeswalkers."
    );
    supported("Goblin Chainwhirler");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let theirs = pw(&mut t, P1, "Chandra, Torch of Defiance", 4);
    salve(&mut t, pl(P1));
    t.enter(P0, "Goblin Chainwhirler");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(dmg(&t, bears), 1);
    assert_eq!(t.counters(theirs, "loyalty"), 3);
}

#[test]
fn nicol_bolas_discard_and_sacrifice_dont_depend_on_the_damage() {
    cr!("608.2c", "615.1a");
    ruling!(
        "Nicol Bolas, Planeswalker",
        "If the damage from Nicol Bolas's third ability is prevented or redirected, the rest of the effect will still happen. Specifically, the number of cards discarded and permanents sacrificed isn't tied to the amount of damage dealt."
    );
    supported("Nicol Bolas, Planeswalker");
    // Prevented: Safe Passage.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.hand(P1, "Grizzly Bears");
    safe_passage(&mut t);
    let lands: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == P1)
        .map(|o| o.id)
        .collect();
    let b = pw(&mut t, P0, "Nicol Bolas, Planeswalker", 9);
    loyalty(&mut t, P0, b, 2, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "all 7 prevented");
    assert_eq!(t.hand_size(P1), 0, "still discards");
    assert!(!t.on_battlefield(wurm), "still sacrifices");
    assert!(lands.iter().all(|l| !t.on_battlefield(*l)));
    // Redirected: Beacon of Destiny takes the damage instead; P1 still discards and
    // sacrifices (the Beacon is dealt 7 and dies anyway).
    let mut t = TestGame::new(2);
    let beacon = t.battlefield(P1, "Beacon of Destiny");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.hand(P1, "Grizzly Bears");
    let b = pw(&mut t, P0, "Nicol Bolas, Planeswalker", 9);
    loyalty(&mut t, P0, b, 2, &[pl(P1)]);
    t.answer_choose(P1, &[obj(b)]);
    t.activate(P1, beacon, 0, &[]).expect("Beacon of Destiny");
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "redirected");
    assert_eq!(t.hand_size(P1), 0);
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn chandra_pyromaster_creature_cant_block_even_if_the_damage_is_prevented() {
    cr!("608.2c", "615.1a");
    ruling!(
        "Chandra, Pyromaster",
        "If the first ability resolves but the damage is prevented or redirected, the target creature still won't be able to block that turn."
    );
    supported("Chandra, Pyromaster");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    safe_passage(&mut t);
    let c = pw(&mut t, P0, "Chandra, Pyromaster", 4);
    loyalty(&mut t, P0, c, 0, &[pl(P1), obj(bears)]);
    t.resolve_all();
    assert_eq!((t.life(P1), dmg(&t, bears)), (20, 0));
    assert!(!t.g.can_block_at_all(bears));
}

/// Chandra, Roaring Flame on the battlefield under P0's control with `loyalty`.
fn roaring_flame(t: &mut TestGame, loyalty: u32) -> ObjectId {
    let c = t.battlefield(P0, "Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    assert!(mtg_engine::dfc::transform(&mut t.g, c));
    let c = t.g.current(c);
    t.g.objects[c.0 as usize]
        .counters
        .insert("loyalty".into(), loyalty);
    t.g.recompute();
    c
}

/// The owners of the emblems in the command zone.
fn emblem_owners(t: &TestGame) -> Vec<PlayerId> {
    let mut v: Vec<PlayerId> = t
        .g
        .objects
        .iter()
        .filter(|o| o.kind == ObjKind::Emblem && o.zone == Zone::Command)
        .map(|o| o.owner)
        .collect();
    v.sort();
    v
}

#[test]
fn chandra_roaring_flame_emblems_go_only_to_players_dealt_damage() {
    cr!("120.3", "608.2c", "615.1a", "614.9");
    ruling!(
        "Chandra, Fire of Kaladesh // Chandra, Roaring Flame",
        "Only players actually dealt damage by the third ability of Chandra, Roaring Flame will get an emblem. If all of that damage to a player is prevented, that player won't get an emblem. If any of that damage is redirected to Chandra's controller, that player will get an emblem."
    );
    supported("Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    supported("Harm's Way");
    // Three players: P1's damage is all prevented; P2 is dealt 6.
    let mut t = TestGame::new(3);
    safe_passage(&mut t);
    let c = roaring_flame(&mut t, 7);
    loyalty(&mut t, P0, c, 2, &[]);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (20, 14));
    assert_eq!(emblem_owners(&t), vec![P2]);
    // Two players: P1 redirects 2 of it to P0 with Harm's Way: both get an emblem.
    let mut t = TestGame::new(2);
    let c = roaring_flame(&mut t, 7);
    loyalty(&mut t, P0, c, 2, &[]);
    t.answer_choose(P1, &[obj(c)]);
    cast_new(&mut t, P1, "Harm's Way", &[pl(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 16));
    assert_eq!(emblem_owners(&t), vec![P0, P1]);
}

#[test]
fn bonfire_damage_to_creatures_doesnt_depend_on_the_player_damage() {
    cr!("608.2c", "615.1a", "614.1a");
    ruling!(
        "Bonfire of the Damned",
        "Preventing some or all of the damage dealt to the player won’t affect the amount of damage dealt to each creature that player controls. The same is true for any effect that increases or decreases the amount of damage dealt to the player."
    );
    // Prevented: Healing Salve on P1.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    salve(&mut t, pl(P1));
    t.lands(P0, "Wastes", 2);
    cast_x(&mut t, P0, "Bonfire of the Damned", NORMAL, Some(2), &[pl(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P1), dmg(&t, wurm)), (20, 2));
    // Increased: Fiendish Duo doubles damage dealt to P0's opponents only.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P0, "Fiendish Duo");
    t.lands(P0, "Wastes", 2);
    cast_x(&mut t, P0, "Bonfire of the Damned", NORMAL, Some(2), &[pl(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P1), dmg(&t, wurm)), (16, 2));
}

#[test]
fn quenchable_fire_delayed_damage_follows_the_target_not_the_damage() {
    cr!("603.7", "608.2c", "615.1a", "614.9");
    ruling!(
        "Quenchable Fire",
        "Quenchable Fire’s delayed triggered ability (and the special action of paying {U}) will affect the player who was targeted by the spell, even if the original 3 damage is prevented or redirected to a different permanent or player."
    );
    supported("Quenchable Fire");
    for redirect in [false, true] {
        let mut t = TestGame::new(2);
        let beacon = t.battlefield(P1, "Beacon of Destiny");
        if !redirect {
            salve(&mut t, pl(P1));
        }
        let s = cast_x(&mut t, P0, "Quenchable Fire", NORMAL, None, &[pl(P1)]);
        if redirect {
            t.answer_choose(P1, &[obj(s)]);
            t.activate(P1, beacon, 0, &[]).expect("Beacon of Destiny");
        }
        t.resolve_all();
        assert_eq!(t.life(P1), 20, "redirect {redirect}");
        // P1 doesn't pay {U}: at P0's next upkeep, P1 is dealt 3.
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.life(P1), 17, "redirect {redirect}");
    }
}

#[test]
fn cant_be_prevented_doesnt_stop_effects_that_dont_say_prevent() {
    cr!("615.12", "614.1a");
    ruling!(
        "Impractical Joke",
        "Impractical Joke only stops damage from being prevented by effects that specifically use the word \"prevent.\""
    );
    ruling!(
        "Questing Beast",
        "Questing Beast only stops combat damage from being prevented by effects that specifically use the word “prevent.”"
    );
    // Impractical Joke resolved; Divine Presence ("If a source would deal 4 or more damage
    // to a permanent or player, it deals 3 damage to that permanent or player instead.")
    // still reduces Lava Axe's 5 to 3, while Healing Salve's shield on P1 prevents none of
    // it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Divine Presence");
    cast_x(&mut t, P0, "Impractical Joke", NORMAL, None, &[]);
    t.resolve_all();
    salve(&mut t, pl(P1));
    cast_x(&mut t, P0, "Lava Axe", NORMAL, None, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "reduced to 3, not prevented");
    // Questing Beast: its 4 combat damage is reduced to 3 by Divine Presence, while a
    // Healing Salve shield on P1 doesn't prevent any of it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Divine Presence");
    let beast = t.battlefield(P0, "Questing Beast");
    attack_with(&mut t, &[(beast, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Questing Beast");
    attack_with(&mut t, &[(beast, pl(P1))]);
    salve(&mut t, pl(P1));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16, "not prevented");
}

#[test]
fn impractical_joke_protection_still_stops_targeting() {
    cr!("615.12", "702.16b", "702.16e");
    ruling!(
        "Impractical Joke",
        "Protection prevents damage, so protection abilities will be unable to prevent damage during a turn in which Impractical Joke has resolved. However, this won't allow spells or abilities to target permanents they wouldn't ordinarily be able to target because of protection, even if that spell or ability would deal damage to those permanents."
    );
    let mut t = TestGame::new(2);
    let fw = t.battlefield(P1, "Kor Firewalker");
    cast_x(&mut t, P0, "Impractical Joke", NORMAL, None, &[]);
    t.resolve_all();
    // Lightning Bolt can't target it.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let from = t.asked().len();
    t.cast(P0, bolt).target(P1).go();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&pl(P1)));
    assert!(!offered.contains(&obj(fw)), "protection from red");
    t.resolve_all();
    // Earthquake doesn't target: the red damage isn't prevented.
    t.answer_yes(P1, false);
    cast_x(&mut t, P0, "Earthquake", NORMAL, Some(2), &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(fw), "Kor Firewalker (2/2) is dealt 2");
}

#[test]
fn skullcrack_life_gain_and_prevention_spells_still_resolve() {
    cr!("119.7", "615.12", "608.2c");
    ruling!(
        "Skullcrack",
        "Spells and abilities that would cause a player to gain life or that would prevent damage still resolve, but the life-gain and damage-prevention parts have no effect."
    );
    let mut t = TestGame::new(2);
    cast_x(&mut t, P0, "Skullcrack", NORMAL, None, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Lightning Helix: deals its 3 damage; P1 gains no life.
    cast_new(&mut t, P1, "Lightning Helix", &[pl(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
    assert!(t.in_graveyard(P1, "Lightning Helix"));
    // Healing Salve's shield resolves but prevents nothing.
    salve(&mut t, pl(P1));
    assert!(t.in_graveyard(P1, "Healing Salve"));
    let src = t.battlefield(P0, "Grizzly Bears");
    hit_player(&mut t, src, P1, 2);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn wall_of_souls_triggers_on_redirected_combat_damage() {
    cr!("510.2", "614.9", "120.2a");
    ruling!(
        "Wall of Souls",
        "Redirected damage is still combat damage, so it will trigger on combat damage that is redirected to it."
    );
    supported("Wall of Souls");
    // Pariah on P1's Wall of Souls: the Bears' combat damage to P1 is dealt to the Wall.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Souls");
    attach_new(&mut t, P1, "Pariah", wall);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P1, &[pl(P0)]);
    attack_with(&mut t, &[(bears, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(dmg(&t, wall), 2);
    assert_eq!(t.life(P0), 18, "the Wall's trigger");
}

#[test]
fn vengeful_archon_you_choose_which_combat_damage_to_prevent() {
    cr!("615.7");
    ruling!(
        "Vengeful Archon",
        "If you would be dealt combat damage by multiple attacking creatures, you choose which of that damage to prevent. For example, if X is 3 and you’d be dealt combat damage by a 1/3 Scroll Thief and a 3/5 Siege Mastodon, you might choose to prevent 1 damage from the Scroll Thief and 2 damage from the Siege Mastodon. You don’t decide until the point at which the creatures would deal their damage."
    );
    // P1 attacks P0 with Scroll Thief ("Whenever this creature deals combat damage to a
    // player, draw a card.") and Siege Mastodon; P0's shield of 3 is set up first.
    // Order [thief, mastodon]: 1 from the Thief and 2 from the Mastodon are prevented (no
    // card drawn); [mastodon, thief]: 3 from the Mastodon, the Thief's 1 is dealt.
    for (thief_first, life, drew) in [(true, 19, 0), (false, 19, 1)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let archon = t.battlefield(P0, "Vengeful Archon");
        let thief = t.battlefield(P1, "Scroll Thief");
        let mastodon = t.battlefield(P1, "Siege Mastodon");
        t.lands(P0, "Wastes", 3);
        t.answer(P0, DecisionKind::X, Answer::Number(3));
        t.g.turn.priority = Some(P0);
        t.activate(P0, archon, 0, &[pl(P1)]).expect("Vengeful Archon");
        t.resolve_all();
        let hand = t.hand_size(P1);
        // The order is asked only as the damage is dealt.
        let order = if thief_first { vec![0, 1] } else { vec![1, 0] };
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        attack_with(&mut t, &[(thief, pl(P0)), (mastodon, pl(P0))]);
        block_and_finish(&mut t, P0, &[]);
        t.resolve_all();
        assert_eq!(t.life(P0), life, "thief_first {thief_first}");
        assert_eq!(t.life(P1), 17, "the Archon deals 3");
        assert_eq!(t.hand_size(P1), hand + drew, "thief_first {thief_first}");
    }
}

#[test]
fn exile_instead_of_dying_applies_for_any_reason_this_turn() {
    cr!("614.6", "614.1a", "608.2c");
    ruling!(
        "Obliterating Bolt",
        "Obliterating Bolt's replacement effect will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Scorching Dragonfire",
        "Scorching Dragonfire's replacement effect will exile the target creature or planeswalker if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Scorching Dragonfire",
        "Scorching Dragonfire's replacement effect will exile the target creature or planeswalker if it would die this turn for any reason, not just due to lethal damage. It applies to the target creature or planeswalker even if Scorching Dragonfire deals no damage to it (most likely due to a prevention effect)."
    );
    for name in ["Obliterating Bolt", "Scorching Dragonfire"] {
        // Indomitable Ancients (2/10) survives the damage; it's destroyed later this turn.
        let mut t = TestGame::new(2);
        let wall = t.battlefield(P1, "Indomitable Ancients");
        cast_x(&mut t, P0, name, NORMAL, None, &[obj(wall)]);
        t.resolve_all();
        assert!(t.on_battlefield(wall), "{name}");
        destroy(&mut t, wall);
        assert!(t.in_exile("Indomitable Ancients"), "{name}");
        assert!(!t.in_graveyard(P1, "Indomitable Ancients"), "{name}");
    }
    // Scorching Dragonfire's damage is all prevented; the planeswalker still dies later
    // (0 loyalty) and is exiled.
    let mut t = TestGame::new(2);
    let theirs = pw(&mut t, P1, "Chandra, Torch of Defiance", 4);
    salve(&mut t, obj(theirs));
    cast_x(
        &mut t,
        P0,
        "Scorching Dragonfire",
        NORMAL,
        None,
        &[obj(theirs)],
    );
    t.resolve_all();
    assert_eq!(t.counters(theirs, "loyalty"), 4, "prevented");
    cast_x(&mut t, P0, "Lightning Bolt", NORMAL, None, &[obj(theirs)]);
    t.resolve_all();
    cast_x(&mut t, P0, "Shock", NORMAL, None, &[obj(theirs)]);
    t.resolve_all();
    assert!(t.in_exile("Chandra, Torch of Defiance"));
}

#[test]
fn stensia_bloodhall_deals_colorless_damage() {
    cr!("105.2c");
    ruling!(
        "Stensia Bloodhall",
        "Like other lands, Stensia Bloodhall is colorless. The damage it deals is from a colorless source, even though activating its ability requires colored mana."
    );
    // Sphere of Law: "If a red source would deal damage to you, prevent 2 of that damage."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Law");
    let hall = t.battlefield(P0, "Stensia Bloodhall");
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.activate(P0, hall, 1, &[pl(P1)]).expect("Stensia Bloodhall");
    t.resolve_all();
    assert!(crate::r_s05_common::colorless(&t, hall));
    assert_eq!(t.life(P1), 18, "not a red source");
    // A red source's 2 damage would be prevented.
    let bolt_src = t.battlefield(P0, "Raging Goblin");
    hit_player(&mut t, bolt_src, P1, 2);
    assert_eq!(t.life(P1), 18);
}
