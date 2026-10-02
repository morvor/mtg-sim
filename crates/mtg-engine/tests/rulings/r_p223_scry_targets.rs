//! Rulings batch P223 — scry on targeted spells and abilities (CR 608.2b, 701.22a): a
//! spell or ability whose targets are all illegal doesn't resolve, so you don't scry; one
//! whose target is legal but unaffected (or that has no targets) still scries.

use crate::r_p223_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::{in_hand_with_mana, to_blockers};
use crate::r_s04_common::add_mana;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// What a single-target spell targets in [`single_target_spells_with_an_illegal_target_dont_scry`].
#[derive(Clone, Copy)]
enum Tgt {
    /// A Grizzly Bears controlled by the given player.
    Bears(PlayerId),
    /// P1's Hill Giant (power 3).
    Giant,
    /// P1's Forest.
    Land,
    /// P1's tapped Grizzly Bears, made illegal by untapping it.
    TappedBears,
    /// P0's attacking Grizzly Bears.
    Attacker,
}

fn rev(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, .. } if name == mtg_engine::reveal::REVEALED))
        .count()
}

/// Casts `name` (P0) at a target of kind `tgt`; if `illegal`, the target becomes illegal
/// before the spell resolves. Returns (scries asked, the target).
fn cast_at(name: &str, tgt: Tgt, illegal: bool) -> (TestGame, usize, ObjectId) {
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.library_top(P0, "Hill Giant");
    }
    let target = match tgt {
        Tgt::Bears(p) => t.battlefield(p, "Grizzly Bears"),
        Tgt::Giant => t.battlefield(P1, "Hill Giant"),
        Tgt::Land => t.battlefield(P1, "Forest"),
        Tgt::TappedBears => {
            let b = t.battlefield(P1, "Grizzly Bears");
            t.g.tap(b);
            b
        }
        Tgt::Attacker => {
            let b = t.battlefield(P0, "Grizzly Bears");
            to_blockers(&mut t, &[(b, Entity::Player(P1))], &[]);
            b
        }
    };
    let spell = in_hand_with_mana(&mut t, P0, name);
    // Gods Willing: protection from green (the Bears stay legal).
    let green = Color::ALL.iter().position(|c| *c == Color::Green).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(green));
    let from = t.asked().len();
    t.cast(P0, spell).target(target).go();
    if illegal {
        match tgt {
            Tgt::TappedBears => {
                t.g.untap(target);
            }
            _ => destroy(&mut t, target),
        }
    }
    t.resolve_all();
    let n = scries_since(&t, from).len();
    (t, n, target)
}

