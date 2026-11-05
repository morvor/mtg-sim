//! Rulings batch P024 — a copy-making spell or ability whose targets are all illegal as it
//! tries to resolve doesn't resolve, and none of its effects happen (CR 608.2b); one with
//! some legal targets does what it can (CR 608.2b, 700.2).

use crate::r_p024_common::*;
use crate::r_s01_common::attack_with;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The YesNo decisions `p` was asked since `from`.
fn yes_no_asked(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

/// P0 casts Giant Growth on `artisan` (Artisan of Forms); its heroic trigger targets
/// `what`. Returns with the trigger on the stack above Giant Growth.
fn heroic_artisan(t: &mut TestGame, artisan: ObjectId, what: ObjectId) {
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    let artisan = t.g.current(artisan);
    t.cast(P0, gg).target(obj(artisan)).go();
    t.answer_targets(P0, &[obj(what)]);
    t.settle();
}

#[test]
fn artisan_of_forms_with_an_illegal_target_remains_what_it_was_a_copy_of() {
    cr!("608.2b", "603.3d");
    ruling!("Artisan of Forms", "If the creature is an illegal target when the ability tries to resolve, the ability won't resolve. Artisan of Forms won't become a copy of that creature. It remains whatever it was a copy of.");
    supported("Artisan of Forms");
    let mut t = TestGame::new(2);
    let artisan = t.battlefield(P0, "Artisan of Forms");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    heroic_artisan(&mut t, artisan, giant);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(name_of(&t, artisan), "Hill Giant");
    // Again, targeting the Bears, which dies in response.
    heroic_artisan(&mut t, artisan, bears);
    kill(&mut t, bears);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, P0, from), 0);
    assert_eq!(name_of(&t, artisan), "Hill Giant");
    assert_eq!(abilities_with(&t, artisan, "Whenever you cast a spell that targets"), 1);
}

#[test]
fn cryptoplasm_with_an_illegal_target_remains_what_it_was() {
    cr!("608.2b", "603.3d");
    ruling!("Cryptoplasm", "If the creature is an illegal target when the ability tries to resolve, it won’t resolve. Cryptoplasm won’t become a copy of that creature; it remains whatever it was before.");
    supported("Cryptoplasm");
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Cryptoplasm");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, plasm), "Hill Giant");
    // Next upkeep it targets the Bears, which dies in response.
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(name_of(&t, plasm), "Hill Giant");
}

#[test]
fn tempt_with_reflections_with_an_illegal_target_makes_no_copies() {
    cr!("608.2b");
    ruling!("Tempt with Reflections", "If the creature you control is an illegal target when Tempt with Reflections tries to resolve, it won’t resolve and none of its effects will happen. No copies will be created.");
    supported("Tempt with Reflections");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        lands_for(&mut t, P0, "{3}{U}");
        let card = t.hand(P0, "Tempt with Reflections");
        t.cast(P0, card).target(obj(giant)).go();
        if !legal {
            kill(&mut t, giant);
        }
        let from = t.asked().len();
        t.answer_yes(P1, true);
        t.resolve_all();
        let giants = t.named_on_battlefield("Hill Giant").len();
        if legal {
            // P0's copy, P1's copy and P0's copy for P1 taking one.
            assert_eq!(giants, 4);
        } else {
            assert_eq!(giants, 0);
            assert_eq!(yes_no_asked(&t, P1, from), 0, "P1 was offered a copy");
        }
    }
}

