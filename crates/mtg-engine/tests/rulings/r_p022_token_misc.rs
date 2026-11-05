//! Rulings batch P022 — more effects that copy creatures: melded cards (CR 712.4, 701.42),
//! copied permanent spells (CR 707.10, 608.3f), costs and additional costs of a copy (CR
//! 707.10, 702.33d), copied "enters with" abilities that count mana spent (CR 601.2h,
//! 707.2), several copy effects on one entering permanent (CR 616.1), targets that become
//! illegal (CR 608.2b) and attack restrictions on tokens that must attack (CR 508.1c,
//! 508.1d).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn mimic_vat_exiles_both_halves_of_a_melded_creature_and_copies_one() {
    cr!("712.21", "712.21c", "607.2a");
    ruling!("Mimic Vat", "If a melded permanent dies and triggers Mimic Vat's triggered ability, both of its component cards are exiled. As Mimic Vat's second ability resolves, its controller chooses one of those cards to make a token copy of.");
    supported("Mimic Vat");
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    let fang = t.battlefield(P0, "Fang, Fearless l'Cie");
    let vanille = t.battlefield(P0, "Vanille, Cheerful l'Cie");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let ragnarok = t.named_on_battlefield("Ragnarok, Divine Deliverance");
    assert_eq!(ragnarok.len(), 1);
    t.answer_yes(P0, true);
    kill(&mut t, ragnarok[0]);
    t.resolve_all();
    let fang = t.g.current(fang);
    let vanille = t.g.current(vanille);
    assert_eq!(t.zone(fang), Zone::Exile);
    assert_eq!(t.zone(vanille), Zone::Exile);
    // P0 chooses Vanille to copy.
    t.lands(P0, "Wastes", 3);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(vanille)]);
    let toks = activate_for_tokens(&mut t, P0, vat, "Create a token", &[]);
    let offered = crate::r_p017_common::choice_candidates(&t, from);
    assert!(offered
        .iter()
        .any(|c| c.contains(&obj(fang)) && c.contains(&obj(vanille))));
    assert_eq!(name_of(&t, one(toks)), "Vanille, Cheerful l'Cie");
}

#[test]
fn case_of_the_shifting_visage_copy_of_a_permanent_spell_becomes_a_token() {
    cr!("707.10", "608.3f", "111.1");
    ruling!("Case of the Shifting Visage", "If a permanent spell is copied, it’s put onto the battlefield as a token as the spell resolves rather than putting the copy of the spell onto the battlefield. The rules that apply to a permanent spell becoming a permanent apply to a copy of a spell becoming a token.");
    supported("Case of the Shifting Visage");
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Shifting Visage");
    assert!(mtg_engine::cases::solve(&mut t.g, case));
    t.g.recompute();
    // P1's Soul Warden sees both enter.
    t.battlefield(P1, "Soul Warden");
    lands_for(&mut t, P0, "{1}{G}");
    let bears = t.hand(P0, "Grizzly Bears");
    let before = t.g.battlefield.clone();
    t.cast(P0, bears).go();
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(name_of(&t, tok), "Grizzly Bears");
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert_eq!(t.life(P1), 22);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_copy_of_gyrus_counts_the_mana_spent_to_cast_it() {
    cr!("707.2", "601.2h", "614.1c");
    ruling!("Gyrus, Waker of Corpses", "If another creature enters the battlefield as a copy of Gyrus, consider how much mana was spent to cast that creature to determine how many +1/+1 counters it enters with.");
    supported("Gyrus, Waker of Corpses");
    let mut t = TestGame::new(2);
    let gyrus = t.battlefield(P1, "Gyrus, Waker of Corpses");
    plus1(&mut t, gyrus, 5);
    // P0 casts Clone ({3}{U}): four mana spent.
    lands_for(&mut t, P0, "{3}{U}");
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[obj(gyrus)]);
    t.resolve_all();
    let c = t.g.current(clone);
    assert_eq!(name_of(&t, c), "Gyrus, Waker of Corpses");
    assert_eq!(t.counters(c, counters::PLUS1), 4);
    // Put onto the battlefield without being cast: none (it dies as a 0/0).
    let mut t = TestGame::new(2);
    let gyrus = t.battlefield(P1, "Gyrus, Waker of Corpses");
    plus1(&mut t, gyrus, 5);
    let c = enter_copying(&mut t, P0, "Clone", gyrus);
    assert!(!t.on_battlefield(c));
}

#[test]
fn demonstrate_copies_are_kicked_like_the_original() {
    cr!("707.10", "702.33d", "702.144a");
    ruling!("Silverquill Lecturer", "If any additional or alternative costs were paid to cast the spell with demonstrate, the copies will see those choices as well. For example, if the spell has a kicker ability and was kicked, the copies will also be kicked");
    supported("Silverquill Lecturer");
    supported("Aether Figment");
    // Aether Figment: kicker {3}; "If this creature was kicked, it enters with two +1/+1
    // counters on it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill Lecturer");
    t.lands(P0, "Island", 5);
    let figment = t.hand(P0, "Aether Figment");
    t.answer_yes(P0, true);
    t.cast(P0, figment).kicked(true).go();
    t.resolve_all();
    let figments = t.named_on_battlefield("Aether Figment");
    assert_eq!(figments.len(), 3);
    for f in figments {
        assert_eq!(t.counters(f, counters::PLUS1), 2);
    }
}