#[test]
fn single_target_spells_with_an_illegal_target_dont_scry() {
    cr!("608.2b", "701.22a");
    ruling!("Cruel Finality", "If the target creature becomes an illegal target, Cruel Finality doesn’t resolve and none of its effects happen. You won’t scry 1.");
    ruling!("Select for Inspection", "If the target creature becomes untapped or otherwise becomes an illegal target before Select for Inspection resolves, the spell doesn't resolve. The creature remains on the battlefield and you won't scry 1.");
    ruling!("Chain to Memory", "If the target creature is an illegal target by the time Chain to Memory tries to resolve, the spell won’t resolve. You won’t scry 2.");
    ruling!("Gods Willing", "If the target creature is an illegal target by the time Gods Willing tries to resolve, the spell doesn't resolve. You won't scry 1.");
    ruling!("Hithlain Knots", "If the target creature is an illegal target by the time Hithlain Knots tries to resolve, the spell doesn't resolve. You won't scry 1, and you won't draw a card.");
    ruling!("Inordinate Rage", "If the target creature is an illegal target by the time Inordinate Rage tries to resolve, the spell doesn’t resolve. You don’t scry 1.");
    ruling!("Jaya's Greeting", "If the target creature is an illegal target by the time Jaya’s Greeting tries to resolve, the spell doesn’t resolve. You won’t scry 1.");
    ruling!("Judge Unworthy", "If the target creature is an illegal target by the time Judge Unworthy tries to resolve, the spell doesn't resolve. You don't scry 3 or reveal any card.");
    ruling!("Portent of Betrayal", "If the target creature is an illegal target by the time Portent of Betrayal tries to resolve, the spell won't resolve. You won't scry 1.");
    ruling!("Samut's Sprint", "If the target creature is an illegal target by the time Samut’s Sprint tries to resolve, the spell doesn’t resolve. You won’t scry 1.");
    ruling!("Skywhaler's Shot", "If the target creature is an illegal target by the time Skywhaler's Shot tries to resolve, the spell doesn't resolve. You don't scry 1.");
    ruling!("Storm Strike", "If the target creature is an illegal target by the time Storm Strike tries to resolve, the spell doesn’t resolve. You won’t scry 1.");
    ruling!("Boon of Safety", "If the target is illegal as Boon of Safety attempts to resolve, it will be removed from the stack and its controller will not scry.");
    ruling!("Jaya's Firenado", "If the target is illegal as Jaya's Firenado tries to resolve, you won't get to scry.");
    ruling!("Freeze in Place", "If the target is not legal as Freeze in Place tries to resolve, Freeze in Place will be removed from the stack. You won't scry.");
    ruling!("Daring Escape", "If the target is not legal as the spell would resolve, its controller does not get to scry.");
    ruling!("Rubble Reading", "If the target land is an illegal target by the time Rubble Reading tries to resolve, the spell doesn’t resolve. You don’t scry 2.");
    let cases: &[(&str, Tgt)] = &[
        ("Cruel Finality", Tgt::Bears(P1)),
        ("Select for Inspection", Tgt::TappedBears),
        ("Chain to Memory", Tgt::Bears(P1)),
        ("Gods Willing", Tgt::Bears(P0)),
        ("Hithlain Knots", Tgt::Bears(P1)),
        ("Inordinate Rage", Tgt::Bears(P0)),
        ("Jaya's Greeting", Tgt::Bears(P1)),
        ("Judge Unworthy", Tgt::Attacker),
        ("Portent of Betrayal", Tgt::Bears(P1)),
        ("Samut's Sprint", Tgt::Bears(P0)),
        ("Skywhaler's Shot", Tgt::Giant),
        ("Storm Strike", Tgt::Bears(P0)),
        ("Boon of Safety", Tgt::Bears(P0)),
        ("Jaya's Firenado", Tgt::Bears(P1)),
        ("Freeze in Place", Tgt::Bears(P1)),
        ("Daring Escape", Tgt::Bears(P0)),
        ("Rubble Reading", Tgt::Land),
    ];
    for (name, tgt) in cases {
        supported(name);
        // With the target still legal, the spell scries.
        let (_, n, _) = cast_at(name, *tgt, false);
        assert_eq!(n, 1, "{name} resolves and scries");
        // With the target illegal, it's removed from the stack and doesn't scry.
        let (t, n, target) = cast_at(name, *tgt, true);
        assert_eq!(n, 0, "{name} doesn't scry");
        assert!(t.in_graveyard(P0, name), "{name} left the stack");
        match *name {
            "Hithlain Knots" => assert_eq!(t.hand_size(P0), 0, "no card drawn"),
            "Judge Unworthy" => assert_eq!(rev(&t), 0, "no card revealed"),
            "Select for Inspection" => {
                assert!(t.on_battlefield(target), "the creature remains");
                assert!(!t.obj_now(target).tapped);
            }
            _ => {}
        }
    }
}

#[test]
fn samuts_loyalty_ability_with_an_illegal_target_doesnt_scry() {
    cr!("608.2b", "701.22a", "606.3");
    ruling!("Samut, Tyrant Smasher", "If the target creature is an illegal target by the time Samut’s loyalty ability tries to resolve, the ability doesn’t resolve. You won’t scry 1.");
    supported("Samut, Tyrant Smasher");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let samut = t.battlefield(P0, "Samut, Tyrant Smasher");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        t.activate(P0, samut, 0, &[Entity::Object(bears)])
            .expect("Samut's -1");
        if illegal {
            destroy(&mut t, bears);
        } else {
            t.resolve_all();
            assert_eq!(t.pt(bears), (4, 3));
        }
        t.resolve_all();
        assert_eq!(scries_since(&t, from).len(), usize::from(!illegal));
    }
}

#[test]
fn a_foretold_poison_the_cup_with_an_illegal_target_doesnt_scry() {
    cr!("608.2b", "702.143c");
    ruling!("Poison the Cup", "If the target creature is an illegal target as Poison the Cup tries to resolve, it won’t resolve and none of its effects will happen. You won’t scry 2 if Poison the Cup was foretold.");
    supported("Poison the Cup");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let cup = t.hand(P0, "Poison the Cup");
        add_mana(&mut t, P0, ManaType::C, 2);
        t.g.turn.priority = Some(P0);
        t.g.take_action(P0, Action::Special(SpecialAction::Foretell { card: cup }));
        t.g.flush_events();
        let foretold = t.g.current(cup);
        assert_eq!(t.zone(foretold), Zone::Exile);
        t.advance_to(P1, Step::Upkeep);
        add_mana(&mut t, P0, ManaType::B, 1);
        add_mana(&mut t, P0, ManaType::C, 1);
        let from = t.asked().len();
        t.cast(P0, foretold)
            .method(CastMethod::Keyword(KeywordKind::Foretell))
            .target(bears)
            .go();
        if illegal {
            // In response, the Bears die some other way.
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        assert_eq!(scry_sizes(&t, P0, from), if illegal { vec![] } else { vec![2] });
    }
}

