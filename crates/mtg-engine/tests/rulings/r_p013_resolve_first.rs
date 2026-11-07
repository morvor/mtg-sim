//! Rulings batch P013 — abilities that trigger when a spell is cast go on the stack on top
//! of it and resolve first (CR 601.2i, 603.3), independently of the spell: they resolve
//! even if the spell is countered or otherwise leaves the stack in response, and a
//! permanent spell isn't on the battlefield yet when its own cast trigger resolves.

use crate::r_s01_common::supported;
use crate::r_s04_common::{stack_items, top_of_stack};
use crate::r_s08_common::is_tapped;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Whether the object on top of the stack is a triggered ability.
fn trigger_on_top(t: &TestGame) -> bool {
    matches!(
        t.g.obj(top_of_stack(t)).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    )
}

/// How the spell leaves the stack in response to its cast trigger.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Leaves {
    Stays,
    Countered,
    Bounced,
}

/// `p` casts the card `name` from hand (lands for its mana cost added, generic paid with
/// `generic_land`); the cast trigger is then above it; the spell leaves the stack as
/// `leaves` says, and everything resolves. Returns the spell.
fn cast_and(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    generic_land: &str,
    x: Option<i64>,
    leaves: Leaves,
) -> ObjectId {
    let c = mtg_engine::card::card(name);
    let cost = c.front().chars.mana_cost.clone().unwrap_or_default();
    for sym in format!("{cost}")
        .split('}')
        .filter_map(|s| s.strip_prefix('{'))
    {
        let land = match sym {
            "W" => "Plains",
            "U" => "Island",
            "B" => "Swamp",
            "R" => "Mountain",
            "G" => "Forest",
            "X" => continue,
            n => {
                t.lands(p, generic_land, n.parse().unwrap());
                continue;
            }
        };
        t.lands(p, land, 1);
    }
    if let Some(x) = x {
        t.lands(p, generic_land, x as usize);
    }
    let card = t.hand(p, name);
    let mut b = t.cast(p, card);
    if let Some(x) = x {
        b = b.x(x);
    }
    let spell = b.go();
    t.settle();
    assert!(trigger_on_top(t), "{name}: its cast trigger is on top");
    let items = stack_items(t);
    assert_eq!(items[items.len() - 2], name);
    match leaves {
        Leaves::Stays => {}
        Leaves::Countered => assert!(t.g.counter(spell, None)),
        Leaves::Bounced => {
            crate::r_p127_common::move_to(t, spell, Zone::Hand(p));
        }
    }
    spell
}

