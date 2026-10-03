//! CR 702.189 Firebending (`src/kw/firebending.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn red(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.count(ManaType::R)
}

/// Declares `attackers` attacking player 1 and advances to the declare attackers step
/// with the attack triggers on the stack.
fn declare(t: &mut TestGame, attackers: &[ObjectId]) {
    let decl: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(decl));
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
}

#[test]
fn firebending_cards_compile() {
    assert_supported(&[
        "Fire Sages",
        "Zuko, Exiled Prince",
        "Firebending Student",
        "Rough Rhino Cavalry",
        "Sozin's Comet",
    ]);
}

#[test]
fn attacking_adds_red_mana_that_lasts_until_end_of_combat() {
    cr!("702.189a");
    ruling!(
        "Fire Sages",
        "Mana from firebending abilities isn't lost until you leave combat and go to your second main phase."
    );
    ruling!(
        "Fire Sages",
        "Firebending abilities aren't mana abilities. They use the stack and can be responded to."
    );
    ruling!(
        "Fire Sages",
        "Any other mana you add during combat will still be lost as normal when moving between steps of combat."
    );
    ruling!(
        "Fire Sages",
        "\"Firebending N\" is a keyword that represents the ability \"Whenever this creature attacks, add N {R}."
    );
    let mut t = TestGame::new(2);
    // Zuko, Exiled Prince: "Firebending 3".
    let zuko = t.battlefield(P0, "Zuko, Exiled Prince");
    declare(&mut t, &[zuko]);
    // The trigger is on the stack.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(red(&t, P0), 0);
    t.resolve_all();
    assert_eq!(red(&t, P0), 3);
    // Other mana added during combat is lost as the step ends.
    add_mana(&mut t, P0, &[ManaType::G]);
    // It stays as combat's steps end.
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(red(&t, P0), 3);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 0);
    assert_eq!(t.life(P1), 16);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(red(&t, P0), 3);
    // It's lost as the combat phase ends.
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(red(&t, P0), 0);
}

#[test]
fn the_mana_can_be_spent_during_combat() {
    cr!("702.189a");
    ruling!(
        "Fire Sages",
        "It can be used at any time during combat, even after combat damage has been dealt."
    );
    // Fire Sages: "Firebending 1. {1}{R}{R}: Put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let sages = t.battlefield(P0, "Fire Sages");
    let zuko = t.battlefield(P0, "Zuko, Exiled Prince");
    declare(&mut t, &[sages, zuko]);
    t.resolve_all();
    assert_eq!(red(&t, P0), 4);
    t.advance_to(P0, Step::EndOfCombat);
    t.activate(P0, sages, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(sages, "+1/+1"), 1);
    assert_eq!(red(&t, P0), 1);
}

#[test]
fn multiple_instances_trigger_separately() {
    cr!("702.189a");
    ruling!(
        "Fire Sages",
        "Multiple instances of firebending on the same creature trigger separately"
    );
    // Sozin's Comet: "Each creature you control gains firebending 5 until end of turn."
    let mut t = TestGame::new(2);
    let sages = t.battlefield(P0, "Fire Sages");
    t.lands(P0, "Mountain", 5);
    let comet = t.hand(P0, "Sozin's Comet");
    t.cast(P0, comet).go();
    t.resolve_all();
    assert_eq!(
        t.obj(sages)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Firebending)
            .count(),
        2
    );
    declare(&mut t, &[sages]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(red(&t, P0), 6);
}

#[test]
fn firebending_x_is_determined_as_it_resolves() {
    cr!("702.189a");
    // Firebending Student (1/2): "Firebending X, where X is this creature's power."
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Firebending Student");
    declare(&mut t, &[student]);
    // In response, it gets +2/+0.
    run(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(student)],
    );
    t.resolve_all();
    assert_eq!(red(&t, P0), 3);
}

#[test]
fn whenever_you_firebend() {
    cr!("702.189b");
    ruling!(
        "Avatar Aang // Aang, Master of Elements",
        "An ability that triggers \"whenever you firebend\" triggers whenever a firebending ability you control resolves."
    );
    let def = custom_card(
        "Fire Watcher",
        "{1}{R}",
        "Enchantment",
        None,
        "Whenever you firebend, you gain 1 life.",
    );
    let mut t = TestGame::new(2);
    put(&mut t, P0, def, Zone::Battlefield);
    let sages = t.battlefield(P0, "Fire Sages");
    let zuko = t.battlefield(P0, "Zuko, Exiled Prince");
    declare(&mut t, &[sages, zuko]);
    // Not when the abilities trigger: when they resolve.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The first firebending ability resolved; "whenever you firebend" triggered.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // An opponent's firebending doesn't trigger it.
    let mut t = TestGame::new(2);
    let def = custom_card(
        "Fire Watcher",
        "{1}{R}",
        "Enchantment",
        None,
        "Whenever you firebend, you gain 1 life.",
    );
    put(&mut t, P1, def, Zone::Battlefield);
    let sages = t.battlefield(P0, "Fire Sages");
    declare(&mut t, &[sages]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}
