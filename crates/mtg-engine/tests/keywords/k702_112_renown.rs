//! CR 702.112 Renown.

use crate::common_k702_111_124::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn renowned(t: &TestGame, id: ObjectId) -> bool {
    let o = t.obj_now(id);
    o.renowned && mtg_engine::kw::renown::is_renowned(&t.g, o.id)
}

/// Attacks `p1` with `attacker` unblocked, through the end of combat.
fn hit(t: &mut TestGame, attacker: ObjectId) {
    declare_attack(t, &[(attacker, Entity::Player(P1))]);
    finish_combat(t, P1, &[]);
}

#[test]
fn renown_puts_counters_on_it_and_it_becomes_renowned() {
    cr!("702.112", "702.112a");
    assert_supported_card("Topan Freeblade");
    let mut t = TestGame::new(2);
    // Topan Freeblade: 2/2 vigilance, renown 1.
    let blade = t.battlefield(P0, "Topan Freeblade");
    assert!(!renowned(&t, blade));
    hit(&mut t, blade);
    assert_eq!(t.life(P1), 18);
    assert!(renowned(&t, blade));
    assert_eq!(t.counters(blade, counters::PLUS1), 1);
    assert_eq!(t.pt(blade), (3, 3));
    // A renowned creature's renown doesn't trigger again (intervening "if", CR 603.4).
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    declare_attack(&mut t, &[(blade, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        decision::Answer::Blockers(vec![]),
    );
    to_step(&mut t, Step::CombatDamage);
    t.settle();
    assert_eq!(on_stack(&t, "Renown"), 0);
    t.resolve_all();
    assert_eq!(t.counters(blade, counters::PLUS1), 1);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn renown_triggers_only_on_combat_damage_to_a_player() {
    cr!("702.112a");
    ruling!(
        "Topan Freeblade",
        "Renown won’t trigger when a creature deals combat damage to a planeswalker or another creature. It also won’t trigger when a creature deals noncombat damage to a player."
    );
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Topan Freeblade");
    let wall = t.battlefield(P1, "Wall of Stone");
    declare_attack(&mut t, &[(blade, Entity::Player(P1))]);
    finish_combat(&mut t, P1, &[(wall, blade)]);
    assert!(!renowned(&t, blade));
    // Attacking a planeswalker.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let jace = t.battlefield(P1, "Jace Beleren");
    declare_attack(&mut t, &[(blade, Entity::Object(jace))]);
    finish_combat(&mut t, P1, &[]);
    assert!(!renowned(&t, blade));
    assert_eq!(t.counters(blade, counters::PLUS1), 0);
}

#[test]
fn renowned_is_a_designation_other_abilities_can_see() {
    cr!("702.112b");
    assert_supported_card("Goblin Glory Chaser");
    let mut t = TestGame::new(2);
    // Goblin Glory Chaser: 1/1 renown 1; "As long as this creature is renowned, it has
    // menace."
    let goblin = t.battlefield(P0, "Goblin Glory Chaser");
    assert!(!has(&t, goblin, KeywordKind::Menace));
    hit(&mut t, goblin);
    assert!(renowned(&t, goblin));
    assert!(has(&t, goblin, KeywordKind::Menace));
}

#[test]
fn renowned_isnt_an_ability() {
    cr!("702.112b");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Goblin Glory Chaser");
    hit(&mut t, goblin);
    // Losing all abilities doesn't make it stop being renowned.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(goblin)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(!has(&t, goblin, KeywordKind::Renown));
    assert!(renowned(&t, goblin));
}

#[test]
fn renowned_isnt_copiable() {
    cr!("702.112b");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Topan Freeblade");
    hit(&mut t, blade);
    assert!(renowned(&t, blade));
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.advance_to(P0, Step::PostcombatMain);
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[Entity::Object(t.g.current(blade))]);
    t.resolve_all();
    let copy = t.g.current(clone);
    assert_eq!(t.obj(copy).chars.name, "Topan Freeblade");
    assert!(has(&t, copy, KeywordKind::Renown));
    assert!(!renowned(&t, copy));
    assert_eq!(t.pt(copy), (2, 2));
}

#[test]
fn a_permanent_stays_renowned_until_it_leaves_the_battlefield() {
    cr!("702.112b");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Topan Freeblade");
    hit(&mut t, blade);
    assert!(renowned(&t, blade));
    t.advance_to(P1, Step::Upkeep);
    assert!(renowned(&t, blade), "still renowned on later turns");
    // It leaves and returns: a new object that isn't renowned (CR 400.7).
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    let shift = t.hand(P0, "Cloudshift");
    let now = t.g.current(blade);
    t.cast(P0, shift).target(now).go();
    t.resolve_all();
    assert!(t.on_battlefield(blade));
    assert!(!renowned(&t, blade));
    assert_eq!(t.counters(blade, counters::PLUS1), 0);
    // And it can become renowned again (once it's no longer summoning sick).
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    hit(&mut t, blade);
    assert!(renowned(&t, blade));
}

#[test]
fn if_it_leaves_before_renown_resolves_it_doesnt_become_renowned() {
    cr!("702.112b");
    ruling!(
        "Valeron Wardens",
        "If a renown ability triggers, but the creature leaves the battlefield before that ability resolves, the creature doesn’t become renowned. Any ability that triggers “whenever a creature becomes renowned” won’t trigger."
    );
    assert_supported_card("Valeron Wardens");
    let mut t = TestGame::new(2);
    // Valeron Wardens: "Whenever a creature you control becomes renowned, draw a card."
    t.battlefield(P0, "Valeron Wardens");
    let blade = t.battlefield(P0, "Topan Freeblade");
    declare_attack(&mut t, &[(blade, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        decision::Answer::Blockers(vec![]),
    );
    to_step(&mut t, Step::CombatDamage);
    t.settle();
    assert_eq!(on_stack(&t, "Renown"), 1);
    let hand = t.hand_size(P0);
    // In response, it's destroyed.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(blade).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Topan Freeblade"));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn abilities_can_trigger_when_a_creature_becomes_renowned() {
    cr!("702.112b");
    let mut t = TestGame::new(2);
    let wardens = t.battlefield(P0, "Valeron Wardens");
    let blade = t.battlefield(P0, "Topan Freeblade");
    let hand = t.hand_size(P0);
    declare_attack(
        &mut t,
        &[(wardens, Entity::Player(P1)), (blade, Entity::Player(P1))],
    );
    finish_combat(&mut t, P1, &[]);
    // Both became renowned: two cards drawn.
    assert!(renowned(&t, wardens) && renowned(&t, blade));
    assert_eq!(t.counters(wardens, counters::PLUS1), 2);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn spells_can_check_whether_a_creature_is_renowned() {
    cr!("702.112b");
    assert_supported_card("Enshrouding Mist");
    let mut t = TestGame::new(2);
    // "Target creature gets +1/+1 until end of turn. Prevent all damage that would be
    // dealt to it this turn. If it's renowned, untap it."
    let blade = t.battlefield(P0, "Topan Freeblade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    hit(&mut t, blade);
    t.g.tap(blade);
    t.g.tap(bears);
    t.lands(P0, "Plains", 2);
    let a = t.hand(P0, "Enshrouding Mist");
    t.cast(P0, a).target(blade).go();
    t.resolve_all();
    assert!(!t.obj_now(blade).tapped);
    let b = t.hand(P0, "Enshrouding Mist");
    t.cast(P0, b).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn with_several_instances_only_the_first_to_resolve_does_anything() {
    cr!("702.112c");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Topan Freeblade");
    gain(&mut t, P0, blade, Keyword::with_n(KeywordKind::Renown, 2));
    declare_attack(&mut t, &[(blade, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        decision::Answer::Blockers(vec![]),
    );
    to_step(&mut t, Step::CombatDamage);
    t.settle();
    // Each triggers.
    assert_eq!(on_stack(&t, "Renown"), 2);
    let top = *t.g.stack.last().unwrap();
    let first_n = if on_stack_named(&t, top, "Renown 2") { 2 } else { 1 };
    t.resolve();
    assert!(renowned(&t, blade));
    assert_eq!(t.counters(blade, counters::PLUS1), first_n);
    // The other one does nothing.
    t.resolve_all();
    assert_eq!(t.counters(blade, counters::PLUS1), first_n);
}

fn on_stack_named(t: &TestGame, id: ObjectId, text: &str) -> bool {
    t.g.obj(id).stack.as_deref().is_some_and(|si| match &si.kind {
        mtg_engine::object::StackKind::Triggered { ability, .. } => ability.text == text,
        _ => false,
    })
}
