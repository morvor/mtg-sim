//! Rulings residue — tokens that are copies of the permanent that made them (batches P030,
//! P032): a token copy of a permanent that has left the battlefield uses its last known
//! copiable values (CR 707.2, 608.2h), including copy effects then applying to it (CR
//! 707.3); a token copy copies only copiable values (CR 707.2) except for the stated
//! exceptions (CR 707.9b); tokens put onto the battlefield attacking weren't declared as
//! attackers (CR 508.4); a token copy's enters abilities (CR 603.6a, 707.9a); conditions
//! checked on resolution (CR 608.2c); the legend rule (CR 704.5j); face-down permanents and
//! their copies (CR 708.2, 708.9); a search of several zones (CR 701.23, 400.11b).

use crate::r_p030_common::{kill, tokens_named};
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s25_common::cast_new;
use crate::r_s26_common::{dress_up, fresh};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Asserts `p` has exactly one token named `name`, and returns it.
fn one_token(t: &TestGame, p: PlayerId, name: &str) -> ObjectId {
    let toks = tokens_named(t, p, name);
    assert_eq!(toks.len(), 1, "tokens named {name}: {toks:?}");
    toks[0]
}

/// Cytoshape ("Choose a nonlegendary creature on the battlefield. Target nonlegendary
/// creature becomes a copy of that creature until end of turn."): `target` becomes a copy
/// of `model`.
fn cytoshape(t: &mut TestGame, target: ObjectId, model: ObjectId) {
    supported("Cytoshape");
    cast_new(t, P0, "Cytoshape", &[obj(target)]);
    t.answer_choose(P0, &[obj(model)]);
    t.resolve();
}

// --- Tokens entering attacking ----------------------------------------------------------------

