//! Rulings batch P138 — several players sacrificing at once: the players choose in APNAP
//! order knowing the earlier choices, then everything is sacrificed at the same time
//! (CR 101.4, 101.4a, 701.21a); ties for "greatest" or "least" are broken by the
//! sacrificing player's choice (CR 700.2 as applied to choices made on resolution,
//! 608.2d).

use crate::r_p138_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Watches the object choices of each player, recording (permanents on the battlefield,
/// earlier choices that player can see) at each.
fn watch_known(t: &mut TestGame, players: &[PlayerId]) -> Vec<Seen<(usize, usize)>> {
    players
        .iter()
        .map(|p| {
            let p = *p;
            match p.0 {
                0 => watch(t, p, is_choose, |g| known(g, PlayerId(0))),
                1 => watch(t, p, is_choose, |g| known(g, PlayerId(1))),
                2 => watch(t, p, is_choose, |g| known(g, PlayerId(2))),
                _ => watch(t, p, is_choose, |g| known(g, PlayerId(3))),
            }
        })
        .collect()
}

fn known(g: &mtg_engine::game::Game, viewer: PlayerId) -> (usize, usize) {
    (
        g.permanents().count(),
        g.known_apnap_choices(viewer)
            .iter()
            .filter(|(_, c)| c.is_some())
            .count(),
    )
}

/// The order of the distinct choosers since `from`.
fn order_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    let mut v = choosers_since(t, from);
    v.dedup();
    v
}

/// Asserts that each watched player chose while all `n` permanents were still there, and
/// saw the choices of the players before them (`seen[i]` saw `i` earlier choices).
fn assert_chose_before_any_sacrifice(seen: &[Seen<(usize, usize)>], n: usize, what: &str) {
    for (i, s) in seen.iter().enumerate() {
        let first = s.lock().unwrap().first().copied();
        assert_eq!(first, Some((n, i)), "{what}: chooser #{i}");
    }
}

#[test]
fn tectonic_hellion_players_with_the_most_lands_choose_in_turn_order() {
    cr!("101.4", "701.21a", "508.1m");
    ruling!(
        "Tectonic Hellion",
        "If more than one player will sacrifice lands as Tectonic Hellion’s triggered ability resolves, first the player whose turn it is chooses two lands they control (if they control the most lands), then each other player in turn order does the same if they control the most lands, knowing the choices made before them. Then all the chosen lands are sacrificed at the same time."
    );
    supported("Tectonic Hellion");
    let mut t = TestGame::new(3);
    let hellion = t.battlefield(P0, "Tectonic Hellion");
    t.lands(P0, "Mountain", 3);
    t.lands(P1, "Forest", 3);
    t.lands(P2, "Plains", 2);
    let seen = watch_known(&mut t, &[P0, P1]);
    let n = t.g.permanents().count();
    let from = t.asked().len();
    attack_with(&mut t, &[(hellion, pl(P1))]);
    t.resolve_all();
    assert_eq!(order_since(&t, from), vec![P0, P1]);
    assert_chose_before_any_sacrifice(&seen, n, "Tectonic Hellion");
    let lands = |t: &TestGame, p| {
        permanents_of(t, p)
            .into_iter()
            .filter(|id| t.obj(*id).is(types::CardType::Land))
            .count()
    };
    assert_eq!([lands(&t, P0), lands(&t, P1), lands(&t, P2)], [1, 1, 2]);
}

#[test]
fn acererak_opponents_decide_in_turn_order_then_all_happens_at_once() {
    cr!("101.4", "608.2d", "118.12");
    ruling!(
        "Acererak the Archlich",
        "In a multiplayer game, each opponent decides in turn order whether or not they will sacrifice a creature and which one they will sacrifice. Each opponent knows the choices made before they make their own choice. Then, once all opponents have chosen, creatures are sacrificed and tokens are created simultaneously."
    );
    supported("Acererak the Archlich");
    let mut t = TestGame::new(3);
    let acererak = t.battlefield(P0, "Acererak the Archlich");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let g1 = t.battlefield(P1, "Hill Giant");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P2, "Hill Giant");
    // P1 sacrifices the Bears; P2 declines.
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(b1)]);
    t.answer_yes(P2, false);
    let seen = watch(&mut t, P2, |d| matches!(d, decision::Decision::YesNo { .. }), |g| {
        g.permanents().count()
    });
    let n = t.g.permanents().count();
    let from = t.asked().len();
    attack_with(&mut t, &[(acererak, pl(P1))]);
    t.resolve_all();
    let askers: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(p, d)| *p != P0 && (is_choose(d) || matches!(d, decision::Decision::YesNo { .. })))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(askers.first(), Some(&P1));
    assert_eq!(askers.last(), Some(&P2));
    // P2 decided while P1's Bears were still there.
    assert_eq!(seen.lock().unwrap().first().copied(), Some(n));
    assert!(!t.on_battlefield(b1) && t.on_battlefield(g1) && t.on_battlefield(b2));
    let zombies = permanents_of(&t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Zombie") && t.obj(*id).is_token())
        .count();
    assert_eq!(zombies, 1);
}

