//! CR 608.3d: a resolving mutating creature spell doesn't become a new permanent: the
//! object representing it merges with the permanent it's targeting (CR 730).

use crate::r703_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::merge;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const MUTATE: CastMethod = CastMethod::Keyword(KeywordKind::Mutate);

/// Casts Gemrazer ("Reach, trample; Mutate {1}{G}{G}; Whenever this creature mutates,
/// destroy target artifact or enchantment an opponent controls.") from `p`'s hand for its
/// mutate cost, targeting `target`. Returns the spell.
fn cast_mutating_gemrazer(t: &mut TestGame, p: PlayerId, target: ObjectId) -> ObjectId {
    let gem = t.hand(p, "Gemrazer");
    let pool = &mut t.g.players[p.idx()].mana_pool;
    pool.add_type(ManaType::G, 2);
    pool.add_type(ManaType::C, 1);
    t.cast(p, gem).method(MUTATE).target(target).go()
}

/// Answers the next "on top or under?" choice of a merging spell.
fn put_on_top(t: &mut TestGame, p: PlayerId, on_top: bool) {
    t.answer(
        p,
        DecisionKind::Option,
        Answer::Index(if on_top { 0 } else { 1 }),
    );
}

#[test]
fn a_mutating_spell_merges_with_the_permanent_it_targets_instead_of_entering() {
    cr!("608.3", "608.3d");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Two non-Human creatures P0 owns; the spell targets the Bears.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let spell = cast_mutating_gemrazer(&mut t, P0, bears);
    let permanents = t.g.battlefield.len();
    put_on_top(&mut t, P0, true);
    t.resolve();
    // The spell left the stack, but it didn't become a permanent of its own (as a creature
    // spell without a target would, CR 608.3a), and it wasn't put into a graveyard.
    assert_eq!(t.g.obj(spell).zone, Zone::Nowhere);
    assert!(!t.g.stack.contains(&spell));
    assert_eq!(t.g.battlefield.len(), permanents);
    assert_eq!(t.graveyard_size(P0), 0);
    // The card that represented the spell now represents the Bears — the permanent the
    // spell targeted, not the other creature — which is still the same object.
    assert_eq!(t.g.current(spell), bears);
    assert!(t.g.is_live(bears));
    let comps = merge::physical_components(&t.g, bears);
    assert_eq!(comps.len(), 2);
    assert_eq!(t.obj(comps[0]).chars.name, "Gemrazer");
    assert_eq!(t.obj(comps[1]).chars.name, "Grizzly Bears");
    assert!(merge::physical_components(&t.g, elves).is_empty());
    assert_eq!(t.named_on_battlefield("Gemrazer"), vec![bears]);
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn the_merged_permanent_stays_under_its_controllers_control() {
    cr!("608.3d");
    supported("Mind Control");
    let mut t = TestGame::new(2);
    // P0 owns the Bears; P1 controls them (Mind Control: "You control enchanted
    // creature.").
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 5);
    let mc = t.hand(P1, "Mind Control");
    t.cast(P1, mc).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj(bears).controller, P1);
    // A mutating creature spell targets a non-Human creature with the same owner as the
    // spell, so P0 can target the Bears P1 controls.
    t.set_step(P0, Step::PrecombatMain);
    let spell = cast_mutating_gemrazer(&mut t, P0, bears);
    put_on_top(&mut t, P0, true);
    t.resolve();
    // It merges with the Bears rather than entering under the spell controller's control:
    // P1 controls the merged creature, and P0 controls no creatures.
    assert_eq!(t.g.current(spell), bears);
    assert_eq!(t.obj(bears).chars.name, "Gemrazer");
    assert_eq!(t.obj(bears).controller, P1);
    assert!(t.g.permanents_controlled_by(P0).is_empty());
}

#[test]
fn a_copy_of_a_mutating_spell_merges_with_the_same_target() {
    cr!("608.3d", "608.3f");
    ruling!(
        "Lithoform Engine",
        "If a permanent spell is copied, new targets can't be chosen for it, if it has any (perhaps because it's an Aura or a mutating creature spell)."
    );
    // (Its first ability, copying an ability, isn't supported; the one under test is.)
    let lithoform = mtg_engine::card::card("Lithoform Engine");
    let unsupported = lithoform.unsupported_text();
    assert!(
        unsupported.iter().all(|a| !a.contains("permanent spell")),
        "{unsupported:?}"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Another non-Human creature P0 owns: a legal target for a mutating spell of theirs.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Wastes", 4);
    let spell = cast_mutating_gemrazer(&mut t, P0, bears);
    // "{4}, {T}: Copy target permanent spell you control. (The copy becomes a token.)"
    t.activate(P0, engine, 1, &[Entity::Object(spell)]).unwrap();
    // If P0 were offered new targets for the copy, they'd move it to the Elves.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert!(
        !t.asked().iter().any(|(_, d)| matches!(d, Decision::YesNo { .. })),
        "new targets were offered for a copy of a permanent spell"
    );
    t.clear_answers();
    let permanents = t.g.battlefield.len();
    // The copy resolves first: it becomes a token as it merges with the Bears (put under
    // them) — it doesn't enter as a token creature of its own, nor merge with the Elves.
    put_on_top(&mut t, P0, false);
    t.resolve();
    assert_eq!(t.g.battlefield.len(), permanents);
    assert!(merge::physical_components(&t.g, elves).is_empty());
    let comps = merge::physical_components(&t.g, bears);
    assert_eq!(comps.len(), 2);
    assert_eq!(t.obj(comps[0]).chars.name, "Grizzly Bears");
    assert!(t.obj(comps[1]).is_token());
    assert_eq!(t.obj(comps[1]).chars.name, "Gemrazer");
    assert!(!t.obj(bears).is_token());
    // Then the original merges with the same creature, on top. (Gemrazer's "whenever this
    // creature mutates" trigger has no legal target: P1 controls no artifact or enchantment.)
    t.settle();
    t.clear_answers();
    for _ in 0..10 {
        if t.g.stack.last() == Some(&spell) {
            break;
        }
        t.resolve();
    }
    assert_eq!(t.g.stack.last(), Some(&spell));
    put_on_top(&mut t, P0, true);
    t.resolve();
    assert_eq!(t.g.battlefield.len(), permanents);
    assert_eq!(t.g.current(spell), bears);
    assert_eq!(merge::physical_components(&t.g, bears).len(), 3);
    assert!(merge::physical_components(&t.g, elves).is_empty());
    assert_eq!(t.obj(bears).chars.name, "Gemrazer");
}