/// P0's Hellrider ("Whenever a creature you control attacks, Hellrider deals 1 damage to
/// the player or planeswalker it's attacking.") and `attacker` (with an attack trigger that
/// creates tokens attacking P2) in a three-player game. `attacker` attacks P1; the
/// triggers resolve. Returns P2's life lost (to Hellrider: no combat damage yet).
fn hellrider_and(attacker: &str, setup: fn(&mut TestGame)) -> (TestGame, i32) {
    supported(attacker);
    supported("Hellrider");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::BeginningOfCombat);
    t.battlefield(P0, "Hellrider");
    let a = t.battlefield(P0, attacker);
    setup(&mut t);
    attack_with(&mut t, &[(a, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    let lost = 20 - t.life(P2);
    (t, lost)
}

#[test]
fn tokens_entering_attacking_weren_t_declared_and_don_t_trigger_attack_abilities() {
    cr!("508.4", "508.3a", "603.2");
    ruling!(
        "Shredder, Shadow Master",
        "Although the token copies created by Shredder's first ability enter attacking, they were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won't trigger when those tokens enter attacking."
    );
    // Shredder attacks P1: a token copy enters attacking P2. Hellrider triggers for
    // Shredder only (1 damage to P1).
    let (t, lost) = hellrider_and("Shredder, Shadow Master", |_| {});
    let tok = one_token(&t, P0, "Shredder, Shadow Master");
    assert_eq!(
        t.g.combat.as_ref().unwrap().attack_target(tok),
        Some(pl(P2))
    );
    assert_eq!(lost, 0);
    assert_eq!(t.life(P1), 19);
    ruling!(
        "Endless Foot Assault",
        "Although the tokens created by the last ability are attacking, they were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won't trigger when those tokens enter attacking."
    );
    // Endless Foot Assault ("Whenever you attack, for each opponent, create a 1/1 black
    // Ninja creature token that's tapped and attacking that player."): two Ninjas enter
    // attacking P1 and P2; Hellrider triggers only for the Grizzly Bears that attacked.
    let (t, lost) = hellrider_and("Grizzly Bears", |t| {
        t.battlefield(P0, "Endless Foot Assault");
    });
    let ninjas =
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Ninja"))
            .count();
    assert_eq!(ninjas, 2);
    assert_eq!(lost, 0);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn shredder_s_tokens_after_shredder_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Shredder, Shadow Master",
        "If Shredder leaves the battlefield before his last ability resolves, the tokens will still enter as a copy of Shredder, using Shredder's copiable values from when he was last on the battlefield."
    );
    // (The ability that makes the tokens.) Shredder is a copy of a Hill Giant (Cytoshape)
    // and then dies before it resolves: the token is a Hill Giant attacking P2.
    for cytoshaped in [false, true] {
        let mut t = TestGame::new(3);
        t.set_step(P0, Step::BeginningOfCombat);
        let sh = t.battlefield(P0, "Shredder, Shadow Master");
        let giant = t.battlefield(P1, "Hill Giant");
        attack_with(&mut t, &[(sh, pl(P1))]);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if cytoshaped {
            cytoshape(&mut t, sh, giant);
            assert_eq!(t.obj_now(sh).chars.name, "Hill Giant");
        }
        kill(&mut t, sh);
        t.resolve_all();
        let name = if cytoshaped {
            "Hill Giant"
        } else {
            "Shredder, Shadow Master"
        };
        let tok = one_token(&t, P0, name);
        assert!(!t.obj_now(tok).chars.is_legendary());
        assert_eq!(
            t.g.combat.as_ref().unwrap().attack_target(tok),
            Some(pl(P2))
        );
    }
}

#[test]
fn shredder_s_tokens_copy_only_copiable_values() {
    cr!("707.2", "707.9b");
    ruling!(
        "Shredder, Shadow Master",
        "The tokens don't copy whether Shredder is tapped or untapped, whether he has any counters on him or Auras attached to him, or any non-copy effects that have changed his types, color, and so on."
    );
    supported("Rancor");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::BeginningOfCombat);
    let sh = t.battlefield(P0, "Shredder, Shadow Master");
    // Rancor (+2/+0 and trample), two +1/+1 counters, +3/+3 and green-blue.
    attach_new(&mut t, P0, "Rancor", obj(sh));
    dress_up(&mut t, sh);
    t.g.untap(sh);
    t.g.recompute();
    assert_eq!(t.pt(sh), (12, 10));
    attack_with(&mut t, &[(sh, pl(P1))]);
    t.resolve_all();
    let tok = one_token(&t, P0, "Shredder, Shadow Master");
    assert_eq!(t.pt(tok), (5, 5));
    assert_eq!(t.obj_now(tok).chars.colors, ColorSet::single(Color::Black));
    assert!(t.obj_now(tok).counters.values().all(|n| *n == 0));
    assert!(t.g.attachments_of(obj(tok)).is_empty());
    // Not legendary (the exception), and tapped only because it enters tapped and
    // attacking.
    assert!(!t.obj_now(tok).chars.is_legendary());
    assert!(t.obj_now(tok).tapped);
}

#[test]
fn a_myriad_token_s_enters_abilities_trigger() {
    cr!("702.116a", "707.9a", "603.6a");
    ruling!(
        "Scurry of Squirrels",
        "Any \"enters\" abilities of the copied creature will trigger when the tokens enter. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied creature will also work."
    );
    supported("Scurry of Squirrels");
    supported("Callidus Assassin");
    // Callidus Assassin enters as a copy of Scurry of Squirrels, except it has "When this
    // creature enters, destroy up to one other target creature with the same name as this
    // creature." (a copiable value, CR 707.9a). It attacks P1: its myriad tokens (for P2)
    // are copies of it with that ability, which triggers as each enters and destroys the
    // original Scurry of Squirrels. (Scurry of Squirrels itself has no "as enters" or
    // "enters with" ability for its tokens to have.)
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let scurry = t.battlefield(P0, "Scurry of Squirrels");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(scurry)]);
    t.answer_targets(P0, &[]);
    let ca = t.enter(P0, "Callidus Assassin");
    t.resolve_all();
    assert_eq!(t.obj_now(ca).chars.name, "Scurry of Squirrels");
    assert!(t.on_battlefield(scurry));
    t.g.untap(ca);
    t.g.objects[ca.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::BeginningOfCombat);
    // Myriad twice: a token for P2 each time ("you may": yes, yes); each token's
    // enters ability targets the original Scurry.
    for _ in 0..2 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(scurry)]);
    }
    let ca = t.g.current(ca);
    attack_with(&mut t, &[(ca, pl(P1))]);
    t.resolve_all();
    let toks: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.name == "Scurry of Squirrels")
            .map(|o| o.id)
            .collect();
    assert!(!toks.is_empty(), "{}", t.dump_log());
    assert!(
        !t.on_battlefield(scurry),
        "a token's enters ability destroyed it"
    );
}

