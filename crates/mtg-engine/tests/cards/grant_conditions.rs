//! Conditional grants (`src/oracle/patterns/grant_conditions.rs`, `grant_grammar.rs`):
//! "~ has/gets ... as long as [condition]" with conditions about this turn's history
//! (spells cast, crimes, sacrifices, surveil), cards in exile, how the permanent entered
//! and how many creatures block it; and a list of predicates each with its own
//! condition (CR 611.3a: a conditional static ability applies exactly while its
//! condition holds).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

fn activated_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, ability::AbilityKind::Activated(_)))
        .count()
}

#[test]
fn tek_each_predicate_has_its_own_condition() {
    cr!("611.3a");
    assert_supported(&["Tek"]);
    let mut t = TestGame::new(2);
    let tek = t.battlefield(P0, "Tek");
    t.settle();
    assert_eq!(t.pt(tek), (2, 2));
    assert!(!has(&t, tek, KeywordKind::Flying));
    t.battlefield(P0, "Plains");
    t.battlefield(P0, "Forest");
    t.settle();
    assert_eq!(t.pt(tek), (2, 4));
    assert!(has(&t, tek, KeywordKind::Trample));
    assert!(!has(&t, tek, KeywordKind::Flying));
    assert!(!has(&t, tek, KeywordKind::FirstStrike));
    t.battlefield(P0, "Swamp");
    t.battlefield(P0, "Island");
    t.battlefield(P0, "Mountain");
    t.settle();
    assert_eq!(t.pt(tek), (4, 4));
    assert!(has(&t, tek, KeywordKind::Flying));
    assert!(has(&t, tek, KeywordKind::FirstStrike));
    // An opponent's lands don't count.
    let mut t = TestGame::new(2);
    let tek = t.battlefield(P0, "Tek");
    t.battlefield(P1, "Swamp");
    t.settle();
    assert_eq!(t.pt(tek), (2, 2));
}

#[test]
fn tribal_golem_grants_quoted_ability_while_you_control_a_zombie() {
    cr!("611.3a", "113.6");
    assert_supported(&["Tribal Golem"]);
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Tribal Golem");
    t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(activated_count(&t, golem), 0);
    assert!(!has(&t, golem, KeywordKind::Trample));
    let zombie = t.battlefield(P0, "Walking Corpse");
    t.battlefield(P0, "Raging Goblin");
    t.settle();
    assert_eq!(activated_count(&t, golem), 1);
    assert!(has(&t, golem, KeywordKind::Haste));
    assert!(!has(&t, golem, KeywordKind::Trample));
    // "{B}: Regenerate this creature": the Golem gets the shield and survives.
    t.lands(P0, "Swamp", 1);
    t.activate(P0, golem, 0, &[]).unwrap();
    t.resolve();
    t.g.destroy_all(vec![golem], None, false);
    t.settle();
    assert!(t.on_battlefield(golem));
    // Without a Zombie, the ability is gone.
    t.g.destroy_all(vec![zombie], None, false);
    t.settle();
    assert_eq!(activated_count(&t, golem), 0);
}

#[test]
fn stoic_sphinx_hexproof_until_you_cast_a_spell() {
    cr!("611.3a", "601.2i");
    assert_supported(&["Stoic Sphinx"]);
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Stoic Sphinx");
    t.settle();
    assert!(has(&t, sphinx, KeywordKind::Hexproof));
    // An opponent's spell doesn't matter.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert!(has(&t, sphinx, KeywordKind::Hexproof));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    // Cast (not yet resolved) is enough.
    assert!(!has(&t, sphinx, KeywordKind::Hexproof));
}

#[test]
fn leapfrog_flies_after_you_cast_an_instant_or_sorcery() {
    cr!("611.3a");
    assert_supported(&["Leapfrog"]);
    let mut t = TestGame::new(2);
    let frog = t.battlefield(P0, "Leapfrog");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    assert!(!has(&t, frog, KeywordKind::Flying), "a creature spell");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert!(has(&t, frog, KeywordKind::Flying));
}

