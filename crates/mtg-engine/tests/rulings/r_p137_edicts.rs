//! Rulings batch P137 — "each opponent sacrifices a creature" in a multiplayer game: the
//! opponents choose in turn order starting with the active player (or the next opponent
//! in turn order if it's your turn), each knowing the earlier choices, and then all the
//! chosen permanents are sacrificed at the same time (CR 101.4, 701.21a).

use crate::r_p137_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// The number of permanents now, then everything on the stack resolved.
fn go(t: &mut TestGame) -> usize {
    t.settle();
    let n = t.g.permanents().count();
    t.resolve_all();
    n
}

/// P0 casts `card` (paying for it with basic lands put onto the battlefield first).
fn cast_and_go(t: &mut TestGame, card: &str) -> usize {
    give_mana_for(t, P0, card);
    let spell = t.hand(P0, card);
    t.cast(P0, spell).go();
    go(t)
}

/// P0's creature `card` enters (its enters trigger goes on the stack).
fn enter_and_go(t: &mut TestGame, card: &str) -> usize {
    t.enter(P0, card);
    go(t)
}

#[test]
fn opponents_choose_in_turn_order_then_sacrifice_at_once_on_your_turn() {
    cr!("101.4", "701.21a");
    ruling!(
        "Mire in Misery",
        "As Mire in Misery resolves, first the next opponent in turn order (or, if it’s an opponent’s turn, the opponent whose turn it is) chooses a creature or an enchantment they control, then each other opponent in turn order does the same, knowing the choices made before them. Then all the chosen permanents are sacrificed at the same time."
    );
    check_edict("Mire in Misery", P0, P1, BEARS_GIANT, |t| {
        cast_and_go(t, "Mire in Misery")
    });
    ruling!(
        "Goremand",
        "As Goremand's triggered ability resolves, first the next opponent in turn order (or, if it's an opponent's turn, the opponent whose turn it is) chooses a creature they control, then each other opponent in turn order does the same knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    check_edict("Goremand", P0, P1, BEARS_GIANT, |t| enter_and_go(t, "Goremand"));
    ruling!(
        "Dusk Mangler",
        "As Dusk Mangler's triggered ability resolves, each opponent, starting with the next player in turn order, chooses a creature, knowing the choices made before them. All of those creatures are sacrificed at the same time. Next, each opponent chooses a card in their hand without revealing it, and all of those cards are discarded at the same time. Then, each opponent loses 4 life."
    );
    let t = check_edict("Dusk Mangler", P0, P1, BEARS_GIANT, |t| {
        t.hand(P1, "Island");
        t.hand(P2, "Island");
        enter_and_go(t, "Dusk Mangler")
    });
    for p in [P1, P2] {
        assert_eq!(t.hand_size(p), 0);
        assert_eq!(t.life(p), 16);
    }
    ruling!(
        "Yawning Fissure",
        "First the opponent whose turn it is — or, more likely, the first opponent to left of the player whose turn it is — chooses and announces which land they will sacrifice, then each other opponent in turn order does the same, then all lands are sacrificed at the same time."
    );
    check_edict("Yawning Fissure", P0, P1, ["Forest", "Mountain"], |t| {
        cast_and_go(t, "Yawning Fissure")
    });
    ruling!(
        "Extus, Oriq Overlord // Awaken the Blood Avatar",
        "As Awaken the Blood Avatar resolves, first the next opponent in turn order (or, if it’s an opponent’s turn, the opponent whose turn it is) chooses a creature they control, then each other opponent in turn order does the same knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    check_edict(
        "Extus, Oriq Overlord // Awaken the Blood Avatar",
        P0,
        P1,
        BEARS_GIANT,
        |t| {
            let card = t.hand(P0, "Extus, Oriq Overlord // Awaken the Blood Avatar");
            t.lands(P0, "Swamp", 1);
            t.lands(P0, "Mountain", 1);
            t.lands(P0, "Wastes", 6);
            t.answer_choose(P0, &[]);
            t.cast(P0, card).method(CastMethod::Half(1)).go();
            go(t)
        },
    );
    ruling!(
        "Malfegor",
        "After you discard your hand and the number of creatures each opponent must sacrifice has been determined, each opponent in turn order, starting with the player whose turn it is if that player is your opponent, chooses which creatures they will sacrifice, then all the chosen creatures are all sacrificed at the same time."
    );
    let t = check_edict("Malfegor", P0, P1, BEARS_GIANT, |t| {
        t.hand(P0, "Island");
        enter_and_go(t, "Malfegor")
    });
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn opponents_choose_starting_with_the_active_opponent_on_their_turn() {
    cr!("101.4", "701.21a");
    ruling!(
        "Liliana's Triumph",
        "As Liliana's Triumph resolves, the opponent whose turn it is (or, if it's your turn, the next opponent in turn order) chooses a creature to sacrifice, then each other opponent in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    // On P2's turn, P2 chooses first, then P1.
    check_edict("Liliana's Triumph", P2, P2, BEARS_GIANT, |t| {
        cast_and_go(t, "Liliana's Triumph")
    });
    check_edict("Liliana's Triumph", P0, P1, BEARS_GIANT, |t| {
        cast_and_go(t, "Liliana's Triumph")
    });
    ruling!(
        "Kaya's Guile",
        "As the first mode is performed, first the opponent whose turn it is (or, if it’s not an opponent’s turn, the next opponent in turn order) chooses a creature they control, then each other opponent in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    let t = check_edict("Kaya's Guile", P2, P2, BEARS_GIANT, |t| {
        give_mana_for(t, P0, "Kaya's Guile");
        let spell = t.hand(P0, "Kaya's Guile");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
        t.cast(P0, spell).modes(&[0, 3]).go();
        go(t)
    });
    assert_eq!(t.life(P0), 24);
    ruling!(
        "Soul Shatter",
        "As Soul Shatter resolves, first the opponent whose turn it is (or the next opponent in turn order, if it's your turn) chooses a creature or planeswalker they control with the highest mana value among that player's creatures and planeswalkers, then each other opponent in turn order does the same, knowing the choices made before them. Then all of the chosen permanents are sacrificed at the same time."
    );
    check_edict("Soul Shatter", P1, P1, TWO_BEARS, |t| {
        cast_and_go(t, "Soul Shatter")
    });
    check_edict("Soul Shatter", P0, P1, TWO_BEARS, |t| {
        cast_and_go(t, "Soul Shatter")
    });
    ruling!(
        "Rampage of the Valkyries",
        "As the last ability resolves, if it’s not your turn, the first player to choose a creature they control is the player whose turn it is. If it is your turn, the next player in turn order chooses first. In either case, each other player in turn order then chooses a creature they control, knowing the choices made before their choice. Then all chosen creatures are sacrificed at the same time."
    );
    for (active, first) in [(P2, P2), (P0, P1)] {
        let t = check_edict("Rampage of the Valkyries", active, first, BEARS_GIANT, |t| {
            t.battlefield(P0, "Rampage of the Valkyries");
            let angel = t.battlefield(P0, "Serra Angel");
            destroy(t, angel);
            go(t)
        });
        // You aren't one of the "other players".
        assert!(t.in_graveyard(P0, "Serra Angel"));
    }
}

#[test]
fn creature_edict_triggers_choose_in_turn_order() {
    cr!("101.4", "701.21a", "603.3b");
    ruling!(
        "Witch of the Moors",
        "As the triggered ability resolves, first the next opponent in turn order chooses a creature they control, then each other opponent in turn order does the same knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    check_edict("Witch of the Moors", P0, P1, BEARS_GIANT, |t| {
        t.battlefield(P0, "Witch of the Moors");
        t.g.gain_life(P0, 1);
        t.advance_to(P0, Step::End);
        go(t)
    });
    ruling!(
        "Pharika's Spawn",
        "As the triggered ability resolves, first the next opponent in turn order (or, if it's an opponent's turn, the opponent whose turn it is) chooses a creature they control, then each other opponent in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    check_edict("Pharika's Spawn", P0, P1, BEARS_GIANT, |t| {
        let spawn = t.graveyard(P0, "Pharika's Spawn");
        for _ in 0..3 {
            t.graveyard(P0, "Island");
        }
        add_mana(t, P0, ManaType::B, 6);
        t.cast(P0, spawn)
            .method(CastMethod::Keyword(KeywordKind::Escape))
            .go();
        t.resolve();
        assert_eq!(t.counters(spawn, mtg_engine::types::counters::PLUS1), 2);
        go(t)
    });
    ruling!(
        "Chittering Harvester",
        "As the triggered ability resolves, first the next opponent in turn order (or, if it’s an opponent’s turn, the opponent whose turn it is) chooses a creature they control, then each other opponent in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    check_edict("Chittering Harvester", P0, P1, BEARS_GIANT, |t| {
        let bear = t.battlefield(P0, "Runeclaw Bear");
        let harvester = t.hand(P0, "Chittering Harvester");
        add_mana(t, P0, ManaType::B, 5);
        t.cast(P0, harvester)
            .method(CastMethod::Keyword(KeywordKind::Mutate))
            .target(bear)
            .go();
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        t.resolve();
        assert_eq!(t.obj_now(bear).chars.name, "Chittering Harvester");
        go(t)
    });
}

