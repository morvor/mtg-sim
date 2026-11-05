//! Rulings batch P050 (second round) — drain cards in interactions: state-based actions
//! and triggers (CR 704, 603), cost increases before reductions (CR 601.2f), control
//! changes (CR 108.4, 603.3a), replacement and prevention of damage (CR 614, 615), last
//! known information about an Aura (CR 608.2h), and permissions to cast from a graveyard.

use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::supported;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s33_common::set_life;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------------
// Circle of Affliction
// ---------------------------------------------------------------------------------------

/// Circle of Affliction (red chosen) and an Island for P0 at `life`; P1 casts Lightning
/// Bolt at P0, and P0 pays {1} for the trigger, targeting P1.
fn circle(life: i32) -> TestGame {
    supported("Circle of Affliction");
    let mut t = TestGame::new(2);
    crate::r_p125_common::choose_color(&mut t, P0, Color::Red);
    crate::r_s05_common::enter(&mut t, P0, "Circle of Affliction");
    t.lands(P0, "Island", 1);
    set_life(&mut t, P0, life);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    t
}

#[test]
fn circle_of_affliction_you_lose_before_the_ability_gives_you_life() {
    cr!("704.3", "704.5a", "603.3");
    ruling!(
        "Circle of Affliction",
        "If the damage reduces you to 0 or less life, you lose the game before Circle of Affliction’s ability has a chance to give you life."
    );
    let t = circle(3);
    assert!(t.has_lost(P0));
    assert_eq!(t.life(P0), 0);
    assert_eq!(t.life(P1), 20, "the ability never resolved");
    // Control: at 10 life, the ability resolves.
    let t = circle(10);
    assert!(!t.has_lost(P0));
    assert_eq!((t.life(P0), t.life(P1)), (8, 19));
}

// ---------------------------------------------------------------------------------------
// Tezzeret, Master of the Bridge
// ---------------------------------------------------------------------------------------

