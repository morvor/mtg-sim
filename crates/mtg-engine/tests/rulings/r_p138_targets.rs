//! Rulings batch P138 — targeted edicts and their illegal targets: an ability or spell
//! whose targets are all illegal doesn't resolve (CR 608.2b), one with some legal targets
//! does what it can (CR 608.2b, 608.2c), and "can't sacrifice" leaves the rest of the
//! effect (CR 701.21a, 609.3).

use crate::r_p138_common::*;
use crate::r_s01_common::stack_library;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Returns the permanent to its owner's hand, making it an illegal target.
fn bounce(t: &mut TestGame, id: ObjectId) {
    let owner = t.obj_now(id).owner;
    move_to(t, id, Zone::Hand(owner));
}

#[test]
fn vaevictis_a_target_that_isnt_sacrificed_means_no_reveal_for_its_controller() {
    cr!("608.2b", "701.21a");
    ruling!(
        "Vaevictis Asmadi, the Dire",
        "If any of the target permanents aren't sacrificed (most likely because one of them became an illegal target, such as by gaining hexproof), that permanent's controller won't reveal the top card of their library or put it onto the battlefield."
    );
    supported("Vaevictis Asmadi, the Dire");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vaevictis Asmadi, the Dire");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = stack_library(&mut t, P0, &["Hill Giant"])[0];
    let theirs = stack_library(&mut t, P1, &["Hill Giant"])[0];
    t.answer_targets(P0, &[obj(forest)]);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(v, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    // P1's target becomes illegal.
    bounce(&mut t, bears);
    t.resolve_all();
    assert!(!t.on_battlefield(forest));
    assert!(t.on_battlefield(mine), "P0 sacrificed: their Hill Giant enters");
    assert_eq!(t.zone(theirs), Zone::Library(P1), "P1 didn't sacrifice");
}

#[test]
fn modal_spells_with_one_legal_target_do_what_they_can() {
    cr!("608.2b", "700.2");
    ruling!(
        "Silverquill Command",
        "If at least one target is still legal as Silverquill Command tries to resolve, it will resolve and do as much as it can."
    );
    supported("Silverquill Command");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    // "Target creature gets +3/+3 ..." and "Target opponent sacrifices a creature".
    lots(&mut t, P0);
    let c = t.hand(P0, "Silverquill Command");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, c).modes(&[0, 3]).go();
    bounce(&mut t, mine);
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn grave_exchange_does_what_its_legal_targets_allow() {
    cr!("608.2b");
    ruling!(
        "Grave Exchange",
        "If both targets are illegal when Grave Exchange tries to resolve, it won’t resolve and none of its effects will happen. If only one target is legal, the effect relevant to that target will happen and the other will not."
    );
    supported("Grave Exchange");
    // (card legal, player legal), with each target made illegal or not.
    for (card_ok, player_ok) in [(false, false), (true, false), (false, true)] {
        let mut t = TestGame::new(2);
        let dead = t.graveyard(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Hill Giant");
        cast(&mut t, P0, "Grave Exchange", &[&[obj(dead)], &[pl(P1)]]);
        if !card_ok {
            move_to(&mut t, dead, Zone::Exile);
        }
        if !player_ok {
            t.battlefield(P1, "Leyline of Sanctity");
        }
        t.resolve_all();
        let what = format!("card {card_ok}, player {player_ok}");
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), card_ok, "{what}");
        assert_eq!(!t.on_battlefield(theirs), player_ok, "{what}");
    }
}

#[test]
fn necrotic_plague_stays_in_the_graveyard_without_a_legal_target() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Necrotic Plague",
        "If no legal target can be chosen for Necrotic Plague's last ability, it just remains in its owner's graveyard. Similarly, if the targeted creature becomes an illegal target by the time the ability resolves, the ability doesn't resolve and Necrotic Plague remains in its owner's graveyard."
    );
    supported("Necrotic Plague");
    for with_target in [false, true] {
        let mut t = TestGame::new(2);
        let host = t.battlefield(P1, "Grizzly Bears");
        let plague = t.battlefield(P0, "Necrotic Plague");
        assert!(t.g.attach(plague, obj(host)));
        let target = with_target.then(|| t.battlefield(P0, "Hill Giant"));
        if let Some(c) = target {
            t.answer_targets(P1, &[obj(c)]);
        }
        kill(&mut t, host);
        assert_eq!(t.zone(plague), Zone::Graveyard(P0));
        if let Some(c) = target {
            assert_eq!(t.stack_len(), 1);
            bounce(&mut t, c);
        } else {
            assert_eq!(t.stack_len(), 0);
        }
        t.resolve_all();
        assert_eq!(t.zone(plague), Zone::Graveyard(P0), "with target {with_target}");
    }
}

