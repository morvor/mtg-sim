//! Rulings batch P060 — other draw effects: last known information for a source that left
//! (CR 608.2h), linked exile that ends with the object (CR 607.2a, 400.7), granted
//! triggered abilities that stack, sacrifice restrictions, simultaneous madness discards
//! (CR 101.4, 702.35), names of double-faced cards in other zones (CR 712.8), destroy
//! results, free casts (CR 118.9), multiplayer opponents, tapping costs, triggers that
//! resolve before their spell, and searches that skip the battlefield.

use crate::r_p007_common::loyalty;
use crate::r_p060_common::*;
use crate::r_s01_common::triggers_on_stack;
use crate::r_s02_common::can_activate;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn communal_brewing_uses_its_last_known_ingredient_counters() {
    cr!("608.2h", "603.10");
    ruling!(
        "Communal Brewing",
        "If Communal Brewing leaves the battlefield before its last ability resolves, use the number of ingredient counters that were on it as it last existed on the battlefield to determine the value of X."
    );
    supported("Communal Brewing");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        // P1 draws a card: two ingredient counters.
        t.answer_targets(P0, &[pl(P1)]);
        let cb = t.enter(P0, "Communal Brewing");
        t.resolve_all();
        assert_eq!(t.counters(cb, "ingredient"), 2);
        t.lands(P0, "Forest", 2);
        let bears = t.hand(P0, "Grizzly Bears");
        t.cast(P0, bears).go();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        if gone {
            destroy(&mut t, cb);
        }
        t.resolve_all();
        let b = t.named_on_battlefield("Grizzly Bears")[0];
        assert_eq!(t.counters(b, "+1/+1"), 2, "gone: {gone}");
    }
}

#[test]
fn induced_amnesia_gone_before_its_first_ability_exiles_forever() {
    cr!("603.6c", "607.2a", "610.3");
    ruling!(
        "Induced Amnesia",
        "If Induced Amnesia leaves the battlefield before its first ability resolves, its second ability will trigger (if appropriate) and do nothing. Then its first ability will resolve, and the exiled cards will be exiled indefinitely."
    );
    supported("Induced Amnesia");
    let mut t = TestGame::new(2);
    let cards = bears_in_hand(&mut t, P1, 2);
    t.answer_targets(P0, &[pl(P1)]);
    let ia = t.enter(P0, "Induced Amnesia");
    t.settle();
    destroy(&mut t, ia);
    // Its second ability triggered and is on top of the first.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, "return the exiled cards"), 1);
    t.resolve();
    assert_eq!(t.zone(cards[0]), Zone::Hand(P1), "nothing returned yet");
    t.resolve_all();
    // P1 exiled the two Bears and drew two; the Bears stay exiled.
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    assert_eq!(t.hand_size(P1), 2);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
}

#[test]
fn induced_amnesia_cards_are_lost_if_it_leaves_other_than_to_a_graveyard() {
    cr!("400.7", "607.2a");
    ruling!(
        "Induced Amnesia",
        "If Induced Amnesia leaves the battlefield but isn’t put into a graveyard, the exiled cards are lost forever. They won’t be returned if another Induced Amnesia is put into a graveyard, even if that Induced Amnesia is represented by the same card."
    );
    // Positive control: destroyed, it returns the cards.
    let mut t = TestGame::new(2);
    let cards = bears_in_hand(&mut t, P1, 2);
    t.answer_targets(P0, &[pl(P1)]);
    let ia = t.enter(P0, "Induced Amnesia");
    t.resolve_all();
    destroy(&mut t, ia);
    t.resolve_all();
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Hand(P1)));
    // Returned to P0's hand by Boomerang, then cast again and destroyed: the first two
    // Bears stay in exile; only what the new object exiled comes back.
    let mut t = TestGame::new(2);
    let cards = bears_in_hand(&mut t, P1, 2);
    t.answer_targets(P0, &[pl(P1)]);
    let ia = t.enter(P0, "Induced Amnesia");
    t.resolve_all();
    t.lands(P0, "Island", 2);
    let boom = t.hand(P0, "Boomerang");
    t.cast(P0, boom).target(ia).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Induced Amnesia"));
    let second: Vec<ObjectId> = t.g.player(P1).hand.clone();
    t.lands(P0, "Island", 3);
    let card = t.g.current(ia);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, card).go();
    t.resolve_all();
    let ia2 = t.named_on_battlefield("Induced Amnesia")[0];
    destroy(&mut t, ia2);
    t.resolve_all();
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Exile), "lost forever");
    assert!(second.iter().all(|c| t.zone(*c) == Zone::Hand(P1)));
}

