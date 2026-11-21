//! Rulings batch P128 — casting a card from its graveyard (or from exile) with its own
//! permission: the normal timing and cost rules still apply, the permission is checked
//! only as casting begins, costs are locked in once casting starts, cost reductions and
//! increases (CR 601.2f), and commander-cast triggers and counts.

use crate::r_p128_common::*;
use crate::r_s13_common::{commander, commander_game};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn casting_from_the_graveyard_or_exile_keeps_normal_timing_and_costs() {
    cr!("601.3", "117.1a", "601.2f");
    ruling!(
        "Risen Executioner",
        "Risen Executioner’s last ability doesn’t change when you can cast it."
    );
    ruling!(
        "Squee, the Immortal",
        "You must follow the normal timing permissions and restrictions and pay its cost to cast Squee from your graveyard or from exile."
    );
    ruling!(
        "Squee, Dubious Monarch",
        "The casting permission granted by Squee's last ability doesn't change when you may cast it from your graveyard."
    );
    ruling!(
        "The Indomitable",
        "The casting permission granted by The Indomitable's last ability doesn't change when you may cast it from your graveyard."
    );
    ruling!(
        "The Indomitable",
        "You still pay The Indomitable's costs when casting it from your graveyard."
    );
    ruling!(
        "Me, the Immortal",
        "You must follow all normal timing rules when casting Me from your graveyard using Me's last ability."
    );
    ruling!(
        "Quilled Greatwurm",
        "You must follow all normal timing rules when casting Quilled Greatwurm from your graveyard with the permission granted by its last ability."
    );
    ruling!(
        "Marang River Prowler",
        "You must follow the normal timing permissions and restrictions and pay its cost to cast Marang River Prowler from your graveyard."
    );
    ruling!(
        "Helbrute",
        "You still need to follow all normal timing restrictions when casting Helbrute from your graveyard using the permission granted by its last ability."
    );
    ruling!(
        "Hundred-Battle Veteran",
        "You still pay all costs and obey timing restrictions when casting Hundred-Battle Veteran from your graveyard with the permission from its last ability."
    );
    ruling!(
        "Misthollow Griffin",
        "Misthollow Griffin's ability doesn't change when you can cast creature spells: when you have priority during your main phase when the stack is empty."
    );
    ruling!(
        "Misthollow Griffin",
        "You must still pay Misthollow Griffin's mana cost when you cast it from exile."
    );
    type Setup = fn(&mut TestGame);
    // (card, zone it's cast from, setup for the permission and its extra costs, the lands
    // for its mana cost)
    let cases: [(&str, Zone, Setup, &[(&str, usize)]); 11] = [
        (
            "Risen Executioner",
            Zone::Graveyard(P0),
            |_| {},
            &[("Swamp", 4)],
        ),
        (
            "Squee, the Immortal",
            Zone::Graveyard(P0),
            |_| {},
            &[("Mountain", 3)],
        ),
        (
            "Squee, the Immortal",
            Zone::Exile,
            |_| {},
            &[("Mountain", 3)],
        ),
        (
            "Squee, Dubious Monarch",
            Zone::Graveyard(P0),
            |t| {
                for _ in 0..4 {
                    t.graveyard(P0, "Forest");
                }
            },
            &[("Mountain", 4)],
        ),
        (
            "The Indomitable",
            Zone::Graveyard(P0),
            |t| {
                for _ in 0..3 {
                    let v = t.battlefield(P0, "Smuggler's Copter");
                    t.g.tap(v);
                }
            },
            &[("Island", 4)],
        ),
        (
            "Me, the Immortal",
            Zone::Graveyard(P0),
            |t| {
                t.hand(P0, "Forest");
                t.hand(P0, "Forest");
            },
            &[("Forest", 1), ("Island", 1), ("Mountain", 1), ("Wastes", 2)],
        ),
        (
            "Quilled Greatwurm",
            Zone::Graveyard(P0),
            |t| {
                let b = t.battlefield(P0, "Grizzly Bears");
                put_counters(t, b, counters::PLUS1, 6);
            },
            &[("Forest", 6)],
        ),
        (
            "Marang River Prowler",
            Zone::Graveyard(P0),
            |t| {
                t.battlefield(P0, "Grizzly Bears");
            },
            &[("Island", 3)],
        ),
        (
            "Helbrute",
            Zone::Graveyard(P0),
            |t| {
                t.graveyard(P0, "Grizzly Bears");
            },
            &[("Swamp", 4), ("Mountain", 1)],
        ),
        (
            "Hundred-Battle Veteran",
            Zone::Graveyard(P0),
            |_| {},
            &[("Swamp", 4)],
        ),
        ("Misthollow Griffin", Zone::Exile, |_| {}, &[("Island", 4)]),
    ];
    for (card, zone, setup, lands) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        let c = match zone {
            Zone::Exile => t.exile(P0, card),
            _ => t.graveyard(P0, card),
        };
        setup(&mut t);
        // No mana: it can't be cast (its cost must be paid).
        assert!(!castable(&mut t, P0, c), "{card} without mana");
        for (land, n) in lands {
            t.lands(P0, land, *n);
        }
        // During the opponent's turn (none of these has flash): no.
        t.set_step(P1, Step::PrecombatMain);
        t.g.turn.priority = Some(P0);
        assert!(!castable(&mut t, P0, c), "{card} on P1's turn");
        // During P0's combat: no.
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!castable(&mut t, P0, c), "{card} in combat");
        // P0's main phase with an empty stack: yes, and it's paid for.
        t.set_step(P0, Step::PrecombatMain);
        assert!(castable(&mut t, P0, c), "{card} in P0's main phase");
        let spell = cast_any(&mut t, P0, c);
        let n: usize = lands.iter().map(|(_, n)| n).sum();
        assert_eq!(tapped_lands(&t, P0), n, "{card}: its mana cost was paid");
        t.resolve_all();
        assert!(t.on_battlefield(spell), "{card}");
    }
}

