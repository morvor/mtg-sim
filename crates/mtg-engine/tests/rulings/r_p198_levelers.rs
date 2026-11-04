//! Rulings batch P198 — creatures that "become" a creature type with a base power and
//! toughness, step by step (Figure of Destiny and its kin): each step's type check is made
//! on resolution (CR 608.2c), the effects are timestamped layer 4 and layer 7b effects that
//! last indefinitely (CR 613.4b, 613.7, 611.2a), and switching is applied last (613.4d).

use crate::r_p198_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::Supertype;
use mtg_engine::*;

/// The steps of each card: (card, a step that needs the previous one, the first step,
/// the creature types and base power and toughness after the first step).
const LADDERS: &[(&str, usize, &[&str], (i32, i32))] = &[
    ("Figure of Fable", 1, &["Kithkin", "Scout"], (2, 3)),
    ("Figure of Destiny", 1, &["Kithkin", "Spirit"], (2, 2)),
    ("Evolved Sleeper", 1, &["Cleric", "Human"], (2, 2)),
    ("Ascendant Spirit", 1, &["Spirit", "Warrior"], (2, 3)),
];

/// A creature `name` for P0 with enough mana (snow mana too) for its steps.
fn leveler(t: &mut TestGame, name: &str) -> ObjectId {
    t.lands(P0, "Snow-Covered Swamp", 12);
    t.battlefield(P0, name)
}

