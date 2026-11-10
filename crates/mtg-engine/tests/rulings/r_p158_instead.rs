//! Rulings batch P158 — self-replacement effects (CR 614.1a, 614.15): an "instead" clause
//! in a spell's or ability's own text replaces the effect it modifies, so only one of the
//! two outcomes happens, chosen by the condition as the effect applies (CR 608.2c): the
//! Sanctuaries, Lightning Surge, Bog Down, Crater's Claws, Regent's Authority, Power
//! Plant Worker, the Empires artifacts, Mythos of Snapdax, Emiel, Expel from Orazca,
//! Nissa's Pilgrimage, Goblin Charbelcher, Rix Maadi Reveler, Increasing Confusion and
//! See the Unwritten.

use crate::r_p158_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Sanctuary with `others` on the battlefield goes to P0's upkeep and everything
/// resolves.
fn sanctuary(sanct: &str, others: &[&str]) -> TestGame {
    supported(sanct);
    let mut t = TestGame::new(2);
    t.battlefield(P0, sanct);
    for o in others {
        t.battlefield(P0, o);
    }
    to_upkeep(&mut t, P0);
    t.resolve_all();
    t
}

#[test]
fn sanctuaries_give_one_amount_or_the_other_never_both() {
    cr!("614.1a", "603.4", "608.2c");
    ruling!("Raka Sanctuary", "It can deal 1 or 3 damage, but never 4.");
    ruling!(
        "Necra Sanctuary",
        "It can make a player lose 0, 1, or 3 life, but never 4."
    );
    ruling!(
        "Necra Sanctuary",
        "One permanent that is both white and green will cause the player to be affected by the lose 3 life portion."
    );
    ruling!(
        "Ceta Sanctuary",
        "The ability has you draw one or two cards, but never three."
    );
    ruling!(
        "Ana Sanctuary",
        "The creature either gets +1/+1 or +5/+5, never +6/+6."
    );
    // Raka (white or blue: 1; white and blue: 3) at P1's Serra Angel.
    let giant_of = |t: &mut TestGame| t.battlefield(P1, "Serra Angel");
    for (others, dmg) in [(&["Savannah Lions"][..], 1), (&["Savannah Lions", "Storm Crow"][..], 3)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Raka Sanctuary");
        for o in others {
            t.battlefield(P0, o);
        }
        let giant = giant_of(&mut t);
        t.answer_targets(P0, &[obj(giant)]);
        to_upkeep(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.obj_now(giant).damage, dmg, "Raka {others:?}");
    }
    // Necra (green or white: 1; green and white: 3; one green-white permanent: 3; neither:
    // nothing) targeting P1.
    for (others, lost) in [
        (&[][..], 0),
        (&["Savannah Lions"][..], 1),
        (&["Savannah Lions", "Grizzly Bears"][..], 3),
        (&["Watchwolf"][..], 3),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Necra Sanctuary");
        for o in others {
            t.battlefield(P0, o);
        }
        t.answer_targets(P0, &[pl(P1)]);
        to_upkeep(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - lost, "Necra {others:?}");
    }
    // Ceta (red or green: draw one then discard; both: draw two then discard), relative
    // to the hand with neither (the turns' draws on the way to the upkeep are the same).
    let t = sanctuary("Ceta Sanctuary", &[]);
    let base = t.hand_size(P0);
    for (others, drawn) in [(&["Gray Ogre"][..], 1), (&["Gray Ogre", "Grizzly Bears"][..], 2)] {
        let t = sanctuary("Ceta Sanctuary", others);
        assert_eq!(t.hand_size(P0), base + drawn - 1, "Ceta {others:?}");
    }
    // Ana (blue or black: +1/+1; both: +5/+5) on the Bears.
    for (others, bonus) in [(&["Storm Crow"][..], 1), (&["Storm Crow", "Cabal Evangel"][..], 5)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ana Sanctuary");
        for o in others {
            t.battlefield(P0, o);
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        to_upkeep(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.pt(bears), (2 + bonus, 2 + bonus), "Ana {others:?}");
    }
}

#[test]
fn lightning_surge_and_bog_down_do_one_amount_or_the_other() {
    cr!("614.1a", "608.2c", "702.33a");
    ruling!(
        "Lightning Surge",
        "It either deals 4 damage or 6 damage. It does not deal 10 damage."
    );
    ruling!("Bog Down", "The player discards 2 cards or 3 cards, not 5 cards.");
    supported("Lightning Surge");
    supported("Bog Down");
    for (threshold, dmg) in [(false, 4), (true, 6)] {
        let mut t = TestGame::new(2);
        if threshold {
            graveyard_n(&mut t, P0, "Grizzly Bears", 7);
        }
        cast_at(&mut t, P0, "Lightning Surge", &[pl(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dmg, "threshold {threshold}");
    }
    for (kicked, discarded) in [(false, 2), (true, 3)] {
        let mut t = TestGame::new(2);
        for _ in 0..5 {
            t.hand(P1, "Grizzly Bears");
        }
        let lands = t.lands(P0, "Swamp", 5);
        let b = t.hand(P0, "Bog Down");
        t.answer_choose(P0, &[obj(lands[3]), obj(lands[4])]);
        t.cast(P0, b).kicked(kicked).target(pl(P1)).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P1), 5 - discarded, "kicked {kicked}");
    }
}

#[test]
fn craters_claws_deals_x_plus_two_with_ferocious() {
    cr!("614.1a", "107.3");
    ruling!(
        "Crater's Claws",
        "If you control a creature with power 4 or greater, Crater’s Claws with X equals 0 will deal 2 damage, with X equals 1 will deal 3 damage, and so on."
    );
    supported("Crater's Claws");
    for (ferocious, x, dmg) in [(true, 0, 2), (true, 1, 3), (true, 3, 5), (false, 1, 1)] {
        let mut t = TestGame::new(2);
        if ferocious {
            t.battlefield(P0, "Lowland Giant");
        }
        lots(&mut t, P0);
        let c = t.hand(P0, "Crater's Claws");
        t.cast(P0, c).x(x).target(pl(P1)).go();
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dmg, "ferocious {ferocious} X {x}");
    }
}

