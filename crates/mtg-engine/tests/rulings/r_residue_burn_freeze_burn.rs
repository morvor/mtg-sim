//! Item `rulings-residue-burn-freeze` — the interaction rulings of batch P009 (burn that
//! hits players, planeswalkers and each player): prevention and replacement effects around
//! damage (CR 615, 614), the source of damage (CR 120.3, 609.7), last known information
//! (CR 608.2h), conditions checked as a spell resolves, costs (CR 601.2f, 118.7), casting
//! spells during resolution (CR 608.2g), ward (CR 702.21), and loops (CR 104.4b).

use crate::r_residue_burn_freeze_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 activates Vengeful Archon's ability with X = `x` targeting `target`; it resolves.
fn archon_shield(t: &mut TestGame, archon: ObjectId, x: i64, target: Entity) {
    t.lands(P0, "Wastes", x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.activate(P0, archon, 0, &[target])
        .expect("Vengeful Archon");
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// Vengeful Archon: a prevention effect with an additional effect, not a redirection
// ---------------------------------------------------------------------------------------

#[test]
fn vengeful_archon_is_the_source_of_noncombat_damage() {
    cr!("615.7", "120.3", "510.2", "702.15b");
    ruling!(
        "Vengeful Archon",
        "The effect of Vengeful Archon’s ability is not a redirection effect. If it prevents damage, Vengeful Archon (not the source of that damage) deals damage to the targeted player as part of that prevention effect. Vengeful Archon is the source of the new damage, so the characteristics of the original source (such as its color, or whether it had lifelink or deathtouch) don’t affect this damage. The new damage is not combat damage, even if the prevented damage was."
    );
    supported("Vengeful Archon");
    // P1 attacks P0 with Vampire Nighthawk (black, lifelink, deathtouch); P0's shield of 2
    // targets P1. The Nighthawk's combat damage is prevented: P1 gains no life; the
    // Archon (white) deals 2 noncombat damage to P1.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let archon = t.battlefield(P0, "Vengeful Archon");
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.g.turn.priority = Some(P0);
    t.activate(P0, archon, 0, &[pl(P1)])
        .expect("Vengeful Archon");
    t.resolve_all();
    let from = t.g.turn_events.len();
    attack_with(&mut t, &[(hawk, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 18, "no lifelink: the Nighthawk dealt no damage");
    let dealt = damage_events(&t, from);
    assert_eq!(dealt, vec![(archon, pl(P1), 2, false)], "{dealt:?}");
    assert!(t.obj_now(archon).chars.colors.contains(Color::White));
}

#[test]
fn vengeful_archon_prevents_the_next_damage_from_any_source() {
    cr!("615.7", "615.1");
    ruling!(
        "Vengeful Archon",
        "Vengeful Archon’s ability doesn’t have you choose a source of damage. It will apply to the next X damage you would be dealt that turn, regardless of where that damage comes from. It doesn’t matter whether the damage would all be dealt at the same time. For example, if X is 4 and you’d be dealt 3 damage by Lightning Bolt, that 3 damage is prevented and Vengeful Archon deals 3 damage to the targeted player. The prevention effect will still apply to the next 1 damage you’d be dealt that turn."
    );
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Vengeful Archon");
    archon_shield(&mut t, archon, 4, pl(P1));
    cast_new(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
    // A different source: 1 of the Shock's 2 is prevented, the Archon deals 1.
    cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (19, 16));
    // The shield is used up.
    cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 16));
}

#[test]
fn vengeful_archon_target_isnt_checked_after_it_resolves() {
    cr!("608.2b", "615.7", "800.4a");
    ruling!(
        "Vengeful Archon",
        "Whether the targeted player is still a legal target is no longer checked after Vengeful Archon’s ability resolves. For example, if a player targeted by Vengeful Archon’s ability puts Leyline of Sanctity (which says “You can’t be the target of spells or abilities your opponents control”) onto the battlefield after the ability resolves but before it prevents damage, the ability will still prevent damage and still deal damage to that player. If Vengeful Archon can’t deal damage to the targeted player (because the player is no longer in the game in a multiplayer game, for example), it will still prevent damage. It just won’t deal any damage itself."
    );
    ruling!(
        "Vengeful Archon",
        "The only target of Vengeful Archon’s ability is the player it may deal damage to. You choose that target as you activate the ability, not at the time it prevents damage."
    );
    // P1 gets Leyline of Sanctity after the ability resolved.
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Vengeful Archon");
    let src = t.battlefield(P1, "Grizzly Bears");
    archon_shield(&mut t, archon, 3, pl(P1));
    player_hexproof(&mut t, P1);
    let from = t.asked().len();
    hit_player(&mut t, src, P0, 2);
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    // No target is chosen as it prevents damage.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    // The targeted player left the game: the damage is still prevented.
    let mut t = TestGame::new(3);
    let archon = t.battlefield(P0, "Vengeful Archon");
    let src = t.battlefield(P1, "Grizzly Bears");
    archon_shield(&mut t, archon, 3, pl(P2));
    t.g.lose_game(P2);
    t.settle();
    hit_player(&mut t, src, P0, 2);
    assert_eq!(t.life(P0), 20);
    assert!(!t.g.turn_events.iter().any(|e| matches!(
        e,
        Event::Damage { source, .. } if *source == archon
    )));
}

// ---------------------------------------------------------------------------------------
// Damage a player would be dealt, and what isn't
// ---------------------------------------------------------------------------------------

#[test]
fn sarkhan_the_mad_damages_himself_past_your_prevention_effects() {
    cr!("120.3c", "606.3", "615.1");
    ruling!(
        "Sarkhan the Mad",
        "The first ability causes Sarkhan to deal damage directly to himself, resulting in that many loyalty counters being removed from him. The damage would never have been dealt to you, so prevention effects that would prevent damage from being dealt to you won't affect it."
    );
    supported("Sarkhan the Mad");
    // P0's Vengeful Archon shield of 5 (targeting P1) stays: Sarkhan's damage to himself
    // was never going to be dealt to P0.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let archon = t.battlefield(P0, "Vengeful Archon");
    archon_shield(&mut t, archon, 5, pl(P1));
    let s = pw(&mut t, P0, "Sarkhan the Mad", 7);
    t.library_top(P0, "Hill Giant");
    loyalty(&mut t, P0, s, 0, &[]);
    t.resolve_all();
    assert_eq!(t.counters(s, "loyalty"), 3, "7, then 4 damage");
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // The shield is still there for damage to P0.
    let src = t.battlefield(P1, "Grizzly Bears");
    hit_player(&mut t, src, P0, 2);
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
}

#[test]
fn flames_of_the_blood_hand_stops_only_the_targets_life_gain() {
    cr!("615.12", "614.1a", "119.7");
    ruling!(
        "Flames of the Blood Hand",
        "The damage is unpreventable, and the targeted player’s life gain that turn is replaced with nothing."
    );
    ruling!(
        "Flames of the Blood Hand",
        "The replacement effect applies only to the target player, not to a permanent or player the damage might be redirected to."
    );
    supported("Flames of the Blood Hand");
    // P1 has a Vengeful Archon shield of 3 (targeting P2): Flames' 4 damage can't be
    // prevented, so the Archon deals nothing.
    let mut t = TestGame::new(3);
    let archon = t.battlefield(P1, "Vengeful Archon");
    t.lands(P1, "Wastes", 3);
    t.answer(P1, DecisionKind::X, Answer::Number(3));
    t.g.turn.priority = Some(P1);
    t.activate(P1, archon, 0, &[pl(P2)])
        .expect("Vengeful Archon");
    t.resolve_all();
    cast_new(&mut t, P0, "Flames of the Blood Hand", &[pl(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (16, 20));
    // P1 gains no life this turn; P2 does.
    crate::r_p009_common::gain(&mut t, P1, 3);
    crate::r_p009_common::gain(&mut t, P2, 3);
    assert_eq!((t.life(P1), t.life(P2)), (16, 23));
}

#[test]
fn flames_of_the_blood_hand_redirected_damage_doesnt_stop_that_players_life_gain() {
    cr!("614.1a", "609.7", "119.7");
    ruling!(
        "Flames of the Blood Hand",
        "The replacement effect applies only to the target player, not to a permanent or player the damage might be redirected to."
    );
    supported("Harm's Way");
    // P1 casts Harm's Way in response, choosing Flames as the source: 2 of its damage is
    // dealt to P2 instead. The damage can't be prevented, but it can be redirected
    // (CR 615.12 is about prevention). P2, dealt the redirected damage, can still gain
    // life; P1, the target, can't.
    let mut t = TestGame::new(3);
    let flames = cast_new(&mut t, P0, "Flames of the Blood Hand", &[pl(P1)]);
    t.answer_choose(P1, &[obj(flames)]);
    cast_new(&mut t, P1, "Harm's Way", &[pl(P2)]);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
    crate::r_p009_common::gain(&mut t, P1, 3);
    crate::r_p009_common::gain(&mut t, P2, 3);
    assert_eq!((t.life(P1), t.life(P2)), (18, 21));
}

#[test]
fn chandra_roaring_flames_emblem_is_a_colorless_source() {
    cr!("114.1", "114.2", "105.2c");
    ruling!(
        "Chandra, Fire of Kaladesh // Chandra, Roaring Flame",
        "The emblem created by Chandra, Roaring Flame is colorless. The damage it deals is from a colorless source."
    );
    supported("Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let c = t.battlefield(P0, "Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    assert!(mtg_engine::dfc::transform(&mut t.g, c));
    let c = t.g.current(c);
    t.g.objects[c.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    loyalty(&mut t, P0, c, 2, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    let emblem =
        t.g.objects
            .iter()
            .find(|o| o.kind == ObjKind::Emblem && o.zone == Zone::Command && o.owner == P1)
            .map(|o| o.id)
            .expect("P1's emblem");
    assert!(t.obj_now(emblem).chars.colors.is_colorless());
    // At P1's upkeep it deals 3 damage to P1, from a colorless source.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
    let src = t.g.turn_events[..]
        .iter()
        .find_map(|e| match e {
            Event::Damage { source, target, .. } if *target == pl(P1) => Some(*source),
            _ => None,
        })
        .expect("emblem damage");
    assert_eq!(src, emblem);
    assert!(t.g.obj(src).chars.colors.is_colorless());
}

// ---------------------------------------------------------------------------------------
// Last known information, and conditions checked as the spell resolves
// ---------------------------------------------------------------------------------------

#[test]
fn corpse_explosion_uses_the_power_the_card_had_in_the_graveyard() {
    cr!("608.2h", "601.2h", "604.3");
    ruling!(
        "Corpse Explosion",
        "Use the card's power as it last existed in the graveyard to determine how much damage is dealt."
    );
    supported("Corpse Explosion");
    // Tarmogoyf in P0's graveyard with a Shock: two card types, a 2/3. Exiled, only the
    // instant is left (it would be a 1/2), but the damage is 2.
    let mut t = TestGame::new(2);
    let goyf = t.graveyard(P0, "Tarmogoyf");
    t.graveyard(P0, "Shock");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(goyf)]);
    cast_new(&mut t, P0, "Corpse Explosion", &[]);
    t.resolve_all();
    assert_eq!(t.zone(goyf), Zone::Exile);
    assert!(!t.on_battlefield(bears), "2 damage");
    assert_eq!(dmg(&t, giant), 2);
}

#[test]
fn concussive_bolt_checks_metalcraft_only_as_it_resolves() {
    cr!("608.2h", "207.2c", "509.1a");
    ruling!(
        "Concussive Bolt",
        "The number of artifacts you control is checked only once: when Concussive Bolt resolves. If you don’t control three or more artifacts at that time, creatures will be able to block that turn, even if you gain control of more artifacts later in the turn."
    );
    supported("Concussive Bolt");
    for (before, after, can_block) in [(2, 3, true), (3, 0, false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let mut stones = vec![];
        for _ in 0..before {
            stones.push(t.battlefield(P0, "Ornithopter"));
        }
        let attacker = t.battlefield(P0, "Grizzly Bears");
        let blocker = t.battlefield(P1, "Hill Giant");
        cast_new(&mut t, P0, "Concussive Bolt", &[pl(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), 16);
        if after > before {
            for _ in before..after {
                t.battlefield(P0, "Ornithopter");
            }
        } else {
            for s in stones {
                crate::r_p009_common::kill(&mut t, s);
            }
        }
        attack_with(&mut t, &[(attacker, pl(P1))]);
        block_and_finish(&mut t, P1, &[(blocker, attacker)]);
        assert_eq!(
            !t.on_battlefield(attacker),
            can_block,
            "before {before}, after {after}"
        );
    }
}

#[test]
fn antagonism_looks_back_over_the_whole_turn() {
    cr!("608.2h", "603.4");
    ruling!(
        "Antagonism",
        "It looks back over the entire turn even if this card was not on the battlefield at the time."
    );
    supported("Antagonism");
    // P0's turn: P1 is dealt damage, then Antagonism enters. At P0's end step it doesn't
    // deal damage to P0 (an opponent of P0 was dealt damage this turn).
    for dealt_before in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        if dealt_before {
            cast_new(&mut t, P0, "Shock", &[pl(P1)]);
            t.resolve_all();
        }
        t.battlefield(P0, "Antagonism");
        t.advance_to(P0, Step::End);
        t.settle();
        t.resolve_all();
        assert_eq!(t.life(P0), if dealt_before { 20 } else { 18 });
    }
}

// ---------------------------------------------------------------------------------------
// Costs and mana value
// ---------------------------------------------------------------------------------------

#[test]
fn march_of_wretched_sorrow_mana_value_ignores_the_reduction() {
    cr!("202.3e", "601.2f", "118.7");
    ruling!(
        "March of Wretched Sorrow",
        "The mana value of March of Wretched Sorrow while it's on the stack is the value chosen for X plus 1, no matter how much mana you actually paid to cast it."
    );
    ruling!(
        "March of Wretched Sorrow",
        "You may exile more cards than necessary for March of Wretched Sorrow's first ability, but you can't reduce the mana it costs to less than {B} this way."
    );
    supported("March of Wretched Sorrow");
    // X = 3; three black cards exiled (a reduction of {6} against {3}): it costs {B}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Craw Wurm");
    let black: Vec<_> = (0..3).map(|_| t.hand(P0, "Dusk Legion Zealot")).collect();
    let swamp = t.lands(P0, "Swamp", 1)[0];
    let march = t.hand(P0, "March of Wretched Sorrow");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(3));
    t.answer_choose(P0, &black.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer_targets(P0, &[obj(giant)]);
    let spell = t.cast(P0, march).x(3).go();
    assert!(is_tapped(&t, swamp));
    assert_eq!(t.g.mana_value_of(spell), 4);
    for c in &black {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    t.resolve_all();
    assert_eq!(dmg(&t, giant), 3);
    assert_eq!(t.life(P0), 23);
    // With no land, it can't be cast even with more cards exiled.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Craw Wurm");
    let black: Vec<_> = (0..3).map(|_| t.hand(P0, "Dusk Legion Zealot")).collect();
    let march = t.hand(P0, "March of Wretched Sorrow");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(3));
    t.answer_choose(P0, &black.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer_targets(P0, &[obj(giant)]);
    assert!(t.cast(P0, march).x(1).try_go().is_err());
    for c in &black {
        assert_eq!(t.zone(*c), Zone::Hand(P0));
    }
}

#[test]
fn invasion_of_the_giants_reduction_applies_after_increases() {
    cr!("601.2f", "202.3", "714.2b");
    ruling!(
        "Invasion of the Giants",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of the effect of the chapter III ability). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Invasion of the Giants");
    supported("Burdened Stoneback");
    // Sphere of Resistance (+{1}) and chapter III ({2} less): Burdened Stoneback ({1}{W})
    // costs {1}{W} + {1} − {2} = {W}. Its mana value is still 2.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Sphere of Resistance");
    let saga = t.battlefield(P0, "Invasion of the Giants");
    crate::r_s29_common::put_counters(&mut t, saga, "lore", 3);
    t.settle();
    t.resolve_all();
    t.lands(P0, "Plains", 1);
    let giant = t.hand(P0, "Burdened Stoneback");
    t.cast(P0, giant).go();
    assert_eq!(t.g.mana_value_of(giant), 2);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(giant)));
}

// ---------------------------------------------------------------------------------------
// Spells cast during a resolution, and lands played from exile
// ---------------------------------------------------------------------------------------

#[test]
fn chandra_pyromaster_copies_ignore_timing_but_not_other_restrictions() {
    cr!("608.2g", "707.12", "307.1");
    ruling!(
        "Chandra, Pyromaster",
        "While casting the copies created by the third ability, timing restrictions based on the card's type (such as creature or sorcery) are ignored. Other restrictions, such as \"Cast [this spell] only during combat,\" are not."
    );
    supported("Chandra, Pyromaster");
    // A sorcery (Divination: draw two) is cast three times while the ability resolves.
    // Angelic Favor ("Cast this spell only during combat.") can't be cast in a main phase.
    for (name, cast) in [("Divination", true), ("Angelic Favor", false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let c = pw(&mut t, P0, "Chandra, Pyromaster", 7);
        let mut lib = vec![name];
        lib.extend(["Grizzly Bears"; 12]);
        stack_library(&mut t, P0, &lib);
        let card = t.g.player(P0).library.last().copied().unwrap();
        t.answer_choose(P0, &[obj(card)]);
        for _ in 0..3 {
            t.answer_yes(P0, true);
        }
        let hand = t.hand_size(P0);
        loyalty(&mut t, P0, c, 2, &[]);
        t.resolve_all();
        if cast {
            assert_eq!(t.hand_size(P0), hand + 6, "{name}");
        } else {
            assert_eq!(t.hand_size(P0), hand, "{name}");
            assert!(tokens(&t, P0).is_empty(), "{name}: no Angel");
        }
    }
}

#[test]
fn nahiris_warcrafting_exiled_land_needs_a_land_play() {
    cr!("305.2", "305.2b", "120.4a");
    ruling!(
        "Nahiri's Warcrafting",
        "Unless an effect allows you to play additional lands that turn, you can play an exiled land card only if you haven't played a land yet that turn."
    );
    supported("Nahiri's Warcrafting");
    for played_one in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        // Grizzly Bears (2/2) is dealt 5: 3 excess, so P0 looks at three cards.
        let bears = t.battlefield(P1, "Grizzly Bears");
        stack_library(&mut t, P0, &["Forest", "Shock", "Shock"]);
        let forest = t.g.player(P0).library.last().copied().unwrap();
        if played_one {
            let land = t.hand(P0, "Mountain");
            t.play_land(P0, land).expect("land play");
        }
        t.answer_choose(P0, &[obj(forest)]);
        cast_new(&mut t, P0, "Nahiri's Warcrafting", &[obj(bears)]);
        t.resolve_all();
        let exiled = t.g.current(forest);
        assert_eq!(t.zone(exiled), Zone::Exile, "played_one {played_one}");
        assert_eq!(
            t.play_land(P0, exiled).is_ok(),
            !played_one,
            "played_one {played_one}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Triggers, ward and loops
// ---------------------------------------------------------------------------------------

#[test]
fn incinerator_of_the_guilty_can_collect_evidence_zero() {
    cr!("701.59a", "603.2", "603.12");
    ruling!(
        "Incinerator of the Guilty",
        "You may collect evidence 0, but you would be a lazy detective with no crispy culprits. Nevertheless, if you choose to do so, abilities that trigger \"whenever you collect evidence\", such as those of Surveillance Monitor and Evidence Examiner, will still trigger."
    );
    supported("Incinerator of the Guilty");
    supported("Surveillance Monitor");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let inc = t.battlefield(P0, "Incinerator of the Guilty");
    t.battlefield(P0, "Surveillance Monitor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    attack_with(&mut t, &[(inc, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.on_battlefield(bears), "X = 0");
    let thopters = tokens(&t, P0);
    assert_eq!(thopters.len(), 1, "{}", t.dump_log());
}

#[test]
fn lithomantic_barrage_isnt_countered_by_ward() {
    cr!("702.21a", "101.2");
    ruling!(
        "Lithomantic Barrage",
        "“This spell can’t be countered” means it can’t be countered even by the ward keyword ability. If you cast Lithomantic Barrage targeting a creature or planeswalker with ward, you may still pay the ward cost if you want to, but Lithomantic Barrage won’t be countered if you don’t."
    );
    supported("Lithomantic Barrage");
    supported("Armored Armadillo");
    // Armored Armadillo (white, 0/4, ward {1}): P0 doesn't pay; the 5 damage destroys it.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let dillo = t.battlefield(P1, "Armored Armadillo");
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Lithomantic Barrage", &[obj(dillo)]);
    t.resolve_all();
    assert!(!t.on_battlefield(dillo));
    assert!(t.in_graveyard(P0, "Lithomantic Barrage"));
}

#[test]
fn arcbond_damage_isnt_combat_damage() {
    cr!("510.2", "603.7", "120.2");
    ruling!(
        "Arcbond",
        "Damage dealt by the creature because of the ability Arcbond creates isn’t combat damage, even if it was combat damage that caused the ability to trigger."
    );
    supported("Arcbond");
    // P1's Hill Giant blocks P0's Arcbonded Craw Wurm: the Wurm is dealt 3 combat
    // damage, then deals 3 noncombat damage to each other creature and each player.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Arcbond", &[obj(wurm)]);
    t.resolve_all();
    attack_with(&mut t, &[(wurm, pl(P1))]);
    let from = t.g.turn_events.len();
    block_and_finish(&mut t, P1, &[(giant, wurm)]);
    t.resolve_all();
    let arc: Vec<_> = damage_events(&t, from)
        .into_iter()
        .filter(|d| d.0 == wurm && d.1 != obj(giant))
        .collect();
    assert!(!arc.is_empty());
    assert!(arc.iter().all(|d| !d.3), "{arc:?}");
    assert!(!t.on_battlefield(bears));
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
}

#[test]
fn two_arcbonds_loop_until_a_creature_dies_or_the_game_is_a_draw() {
    cr!("104.4b", "603.7c");
    ruling!(
        "Arcbond",
        "If two Arcbonds have resolved targeting different creatures, and damage is dealt to one of them, the delayed triggered ability will cause that creature to deal damage to each other creature and player. This will cause the delayed triggered ability from the other Arcbond to trigger. The second creature will then deal damage to each other creature and player. This will continue until one or both of the creatures die or the game ends. If this doesn’t happen (perhaps because both creatures have indestructible and damage dealt to the players is being prevented), and no player breaks the loop, the game will be a draw."
    );
    // Two Hill Giants (3/3), 1 damage to one: they deal 1 back and forth until the
    // first has 3 damage marked; its last trigger still resolves (lethal damage), then the
    // second's, after it died too: each player is dealt 6.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Arcbond", &[obj(a)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Arcbond", &[obj(b)]);
    t.resolve_all();
    hit(&mut t, b, obj(a), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!((t.life(P0), t.life(P1)), (14, 14));
    // Two Darksteel Myr (indestructible), and Solitary Confinement ("Prevent all damage
    // that would be dealt to you.") for each player: the loop never ends, and the game
    // is a draw. The damage marked on the Myr grows each time around, so the engine
    // doesn't recognize a repeated state: the draw comes at the action limit.
    interp!("growing-loops-run-to-the-limit");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            max_actions: 400,
            ..GameConfig::default()
        },
    );
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Darksteel Myr");
    let b = t.battlefield(P1, "Darksteel Myr");
    for p in [P0, P1] {
        t.battlefield(p, "Solitary Confinement");
    }
    cast_new(&mut t, P0, "Arcbond", &[obj(a)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Arcbond", &[obj(b)]);
    t.resolve_all();
    hit(&mut t, b, obj(a), 1);
    assert_eq!(t.g.run(), GameResult::Draw);
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    assert!(t.obj_now(a).damage > 20 && t.obj_now(b).damage > 20);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn blim_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b", "510.3a");
    ruling!(
        "Blim, Comedic Genius",
        "If the target permanent you control is an illegal target by the time Blim's ability tries to resolve, the ability doesn't resolve. The other player doesn't gain control of your permanent, and no one loses life or discards cards."
    );
    supported("Blim, Comedic Genius");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let blim = t.battlefield(P0, "Blim, Comedic Genius");
    let rock = t.battlefield(P0, "Ornithopter");
    // P1 already controls a permanent P0 owns, so "each player loses life and discards"
    // would do something.
    let stolen = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, stolen, P1);
    t.hand(P1, "Shock");
    t.answer_targets(P0, &[obj(rock)]);
    attack_with(&mut t, &[(blim, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    // The trigger is on the stack: the Ornithopter leaves.
    crate::r_p009_common::kill(&mut t, rock);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "only Blim's 4 combat damage");
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P1), 1);
}
