//! CR 608.2c, 608.2e, 603.2c, 603.10: how an "each player" body is split into
//! instructions and what the players' instructions share (`simultaneous.rs`):
//! the events of one instruction are checked for triggers before the next begins; "create
//! a [token] and a [token]" is one instruction; a running total stored by each player's
//! instruction collects every player's part, and "the life lost this way" totals only the
//! players who lost life.

use crate::r603_audit_common::triggers_of;
use crate::r608_each_player_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const N: Var = vars::USER + 4251;

#[test]
fn each_instructions_events_are_checked_for_triggers_before_the_next() {
    cr!("603.2", "603.10", "608.2e");
    // "Each player discards a card, then sacrifices a creature." P0's only creature is
    // Archfiend of Ifnir ("Whenever you cycle or discard another card, put a -1/-1 counter
    // on each creature your opponents control."): the discards are checked while it's
    // still on the battlefield, before it's sacrificed.
    let body = each_player(Effect::Seq(vec![
        Effect::Discard {
            who: PlayerRef::You,
            n: Value::c(1),
            random: false,
            filter: Filter::Any,
        },
        Effect::Sacrifice {
            who: PlayerRef::You,
            filter: Filter::creature(),
            count: Value::c(1),
        },
    ]));
    let mut t = TestGame::new(2);
    let archfiend = t.battlefield(P0, "Archfiend of Ifnir");
    t.hand(P0, "Island");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.hand(P1, "Island");
    t.answer_choose(P1, &[Entity::Object(bears)]);
    cast_body(&mut t, body);
    t.resolve();
    assert!(!t.on_battlefield(archfiend) && !t.on_battlefield(bears));
    assert_eq!(triggers_of(&t, "Archfiend of Ifnir"), 1);
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn creating_two_kinds_of_tokens_is_one_instruction() {
    cr!("603.2c", "608.2e");
    // "Each player creates a Treasure token and a Clue token." P0's Ingenious Artillerist
    // ("Whenever one or more artifacts you control enter, this creature deals that much
    // damage to each opponent.") sees P0's two tokens enter as one event.
    let token = |name: &str| Effect::CreateToken {
        spec: mtg_engine::tokens::predefined(name).unwrap(),
        count: Value::c(1),
        controller: PlayerRef::You,
        tapped: false,
        attacking: false,
    };
    let body = each_player(Effect::Seq(vec![token("Treasure"), token("Clue")]));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ingenious Artillerist");
    cast_body(&mut t, body);
    t.resolve();
    assert_eq!(triggers_of(&t, "Ingenious Artillerist"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

/// "[before]. [Each player ...]. You gain life equal to N", N counted by each player's
/// part (`count` adds 1 to it).
fn counted(each: Effect) -> Effect {
    Effect::Seq(vec![
        Effect::StoreValue {
            var: N,
            value: Value::c(0),
        },
        each,
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::Var(N),
        },
    ])
}

fn count() -> Effect {
    Effect::StoreValue {
        var: N,
        value: Value::Sum(vec![Value::Var(N), Value::c(1)]),
    }
}

#[test]
fn a_running_count_over_each_players_coin_flip_counts_every_player() {
    cr!("608.2e", "705.2");
    // "Each player flips a coin. You gain 1 life for each coin flipped this way." (heads or
    // tails, each flip counts.)
    let flip = Effect::FlipCoins(Box::new(mtg_engine::dice::CoinFlip {
        who: PlayerRef::You,
        call: false,
        on_heads: count(),
        on_tails: count(),
        ..mtg_engine::dice::CoinFlip::new()
    }));
    let mut t = TestGame::new(3);
    cast_body(&mut t, counted(each_player(flip)));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_running_count_of_the_players_who_accept_counts_every_player() {
    cr!("101.4", "608.2e");
    // "Each player may [accept]. You gain 1 life for each player who accepted."
    let accept = Effect::May {
        who: PlayerRef::You,
        effect: Box::new(count()),
    };
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        t.answer_yes(p, true);
    }
    cast_body(&mut t, counted(each_player(accept)));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

/// "Draw a card. Each opponent who controls a creature loses N life. You gain life equal to
/// the life lost this way."
fn drain_after_draw(n: i32) -> Effect {
    Effect::Seq(vec![
        draw(1),
        Effect::ForEachPlayer {
            who: PlayerRef::EachOpponent,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::If {
                    cond: Condition::Exists(creature_you_control()),
                    then: Box::new(Effect::LoseLife {
                        who: PlayerRef::You,
                        n: Value::c(n),
                    }),
                    otherwise: Box::new(Effect::Noop),
                }),
            }),
        },
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::Prev,
        },
    ])
}

#[test]
fn the_life_lost_this_way_counts_only_the_players_who_lost_life() {
    cr!("608.2c", "119.3");
    // P1 controls a creature, P2 doesn't: the life lost this way is P1's.
    for n in [2, 1] {
        let mut t = TestGame::new(3);
        t.battlefield(P1, "Grizzly Bears");
        cast_body(&mut t, drain_after_draw(n));
        t.resolve_all();
        assert_eq!(
            (t.life(P0), t.life(P1), t.life(P2)),
            (20 + n, 20 - n, 20),
            "N = {n}"
        );
    }
}
