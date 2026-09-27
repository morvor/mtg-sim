//! CR 702.169 Solved.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::cases;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn solved_is_followed_by_ability_text_on_case_cards() {
    cr!("702.169", "702.169a");
    ruling!(
        "Case of the Stashed Skeleton",
        "Each Case has two special keyword abilities: to solve and solved."
    );
    // Case of the Filched Falcon: "Solved — {2}{U}, Sacrifice this Case: Put four +1/+1
    // counters on target noncreature artifact. It becomes a 0/0 Bird creature with flying
    // in addition to its other types."
    assert_supported("Case of the Filched Falcon");
    let def = card("Case of the Filched Falcon");
    let solved = def
        .front()
        .chars
        .abilities
        .iter()
        .find(|a| a.text.starts_with("Solved — "))
        .expect("solved ability");
    // The ability text after "Solved —" is an ability the Case has only while solved.
    let AbilityKind::Static(s) = &solved.kind else {
        panic!("not a static ability");
    };
    assert!(matches!(&s.condition, Some(Condition::Custom(n)) if n == cases::SOLVED));
    let StaticEffect::Continuous { mods, .. } = &s.effect else {
        panic!("not a continuous effect");
    };
    assert!(matches!(
        mods.as_slice(),
        [Modification::AddAbility(a)] if matches!(a.kind, AbilityKind::Activated(_))
    ));
}

#[test]
fn a_solved_static_ability_applies_as_long_as_the_case_is_solved() {
    cr!("702.169b");
    ruling!(
        "Case of the Gateway Express",
        "\"Solved — [static ability]\" means \"As long as this Case is solved, [static ability].\""
    );
    // Case of the Gateway Express: "Solved — Creatures you control get +1/+0."
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Gateway Express");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    cases::solve(&mut t.g, case);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn a_solved_triggered_ability_triggers_only_if_the_case_is_solved() {
    cr!("702.169c");
    ruling!(
        "Case of the Ransacked Lab",
        "\"Solved — [Triggered ability]\" means \"[Triggered ability]. This ability triggers only if this Case is solved.\""
    );
    ruling!(
        "Case of the Ransacked Lab",
        "Cases don't lose their other abilities when they become solved."
    );
    // Case of the Ransacked Lab: "Instant and sorcery spells you cast cost {1} less to
    // cast." "Solved — Whenever you cast an instant or sorcery spell, draw a card."
    for solved in [false, true] {
        let mut t = TestGame::new(2);
        let case = t.battlefield(P0, "Case of the Ransacked Lab");
        if solved {
            cases::solve(&mut t.g, case);
        }
        // Divination ({2}{U}, "Draw two cards.") costs {1}{U}.
        let spell = t.hand(P0, "Divination");
        add_mana(&mut t, P0, ManaType::U, 2);
        let hand = t.hand_size(P0);
        t.cast(P0, spell).go();
        assert_eq!(pool(&t, P0), 0);
        t.settle();
        let triggered = t.stack_len() - 1;
        assert_eq!(triggered, solved as usize);
        t.resolve_all();
        // Divination draws two cards; the solved ability draws another.
        assert_eq!(t.hand_size(P0), hand - 1 + 2 + solved as usize);
    }
}

#[test]
fn a_solved_activated_ability_can_be_activated_only_if_the_case_is_solved() {
    cr!("702.169d");
    ruling!(
        "Case of the Stashed Skeleton",
        "\"Solved — [activated ability]\" means \"[Activated ability]. Activate only if this Case is solved.\""
    );
    // Case of the Filched Falcon: "To solve — You control three or more artifacts."
    // "Solved — {2}{U}, Sacrifice this Case: Put four +1/+1 counters on target noncreature
    // artifact. It becomes a 0/0 Bird creature with flying in addition to its other
    // types."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PostcombatMain);
    let case = t.battlefield(P0, "Case of the Filched Falcon");
    let a = t.battlefield(P0, "Chromatic Sphere");
    t.battlefield(P0, "Chromatic Sphere");
    add_mana(&mut t, P0, ManaType::U, 3);
    let solved_uid = |t: &mut TestGame| {
        t.g.recompute();
        t.obj(case)
            .chars
            .abilities
            .iter()
            .find(|x| matches!(x.kind, AbilityKind::Activated(_)) && x.text.starts_with("{2}{U}"))
            .map(|x| x.uid)
    };
    // Unsolved: it has no such ability to activate.
    assert!(solved_uid(&mut t).is_none());
    // Two artifacts at the end step: not solved.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!cases::is_solved(&t.g, case));
    // Three artifacts at the next end step: solved.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PostcombatMain);
    t.battlefield(P0, "Chromatic Sphere");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(cases::is_solved(&t.g, case));
    let uid = solved_uid(&mut t).expect("solved ability");
    add_mana(&mut t, P0, ManaType::U, 3);
    t.answer_targets(P0, &[Entity::Object(a)]);
    activate_uid(&mut t, P0, case, uid).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Case of the Filched Falcon"));
    assert!(t.obj(a).chars.is(CardType::Creature));
    assert_eq!(t.pt(a), (4, 4));
}
