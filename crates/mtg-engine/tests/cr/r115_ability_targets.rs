//! CR 115.1: "target [something]" describes what can be chosen. Spells and abilities on
//! the stack are different objects: "target activated or triggered ability" (Stifle) is
//! never a spell, "target activated ability" (Squelch) is never a triggered ability, and
//! "target spell, activated ability, or triggered ability" (Disallow) is any of them.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The legal choices for the first target of the real spell `name` if `p` cast it now.
fn spell_targets(t: &mut TestGame, p: PlayerId, name: &str) -> Vec<Entity> {
    assert!(card(name).is_fully_supported(), "{name}");
    let spell = t.hand(p, name);
    t.g.recompute();
    let spec = t.g.spell_body(spell).targets[0].clone();
    let ctx = mtg_engine::eval::Ctx::new(Some(spell), p);
    t.g.legal_target_candidates(&spec, &ctx, spell)
}

#[test]
fn ability_target_phrases_describe_only_those_objects_on_the_stack() {
    cr!("115.1a", "115.2", "113.3b", "113.3c");
    let mut t = TestGame::new(2);
    // A triggered ability: Soul Warden's ("Whenever another creature enters, you gain 1
    // life").
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    let triggered = Entity::Object(*t.g.stack.last().unwrap());
    // An activated ability: Prodigal Pyromancer's ("{T}: This creature deals 1 damage to
    // any target").
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    let activated = Entity::Object(*t.g.stack.last().unwrap());
    // A spell.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = Entity::Object(t.cast(P0, bolt).target(P1).go());
    assert_eq!(t.stack_len(), 3);

    let mut sorted = |name: &str| {
        let mut v = spell_targets(&mut t, P1, name);
        v.sort_by_key(|e| format!("{e:?}"));
        v
    };
    let sort = |mut v: Vec<Entity>| {
        v.sort_by_key(|e| format!("{e:?}"));
        v
    };
    // Stifle: "Counter target activated or triggered ability."
    assert_eq!(sorted("Stifle"), sort(vec![triggered, activated]));
    // Squelch: "Counter target activated ability."
    assert_eq!(sorted("Squelch"), vec![activated]);
    // Disallow: "Counter target spell, activated ability, or triggered ability."
    assert_eq!(sorted("Disallow"), sort(vec![triggered, activated, spell]));
    // Cancel: "Counter target spell."
    assert_eq!(sorted("Cancel"), vec![spell]);
}

#[test]
fn stifle_counters_an_ability_but_cant_counter_a_spell() {
    cr!("115.1a", "701.6a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    // With only a spell on the stack, Stifle has no legal target and can't be cast.
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    assert!(t.cast(P1, stifle).target(spell).try_go().is_err());
    assert!(t.in_hand(P1, "Stifle"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // An activated ability on the stack can be targeted, and countering it means none of
    // its effects happen.
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    let ability = *t.g.stack.last().unwrap();
    t.cast(P1, stifle).target(ability).go();
    t.resolve();
    assert!(t.g.stack.is_empty());
    assert!(t.in_graveyard(P1, "Stifle"));
    assert_eq!(t.life(P1), 17);
}
