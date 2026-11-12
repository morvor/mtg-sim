//! Rulings batch P196 — control-changing effects with no duration (CR 611.2, 613.1b):
//! what gaining control of a permanent does and doesn't bring with it (Auras and
//! Equipment stay where they are, 301.5c-d, 303.4e), conditions checked on resolution,
//! "you" on a stolen enchantment, and triggered abilities that look back at the turn.

use crate::r_p196_common::*;
use crate::r_s01_common::{attack_with, custom_card};
use crate::r_s06_common::{activate_containing, give_control};
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1's Grizzly Bears equipped with Leonin Scimitar (+1/+1) and enchanted with Spirit
/// Link, both P1's.
fn equipped_bears(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scimitar = attach_new(t, P1, "Leonin Scimitar", bears);
    let link = attach_new(t, P1, "Spirit Link", bears);
    assert_eq!(t.pt(bears), (3, 3));
    (bears, scimitar, link)
}

/// P0 now controls the Bears; the Scimitar and Spirit Link are still P1's, still on it.
fn attachments_stayed(t: &TestGame, bears: ObjectId, scimitar: ObjectId, link: ObjectId) {
    assert_eq!(ctl(t, bears), P0);
    assert_eq!(ctl(t, scimitar), P1);
    assert_eq!(ctl(t, link), P1);
    assert_eq!(attached_to(t, scimitar), Some(obj(bears)));
    assert_eq!(attached_to(t, link), Some(obj(bears)));
}

/// P0 has gained control of P1's Scimitar on P1's Bears: it stays on the Bears, and P0
/// may equip it to their own creature in their main phase.
fn equipment_stayed_then_equip(t: &mut TestGame, bears: ObjectId, scimitar: ObjectId) {
    assert_eq!(ctl(t, scimitar), P0);
    assert_eq!(attached_to(t, scimitar), Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 3));
    let mine = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    lots(t, P0);
    t.answer_targets(P0, &[obj(mine)]);
    activate_containing(t, P0, scimitar, "Equip").expect("P0 can equip it");
    t.resolve_all();
    assert_eq!(attached_to(t, scimitar), Some(obj(mine)));
    assert_eq!(t.pt(mine), (4, 4));
}

#[test]
fn gaining_control_of_a_creature_leaves_its_auras_and_equipment() {
    cr!("301.5d", "303.4e", "701.12a");
    ruling!(
        "Agent of Treachery",
        "Gaining control of a creature doesn't cause you to gain control of any Auras or Equipment attached to it."
    );
    ruling!(
        "Oko, Thief of Crowns",
        "Gaining control of a permanent doesn't cause you to gain control of any Auras or Equipment attached to it."
    );
    ruling!(
        "Invoke the Winds",
        "Gaining control of a permanent doesn't cause you to gain control of any Auras or Equipment attached to that permanent."
    );
    ruling!(
        "Trade the Helm",
        "Gaining control of an artifact or creature doesn’t cause you to gain control of any Auras or Equipment attached to it."
    );
    for card in [
        "Agent of Treachery",
        "Oko, Thief of Crowns",
        "Invoke the Winds",
        "Trade the Helm",
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let (bears, scimitar, link) = equipped_bears(&mut t);
        match card {
            "Agent of Treachery" => {
                t.answer_targets(P0, &[obj(bears)]);
                t.enter(P0, card);
                t.g.flush_events();
                t.resolve_all();
            }
            "Oko, Thief of Crowns" => {
                let oko = t.battlefield(P0, card);
                t.g.add_counters(obj(oko), "loyalty", 3, None);
                let mine = t.battlefield(P0, "Hill Giant");
                act(&mut t, P0, oko, 2, &[obj(mine), obj(bears)]);
                assert_eq!(ctl(&t, mine), P1);
            }
            "Invoke the Winds" => {
                cast_at(&mut t, P0, card, &[obj(bears)]);
                t.resolve_all();
            }
            _ => {
                let mine = t.battlefield(P0, "Hill Giant");
                cast_at(&mut t, P0, card, &[obj(mine), obj(bears)]);
                t.resolve_all();
                assert_eq!(ctl(&t, mine), P1);
            }
        }
        attachments_stayed(&t, bears, scimitar, link);
    }
}

