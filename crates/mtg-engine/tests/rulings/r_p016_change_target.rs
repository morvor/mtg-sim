//! Rulings batch P016 — spells and abilities that change the target of a spell or ability
//! (CR 115.7): Deflection, Imp's Mischief, Misdirection, Shunt, Swerve, Redirect
//! Lightning ("with a single target", CR 115.9a), Redirect ("choose new targets"), Mizzium
//! Meddler and Hydroelectric Specimen ("change a target ... to this creature").

use crate::r_p016_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The cards that "change the target of target spell with a single target".
const SINGLE_TARGET: [&str; 6] = [
    "Deflection",
    "Imp's Mischief",
    "Misdirection",
    "Shunt",
    "Swerve",
    "Redirect Lightning",
];

fn setup() -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t
}

/// A spell with several targets (or the same target several times) can't be targeted by
/// `name`, even when only one of those targets is still legal.
fn cant_target_multi_target_spell(name: &str) {
    supported(name);
    // Symbiosis has two targets; one of them leaves the battlefield.
    let mut t = setup();
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let shock = cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let symbiosis = cast_multi(&mut t, P1, "Symbiosis", &[o(a), o(b)]);
    t.g.destroy(a, None);
    t.settle();
    let from = t.asked().len();
    cast_at(&mut t, P0, name, &[o(shock)]);
    let offered = &target_choices(&t, P0, from, "")[0];
    assert!(offered.contains(&o(shock)), "{name}: {offered:?}");
    assert!(!offered.contains(&o(symbiosis)), "{name}: {offered:?}");

    // Agony Warp targets the same creature twice.
    let mut t = setup();
    let a = t.battlefield(P0, "Grizzly Bears");
    let shock = cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let warp = cast_at(&mut t, P1, "Agony Warp", &[o(a), o(a)]);
    assert_eq!(stack_targets(&t, warp), vec![o(a), o(a)]);
    let from = t.asked().len();
    cast_at(&mut t, P0, name, &[o(shock)]);
    let offered = &target_choices(&t, P0, from, "")[0];
    assert!(offered.contains(&o(shock)), "{name}: {offered:?}");
    assert!(!offered.contains(&o(warp)), "{name}: {offered:?}");
}

#[test]
fn single_target_changers_cant_target_spells_with_several_targets() {
    cr!("115.9a", "115.7a");
    ruling!(
        "Redirect Lightning",
        "If a spell or ability targets multiple things, you can't target it with Redirect Lightning, even if all but one of those targets have become illegal."
    );
    ruling!(
        "Redirect Lightning",
        "If a spell or ability targets the same player or object multiple times, you can't target it with Redirect Lightning."
    );
    ruling!(
        "Swerve",
        "If a spell targets multiple things, you can't target it with Swerve, even if all but one of those targets has become illegal."
    );
    ruling!(
        "Shunt",
        "If a spell targets multiple things, you can’t target it with Shunt, even if all but one of those targets has become illegal."
    );
    ruling!(
        "Swerve",
        "If a spell targets the same player or object multiple times, you can't target it with Swerve."
    );
    ruling!(
        "Shunt",
        "If a spell targets the same player or object multiple times, you can’t target it with Shunt."
    );
    for name in SINGLE_TARGET {
        cant_target_multi_target_spell(name);
    }
}

/// `name` targets P1's Shock aimed at `orig`: the new target is chosen on resolution and
/// must be another legal target if there is one; returns (game, Shock, `name`'s spell, the
/// index of the decisions asked once `name` was cast).
fn change_shock(
    t: &mut TestGame,
    name: &str,
    orig: Entity,
    answer_new: &[Entity],
) -> (ObjectId, ObjectId, usize) {
    let shock = cast_at(t, P1, "Shock", &[orig]);
    let s = cast_at(t, P0, name, &[o(shock)]);
    let from = t.asked().len();
    t.answer_targets(P0, answer_new);
    t.resolve();
    (shock, s, from)
}

