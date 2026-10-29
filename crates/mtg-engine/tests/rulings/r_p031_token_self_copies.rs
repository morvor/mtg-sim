//! Rulings batch P031 — permanents that create token copies of themselves: the token has
//! the copiable values only (CR 707.2) — so it has the ability that made it and can make
//! copies of itself — and none of the counters, damage, attachments, tapped status or
//! non-copy effects; a copy effect that applies to the permanent when the ability resolves
//! is copied (CR 707.3), and if the permanent has left the battlefield its last known
//! copiable values are used (CR 608.2h).

use crate::r_p030_common::{draw, surveil};
use crate::r_p031_common::*;
use crate::r_p032_common::{abilities_with, assert_plain_copy, dress};
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{activate_containing, attach_new, damage};
use crate::r_s17_common::become_copy;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// One self-copying permanent: how to put it onto the battlefield (with whatever it needs)
/// and how to make its copying ability trigger or be activated (leaving it on the stack,
/// with any answers and mana its resolution needs queued).
struct Case {
    name: &'static str,
    setup: fn(&mut TestGame) -> ObjectId,
    fire: fn(&mut TestGame, ObjectId),
    /// The effect creates the token tapped.
    tapped: bool,
}

fn put(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, name)
}

/// Casts Lava Axe ({4}{R} sorcery, mana value 5) at P1.
fn lava_axe(t: &mut TestGame, _: ObjectId) {
    cast_new(t, P0, "Lava Axe", &[Entity::Player(P1)]);
    t.settle();
}

/// Casts Shock at P1.
fn shock(t: &mut TestGame, _: ObjectId) {
    cast_new(t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
}

fn copy_catchers_fire(t: &mut TestGame, _: ObjectId) {
    surveil(t, P0, 1);
    for _ in 0..t.stack_len() {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 1);
        t.answer_yes(P0, true);
    }
}

/// Goes to P0's next precombat main phase (drawing a card in the draw step), then draws
/// a second card.
fn horde_fire(t: &mut TestGame, _: ObjectId) {
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    draw(t, P0, 1);
}

/// Activates the token-making ability of each Improvised Arsenal P0 controls.
fn arsenal_fire(t: &mut TestGame, _: ObjectId) {
    for a in named(t, P0, "Improvised Arsenal") {
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 4);
        activate_containing(t, P0, a, "Create a token").unwrap();
    }
}

fn swarm_setup(t: &mut TestGame) -> ObjectId {
    for _ in 0..7 {
        t.graveyard(P0, "Forest");
    }
    put(t, "Scouring Swarm")
}

fn swarm_fire(t: &mut TestGame, _: ObjectId) {
    let land = t.battlefield(P0, "Forest");
    t.g.sacrifice(land, P0);
    t.g.flush_events();
    t.settle();
}

/// A Craw Wurm (6/4) enters; the evolve trigger resolves, so the "whenever this creature
/// evolves" trigger is on the stack.
fn radstag_fire(t: &mut TestGame, _: ObjectId) {
    t.enter(P0, "Craw Wurm");
    t.g.flush_events();
    t.settle();
    t.resolve();
}

fn blob_setup(t: &mut TestGame) -> ObjectId {
    let c = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[c.0 as usize].is_commander = true;
    t.g.dirty = true;
    put(t, "Biowaste Blob")
}

fn blob_fire(t: &mut TestGame, _: ObjectId) {
    if t.g.turn.step == Step::Upkeep {
        t.advance_to(P1, Step::PrecombatMain);
    }
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

fn compy_fire(t: &mut TestGame, _: ObjectId) {
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(t, bears);
    t.advance_to(P0, Step::End);
    t.settle();
}

const CASES: &[Case] = &[
    Case {
        name: "Biowaste Blob",
        setup: blob_setup,
        fire: blob_fire,
        tapped: false,
    },
    Case {
        name: "Colorstorm Stallion",
        setup: |t| put(t, "Colorstorm Stallion"),
        fire: lava_axe,
        tapped: false,
    },
    Case {
        name: "Compy Swarm",
        setup: |t| put(t, "Compy Swarm"),
        fire: compy_fire,
        tapped: true,
    },
    Case {
        name: "Copy Catchers",
        setup: |t| put(t, "Copy Catchers"),
        fire: copy_catchers_fire,
        tapped: false,
    },
    Case {
        name: "Homunculus Horde",
        setup: |t| put(t, "Homunculus Horde"),
        fire: horde_fire,
        tapped: false,
    },
    Case {
        name: "Improvised Arsenal",
        setup: |t| put(t, "Improvised Arsenal"),
        fire: arsenal_fire,
        tapped: false,
    },
    Case {
        name: "Leitmotif Composer",
        setup: |t| put(t, "Leitmotif Composer"),
        fire: lava_axe,
        tapped: false,
    },
    Case {
        name: "Scouring Swarm",
        setup: swarm_setup,
        fire: swarm_fire,
        tapped: true,
    },
    Case {
        name: "Stormsplitter",
        setup: |t| put(t, "Stormsplitter"),
        fire: shock,
        tapped: false,
    },
    Case {
        name: "Watchful Radstag",
        setup: |t| put(t, "Watchful Radstag"),
        fire: radstag_fire,
        tapped: false,
    },
];

fn case(name: &str) -> &'static Case {
    CASES.iter().find(|c| c.name == name).expect("no such case")
}