#[test]
fn gaining_control_of_an_equipment_doesnt_move_it() {
    cr!("301.5c", "301.5d", "702.6a", "701.12a");
    ruling!(
        "Agent of Treachery",
        "Gaining control of an Aura or Equipment doesn't change what it's currently attached to."
    );
    ruling!(
        "Oko, Thief of Crowns",
        "Gaining control of an Equipment doesn't cause it to become unattached, although its new controller may activate its equip abilities during their main phase."
    );
    ruling!(
        "Daring Thief",
        "Gaining control of an Aura or Equipment doesn't cause it to move, though gaining control of an Equipment will allow you to activate its equip ability to attach it to a creature you control later."
    );
    ruling!(
        "Shifting Loyalties",
        "Gaining control of an Aura or Equipment doesn’t cause it to move, though gaining control of an Equipment will allow you to activate its equip ability later to attach it to a creature you control."
    );
    ruling!(
        "Shrewd Negotiation",
        "Gaining control of an Equipment doesn't cause it to move, though it will allow you to activate its equip ability later to attach it to a creature you control."
    );
    // Each takes P1's Scimitar (on P1's Bears); the exchanges give P0's Sol Ring.
    for card in [
        "Agent of Treachery",
        "Daring Thief",
        "Shifting Loyalties",
        "Shrewd Negotiation",
    ] {
        let mut t = TestGame::new(2);
        let (bears, scimitar, link) = equipped_bears(&mut t);
        let ring = t.battlefield(P0, "Sol Ring");
        match card {
            "Agent of Treachery" => {
                t.answer_targets(P0, &[obj(link)]);
                t.enter(P0, card);
                t.g.flush_events();
                t.resolve_all();
                // The Aura stays on P1's Bears too.
                assert_eq!(ctl(&t, link), P0);
                assert_eq!(attached_to(&t, link), Some(obj(bears)));
                give_control(&mut t, scimitar, P0);
            }
            "Daring Thief" => {
                let thief = t.battlefield(P0, card);
                t.g.tap(thief);
                t.answer_targets(P0, &[obj(ring)]);
                t.answer_targets(P0, &[obj(scimitar)]);
                t.answer_yes(P0, true);
                t.g.untap(thief);
                t.g.flush_events();
                t.resolve_all();
            }
            "Shifting Loyalties" => {
                cast_multi(&mut t, P0, card, &[obj(ring), obj(scimitar)]);
                t.resolve_all();
            }
            _ => {
                cast_at(&mut t, P0, card, &[obj(ring), obj(scimitar)]);
                t.resolve_all();
            }
        }
        if card != "Agent of Treachery" {
            assert_eq!(ctl(&t, ring), P1, "{card}");
        }
        equipment_stayed_then_equip(&mut t, bears, scimitar);
    }
    // Oko's exchange: P1 gets P0's Scimitar, still on P0's creature.
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, Thief of Crowns");
    t.g.add_counters(obj(oko), "loyalty", 3, None);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let scimitar = attach_new(&mut t, P0, "Leonin Scimitar", elves);
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, oko, 2, &[obj(scimitar), obj(bears)]);
    assert_eq!(ctl(&t, scimitar), P1);
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(attached_to(&t, scimitar), Some(obj(elves)));
    assert_eq!(t.pt(elves), (2, 2));
    // P1 equips it to one of their creatures in their main phase.
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    t.answer_targets(P1, &[obj(giant)]);
    activate_containing(&mut t, P1, scimitar, "Equip").expect("P1 can equip it");
    t.resolve_all();
    assert_eq!(attached_to(&t, scimitar), Some(obj(giant)));
}

#[test]
fn aura_graft_control_lasts_until_something_changes_it() {
    cr!("611.2a", "613.1b", "400.7");
    ruling!(
        "Aura Graft",
        "Aura Graft’s effect has no duration. You’ll retain control of the Aura until the game ends, the Aura leaves the battlefield, or an effect causes someone else to gain control of the Aura."
    );
    supported("Aura Graft");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let link = attach_new(&mut t, P1, "Spirit Link", theirs);
    let mine = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(mine)]);
    cast_at(&mut t, P0, "Aura Graft", &[obj(link)]);
    t.resolve_all();
    assert_eq!(ctl(&t, link), P0);
    assert_eq!(attached_to(&t, link), Some(obj(mine)));
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert_eq!(ctl(&t, link), P0);
    // Another effect gives it back.
    give_control(&mut t, link, P1);
    assert_eq!(ctl(&t, link), P1);
    // Leaving the battlefield ends it: P0 takes it again, then it's bounced and recast
    // by its owner.
    give_control(&mut t, link, P0);
    crate::r_p189_common::bounce(&mut t, link);
    assert!(t.in_hand(P1, "Spirit Link"));
}

