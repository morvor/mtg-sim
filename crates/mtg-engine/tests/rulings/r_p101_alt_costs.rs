//! Rulings batch P101 — alternative costs (CR 118.9): they change neither the mana cost
//! nor the mana value (118.9c, 202.3) nor when a spell may be cast (118.9b); cost
//! increases still apply (118.9d, 601.2f); the conditions of the Traps (searching,
//! casting, a single opponent) and of commander-free spells; Phyrexian mana and X in a
//! mana value (202.3e, 202.3g); exiling a spell from the stack isn't countering it.

use crate::r_p101_common::*;
use crate::r_p102_common::alternatives;
use crate::r_p154_common::commander_on_battlefield;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The one alternative-cost method `p` could cast `card` with now.
fn alt(t: &mut TestGame, p: PlayerId, card: ObjectId) -> CastMethod {
    let v = alternatives(t, p, card);
    assert_eq!(v.len(), 1, "{v:?}");
    v[0].clone()
}

/// P0's Demon of Death's Gate in hand with three Vampire Nighthawks (black) to sacrifice.
fn demon(t: &mut TestGame) -> (ObjectId, Vec<ObjectId>) {
    let d = t.hand(P0, "Demon of Death's Gate");
    let hawks = (0..3)
        .map(|_| t.battlefield(P0, "Vampire Nighthawk"))
        .collect();
    (d, hawks)
}

#[test]
fn demon_of_deaths_gates_alternative_cost_keeps_its_mana_value_and_timing() {
    cr!("118.9", "118.9c", "202.3", "302.1", "117.1a");
    ruling!(
        "Demon of Death's Gate",
        "Casting Demon of Death's Gate by paying its alternative cost doesn't change its mana cost or mana value."
    );
    ruling!(
        "Demon of Death's Gate",
        "Casting Demon of Death's Gate by paying its alternative cost doesn't change when you can cast it. You can cast it only at the normal time you could cast a creature spell."
    );
    ruling!(
        "Salvage Titan",
        "Casting Salvage Titan by paying its alternative cost doesn't change when you can cast it. You can cast it only at the normal time you could cast a creature spell."
    );
    supported("Demon of Death's Gate");
    supported("Salvage Titan");
    let mut t = TestGame::new(2);
    let (d, hawks) = demon(&mut t);
    let m = alt(&mut t, P0, d);
    t.answer_choose(P0, &hawks.iter().map(|h| obj(*h)).collect::<Vec<_>>());
    let spell = t.cast(P0, d).method(m).go();
    assert_eq!(t.life(P0), 14);
    assert_eq!(mv(&t, spell), 9);
    assert_eq!(
        format!("{}", t.obj_now(spell).chars.mana_cost.clone().unwrap()),
        "{6}{B}{B}{B}"
    );
    t.resolve_all();
    assert_eq!(mv(&t, spell), 9);
    assert!(hawks.iter().all(|h| !t.on_battlefield(*h)));
    // Not in an opponent's turn, in combat, or with a spell on the stack.
    for name in ["Demon of Death's Gate", "Salvage Titan"] {
        let setup = |t: &mut TestGame| -> ObjectId {
            if name == "Salvage Titan" {
                for _ in 0..3 {
                    t.battlefield(P0, "Ornithopter");
                }
                t.hand(P0, name)
            } else {
                demon(t).0
            }
        };
        let mut t = TestGame::new(2);
        let c = setup(&mut t);
        assert_eq!(alternatives(&mut t, P0, c).len(), 1, "{name}: main phase");
        t.set_step(P1, Step::PrecombatMain);
        assert!(
            alternatives(&mut t, P0, c).is_empty(),
            "{name}: opponent's turn"
        );
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(alternatives(&mut t, P0, c).is_empty(), "{name}: combat");
        t.set_step(P0, Step::PostcombatMain);
        cast_new(&mut t, P1, "Shock", &[pl(P1)]);
        assert!(
            alternatives(&mut t, P0, c).is_empty(),
            "{name}: stack not empty"
        );
    }
}

