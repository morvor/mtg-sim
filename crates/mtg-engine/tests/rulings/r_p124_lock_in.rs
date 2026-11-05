//! Rulings batch P124 — spells and abilities that give "creatures you control" an ability
//! or bonus until a time: the affected set is locked in as they resolve (CR 611.2c), so
//! creatures that enter, come under your control or start matching later aren't affected,
//! and the ones affected keep it whatever happens later (CR 611.2a).

use crate::r_p124_common::*;
use crate::r_s06_common::give_control;
use mtg_engine::ability::{AbilityKind, Modification};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

/// Whether the creature has the effect being looked for.
type Has = fn(&mut TestGame, ObjectId) -> bool;

fn activated_count(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

fn make_white(t: &mut TestGame, id: ObjectId) {
    modify_until_eot(
        t,
        id,
        vec![Modification::SetColors(ColorSet::single(Color::White))],
    );
}

#[test]
fn only_creatures_you_control_as_it_resolves_are_affected() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Break of Day",
        "Because Break of Day affects only creatures you control when it resolves. It won’t affect creatures that come under your control later in the turn."
    );
    ruling!(
        "Emeria's Call // Emeria, Shattered Skyclave",
        "Emeria's Call affects only creatures you control at the time it resolves. Creatures you begin to control later won't gain indestructible."
    );
    ruling!(
        "Dauntless Escort",
        "Once Dauntless Escort's ability resolves, the set of creatures it affects is locked in. It won't affect creatures that enter the battlefield after the ability has finished resolving."
    );
    ruling!(
        "Spare from Evil",
        "Only creatures you control when Spare from Evil resolves will be affected. Creatures that enter or that you gain control of later in the turn won't be."
    );
    ruling!(
        "Resuscitate",
        "Only grants the ability to creatures you control when this spell resolves. Ones entering later in the turn do not get it."
    );
    ruling!(
        "Brave the Elements",
        "Only white creatures you control at the time Brave the Elements resolves gain the protection ability. Creatures that come under your control later in the turn, or that turn white later in the turn, won’t have it."
    );
    // (card, the creature's state before, the effect looked for, how it's applied).
    type Apply = fn(&mut TestGame);
    let cases: [(&str, Has, Apply); 6] = [
        (
            "Break of Day",
            |t, id| {
                // (All the creatures here are 2/2s.)
                t.g.recompute();
                t.pt(t.g.current(id)).0 > 2
            },
            |t| cast_resolve(t, P0, "Break of Day", &[]),
        ),
        (
            "Emeria's Call // Emeria, Shattered Skyclave",
            |t, id| indestructible(t, id),
            |t| cast_resolve(t, P0, "Emeria's Call // Emeria, Shattered Skyclave", &[]),
        ),
        (
            "Dauntless Escort",
            |t, id| indestructible(t, id),
            |t| {
                let escort = t.battlefield(P0, "Dauntless Escort");
                t.activate(P0, escort, 0, &[]).unwrap();
                t.resolve_all();
            },
        ),
        (
            "Spare from Evil",
            |t, id| has_kw(t, id, KeywordKind::Protection),
            |t| cast_resolve(t, P0, "Spare from Evil", &[]),
        ),
        (
            "Resuscitate",
            |t, id| activated_count(t, id) > 0,
            |t| cast_resolve(t, P0, "Resuscitate", &[]),
        ),
        (
            "Brave the Elements",
            |t, id| has_kw(t, id, KeywordKind::Protection),
            |t| {
                choose_color(t, P0, Color::Red);
                cast_resolve(t, P0, "Brave the Elements", &[]);
            },
        ),
    ];
    for (card, has, apply) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        // White creatures, so Brave the Elements applies to the early one.
        let early = t.battlefield(P0, "Benalish Knight");
        let stolen = t.battlefield(P1, "Benalish Knight");
        apply(&mut t);
        assert!(has(&mut t, early), "{card}: the creature it locked in");
        let late = t.battlefield(P0, "Benalish Knight");
        give_control(&mut t, stolen, P0);
        assert!(!has(&mut t, late), "{card}: a creature that entered later");
        assert!(!has(&mut t, stolen), "{card}: a creature gained later");
        if card == "Brave the Elements" {
            // A creature that turns white later doesn't get it either.
            let bears = t.battlefield(P0, "Grizzly Bears");
            make_white(&mut t, bears);
            assert!(!has(&mut t, bears), "{card}: turned white later");
        }
    }
}

#[test]
fn locked_in_effects_stay_once_the_condition_changes() {
    cr!("611.2c", "611.2a", "608.2h", "108.4");
    ruling!(
        "Break of Day",
        "Break of Day checks your life total only at the time it resolves. Any creatures that gain indestructible will continue to be indestructible that turn even if you gain life."
    );
    ruling!(
        "Squad Commander",
        "Once Squad Commander's last ability has resolved, the affected creatures won't lose +1/+0 or indestructible if you no longer have a full party."
    );
    ruling!(
        "Dauntless Escort",
        "If any creature leaves your control, it will retain indestructible."
    );
    // Break of Day at 5 life, then P0 gains life.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Break of Day", &[]);
    assert!(indestructible(&mut t, bears));
    t.g.gain_life(P0, 10);
    t.settle();
    assert!(indestructible(&mut t, bears));
    // At 6 life: no indestructible.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 6);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Break of Day", &[]);
    assert!(!indestructible(&mut t, bears));
    // Squad Commander (a Warrior) and three changelings: a full party at the beginning
    // of combat. Two changelings leave after it resolves.
    supported("Squad Commander");
    let mut t = TestGame::new(2);
    let commander = t.battlefield(P0, "Squad Commander");
    let a = t.battlefield(P0, "Woodland Changeling");
    let b = t.battlefield(P0, "Woodland Changeling");
    let c = t.battlefield(P0, "Woodland Changeling");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(commander), (4, 3));
    assert!(indestructible(&mut t, commander));
    exile_now(&mut t, a);
    exile_now(&mut t, b);
    t.g.recompute();
    assert_eq!(t.pt(commander), (4, 3));
    assert!(indestructible(&mut t, commander));
    assert!(indestructible(&mut t, c));
    // Dauntless Escort: a creature that changes control keeps indestructible.
    let mut t = TestGame::new(2);
    let escort = t.battlefield(P0, "Dauntless Escort");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, escort, 0, &[]).unwrap();
    t.resolve_all();
    give_control(&mut t, bears, P1);
    assert_eq!(t.obj_now(bears).controller, P1);
    assert!(indestructible(&mut t, bears));
}