/// Sets up the case's permanent, fires its ability and resolves everything; returns the
/// permanent and the new tokens named like it.
fn run(c: &Case) -> (TestGame, ObjectId, Vec<ObjectId>) {
    supported(c.name);
    let mut t = TestGame::new(2);
    let src = (c.setup)(&mut t);
    let before = t.g.battlefield.clone();
    (c.fire)(&mut t, src);
    t.resolve_all();
    let tok: Vec<ObjectId> = tokens_since(&t, P0, &before)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.name == c.name)
        .collect();
    (t, src, tok)
}

#[test]
fn the_token_copy_has_the_ability_and_makes_copies_of_itself() {
    cr!("707.2");
    ruling!(
        "Biowaste Blob",
        "The token copy will have Biowaste Blob's abilities, including the ability to create token copies of itself."
    );
    ruling!(
        "Colorstorm Stallion",
        "The token copy will have Colorstorm Stallion's ability and will be able to create copies of itself."
    );
    ruling!(
        "Compy Swarm",
        "The token copy will have Compy Swarm's ability. It will also be able to create copies of itself."
    );
    ruling!(
        "Copy Catchers",
        "The token copy will have Copy Catchers’s abilities and will be able to create copies of itself."
    );
    ruling!(
        "Homunculus Horde",
        "The token copy will have Homunculus Horde's ability and will be able to create copies of itself."
    );
    ruling!(
        "Improvised Arsenal",
        "The token copy will have Improvised Arsenal's abilities and will be able to create copies of itself."
    );
    ruling!(
        "Leitmotif Composer",
        "The token copy will have Leitmotif Composer's ability and will be able to create copies of itself."
    );
    ruling!(
        "Scouring Swarm",
        "The token copy will have Scouring Swarm’s abilities and will be able to create more copies of itself."
    );
    ruling!(
        "Stormsplitter",
        "The token copy will have Stormsplitter’s abilities and will be able to create copies of itself."
    );
    ruling!(
        "Watchful Radstag",
        "The token copy will have Watchful Radstag’s abilities. It will also be able to create copies of itself."
    );
    for c in CASES {
        let (mut t, src, tok) = run(c);
        assert_eq!(tok.len(), 1, "{}: one token", c.name);
        let printed = t.obj_now(src).chars.abilities.len();
        assert_eq!(
            t.obj_now(tok[0]).chars.abilities.len(),
            printed,
            "{}: the token's abilities",
            c.name
        );
        // Fire again: the original and the token each make a copy.
        let before = t.g.battlefield.clone();
        (c.fire)(&mut t, src);
        t.resolve_all();
        let more = tokens_since(&t, P0, &before)
            .into_iter()
            .filter(|id| t.obj_now(*id).chars.name == c.name)
            .count();
        assert_eq!(more, 2, "{}: the token made a copy of itself too", c.name);
    }
}