#[test]
fn cost_increases_apply_to_alternative_costs() {
    cr!("118.9d", "601.2f");
    ruling!(
        "Demon of Death's Gate",
        "Effects that increase or reduce the cost to cast Demon of Death's Gate will apply to it even if you choose to pay its alternative cost."
    );
    ruling!(
        "Pyrokinesis",
        "If another effect causes Pyrokinesis to cost more, you must pay that additional cost even if you pay its alternative cost."
    );
    ruling!(
        "Ravenous Trap",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as Ravenous Trap's alternative cost), add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Pyrokinesis");
    supported("Ravenous Trap");
    // Sphere of Resistance ("Spells cost {1} more to cast."): each alternative cost now
    // needs {1} too.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    let (d, hawks) = demon(&mut t);
    assert!(
        alternatives(&mut t, P0, d).is_empty(),
        "no mana for the {{1}}"
    );
    mana(&mut t, P0, "1");
    let m = alt(&mut t, P0, d);
    t.answer_choose(P0, &hawks.iter().map(|h| obj(*h)).collect::<Vec<_>>());
    t.cast(P0, d).method(m).go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert_eq!(count_named(&t, "Demon of Death's Gate"), 1);
    // Pyrokinesis, exiling a red card (Shock): {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pyro = t.hand(P0, "Pyrokinesis");
    let shock = t.hand(P0, "Shock");
    assert!(
        alternatives(&mut t, P0, pyro).is_empty(),
        "no mana for the {{1}}"
    );
    mana(&mut t, P0, "1");
    let m = alt(&mut t, P0, pyro);
    t.answer_choose(P0, &[obj(shock)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, pyro).method(m).go();
    t.resolve_all();
    assert!(t.in_exile("Shock"));
    assert!(!t.on_battlefield(bears));
    // Ravenous Trap with P1 having had three cards put into their graveyard: {0} + {1};
    // its mana value on the stack stays 4.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    t.g.mill(P1, 3);
    t.settle();
    let trap = t.hand(P0, "Ravenous Trap");
    assert!(
        alternatives(&mut t, P0, trap).is_empty(),
        "no mana for the {{1}}"
    );
    mana(&mut t, P0, "1");
    let m = alt(&mut t, P0, trap);
    let spell = t.cast(P0, trap).method(m).target(pl(P1)).go();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(mv(&t, spell), 4);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn mogg_salvage_free_keeps_mana_value_3_and_any_opponents_island_counts() {
    cr!("118.9c", "202.3", "118.9");
    ruling!(
        "Mogg Salvage",
        "Casting a spell without paying its mana cost has no effect on its mana value. Mogg Salvage's mana value is always 3."
    );
    ruling!(
        "Mogg Salvage",
        "To cast Mogg Salvage without paying its mana cost, any opponent may control an Island, not just the one who controls the target artifact."
    );
    // Three players: P2 controls the Island, P1 the artifact.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Mountain");
    t.battlefield(P2, "Island");
    let ring = t.battlefield(P1, "Sol Ring");
    let salvage = t.hand(P0, "Mogg Salvage");
    let m = alt(&mut t, P0, salvage);
    let spell = t.cast(P0, salvage).method(m).target(obj(ring)).go();
    assert_eq!(mv(&t, spell), 3);
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    // No opponent controls an Island: no free cast.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Mountain");
    t.battlefield(P0, "Island");
    let salvage = t.hand(P0, "Mogg Salvage");
    assert!(alternatives(&mut t, P0, salvage).is_empty());
}

#[test]
fn mental_misstep_counts_phyrexian_mana_as_1_and_x_as_chosen() {
    cr!("202.3g", "202.3e", "107.4f", "115.1");
    ruling!(
        "Mental Misstep",
        "Each Phyrexian mana symbol in a spell's mana cost contributes 1 to that spell's mana value. For example, Mental Misstep's mana value is 1, regardless of how its cost was paid."
    );
    ruling!(
        "Mental Misstep",
        "If a spell has {X} in its mana cost, X is considered to be the value chosen for it while that spell is on the stack."
    );
    // P1's Mental Misstep, paid with 2 life, is countered by P0's Mental Misstep.
    let mut t = TestGame::new(2);
    let theirs = t.hand(P1, "Mental Misstep");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, theirs).target(obj(bolt)).go();
    assert_eq!(t.life(P1), 18, "paid with life");
    assert_eq!(mv(&t, spell), 1);
    let ours = t.hand(P0, "Mental Misstep");
    t.g.turn.priority = Some(P0);
    t.cast(P0, ours).target(obj(spell)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Mental Misstep"));
    assert_eq!(t.life(P1), 15, "the Bolt resolved");
    // P1's Blaze ({X}{R}) with X = 0 has mana value 1 on the stack (a legal target); with
    // X = 1 it has 2 (not a legal target).
    for (x, legal) in [(0, true), (1, false)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let blaze = cast_x(&mut t, P1, "Blaze", x, &[pl(P0)]);
        assert_eq!(mv(&t, blaze) as i64, x + 1);
        let m = t.hand(P0, "Mental Misstep");
        t.g.turn.priority = Some(P0);
        assert_eq!(
            t.cast(P0, m).target(obj(blaze)).try_go().is_ok(),
            legal,
            "X = {x}"
        );
    }
}

/// `p` casts Opt (paid from a pool), which resolves.
fn opt(t: &mut TestGame, p: PlayerId) {
    mana(t, p, "U");
    let c = t.hand(p, "Opt");
    t.g.turn.priority = Some(p);
    t.cast(p, c).go();
    t.resolve_all();
}

#[test]
fn mindbreak_traps_condition_needs_one_opponent_casting_three_spells_resolved_or_not() {
    cr!("118.9", "601.2", "406.3", "701.6a");
    ruling!(
        "Mindbreak Trap",
        "For Mindbreak Trap's alternative cost to apply, a single opponent must cast three or more spells. Three opponents each casting a single spell won't work, for example."
    );
    ruling!(
        "Mindbreak Trap",
        "Mindbreak Trap's alternative cost condition checks whether an opponent cast three or more spells this turn, not whether those spells have resolved."
    );
    ruling!(
        "Mindbreak Trap",
        "If a spell is exiled, it's removed from the stack and thus will not resolve. The spell isn't countered; it just no longer exists. This works on spells that can't be countered."
    );
    supported("Mindbreak Trap");
    // Three opponents casting one spell each: no.
    let mut t = TestGame::new(4);
    for p in [P1, P2, P3] {
        opt(&mut t, p);
    }
    let trap = t.hand(P0, "Mindbreak Trap");
    assert!(alternatives(&mut t, P0, trap).is_empty());
    // P1 casts three spells, which are all still on the stack (the last one Abrupt Decay,
    // which can't be countered, at P0's Grizzly Bears): P0 exiles them all for {0}.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = cast_new(&mut t, P1, "Opt", &[]);
    let b = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    let decay = cast_new(&mut t, P1, "Abrupt Decay", &[obj(bears)]);
    let trap = t.hand(P0, "Mindbreak Trap");
    let m = alt(&mut t, P0, trap);
    t.cast(P0, trap)
        .method(m)
        .targets(&[obj(a), obj(b), obj(decay)])
        .go();
    t.resolve_all();
    for s in [a, b, decay] {
        assert_eq!(t.zone(s), Zone::Exile);
    }
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 20);
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::Countered { .. })));
}