#[test]
fn opponents_choose_in_turn_order_then_sacrifice_together() {
    cr!("101.4", "101.4a", "701.21a");
    ruling!(
        "Baleful Beholder",
        "In a multiplayer game, each opponent in turn order will choose which enchantment to sacrifice, and they get to know the choices made before them by other players. Once those decisions are made, the chosen enchantments are all sacrificed at the same time."
    );
    supported("Baleful Beholder");
    let mut t = TestGame::new(3);
    let mut keep = vec![];
    let mut lose = vec![];
    for p in [P1, P2] {
        let a = t.battlefield(p, "Glorious Anthem");
        let b = t.battlefield(p, "Bad Moon");
        t.answer_choose(p, &[obj(b)]);
        keep.push(a);
        lose.push(b);
    }
    let seen = watch_known(&mut t, &[P1, P2]);
    t.answer(P0, DecisionKind::Modes, decision::Answer::Indices(vec![0]));
    t.enter(P0, "Baleful Beholder");
    t.settle();
    let n = t.g.permanents().count();
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(order_since(&t, from), vec![P1, P2]);
    assert_chose_before_any_sacrifice(&seen, n, "Baleful Beholder");
    assert!(keep.iter().all(|k| t.on_battlefield(*k)));
    assert!(lose.iter().all(|l| !t.on_battlefield(*l)));
}

#[test]
fn starting_with_the_active_player_when_cast_on_an_opponents_turn() {
    cr!("101.4", "101.4a", "701.21a");
    ruling!(
        "Crackling Doom",
        "In a multiplayer game, each opponent, starting with the active player and proceeding in turn order, chooses which creature they will sacrifice and then all creatures are sacrificed at the same time. Each successive player will know what the players before them chose to sacrifice."
    );
    ruling!(
        "Flare of Malice",
        "Starting with the next opponent in turn order (or, if you cast Flare of Malice on an opponent's turn, starting with the opponent whose turn it is) and proceeding in turn order, each opponent chooses a creature or planeswalker with the greatest mana value among creatures and planeswalkers they control to sacrifice. Then those permanents are sacrificed at the same time."
    );
    for name in ["Crackling Doom", "Flare of Malice"] {
        supported(name);
        for active in [P0, P2] {
            let mut t = TestGame::new(4);
            t.set_step(active, Step::PrecombatMain);
            // Each opponent: two Hill Giants (tied for greatest power and mana value).
            let mut giants = vec![];
            for p in [P1, P2, P3] {
                giants.push(t.battlefield(p, "Hill Giant"));
                let g2 = t.battlefield(p, "Hill Giant");
                t.answer_choose(p, &[obj(g2)]);
            }
            let expected = if active == P0 {
                vec![P1, P2, P3]
            } else {
                vec![P2, P3, P1]
            };
            let seen = watch_known(&mut t, &expected);
            cast(&mut t, P0, name, &[]);
            let n = t.g.permanents().count();
            let from = t.asked().len();
            t.resolve_all();
            let what = format!("{name} on {active:?}'s turn");
            assert_eq!(order_since(&t, from), expected, "{what}");
            assert_chose_before_any_sacrifice(&seen, n, &what);
            assert!(giants.iter().all(|g| t.on_battlefield(*g)), "{what}");
            for p in [P1, P2, P3] {
                assert_eq!(creatures(&t, p).len(), 1, "{what}");
            }
        }
    }
}