#[test]
fn each_step_checks_the_creature_type_as_it_resolves() {
    cr!("608.2c", "602.2");
    ruling!(
        "Figure of Destiny",
        "You can activate Figure of Destiny's second and third abilities regardless of what creature types it is. Each of those abilities checks Figure of Destiny's creature types when that ability resolves. If Figure of Destiny isn't the appropriate creature type at that time, the ability does nothing."
    );
    ruling!(
        "Figure of Fable",
        "You can activate Figure of Fable's abilities regardless of what creature types it currently has. Each ability checks Figure of Fable's creature types when it resolves. If Figure of Fable doesn't have the appropriate creature type at that time, the ability will do nothing. (Go figure.)"
    );
    ruling!(
        "Evolved Sleeper",
        "You can activate Evolved Sleeper's second and third abilities regardless of what creature types it is. Each of those abilities checks Evolved Sleeper's creature types when that ability resolves. If Evolved Sleeper isn't the appropriate creature type at that time, the ability does nothing."
    );
    for (name, second, types, pt) in LADDERS {
        supported(name);
        // The second step activated with the first on the stack beneath it: it resolves
        // first and does nothing, though it could be activated.
        let mut t = TestGame::new(2);
        let c = leveler(&mut t, name);
        act_only(&mut t, P0, c, 0, &[]);
        act_only(&mut t, P0, c, *second, &[]);
        assert_eq!(t.stack_len(), 2, "{name}");
        t.resolve();
        assert_eq!(t.pt(c), (1, 1), "{name}");
        assert_eq!(t.counters(c, "deathtouch"), 0, "{name}");
        t.resolve_all();
        assert_eq!(t.pt(c), *pt, "{name}");
        assert_eq!(creature_types(&t, c), types.to_vec(), "{name}");
        // In the other order it works.
        act(&mut t, P0, c, *second, &[]);
        assert!(t.pt(c).0 > pt.0, "{name}");
    }
    // Kellan's second step needs a Detective; Kellan starts a Scout.
    supported("Kellan, Planar Trailblazer");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kellan, Planar Trailblazer");
    act(&mut t, P0, k, 1, &[]);
    assert_eq!(t.pt(k), (2, 1));
    assert!(creature_types(&t, k).contains(&"Scout".to_string()));
    act(&mut t, P0, k, 0, &[]);
    assert_eq!(
        creature_types(&t, k),
        vec!["Detective".to_string(), "Faerie".into(), "Human".into()]
    );
    // The first step again does nothing now (it isn't a Scout).
    act(&mut t, P0, k, 0, &[]);
    act(&mut t, P0, k, 1, &[]);
    assert_eq!(t.pt(k), (3, 2));
    assert!(t
        .obj_now(k)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
}

#[test]
fn kellans_steps_check_his_types_on_resolution() {
    cr!("608.2c");
    ruling!(
        "Kellan, Planar Trailblazer",
        "You can activate Kellan's abilities regardless of what creature types he currently has. Each ability checks Kellan's creature types when it resolves. If Kellan doesn't have the appropriate creature type at that time, the ability will do nothing."
    );
    // Both activated while he's a Scout: the Detective step resolves first and does
    // nothing.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kellan, Planar Trailblazer");
    act_only(&mut t, P0, k, 0, &[]);
    act_only(&mut t, P0, k, 1, &[]);
    t.resolve_all();
    assert_eq!(t.pt(k), (2, 1));
    assert!(creature_types(&t, k).contains(&"Detective".to_string()));
}

#[test]
fn the_steps_overwrite_only_earlier_setting_effects_and_switching_is_last() {
    cr!("613.4b", "613.4c", "613.4d", "613.7");
    ruling!(
        "Figure of Fable",
        "The effects from Figure of Fable's abilities overwrite other effects that set power and/or toughness if and only if those effects existed before the ability resolved."
    );
    ruling!(
        "Kellan, Planar Trailblazer",
        "The effects from Kellan's abilities overwrite other effects that set power and/or toughness if and only if those effects existed before the ability resolved."
    );
    ruling!(
        "Evolved Sleeper",
        "The effects from the first and second abilities overwrite other effects that set power and/or toughness if and only if those effects existed before the ability resolved."
    );
    ruling!(
        "Ascendant Spirit",
        "The first and second abilities overwrite any previous effects that set Ascendant Spirit's base power and toughness to specific values. Any effects that set Ascendant Spirit's base power and toughness that start to apply after one of these abilities will override the effect of that ability."
    );
    supported("Diminish");
    supported("Twisted Image");
    for (name, _, _, pt) in LADDERS {
        // Diminish (1/1) before the step: overwritten.
        let mut t = TestGame::new(2);
        let c = leveler(&mut t, name);
        cast_at(&mut t, P0, "Diminish", &[obj(c)]);
        t.resolve_all();
        act(&mut t, P0, c, 0, &[]);
        assert_eq!(t.pt(c), *pt, "{name}");
        // Diminish after: it wins. A +1/+1 counter still applies on top.
        cast_at(&mut t, P0, "Diminish", &[obj(c)]);
        t.resolve_all();
        assert_eq!(t.pt(c), (1, 1), "{name}");
        t.g.add_counters(obj(c), "+1/+1", 1, None);
        t.g.recompute();
        assert_eq!(t.pt(c), (2, 2), "{name}");
        // A switch from before the step still applies after it.
        let mut t = TestGame::new(2);
        let c = leveler(&mut t, name);
        t.g.add_counters(obj(c), "+1/+1", 1, None);
        cast_at(&mut t, P0, "Twisted Image", &[obj(c)]);
        t.resolve_all();
        act(&mut t, P0, c, 0, &[]);
        assert_eq!(t.pt(c), (pt.1 + 1, pt.0 + 1), "{name}");
    }
    // Kellan's 3/2 step.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kellan, Planar Trailblazer");
    act(&mut t, P0, k, 0, &[]);
    cast_at(&mut t, P0, "Diminish", &[obj(k)]);
    cast_at(&mut t, P0, "Twisted Image", &[obj(k)]);
    t.resolve_all();
    act(&mut t, P0, k, 1, &[]);
    assert_eq!(t.pt(k), (2, 3));
}

#[test]
fn the_switch_applies_after_these_effects_and_counters() {
    cr!("613.4d");
    ruling!(
        "Figure of Fable",
        "nor will they overwrite effects that set power and toughness which come into existence after they resolve. Effects that switch a creature's power and toughness are always applied after any other power- or toughness-changing effects, regardless of the order in which they are created."
    );
    ruling!(
        "Kellan, Planar Trailblazer",
        "Effects that switch the creature's power and toughness are always applied after any other power or toughness changing effects, including these two, regardless of the order in which they are created."
    );
    ruling!(
        "Evolved Sleeper",
        "Effects that switch the creature's power and toughness are always applied after any other power or toughness changing effects, including this one, regardless of the order in which they are created."
    );
    // Switched between the two steps, then pumped +3/+1: the switch still comes last.
    supported("Brute Strength");
    let mut t = TestGame::new(2);
    let c = leveler(&mut t, "Evolved Sleeper");
    act(&mut t, P0, c, 0, &[]);
    cast_at(&mut t, P0, "Twisted Image", &[obj(c)]);
    t.resolve_all();
    act(&mut t, P0, c, 1, &[]);
    assert_eq!(t.counters(c, "deathtouch"), 1);
    assert_eq!(t.pt(c), (3, 3));
    cast_at(&mut t, P0, "Brute Strength", &[obj(c)]);
    t.resolve_all();
    assert_eq!(t.pt(c), (4, 6));
    let mut t = TestGame::new(2);
    let c = leveler(&mut t, "Figure of Fable");
    act(&mut t, P0, c, 0, &[]);
    cast_at(&mut t, P0, "Twisted Image", &[obj(c)]);
    t.resolve_all();
    assert_eq!(t.pt(c), (3, 2));
    act(&mut t, P0, c, 1, &[]);
    assert_eq!(t.pt(c), (5, 4));
}

#[test]
fn ascendant_spirit_loses_its_other_creature_types_but_stays_snow() {
    cr!("205.1b", "613.1d", "613.7");
    ruling!(
        "Ascendant Spirit",
        "The first and second abilities cause Ascendant Spirit to lose any other creature types it may have. It retains any card types or supertypes it may have."
    );
    // Olivia made it a Vampire before the first step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Olivia, Mobilized for War");
    t.hand(P0, "Island");
    t.lands(P0, "Snow-Covered Swamp", 12);
    let s = t.enter(P0, "Ascendant Spirit");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.obj_now(s).chars.has_subtype("Vampire"));
    act(&mut t, P0, s, 0, &[]);
    assert_eq!(creature_types(&t, s), vec!["Spirit", "Warrior"]);
    act(&mut t, P0, s, 1, &[]);
    assert_eq!(creature_types(&t, s), vec!["Angel", "Spirit", "Warrior"]);
    let o = t.obj_now(s);
    assert!(o.chars.has_supertype(Supertype::Snow));
    assert!(o.is_creature());
    assert_eq!(t.counters(s, "flying"), 1);
    // 4/4 base plus Olivia's +1/+1 counter.
    assert_eq!(t.pt(s), (5, 5));
}