#[test]
fn battle_angels_of_tyr_checks_each_condition_on_resolution() {
    cr!("608.2c", "603.4");
    ruling!(
        "Battle Angels of Tyr",
        "Each condition is checked as the triggered ability resolves, not when it is placed on the stack."
    );
    supported("Battle Angels of Tyr");
    for true_on_resolution in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::BeginningOfCombat);
        let angels = t.battlefield(P0, "Battle Angels of Tyr");
        // Before damage, P1 has more cards, lands and life than P0 when
        // `!true_on_resolution`, and fewer otherwise; it's swapped before the ability
        // resolves.
        let (rich, poor) = if true_on_resolution {
            (P0, P1)
        } else {
            (P1, P0)
        };
        let cards: Vec<ObjectId> = (0..3).map(|_| t.hand(rich, "Grizzly Bears")).collect();
        let lands = t.lands(rich, "Forest", 2);
        t.g.player_mut(rich).life = 30;
        attack_with(&mut t, &[(angels, pl(P1))]);
        t.resolve_all();
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        // Swap: the cards, lands and life go to the other player.
        for c in cards {
            let c = t.g.current(c);
            t.g.discard(rich, c, None);
        }
        for _ in 0..3 {
            t.hand(poor, "Grizzly Bears");
        }
        for l in lands {
            t.g.destroy(l, None);
        }
        t.lands(poor, "Forest", 2);
        t.g.player_mut(poor).life = 40;
        t.g.player_mut(rich).life = 10;
        t.g.flush_events();
        let hand = t.hand_size(P0);
        let life = t.life(P0);
        t.resolve_all();
        let treasures =
            t.g.permanents()
                .filter(|o| o.controller == P0 && o.chars.has_subtype("Treasure"))
                .count();
        if true_on_resolution {
            // P1 now has more cards (3 to P0's), lands and life: all three.
            assert_eq!(t.hand_size(P0), hand + 1);
            assert_eq!(treasures, 1);
            assert_eq!(t.life(P0), life + 3);
        } else {
            assert_eq!(t.hand_size(P0), hand);
            assert_eq!(treasures, 0);
            assert_eq!(t.life(P0), life);
        }
    }
}

#[test]
fn ran_and_shaw_s_token_copies_only_printed_values_but_isn_t_legendary() {
    cr!("707.2", "707.9b");
    ruling!(
        "Ran and Shaw",
        "Except for the legendary supertype, the token copies only what is printed on Ran and Shaw unless Ran and Shaw are being modified by another copy effect. It doesn't copy whether Ran and Shaw are tapped or untapped, whether they have any counters on them or Auras or Equipment attached to them, or any non-copy effects that have changed their power, toughness, types, color, or so on."
    );
    supported("Ran and Shaw");
    supported("Rancor");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.graveyard(P0, "Shivan Dragon");
    }
    let rs = cast_new(&mut t, P0, "Ran and Shaw", &[]);
    t.resolve();
    let rs = t.g.current(rs);
    assert!(t.on_battlefield(rs));
    assert_eq!(t.stack_len(), 1);
    // Before the ability resolves: Rancor, counters, +3/+3, green-blue, tapped.
    attach_new(&mut t, P0, "Rancor", obj(rs));
    dress_up(&mut t, rs);
    t.resolve_all();
    let tok = one_token(&t, P0, "Ran and Shaw");
    assert_eq!(t.pt(tok), (4, 4));
    assert!(fresh(&t, tok));
    assert_eq!(t.obj_now(tok).chars.colors, ColorSet::single(Color::Red));
    assert!(!t.obj_now(tok).chars.is_legendary());
    assert!(t.obj_now(rs).chars.is_legendary());
    // A copy effect on it is copied: Ran and Shaw is a Hill Giant (Cytoshape needs a
    // nonlegendary target, so the token itself, a nonlegendary Ran and Shaw, is used).
    let giant = t.battlefield(P1, "Hill Giant");
    cytoshape(&mut t, tok, giant);
    assert_eq!(t.obj_now(tok).chars.name, "Hill Giant");
}

