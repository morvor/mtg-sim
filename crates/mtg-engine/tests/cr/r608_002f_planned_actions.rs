//! CR 608.2f, 701.21a, 118.12b, 607.2c: actions planned for every player of an "each
//! player" instruction and performed together (`simultaneous.rs`) do what the instruction
//! does for one player: "sacrifice ~" is done only by its controller, "if they do" after a
//! search is whether they searched, and what a player puts onto the battlefield is put
//! there with the source.

use crate::r600_common::{act, mana_cost, CB};
use crate::r608_each_player_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// An artifact with "{0}: For each opponent, [e]."
fn for_each_opponent_artifact(e: Effect) -> CardDef {
    CB::new("Each Opponent Engine")
        .artifact()
        .cost("{0}")
        .ability(act(mana_cost("{0}"), Body::effect(for_each_opponent(e))))
        .build()
}

#[test]
fn sacrifice_this_is_done_by_its_controller_only() {
    cr!("701.21a", "608.2f");
    let sac_this = || Effect::SacrificeObjects { what: Sel::This };
    // (row, the instruction for each opponent, whether the artifact is sacrificed)
    let rows: Vec<(&str, Effect, bool)> = vec![
        // "For each opponent, sacrifice ~ unless that player pays 2 life." P1 declines:
        // you, its controller, sacrifice it.
        (
            "you sacrifice it",
            Effect::PayOptional {
                who: PlayerRef::Iterated,
                cost: Cost::free().with(CostPart::PayLife(Value::c(2))),
                then: Box::new(Effect::Noop),
                otherwise: Box::new(sac_this()),
            },
            true,
        ),
        // "Each opponent sacrifices ~.": a player can't sacrifice a permanent they don't
        // control.
        (
            "they sacrifice it",
            Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(sac_this()),
            },
            false,
        ),
    ];
    for (row, e, sacrificed) in rows {
        let mut t = TestGame::new(2);
        let src = t.custom(P0, for_each_opponent_artifact(e), Zone::Battlefield);
        t.answer_yes(P1, false);
        t.activate(P0, src, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.on_battlefield(src), !sacrificed, "{row}");
    }
}

#[test]
fn each_opponents_if_they_do_after_their_search_is_whether_they_searched() {
    cr!("118.12b", "608.2e");
    // "Each opponent searches their library for a basic land card and puts it onto the
    // battlefield. If they do, they gain 2 life." (The each-player form of
    // `r118_costs::a_player_who_cant_search_doesnt_search_so_if_you_do_is_false`.)
    let body = Effect::ForEachPlayer {
        who: PlayerRef::EachOpponent,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    whose: PlayerRef::You,
                    filter: Filter::and(vec![
                        Filter::Type(CardType::Land),
                        Filter::Supertype(Supertype::Basic),
                    ]),
                    count: Value::c(1),
                    to: Destination::battlefield(),
                    reveal: false,
                    shuffle: false,
                },
                Effect::If {
                    cond: Condition::PrevHappened,
                    then: Box::new(Effect::GainLife {
                        who: PlayerRef::You,
                        n: Value::c(2),
                    }),
                    otherwise: Box::new(Effect::Noop),
                },
            ])),
        }),
    };
    for stranglehold in [false, true] {
        let mut t = TestGame::new(2);
        if stranglehold {
            // P0's: "Your opponents can't search libraries."
            t.battlefield(P0, "Stranglehold");
        }
        t.library_top(P1, "Forest");
        cast_body(&mut t, body.clone());
        t.resolve_all();
        assert_eq!(
            t.life(P1),
            if stranglehold { 20 } else { 22 },
            "Stranglehold: {stranglehold}"
        );
    }
}

#[test]
fn creatures_each_player_puts_onto_the_battlefield_are_put_there_with_the_source() {
    cr!("607.2c", "608.2f");
    // "{0}: Each player returns a creature card from their graveyard to the battlefield."
    // "{0}: Exile each creature put onto the battlefield with this artifact."
    let each_returns = each_player(Effect::Move {
        what: you_choose(Filter::and(vec![
            Filter::creature(),
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ])),
        to: Destination::battlefield(),
    });
    let exile_them = Effect::Exile {
        what: Sel::All(Filter::and(vec![
            Filter::creature(),
            Filter::InZone(ZoneKind::Battlefield),
            Filter::In(Box::new(Sel::Linked)),
        ])),
        face_down: false,
        link: false,
    };
    let def = CB::new("Linked Reanimator")
        .artifact()
        .cost("{0}")
        .ability(act(mana_cost("{0}"), Body::effect(each_returns)))
        .ability(act(mana_cost("{0}"), Body::effect(exile_them)))
        .build();
    let mut t = TestGame::new(2);
    let src = t.custom(P0, def, Zone::Battlefield);
    // Not put onto the battlefield with it.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.activate(P0, src, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    t.activate(P0, src, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    assert!(t.on_battlefield(elves));
}

#[test]
fn a_card_put_into_exile_instead_of_onto_the_battlefield_wasnt_returned() {
    cr!("614.6", "608.2c", "608.2f");
    // "Each player returns a creature card from their graveyard to the battlefield. Each
    // player who does gains 2 life." P0's Containment Priest ("If a nontoken creature would
    // enter and it wasn't cast, exile it instead."): both cards are exiled instead, so
    // nobody returned one.
    let body = each_player(Effect::Seq(vec![
        Effect::Move {
            what: you_choose(Filter::and(vec![
                Filter::creature(),
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
            ])),
            to: Destination::battlefield(),
        },
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::c(2),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Containment Priest");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    cast_body(&mut t, body);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}