#[test]
fn three_steps_ahead_resolves_the_modes_with_legal_targets() {
    cr!("608.2b", "702.172a", "700.2");
    ruling!("Three Steps Ahead", "If the first and/or second modes are chosen, then Three Steps Ahead has one or more targets. If so, and all its targets are illegal as it tries to resolve, Three Steps Ahead won’t resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve and modes with legal targets (and the third mode, if chosen) will have their effects.");
    supported("Three Steps Ahead");
    // (spell target countered first, creature target killed) → (countered, token, draws)
    for (spell_gone, creature_gone) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        // P0 casts Grizzly Bears, then (holding priority) all three modes of Three Steps
        // Ahead: counter the Bears, copy the Giant, draw two and discard one.
        lands_for(&mut t, P0, "{1}{G}");
        let bears = t.hand(P0, "Grizzly Bears");
        let spell = t.cast(P0, bears).go();
        lands_for(&mut t, P0, "{U}{1}{U}{3}{2}");
        let tsa = t.hand(P0, "Three Steps Ahead");
        t.g.turn.priority = Some(P0);
        t.cast(P0, tsa).modes(&[0, 1, 2]).target(obj(spell)).target(obj(giant)).go();
        if spell_gone {
            // Countered by a Cancel cast on top of Three Steps Ahead.
            lands_for(&mut t, P0, "{1}{U}{U}");
            let cancel = t.hand(P0, "Cancel");
            t.g.turn.priority = Some(P0);
            t.cast(P0, cancel).target(obj(spell)).go();
        }
        if creature_gone {
            kill(&mut t, giant);
        }
        let hand = t.hand_size(P0);
        let before = t.g.battlefield.clone();
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        let resolves = !(spell_gone && creature_gone);
        assert_eq!(toks.len(), usize::from(!creature_gone), "{spell_gone} {creature_gone}");
        // The Bears never resolve: countered by one spell or the other.
        assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        // Drew two and discarded one only if it resolved.
        let drew = if resolves { 1 } else { 0 };
        assert_eq!(t.hand_size(P0), hand + drew, "{spell_gone} {creature_gone}");
    }
}

#[test]
fn fleeting_reflection_with_an_illegal_second_target_still_untaps_and_gives_hexproof() {
    cr!("608.2b", "608.2c");
    ruling!("Fleeting Reflection", "If the second target creature is an illegal target as Fleeting Reflection tries to resolve, the first target creature will still gain hexproof until end of turn, and you’ll still untap it if it was tapped.");
    supported("Fleeting Reflection");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{1}{U}");
    let spell = t.hand(P0, "Fleeting Reflection");
    t.cast(P0, spell).target(obj(bears)).target(obj(giant)).go();
    kill(&mut t, giant);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(has_kw(&t, bears, KeywordKind::Hexproof));
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
}

#[test]
fn absorb_identity_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!("Absorb Identity", "If the target creature is an illegal target as Absorb Identity tries to resolve, it won't resolve and none of its effects will happen. Shapeshifters you control won't become copies of the creature.");
    supported("Absorb Identity");
    let mut t = TestGame::new(2);
    let sharer = t.battlefield(P0, "Shapesharer");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{1}{U}");
    let spell = t.hand(P0, "Absorb Identity");
    t.cast(P0, spell).target(obj(giant)).go();
    // It's exiled in response: the target is gone.
    let g = t.g.current(giant);
    t.g.exile_object(g, None);
    t.settle();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, P0, from), 0);
    assert_eq!(name_of(&t, sharer), "Shapesharer");
    assert!(!t.in_hand(P1, "Hill Giant"));
}

#[test]
fn kimahri_with_an_illegal_target_gets_no_counter_and_doesnt_copy() {
    cr!("608.2b", "603.3d");
    ruling!("Kimahri, Valiant Guardian", "If the target creature is an illegal target as Kimahri's last ability tries to resolve, it won't resolve and none of its effects will happen. You won't put a counter on Kimahri, and Kimahri won't become a copy of anything.");
    supported("Kimahri, Valiant Guardian");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let kimahri = t.battlefield(P0, "Kimahri, Valiant Guardian");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(giant)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if !legal {
            kill(&mut t, giant);
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        if legal {
            assert_eq!(t.counters(kimahri, counters::PLUS1), 1);
            assert!(t.obj_now(giant).tapped);
            assert!(has_subtype(&t, kimahri, "Giant"));
        } else {
            assert_eq!(t.counters(kimahri, counters::PLUS1), 0);
            assert!(has_subtype(&t, kimahri, "Cat") && !has_subtype(&t, kimahri, "Giant"));
            assert_eq!(t.pt(kimahri), (3, 3));
        }
    }
}

