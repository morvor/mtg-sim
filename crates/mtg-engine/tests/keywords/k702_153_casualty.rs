//! CR 702.153 Casualty.

use crate::common_k702_153_167::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn casualty_sacrifices_a_creature_to_copy_the_spell_with_new_targets() {
    cr!("702.153", "702.153a");
    ruling!(
        "Light 'Em Up",
        "If you pay the casualty cost of a spell, the copy will resolve before the original spell."
    );
    ruling!(
        "Make Disappear",
        "Casualty N means “As an additional cost to cast this spell, you may sacrifice a creature with power N or greater.”"
    );
    assert_supported("Light 'Em Up");
    let mut t = TestGame::new(2);
    // Light 'Em Up: casualty 2; 2 damage to target creature or planeswalker.
    let giant = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Light 'Em Up");
    pay_with(&mut t, P0, &[giant]);
    t.cast(P0, spell).target(b1).go();
    // The creature was sacrificed as the spell was cast (an additional cost).
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(optional_costs_offered(&t, P0), vec!["casualty#1".to_string()]);
    t.settle();
    // "When you cast this spell, if a casualty cost was paid for it, copy it."
    assert_eq!(triggers_named(&t, "Casualty").len(), 1);
    // The copy may have a new target.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b2)]);
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Light 'Em Up"), 1);
    // The copy is on top: it resolves first.
    t.resolve();
    assert!(!t.on_battlefield(b2));
    assert!(t.on_battlefield(b1));
    t.resolve_all();
    assert!(!t.on_battlefield(b1));
    // The copy wasn't cast.
    assert_eq!(t.g.history.spells_cast.len(), 1);
}

#[test]
fn casualty_is_optional_and_needs_a_creature_with_enough_power() {
    cr!("702.153a");
    ruling!(
        "Make Disappear",
        "You may sacrifice only one creature to pay a spell's casualty cost, and you copy the spell only once."
    );
    // Not paying: no copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Light 'Em Up");
    pay_optional(&mut t, P0, false);
    t.cast(P0, spell).target(b1).go();
    t.settle();
    assert!(triggers_named(&t, "Casualty").is_empty());
    t.resolve_all();
    assert_eq!(named(&t, P0, "Hill Giant").len(), 1);
    // Only creatures with power 2 or greater can be sacrificed for casualty 2: with only a
    // 1/1, the cost isn't offered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging Goblin");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Light 'Em Up");
    t.cast(P0, spell).target(b1).go();
    assert!(optional_costs_offered(&t, P0).is_empty());
    assert_eq!(named(&t, P0, "Raging Goblin").len(), 1);
    // A creature with exactly power 2 is enough; only one creature is sacrificed and
    // the spell is copied once.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 5);
    // Join the Maestros: casualty 2; create a 4/3 Ogre Warrior token.
    let spell = t.hand(P0, "Join the Maestros");
    pay_with(&mut t, P0, &[bears]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(other));
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn each_instance_of_casualty_is_paid_separately_and_triggers_for_its_own_payment() {
    cr!("702.153b");
    assert_supported("Silverquill, the Disputant");
    // Silverquill gives Light 'Em Up (casualty 2) a second instance: casualty 1.
    for (pay2, pay1, copies) in [
        (false, false, 0usize),
        (true, false, 1),
        (false, true, 1),
        (true, true, 2),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Silverquill, the Disputant");
        let giant = t.battlefield(P0, "Hill Giant");
        let goblin = t.battlefield(P0, "Raging Goblin");
        let targets: Vec<ObjectId> = (0..3)
            .map(|_| t.battlefield(P1, "Grizzly Bears"))
            .collect();
        t.lands(P0, "Mountain", 2);
        let spell = t.hand(P0, "Light 'Em Up");
        if pay2 {
            pay_with(&mut t, P0, &[giant]);
        } else {
            pay_optional(&mut t, P0, false);
        }
        if pay1 {
            pay_with(&mut t, P0, &[goblin]);
        } else {
            pay_optional(&mut t, P0, false);
        }
        let s = t.cast(P0, spell).target(targets[0]).go();
        assert_eq!(t.g.obj(s).chars.keyword_count(KeywordKind::Casualty), 2);
        t.settle();
        assert_eq!(triggers_named(&t, "Casualty").len(), copies);
        for (i, _) in (0..copies).enumerate() {
            t.answer_yes(P0, true);
            t.answer_targets(P0, &[Entity::Object(targets[i + 1])]);
        }
        t.resolve_all();
        let dead = targets.iter().filter(|b| !t.on_battlefield(**b)).count();
        assert_eq!(dead, 1 + copies);
        assert_eq!(named(&t, P0, "Hill Giant").is_empty(), pay2);
        assert_eq!(named(&t, P0, "Raging Goblin").is_empty(), pay1);
    }
}

#[test]
fn the_first_instant_or_sorcery_spell_each_turn_has_casualty() {
    cr!("702.153a");
    ruling!(
        "Anhelo, the Painter",
        "The copy of the spell is created on the stack, so it’s not “cast.”"
    );
    assert_supported("Anhelo, the Painter");
    // Anhelo: "The first instant or sorcery spell you cast each turn has casualty 2."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anhelo, the Painter");
    let giant = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 1);
    // A creature spell first: it isn't an instant or sorcery.
    let elves = t.hand(P0, "Llanowar Elves");
    let s = t.cast(P0, elves).go();
    assert_eq!(t.g.obj(s).chars.keyword_count(KeywordKind::Casualty), 0);
    t.resolve_all();
    // The first instant: casualty 2.
    let bolt = t.hand(P0, "Lightning Bolt");
    pay_with(&mut t, P0, &[giant]);
    let s = t.cast(P0, bolt).target(b1).go();
    assert_eq!(t.g.obj(s).chars.keyword_count(KeywordKind::Casualty), 1);
    t.settle();
    assert_eq!(triggers_named(&t, "Casualty").len(), 1);
    // The copy (not cast) doesn't have casualty itself.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b2)]);
    t.resolve();
    let copy = t
        .g
        .stack
        .iter()
        .copied()
        .find(|id| t.g.obj(*id).kind == mtg_engine::object::ObjKind::SpellCopy)
        .expect("a copy");
    assert_eq!(t.g.obj(copy).chars.keyword_count(KeywordKind::Casualty), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(b1) && !t.on_battlefield(b2));
    // The second one this turn doesn't.
    let bolt2 = t.hand(P0, "Lightning Bolt");
    let s = t.cast(P0, bolt2).target(P1).go();
    assert_eq!(t.g.obj(s).chars.keyword_count(KeywordKind::Casualty), 0);
}