#[test]
fn the_new_target_is_chosen_on_resolution_and_must_be_changed_if_possible() {
    cr!("115.7a", "608.2b");
    ruling!(
        "Redirect Lightning",
        "You don't choose the new target for the spell or ability until Redirect Lightning resolves. You must change the target if possible."
    );
    ruling!(
        "Swerve",
        "You don't choose the new target for the spell until Swerve resolves. You must change the target if possible."
    );
    ruling!(
        "Deflection",
        "You choose the spell to target on announcement, but you pick the new target for that spell on resolution."
    );
    for name in SINGLE_TARGET {
        // The original target was destroyed (it's illegal): the target still changes, even
        // when P0 tries to decline.
        let mut t = setup();
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let shock = cast_at(&mut t, P1, "Shock", &[o(bears)]);
        t.g.destroy(bears, None);
        t.settle();
        let s = cast_at(&mut t, P0, name, &[o(shock)]);
        // Nothing about the new target was asked while casting.
        let from = t.asked().len();
        assert!(target_choices(&t, P0, 0, "New target").is_empty(), "{name}");
        t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![]));
        t.resolve();
        let offered = &target_choices(&t, P0, from, "New target")[0];
        assert!(offered.contains(&o(giant)), "{name}");
        assert!(!t.g.stack.contains(&s), "{name}");
        assert_eq!(stack_targets(&t, shock), vec![o(giant)], "{name}");
        t.resolve();
        assert_eq!(t.obj_now(giant).damage, 2, "{name}");
    }
}

#[test]
fn with_no_other_legal_target_the_target_is_unchanged() {
    cr!("115.7a");
    ruling!(
        "Redirect Lightning",
        "However, you can't change the target to an illegal target. If there are no legal targets to choose from, the target isn't changed. It doesn't matter if the original target has somehow become illegal itself."
    );
    ruling!(
        "Swerve",
        "However, you can't change the target to an illegal target. If there are no legal targets, the target isn't changed. It doesn't matter if the original target of that spell has somehow become illegal."
    );
    for name in ["Swerve", "Redirect Lightning"] {
        // Doom Blade at P0's Bears; the only other creature is black.
        let mut t = setup();
        let bears = t.battlefield(P0, "Grizzly Bears");
        let hawk = t.battlefield(P1, "Vampire Nighthawk");
        let blade = cast_at(&mut t, P1, "Doom Blade", &[o(bears)]);
        cast_at(&mut t, P0, name, &[o(blade)]);
        t.answer_targets(P0, &[o(hawk)]);
        t.resolve();
        assert_eq!(stack_targets(&t, blade), vec![o(bears)], "{name}");
        t.resolve();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        assert!(t.on_battlefield(hawk), "{name}");
    }
}