#[test]
fn gods_willing_scries_even_if_its_target_becomes_illegal_while_it_resolves() {
    cr!("608.2b", "608.2c", "702.16b");
    ruling!("Gods Willing", "If the target creature becomes an illegal target while Gods Willing is resolving (most likely because you gave it protection from white), you do scry 1.");
    supported("Gods Willing");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gw = in_hand_with_mana(&mut t, P0, "Gods Willing");
    let white = Color::ALL.iter().position(|c| *c == Color::White).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(white));
    let from = t.asked().len();
    t.cast(P0, gw).target(bears).go();
    t.resolve_all();
    // The Bears have protection from white (so Gods Willing couldn't target them now)...
    let gw2 = in_hand_with_mana(&mut t, P0, "Gods Willing");
    assert!(t.cast(P0, gw2).target(bears).try_go().is_err());
    // ...but it scried.
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn a_legal_target_that_doesnt_become_tapped_still_lets_you_scry_and_draw() {
    cr!("608.2b", "608.2c", "701.26a");
    ruling!("Hithlain Knots", "However, if the target is legal but doesn't become tapped (most likely because it's already tapped), you do scry 1, and you do draw a card.");
    ruling!("Plunge into Winter", "If the target is already tapped at the time Plunge into Winter resolves, you'll still scry 1 and draw a card.");
    for name in ["Hithlain Knots", "Plunge into Winter"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let spell = in_hand_with_mana(&mut t, P0, name);
        t.cast(P0, spell).target(bears).go();
        // It becomes tapped in response.
        t.g.tap(bears);
        let from = t.asked().len();
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert!(t.obj_now(bears).tapped);
        assert_eq!(scry_sizes(&t, P0, from), vec![1], "{name}");
        assert_eq!(t.hand_size(P0), hand + 1, "{name} draws");
    }
}

#[test]
fn rubble_reading_scries_even_if_the_land_isnt_destroyed() {
    cr!("608.2b", "702.12b");
    ruling!("Rubble Reading", "If the target is legal but not destroyed (most likely because it has indestructible), you do scry 2.");
    supported("Rubble Reading");
    let mut t = TestGame::new(2);
    // Darksteel Citadel: an indestructible artifact land.
    let citadel = t.battlefield(P1, "Darksteel Citadel");
    let rr = in_hand_with_mana(&mut t, P0, "Rubble Reading");
    let from = t.asked().len();
    t.cast(P0, rr).target(citadel).go();
    t.resolve_all();
    assert!(t.on_battlefield(citadel));
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
}

/// Casts `name` (P0) with the given targets (all P1's Grizzly Bears), making all of them
/// illegal if `illegal`; returns the scry sizes.
fn up_to_n(name: &str, n_targets: usize, x: Option<i64>, illegal: bool) -> Vec<usize> {
    let mut t = TestGame::new(2);
    let bears: Vec<ObjectId> = (0..n_targets)
        .map(|_| t.battlefield(P1, "Grizzly Bears"))
        .collect();
    let spell = in_hand_with_mana(&mut t, P0, name);
    if let Some(x) = x {
        t.lands(P0, "Wastes", x as usize);
    }
    let from = t.asked().len();
    let es: Vec<Entity> = bears.iter().map(|b| Entity::Object(*b)).collect();
    let mut c = t.cast(P0, spell).targets(&es);
    if let Some(x) = x {
        c = c.x(x);
    }
    c.try_go()
        .unwrap_or_else(|e| panic!("{name} with {n_targets} targets: {e:?}"));
    if illegal {
        for b in &bears {
            destroy(&mut t, *b);
        }
    }
    t.resolve_all();
    assert!(t.in_graveyard(P0, name));
    scry_sizes(&t, P0, from)
}

