//! Rulings batch P127 — casting a card from its owner's graveyard: the normal timing and
//! cost rules still apply, the spell moves to the stack as casting begins, it can be
//! countered (and cast again from the graveyard), "died this turn" and "gained life this
//! turn" conditions, last known names, counters kept through zone changes, and permissions
//! from other effects.

use crate::r_p127_common::*;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `p` may cast the card (in any way) now.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    !legal_cast_methods(t, p, card).is_empty()
}

/// Casts the card from wherever it is with the first legal method; returns the spell.
fn cast_any(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    let m = legal_cast_methods(t, p, card);
    assert!(!m.is_empty(), "no legal way to cast it");
    let card = t.g.current(card);
    t.cast(p, card).method(m[0].clone()).go()
}

/// The alternative-cost method of the card's graveyard permission.
fn alt_method(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Option<CastMethod> {
    legal_cast_methods(t, p, card)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
}

#[test]
fn prized_amalgam_ignores_a_madness_card_cast_from_exile() {
    cr!("702.35a", "603.4");
    ruling!(
        "Prized Amalgam",
        "A card cast using its madness ability is cast from exile, not from your graveyard."
    );
    supported("Prized Amalgam");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Prized Amalgam");
    let rw = t.hand(P0, "Basking Rootwalla");
    t.g.discard(P0, rw, None);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(named_by(&t, P0, "Basking Rootwalla").len(), 1);
    end_step(&mut t, P0);
    assert!(t.in_graveyard(P0, "Prized Amalgam"));
    // A creature cast from the graveyard does trigger it.
    let mut t = TestGame::new(2);
    let pa = t.graveyard(P0, "Prized Amalgam");
    t.battlefield(P0, "Walking Corpse");
    let gc = t.graveyard(P0, "Gravecrawler");
    t.lands(P0, "Swamp", 1);
    cast_any(&mut t, P0, gc);
    t.resolve_all();
    end_step(&mut t, P0);
    assert!(t.on_battlefield(pa));
}

#[test]
fn scourge_of_nel_toth_is_on_the_stack_as_its_costs_are_paid() {
    cr!("601.2a", "601.2h");
    ruling!(
        "Scourge of Nel Toth",
        "As soon as you start to cast Scourge of Nel Toth from your graveyard, it moves to the stack."
    );
    supported("Scourge of Nel Toth");
    let mut t = TestGame::new(2);
    let sc = t.graveyard(P0, "Scourge of Nel Toth");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let m = alt_method(&mut t, P0, sc).expect("alternative cost offered");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    // When P0 chooses the creatures to sacrifice, the Scourge is already on the stack.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }),
        |g| g.stack.len(),
    );
    let spell = t.cast(P0, sc).method(m).go();
    assert!(seen.lock().unwrap().iter().all(|n| *n == 1));
    assert!(!seen.lock().unwrap().is_empty());
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Hill Giant"));
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn hundred_battle_veteran_moves_to_the_stack_without_anyone_getting_priority() {
    cr!("601.2a", "601.2i");
    ruling!(
        "Hundred-Battle Veteran",
        "Once you begin casting a spell from your graveyard, it immediately moves to the stack. Players can’t take any actions until you’ve finished casting the spell."
    );
    supported("Hundred-Battle Veteran");
    let mut t = TestGame::new(2);
    let hb = t.graveyard(P0, "Hundred-Battle Veteran");
    t.lands(P0, "Swamp", 4);
    let from = t.asked().len();
    let spell = cast_any(&mut t, P0, hb);
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    t.resolve_all();
    assert_eq!(finality(&t, spell), 1);
}

#[test]
fn casting_from_the_graveyard_keeps_normal_timing_and_costs() {
    cr!("601.3", "117.1a", "307.1");
    ruling!(
        "Ebondeath, Dracolich",
        "Casting Ebondeath, Dracolich from your graveyard follows the normal rules for casting that card. You must pay its costs, and you must follow all applicable timing rules."
    );
    ruling!(
        "Oathsworn Vampire",
        "Casting Oathsworn Vampire from your graveyard follows the normal rules for casting that card. You must pay its costs, and you must follow all applicable timing rules."
    );
    ruling!(
        "Scourge of Nel Toth",
        "Casting Scourge of Nel Toth from your graveyard by paying the alternative cost doesn’t change when you can cast it. You can cast it only at the normal time you could cast a creature spell."
    );
    ruling!(
        "Demilich",
        "Demilich's last ability doesn't change when you can cast it. In most cases, you can cast Demilich only during your main phase."
    );
    ruling!(
        "Gravecrawler",
        "Gravecrawler's ability doesn't change when you could cast it."
    );
    // (card, setup for the permission, lands, has flash)
    type Setup = fn(&mut TestGame);
    let cases: [(&str, Setup, &str, usize, bool); 5] = [
        (
            "Ebondeath, Dracolich",
            |t| a_creature_dies(t, P0),
            "Swamp",
            4,
            true,
        ),
        (
            "Oathsworn Vampire",
            |t| {
                cast_new(t, P0, "Healing Salve", &[Entity::Player(P0)]);
                t.resolve_all();
            },
            "Swamp",
            2,
            false,
        ),
        (
            "Scourge of Nel Toth",
            |t| {
                t.battlefield(P0, "Grizzly Bears");
                t.battlefield(P0, "Hill Giant");
            },
            "Swamp",
            2,
            false,
        ),
        (
            "Demilich",
            |t| {
                for n in ["Opt", "Shock", "Divination", "Lightning Bolt"] {
                    t.graveyard(P0, n);
                }
            },
            "Island",
            4,
            false,
        ),
        (
            "Gravecrawler",
            |t| {
                t.battlefield(P0, "Walking Corpse");
            },
            "Swamp",
            1,
            false,
        ),
    ];
    for (card, setup, land, n, flash) in cases {
        supported(card);
        // No mana: it can't be cast.
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, card);
        setup(&mut t);
        assert!(!castable(&mut t, P0, c), "{card} without mana");
        // Mana, but during the opponent's turn: only with flash.
        t.lands(P0, land, n);
        t.set_step(P1, Step::PrecombatMain);
        t.g.turn.priority = Some(P0);
        assert_eq!(castable(&mut t, P0, c), flash, "{card} on P1's turn");
        // P0's main phase with an empty stack: castable.
        t.set_step(P0, Step::PrecombatMain);
        assert!(castable(&mut t, P0, c), "{card} in P0's main phase");
    }
}