#[test]
fn redirect_lightning_changes_the_target_of_an_ability() {
    cr!("115.7a");
    ruling!(
        "Redirect Lightning",
        "You don't choose the new target for the spell or ability until Redirect Lightning resolves."
    );
    // Prodigal Pyromancer: "{T}: This creature deals 1 damage to any target."
    let mut t = setup();
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.g.turn.priority = Some(P1);
    let ab = t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    let ab = ab.expect("ability on the stack");
    cast_at(&mut t, P0, "Redirect Lightning", &[o(ab)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(stack_targets(&t, ab), vec![Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn the_new_target_becomes_targeted_and_triggers_becomes_the_target_abilities() {
    cr!("115.7", "603.2");
    ruling!(
        "Deflection",
        "Once the spell resolves, the new target is considered to be targeted by the deflected spell. This will trigger any effects which trigger on being targeted."
    );
    for name in SINGLE_TARGET {
        // Phantasmal Bear: "When this creature becomes the target of a spell or ability,
        // sacrifice it."
        let mut t = setup();
        let giant = t.battlefield(P0, "Hill Giant");
        let bear = t.battlefield(P0, "Phantasmal Bear");
        let (shock, _, _) = change_shock(&mut t, name, o(giant), &[o(bear)]);
        assert_eq!(stack_targets(&t, shock), vec![o(bear)], "{name}");
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Phantasmal Bear"), "{name}");
        assert_eq!(t.obj_now(giant).damage, 0, "{name}");
    }
}

#[test]
fn only_the_spell_is_targeted_not_its_original_or_new_target() {
    cr!("115.1", "115.7");
    ruling!(
        "Deflection",
        "This only targets the spell being changed, not the original or new target of the spell it is affecting."
    );
    ruling!(
        "Swerve",
        "Swerve targets only the spell whose target will be changed. It doesn't directly affect the original target of that spell or the new target of that spell."
    );
    for name in SINGLE_TARGET {
        // P1's Giant Growth targets P1's hexproof Gladecover Scout: P0 can still target
        // the spell, and change its target to P0's Bears.
        let mut t = setup();
        let scout = t.battlefield(P1, "Gladecover Scout");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let growth = cast_at(&mut t, P1, "Giant Growth", &[o(scout)]);
        let s = cast_at(&mut t, P0, name, &[o(growth)]);
        assert_eq!(stack_targets(&t, s), vec![o(growth)], "{name}");
        t.answer_targets(P0, &[o(bears)]);
        t.resolve();
        t.resolve();
        assert_eq!(t.pt(bears), (5, 5), "{name}");
        assert_eq!(t.pt(scout), (1, 1), "{name}");
    }
}

/// P1 casts Shock; P0 casts Cancel at it; then `extra` (if any) is cast by P1 above the
/// Cancel. Returns (Shock, Cancel, extra).
fn cancel_on_stack(
    t: &mut TestGame,
    extra: Option<&str>,
) -> (ObjectId, ObjectId, Option<ObjectId>) {
    let shock = cast_at(t, P1, "Shock", &[Entity::Player(P0)]);
    let cancel = cast_at(t, P0, "Cancel", &[o(shock)]);
    let extra = extra.map(|n| cast_at(t, P1, n, &[Entity::Player(P0)]));
    (shock, cancel, extra)
}

#[test]
fn a_spell_targeting_a_spell_can_be_changed_to_the_changer_but_not_to_itself() {
    cr!("115.5", "115.7a", "608.2b");
    ruling!(
        "Swerve",
        "If you cast Swerve on a spell that targets a spell on the stack (like Cancel does, for example), you can't change that spell's target to itself. You can, however, change that spell's target to Swerve."
    );
    ruling!(
        "Deflection",
        "You can choose to make a spell on the stack target this spell (if such a target choice would be legal had the spell been cast while this spell was on the stack). The new target for the deflected spell is not chosen until this spell resolves. This spell is still on the stack when new targets are selected for the spell."
    );
    ruling!(
        "Deflection",
        "You can’t make a spell which is on the stack target itself."
    );
    for name in SINGLE_TARGET {
        let mut t = setup();
        let (shock, cancel, _) = cancel_on_stack(&mut t, None);
        let s = cast_at(&mut t, P1, name, &[o(cancel)]);
        let from = t.asked().len();
        t.answer_targets(P1, &[o(s)]);
        t.resolve();
        let offered = &target_choices(&t, P1, from, "New target")[0];
        assert!(offered.contains(&o(s)), "{name}: {offered:?}");
        assert!(!offered.contains(&o(cancel)), "{name}: {offered:?}");
        assert_eq!(stack_targets(&t, cancel), vec![o(s)], "{name}");
        // The Cancel's only target left the stack: it doesn't resolve; Shock does.
        t.resolve();
        assert!(t.in_graveyard(P0, "Cancel"), "{name}");
        assert!(t.g.stack.contains(&shock), "{name}");
        t.resolve();
        assert_eq!(t.life(P0), 18, "{name}");
    }
}

#[test]
fn a_spell_targeting_a_spell_can_be_changed_to_one_that_resolves_first() {
    cr!("115.7a", "608.2b");
    ruling!(
        "Deflection",
        "The target of a spell that targets another spell on the stack can be changed to any other spell on the stack, even if the new target will resolve before the spell does."
    );
    for name in SINGLE_TARGET {
        // Shock, then P0's Cancel at it, then P1's Lightning Bolt above the Cancel: the
        // Cancel's target becomes the Bolt, which resolves first.
        let mut t = setup();
        let (shock, cancel, bolt) = cancel_on_stack(&mut t, Some("Lightning Bolt"));
        let bolt = bolt.unwrap();
        cast_at(&mut t, P1, name, &[o(cancel)]);
        t.answer_targets(P1, &[o(bolt)]);
        t.resolve();
        assert_eq!(stack_targets(&t, cancel), vec![o(bolt)], "{name}");
        t.resolve_all();
        let _ = shock;
        // Both burn spells resolved; the Cancel had no legal target left.
        assert_eq!(t.life(P0), 15, "{name}");
    }
}

#[test]
fn redirect_works_on_any_spell_but_does_nothing_to_a_spell_without_targets() {
    cr!("115.7d", "303.4a");
    ruling!(
        "Redirect",
        "Redirect can target any spell, not just an instant or sorcery spell. For example, you could use it to change the target of an Aura spell. However, if the targeted spell has no targets"
    );
    // An Aura spell: Rancor's target changes to P1's Giant.
    let mut t = setup();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let rancor = cast_at(&mut t, P1, "Rancor", &[o(bears)]);
    cast_at(&mut t, P0, "Redirect", &[o(rancor)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(giant)]);
    t.resolve();
    assert_eq!(stack_targets(&t, rancor), vec![o(giant)]);
    t.resolve();
    assert_eq!(t.pt(giant), (5, 3));
    assert_eq!(t.pt(bears), (2, 2));
    // A creature spell: Redirect can target it; nothing happens.
    let mut t = setup();
    let bears_spell = cast_at(&mut t, P1, "Grizzly Bears", &[]);
    cast_at(&mut t, P0, "Redirect", &[o(bears_spell)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Redirect"));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

// ---------------------------------------------------------------------------
// Mizzium Meddler and Hydroelectric Specimen: "change a target ... to this creature"
// ---------------------------------------------------------------------------

/// P0's `name` enters with its enters ability targeting `spell`; P0 says yes to "may".
fn meddle(t: &mut TestGame, name: &str, spell: ObjectId) -> ObjectId {
    supported(name);
    t.answer_targets(P0, &[o(spell)]);
    t.answer_yes(P0, true);
    let m = crate::r_s05_common::enter(t, P0, name);
    t.resolve();
    m
}

#[test]
fn meddler_can_target_a_spell_it_couldnt_be_a_target_of_but_nothing_changes() {
    cr!("115.7b");
    ruling!(
        "Mizzium Meddler",
        "Mizzium Meddler’s triggered ability can target a spell or ability even if Mizzium Meddler wouldn’t be a legal target for that spell or ability. However, the target of that spell or ability will remain unchanged."
    );
    ruling!(
        "Mizzium Meddler",
        "The target of the spell or ability won’t change unless Mizzium Meddler fulfills all the targeting criteria."
    );
    // Lava Spike targets only players.
    let mut t = setup();
    let spike = cast_at(&mut t, P1, "Lava Spike", &[Entity::Player(P0)]);
    meddle(&mut t, "Mizzium Meddler", spike);
    assert_eq!(stack_targets(&t, spike), vec![Entity::Player(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    // Fall of the Hammer's first target must be a creature its controller controls.
    let mut t = setup();
    let mine = t.battlefield(P1, "Hill Giant");
    let theirs = t.battlefield(P0, "Grizzly Bears");
    let hammer = cast_at(&mut t, P1, "Fall of the Hammer", &[o(mine), o(theirs)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let m = meddle(&mut t, "Mizzium Meddler", hammer);
    // Only the second target could become the Meddler.
    assert_eq!(stack_targets(&t, hammer), vec![o(mine), o(m)]);
}

#[test]
fn meddler_changes_only_one_target_chosen_on_resolution() {
    cr!("115.7b");
    ruling!(
        "Mizzium Meddler",
        "If a spell or ability has multiple targets, such as Send to Sleep, you can change only one of the targets to Mizzium Meddler."
    );
    ruling!(
        "Mizzium Meddler",
        "If the spell or ability has multiple instances of the word “target,” you choose which target you’re changing to Mizzium Meddler as Mizium Meddler’s ability resolves."
    );
    // Send to Sleep: "Tap up to two target creatures."
    let mut t = setup();
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let sleep = cast_multi(&mut t, P1, "Send to Sleep", &[o(a), o(b)]);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let m = meddle(&mut t, "Mizzium Meddler", sleep);
    assert_eq!(stack_targets(&t, sleep), vec![o(a), o(m)]);
    // The choice of which target was made as the ability resolved.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && format!("{d:?}").contains("Change which target")));
}

#[test]
fn meddler_wont_change_a_target_if_that_makes_another_target_illegal() {
    cr!("115.7b", "115.7e");
    ruling!(
        "Mizzium Meddler",
        "If changing one target of a spell or ability to Mizzium Meddler would make other targets of that spell or ability illegal, the targets remain unchanged."
    );
    // Mutiny: "Target creature an opponent controls deals damage equal to its power to
    // another target creature that player controls." P1 targets two of P2's creatures; P0's
    // Meddler replacing the first would leave the second not controlled by "that player".
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P2, "Hill Giant");
    let bears = t.battlefield(P2, "Grizzly Bears");
    let mutiny = cast_at(&mut t, P1, "Mutiny", &[o(giant), o(bears)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    meddle(&mut t, "Mizzium Meddler", mutiny);
    assert_eq!(stack_targets(&t, mutiny), vec![o(giant), o(bears)]);
    t.resolve();
    assert!(t.in_graveyard(P2, "Grizzly Bears"));
}

#[test]
fn hydroelectric_specimen_changes_the_target_only_if_it_is_legal() {
    cr!("115.7a");
    ruling!(
        "Hydroelectric Specimen // Hydroelectric Laboratory",
        "The spell's target is changed to Hydroelectric Specimen only if Hydroelectric Specimen is a legal target for that spell."
    );
    let mut t = setup();
    let spike = cast_at(&mut t, P1, "Lava Spike", &[Entity::Player(P0)]);
    meddle(&mut t, "Hydroelectric Specimen", spike);
    assert_eq!(stack_targets(&t, spike), vec![Entity::Player(P0)]);
    let mut t = setup();
    let shock = cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let s = meddle(&mut t, "Hydroelectric Specimen", shock);
    assert_eq!(stack_targets(&t, shock), vec![o(s)]);
}