#[test]
fn the_token_copies_none_of_the_non_copiable_state() {
    cr!("707.2");
    ruling!(
        "Biowaste Blob",
        "The token copy won't copy counters or damage marked on Biowaste Blob, nor will it copy other effects that have changed Biowaste Blob's power, toughness, types, color, and so on."
    );
    ruling!(
        "Compy Swarm",
        "The token copy won't copy counters or damage marked on Compy Swarm, nor will it copy other effects that have changed Compy Swarm's power, toughness, types, color, and so on."
    );
    ruling!(
        "Watchful Radstag",
        "The token copy won’t copy counters or damage marked on Watchful Radstag, nor will it copy other effects that have changed Watchful Radstag’s power, toughness, types, color, and so on."
    );
    ruling!(
        "Colorstorm Stallion",
        "The token doesn't copy whether Colorstorm Stallion is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Homunculus Horde",
        "The token doesn't copy whether Homunculus Horde is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Improvised Arsenal",
        "The token doesn't copy whether Improvised Arsenal is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its types, color, and so on."
    );
    ruling!(
        "Leitmotif Composer",
        "The token doesn't copy whether Leitmotif Composer is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Copy Catchers",
        "The token doesn’t copy whether Copy Catchers is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Scouring Swarm",
        "The token doesn’t copy whether Scouring Swarm is tapped or untapped, whether it has any counters on it or Auras or Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Stormsplitter",
        "The token doesn’t copy whether Stormsplitter is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    for c in CASES {
        supported(c.name);
        let mut t = TestGame::new(2);
        let src = (c.setup)(&mut t);
        let printed_pt = {
            let o = t.obj_now(src);
            o.is(CardType::Creature).then(|| t.pt(src))
        };
        let before = t.g.battlefield.clone();
        (c.fire)(&mut t, src);
        // With the ability on the stack, the permanent is tapped, gets counters, damage,
        // +3/+3, new colors, and (if it's a creature) an Aura and an Equipment.
        dress(&mut t, src, printed_pt.is_some());
        if printed_pt.is_some() {
            attach_new(&mut t, P0, "Holy Strength", src);
            attach_new(&mut t, P0, "Bonesplitter", src);
        }
        t.resolve_all();
        let tok: Vec<ObjectId> = tokens_since(&t, P0, &before)
            .into_iter()
            .filter(|id| t.obj_now(*id).chars.name == c.name)
            .collect();
        assert_eq!(tok.len(), 1, "{}: one token", c.name);
        if c.tapped {
            assert!(t.obj_now(tok[0]).tapped, "{}: enters tapped", c.name);
            t.g.untap(tok[0]);
        }
        // Biowaste Blob's "Oozes you control get +1/+1" applies to its token.
        let pt = match c.name {
            "Biowaste Blob" => Some((2, 2)),
            _ => printed_pt,
        };
        assert_plain_copy(&t, tok[0], c.name, pt);
    }
}

#[test]
fn the_token_is_a_copy_of_what_the_permanent_became() {
    cr!("707.2", "707.3", "608.2h");
    ruling!(
        "Colorstorm Stallion",
        "In the unusual case where Colorstorm Stallion becomes a copy of something else while its triggered ability is on the stack but before it resolves, the token will enter as a copy of whatever Colorstorm Stallion is copying."
    );
    ruling!(
        "Copy Catchers",
        "In the unusual case where Copy Catchers becomes a copy of something else while its triggered ability is on the stack but before it resolves, the token will enter the battlefield as a copy of whatever Copy Catchers is copying."
    );
    ruling!(
        "Homunculus Horde",
        "In the unusual case where Homunculus Horde becomes a copy of something else while its triggered ability is on the stack but before it resolves, the token will enter as a copy of whatever Homunculus Horde is copying."
    );
    ruling!(
        "Improvised Arsenal",
        "In the unusual case where Improvised Arsenal becomes a copy of something else while its second ability is on the stack but before it resolves, the token will enter as a copy of whatever Improvised Arsenal is copying."
    );
    ruling!(
        "Leitmotif Composer",
        "In the unusual case where Leitmotif Composer becomes a copy of something else while its triggered ability is on the stack but before it resolves, the token will enter as a copy of whatever Leitmotif Composer is copying."
    );
    ruling!(
        "Scouring Swarm",
        "In the unusual case where Scouring Swarm becomes a copy of something else while its triggered ability is on the stack but before it resolves and seven or more land cards are in your graveyard when the ability resolves, the token will enter as a copy of whatever Scouring Swarm is copying."
    );
    ruling!(
        "Stormsplitter",
        "In the unusual case where Stormsplitter becomes a copy of something else while its triggered ability is on the stack but before it resolves, the token will enter as a copy of whatever Stormsplitter is copying."
    );
    for name in [
        "Colorstorm Stallion",
        "Copy Catchers",
        "Homunculus Horde",
        "Improvised Arsenal",
        "Leitmotif Composer",
        "Scouring Swarm",
        "Stormsplitter",
    ] {
        let c = case(name);
        supported(name);
        let mut t = TestGame::new(2);
        let src = (c.setup)(&mut t);
        let giant = t.battlefield(P1, "Hill Giant");
        let before = t.g.battlefield.clone();
        (c.fire)(&mut t, src);
        assert!(t.stack_len() >= 1);
        become_copy(&mut t, src, giant);
        assert_eq!(t.obj_now(src).chars.name, "Hill Giant");
        t.resolve_all();
        let tok = tokens_since(&t, P0, &before);
        let giants: Vec<_> = tok
            .iter()
            .filter(|id| t.obj_now(**id).chars.name == "Hill Giant")
            .collect();
        assert_eq!(giants.len(), 1, "{name}: the token is a Hill Giant");
        assert_eq!(t.pt(*giants[0]), (3, 3), "{name}");
        assert!(
            tok.iter().all(|id| t.obj_now(*id).chars.name != name),
            "{name}: no token of the printed card"
        );
    }
    // Scouring Swarm with fewer than seven land cards: an Insect, not a copy.
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Scouring Swarm");
    let giant = t.battlefield(P1, "Hill Giant");
    swarm_fire(&mut t, src);
    become_copy(&mut t, src, giant);
    t.resolve_all();
    assert_eq!(named(&t, P0, "Hill Giant").len(), 1, "only the Swarm itself");
    assert_eq!(named(&t, P0, "Insect Token").len(), 1);
}