#[test]
fn regents_authority_on_a_legendary_enchantment_creature_applies_once() {
    cr!("614.1a", "608.2c");
    ruling!(
        "Regent's Authority",
        "If it is both an enchantment creature and a legendary creature, you still only put one +1/+1 counter on it and it gets +1/+1 until end of turn."
    );
    supported("Regent's Authority");
    supported("Sythis, Harvest's Hand");
    // Sythis (1/2, a legendary enchantment creature): one counter and +1/+1: 3/4.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sythis, Harvest's Hand");
    cast_at(&mut t, P0, "Regent's Authority", &[obj(s)]);
    t.resolve_all();
    assert_eq!(t.counters(s, "+1/+1"), 1);
    assert_eq!(t.pt(s), (3, 4));
    // Grizzly Bears: +2/+2, no counter.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Regent's Authority", &[obj(b)]);
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 0);
    assert_eq!(t.pt(b), (4, 4));
}

#[test]
fn power_plant_worker_checks_the_other_workers_as_the_ability_resolves() {
    cr!("614.1a", "608.2c");
    ruling!(
        "Power Plant Worker",
        "Power Plant Worker will get two +1/+1 counters instead of getting +2/+2 if, at the time the ability resolves, you control at least one creature named Mine Worker and at least one creature named Tower Worker."
    );
    supported("Power Plant Worker");
    // The other Workers enter in response: counters. One leaves in response: +2/+2.
    for (before, counters) in [(false, 2), (true, 0)] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Power Plant Worker");
        let mut mine = None;
        if before {
            mine = Some(t.battlefield(P0, "Mine Worker"));
            t.battlefield(P0, "Tower Worker");
        }
        act_only(&mut t, P0, w, 0, &[]);
        if let Some(m) = mine {
            kill(&mut t, m);
        } else {
            t.battlefield(P0, "Mine Worker");
            t.battlefield(P0, "Tower Worker");
        }
        t.resolve_all();
        assert_eq!(t.counters(w, "+1/+1"), counters, "before {before}");
        assert_eq!(t.pt(w), (6, 6), "before {before}: a 4/4 either way");
    }
}

/// Removes the artifact type from the permanent until end of turn.
fn not_an_artifact(t: &mut TestGame, id: ObjectId) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::RemoveTypes(vec![CardType::Artifact])],
    );
    assert!(!is_artifact(t, id));
}