#[test]
fn tezzeret_affinity_applies_after_cost_increases() {
    cr!("601.2f", "702.41a");
    ruling!(
        "Tezzeret, Master of the Bridge",
        "If there are any additional costs or cost increases that would apply to a spell with affinity, apply those before applying any cost reductions."
    );
    supported("Tezzeret, Master of the Bridge");
    supported("Sphere of Resistance");
    // Sphere of Resistance ("Spells cost {1} more to cast.") and two Ornithopters: three
    // artifacts. Llanowar Elves ({G}) costs {1}{G}, then {1} less for each artifact (only
    // the generic part can be reduced): {G}. Reducing first and then increasing would
    // leave {1}{G}.
    for with_tezzeret in [true, false] {
        let mut t = TestGame::new(2);
        if with_tezzeret {
            t.battlefield(P0, "Tezzeret, Master of the Bridge");
        }
        t.battlefield(P0, "Sphere of Resistance");
        t.battlefield(P0, "Ornithopter");
        t.battlefield(P0, "Ornithopter");
        t.lands(P0, "Forest", 1);
        let elves = t.hand(P0, "Llanowar Elves");
        let cast = t.cast(P0, elves).try_go().is_ok();
        assert_eq!(cast, with_tezzeret, "Tezzeret: {with_tezzeret}");
        if cast {
            t.resolve_all();
            assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
            assert_eq!(t.g.player(P0).mana_pool.total(), 0);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Sorin the Mirthless
// ---------------------------------------------------------------------------------------

#[test]
fn sorin_the_mirthless_unrevealed_card_stays_on_top() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Sorin the Mirthless",
        "If you choose not to reveal the card you looked at with Sorin's first loyalty ability, it stays on top of your library."
    );
    supported("Sorin the Mirthless");
    for reveal in [false, true] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Sorin the Mirthless");
        let giant = t.library_top(P0, "Hill Giant");
        let lib = t.library_size(P0);
        t.answer_yes(P0, reveal);
        t.activate(P0, s, 0, &[]).unwrap();
        t.resolve_all();
        if reveal {
            assert!(t.in_hand(P0, "Hill Giant"));
            assert_eq!(t.life(P0), 16, "Hill Giant's mana value");
            assert_eq!(t.library_size(P0), lib - 1);
        } else {
            assert_eq!(t.g.player(P0).library.last(), Some(&giant), "still on top");
            assert_eq!(t.library_size(P0), lib);
            assert_eq!(t.life(P0), 20);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Obzedat, Ghost Council
// ---------------------------------------------------------------------------------------

#[test]
fn obzedat_controlled_until_end_of_turn_returns_at_your_next_upkeep() {
    cr!("603.7a", "603.7d", "610.3c", "110.2");
    ruling!(
        "Obzedat, Ghost Council",
        "If you gain control of Obzedat \"until end of turn,\" you'll control it during your end step and you choose whether to exile it. If you do, it returns to the battlefield under its owner's control at the beginning of your next upkeep, not its owner's next upkeep."
    );
    supported("Obzedat, Ghost Council");
    supported("Act of Treason");
    let mut t = TestGame::new(2);
    let obz = t.battlefield(P1, "Obzedat, Ghost Council");
    cast_new(&mut t, P0, "Act of Treason", &[obj(obz)]);
    t.resolve_all();
    assert_eq!(t.obj_now(obz).controller, P0);
    // P0's end step: P0 is asked, and exiles it.
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let yes_nos: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(yes_nos, vec![P0]);
    assert!(t.in_exile("Obzedat, Ghost Council"));
    // Not at its owner's (P1's) upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_exile("Obzedat, Ghost Council"));
    t.advance_to(P1, Step::Draw);
    assert!(t.in_exile("Obzedat, Ghost Council"));
    // At P0's next upkeep, under its owner's control.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let now = t.named_on_battlefield("Obzedat, Ghost Council");
    assert_eq!(now.len(), 1);
    assert_eq!(t.g.obj(now[0]).controller, P1);
}

// ---------------------------------------------------------------------------------------
// Roiling Horror
// ---------------------------------------------------------------------------------------

#[test]
fn roiling_horror_dies_when_your_life_isnt_higher() {
    cr!("704.5f", "208.2a");
    ruling!(
        "Roiling Horror",
        "If your life total is less than or equal to the life total of the opponent with the most life, Roiling Horror's toughness is 0 or less. It will be put into its owner's graveyard as a state-based action."
    );
    supported("Roiling Horror");
    // (P0's life, P1's, P2's): P0 minus the opponent with the most life.
    for (lives, pt) in [
        ((20, 20, 20), None),
        ((15, 18, 10), None),
        ((18, 18, 25), None),
        ((25, 18, 10), Some((7, 7))),
        ((25, 10, 23), Some((2, 2))),
    ] {
        let mut t = TestGame::new(3);
        set_life(&mut t, P0, lives.0);
        set_life(&mut t, P1, lives.1);
        set_life(&mut t, P2, lives.2);
        let h = t.battlefield(P0, "Roiling Horror");
        t.settle();
        match pt {
            None => {
                assert!(t.in_graveyard(P0, "Roiling Horror"), "{lives:?}");
            }
            Some(pt) => {
                assert!(t.on_battlefield(h), "{lives:?}");
                assert_eq!(t.pt(h), pt, "{lives:?}");
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Kaya, Orzhov Usurper
// ---------------------------------------------------------------------------------------

/// P1 owns three cards in exile; `setup` runs; then Kaya's −5 targets P1.
fn kaya_usurper(setup: fn(&mut TestGame)) -> TestGame {
    supported("Kaya, Orzhov Usurper");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.exile(P1, "Grizzly Bears");
    }
    t.exile(P0, "Grizzly Bears");
    setup(&mut t);
    let k = crate::r_p007_common::pw(&mut t, P0, "Kaya, Orzhov Usurper", 5);
    t.activate(P0, k, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t
}

#[test]
fn kaya_orzhov_usurper_gains_the_number_of_cards_not_the_damage() {
    cr!("614.1a", "615.1a", "608.2c");
    ruling!(
        "Kaya, Orzhov Usurper",
        "Kaya's last ability causes you to gain life equal to the number of cards the target player owns in exile, even if an effect causes her to deal more or less than that amount of damage to that player."
    );
    // Plain: 3 damage, 3 life.
    let t = kaya_usurper(|_| {});
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
    // Furnace of Rath doubles the damage: 6 damage, still 3 life.
    let t = kaya_usurper(|t| {
        t.battlefield(P0, "Furnace of Rath");
    });
    assert_eq!((t.life(P0), t.life(P1)), (23, 14));
    // Healing Salve prevents the next 3 damage to P1: no damage, still 3 life.
    let t = kaya_usurper(|t| {
        supported("Healing Salve");
        lands_for_cost(t, P1, "Healing Salve");
        let c = t.hand(P1, "Healing Salve");
        t.cast(P1, c).modes(&[1]).target(P1).go();
        t.resolve_all();
    });
    assert_eq!((t.life(P0), t.life(P1)), (23, 20));
}

// ---------------------------------------------------------------------------------------
// Kaya, Intangible Slayer
// ---------------------------------------------------------------------------------------

#[test]
fn kaya_intangible_slayer_checks_whether_it_was_an_aura_on_the_battlefield() {
    cr!("608.2h", "702.103b", "702.103f");
    ruling!(
        "Kaya, Intangible Slayer",
        "Kaya's last ability checks to see whether the permanent was an Aura when it last existed on the battlefield, not whether it is an Aura card in exile. For example, if an enchantment creature with the bestow ability (an ability that allows a creature card to be cast as an Aura instead) is an Aura and the target of Kaya's last ability, the ability will not create a copy of it."
    );
    supported("Kaya, Intangible Slayer");
    supported("Boon Satyr");
    for bestowed in [true, false] {
        let mut t = TestGame::new(2);
        let satyr = if bestowed {
            let bears = t.battlefield(P1, "Grizzly Bears");
            t.lands(P1, "Forest", 5);
            let s = t.hand(P1, "Boon Satyr");
            t.cast(P1, s)
                .method(CastMethod::Keyword(KeywordKind::Bestow))
                .target(bears)
                .go();
            t.resolve_all();
            let s = t.g.current(s);
            assert!(t.g.obj(s).chars.has_subtype("Aura"));
            assert_eq!(t.pt(bears), (6, 4));
            s
        } else {
            t.battlefield(P1, "Boon Satyr")
        };
        let k = t.battlefield(P0, "Kaya, Intangible Slayer");
        t.activate(P0, k, 2, &[obj(satyr)]).unwrap();
        t.resolve_all();
        assert!(t.in_exile("Boon Satyr"), "bestowed: {bestowed}");
        let tokens: Vec<_> =
            t.g.permanents()
                .filter(|o| o.is_token() && o.controller == P0)
                .map(|o| o.id)
                .collect();
        if bestowed {
            assert!(tokens.is_empty(), "no copy of an Aura");
        } else {
            assert_eq!(tokens.len(), 1);
            let o = t.g.obj(tokens[0]);
            assert_eq!(o.chars.name.as_str(), "Boon Satyr");
            assert!(o.chars.has_subtype("Spirit"));
            assert_eq!(t.pt(tokens[0]), (1, 1));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Obelisk Spider
// ---------------------------------------------------------------------------------------

#[test]
fn obelisk_spider_puts_one_counter_however_much_damage() {
    cr!("510.2", "603.2c");
    ruling!(
        "Obelisk Spider",
        "Obelisk Spider’s first triggered ability puts only one -1/-1 counter on the creature, no matter how much combat damage it dealt."
    );
    supported("Obelisk Spider");
    // Giant Growth makes the Spider 4/7; it deals 4 combat damage to a blocking Colossal
    // Dreadmaw (6/6).
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Obelisk Spider");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Giant Growth", &[obj(s)]);
    t.resolve_all();
    assert_eq!(t.pt(s), (4, 7));
    t.attack(&[(s, Entity::Player(P1))], &[(dreadmaw, s)]);
    t.resolve_all();
    assert!(t.on_battlefield(dreadmaw));
    assert_eq!(t.obj_now(dreadmaw).damage, 4);
    assert_eq!(t.counters(dreadmaw, "-1/-1"), 1);
    assert_eq!(t.pt(dreadmaw), (5, 5));
    // The last ability triggered once.
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

// ---------------------------------------------------------------------------------------
// Liliana, Untouched by Death
// ---------------------------------------------------------------------------------------

#[test]
fn liliana_zombie_cards_entering_the_graveyard_later_may_be_cast() {
    cr!("601.3", "611.3a");
    ruling!(
        "Liliana, Untouched by Death",
        "Once Liliana’s third ability has resolved, any Zombie cards in your graveyard may be cast this turn, including Zombie cards that enter your graveyard later in the turn."
    );
    supported("Liliana, Untouched by Death");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let before = t.graveyard(P0, "Diregraf Ghoul");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let ghoul = t.battlefield(P0, "Diregraf Ghoul");
    assert!(legal_cast_methods(&mut t, P0, before).is_empty());
    let l = t.battlefield(P0, "Liliana, Untouched by Death");
    t.activate(P0, l, 2, &[]).unwrap();
    t.resolve_all();
    assert!(!legal_cast_methods(&mut t, P0, before).is_empty());
    assert!(
        legal_cast_methods(&mut t, P0, bears).is_empty(),
        "not a Zombie"
    );
    // A Zombie card put into the graveyard afterwards.
    crate::r_s02_common::destroy(&mut t, ghoul);
    let later = t.g.current(ghoul);
    assert_eq!(t.zone(later), Zone::Graveyard(P0));
    let m = legal_cast_methods(&mut t, P0, later);
    assert!(!m.is_empty());
    t.cast(P0, later).method(m[0].clone()).go();
    t.resolve_all();
    assert!(t.on_battlefield(later));
}

// ---------------------------------------------------------------------------------------
// Sol'Kanar the Tainted
// ---------------------------------------------------------------------------------------

#[test]
fn sol_kanar_controlled_by_an_opponent() {
    cr!("603.3a", "700.2b", "102.2");
    ruling!(
        "Sol'Kanar the Tainted",
        "Once Sol'kanar the Tainted is controlled by your opponent, its ability triggers during that player's end step and that player makes all choices for it. That player draws a card for its first mode and gains life for its second mode, that player's opponents lose life for its second mode, and that player chooses one of their opponents for its last mode."
    );
    supported("Sol'Kanar the Tainted");
    // Three players. P0's end step: the last mode, giving Sol'Kanar to P1.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Sol'Kanar the Tainted");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3]));
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let sk = t.named_on_battlefield("Sol'Kanar the Tainted");
    assert_eq!(sk.len(), 1);
    assert_eq!(t.g.obj(sk[0]).controller, P1);
    // The modes asked of a player since `from`, and the players they're asked of.
    let modes_since = |t: &TestGame, from: usize| -> Vec<(PlayerId, Vec<usize>)> {
        t.asked()[from..]
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseModes { available, .. } => Some((*p, available.clone())),
                _ => None,
            })
            .collect()
    };
    // P0's end step: it doesn't trigger for P0 any more.
    t.advance_to(P1, Step::Upkeep);
    // P1's end step: P1 chooses the second mode; P1 gains 2, each of P1's opponents loses 2.
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    let from = t.asked().len();
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(modes_since(&t, from), vec![(P1, vec![0, 1, 2, 3])]);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (18, 22, 18));
    // P1's next end step: the first mode; P1 draws.
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.advance_to(P1, Step::PostcombatMain);
    let hands = (t.hand_size(P0), t.hand_size(P1));
    let from = t.asked().len();
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(modes_since(&t, from), vec![(P1, vec![0, 2, 3])]);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (hands.0, hands.1 + 1));
    // P1's next end step: the last mode; P1 chooses one of P1's opponents (P0 or P2).
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![3]));
    t.answer_choose(P1, &[Entity::Player(P2)]);
    t.advance_to(P1, Step::PostcombatMain);
    let from = t.asked().len();
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(modes_since(&t, from), vec![(P1, vec![2, 3])]);
    let players: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P1 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(players, vec![vec![Entity::Player(P0), Entity::Player(P2)]]);
    let sk = t.named_on_battlefield("Sol'Kanar the Tainted");
    assert_eq!(t.g.obj(sk[0]).controller, P2);
}