#[test]
fn watchful_radstag_s_token_after_it_left() {
    cr!("707.2", "608.2h", "702.100b");
    ruling!(
        "Watchful Radstag",
        "If Watchful Radstag leaves the battlefield before its last ability resolves, the token will still enter the battlefield as a copy of Watchful Radstag, using Watchful Radstag’s copiable values from when it was last on the battlefield."
    );
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Watchful Radstag");
    radstag_fire(&mut t, r);
    assert_eq!(t.counters(r, counters::PLUS1), 1);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, r);
    t.resolve_all();
    let tok = crate::r_p032_common::tokens_named(&t, P0, "Watchful Radstag");
    assert_eq!(tok.len(), 1);
    assert_eq!(t.pt(tok[0]), (2, 2));
}

// --- Shredder, Shadow Master ----------------------------------------------------------

/// A three-player game: Shredder attacks P1; its trigger creates a token attacking P2.
fn shredder_attacks(t: &mut TestGame, shredder: ObjectId) -> Vec<ObjectId> {
    let before = t.g.battlefield.clone();
    attack_with(t, &[(shredder, Entity::Player(P1))]);
    t.resolve_all();
    tokens_since(t, P0, &before)
}

#[test]
fn shredder_s_token_copies_have_its_abilities() {
    cr!("707.2", "707.9b");
    ruling!("Shredder, Shadow Master", "The token copies will have Shredder's abilities.");
    supported("Shredder, Shadow Master");
    let mut t = TestGame::new(4);
    let s = t.battlefield(P0, "Shredder, Shadow Master");
    let tok = shredder_attacks(&mut t, s);
    assert_eq!(tok.len(), 2, "one token for each other opponent");
    for id in tok {
        let o = t.obj_now(id);
        assert_eq!(o.chars.name, "Shredder, Shadow Master");
        assert!(!o.chars.supertypes.contains(Supertype::Legendary));
        assert_eq!(o.chars.abilities.len(), t.obj_now(s).chars.abilities.len());
        assert_eq!(abilities_with(&t, id, "half their life"), 1);
        assert_eq!(abilities_with(&t, id, "attacks a player"), 1);
        assert!(o.tapped);
    }
}

#[test]
fn shredder_s_tokens_copy_what_it_s_copying() {
    cr!("707.2", "707.3", "707.9b");
    ruling!(
        "Shredder, Shadow Master",
        "In the unusual case where Shredder becomes a copy of something else while his last ability is on the stack but before it resolves, the tokens will enter as a copy of whatever Shredder is copying."
    );
    let mut t = TestGame::new(3);
    let s = t.battlefield(P0, "Shredder, Shadow Master");
    let giant = t.battlefield(P1, "Hill Giant");
    let before = t.g.battlefield.clone();
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    become_copy(&mut t, s, giant);
    t.resolve_all();
    let tok = tokens_since(&t, P0, &before);
    assert_eq!(tok.len(), 1);
    assert_eq!(t.obj_now(tok[0]).chars.name, "Hill Giant");
    assert_eq!(
        crate::r_s12_common::attack_target(&t, tok[0]),
        Some(Entity::Player(P2))
    );
}