#[test]
fn mathas_two_bounty_abilities_trigger_separately() {
    cr!("113.2c", "603.2");
    ruling!(
        "Mathas, Fiend Seeker",
        "If Mathas's ability resolves two separate times targeting the same creature, that creature will have two instances of the triggered ability. Each triggers separately when that creature dies."
    );
    supported("Mathas, Fiend Seeker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mathas, Fiend Seeker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Two of P0's end steps.
    for _ in 0..2 {
        t.set_step(P0, Step::PostcombatMain);
        t.answer_targets(P0, &[obj(bears)]);
        t.advance_to(P0, Step::End);
        t.resolve_all();
    }
    assert_eq!(t.counters(bears, "bounty"), 2);
    let before = libraries(&t);
    destroy(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, "gains 2 life"), 2);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![2, 0]);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn garland_you_must_sacrifice_a_creature_you_own() {
    cr!("701.21a", "101.3");
    ruling!(
        "Garland, Royal Kidnapper",
        "If an effect instructs you to sacrifice a creature and you control any creatures other than ones you control but don't own, you must sacrifice one of those other creatures. You can't try to sacrifice a creature you control but don't own."
    );
    supported("Garland, Royal Kidnapper");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Garland makes P1 the monarch; P0 gains control of P1's Bears.
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(bears)]);
    let garland = t.enter(P0, "Garland, Royal Kidnapper");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    // P1's Diabolic Edict: P0 sacrifices a creature. P0 tries to choose the Bears.
    t.lands(P1, "Swamp", 2);
    let edict = t.hand(P1, "Diabolic Edict");
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.cast(P1, edict).target(pl(P0)).go();
    t.resolve_all();
    // The Bears weren't offered; Garland was sacrificed.
    for (p, d) in &t.asked()[from..] {
        if let (true, Decision::ChooseEntities { candidates, .. }) = (*p == P0, d) {
            assert!(!candidates.contains(&obj(bears)));
        }
    }
    assert!(!t.on_battlefield(garland));
    assert!(t.on_battlefield(bears));
}

#[test]
fn simultaneous_madness_discards_stack_in_apnap_order() {
    cr!("101.4", "603.3b", "702.35a");
    ruling!(
        "Geier Reach Sanitarium",
        "If more than one player discards a card with madness simultaneously, the player whose turn it is puts their madness abilities onto the stack first, then each other player does the same in turn order. The last one put onto the stack resolves first, and a spell cast this way will resolve before resolving the next madness ability."
    );
    supported("Geier Reach Sanitarium");
    supported("Fiery Temper");
    // Both players discard Fiery Temper (madness {R}: 3 damage to any target), each
    // aiming at the other. P1's madness ability resolves first and P1's Temper resolves
    // before P0's madness ability: P0 has taken 3 when asked whether to cast theirs.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Geier Reach Sanitarium");
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 1);
    let f0 = t.hand(P0, "Fiery Temper");
    let f1 = t.hand(P1, "Fiery Temper");
    t.answer_choose_moved(P0, &[obj(f0)]);
    t.answer_choose_moved(P1, &[obj(f1)]);
    for p in [P0, P1] {
        t.answer_yes(p, true);
    }
    t.answer_targets(P1, &[pl(P0)]);
    t.answer_targets(P0, &[pl(P1)]);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.player(PlayerId(0)).life,
    );
    crate::r_s06_common::activate_containing(&mut t, P0, s, "Each player draws").unwrap();
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![17]);
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
}