#[test]
fn everyone_chooses_what_to_keep_in_turn_order_then_the_rest_go_at_once() {
    cr!("101.4", "101.4a", "701.21a");
    ruling!(
        "Razia's Purification",
        "Players choose permanents in turn order around the table, then simultaneously sacrifice all permanents not chosen."
    );
    ruling!(
        "Covetous Elegy",
        "Starting with the player whose turn it is, each player in turn order chooses up to two creatures they control, then all of the creatures not chosen by any player are sacrificed at the same time. Players get to know the choices made by players who chose before them."
    );
    for (name, keep) in [("Razia's Purification", 3usize), ("Covetous Elegy", 2)] {
        supported(name);
        let mut t = TestGame::new(3);
        let mut kept = vec![];
        let mut lost = vec![];
        for p in [P0, P1, P2] {
            let all: Vec<ObjectId> = (0..keep + 1)
                .map(|_| t.battlefield(p, "Grizzly Bears"))
                .collect();
            if name == "Covetous Elegy" {
                // "Up to two": one creature at a time.
                for id in &all[..keep] {
                    t.answer_choose(p, &[obj(*id)]);
                }
            } else {
                t.answer_choose(p, &all[..keep].iter().map(|id| obj(*id)).collect::<Vec<_>>());
            }
            kept.extend_from_slice(&all[..keep]);
            lost.push(all[keep]);
        }
        let seen = watch_known(&mut t, &[P0, P1, P2]);
        cast(&mut t, P0, name, &[]);
        let n = t.g.permanents().count();
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(order_since(&t, from), vec![P0, P1, P2], "{name}");
        assert_chose_before_any_sacrifice(&seen, n, name);
        assert!(kept.iter().all(|k| t.on_battlefield(*k)), "{name}");
        assert!(lost.iter().all(|l| !t.on_battlefield(*l)), "{name}");
        if name == "Covetous Elegy" {
            // A Treasure for each creature the opponents control (four).
            let treasures = permanents_of(&t, P0)
                .into_iter()
                .filter(|id| t.obj(*id).chars.has_subtype("Treasure"))
                .count();
            assert_eq!(treasures, 4);
        }
    }
}

#[test]
fn ties_for_greatest_or_least_are_broken_by_the_sacrificing_player() {
    cr!("701.21a", "608.2d");
    ruling!(
        "Flare of Malice",
        "If an opponent has multiple creatures and/or planeswalkers tied for the greatest mana value, that player chooses which one to sacrifice."
    );
    ruling!(
        "Consecrate // Consume",
        "If the target player controls more than one creature with the greatest power among creatures they control, that player chooses which to sacrifice."
    );
    ruling!(
        "Witch-king, Bringer of Ruin",
        "If the defending player controls two or more creatures tied for the least power among creatures they control, that player chooses one of them to sacrifice."
    );
    // Flare of Malice: a Hill Giant and Garruk Wildspeaker (both mana value 4) and a
    // Grizzly Bears; P1 sacrifices Garruk.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(garruk)]);
    cast(&mut t, P0, "Flare of Malice", &[]);
    let from = t.asked().len();
    t.resolve_all();
    let mut offer = first_offer(&t, P1, from);
    offer.sort();
    assert_eq!(offer, vec![giant, garruk], "Flare of Malice");
    assert!(!t.on_battlefield(garruk) && t.on_battlefield(giant) && t.on_battlefield(bears));

    // Consume: two Hill Giants (power 3) and a Grizzly Bears.
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(g2)]);
    lots(&mut t, P0);
    let c = t.hand(P0, "Consecrate // Consume");
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, c)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    let from = t.asked().len();
    t.resolve_all();
    let mut offer = first_offer(&t, P1, from);
    offer.sort();
    assert_eq!(offer, vec![g1, g2], "Consume");
    assert!(!t.on_battlefield(g2) && t.on_battlefield(g1) && t.on_battlefield(bears));
    assert_eq!(t.life(P0), 23);

    // Witch-king: two Grizzly Bears (power 2) and a Hill Giant.
    let mut t = TestGame::new(2);
    let wk = t.battlefield(P0, "Witch-king, Bringer of Ruin");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(b2)]);
    let from = t.asked().len();
    attack_with(&mut t, &[(wk, pl(P1))]);
    t.resolve_all();
    let mut offer = first_offer(&t, P1, from);
    offer.sort();
    assert_eq!(offer, vec![b1, b2], "Witch-king");
    assert!(!t.on_battlefield(b2) && t.on_battlefield(b1) && t.on_battlefield(giant));
}