#[test]
fn elvish_hydromancers_token_of_itself_isnt_kicked() {
    cr!("707.2", "702.33d", "603.4");
    ruling!("Elvish Hydromancer", "If it was kicked, Elvish Hydromancer's ability can target any creature you control, including itself. If it targets itself, the token that is created won't have been kicked, so the ability will not trigger again.");
    supported("Elvish Hydromancer");
    let mut t = TestGame::new(2);
    lands_for(&mut t, P0, "{2}{G}{3}{U}");
    let h = t.hand(P0, "Elvish Hydromancer");
    t.cast(P0, h).kicked(true).go();
    t.resolve();
    let h = t.g.current(h);
    assert!(t.on_battlefield(h));
    t.answer_targets(P0, &[obj(h)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Elvish Hydromancer").len(), 2);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn several_mystic_reflections_apply_in_the_order_the_controller_chooses() {
    cr!("616.1", "707.2", "614.1c");
    ruling!("Mystic Reflection", "If more than one Mystic Reflection applies to the same permanent entering the battlefield, the controller of that permanent chooses the order of those copy effects. The copy effect they apply last will determine which creature the entering permanent copies.");
    supported("Mystic Reflection");
    let mut names = Vec::new();
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let angel = t.battlefield(P0, "Serra Angel");
        for what in [giant, angel] {
            lands_for(&mut t, P0, "{1}{U}");
            let mr = t.hand(P0, "Mystic Reflection");
            t.cast(P0, mr).target(obj(what)).go();
            t.resolve_all();
        }
        // P1's Grizzly Bears enters; P1 chooses which copy effect applies first.
        t.answer(P1, DecisionKind::Replacement, Answer::Index(first));
        let from = t.asked().len();
        let b = t.enter(P1, "Grizzly Bears");
        t.settle();
        assert!(t.asked()[from..].iter().any(|(p, d)| *p == P1
            && matches!(d, mtg_engine::decision::Decision::ChooseReplacement { .. })));
        names.push(name_of(&t, b));
    }
    names.sort();
    assert_eq!(
        names,
        vec!["Hill Giant".to_string(), "Serra Angel".to_string()]
    );
}

#[test]
fn hate_mirage_with_one_illegal_target_makes_one_token() {
    cr!("608.2b");
    ruling!("Hate Mirage", "If one of the two target creatures become illegal, you create one token that's a copy of the one that's still a legal target. You don't create two tokens.");
    supported("Hate Mirage");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    lands_for(&mut t, P0, "{3}{R}");
    let hm = t.hand(P0, "Hate Mirage");
    let before = t.g.battlefield.clone();
    t.cast(P0, hm).target(obj(giant)).target(obj(angel)).go();
    kill(&mut t, angel);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(name_of(&t, tok), "Hill Giant");
}

#[test]
fn wire_surgeons_encore_token_stopped_from_attacking_its_player() {
    cr!("508.1c", "508.1d", "702.141a");
    ruling!("Wire Surgeons", "If an effect stops a token from attacking a specific player, that token can attack any player or planeswalker, or not attack at all. If the effect stops the token from attacking a specific player unless a cost is paid, you don’t have to pay that cost unless you want to attack that player.");
    supported("Wire Surgeons");
    supported("Blazing Archon");
    // Three players; P1 controls Blazing Archon ("Creatures can't attack you."). Encore
    // Ornithopter ({0}): a token for P1 and one for P2.
    for attack_p2 in [false, true] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Wire Surgeons");
        t.battlefield(P1, "Blazing Archon");
        let card = t.graveyard(P0, "Ornithopter");
        let before = t.g.battlefield.clone();
        activate_containing(&mut t, P0, card, "Encore").expect("encore");
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), 2);
        // Which token must attack P1: the one that isn't required to attack P2 is the
        // other; P0 declares only one token attacking P2 (or none at all).
        t.set_step(P0, Step::BeginningOfCombat);
        let decl = if attack_p2 {
            vec![(toks[0], Entity::Player(P2)), (toks[1], Entity::Player(P2))]
        } else {
            vec![]
        };
        t.answer(P0, DecisionKind::Attackers, Answer::Attackers(decl));
        t.advance_to(P0, Step::DeclareAttackers);
        let attacking: Vec<bool> = toks.iter().map(|x| t.g.is_attacking(*x)).collect();
        if attack_p2 {
            assert_eq!(attacking, vec![true, true]);
        } else {
            // The token for P2 must attack P2; the one for P1 needn't attack at all.
            assert_eq!(attacking.iter().filter(|a| **a).count(), 1);
        }
        assert_eq!(t.life(P1), 20);
    }
}