#[test]
fn ravenous_rotbelly_reflexive_trigger_can_be_responded_to() {
    cr!("603.12", "101.4", "701.21a");
    ruling!(
        "Ravenous Rotbelly",
        "After you sacrifice one or more Zombies, a second ability triggers. Players may respond to it before they have to sacrifice creatures."
    );
    ruling!(
        "Ravenous Rotbelly",
        "As the reflexive triggered ability resolves, first the next opponent in turn order (or, if it's an opponent's turn, the opponent whose turn it is) chooses the creatures they plan to sacrifice. Then each other opponent in turn order does the same. Finally all chosen creatures are sacrificed at the same time."
    );
    check_edict("Ravenous Rotbelly", P0, P1, BEARS_GIANT, |t| {
        let zombie = t.battlefield(P0, "Walking Corpse");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(zombie)]);
        t.enter(P0, "Ravenous Rotbelly");
        t.resolve();
        // The Zombie is gone; the reflexive trigger waits on the stack, and the
        // opponents haven't sacrificed anything yet.
        assert!(t.in_graveyard(P0, "Walking Corpse"));
        assert_eq!(t.stack_len(), 1);
        assert_eq!(creatures(t, P1).len(), 2);
        assert_eq!(creatures(t, P2).len(), 2);
        go(t)
    });
}
