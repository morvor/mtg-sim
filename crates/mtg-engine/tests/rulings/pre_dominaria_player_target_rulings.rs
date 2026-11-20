//! Item `user-decisions-nov20`, part 5: rulings older than the Dominaria Oracle changes
//! (Eli Shiffrin, 2018-04-13: "Abilities that damage or prevent damage to "target player"
//! have been changed to "target player or planeswalker."" and "Abilities that read "target
//! creature or player" have been changed to "any target."") that say "a player" (or
//! "creatures and/or players") where the card now says "target player or planeswalker",
//! "target opponent or planeswalker" or "any target". Their "a player" parts give way to the
//! Oracle text (policy `pre-dominaria-player-target-rulings-give-way`, decided by the user
//! Nov 20); each test here shows the card's planeswalker target working, and cites the parts
//! of the ruling that still hold where the test exercises them with a planeswalker.

use crate::r_residue_burn_freeze_common::*;
use crate::r_s01_common::with_subtype;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

/// P1's planeswalker (Gideon Jura) with `n` loyalty counters.
fn walker(t: &mut TestGame, n: u32) -> ObjectId {
    pw(t, P1, "Gideon Jura", n)
}

fn loyalty_of(t: &TestGame, w: ObjectId) -> u32 {
    t.counters(w, "loyalty")
}

/// P1 casts Healing Salve: prevent the next 3 damage that would be dealt to `to`.
fn salve(t: &mut TestGame, to: Entity) {
    supported("Healing Salve");
    t.lands(P1, "Plains", 1);
    let s = t.hand(P1, "Healing Salve");
    t.g.turn.priority = Some(P1);
    t.cast(P1, s).modes(&[1]).target(to).go();
    t.resolve();
}

/// The special actions offered by effects that `p` could take now (Quenchable Fire's).
fn offers(t: &mut TestGame, p: PlayerId) -> Vec<Action> {
    t.g.recompute();
    mtg_engine::special_actions::available(&t.g, p)
        .into_iter()
        .filter(|a| matches!(a, Action::Special(SpecialAction::Offer { .. })))
        .collect()
}

