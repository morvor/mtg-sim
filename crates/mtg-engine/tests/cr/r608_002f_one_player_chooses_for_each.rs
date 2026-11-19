//! CR 608.2d, 608.2f, 101.4b: an instruction the resolving player performs for each of
//! several players ("for each opponent, sacrifice a creature"): it's performed for each
//! opponent in turn (`simultaneous.rs`, module doc), so an object acted on for one opponent
//! isn't chosen again for the next, the choices are recorded as the resolving player's,
//! not the opponents', and "the damage dealt this way" counts each opponent's damage once.

use crate::r608_each_player_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// "a creature card in your graveyard" / "a creature you control", chosen by you.
fn yours(zone: ZoneKind) -> Sel {
    let whose = if zone == ZoneKind::Battlefield {
        Filter::ControlledBy(PlayerRel::You)
    } else {
        Filter::OwnedBy(PlayerRel::You)
    };
    you_choose(Filter::and(vec![
        Filter::creature(),
        Filter::InZone(zone),
        whose,
    ]))
}

#[test]
fn what_you_act_on_for_one_opponent_isnt_chosen_again_for_the_next() {
    interp!("choices-for-each-player-made-in-turn");
    cr!("608.2d", "608.2f", "701.21a", "701.9a", "701.23h");
    let basic_land = Filter::and(vec![
        Filter::Supertype(Supertype::Basic),
        Filter::Type(CardType::Land),
    ]);
    // (row, what you do for each opponent, where the two objects start, where both end)
    let rows: Vec<(&str, Effect, Zone, Zone)> = vec![
        (
            "return",
            Effect::Move {
                what: yours(ZoneKind::Graveyard),
                to: Destination::zone(ZoneKind::Hand),
            },
            Zone::Graveyard(P0),
            Zone::Hand(P0),
        ),
        (
            "exile",
            Effect::Exile {
                what: yours(ZoneKind::Graveyard),
                face_down: false,
                link: false,
            },
            Zone::Graveyard(P0),
            Zone::Exile,
        ),
        (
            "sacrifice it",
            Effect::SacrificeObjects {
                what: yours(ZoneKind::Battlefield),
            },
            Zone::Battlefield,
            Zone::Graveyard(P0),
        ),
        (
            "sacrifice",
            Effect::Sacrifice {
                who: PlayerRef::You,
                filter: Filter::creature(),
                count: Value::c(1),
            },
            Zone::Battlefield,
            Zone::Graveyard(P0),
        ),
        (
            "discard",
            Effect::Discard {
                who: PlayerRef::You,
                n: Value::c(1),
                random: false,
                filter: Filter::Any,
            },
            Zone::Hand(P0),
            Zone::Graveyard(P0),
        ),
        (
            "search",
            Effect::Search {
                who: PlayerRef::You,
                whose: PlayerRef::You,
                filter: basic_land,
                count: Value::c(1),
                to: Destination::zone(ZoneKind::Hand),
                reveal: false,
                shuffle: false,
            },
            Zone::Library(P0),
            Zone::Hand(P0),
        ),
    ];
    for (row, e, from, to) in rows {
        // Two opponents: P0 performs the instruction twice, once for each.
        let mut t = TestGame::new(3);
        let names = match from {
            Zone::Hand(_) | Zone::Library(_) => ["Forest", "Island"],
            _ => ["Grizzly Bears", "Hill Giant"],
        };
        let objs: Vec<ObjectId> = names
            .iter()
            .map(|n| match from {
                Zone::Graveyard(_) => t.graveyard(P0, n),
                Zone::Battlefield => t.battlefield(P0, n),
                Zone::Hand(_) => t.hand(P0, n),
                _ => t.library_top(P0, n),
            })
            .collect();
        // P0 names the same object both times. Performed in turn, the second answer is
        // invalid (that object has already moved) and the other object is taken.
        t.answer_choose(P0, &[Entity::Object(objs[0])]);
        t.answer_choose(P0, &[Entity::Object(objs[0])]);
        cast_body(&mut t, for_each_opponent(e));
        t.resolve_all();
        for (n, o) in names.iter().zip(&objs) {
            assert_eq!(t.zone(t.g.current(*o)), to, "{row}: {n}");
        }
    }
}