#[test]
fn up_to_n_targets_zero_targets_still_scries_but_all_illegal_targets_dont() {
    cr!("608.2b", "601.2c", "115.1a");
    ruling!("Sudden Storm", "If you choose zero targets, you’ll just scry 1 when the ability resolves. However, if you choose at least one target and all of Sudden Storm’s targets are illegal as it tries to resolve, the spell won’t resolve and none of its effects will happen. You won’t scry in that case.");
    ruling!("Glimpse the Sun God", "If you choose zero targets, you’ll just scry 1 when the spell resolves. However, if you choose at least one target and all of Glimpse the Sun God’s targets are illegal as it tries to resolve, the spell won’t resolve and none of its effects will happen. You won’t scry in that case.");
    ruling!("Sea God's Revenge", "You can cast Sea God's Revenge with no targets. If you do, you'll scry 1 when it resolves. If you choose at least one target, and all chosen targets are illegal when Sea God's Revenge tries to resolve, it won't resolve and none of its effects will happen.");
    // Sudden Storm: "Tap up to two target creatures."; Sea God's Revenge: "Return up to
    // three target creatures your opponents control"; Glimpse the Sun God: "Tap X target
    // creatures."
    for (name, n, x) in [
        ("Sudden Storm", 2, None),
        ("Sea God's Revenge", 3, None),
        ("Glimpse the Sun God", 2, Some(2)),
    ] {
        supported(name);
        let zero_x = x.map(|_| 0);
        assert_eq!(up_to_n(name, 0, zero_x, false), vec![1], "{name}: no targets");
        assert_eq!(up_to_n(name, n, x, false), vec![1], "{name}: legal targets");
        assert_eq!(up_to_n(name, n, x, true), Vec::<usize>::new(), "{name}: all illegal");
        // One of two targets illegal: it resolves.
        if n >= 2 {
            let mut t = TestGame::new(2);
            let a = t.battlefield(P1, "Grizzly Bears");
            let b = t.battlefield(P1, "Grizzly Bears");
            let spell = in_hand_with_mana(&mut t, P0, name);
            if let Some(x) = x {
                t.lands(P0, "Wastes", x as usize);
            }
            let from = t.asked().len();
            let mut c = t
                .cast(P0, spell)
                .targets(&[Entity::Object(a), Entity::Object(b)]);
            if let Some(x) = x {
                c = c.x(x);
            }
            c.go();
            destroy(&mut t, a);
            t.resolve_all();
            assert_eq!(scry_sizes(&t, P0, from), vec![1], "{name}: one legal");
        }
    }
}

#[test]
fn nobodys_optional_target_zero_targets_scries_an_illegal_target_doesnt() {
    cr!("608.2b", "603.3d");
    ruling!("Nobody", "You don't have to choose a target for Nobody's ability. However, if you do and the target is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't scry.");
    supported("Nobody");
    // "When this creature enters, return up to one other target artifact you control to
    // its owner's hand. Scry 1."
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    t.answer_targets(P0, &[]);
    t.enter(P0, "Nobody");
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let relic = t.battlefield(P0, "Ornithopter");
        let from = t.asked().len();
        t.answer_targets(P0, &[Entity::Object(relic)]);
        t.enter(P0, "Nobody");
        t.settle();
        if illegal {
            destroy(&mut t, relic);
        }
        t.resolve_all();
        assert_eq!(scries_since(&t, from).len(), usize::from(!illegal));
        if !illegal {
            assert!(t.in_hand(P0, "Ornithopter"));
        }
    }
}

#[test]
fn prison_realms_scry_is_a_separate_ability() {
    cr!("603.2", "608.2b");
    ruling!("Prison Realm", "You’ll still scry 1 even if Prison Realm’s first ability has no legal target or if that target becomes an illegal target before the ability resolves.");
    supported("Prison Realm");
    // No legal target.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    t.enter(P0, "Prison Realm");
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
    // The target becomes illegal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Prison Realm");
    t.settle();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn alibous_trigger_with_an_illegal_target_doesnt_scry() {
    cr!("608.2b", "603.3d");
    ruling!("Alibou, Ancient Witness", "You choose a target as Alibou's triggered ability is put on the stack. If that target isn't legal as the triggered ability tries to resolve, it doesn't resolve and you don't scry.");
    supported("Alibou, Ancient Witness");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let alibou = t.battlefield(P0, "Alibou, Ancient Witness");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        let from = t.asked().len();
        attack_until_trigger(&mut t, alibou);
        if illegal {
            assert_eq!(t.stack_len(), 1);
            destroy(&mut t, bears);
            t.resolve_all();
            assert_eq!(scries_since(&t, from).len(), 0);
        } else {
            t.resolve_all();
            // Alibou itself is a tapped artifact: X = 1.
            assert!(!t.on_battlefield(bears) || t.obj_now(bears).damage == 1);
            assert_eq!(scry_sizes(&t, P0, from), vec![1]);
        }
    }
}

/// P0 attacks with `attacker`; stops in the declare attackers step with the attack
/// trigger on the stack.
fn attack_until_trigger(t: &mut TestGame, attacker: ObjectId) {
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
    );
    let ok = t
        .g
        .run_until(1_000, |g| g.turn.step == Step::DeclareAttackers && !g.stack.is_empty());
    assert!(ok);
}