#[test]
fn risen_executioners_cost_is_locked_in_as_it_is_cast() {
    cr!("601.2a", "601.2f", "601.2i");
    ruling!(
        "Risen Executioner",
        "Once you start to cast Risen Executioner from your graveyard, you’ll finish casting it, including paying its cost, before any player receives priority."
    );
    supported("Risen Executioner");
    // "You may cast this creature from your graveyard if you pay {1} more to cast it for
    // each other creature card in your graveyard."
    let mut t = TestGame::new(2);
    let re = t.graveyard(P0, "Risen Executioner");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    assert!(!castable(&mut t, P0, re), "{{2}}{{B}}{{B}} plus {{2}}");
    t.lands(P0, "Swamp", 1);
    assert!(castable(&mut t, P0, re));
    let from = t.asked().len();
    let spell = cast_any(&mut t, P0, re);
    // Nobody received priority while it was being cast, and the full cost was paid.
    assert_eq!(priority_asks(&t, from), 0);
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(tapped_lands(&t, P0), 6);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn cards_castable_from_other_zones_can_still_be_cast_from_hand() {
    cr!("601.2a", "601.3");
    ruling!(
        "Squee, the Immortal",
        "Squee's ability doesn't prevent you from casting Squee from any other zone."
    );
    ruling!(
        "Misthollow Griffin",
        "Misthollow Griffin can also be cast from a player's hand."
    );
    ruling!(
        "Misthollow Griffin",
        "Misthollow Griffin doesn't have an ability that exiles itself. Some other effect will be needed to get the card into exile."
    );
    for (card, land, n) in [
        ("Squee, the Immortal", "Mountain", 3),
        ("Misthollow Griffin", "Island", 4),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let c = t.hand(P0, card);
        t.lands(P0, land, n);
        assert!(castable(&mut t, P0, c), "{card} from hand");
        let spell = cast_any(&mut t, P0, c);
        t.resolve_all();
        assert!(t.on_battlefield(spell), "{card}");
    }
    // The Griffin has no ability that exiles it: from the graveyard it can't be cast (or
    // exiled by itself); once some other effect exiles it, it can be cast.
    let mut t = TestGame::new(2);
    let g = t.graveyard(P0, "Misthollow Griffin");
    t.lands(P0, "Island", 4);
    assert!(!castable(&mut t, P0, g));
    assert!(!activatable(&mut t, P0, g));
    exile_now(&mut t, g);
    assert!(castable(&mut t, P0, g));
}

#[test]
fn skaab_ruinator_always_exiles_three_other_creature_cards() {
    cr!("601.2a", "601.2b", "601.2h");
    ruling!(
        "Skaab Ruinator",
        "Skaab Ruinator is on the stack when you pay its costs. It can't be exiled to pay for itself."
    );
    ruling!(
        "Skaab Ruinator",
        "You must exile three creature cards from your graveyard no matter what zone you're casting Skaab Ruinator from."
    );
    supported("Skaab Ruinator");
    for from_graveyard in [true, false] {
        let mut t = TestGame::new(2);
        let sr = if from_graveyard {
            t.graveyard(P0, "Skaab Ruinator")
        } else {
            t.hand(P0, "Skaab Ruinator")
        };
        t.lands(P0, "Island", 3);
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Hill Giant");
        // Two other creature cards (plus the Ruinator itself in the graveyard): not enough.
        assert!(!castable(&mut t, P0, sr), "graveyard: {from_graveyard}");
        t.graveyard(P0, "Craw Wurm");
        assert!(castable(&mut t, P0, sr), "graveyard: {from_graveyard}");
        let spell = cast_any(&mut t, P0, sr);
        for n in ["Grizzly Bears", "Hill Giant", "Craw Wurm"] {
            assert!(t.in_exile(n), "{n}");
        }
        t.resolve_all();
        assert!(t.on_battlefield(spell));
    }
}

#[test]
fn the_indomitable_counts_only_tapped_pirates_and_vehicles() {
    cr!("601.3", "301.7");
    ruling!(
        "The Indomitable",
        "Only tapped Pirates and tapped Vehicles count toward the casting permission from The Indomitable's last ability."
    );
    supported("The Indomitable");
    supported("Kitesail Corsair");
    // (permanents, tapped) -> castable
    let cases: [(&[&str], bool, bool); 4] = [
        (
            &["Smuggler's Copter", "Smuggler's Copter", "Kitesail Corsair"],
            false,
            false,
        ),
        (
            &["Grizzly Bears", "Grizzly Bears", "Grizzly Bears"],
            true,
            false,
        ),
        (
            &["Smuggler's Copter", "Smuggler's Copter", "Grizzly Bears"],
            true,
            false,
        ),
        (
            &["Smuggler's Copter", "Smuggler's Copter", "Kitesail Corsair"],
            true,
            true,
        ),
    ];
    for (perms, tapped, ok) in cases {
        let mut t = TestGame::new(2);
        let ind = t.graveyard(P0, "The Indomitable");
        t.lands(P0, "Island", 4);
        for n in perms {
            let id = t.battlefield(P0, n);
            if tapped {
                t.g.tap(id);
            }
        }
        assert_eq!(castable(&mut t, P0, ind), ok, "{perms:?} tapped: {tapped}");
    }
}

#[test]
fn the_permission_is_checked_only_as_casting_begins() {
    cr!("601.2", "601.3", "608.1");
    ruling!(
        "The Indomitable",
        "The Indomitable checks if you control three or more tapped Pirates and/or Vehicles only as you begin to cast it."
    );
    ruling!(
        "Gravecrawler",
        "Once you've cast Gravecrawler and it's on the stack, it doesn't matter if you lose control of the other Zombie."
    );
    // The Indomitable: the Vehicles are destroyed while it's on the stack.
    let mut t = TestGame::new(2);
    let ind = t.graveyard(P0, "The Indomitable");
    t.lands(P0, "Island", 4);
    let copters: Vec<_> = (0..3)
        .map(|_| {
            let v = t.battlefield(P0, "Smuggler's Copter");
            t.g.tap(v);
            v
        })
        .collect();
    let spell = cast_any(&mut t, P0, ind);
    for c in copters {
        destroy(&mut t, c);
    }
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    // Gravecrawler: P1 gains control of the Zombie while Gravecrawler is on the stack.
    supported("Gravecrawler");
    let mut t = TestGame::new(2);
    let gc = t.graveyard(P0, "Gravecrawler");
    let zombie = t.battlefield(P0, "Walking Corpse");
    t.lands(P0, "Swamp", 1);
    let spell = cast_any(&mut t, P0, gc);
    give_control(&mut t, zombie, P1);
    assert_eq!(t.obj_now(zombie).controller, P1);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn syrix_must_be_cast_as_its_trigger_resolves() {
    cr!("608.2g", "601.2");
    ruling!(
        "Syrix, Carrier of the Flame",
        "You must pay all costs for Syrix when you cast it using its last ability and you must cast it as the ability resolves."
    );
    supported("Syrix, Carrier of the Flame");
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        let sx = t.graveyard(P0, "Syrix, Carrier of the Flame");
        let phoenix = t.battlefield(P0, "Akoum Firebird");
        t.lands(P0, "Swamp", 2);
        t.lands(P0, "Mountain", 2);
        t.answer_yes(P0, cast);
        destroy(&mut t, phoenix);
        assert_eq!(t.stack_len(), 1);
        t.resolve();
        if cast {
            // Syrix is a spell on the stack now, its cost paid.
            assert_eq!(t.zone(sx), Zone::Stack);
            assert_eq!(tapped_lands(&t, P0), 4);
            t.resolve_all();
            assert!(t.on_battlefield(sx));
        } else {
            // It can't be cast later in the turn.
            t.resolve_all();
            assert!(is_in(&t, sx, Zone::Graveyard(P0)));
            assert!(!castable(&mut t, P0, sx));
        }
    }
}

#[test]
fn sproutback_trudge_cost_reduction() {
    cr!("601.2f", "118.7a", "202.3");
    ruling!(
        "Sproutback Trudge",
        "Sproutback Trudge’s ability counts the total amount of life gained without considering any life you lost during that turn. For example, if you lost 3 life and gained 4 life earlier in the turn, Sproutback Trudge will cost {3}{G}{G}."
    );
    ruling!(
        "Sproutback Trudge",
        "The cost reduction can reduce only generic mana in a spell’s total cost. It can’t reduce the cost to cast Sproutback Trudge beyond {G}{G}."
    );
    ruling!(
        "Sproutback Trudge",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases, then apply any cost reductions (such as that of Sproutback Trudge’s first ability). Sproutback Trudge’s mana value is always 9, no matter what the total cost to cast the spell was."
    );
    supported("Sproutback Trudge");
    supported("Sphere of Resistance");
    // (life lost, life gained, Sphere of Resistance, Forests needed)
    for (lost, gained, sphere, forests) in [
        (3, 4, false, 5),
        (0, 20, false, 2),
        (0, 8, true, 2),
        (0, 7, true, 3),
    ] {
        let mut t = TestGame::new(2);
        let tr = t.hand(P0, "Sproutback Trudge");
        if sphere {
            t.battlefield(P1, "Sphere of Resistance");
        }
        t.g.lose_life(P0, lost);
        t.g.gain_life(P0, gained);
        t.g.flush_events();
        t.lands(P0, "Forest", forests - 1);
        assert!(!castable(&mut t, P0, tr), "{lost}/{gained}/{sphere}");
        t.lands(P0, "Forest", 1);
        assert!(castable(&mut t, P0, tr), "{lost}/{gained}/{sphere}");
        let spell = cast_any(&mut t, P0, tr);
        assert_eq!(mana_value(&t, spell), 9);
        t.resolve_all();
        assert!(t.on_battlefield(spell));
    }
}

#[test]
fn demilichs_reduction_applies_to_an_additional_generic_cost() {
    cr!("601.2f", "118.7c");
    ruling!(
        "Demilich",
        "While casting Demilich, if you are required to pay an additional cost that includes a generic mana component, Demilich's cost reduction effect can apply to it."
    );
    supported("Demilich");
    for (spells, castable_free) in [(5, true), (4, false)] {
        let mut t = TestGame::new(2);
        let d = t.hand(P0, "Demilich");
        for _ in 0..spells {
            cast_new(&mut t, P0, "Opt", &[]);
            t.resolve_all();
        }
        // (No untapped lands are left.)
        assert_eq!(tapped_lands(&t, P0), spells);
        assert_eq!(t.g.battlefield.len(), spells);
        t.battlefield(P1, "Sphere of Resistance");
        assert_eq!(castable(&mut t, P0, d), castable_free, "{spells} spells");
        if castable_free {
            let spell = cast_any(&mut t, P0, d);
            t.resolve_all();
            assert!(t.on_battlefield(spell));
        } else {
            t.lands(P0, "Wastes", 1);
            assert!(castable(&mut t, P0, d), "{{1}} with four spells");
        }
    }
}

#[test]
fn demilichs_copy_has_to_be_paid_for() {
    cr!("707.12", "601.2f", "601.2h");
    ruling!(
        "Demilich",
        "You must pay all normal costs for casting the copy."
    );
    supported("Demilich");
    for with_mana in [false, true] {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, "Demilich");
        let bolt = t.graveyard(P0, "Lightning Bolt");
        if with_mana {
            t.lands(P0, "Mountain", 1);
        }
        t.answer_targets(P0, &[obj(bolt)]);
        attack_with(&mut t, &[(d, Entity::Player(P1))]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve_all();
        assert!(is_in(&t, bolt, Zone::Exile));
        assert_eq!(
            t.life(P1),
            if with_mana { 17 } else { 20 },
            "mana: {with_mana}"
        );
    }
}

#[test]
fn vanguard_counts_commander_casts_that_were_countered_or_are_on_the_stack() {
    cr!("903.8", "601.2");
    ruling!(
        "Vanguard of the Restless",
        "Vanguard of the Restless's second ability counts each time you've cast your commander from the command zone, even the times it was countered or when your commander spell is still on the stack."
    );
    supported("Vanguard of the Restless");
    let mut t = commander_game();
    let v = t.battlefield(P0, "Vanguard of the Restless");
    let cmd = commander(&mut t, P0, "Tymna the Weaver");
    assert_eq!(t.pt(v), (2, 2));
    lands_for_cost(&mut t, P0, "Tymna the Weaver");
    let spell = t.cast(P0, cmd).go();
    t.g.recompute();
    assert_eq!(t.pt(v), (3, 3), "the commander is still on the stack");
    t.lands(P1, "Island", 2);
    cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
    // The countered commander goes to the command zone (or the graveyard).
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(spell));
    assert_eq!(t.pt(v), (3, 3), "the countered cast still counts");
}