#[test]
fn omenport_vigilante_double_strike_after_a_crime() {
    cr!("611.3a", "700.13");
    ruling!(
        "Omenport Vigilante",
        "As soon as you're finished casting the spell, activating the ability"
    );
    assert_supported(&["Omenport Vigilante", "Slickshot Vault-Buster"]);
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Omenport Vigilante");
    let s = t.battlefield(P0, "Slickshot Vault-Buster");
    t.settle();
    assert!(!has(&t, v, KeywordKind::DoubleStrike));
    assert_eq!(t.pt(s).0, 1);
    // Targeting yourself isn't a crime.
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P0).go();
    t.resolve();
    assert!(!has(&t, v, KeywordKind::DoubleStrike));
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert!(has(&t, v, KeywordKind::DoubleStrike));
    assert_eq!(t.pt(s).0, 3);
}

#[test]
fn goblin_blast_runner_after_you_sacrificed_a_permanent() {
    cr!("611.3a", "701.21a");
    ruling!(
        "Goblin Blast-Runner",
        "doesn't need to have been on the battlefield at the time the permanent was sacrificed"
    );
    assert_supported(&["Goblin Blast-Runner"]);
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Mountain");
    // An opponent's sacrifice doesn't count.
    let theirs = t.battlefield(P1, "Mountain");
    t.g.sacrifice(theirs, P1);
    t.g.sacrifice(land, P0);
    t.settle();
    // It wasn't on the battlefield then.
    let g = t.battlefield(P0, "Goblin Blast-Runner");
    t.settle();
    assert_eq!(t.pt(g), (3, 2));
    assert!(has(&t, g, KeywordKind::Menace));
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Blast-Runner");
    let theirs = t.battlefield(P1, "Mountain");
    t.g.sacrifice(theirs, P1);
    t.settle();
    assert_eq!(t.pt(g), (1, 2));
    assert!(!has(&t, g, KeywordKind::Menace));
}

#[test]
fn warden_of_the_beyond_counts_opponents_exiled_cards_once() {
    cr!("611.3a");
    ruling!("Warden of the Beyond", "can't get more than +2/+2");
    assert_supported(&["Warden of the Beyond"]);
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Warden of the Beyond");
    t.exile(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(w), (2, 2), "your own exiled card");
    t.exile(P1, "Grizzly Bears");
    t.exile(P1, "Lightning Bolt");
    t.settle();
    assert_eq!(t.pt(w), (4, 4));
}

#[test]
fn the_tarrasque_has_haste_and_ward_only_if_cast() {
    cr!("611.3a", "601.2i", "702.21a");
    assert_supported(&["The Tarrasque"]);
    let mut t = TestGame::new(2);
    let put = t.battlefield(P0, "The Tarrasque");
    t.settle();
    assert!(!has(&t, put, KeywordKind::Haste));
    assert!(!has(&t, put, KeywordKind::Ward));
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 9);
    let c = t.hand(P0, "The Tarrasque");
    t.cast(P0, c).go();
    t.resolve();
    let tq = t.named_on_battlefield("The Tarrasque")[0];
    assert!(has(&t, tq, KeywordKind::Haste));
    let ward = t.obj_now(tq).chars.keyword(KeywordKind::Ward).cloned();
    assert_eq!(
        ward.and_then(|k| k.cost)
            .and_then(|c| c.mana)
            .map(|m| m.mana_value()),
        Some(10)
    );
}

#[test]
fn rampaging_cyclops_weaker_while_two_creatures_block_it() {
    cr!("611.3a", "509.1a");
    assert_supported(&["Rampaging Cyclops"]);
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Rampaging Cyclops");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(c), (4, 4));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(c, Entity::Player(P1))], &[(a, c), (b, c)]);
    // 2 power: only one Bear can be dealt lethal damage.
    let dead = [a, b].iter().filter(|x| !t.on_battlefield(**x)).count();
    assert_eq!(dead, 1);
}

#[test]
fn darkblade_agent_after_you_surveil() {
    cr!("611.3a", "701.25a", "113.6");
    assert_supported(&["Darkblade Agent", "Notion Rain"]);
    let mut t = TestGame::new(2);
    let agent = t.battlefield(P0, "Darkblade Agent");
    t.settle();
    assert!(!has(&t, agent, KeywordKind::Deathtouch));
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Island", 1);
    let nr = t.hand(P0, "Notion Rain");
    t.cast(P0, nr).go();
    t.resolve_all();
    assert!(has(&t, agent, KeywordKind::Deathtouch));
    // "Whenever this creature deals combat damage to a player, you draw a card."
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(agent, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