#[test]
fn arcum_dagsson_searches_if_the_target_is_legal_even_when_not_sacrificed() {
    cr!("608.2b", "701.21a");
    ruling!(
        "Arcum Dagsson",
        "If the target artifact creature is an illegal target by the time Arcum's ability tries to resolve, the ability won't resolve. No player searches their library. If the target is legal but not sacrificed (most likely because of an effect such as that of Sigarda, Host of Herons), its controller does search their library for a noncreature artifact card."
    );
    for legal in [false, true] {
        let mut t = TestGame::new(2);
        let arcum = t.battlefield(P0, "Arcum Dagsson");
        t.battlefield(P1, "Sigarda, Host of Herons");
        let thopter = t.battlefield(P1, "Ornithopter");
        let ring = t.library_top(P1, "Sol Ring");
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[obj(ring)]);
        t.activate(P0, arcum, 0, &[obj(thopter)]).expect("activated");
        if !legal {
            bounce(&mut t, thopter);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(thopter), legal, "legal {legal}");
        assert_eq!(t.on_battlefield(ring), legal, "legal {legal}: searched");
    }
}

#[test]
fn star_athlete_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Star Athlete",
        "If the target permanent is an illegal target as Star Athlete's second ability tries to resolve, it won't resolve and none of its effects will happen. No player will be dealt damage, and no permanents will be sacked . . . er, sacrificed."
    );
    supported("Star Athlete");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let athlete = t.battlefield(P0, "Star Athlete");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_yes(P1, false);
        attack_with(&mut t, &[(athlete, pl(P1))]);
        if !legal {
            bounce(&mut t, bears);
        }
        t.resolve_all();
        // (P1 declines to sacrifice: 5 damage, if the target is legal.)
        assert_eq!(t.life(P1), if legal { 15 } else { 20 }, "legal {legal}");
    }
}

#[test]
fn shape_anew_illegal_target_nothing_and_unsacrificed_artifact_still_reveals() {
    cr!("608.2b", "701.21a", "701.20a");
    ruling!(
        "Shape Anew",
        "If the targeted artifact is an illegal target by the time Shape Anew resolves, the spell doesn’t resolve. Nothing else happens."
    );
    ruling!(
        "Shape Anew",
        "If the targeted artifact’s controller can’t sacrifice it (due to Tajuru Preserver, perhaps), the other effects of the spell will still happen."
    );
    supported("Shape Anew");
    // 0: illegal target; 1: Tajuru Preserver.
    for case in 0..2 {
        let mut t = TestGame::new(2);
        let ring = t.battlefield(P1, "Sol Ring");
        if case == 1 {
            t.battlefield(P1, "Tajuru Preserver");
        }
        let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Ornithopter"]);
        cast(&mut t, P0, "Shape Anew", &[&[obj(ring)]]);
        if case == 0 {
            bounce(&mut t, ring);
        }
        t.resolve_all();
        if case == 0 {
            assert_eq!(t.zone(lib[1]), Zone::Library(P1));
            assert_eq!(t.zone(lib[0]), Zone::Library(P1));
        } else {
            assert!(t.on_battlefield(ring), "not sacrificed");
            assert!(t.on_battlefield(lib[1]), "the revealed artifact enters");
        }
    }
}

#[test]
fn shape_anew_without_artifacts_reveals_the_library_and_shuffles() {
    cr!("701.20a", "701.24a");
    ruling!(
        "Shape Anew",
        "If there are no artifact cards in the player’s library, all the cards in that library are revealed, then the library is shuffled. (The targeted artifact remains sacrificed.)"
    );
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let before = t.library_size(P1);
    cast(&mut t, P0, "Shape Anew", &[&[obj(ring)]]);
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert_eq!(t.zone(ring), Zone::Graveyard(P1));
    assert_eq!(t.library_size(P1), before);
    assert_eq!(permanents_of(&t, P1).len(), 0);
}

#[test]
fn din_of_the_fireherd_with_an_illegal_opponent_creates_no_token() {
    cr!("608.2b");
    ruling!(
        "Din of the Fireherd",
        "If the targeted opponent becomes an illegal target, the entire spell doesn’t resolve. You won’t get a token."
    );
    supported("Din of the Fireherd");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Din of the Fireherd", &[&[pl(P1)]]);
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert!(creatures(&t, P0).is_empty());
    assert!(t.on_battlefield(theirs));
}

#[test]
fn consuming_vapors_gains_nothing_if_nothing_is_sacrificed() {
    cr!("701.21a", "609.3");
    ruling!(
        "Consuming Vapors",
        "If the targeted player doesn't sacrifice a creature (because the player didn't control any creatures or due to Tajuru Preserver, perhaps), you gain no life."
    );
    supported("Consuming Vapors");
    for preserver in [false, true] {
        let mut t = TestGame::new(2);
        if preserver {
            t.battlefield(P1, "Tajuru Preserver");
        }
        cast(&mut t, P0, "Consuming Vapors", &[&[pl(P1)]]);
        t.resolve();
        assert_eq!(t.life(P0), 20, "preserver {preserver}");
    }
}