#[test]
fn a_choice_you_make_for_an_opponent_isnt_recorded_as_that_opponents() {
    cr!("101.4", "101.4b");
    interp!("choices-for-each-player-made-in-turn");
    // (row, what you do for each opponent, objects in the graveyard rather than on the
    // battlefield)
    let rows: Vec<(&str, Effect, bool)> = vec![
        (
            "return",
            Effect::Move {
                what: yours(ZoneKind::Graveyard),
                to: Destination::zone(ZoneKind::Hand),
            },
            true,
        ),
        (
            "exile",
            Effect::Exile {
                what: yours(ZoneKind::Graveyard),
                face_down: false,
                link: false,
            },
            true,
        ),
        (
            "counter",
            Effect::AddCounters {
                what: yours(ZoneKind::Battlefield),
                kind: counters::PLUS1.into(),
                n: Value::c(1),
            },
            false,
        ),
        (
            "copy",
            Effect::CreateTokenCopy {
                of: yours(ZoneKind::Battlefield),
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
                mods: vec![],
            },
            false,
        ),
    ];
    for (row, e, graveyard) in rows {
        let mut t = TestGame::new(3);
        let objs: Vec<ObjectId> = ["Grizzly Bears", "Hill Giant"]
            .iter()
            .map(|n| {
                if graveyard {
                    t.graveyard(P0, n)
                } else {
                    t.battlefield(P0, n)
                }
            })
            .collect();
        // A different object for each opponent.
        for o in &objs {
            t.answer_choose(P0, &[Entity::Object(*o)]);
        }
        let seen = watch(&mut t, P0, is_choose, known);
        cast_body(&mut t, for_each_opponent(e));
        t.resolve_all();
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 2, "{row}");
        // P0 made both choices: neither opponent is recorded as having chosen anything.
        assert!(
            seen.iter().flatten().all(|(who, _)| *who == P0),
            "{row}: {seen:?}"
        );
    }
}

#[test]
fn the_damage_dealt_this_way_counts_each_opponents_damage_once() {
    cr!("608.2c", "608.2f", "119.3");
    interp!("choices-for-each-player-made-in-turn");
    // "For each opponent, this spell deals 2 damage to a creature of your choice. You gain
    // life equal to the damage dealt this way." P0 picks the same Craw Wurm both times.
    let body = Effect::Seq(vec![
        for_each_opponent(Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(2),
            to: you_choose(Filter::and(vec![
                Filter::creature(),
                Filter::InZone(ZoneKind::Battlefield),
            ])),
        }),
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::Prev,
        },
    ]);
    let mut t = TestGame::new(3);
    // 6/4: the 4 damage is lethal once the spell has resolved.
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    cast_body(&mut t, body);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(t.life(P0), 24);
}

#[test]
fn each_player_searches_once_for_each_opponent() {
    cr!("701.23i", "608.2f");
    // "For each opponent, each player searches their library for a basic land card, puts
    // it into their hand, then shuffles." The search isn't one the instruction's player
    // makes alone: it's performed as written for each opponent in turn, every player
    // searching each time.
    let body = for_each_opponent(Effect::Search {
        who: PlayerRef::EachPlayer,
        whose: PlayerRef::Iterated,
        filter: Filter::and(vec![
            Filter::Supertype(Supertype::Basic),
            Filter::Type(CardType::Land),
            Filter::InZone(ZoneKind::Library),
        ]),
        count: Value::c(1),
        to: Destination::zone(ZoneKind::Hand),
        reveal: false,
        shuffle: true,
    });
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        t.library_top(p, "Forest");
        t.library_top(p, "Island");
    }
    cast_body(&mut t, body);
    let hands = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.resolve_all();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (hands.0 + 2, hands.1 + 2, hands.2 + 2)
    );
    for p in [P0, P1, P2] {
        assert!(t.in_hand(p, "Forest") && t.in_hand(p, "Island"), "{p}");
    }
}