#[test]
fn nacatl_war_pride_s_tokens_copy_what_it_is_or_last_was() {
    cr!("707.2", "707.3", "608.2h");
    ruling!(
        "Nacatl War-Pride",
        "If Nacatl War-Pride becomes a copy of something else after the ability triggers but before it resolves, the tokens will enter as copies of whatever the ex-Nacatl War-Pride is now. If Nacatl War-Pride became a copy of something else and then left the battlefield, the tokens will enter as copies of whatever it was copying when it last existed on the battlefield."
    );
    ruling!(
        "Nacatl War-Pride",
        "If Nacatl War-Pride has left the battlefield by the time the triggered ability resolves, the tokens will still enter as copies of Nacatl War-Pride."
    );
    supported("Nacatl War-Pride");
    // P1 controls one creature (a Hill Giant): one token, a copy of what the War-Pride is
    // now (0: unchanged, left; 1: Cytoshaped; 2: Cytoshaped, then left).
    for case in 0..3 {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::BeginningOfCombat);
        let pride = t.battlefield(P0, "Nacatl War-Pride");
        let giant = t.battlefield(P1, "Hill Giant");
        attack_with(&mut t, &[(pride, pl(P1))]);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if case > 0 {
            cytoshape(&mut t, pride, giant);
        }
        if case != 1 {
            kill(&mut t, pride);
        }
        t.resolve_all();
        let name = if case == 0 {
            "Nacatl War-Pride"
        } else {
            "Hill Giant"
        };
        let tok = one_token(&t, P0, name);
        assert!(t.obj_now(tok).tapped, "case {case}");
        assert!(t.g.attackers().contains(&tok), "case {case}");
    }
}

#[test]
fn phoenix_fleet_airship_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Phoenix Fleet Airship",
        "If Phoenix Fleet Airship leaves the battlefield before its triggered ability resolves, the token will still enter as a copy of Phoenix Fleet Airship, using its copiable values from when it was last on the battlefield."
    );
    supported("Phoenix Fleet Airship");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ship = t.battlefield(P0, "Phoenix Fleet Airship");
    let food = t.battlefield(P0, "Ornithopter");
    t.g.sacrifice(food, P0);
    t.g.flush_events();
    t.settle();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, ship);
    t.resolve_all();
    one_token(&t, P0, "Phoenix Fleet Airship");
}

#[test]
fn scouring_swarm_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Scouring Swarm",
        "If Scouring Swarm leaves the battlefield before its triggered ability resolves and seven or more land cards are in your graveyard when the ability resolves, the token will still enter as a copy of Scouring Swarm, using Scouring Swarm’s copiable values from when it was last on the battlefield."
    );
    supported("Scouring Swarm");
    supported("Zuran Orb");
    // Six lands in the graveyard; Zuran Orb ("Sacrifice a land: You gain 2 life.")
    // sacrifices a seventh. The Swarm dies in response.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let swarm = t.battlefield(P0, "Scouring Swarm");
    let orb = t.battlefield(P0, "Zuran Orb");
    for _ in 0..6 {
        t.graveyard(P0, "Forest");
    }
    let land = t.battlefield(P0, "Forest");
    t.answer_choose(P0, &[obj(land)]);
    activate_containing(&mut t, P0, orb, "Sacrifice a land").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    kill(&mut t, swarm);
    t.resolve_all();
    let tok = one_token(&t, P0, "Scouring Swarm");
    assert!(t.obj_now(tok).tapped);
    assert_eq!(t.pt(tok), (1, 1));
}

// --- Jace, Mirror Mage -----------------------------------------------------------------------

#[test]
fn one_legendary_jace_and_any_number_of_nonlegendary_copies() {
    cr!("704.5j", "707.9b");
    ruling!(
        "Jace, Mirror Mage",
        "You can control one legendary Jace, Mirror Mage in addition to any number of nonlegendary copies of Jace, Mirror Mage."
    );
    // Two kicked Jaces: the legend rule keeps one of the two cards; both tokens stay.
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        t.lands(P0, "Island", 5);
        let j = t.hand(P0, "Jace, Mirror Mage");
        t.cast(P0, j).kicked(true).go();
        t.resolve_all();
    }
    let jaces = t.named_on_battlefield("Jace, Mirror Mage");
    let tokens = jaces.iter().filter(|j| t.obj_now(**j).is_token()).count();
    assert_eq!(tokens, 2);
    assert_eq!(jaces.len(), 3);
}