#[test]
fn the_empires_artifacts_count_only_artifacts_with_the_names() {
    cr!("614.1a", "608.2c", "201.2");
    ruling!(
        "Crown of Empires",
        "If any of the named cards stops being an artifact, it won’t be considered by these abilities."
    );
    ruling!(
        "Throne of Empires",
        "If any of the named cards stops being an artifact, it won’t be considered by these abilities."
    );
    ruling!(
        "Crown of Empires",
        "If you control artifacts named Scepter of Empires and Throne of Empires, the targeted creature won’t be tapped. You’ll just gain control of it."
    );
    supported("Crown of Empires");
    supported("Throne of Empires");
    supported("Scepter of Empires");
    // Crown with the other two: gain control, untapped; with the Scepter no longer an
    // artifact: tapped, not gained.
    for complete in [true, false] {
        let mut t = TestGame::new(2);
        let crown = t.battlefield(P0, "Crown of Empires");
        let scepter = t.battlefield(P0, "Scepter of Empires");
        t.battlefield(P0, "Throne of Empires");
        let bears = t.battlefield(P1, "Grizzly Bears");
        if !complete {
            not_an_artifact(&mut t, scepter);
        }
        act(&mut t, P0, crown, 0, &[obj(bears)]);
        assert_eq!(ctl(&t, bears), if complete { P0 } else { P1 }, "complete {complete}");
        assert_eq!(t.obj_now(bears).tapped, !complete, "complete {complete}");
    }
    // Throne with the other two: five Soldiers; with the Crown no longer an artifact: one.
    for complete in [true, false] {
        let mut t = TestGame::new(2);
        let throne = t.battlefield(P0, "Throne of Empires");
        let crown = t.battlefield(P0, "Crown of Empires");
        t.battlefield(P0, "Scepter of Empires");
        if !complete {
            not_an_artifact(&mut t, crown);
        }
        act(&mut t, P0, throne, 0, &[]);
        assert_eq!(with_subtype(&t, P0, "Soldier").len(), if complete { 5 } else { 1 }, "complete {complete}");
    }
}

#[test]
fn mythos_of_snapdax_may_keep_one_permanent_for_several_types_and_spares_lands() {
    cr!("608.2c", "205.2a");
    ruling!(
        "Mythos of Snapdax",
        "If you control a permanent with more than one type, you may choose it for each of those types. For example, you could choose an artifact creature as the artifact you keep and also as the creature you keep."
    );
    ruling!(
        "Mythos of Snapdax",
        "Lands with another permanent type can’t be chosen and won’t be sacrificed."
    );
    supported("Mythos of Snapdax");
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let arbor = t.battlefield(P0, "Dryad Arbor");
    let rock = t.battlefield(P0, "Sunglasses of Urza");
    t.lands(P0, "Plains", 4);
    let m = t.hand(P0, "Mythos of Snapdax");
    // The artifact kept: the Ornithopter; the creature kept: the Ornithopter again.
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_choose(P0, &[obj(thopter)]);
    let from = t.asked().len();
    t.cast(P0, m).go();
    t.resolve_all();
    assert!(t.on_battlefield(thopter));
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(rock));
    assert!(t.on_battlefield(arbor), "a land, even a creature, is neither chosen nor sacrificed");
    for (p, d) in asked_since(&t, from) {
        if let Decision::ChooseEntities { candidates, .. } = d {
            assert!(p == P0 && !candidates.contains(&obj(arbor)), "{candidates:?}");
        }
    }
}

#[test]
fn emiel_still_charges_for_a_unicorns_two_counters() {
    cr!("614.1a", "118.12");
    ruling!(
        "Emiel the Blessed",
        "If the entering creature is a Unicorn, you still have to pay {G/W} to put two +1/+1 counters on it."
    );
    supported("Emiel the Blessed");
    supported("Regal Unicorn");
    for (creature, pay, counters) in [
        ("Regal Unicorn", true, 2),
        ("Regal Unicorn", false, 0),
        ("Grizzly Bears", true, 1),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Emiel the Blessed");
        if pay {
            mana(&mut t, P0, ManaType::W, 1);
        }
        t.answer_yes(P0, true);
        let c = t.enter(P0, creature);
        t.resolve_all();
        assert_eq!(t.counters(c, "+1/+1"), counters, "{creature} pay {pay}");
        assert_eq!(t.g.players[0].mana_pool.total(), 0, "{creature} pay {pay}");
    }
}

#[test]
fn expel_from_orazca_returns_to_hand_when_you_decline_the_library() {
    cr!("614.1a", "702.131a");
    ruling!(
        "Expel from Orazca",
        "If you have the city’s blessing and choose not to put the target nonland permanent on top of its owner’s library, it’s returned to its owner’s hand."
    );
    supported("Expel from Orazca");
    for (blessing, top) in [(true, true), (true, false), (false, false)] {
        let mut t = TestGame::new(2);
        t.g.players[0].has_citys_blessing = blessing;
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_at(&mut t, P0, "Expel from Orazca", &[obj(bears)]);
        let from = t.asked().len();
        t.answer_yes(P0, top);
        t.resolve_all();
        let asked = asked_since(&t, from)
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }));
        assert_eq!(asked, blessing, "blessing {blessing}");
        assert_eq!(t.in_hand(P1, "Grizzly Bears"), !top, "blessing {blessing} top {top}");
        if top {
            assert_eq!(crate::r_s33_common::library_top_first(&t, P1)[0], t.g.current(bears));
        }
    }
}