#[test]
fn connive_control_lasts_and_ignores_later_power() {
    cr!("611.2a", "611.2b", "608.2b");
    ruling!(
        "Connive // Concoct",
        "Connive's effect lasts indefinitely. It doesn't wear off during the cleanup step, and it doesn't expire if the creature's power becomes greater after Connive has resolved."
    );
    supported("Connive // Concoct");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let c = t.hand(P0, "Connive // Concoct");
    t.cast(P0, c)
        .method(CastMethod::Half(0))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    t.g.add_counters(obj(bears), "+1/+1", 3, None);
    t.g.recompute();
    assert_eq!(t.pt(bears).0, 5);
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert_eq!(ctl(&t, bears), P0);
}

#[test]
fn exert_influence_checks_power_as_it_resolves() {
    cr!("702.119a", "608.2c", "601.2h");
    ruling!(
        "Exert Influence",
        "Check the creature’s power as Exert Influence resolves to see if it’s less than or equal to the number of colors spent to cast Exert Influence. If it’s greater, Exert Influence will finish resolving with no effect."
    );
    supported("Exert Influence");
    // Two colors spent ({U}{R} and three generic); the Bears (power 2) grow in response.
    for grow in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        mana(&mut t, P0, ManaType::U, 1);
        mana(&mut t, P0, ManaType::R, 1);
        mana(&mut t, P0, ManaType::C, 3);
        let c = t.hand(P0, "Exert Influence");
        t.cast(P0, c).target(bears).go();
        if grow {
            pump(&mut t, bears, 1, 0);
        }
        t.resolve_all();
        assert_eq!(ctl(&t, bears), if grow { P1 } else { P0 }, "grow {grow}");
        assert!(t.in_graveyard(P0, "Exert Influence"));
    }
}

#[test]
fn emrakuls_cast_trigger_resolves_first_and_even_if_emrakul_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Emrakul, the World Anew",
        "Emrakul, the World Anew's triggered ability will resolve before Emrakul does. If Emrakul is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    supported("Emrakul, the World Anew");
    supported("Counterspell");
    for counter in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        lots(&mut t, P0);
        t.g.players[0].mana_pool.add_type(ManaType::C, 12);
        let c = t.hand(P0, "Emrakul, the World Anew");
        let spell = t.cast(P0, c).target(pl(P1)).go();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        if counter {
            cast_at(&mut t, P1, "Counterspell", &[obj(spell)]);
            t.resolve();
            assert!(t.in_graveyard(P0, "Emrakul, the World Anew"));
        }
        // The trigger resolves: P0 controls the Bears; Emrakul (if not countered) is
        // still on the stack.
        t.resolve();
        assert_eq!(ctl(&t, bears), P0, "counter {counter}");
        assert_eq!(t.stack_len(), if counter { 0 } else { 1 });
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Emrakul, the World Anew").len(),
            usize::from(!counter)
        );
    }
}

#[test]
fn aura_thief_only_matters_for_you_and_activated_abilities() {
    cr!("109.5", "602.2", "613.1b");
    ruling!(
        "Aura Thief",
        "Gaining control of an enchantment often isn’t very interesting since it probably won’t change what the enchantment is doing. It only matters if the enchantment does something specifically to “you” or “an opponent” or if the enchantment has an activated ability (which only the controller can use)."
    );
    supported("Aura Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Aura Thief");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let words = t.battlefield(P1, "Words of Worship");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let link = attach_new(&mut t, P1, "Spirit Link", theirs);
    let mine = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(theirs), (3, 3));
    kill(&mut t, thief);
    t.resolve_all();
    for e in [anthem, words, link] {
        assert_eq!(ctl(&t, e), P0);
    }
    // "Creatures you control" now means P0's.
    assert_eq!(t.pt(mine), (4, 4));
    assert_eq!(t.pt(theirs), (2, 2));
    // Spirit Link stays on P1's Bears, but P0 gains the life.
    assert_eq!(attached_to(&t, link), Some(obj(theirs)));
    crate::r_s06_common::damage(&mut t, theirs, 2, mine);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 20);
    // Only P0 can activate Words of Worship now.
    lots(&mut t, P1);
    assert!(!crate::r_s02_common::can_activate(&mut t, P1, words));
    lots(&mut t, P0);
    assert!(crate::r_s02_common::can_activate(&mut t, P0, words));
}

/// A 1/1 Pirate with the given rules text, for P0.
fn pirate(t: &mut TestGame, text: &str) -> ObjectId {
    let def = custom_card("Test Pirate", "Creature — Human Pirate", "{1}", Some((1, 1)), text);
    t.custom(P0, def, Zone::Battlefield)
}