#[test]
fn soul_of_shandalar_targets_a_planeswalker() {
    cr!("115.1", "602.2b", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!(
        "Soul of Shandalar",
        "but you can choose to not target any creatures"
    );
    supported("Soul of Shandalar");
    // From the battlefield, with the planeswalker's controller's creature as the second
    // target; then from the graveyard, with no creature.
    for from_graveyard in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let soul = if from_graveyard {
            t.graveyard(P0, "Soul of Shandalar")
        } else {
            t.battlefield(P0, "Soul of Shandalar")
        };
        let w = walker(&mut t, 6);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Mountain", 5);
        t.answer_targets(P0, &[obj(w)]);
        let creature: Vec<Entity> = if from_graveyard { vec![] } else { vec![obj(wurm)] };
        t.answer_targets(P0, &creature);
        t.activate(P0, soul, usize::from(from_graveyard), &[])
            .unwrap_or_else(|e| panic!("graveyard {from_graveyard}: {e:?}"));
        t.resolve_all();
        assert_eq!(loyalty_of(&t, w), 3, "graveyard {from_graveyard}");
        assert_eq!(dmg(&t, wurm), if from_graveyard { 0 } else { 3 });
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn chandra_pyromaster_targets_a_planeswalker_and_its_controllers_creature() {
    cr!("115.1", "606.3", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    supported("Chandra, Pyromaster");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let c = pw(&mut t, P0, "Chandra, Pyromaster", 4);
    let w = walker(&mut t, 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(w)]);
    t.answer_targets(P0, &[obj(bears)]);
    loyalty(&mut t, P0, c, 0, &[]);
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 5);
    assert_eq!(dmg(&t, bears), 1);
    assert!(!t.g.can_block_at_all(bears), "that creature can't block");
    assert_eq!(t.life(P1), 20);
}

#[test]
fn searing_blaze_targets_a_planeswalker_and_a_creature_its_controller_controls() {
    cr!("601.2c", "115.1", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!(
        "Searing Blaze",
        "If you can't (because there are no creatures on the battlefield, perhaps), then you can't cast the spell."
    );
    supported("Searing Blaze");
    // The planeswalker and its controller's creature.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let w = walker(&mut t, 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let blaze = t.hand(P0, "Searing Blaze");
    t.answer_targets(P0, &[obj(w)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, blaze).try_go().expect("Searing Blaze");
    t.resolve_all();
    assert_eq!((loyalty_of(&t, w), dmg(&t, bears), t.life(P1)), (5, 1, 20));
    // With the planeswalker targeted, a creature its controller doesn't control can't be
    // the second target; with no creature of P1's, the spell can't be cast.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let w = walker(&mut t, 6);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let blaze = t.hand(P0, "Searing Blaze");
    t.answer_targets(P0, &[obj(w)]);
    t.answer_targets(P0, &[obj(mine)]);
    assert!(t.cast(P0, blaze).try_go().is_err());
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn sarkhan_the_mad_dragons_deal_damage_to_a_planeswalker() {
    cr!("115.1", "120.3c", "606.3");
    interp!("pre-dominaria-player-target-rulings-give-way");
    supported("Sarkhan the Mad");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = pw(&mut t, P0, "Sarkhan the Mad", 7);
    t.battlefield(P0, "Shivan Dragon");
    t.battlefield(P0, "Shivan Dragon");
    let w = walker(&mut t, 12);
    loyalty(&mut t, P0, s, 2, &[obj(w)]);
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 2, "5 from each Dragon");
    assert_eq!(t.life(P1), 20);
}

#[test]
fn vengeful_archon_deals_the_prevented_damage_to_a_planeswalker() {
    cr!("615.7", "120.3c", "602.2b");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!(
        "Vengeful Archon",
        "You choose that target as you activate the ability, not at the time it prevents damage."
    );
    supported("Vengeful Archon");
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Vengeful Archon");
    let w = walker(&mut t, 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, archon, 0, &[obj(w)])
        .expect("Vengeful Archon");
    t.resolve_all();
    hit_player(&mut t, bears, P0, 2);
    assert_eq!(t.life(P0), 20, "prevented");
    assert_eq!(loyalty_of(&t, w), 4, "the Archon deals 2 to the planeswalker");
    assert_eq!(t.life(P1), 20);
}

#[test]
fn two_target_burn_targets_a_planeswalker_and_a_creature() {
    cr!("601.2c", "115.1", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    // (card, method, cast from the graveyard, the creature's slot comes first, damage to
    // the planeswalker, damage to the creature)
    let aftermath = CastMethod::Keyword(KeywordKind::Aftermath);
    let cases: [(&str, CastMethod, bool, bool, u32, u32); 4] = [
        ("Punish the Enemy", NORMAL, false, false, 3, 3),
        ("Cunning Strike", NORMAL, false, true, 2, 2),
        ("Hungry Flames", NORMAL, false, true, 2, 3),
        ("Insult // Injury", aftermath, true, true, 2, 2),
    ];
    for (name, method, from_gy, creature_first, to_walker, to_creature) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let w = walker(&mut t, 6);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.library_top(P0, "Forest");
        lands_for_cost(&mut t, P0, name);
        t.lands(P0, "Mountain", 2);
        let card = if from_gy {
            t.graveyard(P0, name)
        } else {
            t.hand(P0, name)
        };
        let order = if creature_first {
            [obj(wurm), obj(w)]
        } else {
            [obj(w), obj(wurm)]
        };
        for e in order {
            t.answer_targets(P0, &[e]);
        }
        let hand = t.hand_size(P0);
        t.cast(P0, card)
            .method(method)
            .try_go()
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        t.resolve_all();
        assert_eq!(loyalty_of(&t, w), 6 - to_walker, "{name}");
        assert_eq!(dmg(&t, wurm), to_creature, "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
        if name == "Cunning Strike" {
            assert_eq!(t.hand_size(P0), hand, "drew a card (the spell left the hand)");
        }
    }
}

#[test]
fn exploding_borders_damages_a_planeswalker() {
    cr!("115.1", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    // lands_for_cost pays {2}{R}{G} with Wastes, Wastes, Mountain, Forest: 2 basic land
    // types (no basic land card in the library to put onto the battlefield).
    let mut t = TestGame::new(2);
    let w = walker(&mut t, 6);
    cast_x(&mut t, P0, "Exploding Borders", NORMAL, None, &[obj(w)]);
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 4);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn bonfire_and_chandras_fury_target_a_planeswalker_and_hit_its_controllers_creatures() {
    cr!("115.1", "115.10a", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!(
        "Bonfire of the Damned",
        "Creatures with shroud or hexproof that player controls will be dealt damage by Bonfire of the Damned if it resolves."
    );
    ruling!(
        "Chandra's Fury",
        "Chandra’s Fury will deal 1 damage to a creature with hexproof, for example."
    );
    // (spell, X, damage to the planeswalker and to each of its controller's creatures)
    for (name, x, n) in [("Bonfire of the Damned", Some(2), 2u32), ("Chandra's Fury", None, 1)] {
        let mut t = TestGame::new(2);
        let w = walker(&mut t, 6);
        let bogle = t.battlefield(P1, "Slippery Bogle");
        let wurm = t.battlefield(P1, "Craw Wurm");
        let mine = t.battlefield(P0, "Craw Wurm");
        if let Some(x) = x {
            t.lands(P0, "Wastes", x as usize);
        }
        cast_x(&mut t, P0, name, NORMAL, x, &[obj(w)]);
        t.resolve_all();
        let to_walker = if name == "Chandra's Fury" { 4 } else { n };
        assert_eq!(loyalty_of(&t, w), 6 - to_walker, "{name}");
        assert!(!t.on_battlefield(bogle), "{name}: hexproof creature dealt damage");
        assert_eq!(dmg(&t, wurm), n, "{name}");
        assert_eq!(dmg(&t, mine), 0, "{name}: not the caster's creatures");
        assert_eq!(t.life(P1), 20, "{name}: not the player");
    }
}

#[test]
fn flames_of_the_blood_hand_stops_the_planeswalkers_controllers_life_gain() {
    cr!("615.12", "614.1a", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!("Flames of the Blood Hand", "The damage is unpreventable");
    supported("Flames of the Blood Hand");
    // "Flames of the Blood Hand deals 4 damage to target player or planeswalker. The damage
    // can't be prevented. If that player or that planeswalker's controller would gain life
    // this turn, that player gains no life instead." P1's planeswalker is targeted, with a
    // Healing Salve shield on it: it loses 4 loyalty; P1 gains no life this turn, P0 does.
    let mut t = TestGame::new(2);
    let w = walker(&mut t, 6);
    cast_new(&mut t, P0, "Flames of the Blood Hand", &[obj(w)]);
    salve(&mut t, obj(w));
    assert_eq!(t.stack_len(), 1, "only Flames");
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 2, "can't be prevented");
    crate::r_p009_common::gain(&mut t, P1, 3);
    crate::r_p009_common::gain(&mut t, P0, 3);
    assert_eq!((t.life(P1), t.life(P0)), (20, 23));
}

#[test]
fn blightning_and_rakdos_s_return_make_the_planeswalkers_controller_discard() {
    cr!("608.2c", "615.1", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!(
        "Blightning",
        "will discard two cards even if some or all of Blightning's damage is prevented or redirected"
    );
    ruling!(
        "Rakdos's Return",
        "will discard X cards even if some or all of the damage dealt by Rakdos’s Return is prevented or redirected"
    );
    supported("Blightning");
    supported("Rakdos's Return");
    // Each targets P1's planeswalker, which a Healing Salve shield protects: no loyalty is
    // lost, and P1 (its controller) discards two cards.
    for name in ["Blightning", "Rakdos's Return"] {
        let mut t = TestGame::new(2);
        let w = walker(&mut t, 6);
        for _ in 0..3 {
            t.hand(P1, "Grizzly Bears");
        }
        if name == "Blightning" {
            cast_new(&mut t, P0, name, &[obj(w)]);
        } else {
            t.lands(P0, "Swamp", 1);
            t.lands(P0, "Mountain", 3);
            let rr = t.hand(P0, name);
            t.cast(P0, rr).x(2).target(obj(w)).go();
        }
        salve(&mut t, obj(w));
        t.resolve_all();
        assert_eq!(loyalty_of(&t, w), 6, "{name}: prevented");
        assert_eq!(t.hand_size(P1), 1, "{name}: two discarded");
        assert_eq!(t.hand_size(P0), 0, "{name}");
    }
}

#[test]
fn concussive_bolt_on_a_planeswalker_stops_its_controllers_blockers() {
    cr!("611.2c", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!(
        "Concussive Bolt",
        "That is, the set of affected creatures isn’t locked in as the spell resolves."
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Ornithopter");
    }
    let w = walker(&mut t, 6);
    let before = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    cast_x(&mut t, P0, "Concussive Bolt", NORMAL, None, &[obj(w)]);
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 2);
    let later = t.enter(P1, "Hill Giant");
    t.settle();
    assert!(!t.g.can_block_at_all(before));
    assert!(!t.g.can_block_at_all(later));
    assert!(t.g.can_block_at_all(mine), "not the caster's creatures");
}

#[test]
fn quenchable_fire_on_a_planeswalker_its_controller_may_pay() {
    cr!("603.7", "116.2c", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!(
        "Quenchable Fire",
        "even if the original 3 damage is prevented or redirected to a different permanent or player"
    );
    supported("Quenchable Fire");
    // P1's planeswalker, shielded by Healing Salve: the first 3 damage is prevented, P1 (its
    // controller) may pay {U}, P0 may not; P1 doesn't, and the planeswalker is dealt 3 at
    // P0's next upkeep.
    let mut t = TestGame::new(2);
    let w = walker(&mut t, 10);
    cast_x(&mut t, P0, "Quenchable Fire", NORMAL, None, &[obj(w)]);
    salve(&mut t, obj(w));
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 10, "prevented");
    t.lands(P1, "Island", 1);
    t.lands(P0, "Island", 1);
    assert_eq!(offers(&mut t, P1).len(), 1, "its controller may pay");
    assert!(offers(&mut t, P0).is_empty(), "the caster may not");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 7);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn acorn_catapult_gives_the_planeswalkers_controller_the_squirrel() {
    cr!("115.4", "608.2c", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    ruling!(
        "Acorn Catapult",
        "regardless of how much damage was dealt or what the damage was ultimately dealt to"
    );
    supported("Acorn Catapult");
    // Dealt to the planeswalker, then prevented by a Healing Salve shield: P1, its
    // controller, creates the Squirrel each time.
    for prevented in [false, true] {
        let mut t = TestGame::new(2);
        let cat = t.battlefield(P0, "Acorn Catapult");
        let w = walker(&mut t, 6);
        if prevented {
            salve(&mut t, obj(w));
        }
        t.lands(P0, "Wastes", 1);
        t.g.turn.priority = Some(P0);
        t.activate(P0, cat, 0, &[obj(w)]).expect("Acorn Catapult");
        t.resolve_all();
        assert_eq!(loyalty_of(&t, w), if prevented { 6 } else { 5 });
        assert_eq!(with_subtype(&t, P1, "Squirrel").len(), 1, "{prevented}");
        assert!(with_subtype(&t, P0, "Squirrel").is_empty(), "{prevented}");
    }
}

#[test]
fn firestorm_and_chandra_the_firebrand_target_planeswalkers_among_their_targets() {
    cr!("115.4", "115.1c", "120.3c");
    interp!("pre-dominaria-player-target-rulings-give-way");
    supported("Firestorm");
    supported("Chandra, the Firebrand");
    // Firestorm, X = 2: the planeswalker and P1.
    let mut t = TestGame::new(2);
    let w = walker(&mut t, 6);
    t.hand(P0, "Forest");
    t.hand(P0, "Island");
    t.lands(P0, "Mountain", 1);
    let c = t.hand(P0, "Firestorm");
    t.cast(P0, c).x(2).targets(&[obj(w), pl(P1)]).go();
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 4);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.graveyard_size(P0), 3, "two discarded");
    // Chandra, the Firebrand's −6: the planeswalker, P1 and a creature.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let chandra = pw(&mut t, P0, "Chandra, the Firebrand", 7);
    let w = walker(&mut t, 8);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(w), pl(P1), obj(wurm)]);
    crate::r_s06_common::activate_containing(&mut t, P0, chandra, "each of up to six")
        .expect("Chandra's −6");
    t.resolve_all();
    assert_eq!(loyalty_of(&t, w), 2);
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}