#[test]
fn cast_triggers_of_creature_spells_resolve_even_if_the_spell_leaves_the_stack() {
    cr!("603.3", "601.2i", "701.6a", "608.2");
    ruling!(
        "Hatchery Spider",
        "Hatchery Spider’s ability triggers as you cast it, and that ability resolves before the spell itself. It resolves even if that spell is countered."
    );
    ruling!(
        "Herigast, Erupting Nullkite",
        "Herigast's triggered ability will resolve before Herigast does. If Herigast is countered or otherwise leaves the stack in response to its triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Hope-Ender Coatl",
        "Hope-Ender Coatl's triggered ability will resolve before Hope-Ender Coatl does. If Hope-Ender Coatl is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Hydroid Krasis",
        "Hydroid Krasis's first ability triggers as you cast it, and that ability resolves before the spell itself. It resolves even if Hydroid Krasis is countered."
    );
    ruling!(
        "Kozilek, the Broken Reality",
        "Kozilek, the Broken Reality's triggered ability will resolve before Kozilek does. If Kozilek is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Petrifying Meddler",
        "Petrifying Meddler's triggered ability will resolve before Petrifying Meddler does. If Petrifying Meddler is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Twisted Riddlekeeper",
        "The triggered ability will resolve before Twisted Riddlekeeper does. If Twisted Riddlekeeper is countered or otherwise leaves the stack in response to its triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Ulamog, the Ceaseless Hunger",
        "Ulamog's first ability resolves independently of Ulamog once they've both been put on the stack. If Ulamog is countered, that triggered ability will still resolve. That triggered ability will always resolve before Ulamog does."
    );
    ruling!(
        "Wumpus Aberration",
        "Wumpus Aberration's triggered ability will resolve before Wumpus Aberration does. If Wumpus Aberration is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Titans' Vanguard",
        "When you cast Titans' Vanguard, its triggered ability will resolve before Titans' Vanguard does. If Titans' Vanguard is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    /// A case: the card, the land paying its generic cost, X, a setup (answers queued,
    /// returns objects to check) and the check of the trigger's effect.
    struct Case {
        name: &'static str,
        generic: &'static str,
        x: Option<i64>,
        setup: fn(&mut TestGame) -> Vec<ObjectId>,
        check: fn(&TestGame, &[ObjectId]),
    }
    let cases = [
        Case {
            // Undergrowth: X = 1 creature card in the graveyard; a green permanent card
            // with mana value 1 or less on top of the library.
            name: "Hatchery Spider",
            generic: "Forest",
            x: None,
            setup: |t| {
                t.graveyard(P0, "Grizzly Bears");
                let elves = t.library_top(P0, "Llanowar Elves");
                t.answer_choose(P0, &[obj(elves)]);
                vec![elves]
            },
            check: |t, o| assert!(t.on_battlefield(o[0])),
        },
        Case {
            name: "Herigast, Erupting Nullkite",
            generic: "Wastes",
            x: None,
            setup: |t| {
                t.hand(P0, "Grizzly Bears");
                t.answer_yes(P0, true);
                vec![]
            },
            check: |t, _| {
                assert!(t.in_exile("Grizzly Bears"));
                assert_eq!(t.hand_size(P0), 3);
                assert_eq!(t.library_size(P0), 27);
            },
        },
        Case {
            // Cast in response to an opponent's spell; the opponent can't pay {1}.
            name: "Hope-Ender Coatl",
            generic: "Island",
            x: None,
            setup: |t| {
                t.lands(P1, "Mountain", 1);
                let bolt = t.hand(P1, "Lightning Bolt");
                let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
                t.answer_targets(P0, &[obj(bolt)]);
                vec![bolt]
            },
            check: |t, _| {
                assert!(t.in_graveyard(P1, "Lightning Bolt"));
                assert_eq!(t.life(P0), 20);
            },
        },
        Case {
            name: "Hydroid Krasis",
            generic: "Forest",
            x: Some(4),
            setup: |_| vec![],
            check: |t, _| {
                assert_eq!(t.life(P0), 22);
                assert_eq!(t.library_size(P0), 28);
            },
        },
        Case {
            name: "Kozilek, the Broken Reality",
            generic: "Wastes",
            x: None,
            setup: |t| {
                t.hand(P1, "Grizzly Bears");
                t.hand(P1, "Hill Giant");
                t.answer_targets(P0, &[Entity::Player(P1)]);
                vec![]
            },
            check: |t, _| {
                let facedown = t
                    .g
                    .permanents()
                    .filter(|o| o.controller == P1 && o.face_down)
                    .count();
                assert_eq!(facedown, 2);
                assert_eq!(t.hand_size(P1), 0);
                // P0 drew a card for each.
                assert_eq!(t.library_size(P0), 28);
            },
        },
        Case {
            name: "Petrifying Meddler",
            generic: "Island",
            x: None,
            setup: |t| {
                let giant = t.battlefield(P1, "Hill Giant");
                t.answer_targets(P0, &[obj(giant)]);
                vec![giant]
            },
            check: |t, o| {
                assert!(is_tapped(t, o[0]));
                assert_eq!(t.counters(o[0], "stun"), 1);
            },
        },
        Case {
            name: "Twisted Riddlekeeper",
            generic: "Wastes",
            x: None,
            setup: |t| {
                let a = t.battlefield(P1, "Hill Giant");
                let b = t.battlefield(P1, "Forest");
                t.answer_targets(P0, &[obj(a), obj(b)]);
                vec![a, b]
            },
            check: |t, o| {
                for &id in o {
                    assert!(is_tapped(t, id));
                    assert_eq!(t.counters(id, "stun"), 1);
                }
            },
        },
        Case {
            name: "Ulamog, the Ceaseless Hunger",
            generic: "Wastes",
            x: None,
            setup: |t| {
                let a = t.battlefield(P1, "Hill Giant");
                let b = t.battlefield(P1, "Forest");
                t.answer_targets(P0, &[obj(a), obj(b)]);
                vec![a, b]
            },
            check: |t, o| {
                for &id in o {
                    assert_eq!(t.zone(id), Zone::Exile);
                    // Exiled face up.
                    assert!(!t.obj_now(id).face_down);
                }
            },
        },
        Case {
            // Paid with Forests only, so {C} wasn't spent.
            name: "Wumpus Aberration",
            generic: "Forest",
            x: None,
            setup: |t| {
                let bears = t.hand(P1, "Grizzly Bears");
                t.answer_targets(P0, &[Entity::Player(P1)]);
                t.answer_choose(P1, &[obj(bears)]);
                vec![bears]
            },
            check: |t, o| assert!(t.on_battlefield(o[0])),
        },
        Case {
            name: "Titans' Vanguard",
            generic: "Wastes",
            x: None,
            setup: |t| vec![t.battlefield(P0, "Ornithopter")],
            check: |t, o| assert_eq!(t.counters(o[0], "+1/+1"), 1),
        },
    ];
    for c in cases {
        supported(c.name);
        for leaves in [Leaves::Stays, Leaves::Countered, Leaves::Bounced] {
            let mut t = TestGame::new(2);
            let objs = (c.setup)(&mut t);
            let spell = cast_and(&mut t, P0, c.name, c.generic, c.x, leaves);
            // The trigger resolves first, while the spell (if still there) waits.
            t.resolve();
            if leaves == Leaves::Stays {
                assert_eq!(t.zone(spell), Zone::Stack, "{}", c.name);
            }
            (c.check)(&t, &objs);
            t.resolve_all();
            let expect = match leaves {
                Leaves::Stays => Zone::Battlefield,
                Leaves::Countered => Zone::Graveyard(P0),
                // Herigast returned to hand is exiled with the rest of the hand.
                Leaves::Bounced if c.name.starts_with("Herigast") => Zone::Exile,
                Leaves::Bounced => Zone::Hand(P0),
            };
            assert_eq!(t.zone(spell), expect, "{} {leaves:?}", c.name);
        }
    }
}

#[test]
fn a_creature_spell_is_not_on_the_battlefield_when_its_cast_trigger_resolves() {
    cr!("603.3", "601.2i", "608.3");
    ruling!(
        "Decimator of the Provinces",
        "The triggered ability resolves before Decimator of the Provinces enters the battlefield, so it won’t get +2/+2 from its own triggered ability."
    );
    ruling!(
        "Titans' Vanguard",
        "Since Titans' Vanguard's triggered ability resolves before Titans' Vanguard does, it won't be on the battlefield when that ability resolves, and it won't receive a +1/+1 counter."
    );
    supported("Decimator of the Provinces");
    supported("Titans' Vanguard");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dec = cast_and(
        &mut t,
        P0,
        "Decimator of the Provinces",
        "Wastes",
        None,
        Leaves::Stays,
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    assert!(t.on_battlefield(dec));
    assert_eq!(t.pt(dec), (7, 7));

    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let tv = cast_and(&mut t, P0, "Titans' Vanguard", "Wastes", None, Leaves::Stays);
    t.resolve_all();
    assert_eq!(t.counters(thopter, "+1/+1"), 1);
    assert!(t.on_battlefield(tv));
    assert_eq!(t.counters(tv, "+1/+1"), 0);
    assert_eq!(t.pt(tv), (5, 5));
}

/// P1 casts `name` (lands added) with the given targets; returns the spell, with the
/// triggers on the stack above it.
fn opponent_casts(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    lands_for_cost(t, P1, name);
    let card = t.hand(P1, name);
    let spell = t.cast_with(P1, card, targets).unwrap();
    t.settle();
    spell
}

#[test]
fn triggers_on_an_opponents_spell_resolve_first_even_if_it_is_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Magebane Lizard",
        "Magebane Lizard’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Mindsparker",
        "Mindsparker's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Aether Sting",
        "The ability triggers when an opponent casts a creature spell, and goes on the stack above that spell. As such, the ability will deal 1 damage to that player before the creature enters."
    );
    ruling!(
        "Hidden Gibbons",
        "The ability triggers when the spell is cast, so it becomes a creature before the spell resolves."
    );
    ruling!(
        "Painful Quandary",
        "Whenever an opponent casts a spell, Painful Quandary's ability triggers and goes on the stack on top of it. It will resolve before the spell does."
    );
    // (permanent, P1's spell, its targets, check after the trigger alone resolved)
    type Check = fn(&TestGame, ObjectId);
    let cases: [(&str, &str, bool, Check); 5] = [
        ("Magebane Lizard", "Divination", false, |t, _| {
            assert_eq!(t.life(P1), 19)
        }),
        ("Mindsparker", "Divination", false, |t, _| {
            assert_eq!(t.life(P1), 18)
        }),
        ("Aether Sting", "Grizzly Bears", false, |t, _| {
            assert_eq!(t.life(P1), 19);
            assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
        }),
        ("Hidden Gibbons", "Lightning Bolt", true, |t, g| {
            assert!(t.obj_now(g).is(CardType::Creature));
            assert_eq!(t.pt(g), (4, 4));
        }),
        // P1 has no cards in hand to discard: loses 5 life.
        ("Painful Quandary", "Divination", false, |t, _| {
            assert_eq!(t.life(P1), 15)
        }),
    ];
    for (perm, spell_name, targets_p0, check) in cases {
        supported(perm);
        for leaves in [Leaves::Stays, Leaves::Countered, Leaves::Bounced] {
            let mut t = TestGame::new(2);
            let p = t.battlefield(P0, perm);
            let targets = if targets_p0 {
                vec![Entity::Player(P0)]
            } else {
                vec![]
            };
            let spell = opponent_casts(&mut t, spell_name, &targets);
            assert!(trigger_on_top(&t), "{perm}");
            match leaves {
                Leaves::Stays => {
                    t.resolve();
                    assert_eq!(t.zone(spell), Zone::Stack);
                }
                Leaves::Countered => {
                    assert!(t.g.counter(spell, None));
                    t.resolve();
                }
                Leaves::Bounced => {
                    crate::r_p127_common::move_to(&mut t, spell, Zone::Hand(P1));
                    t.resolve();
                }
            }
            check(&t, p);
        }
    }
}