#[test]
fn demilich_copy_must_be_cast_now_and_the_card_stays_exiled() {
    cr!("707.12", "608.2g");
    ruling!(
        "Demilich",
        "If you choose to cast it, you must do so right away. If you choose not to cast it right away, the copy ceases to exist the next time a player would receive priority."
    );
    ruling!(
        "Demilich",
        "If you exile a card as the triggered ability resolves, that card will remain exiled whether you cast the copy or not."
    );
    supported("Demilich");
    for cast in [false, true] {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, "Demilich");
        let bolt = t.graveyard(P0, "Lightning Bolt");
        // Casting the copy means paying its mana cost.
        t.lands(P0, "Mountain", 1);
        t.answer_targets(P0, &[obj(bolt)]);
        attack_with(&mut t, &[(d, Entity::Player(P1))]);
        t.answer_yes(P0, cast);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve_all();
        assert!(is_in(&t, bolt, Zone::Exile), "cast: {cast}");
        assert_eq!(t.life(P1), if cast { 17 } else { 20 });
        assert_eq!(t.stack_len(), 0);
        // No copy lingers anywhere.
        assert!(!t
            .g
            .objects
            .iter()
            .any(|o| t.g.is_live(o.id) && o.zone != Zone::Exile && o.chars.name == "Lightning Bolt" && o.zone != Zone::Graveyard(P0)));
    }
}

#[test]
fn ebondeath_cares_about_any_non_ebondeath_death_this_turn() {
    cr!("700.4", "111.7");
    ruling!(
        "Ebondeath, Dracolich",
        "For example, if the creature that died is a token or the creature card leaves the graveyard later in the turn, you can still cast Ebondeath, Dracolich."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let other = t.graveyard(P0, "Ebondeath, Dracolich");
    assert!(!castable(&mut t, P0, other), "no creature died yet");
    // A token dies while Ebondeath is still in P0's hand.
    let tok = create_token(&mut t, P1, "Soldier");
    destroy(&mut t, tok);
    let eb = t.hand(P0, "Ebondeath, Dracolich");
    let eb = move_to(&mut t, eb, Zone::Graveyard(P0)).unwrap();
    assert!(castable(&mut t, P0, eb));
}

#[test]
fn ebondeath_uses_the_dead_creatures_last_name() {
    cr!("608.2h", "707.2");
    ruling!(
        "Ebondeath, Dracolich",
        "Ebondeath Dracolich cares what the creature's name was while it was last on the battlefield, not what it is in the graveyard."
    );
    let mut t = TestGame::new(2);
    let eb = t.graveyard(P0, "Ebondeath, Dracolich");
    t.lands(P0, "Swamp", 4);
    // P1's Clone enters as a copy of P1's Ebondeath; the legend rule puts the Clone into
    // the graveyard (it died named Ebondeath, Dracolich).
    let theirs = t.battlefield(P1, "Ebondeath, Dracolich");
    t.answer_choose(P1, &[obj(theirs)]);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(theirs)]);
    t.enter(P1, "Clone");
    t.settle();
    assert!(t.in_graveyard(P1, "Clone"));
    assert!(t.on_battlefield(theirs));
    assert!(!castable(&mut t, P0, eb));
    // A creature with another name dies: now it can be cast.
    a_creature_dies(&mut t, P1);
    assert!(castable(&mut t, P0, eb));
}