#[test]
fn archive_traps_condition_is_any_search_of_any_opponent() {
    cr!("701.23a", "118.9", "701.17a", "701.17b");
    ruling!(
        "Archive Trap",
        "A spell or ability causes a player to search their library only if it specifically contains the word “search” in its text. For example, if a spell or ability lets a player look at the top four cards of their library and do something with one of those cards, that’s not a search."
    );
    ruling!(
        "Archive Trap",
        "Archive Trap checks only whether an opponent searched their library. It doesn’t matter whether that player found a card during the search."
    );
    ruling!(
        "Archive Trap",
        "If a search effect is affected by Aven Mindcensor’s ability (which causes a player to search the top four cards of their library instead), that still counts as searching that library."
    );
    ruling!(
        "Archive Trap",
        "The opponent you target doesn’t have to be the opponent who searched their library."
    );
    ruling!(
        "Archive Trap",
        "If the targeted opponent has fewer than thirteen cards in their library, that player puts their entire library into their graveyard."
    );
    supported("Archive Trap");
    // Impulse ("Look at the top four cards ...") isn't a search.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Impulse", &[]);
    t.resolve_all();
    let trap = t.hand(P0, "Archive Trap");
    assert!(
        alternatives(&mut t, P0, trap).is_empty(),
        "looking isn't searching"
    );
    // Rampant Growth finding nothing (no basic land in a library of fillers) is a search;
    // so is one with Aven Mindcensor (the top four only).
    for mindcensor in [false, true] {
        let mut t = TestGame::new(3);
        if mindcensor {
            t.battlefield(P0, "Aven Mindcensor");
        }
        t.set_step(P2, Step::PrecombatMain);
        cast_new(&mut t, P2, "Rampant Growth", &[]);
        t.resolve_all();
        assert_eq!(count_named(&t, "Forest"), 1, "only its own Forest");
        let trap = t.hand(P0, "Archive Trap");
        let m = alt(&mut t, P0, trap);
        // P1, who didn't search, is the target; P1 has 30 cards, mills 13.
        let lib = t.library_size(P1);
        t.g.turn.priority = Some(P0);
        t.cast(P0, trap).method(m).target(pl(P1)).go();
        t.resolve_all();
        assert_eq!(t.library_size(P1), lib - 13, "mindcensor = {mindcensor}");
    }
    // Fewer than thirteen cards: the whole library.
    let mut t = TestGame::new(2);
    set_library(&mut t, P1, &["Island"; 5]);
    cast_resolve(&mut t, P0, "Archive Trap", "3UU", &[pl(P1)]);
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 5);
}