#[test]
fn a_back_face_name_isnt_found_in_other_zones() {
    cr!("712.8a", "712.8c", "201.2");
    ruling!(
        "Test of Talents",
        "If the back face of a modal double-faced card is countered, you will not be able to exile any cards, including the one that you countered, because those cards have only their front-face characteristics (including name) in the graveyard, hand, and library."
    );
    ruling!(
        "The End",
        "If the permanent that's exiled was the back face of a double-faced card, you will not be able to exile any additional cards, because those cards have only their front-face characteristics (including name) in the graveyard, hand, and library."
    );
    supported("Test of Talents");
    supported("The End");
    const MDFC: &str = "Augmenter Pugilist // Echoing Equation";
    // P1 casts Echoing Equation (the sorcery back face) with another copy of the card in
    // hand and in library; P0 counters it with Test of Talents: nothing is exiled.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Island", 2);
    t.lands(P1, "Wastes", 3);
    let eq = t.hand(P1, MDFC);
    let other = t.hand(P1, MDFC);
    let lib = t.library_top(P1, MDFC);
    t.cast(P1, eq)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    t.lands(P0, "Island", 2);
    let tot = t.hand(P0, "Test of Talents");
    t.cast(P0, tot).target(eq).go();
    let hand = t.hand_size(P1);
    t.resolve();
    assert_eq!(t.zone(eq), Zone::Graveyard(P1), "countered, not exiled");
    assert_eq!(t.zone(other), Zone::Hand(P1));
    assert_eq!(t.zone(lib), Zone::Library(P1));
    assert_eq!(t.hand_size(P1), hand, "nothing exiled from hand, nothing drawn");
    // Positive control: a Lightning Bolt with another in hand is exiled with it.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let other = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(pl(P0)).go();
    t.lands(P0, "Island", 2);
    let tot = t.hand(P0, "Test of Talents");
    t.cast(P0, tot).target(bolt).go();
    t.resolve();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.zone(other), Zone::Exile);
    assert_eq!(t.hand_size(P1), 1, "drew one for the card exiled from hand");
    // The End exiles P1's Tibalt, Cosmic Impostor (the planeswalker back face of Valki):
    // the Valki cards in P1's hand and library aren't found.
    const VALKI: &str = "Valki, God of Lies // Tibalt, Cosmic Impostor";
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 6);
    t.lands(P1, "Swamp", 1);
    let tibalt = t.hand(P1, VALKI);
    let other = t.hand(P1, VALKI);
    let lib = t.library_top(P1, VALKI);
    t.cast(P1, tibalt).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let tibalt = t.g.current(tibalt);
    assert!(t.on_battlefield(tibalt));
    assert_eq!(t.obj(tibalt).chars.name, "Tibalt, Cosmic Impostor");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 4);
    let end = t.hand(P0, "The End");
    let hand = t.hand_size(P1);
    t.cast(P0, end).target(tibalt).go();
    t.resolve_all();
    assert_eq!(t.zone(tibalt), Zone::Exile);
    assert_eq!(t.zone(other), Zone::Hand(P1));
    assert_eq!(t.zone(lib), Zone::Library(P1));
    assert_eq!(t.hand_size(P1), hand);
}

#[test]
fn acolyte_hybrid_draws_only_if_the_artifact_was_destroyed() {
    cr!("701.8a", "702.12b", "614.1a");
    ruling!(
        "Acolyte Hybrid",
        "If the target artifact has indestructible, it isn't destroyed this way and you won't draw a card. If it is destroyed but put into a zone other than a graveyard, you'll draw a card."
    );
    supported("Acolyte Hybrid");
    // (artifact, Rest in Peace, P1 draws)
    for (art, rip, drawn) in [
        ("Sol Ring", false, 1),
        ("Darksteel Myr", false, 0),
        ("Sol Ring", true, 1),
    ] {
        let mut t = TestGame::new(2);
        let hybrid = t.battlefield(P0, "Acolyte Hybrid");
        let a = t.battlefield(P1, art);
        if rip {
            t.battlefield(P0, "Rest in Peace");
        }
        t.answer_targets(P0, &[obj(a)]);
        let before = libraries(&t);
        crate::r_s01_common::attack_with(&mut t, &[(hybrid, pl(P1))]);
        t.resolve_all();
        assert_eq!(drawn_since(&t, &before), vec![0, drawn], "{art}, {rip}");
        let gone = art == "Sol Ring";
        assert_eq!(!t.on_battlefield(a), gone);
        if rip {
            assert_eq!(t.zone(a), Zone::Exile);
        }
    }
}