// --- Bloodforged Battle-Axe ---------------------------------------------------------------

#[test]
fn bloodforged_battle_axe_s_token_is_unattached_and_makes_more() {
    cr!("707.2", "301.5b");
    ruling!("Bloodforged Battle-Axe", "The token enters the battlefield unattached.");
    ruling!(
        "Bloodforged Battle-Axe",
        "The token has all three of Bloodforged Battle-Axe's abilities, including the one that creates more tokens."
    );
    supported("Bloodforged Battle-Axe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let axe = attach_new(&mut t, P0, "Bloodforged Battle-Axe", bears);
    assert_eq!(t.pt(bears), (4, 2));
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 16);
    t.resolve_all();
    let tok = crate::r_p032_common::tokens_named(&t, P0, "Bloodforged Battle-Axe");
    assert_eq!(tok.len(), 1);
    assert_eq!(crate::r_s06_common::attached_to(&t, tok[0]), None);
    assert_eq!(t.pt(bears), (4, 2), "the token isn't attached to the Bears");
    assert_eq!(t.obj_now(tok[0]).chars.abilities.len(), t.obj_now(axe).chars.abilities.len());
    // Next turn, both Axes on one creature: each makes a token.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.g.attach(tok[0], obj(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (6, 2));
    hit(&mut t, bears);
    t.resolve_all();
    assert_eq!(
        crate::r_p032_common::tokens_named(&t, P0, "Bloodforged Battle-Axe").len(),
        3
    );
}

// --- Spawnwrithe, Sprouting Phytohydra, Mirror-Sigil Sergeant, Myr Propagator ------------

#[test]
fn a_mirrorweaved_spawnwrithe_makes_plain_spawnwrithes() {
    cr!("707.2", "707.3", "611.2a");
    ruling!(
        "Spawnwrithe",
        "If a copy effect such as Mirrorweave causes some other creature to become a copy of Spawnwrithe, then that creature deals combat damage to a player, the token that’s put onto the battlefield is simply a copy of Spawnwrithe."
    );
    supported("Mirrorweave");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Spawnwrithe");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Mirrorweave", &[obj(s)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Spawnwrithe");
    hit(&mut t, bears);
    t.resolve_all();
    let tok = crate::r_p032_common::tokens_named(&t, P0, "Spawnwrithe");
    assert_eq!(tok.len(), 1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.obj_now(tok[0]).chars.name, "Spawnwrithe");
    assert_eq!(t.pt(tok[0]), (2, 2));
}

#[test]
fn a_sprouting_phytohydra_token_makes_another_when_dealt_damage() {
    cr!("707.2");
    ruling!(
        "Sprouting Phytohydra",
        "If a token copy of Sprouting Phytohydra is dealt damage, it will create another token copy."
    );
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Sprouting Phytohydra");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    damage(&mut t, giant, 1, ph);
    t.resolve_all();
    let tok = crate::r_p032_common::tokens_named(&t, P0, "Sprouting Phytohydra");
    assert_eq!(tok.len(), 1);
    t.answer_yes(P0, true);
    damage(&mut t, giant, 1, tok[0]);
    t.resolve_all();
    assert_eq!(
        crate::r_p032_common::tokens_named(&t, P0, "Sprouting Phytohydra").len(),
        2
    );
}

#[test]
fn mirror_sigil_sergeant_doubles_each_upkeep() {
    cr!("707.2", "603.4");
    ruling!(
        "Mirror-Sigil Sergeant",
        "At the beginning of your next upkeep, if you still control the original Sergeant, the token copy, and a blue permanent, you’ll get two more token copies; the turn after that you’ll get four; then eight; and so on."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror-Sigil Sergeant");
    t.battlefield(P0, "Wind Drake");
    for want in [2, 4, 8] {
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        for _ in 0..t.stack_len() {
            t.answer_yes(P0, true);
        }
        t.resolve_all();
        assert_eq!(named(&t, P0, "Mirror-Sigil Sergeant").len(), want);
        t.advance_to(P0, Step::PrecombatMain);
    }
}

#[test]
fn myr_propagator_s_token_has_the_token_making_ability() {
    cr!("707.2");
    ruling!(
        "Myr Propagator",
        "Normally, when a token is created by this ability, it will simply be a Myr Propagator, so it’ll also have the token-creating ability."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Myr Propagator");
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, m, "Create a token").unwrap();
    t.resolve_all();
    let tok = crate::r_p032_common::tokens_named(&t, P0, "Myr Propagator");
    assert_eq!(tok.len(), 1);
    // The token isn't summoning sick only from the next turn; give it a turn.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, tok[0], "Create a token").unwrap();
    t.resolve_all();
    assert_eq!(
        crate::r_p032_common::tokens_named(&t, P0, "Myr Propagator").len(),
        2
    );
}

// --- Vaultborn Tyrant ----------------------------------------------------------------------

#[test]
fn vaultborn_tyrant_copying_something_else_makes_that() {
    cr!("707.2", "707.9b", "603.10a");
    ruling!(
        "Vaultborn Tyrant",
        "If Vaultborn Tyrant was copying something else when it was last on the battlefield but somehow still had its last ability, then the token enters the battlefield as whatever Vaultborn Tyrant copied, with the listed exceptions."
    );
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vaultborn Tyrant");
    let dies = t
        .obj_now(v)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Triggered(_)) && a.text.contains("dies"))
        .cloned()
        .expect("the dies ability");
    let giant = t.battlefield(P1, "Hill Giant");
    become_copy(&mut t, v, giant);
    crate::r_s26_common::modify_until_eot(
        &mut t,
        v,
        vec![mtg_engine::ability::Modification::AddAbility(dies)],
    );
    assert_eq!(t.obj_now(v).chars.name, "Hill Giant");
    kill(&mut t, v);
    t.resolve_all();
    let tok = crate::r_p032_common::tokens_named(&t, P0, "Hill Giant");
    assert_eq!(tok.len(), 1);
    let o = t.obj_now(tok[0]);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert_eq!(t.pt(tok[0]), (3, 3));
    assert_eq!(abilities_with(&t, tok[0], "dies"), 0, "the granted ability isn't copied");
}