#[test]
fn me_the_immortal_returns_a_new_object_free_of_counter_durations() {
    cr!("400.7", "611.2b");
    ruling!(
        "Me, the Immortal",
        "Effects that last \"for as long as that creature has a [kind of] counter on it,\" such as Aven Mimeomancer's, stop applying to Me, the immortal once it leaves the battlefield."
    );
    supported("Me, the Immortal");
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, "Me, the Immortal");
    t.battlefield(P0, "Aven Mimeomancer");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(me)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(me, "feather"), 1);
    assert_eq!(t.pt(me), (3, 1));
    destroy(&mut t, me);
    assert_eq!(t.counters(me, "feather"), 1);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    let spell = cast_any(&mut t, P0, me);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(t.counters(spell, "feather"), 1);
    assert_eq!(t.pt(spell).1, 3);
    assert!(!t
        .obj_now(spell)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn countered_after_casting_from_the_graveyard_it_can_be_cast_again() {
    cr!("701.6a", "601.2a");
    ruling!(
        "Hundred-Battle Veteran",
        "If Hundred-Battle Veteran is countered after being cast from your graveyard, it will return to your graveyard. It may be cast this way again."
    );
    ruling!(
        "Squee, Dubious Monarch",
        "If Squee is countered or dies after being cast from your graveyard, it returns to the graveyard. It may be cast this way again later."
    );
    ruling!(
        "The Indomitable",
        "If The Indomitable is countered or dies after being cast from your graveyard, it returns to the graveyard. It may be cast this way again later."
    );
    ruling!(
        "Undead Sprinter",
        "If Undead Sprinter is countered or dies after being cast from your graveyard, it will return to your graveyard. It may be cast this way again."
    );
    ruling!(
        "Scourge of Nel Toth",
        "If you cast Scourge of Nel Toth from your graveyard, it is a spell and can be countered."
    );
    type Setup = fn(&mut TestGame);
    let cases: [(&str, Setup); 5] = [
        ("Hundred-Battle Veteran", |t| {
            t.lands(P0, "Swamp", 8);
        }),
        ("Squee, Dubious Monarch", |t| {
            t.lands(P0, "Mountain", 8);
            for _ in 0..8 {
                t.graveyard(P0, "Forest");
            }
        }),
        ("The Indomitable", |t| {
            t.lands(P0, "Island", 8);
            for _ in 0..3 {
                let v = t.battlefield(P0, "Smuggler's Copter");
                t.g.tap(v);
            }
        }),
        ("Undead Sprinter", |t| {
            t.lands(P0, "Swamp", 4);
            t.lands(P0, "Mountain", 4);
            a_creature_dies(t, P1);
        }),
        ("Scourge of Nel Toth", |t| {
            t.lands(P0, "Swamp", 4);
            for _ in 0..4 {
                t.battlefield(P0, "Grizzly Bears");
            }
        }),
    ];
    for (card, setup) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, card);
        setup(&mut t);
        for round in 0..2 {
            let spell = cast_any(&mut t, P0, c);
            assert_eq!(t.zone(spell), Zone::Stack, "{card} round {round}");
            cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
            t.resolve_all();
            assert!(t.in_graveyard(P0, card), "{card} countered");
        }
    }
}

#[test]
fn brokkos_uses_another_permission_with_its_cost() {
    cr!("601.2b", "118.9");
    ruling!(
        "Brokkos, Apex of Forever",
        "If another effect gives you permission to cast Brokkos from your graveyard, you may cast it for any cost associated with that permission."
    );
    supported("Brokkos, Apex of Forever");
    supported("Osteomancer Adept");
    let mut t = TestGame::new(2);
    let br = t.graveyard(P0, "Brokkos, Apex of Forever");
    let adept = t.battlefield(P0, "Osteomancer Adept");
    // Without the Adept's permission, only the mutate cost is offered.
    let before = legal_cast_methods(&mut t, P0, br);
    lands_for_cost(&mut t, P0, "Brokkos, Apex of Forever");
    for _ in 0..3 {
        t.graveyard(P0, "Forest");
    }
    let without = legal_cast_methods(&mut t, P0, br);
    assert!(!without.contains(&CastMethod::Normal));
    activate_containing(&mut t, P0, adept, "Until end of turn").unwrap();
    t.resolve_all();
    let with = legal_cast_methods(&mut t, P0, br);
    assert!(with.len() > without.len(), "{before:?} {without:?} {with:?}");
    let m = with.iter().find(|m| !without.contains(m)).unwrap().clone();
    let spell = t.cast(P0, br).method(m).go();
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(finality(&t, spell), 1);
}