#[test]
fn jace_beleren_mills_the_whole_library_if_it_has_fewer_than_twenty() {
    cr!("701.17a", "701.17b");
    ruling!(
        "Jace Beleren",
        "If there are fewer than twenty cards in the targeted player's library, that player puts all the cards from their library into their graveyard."
    );
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    library_of(&mut t, P1, 12);
    let jace = pw(&mut t, P0, "Jace Beleren", 10);
    loyalty(&mut t, P0, jace, 2, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 12);
    assert!(!t.has_lost(P1), "milling isn't drawing");
}

#[test]
fn chandra_ablaze_free_casts_pay_kicker_but_no_alternative_cost() {
    cr!("118.9", "118.9a", "601.2b", "601.2f");
    ruling!(
        "Chandra Ablaze",
        "If you cast a card \"without paying its mana cost,\" you can't pay any alternative costs. You can pay additional costs, such as kicker costs."
    );
    supported("Chandra Ablaze");
    // Chandra's −7 casts Burst Lightning (kicker {4}) kicked, and Fireblast without its
    // alternative cost (sacrificing two Mountains) being offered.
    let mut t = TestGame::new(2);
    let ch = pw(&mut t, P0, "Chandra Ablaze", 7);
    let burst = t.graveyard(P0, "Burst Lightning");
    let fb = t.graveyard(P0, "Fireblast");
    let mountains = t.lands(P0, "Mountain", 4);
    t.answer_choose(P0, &[obj(burst), obj(fb)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    let from = t.asked().len();
    loyalty(&mut t, P0, ch, 2, &[]);
    t.resolve_all();
    // Kicked Burst Lightning: 4; Fireblast: 4.
    assert_eq!(t.life(P1), 12);
    assert!(mountains.iter().all(|m| t.on_battlefield(*m)), "nothing sacrificed");
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseCastingMethod { .. }
    )));
}

#[test]
fn malignant_growth_affects_each_opponent() {
    cr!("102.2", "102.3");
    ruling!("Malignant Growth", "In multiplayer games it affects all opponents.");
    supported("Malignant Growth");
    let mut t = TestGame::new(3);
    let mg = t.battlefield(P0, "Malignant Growth");
    t.g.objects[mg.0 as usize].counters.insert("growth".into(), 2);
    for p in [P1, P2] {
        t.set_step(PlayerId(p.0 - 1), Step::End);
        let before = libraries(&t);
        t.advance_to(p, Step::Draw);
        t.settle();
        t.resolve_all();
        let d = drawn_since(&t, &before);
        // The draw-step draw plus two.
        assert_eq!(d[p.idx()], 3, "{p}");
        assert_eq!(t.life(p), 18, "{p}");
    }
    assert_eq!(t.life(P0), 20);
}

#[test]
fn folio_of_fancies_needs_to_untap_for_both_abilities() {
    cr!("602.2b", "118.3");
    ruling!(
        "Folio of Fancies",
        "It's not possible to pay {X}{X}{2}{U} and activate both abilities of Folio of Fancies. You'll need to find a way to untap it to activate both abilities during one turn."
    );
    let mut t = TestGame::new(2);
    let folio = t.battlefield(P0, "Folio of Fancies");
    t.lands(P0, "Island", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, folio, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(folio).tapped);
    assert!(!can_activate(&mut t, P0, folio), "tapped: neither ability");
    // Voltaic Key untaps it: the second ability can be activated.
    let key = t.battlefield(P0, "Voltaic Key");
    t.activate(P0, key, 0, &[obj(folio)]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(folio).tapped);
    let lib = t.library_size(P1);
    crate::r_s06_common::activate_containing(&mut t, P0, folio, "Each opponent mills").unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - t.hand_size(P1));
}