#[test]
fn muddle_with_an_illegal_target_doesnt_get_myriad() {
    cr!("608.2b", "603.3d");
    ruling!("Muddle, the Ever-Changing", "If the target creature is an illegal target as Muddle's ability tries to resolve, it won't resolve and none of its effects will happen. Muddle won't have myriad.");
    supported("Muddle, the Ever-Changing");
    let mut t = TestGame::new(2);
    let muddle = t.battlefield(P0, "Muddle, the Ever-Changing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for(&mut t, P0, "{G}");
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(obj(giant)).go();
    t.answer_targets(P0, &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(name_of(&t, muddle), "Muddle, the Ever-Changing");
    assert!(!has_kw(&t, muddle, KeywordKind::Myriad));
    assert_eq!(t.pt(giant), (6, 6));
}

#[test]
fn niko_with_an_illegal_target_leaves_the_shards_alone() {
    cr!("608.2b");
    ruling!("Niko, Light of Hope", "If the target creature is an illegal target as Niko's last ability tries to resolve, it won't resolve and none of its effects will happen. Shards you control won't become copies of the creature.");
    supported("Niko, Light of Hope");
    let mut t = TestGame::new(2);
    let niko = t.enter(P0, "Niko, Light of Hope");
    t.resolve_all();
    unsick(&mut t, niko);
    let shards = crate::r_s01_common::with_subtype(&t, P0, "Shard");
    assert_eq!(shards.len(), 2, "{}", t.dump_log());
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, niko, "Exile target").expect("activation");
    kill(&mut t, giant);
    t.resolve_all();
    for s in shards {
        assert!(has_subtype(&t, s, "Shard"));
        assert!(!is_creature(&t, s));
    }
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn blue_suns_twilight_with_an_illegal_target_makes_no_token() {
    cr!("608.2b", "107.3a");
    ruling!("Blue Sun's Twilight", "If the target creature is an illegal target by the time the spell tries to resolve, the spell will not resolve. No token is created, even if X is 5 or more.");
    supported("Blue Sun's Twilight");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        lands_for(&mut t, P0, "{6}{U}{U}");
        let card = t.hand(P0, "Blue Sun's Twilight");
        let before = t.g.battlefield.clone();
        t.cast(P0, card).x(6).target(obj(wurm)).go();
        if !legal {
            kill(&mut t, wurm);
        }
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        if legal {
            assert_eq!(t.obj_now(wurm).controller, P0);
            assert_eq!(toks.len(), 1);
            assert_eq!(name_of(&t, toks[0]), "Craw Wurm");
        } else {
            assert!(toks.is_empty());
        }
    }
}

#[test]
fn tilonallis_skinshifter_with_an_illegal_target_remains_a_0_1_attacker() {
    cr!("608.2b", "603.3d", "508.1");
    ruling!("Tilonalli's Skinshifter", "If the target of the ability of Tilonalli’s Skinshifter becomes an illegal target, Tilonalli’s Skinshifter won’t become a copy of anything. It will remain a 0/1 attacking creature.");
    supported("Tilonalli's Skinshifter");
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, "Tilonalli's Skinshifter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(skin, Entity::Player(P1)), (bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(name_of(&t, skin), "Tilonalli's Skinshifter");
    assert_eq!(t.pt(skin), (0, 1));
    assert!(t.g.combat.as_ref().unwrap().attackers.iter().any(|a| a.id == skin));
}

#[test]
fn silent_hallcreeper_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "700.2");
    ruling!("Silent Hallcreeper", "If the third mode is chosen for Silent Hallcreeper's last ability but the target creature is an illegal target as the ability tries to resolve, it won't resolve and none of its effects will happen.");
    supported("Silent Hallcreeper");
    let mut t = TestGame::new(2);
    let creeper = t.battlefield(P0, "Silent Hallcreeper");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &[(creeper, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(name_of(&t, creeper), "Silent Hallcreeper");
    assert_eq!(t.pt(creeper), (1, 1));
}