#[test]
fn admiral_beckett_brass_targets_one_permanent_of_either_player() {
    cr!("603.10", "115.1", "608.2b");
    ruling!(
        "Admiral Beckett Brass",
        "If more than one player was dealt combat damage by three Pirates, you target only one permanent. It may be controlled by either of those players."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Admiral Beckett Brass");
    let ps: Vec<ObjectId> = (0..6).map(|_| pirate(&mut t, "")).collect();
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    let decl: Vec<(ObjectId, Entity)> = ps
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, pl(if i < 3 { P1 } else { P2 })))
        .collect();
    t.attack(&decl, &[]);
    assert_eq!((t.life(P1), t.life(P2)), (14, 14));
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(b2)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let asks: Vec<(Vec<Entity>, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, max, .. } if *p == P0 => {
                Some((candidates.clone(), *max))
            }
            _ => None,
        })
        .collect();
    assert_eq!(asks.len(), 1, "one target");
    assert_eq!(asks[0].1, 1);
    assert!(asks[0].0.contains(&obj(b1)) && asks[0].0.contains(&obj(b2)));
    assert_eq!(ctl(&t, b2), P0);
    assert_eq!(ctl(&t, b1), P1);
}

#[test]
fn admiral_beckett_brass_counts_a_double_striker_once() {
    cr!("603.10", "702.4b", "510.4");
    ruling!(
        "Admiral Beckett Brass",
        "If one Pirate deals combat damage multiple times, most likely because it has double strike, it counts as only one of the three Pirates for Admiral Beckett Brass's triggered ability."
    );
    // Two Pirates, one with double strike: three hits but two Pirates. A third Pirate
    // makes it three.
    for third in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Admiral Beckett Brass");
        let mut ps = vec![pirate(&mut t, "Double strike"), pirate(&mut t, "")];
        if third {
            ps.push(pirate(&mut t, ""));
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        let decl: Vec<(ObjectId, Entity)> = ps.iter().map(|p| (*p, pl(P1))).collect();
        t.attack(&decl, &[]);
        // Each Pirate is 2/2 with the Admiral; the double striker hits twice.
        assert_eq!(t.life(P1), 20 - 2 * (ps.len() as i32 + 1));
        t.answer_targets(P0, &[obj(bears)]);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(ctl(&t, bears), if third { P0 } else { P1 }, "third {third}");
    }
}

#[test]
fn gollum_counts_any_gollum_and_only_the_life_gained() {
    cr!("603.10", "119.9", "201.2");
    ruling!(
        "Gollum, Obsessed Stalker",
        "Gollum, Obsessed Stalker's last ability counts players who were dealt combat damage by any creature named Gollum, Obsessed Stalker. It doesn't matter who controlled those creatures."
    );
    ruling!(
        "Gollum, Obsessed Stalker",
        "Gollum, Obsessed Stalker's last ability counts the total amount of life you gained without taking into account any life you lost during that turn. For example, if you gained 3 life and lost 3 life earlier in the turn, each opponent dealt combat damage this game by a creature named Gollum, Obsessed Stalker loses 3 life."
    );
    supported("Gollum, Obsessed Stalker");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Gollum, Obsessed Stalker");
    // P2's Gollum hits P1 in P2's turn.
    let theirs = t.battlefield(P2, "Gollum, Obsessed Stalker");
    t.advance_to(P2, Step::BeginningOfCombat);
    attack_with(&mut t, &[(theirs, pl(P1))]);
    t.advance_to(P2, Step::End);
    assert_eq!(t.life(P1), 19);
    // P0's turn: gains 3, loses 3; at P0's end step P1 loses 3, P2 nothing.
    t.advance_to(P0, Step::PrecombatMain);
    crate::r_p009_common::gain(&mut t, P0, 3);
    crate::r_p009_common::lose(&mut t, P0, 3);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P2), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn olivia_voldarens_ping_makes_it_a_vampire_before_it_dies() {
    cr!("608.2c", "704.5g", "603.10a");
    ruling!(
        "Olivia Voldaren",
        "If Olivia Voldaren deals lethal damage to a creature with its first activated ability, that creature will become a Vampire before dying."
    );
    supported("Olivia Voldaren");
    let mut t = TestGame::new(2);
    let olivia = t.battlefield(P0, "Olivia Voldaren");
    let elves = t.battlefield(P1, "Llanowar Elves");
    act(&mut t, P0, olivia, 0, &[obj(elves)]);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    // It died as a Vampire (its last known information).
    assert!(t.g.obj(elves).chars.has_subtype("Vampire"));
    assert!(t.g.obj(elves).chars.has_subtype("Elf"));
    assert_eq!(t.counters(olivia, "+1/+1"), 1);
    // A survivor stays a Vampire after the turn (no duration).
    let giant = t.battlefield(P1, "Hill Giant");
    act(&mut t, P0, olivia, 0, &[obj(giant)]);
    pass_turn(&mut t);
    assert!(t.obj_now(giant).chars.has_subtype("Vampire"));
}