// --- Magar of the Magic Strings ----------------------------------------------------------------

/// P0's Magar puts the Lightning Bolt in P0's graveyard onto the battlefield face down.
fn magar_face_down(t: &mut TestGame) -> ObjectId {
    supported("Magar of the Magic Strings");
    let magar = t.battlefield(P0, "Magar of the Magic Strings");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[obj(bolt)]);
    activate_containing(t, P0, magar, "Note the name").unwrap();
    t.resolve_all();
    let fd = t.g.current(bolt);
    assert!(t.on_battlefield(fd) && t.obj(fd).face_down);
    fd
}

#[test]
fn a_copy_of_magar_s_face_down_creature_doesn_t_know_the_noted_name() {
    cr!("708.2", "707.2", "707.14");
    ruling!(
        "Magar of the Magic Strings",
        "If a creature enters the battlefield as a copy of a face-down creature (or if a token is created that’s a copy of one), that copy has the same characteristics as the face-down creature (in this case, it’s a 3/3 creature with the two abilities and no other characteristics), even though the copy is face up. Importantly, the noted name is not part of the face-down creature’s copiable values, so copies of the face-down creature won’t know the noted name. If one deals combat damage to a player, its controller won’t create a copy of any card."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let fd = magar_face_down(&mut t);
    // Clone enters as a copy of it: a face-up 3/3 with no name and the two abilities.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(fd)]);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    let c = t.obj_now(clone);
    assert!(!c.face_down);
    assert_eq!(t.pt(clone), (3, 3));
    assert!(c.chars.name.is_empty());
    assert_eq!(
        c.chars
            .mana_cost
            .as_ref()
            .map(|m| format!("{m}"))
            .unwrap_or_default(),
        ""
    );
    assert_eq!(c.chars.colors, ColorSet::default());
    assert!(c.chars.subtypes.is_empty());
    let texts: Vec<String> = c
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect();
    assert!(
        texts
            .iter()
            .any(|x| x.contains("deals combat damage to a player")),
        "{texts:?}"
    );
    assert!(
        texts.iter().any(|x| x.contains("exile it instead")),
        "{texts:?}"
    );
    // It hits P1: no copy of any card is created (P1 takes only the 3 combat damage).
    let clone = t.g.current(clone);
    t.g.untap(clone);
    t.g.objects[clone.0 as usize].summoning_sick = false;
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.answer_targets(P0, &[pl(P1)]);
    attack_with(&mut t, &[(clone, pl(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.g.stack.is_empty());
}

#[test]
fn magar_s_face_down_card_is_revealed_as_it_leaves() {
    cr!("708.9");
    ruling!(
        "Magar of the Magic Strings",
        "If a face-down permanent you control leaves the battlefield, or if the game ends or you leave the game while you control it, you must reveal it to all players (even though in this case, everyone saw it before it entered)."
    );
    let revealed = |t: &TestGame, from: usize, id: ObjectId| {
        t.g.turn_events[from..].iter().any(|e| match e {
            Event::Custom {
                name, obj: Some(o), ..
            } => {
                (name == mtg_engine::facedown::REVEALED || name == mtg_engine::reveal::REVEALED)
                    && *o == id
            }
            _ => false,
        })
    };
    // It leaves the battlefield (destroyed: exiled instead).
    let mut t = TestGame::new(2);
    let fd = magar_face_down(&mut t);
    let from = t.g.turn_events.len();
    t.g.destroy(fd, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert!(revealed(&t, from, fd), "revealed as it left");
    // Its controller leaves the game (a multiplayer game).
    let mut t = TestGame::new(3);
    let fd = magar_face_down(&mut t);
    let from = t.g.turn_events.len();
    t.g.lose_game(P0);
    t.g.flush_events();
    t.resolve_all();
    assert!(
        revealed(&t, from, fd),
        "revealed as its controller left the game"
    );
}

// --- Invasion of Arcavios ----------------------------------------------------------------------