#[test]
fn illusory_demon_is_sacrificed_before_the_spell_resolves() {
    cr!("603.3", "601.2i");
    ruling!(
        "Illusory Demon",
        "If you cast a spell, the spell goes on the stack, then Illusory Demon’s triggered ability goes on the stack on top of it. The triggered ability will resolve (and you’ll sacrifice Illusory Demon) before the spell resolves."
    );
    supported("Illusory Demon");
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Illusory Demon");
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    let bears = t.cast(P0, bears).go();
    t.settle();
    assert!(trigger_on_top(&t));
    t.resolve();
    assert!(t.in_graveyard(P0, "Illusory Demon"));
    assert!(!t.g.is_live(demon));
    assert_eq!(t.zone(bears), Zone::Stack);
}

#[test]
fn tovolars_magehunter_can_make_the_caster_lose_before_the_spell_resolves() {
    cr!("603.3", "704.5a", "800.4a");
    ruling!(
        "Mondronen Shaman",
        "Tovolar's Magehunter's first triggered ability will resolve before the spell that caused it to trigger. If this causes the player who cast that spell to lose the game, that spell won't resolve."
    );
    supported("Mondronen Shaman");
    // Three players, so the game goes on after P1 loses (CR 800.4a: their spell leaves).
    let mut t = TestGame::new(3);
    let shaman = t.battlefield(P0, "Mondronen Shaman");
    assert!(mtg_engine::dfc::transform(&mut t.g, shaman));
    t.g.flush_events();
    assert_eq!(t.obj_now(shaman).chars.name, "Tovolar's Magehunter");
    t.g.players[1].life = 2;
    let bears = t.battlefield(P2, "Grizzly Bears");
    let bolt = opponent_casts(&mut t, "Lightning Bolt", &[obj(bears)]);
    assert!(trigger_on_top(&t));
    t.resolve_all();
    assert!(t.has_lost(P1));
    assert!(!t.g.is_live(bolt) || t.zone(bolt) != Zone::Stack);
    // The Bolt never resolved.
    assert!(t.on_battlefield(bears));
}