#[test]
fn ms_bumbleflowers_trigger_resolves_first_even_if_its_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Ms. Bumbleflower",
        "Ms. Bumbleflower's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Ms. Bumbleflower");
    let mut t = TestGame::new(2);
    let mb = t.battlefield(P0, "Ms. Bumbleflower");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(mb)]);
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters the Bears.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 1]);
    assert_eq!(t.counters(mb, "+1/+1"), 1);
    // Without the counter, the trigger resolves before the spell: the Bears are still on
    // the stack as it does.
    let mut t = TestGame::new(2);
    let mb = t.battlefield(P0, "Ms. Bumbleflower");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(mb)]);
    t.cast(P0, bears).go();
    t.settle();
    t.g.resolve_top();
    assert_eq!(t.counters(mb, "+1/+1"), 1);
    assert_eq!(t.zone(bears), Zone::Stack);
}

#[test]
fn martyrs_cry_exiles_only_creatures_on_the_battlefield() {
    cr!("109.2", "406.1");
    ruling!(
        "Martyr's Cry",
        "Only affects creatures on the battlefield, not ones in hands or graveyards."
    );
    supported("Martyr's Cry");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let in_hand = t.hand(P1, "White Knight");
    let in_gy = t.graveyard(P1, "White Knight");
    t.lands(P0, "Plains", 2);
    let mc = t.hand(P0, "Martyr's Cry");
    t.cast(P0, mc).go();
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(t.zone(knight), Zone::Exile);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(in_hand), Zone::Hand(P1));
    assert_eq!(t.zone(in_gy), Zone::Graveyard(P1));
    assert_eq!(drawn_since(&t, &before), vec![0, 1]);
}

#[test]
fn naming_searches_skip_permanents_on_the_battlefield() {
    cr!("701.23a", "400.1");
    ruling!(
        "Ancient Vendetta",
        "The ability has no effect on permanents with the chosen name that are on the battlefield."
    );
    ruling!(
        "The Stone Brain",
        "The ability has no effect on permanents with the chosen name that are on the battlefield."
    );
    supported("Ancient Vendetta");
    supported("The Stone Brain");
    for card in ["Ancient Vendetta", "The Stone Brain"] {
        let mut t = TestGame::new(2);
        let on_bf = t.battlefield(P1, "Grizzly Bears");
        let h = t.hand(P1, "Grizzly Bears");
        let g = t.graveyard(P1, "Grizzly Bears");
        let l = t.library_top(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
        t.answer_choose(P0, &[obj(h), obj(g), obj(l)]);
        if card == "Ancient Vendetta" {
            t.lands(P0, "Swamp", 4);
            let v = t.hand(P0, card);
            t.cast(P0, v).target(pl(P1)).go();
        } else {
            let sb = t.battlefield(P0, card);
            t.lands(P0, "Wastes", 2);
            t.activate(P0, sb, 0, &[pl(P1)]).unwrap();
        }
        t.resolve_all();
        assert!(t.on_battlefield(on_bf), "{card}");
        for c in [h, g, l] {
            assert_eq!(t.zone(c), Zone::Exile, "{card}");
        }
    }
}

#[test]
fn expansion_copies_a_spell_without_targets() {
    cr!("707.10", "115.1");
    ruling!(
        "Expansion // Explosion",
        "Expansion can copy any instant or sorcery spell with an appropriate mana value, not just one with targets."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    let exp = t.hand(P0, "Expansion // Explosion");
    t.cast(P0, exp)
        .method(CastMethod::Half(0))
        .target(div)
        .go();
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![4, 0]);
}
