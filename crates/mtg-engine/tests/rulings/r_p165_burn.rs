//! Rulings batch P165 — burn synergies: Tamanoa, Blaze Commando, damage-increasing
//! replacement effects (Pyromancer's Swath, Taii Wakeen, Torture Pit, Aether Revolt), and
//! "whenever an opponent is dealt noncombat damage" (Chandra's Spitfire, Wildfire
//! Elemental, Chandra's Pyreling).

use crate::r_p189_common::{bounce, triggers_of};
use crate::r_s01_common::supported;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// (source, recipient, amount) of each damage event this turn.
fn damage_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}

fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

/// Casts Forked Bolt dividing 1 damage to each of two targets.
fn forked_bolt(t: &mut TestGame, p: PlayerId, a: Entity, b: Entity) -> ObjectId {
    supported("Forked Bolt");
    t.answer(p, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    lands_for_cost(t, p, "Forked Bolt");
    let fb = t.hand(p, "Forked Bolt");
    t.cast(p, fb).targets(&[a, b]).go()
}

// ---------------------------------------------------------------------------------------
// Tamanoa
// ---------------------------------------------------------------------------------------

#[test]
fn tamanoa_ignores_a_creature_card_dealing_damage_from_a_graveyard() {
    cr!("603.10", "109.1");
    ruling!(
        "Tamanoa",
        "A noncreature source is a source that doesn’t have the type creature. If a creature card that’s not on the battlefield deals damage (for example, a cycled Gempalm Incinerator), Tamanoa’s ability won’t trigger."
    );
    supported("Tamanoa");
    supported("Gempalm Incinerator");
    let mut t = TestGame::new(2);
    let tam = t.battlefield(P0, "Tamanoa");
    t.battlefield(P0, "Goblin Piker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gempalm = t.hand(P0, "Gempalm Incinerator");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.activate(P0, gempalm, 0, &[o(bears)]).expect("cycle");
    t.resolve_all();
    assert!(!t.on_battlefield(bears) || damage_on(&t, bears) == 1);
    assert_eq!(damage_events(&t).len(), 1);
    assert_eq!(triggers_of(&t, tam), 0);
    assert_eq!(t.life(P0), 20);
    // A noncreature source (Shock) does trigger it.
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn tamanoa_triggers_for_any_recipient_but_you_can_lose_first() {
    cr!("704.5a", "117.5");
    ruling!(
        "Tamanoa",
        "Tamanoa’s ability triggers no matter who the recipient of the damage is: another player, a creature, or even you. If a noncreature source you control deals damage to you that drops your life total to 0 or less, you’ll lose the game before Tamanoa’s ability can resolve."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tamanoa");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Shock", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    cast_new(&mut t, P0, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // At 2 life, Shocking yourself: you lose before the trigger resolves.
    t.g.players[0].life = 2;
    cast_new(&mut t, P0, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert!(t.has_lost(P0));
}

// ---------------------------------------------------------------------------------------
// Blaze Commando
// ---------------------------------------------------------------------------------------

fn soldiers(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s01_common::tokens(t, p).len()
}

#[test]
fn blaze_commando_triggers_once_per_deals_in_the_spell() {
    cr!("603.2c", "120.2");
    ruling!(
        "Blaze Commando",
        "Blaze Commando’s ability triggers each time an instant or sorcery spell you control deals damage (or, put another way, the number of times the word “deals” appears in its instructions), no matter how much damage is dealt or how many players or permanents are dealt damage."
    );
    supported("Blaze Commando");
    supported("Punish the Enemy");
    let mut t = TestGame::new(2);
    let bc = t.battlefield(P0, "Blaze Commando");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Punish the Enemy: "deals 3 damage to target player or planeswalker and 3 damage to
    // target creature": one trigger, two Soldiers.
    cast_new(&mut t, P0, "Punish the Enemy", &[pl(P1), o(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(bears));
    assert_eq!(triggers_of(&t, bc), 1);
    assert_eq!(soldiers(&t, P0), 2);
    // Pyroclasm hitting several creatures: one trigger.
    let mut t = TestGame::new(2);
    let bc = t.battlefield(P0, "Blaze Commando");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, bc), 1);
}

#[test]
fn blaze_commando_ignores_spells_that_make_other_objects_deal_damage() {
    cr!("603.2c", "120.1");
    ruling!(
        "Blaze Commando",
        "Some instant or sorcery spells cause other objects to deal damage, rather than deal damage themselves. Such spells don’t cause Blaze Commando to trigger."
    );
    supported("Rabid Bite");
    let mut t = TestGame::new(2);
    let bc = t.battlefield(P0, "Blaze Commando");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Rabid Bite", &[o(bc), o(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(triggers_of(&t, bc), 0);
}

// ---------------------------------------------------------------------------------------
// Pyromancer's Swath
// ---------------------------------------------------------------------------------------

#[test]
fn pyromancers_swath_doesnt_apply_to_prevented_or_zero_damage() {
    cr!("614.1a", "615.1", "120.8");
    ruling!(
        "Pyromancer's Swath",
        "If all damage from a source is prevented (because of Pay No Heed, for example) or if a source would deal 0 damage, the effect of Pyromancer’s Swath will not apply as that source is no longer dealing damage."
    );
    supported("Pyromancer's Swath");
    supported("Shielded Passage");
    supported("Blaze");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyromancer's Swath");
    let giant = t.battlefield(P1, "Hill Giant");
    // P1 prevents all damage to the Giant, choosing prevention first.
    cast_new(&mut t, P1, "Shielded Passage", &[o(giant)]);
    t.resolve_all();
    t.answer(P1, DecisionKind::Replacement, Answer::Index(0));
    t.answer(P1, DecisionKind::Replacement, Answer::Index(0));
    cast_new(&mut t, P0, "Shock", &[o(giant)]);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 0);
    assert!(damage_events(&t).is_empty());
    // Blaze for X = 0: no damage, so no +2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyromancer's Swath");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Blaze");
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(0).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_on(&t, bears), 0);
}

#[test]
fn pyromancers_swath_adds_two_to_each_recipient_and_keeps_source_and_recipient() {
    cr!("614.1a", "120.3");
    ruling!(
        "Pyromancer's Swath",
        "If damage is being dealt to multiple creatures and/or players at the same time, the damage being dealt to each one is increased by 2."
    );
    ruling!(
        "Pyromancer's Swath",
        "Pyromancer’s Swath doesn’t change the source or recipient of the damage."
    );
    let mut t = TestGame::new(2);
    let swath = t.battlefield(P0, "Pyromancer's Swath");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Wall of Stone");
    // Pyroclasm: 4 to each creature.
    let spell = cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert_eq!(damage_on(&t, b), 4);
    let evs = damage_events(&t);
    assert_eq!(evs.len(), 2);
    for (src, _, amount) in &evs {
        assert_eq!(*amount, 4);
        assert_eq!(t.g.current(*src), t.g.current(spell));
        assert_ne!(*src, swath);
    }
    let mut targets: Vec<Entity> = evs.iter().map(|e| e.1).collect();
    targets.sort_by_key(|e| format!("{e:?}"));
    let mut want = vec![o(a), o(b)];
    want.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(targets, want);
    // Forked Bolt divided 1 / 1 between a player and a creature: 3 each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyromancer's Swath");
    let w = t.battlefield(P1, "Wall of Stone");
    forked_bolt(&mut t, P0, pl(P1), o(w));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(damage_on(&t, w), 3);
}

// ---------------------------------------------------------------------------------------
// Taii Wakeen, Perfect Shot
// ---------------------------------------------------------------------------------------

/// Taii Wakeen with its "{X}, {T}" ability activated for `x`.
fn taii(t: &mut TestGame, x: i64) -> ObjectId {
    supported("Taii Wakeen, Perfect Shot");
    let taii = t.battlefield(P0, "Taii Wakeen, Perfect Shot");
    t.lands(P0, "Wastes", x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.activate(P0, taii, 0, &[]).expect("taii");
    t.resolve_all();
    taii
}

#[test]
fn taii_wakeen_all_damage_prevented_means_no_extra_damage() {
    cr!("616.1", "615.1");
    ruling!(
        "Taii Wakeen, Perfect Shot",
        "If another effect modifies how much damage a source would deal, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Taii Wakeen’s effect no longer applies."
    );
    let mut t = TestGame::new(2);
    taii(&mut t, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P1, "Shielded Passage", &[o(giant)]);
    t.resolve_all();
    let from = t.asked().len();
    t.answer(P1, DecisionKind::Replacement, Answer::Index(0));
    t.answer(P1, DecisionKind::Replacement, Answer::Index(0));
    cast_new(&mut t, P0, "Shock", &[o(giant)]);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 0);
    // The permanent's controller was the one choosing (if there was a choice).
    for (p, d) in &t.asked()[from..] {
        if matches!(d, mtg_engine::decision::Decision::ChooseReplacement { .. }) {
            assert_eq!(*p, P1);
        }
    }
}

#[test]
fn taii_wakeen_checks_damage_dealt_not_damage_marked() {
    cr!("120.3e", "603.2");
    ruling!(
        "Taii Wakeen, Perfect Shot",
        "Taii Wakeen’s first ability doesn’t care about any damage already marked on a creature. It just cares about whether the noncombat damage dealt to a creature by a source you control was equal to that creature’s toughness."
    );
    supported("Taii Wakeen, Perfect Shot");
    // Hill Giant (3/3) with 1 damage marked: Shock (2) kills it but isn't equal to its
    // toughness: no draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Taii Wakeen, Perfect Shot");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].damage = 1;
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Shock", &[o(giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.hand_size(P0), hand);
    // With 2 damage marked, Lightning Bolt (3) equals its toughness: draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Taii Wakeen, Perfect Shot");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].damage = 2;
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Lightning Bolt", &[o(giant)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn taii_wakeens_extra_damage_is_dealt_by_the_original_source() {
    cr!("614.1a", "120.3");
    ruling!(
        "Taii Wakeen, Perfect Shot",
        "The additional X damage is dealt by the same source as the original source of damage. The damage isn’t dealt by Taii Wakeen unless Taii Wakeen is the original source of damage."
    );
    let mut t = TestGame::new(2);
    let taii = taii(&mut t, 2);
    let shock = cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    let evs = damage_events(&t);
    assert_eq!(evs.len(), 1);
    assert_eq!(t.g.current(evs[0].0), t.g.current(shock));
    assert_ne!(evs[0].0, taii);
    assert_eq!(evs[0].2, 4);
}

// ---------------------------------------------------------------------------------------
// Torture Pit, Aether Revolt
// ---------------------------------------------------------------------------------------

/// Spiked Corridor // Torture Pit on `p`'s battlefield with Torture Pit unlocked.
fn torture_pit(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Spiked Corridor // Torture Pit");
    let room = t.battlefield(p, "Spiked Corridor // Torture Pit");
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, p));
    t.g.recompute();
    t.settle();
    t.g.current(room)
}

#[test]
fn torture_pit_doesnt_apply_once_all_the_damage_is_prevented() {
    cr!("616.1", "615.1");
    ruling!(
        "Spiked Corridor // Torture Pit",
        "If another effect modifies how much noncombat damage the source would deal, including preventing some of it, the player being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Torture Pit's effect no longer applies."
    );
    supported("Healing Salve");
    // P1 prevents the next 3 damage. Prevention first: Shock's 2 is all prevented: 0.
    // Torture Pit first: 4, 3 of it prevented: 1.
    for (choice, life) in [(1usize, 20), (0, 19)] {
        let mut t = TestGame::new(2);
        torture_pit(&mut t, P0);
        t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
        t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
        cast_new(&mut t, P1, "Healing Salve", &[pl(P1)]);
        t.resolve_all();
        t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
        let from = t.asked().len();
        t.answer(P1, DecisionKind::Replacement, Answer::Index(choice));
        cast_new(&mut t, P0, "Shock", &[pl(P1)]);
        t.resolve_all();
        let asked: Vec<_> = t.asked()[from..]
            .iter()
            .filter_map(|(p, d)| match d {
                mtg_engine::decision::Decision::ChooseReplacement { options } => {
                    Some((*p, options.clone()))
                }
                _ => None,
            })
            .collect();
        assert_eq!(asked.len(), 1, "{asked:?}");
        assert_eq!(asked[0].0, P1);
        assert_eq!(t.life(P1), life, "{asked:?}");
    }
}

#[test]
fn torture_pit_adds_two_after_dividing_and_the_source_deals_it() {
    cr!("614.1a", "601.2d", "120.3");
    ruling!(
        "Spiked Corridor // Torture Pit",
        "If noncombat damage dealt by a source you control is being divided or assigned among multiple players, divide the original amount before adding 2."
    );
    ruling!(
        "Spiked Corridor // Torture Pit",
        "The additional 2 damage is dealt by the same source as the original source of damage. The additional damage isn't dealt by Torture Pit unless Torture Pit is the original source of damage."
    );
    let mut t = TestGame::new(3);
    let room = torture_pit(&mut t, P0);
    let fb = forked_bolt(&mut t, P0, pl(P1), pl(P2));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 17);
    for (src, _, amount) in damage_events(&t) {
        assert_eq!(amount, 3);
        assert_eq!(t.g.current(src), t.g.current(fb));
        assert_ne!(src, room);
    }
}

#[test]
fn aether_revolt_adds_two_after_dividing() {
    cr!("614.1a", "601.2d");
    ruling!(
        "Aether Revolt",
        "If noncombat damage dealt by a source you control is being divided or assigned among multiple opponents and/or permanents opponents control, divide the original amount before adding 2."
    );
    supported("Aether Revolt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aether Revolt");
    // Revolt: a permanent left the battlefield under P0's control this turn.
    let land = t.battlefield(P0, "Island");
    bounce(&mut t, land);
    let w = t.battlefield(P1, "Wall of Stone");
    forked_bolt(&mut t, P0, pl(P1), o(w));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(damage_on(&t, w), 3);
}

// ---------------------------------------------------------------------------------------
// Chandra's Spitfire, Wildfire Elemental, Chandra's Pyreling
// ---------------------------------------------------------------------------------------

#[test]
fn opponent_damage_triggers_once_per_opponent_in_one_event() {
    cr!("603.2c", "120.3");
    ruling!(
        "Chandra's Spitfire",
        "In a multiplayer game, if a source deals damage to multiple opponents at the same time, the last ability of Chandra's Spitfire will trigger as many times as there are opponents who were dealt damage."
    );
    ruling!(
        "Wildfire Elemental",
        "In a multiplayer game, if a source deals damage to multiple opponents at the same time, the last ability of Wildfire Elemental will trigger that many times."
    );
    ruling!(
        "Chandra's Pyreling",
        "The ability of Chandra's Pyreling triggers just once per event for each opponent who is dealt noncombat damage by a source you control, regardless of how much damage that player is dealt."
    );
    supported("Chandra's Spitfire");
    supported("Wildfire Elemental");
    supported("Chandra's Pyreling");
    supported("Boltwave");
    // Four players: Boltwave deals 3 damage to each opponent: three triggers each.
    let mut t = TestGame::new(4);
    let sp = t.battlefield(P0, "Chandra's Spitfire");
    let we = t.battlefield(P0, "Wildfire Elemental");
    let py = t.battlefield(P0, "Chandra's Pyreling");
    cast_new(&mut t, P0, "Boltwave", &[]);
    t.resolve_all();
    for q in [P1, P2, P3] {
        assert_eq!(t.life(q), 17);
    }
    assert_eq!(triggers_of(&t, sp), 3);
    assert_eq!(triggers_of(&t, we), 3);
    assert_eq!(triggers_of(&t, py), 3);
    // Spitfire 1/3 + 9/+0 and Wildfire's +3/+0.
    assert_eq!(t.pt(sp).0, 1 + 9 + 3);
    assert_eq!(t.pt(py).0, 1 + 3 + 3);
}

#[test]
fn opponent_damage_triggers_once_per_event_regardless_of_amount() {
    cr!("603.2c");
    ruling!(
        "Chandra's Spitfire",
        "The last ability of Chandra's Spitfire triggers just once for each event in which an opponent is dealt noncombat damage, regardless of how much damage that player is dealt."
    );
    ruling!(
        "Wildfire Elemental",
        "The last ability of Wildfire Elemental triggers once for each event in which an opponent is dealt noncombat damage, regardless of how much damage that player is dealt."
    );
    let mut t = TestGame::new(2);
    let sp = t.battlefield(P0, "Chandra's Spitfire");
    let we = t.battlefield(P0, "Wildfire Elemental");
    cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, sp), 1);
    assert_eq!(triggers_of(&t, we), 1);
    assert_eq!(t.pt(sp), (5, 3));
    // A second event: a second trigger each.
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, sp), 2);
    assert_eq!(triggers_of(&t, we), 2);
}

#[test]
fn wildfire_elemental_affects_only_creatures_you_control_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Wildfire Elemental",
        "Wildfire Elemental's triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +1/+0."
    );
    let mut t = TestGame::new(2);
    let we = t.battlefield(P0, "Wildfire Elemental");
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.settle();
    // Bears arrive in response: they get it.
    let early = t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(we), (4, 3));
    assert_eq!(t.pt(early), (3, 2));
    let late = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(late), (2, 2));
}
