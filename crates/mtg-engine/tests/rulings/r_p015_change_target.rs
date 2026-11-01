//! Rulings batch P015 — changing targets (CR 115.7): Swerve and an Aura spell, Reflecting
//! Mirror and abilities, Mizzium Meddler and Hydroelectric Specimen; and Mendicant Core's
//! copies of artifact spells.

use crate::r_p015_common::*;
use crate::r_p016_common::{cast_at, cast_multi, lots, o, stack_targets, target_choices};
use crate::r_s01_common::supported;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn setup() -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t
}

#[test]
fn swerve_changes_the_creature_an_aura_spell_will_enchant() {
    cr!("115.7a", "303.4a", "303.4f");
    ruling!(
        "Swerve",
        "An Aura spell on the stack targets the object or player which it will enchant upon entering. Thus, an Aura spell is a \"spell with a single target\", which means you may use Swerve to change that target. Doing so will cause it to enter enchanting the new target rather than the original one."
    );
    supported("Swerve");
    let mut t = setup();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let pacifism = cast_at(&mut t, P1, "Pacifism", &[o(bears)]);
    cast_at(&mut t, P0, "Swerve", &[o(pacifism)]);
    t.answer_targets(P0, &[o(giant)]);
    t.resolve();
    assert_eq!(stack_targets(&t, pacifism), vec![o(giant)]);
    t.resolve();
    let aura = t.named_on_battlefield("Pacifism");
    assert_eq!(aura.len(), 1);
    assert_eq!(t.obj_now(aura[0]).attached_to, Some(o(t.g.current(giant))));
}

#[test]
fn reflecting_mirror_cant_target_abilities() {
    cr!("115.7a", "115.9a");
    ruling!(
        "Reflecting Mirror",
        "Can’t be used on abilities of permanents. Those are abilities and not spells."
    );
    supported("Reflecting Mirror");
    let mut t = setup();
    let mirror = t.battlefield(P0, "Reflecting Mirror");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.g.turn.priority = Some(P1);
    let ab = t
        .activate(P1, pyro, 0, &[Entity::Player(P0)])
        .unwrap()
        .expect("ability on the stack");
    lots(&mut t, P0);
    t.g.turn.priority = Some(P0);
    // Even with X = 0 (an ability has no mana value), it isn't a spell.
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    assert!(t.activate(P0, mirror, 0, &[o(ab)]).is_err());
    t.clear_answers();
    // A spell that targets P0 can be.
    let shock = cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    lots(&mut t, P0);
    t.g.turn.priority = Some(P0);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let r = t.activate(P0, mirror, 0, &[o(shock)]);
    assert!(r.is_ok(), "{r:?}");
}

/// P0's `name` enters with its enters ability targeting `spell`; P0 says yes to "may".
/// The trigger is left on the stack.
fn meddler_enters(t: &mut TestGame, name: &str, spell: ObjectId) -> ObjectId {
    supported(name);
    t.answer_targets(P0, &[o(spell)]);
    t.answer_yes(P0, true);
    enter(t, P0, name)
}

#[test]
fn meddler_gone_before_its_trigger_resolves_leaves_targets_unchanged() {
    cr!("115.7b", "400.7");
    ruling!(
        "Mizzium Meddler",
        "If Mizzium Meddler leaves the battlefield before its ability resolves, the targets remain unchanged."
    );
    // Control: with Meddler still on the battlefield, Shock is redirected to it.
    let mut t = setup();
    let shock = cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let m = meddler_enters(&mut t, "Mizzium Meddler", shock);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(stack_targets(&t, shock), vec![o(t.g.current(m))]);

    let mut t = setup();
    let shock = cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let m = meddler_enters(&mut t, "Mizzium Meddler", shock);
    assert_eq!(t.stack_len(), 2);
    move_to(&mut t, m, Zone::Hand(P0));
    t.resolve();
    assert_eq!(stack_targets(&t, shock), vec![Entity::Player(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn meddler_targeting_a_spell_without_targets_does_nothing() {
    cr!("115.7b");
    ruling!(
        "Mizzium Meddler",
        "If Mizzium Meddler’s triggered ability targets a spell or ability with no targets, nothing happens."
    );
    let mut t = setup();
    t.library_top(P1, "Forest");
    t.library_top(P1, "Forest");
    let div = cast_at(&mut t, P1, "Divination", &[]);
    let m = meddler_enters(&mut t, "Mizzium Meddler", div);
    t.resolve();
    assert!(stack_targets(&t, div).is_empty());
    t.resolve();
    assert_eq!(t.hand_size(P1), 2);
    assert!(on_bf(&t, m));
}

#[test]
fn hydroelectric_specimen_cant_target_a_spell_with_several_targets() {
    cr!("115.9a");
    ruling!(
        "Hydroelectric Specimen // Hydroelectric Laboratory",
        "If a spell has multiple targets, you can't target it with Hydroelectric Specimen's ability, even if only one of those targets is a creature or all but one of those targets have become illegal."
    );
    let name = "Hydroelectric Specimen // Hydroelectric Laboratory";
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
    enter(&mut t, P0, name);
    let offered = &target_choices(&t, P0, from, "")[0];
    assert!(offered.contains(&o(shock)), "{offered:?}");
    assert!(!offered.contains(&o(symbiosis)), "{offered:?}");
    // Electrolyze divides damage among one or two targets: a creature and a player.
    let mut t = setup();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shock = cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let electrolyze = cast_multi(&mut t, P1, "Electrolyze", &[o(bears), Entity::Player(P0)]);
    assert_eq!(stack_targets(&t, electrolyze).len(), 2);
    let from = t.asked().len();
    enter(&mut t, P0, name);
    let offered = &target_choices(&t, P0, from, "")[0];
    assert!(offered.contains(&o(shock)), "{offered:?}");
    assert!(!offered.contains(&o(electrolyze)), "{offered:?}");
}

#[test]
fn mendicant_core_copy_remembers_x() {
    cr!("707.10", "707.2");
    ruling!(
        "Mendicant Core, Guidelight",
        "The copy remembers any decisions that were made for the original spell as it was cast, including values chosen for X in its mana cost and whether any alternative or additional costs were chosen."
    );
    supported("Mendicant Core, Guidelight");
    supported("Walking Ballista");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.g.players[0].speed = Some(4);
    t.battlefield(P0, "Mendicant Core, Guidelight");
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::C, 5);
    t.answer_yes(P0, true);
    let c = t.hand(P0, "Walking Ballista");
    t.cast(P0, c).x(2).go();
    t.resolve_all();
    let ballistas = t.named_on_battlefield("Walking Ballista");
    assert_eq!(ballistas.len(), 2);
    for b in ballistas {
        assert_eq!(t.counters(b, mtg_engine::types::counters::PLUS1), 2);
    }
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
}