#[test]
fn vaultborn_tyrant_sees_the_power_a_creature_enters_with() {
    cr!("603.6a", "603.6b", "707.5");
    ruling!(
        "Vaultborn Tyrant",
        "If one or more static abilities that apply to a creature entering the battlefield change its power, those abilities are considered when determining whether or not Vaultborn Tyrant’s second ability triggers."
    );
    // (Each part: does the entering creature trigger it?)
    let check = |setup: &dyn Fn(&mut TestGame), name: &str, want: bool| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Vaultborn Tyrant");
        setup(&mut t);
        let life = t.life(P0);
        t.enter(P0, name);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        assert_eq!(t.life(P0) == life + 3, want, "{name}");
    };
    // Hill Giant (3/3): no trigger alone; with Glorious Anthem it enters as a 4/4.
    check(&|_| {}, "Hill Giant", false);
    check(
        &|t| {
            t.battlefield(P0, "Glorious Anthem");
        },
        "Hill Giant",
        true,
    );
    // Spike Colony (0/0) enters with four +1/+1 counters.
    check(&|_| {}, "Spike Colony", true);
    // Clone entering as a copy of Craw Wurm (6/4) or of Grizzly Bears (2/2).
    let wurm = |t: &mut TestGame| {
        let w = t.battlefield(P1, "Craw Wurm");
        t.answer_choose(P0, &[obj(w)]);
    };
    check(&wurm, "Clone", true);
    let bears = |t: &mut TestGame| {
        let b = t.battlefield(P1, "Grizzly Bears");
        t.answer_choose(P0, &[obj(b)]);
    };
    check(&bears, "Clone", false);
}

#[test]
fn vaultborn_tyrant_s_trigger_isn_t_undone_by_lowering_power() {
    cr!("603.2");
    ruling!(
        "Vaultborn Tyrant",
        "Once Vaultborn Tyrant’s second ability has triggered, lowering the power of the creature or removing it from the battlefield won’t stop you from gaining life and drawing a card."
    );
    for remove in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Vaultborn Tyrant");
        let w = t.enter(P0, "Craw Wurm");
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if remove {
            kill(&mut t, w);
        } else {
            crate::r_s26_common::modify_until_eot(
                &mut t,
                w,
                vec![mtg_engine::ability::Modification::ModifyPT(
                    mtg_engine::ability::Value::c(-5),
                    mtg_engine::ability::Value::c(0),
                )],
            );
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.life(P0), 23);
        assert_eq!(t.hand_size(P0), hand + 1);
    }
}