#[test]
fn feasting_troll_king_trigger_needs_a_cast_from_hand() {
    cr!("707.2", "603.4", "603.6a");
    ruling!(
        "Feasting Troll King",
        "If you cast a creature spell that enters the battlefield as a copy of Feasting Troll King, such as Clone, the enters-the-battlefield ability will trigger."
    );
    ruling!(
        "Feasting Troll King",
        "If you put Feasting Troll King onto the battlefield from your hand without casting it, its enters-the-battlefield ability won't trigger."
    );
    supported("Feasting Troll King");
    // Put onto the battlefield from hand without being cast: no Food.
    let mut t = TestGame::new(2);
    let k = t.hand(P0, "Feasting Troll King");
    move_to(&mut t, k, Zone::Battlefield);
    t.resolve_all();
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Food"), 0);
    // A Clone cast from hand copying it: three Foods.
    let mut t = TestGame::new(2);
    let king = t.battlefield(P0, "Feasting Troll King");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(king)]);
    cast_new(&mut t, P0, "Clone", &[]);
    t.resolve_all();
    let foods = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Food"))
        .count();
    assert_eq!(foods, 3);
}

#[test]
fn vanguard_of_the_restless_counts_casts_of_each_commander() {
    cr!("903.8", "702.124a");
    ruling!(
        "Vanguard of the Restless",
        "If you have more than one commander (probably because of a partner ability), Vanguard of the Restless's second ability counts the total number of times you've cast each of your commanders from the command zone."
    );
    supported("Vanguard of the Restless");
    let mut t = TestGame::with_config(
        2,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    );
    let v = t.battlefield(P0, "Vanguard of the Restless");
    for name in ["Kraum, Ludevic's Opus", "Tymna the Weaver"] {
        let id = t.command(P0, name);
        t.g.objects[id.0 as usize].is_commander = true;
        t.g.players[0].commander_names.push(name.into());
        lands_for_cost(&mut t, P0, name);
        t.cast(P0, id).go();
        t.resolve_all();
    }
    assert_eq!(t.pt(v), (4, 4));
}

#[test]
fn marang_river_prowler_checks_only_as_casting_begins() {
    cr!("601.2", "601.3");
    ruling!(
        "Marang River Prowler",
        "Once you begin to cast Marang River Prowler from your graveyard, it doesn't matter what happens to the black or green permanents you control."
    );
    supported("Marang River Prowler");
    let mut t = TestGame::new(2);
    let mrp = t.graveyard(P0, "Marang River Prowler");
    t.lands(P0, "Island", 2);
    assert!(!castable(&mut t, P0, mrp));
    // Blood Pet (black) is the only remaining mana source: sacrificing it for mana while
    // casting doesn't undo the cast.
    let pet = t.battlefield(P0, "Blood Pet");
    assert!(castable(&mut t, P0, mrp));
    let spell = cast_any(&mut t, P0, mrp);
    assert!(!t.on_battlefield(pet));
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn oathsworn_vampire_needs_only_some_life_gained_this_turn() {
    cr!("119.9", "601.3");
    ruling!(
        "Oathsworn Vampire",
        "It doesn't care how much you gained, whether you also lost life, or even whether you lost more life than you gained."
    );
    supported("Oathsworn Vampire");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let other = t.graveyard(P0, "Oathsworn Vampire");
    assert!(!castable(&mut t, P0, other), "no life gained yet");
    // Life gained while it was still in hand, then more life lost.
    cast_new(&mut t, P0, "Healing Salve", &[Entity::Player(P0)]);
    t.resolve_all();
    t.g.lose_life(P0, 6);
    let ov = t.hand(P0, "Oathsworn Vampire");
    let ov = move_to(&mut t, ov, Zone::Graveyard(P0)).unwrap();
    assert!(castable(&mut t, P0, ov));
}

#[test]
fn ashcloud_phoenix_returns_as_a_known_face_down_permanent() {
    cr!("708.2", "400.7");
    ruling!(
        "Ashcloud Phoenix",
        "Once Ashcloud Phoenix returns to the battlefield face down, each player will know which face-down creature it is."
    );
    supported("Ashcloud Phoenix");
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Ashcloud Phoenix");
    crate::r_s12_common::morph(&mut t, P0, "Ashcloud Phoenix");
    destroy(&mut t, ph);
    t.resolve_all();
    // The returned Phoenix is the object followed from the one that died, face down.
    let now = t.g.current(ph);
    assert!(t.on_battlefield(now));
    assert!(t.obj_now(now).face_down);
    assert_eq!(t.g.obj(now).owner, P0);
    assert_eq!(t.g.obj(now).card.as_ref().map(|c| c.name.as_str()), Some("Ashcloud Phoenix"));
}