#[test]
fn commandeers_alternative_cost_needs_two_blue_cards_in_hand() {
    cr!("118.9", "601.2b", "601.2h");
    ruling!(
        "Commandeer",
        "If you don't have two cards of the right color in your hand, you can't choose to cast the spell using the alternative cost."
    );
    for (blue, ok) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        cast_new(&mut t, P1, "Shock", &[pl(P0)]);
        let c = t.hand(P0, "Commandeer");
        t.hand(P0, "Island");
        t.hand(P0, "Lightning Bolt");
        for _ in 0..blue {
            t.hand(P0, "Opt");
        }
        assert_eq!(
            alternatives(&mut t, P0, c).len(),
            ok as usize,
            "{blue} blue cards"
        );
    }
}

#[test]
fn fires_of_invention_lets_you_pay_the_mana_cost_or_another_alternative_cost() {
    cr!("118.9", "118.9a", "702.74a");
    ruling!(
        "Fires of Invention",
        "You may choose to pay the mana cost or an alternative cost for spells you cast, even if their mana value is less than or equal to the number of lands you control."
    );
    supported("Fires of Invention");
    // Mulldrifter (5) with five lands: normally, evoked, or free with Fires.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    t.lands(P0, "Island", 5);
    let m = t.hand(P0, "Mulldrifter");
    let methods = crate::r_s07_common::cast_methods(&mut t, P0, m);
    assert!(methods.contains(&CastMethod::Normal), "{methods:?}");
    assert!(
        methods.contains(&CastMethod::Keyword(KeywordKind::Evoke)),
        "{methods:?}"
    );
    assert!(methods.len() >= 3, "and the free cast: {methods:?}");
    t.cast(P0, m)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    assert_eq!(
        crate::r_s01_common::tapped_lands(&t, P0),
        3,
        "paid {{2}}{{U}}"
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mulldrifter"), "evoked");
}

#[test]
fn a_commander_free_spell_is_cast_with_no_one_acting_in_between() {
    cr!("601.2", "903.3", "118.9");
    ruling!("Deadly Rollick", "Once you begin casting this spell, players can't take any other actions until you're done casting it. Notably, they can't try to remove the commander you control to make you pay its cost.");
    ruling!("Deflecting Swat", "Once you begin casting this spell, players can't take any other actions until you're done casting it. Notably, they can't try to remove the commander you control to make you pay its cost.");
    ruling!("Fierce Guardianship", "Once you begin casting this spell, players can't take any other actions until you're done casting it. Notably, they can't try to remove the commander you control to make you pay its cost.");
    ruling!("Flawless Maneuver", "Once you begin casting this spell, players can't take any other actions until you're done casting it. Notably, they can't try to remove the commander you control to make you pay its cost.");
    ruling!("Obscuring Haze", "Once you begin casting this spell, players can't take any other actions until you're done casting it. Notably, they can't try to remove the commander you control to make you pay its cost.");
    for name in [
        "Deadly Rollick",
        "Deflecting Swat",
        "Fierce Guardianship",
        "Flawless Maneuver",
        "Obscuring Haze",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P1, Step::PrecombatMain);
        let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
        let c = t.hand(P0, name);
        assert!(
            alternatives(&mut t, P0, c).is_empty(),
            "{name}: no commander"
        );
        commander_on_battlefield(&mut t, "Grizzly Bears");
        let m = alt(&mut t, P0, c);
        let targets: Vec<Entity> = match name {
            "Deadly Rollick" => vec![obj(bears)],
            "Deflecting Swat" | "Fierce Guardianship" => vec![obj(shock)],
            _ => vec![],
        };
        let from = t.asked().len();
        t.g.turn.priority = Some(P0);
        let spell = t.cast(P0, c).method(m).targets(&targets).go();
        assert_eq!(t.zone(spell), Zone::Stack, "{name}");
        assert!(others_asked_since(&t, from, P0).is_empty(), "{name}");
        assert!(t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    }
}

#[test]
fn summoning_trap_looks_at_all_of_a_short_library() {
    cr!("608.2c");
    ruling!(
        "Summoning Trap",
        "If there are seven or fewer cards in your library, you'll look at all of them."
    );
    supported("Summoning Trap");
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P0,
        &["Island", "Island", "Island", "Island", "Hill Giant"],
    );
    t.answer_choose(P0, &[obj(lib[4])]);
    cast_resolve(&mut t, P0, "Summoning Trap", "4GG", &[]);
    assert_eq!(count_named(&t, "Hill Giant"), 1);
    assert_eq!(t.library_size(P0), 4);
}