#[test]
fn skyfire_phoenix_returns_before_the_commander_resolves_even_if_countered() {
    cr!("903.8", "603.3", "608.1");
    ruling!(
        "Skyfire Phoenix",
        "Skyfire Phoenix's triggered ability resolves before the commander spell that caused the ability to trigger. The ability resolves even if that commander spell is countered."
    );
    supported("Skyfire Phoenix");
    let mut t = commander_game();
    let ph = t.graveyard(P0, "Skyfire Phoenix");
    let cmd = commander(&mut t, P0, "Tymna the Weaver");
    lands_for_cost(&mut t, P0, "Tymna the Weaver");
    let spell = t.cast(P0, cmd).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The Phoenix's trigger is above the commander spell.
    t.resolve();
    assert!(t.on_battlefield(ph));
    assert_eq!(t.zone(spell), Zone::Stack);
    // Countered: the Phoenix (another one) still returns.
    let mut t = commander_game();
    let ph = t.graveyard(P0, "Skyfire Phoenix");
    let cmd = commander(&mut t, P0, "Tymna the Weaver");
    lands_for_cost(&mut t, P0, "Tymna the Weaver");
    let spell = t.cast(P0, cmd).go();
    t.settle();
    t.lands(P1, "Island", 2);
    cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(spell));
    assert!(t.on_battlefield(ph));
}