#[test]
fn nissas_pilgrimage_puts_a_lone_forest_onto_the_battlefield() {
    cr!("608.2c", "701.20a");
    ruling!(
        "Nissa's Pilgrimage",
        "If you only find one basic Forest card, you’ll put it onto the battlefield tapped. You won’t be able to put it into your hand, even if you want to."
    );
    supported("Nissa's Pilgrimage");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    cast_at(&mut t, P0, "Nissa's Pilgrimage", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert!(t.obj_now(forest).tapped);
    assert!(!t.in_hand(P0, "Forest"));
}

#[test]
fn goblin_charbelcher_without_a_land_counts_every_card_revealed() {
    cr!("608.2c", "701.20a");
    ruling!(
        "Goblin Charbelcher",
        "If you reveal no land cards, Goblin Charbelcher deals damage equal to the number of cards revealed, and then you may order your library as you like."
    );
    supported("Goblin Charbelcher");
    let mut t = TestGame::new(2);
    let belcher = t.battlefield(P0, "Goblin Charbelcher");
    let lib = crate::r_p189_common::set_library(&mut t, P0, &["Shock", "Grizzly Bears", "Hill Giant"]);
    t.lands(P0, "Plains", 3);
    act(&mut t, P0, belcher, 0, &[pl(P1)]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.library_size(P0), 3);
    let now: Vec<ObjectId> = lib.iter().map(|id| t.g.current(*id)).collect();
    let mut sorted = crate::r_s33_common::library_top_first(&t, P0);
    sorted.sort();
    let mut expected = now;
    expected.sort();
    assert_eq!(sorted, expected, "the cards are back in the library");
}

#[test]
fn rix_maadi_reveler_with_no_cards_in_hand_just_draws() {
    cr!("608.2c", "701.8b", "702.137a");
    ruling!(
        "Rix Maadi Reveler",
        "If you have no cards in hand, you won't discard any cards, and then you'll draw one or three cards as appropriate."
    );
    supported("Rix Maadi Reveler");
    for (spectacle, drawn) in [(false, 1), (true, 3)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        if spectacle {
            lose(&mut t, P1, 1);
        }
        lots(&mut t, P0);
        let r = t.hand(P0, "Rix Maadi Reveler");
        assert_eq!(t.hand_size(P0), 1);
        let method = if spectacle {
            CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Spectacle)
        } else {
            CastMethod::Normal
        };
        t.cast(P0, r).method(method).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), drawn, "spectacle {spectacle}");
    }
}

#[test]
fn increasing_confusion_mills_the_target_from_either_zone() {
    cr!("614.1a", "702.34a", "701.13a");
    ruling!(
        "Increasing Confusion",
        "Regardless of which zone Increasing Confusion is cast from, it puts cards from the top of target player's library into that player's graveyard."
    );
    supported("Increasing Confusion");
    for (from_graveyard, milled) in [(false, 2), (true, 4)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let library = t.library_size(P1);
        let own = t.library_size(P0);
        if from_graveyard {
            let c = t.graveyard(P0, "Increasing Confusion");
            lots(&mut t, P0);
            t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
            cast_from_graveyard(&mut t, P0, c, &[pl(P1)]);
        } else {
            lots(&mut t, P0);
            let c = t.hand(P0, "Increasing Confusion");
            t.cast(P0, c).x(2).target(pl(P1)).go();
        }
        t.resolve_all();
        assert_eq!(t.library_size(P1), library - milled, "from graveyard {from_graveyard}");
        assert_eq!(t.library_size(P0), own, "from graveyard {from_graveyard}");
        assert_eq!(t.graveyard_size(P1), milled, "from graveyard {from_graveyard}");
    }
}

#[test]
fn see_the_unwritten_puts_an_x_creature_onto_the_battlefield_with_x_zero() {
    cr!("107.3", "608.2c", "704.5f");
    ruling!(
        "See the Unwritten",
        "If a card you put onto the battlefield this way has {X} in its mana cost, X is considered to be 0."
    );
    supported("See the Unwritten");
    supported("Endless One");
    let mut t = TestGame::new(2);
    let endless = t.library_top(P0, "Endless One");
    t.answer_choose(P0, &[obj(endless)]);
    cast_at(&mut t, P0, "See the Unwritten", &[]);
    t.resolve_all();
    // Endless One enters with X = 0 +1/+1 counters: a 0/0, it dies.
    assert!(t.in_graveyard(P0, "Endless One"));
    assert_eq!(t.counters(endless, "+1/+1"), 0);
}
