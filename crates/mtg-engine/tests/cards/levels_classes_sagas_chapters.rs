//! Saga chapter abilities with flavor words (CR 714.2b, 207.2d) and "When you next cast
//! [a spell] this turn, [effect]" delayed triggers (CR 603.7b) in chapters and spells.

use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn add_lore(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.add_counters(Entity::Object(id), counters::LORE, n, None);
    t.g.flush_events();
}

fn bolt(t: &mut TestGame, p: PlayerId, target: PlayerId) {
    t.lands(p, "Mountain", 1);
    let b = t.hand(p, "Lightning Bolt");
    t.cast(p, b).target(Entity::Player(target)).go();
    t.resolve_all();
}

#[test]
fn chapter_abilities_with_flavor_words_compile_without_them() {
    cr!("207.2d", "714.2b");
    for c in [
        "Summon: Primal Odin",
        "Summon: G.F. Cerberus",
        "Summon: Shiva",
        "Summon: Anima",
    ] {
        let def = mtg_engine::card::card(c);
        assert!(def.unsupported_text().is_empty(), "{c}: {:?}", def.unsupported_text());
    }
    // A clause before a dash that isn't a flavor word stays part of the effect.
    let def = mtg_engine::card::card("Genesis of the Daleks");
    assert!(def
        .unsupported_text()
        .iter()
        .any(|u| u.contains("villainous choice")));
}

#[test]
fn flavor_worded_chapters_do_what_they_say() {
    cr!("207.2d", "714.2b", "714.3a");
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(victim)]);
    // I — Gungnir — Destroy target creature an opponent controls.
    let odin = t.enter(P0, "Summon: Primal Odin");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // III — Hall of Sorrow — Draw two cards. Each player loses 2 life.
    let hand = t.hand_size(P0);
    add_lore(&mut t, odin, 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn next_instant_or_sorcery_spell_is_copied() {
    cr!("603.7b", "707.10");
    ruling!(
        "Howl of the Horde",
        "Howl of the Horde creates a delayed triggered ability that triggers when you cast your next instant or sorcery spell that turn."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let howl = t.hand(P0, "Howl of the Horde");
    t.cast(P0, howl).go();
    t.resolve_all();
    // The next one is copied: 3 + 3.
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 14);
    // Only the next one.
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 11);
}

#[test]
fn raid_adds_a_second_delayed_trigger() {
    cr!("603.7b", "707.10");
    ruling!(
        "Howl of the Horde",
        "The raid ability creates a second, identical delayed triggered ability."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 18);
    t.lands(P0, "Mountain", 3);
    let howl = t.hand(P0, "Howl of the Horde");
    t.cast(P0, howl).go();
    t.resolve_all();
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 18 - 9);
}

#[test]
fn the_delayed_trigger_ends_with_the_turn() {
    cr!("603.7b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let howl = t.hand(P0, "Howl of the Horde");
    t.cast(P0, howl).go();
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_creature_spell_is_not_an_instant_or_sorcery_spell() {
    cr!("603.7b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let howl = t.hand(P0, "Howl of the Horde");
    t.cast(P0, howl).go();
    t.resolve_all();
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Still waiting for an instant or sorcery.
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn chapter_copies_the_next_spell_twice() {
    cr!("714.2b", "603.7b", "707.10");
    let mut t = TestGame::new(2);
    let saga = t.enter(P0, "Summon: G.F. Cerberus");
    t.resolve_all();
    // III — Triple — copy it twice.
    add_lore(&mut t, saga, 2);
    t.resolve_all();
    bolt(&mut t, P0, P1);
    // II and III both triggered: 1 + 1 + 2 copies.
    assert_eq!(t.life(P1), 20 - 3 * 4);
}

#[test]
fn next_creature_spell_enters_with_an_additional_counter() {
    cr!("714.2b", "603.7b", "614.1c");
    let mut t = TestGame::new(2);
    let saga = t.enter(P0, "Kumano Faces Kakkazan // Etching of Kumano");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    t.lands(P0, "Forest", 4);
    let first = t.hand(P0, "Grizzly Bears");
    t.cast(P0, first).go();
    t.resolve_all();
    let second = t.hand(P0, "Grizzly Bears");
    t.cast(P0, second).go();
    t.resolve_all();
    let mut pts: Vec<(i32, i32)> = t
        .named_on_battlefield("Grizzly Bears")
        .into_iter()
        .map(|b| t.pt(b))
        .collect();
    pts.sort();
    // Only the next creature spell.
    assert_eq!(pts, vec![(2, 2), (3, 3)]);
}

#[test]
fn the_next_creature_spell_gains_haste_and_keeps_it_as_a_permanent() {
    cr!("714.2b", "603.7b", "400.7a");
    let mut t = TestGame::new(2);
    // II, III — Gestalt Mode — When you next cast a creature spell this turn, it gains
    // haste until end of turn.
    let saga = t.enter(P0, "Summon: Brynhildr");
    t.resolve_all();
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    t.lands(P0, "Forest", 4);
    let first = t.hand(P0, "Grizzly Bears");
    t.cast(P0, first).go();
    t.resolve_all();
    let second = t.hand(P0, "Grizzly Bears");
    t.cast(P0, second).go();
    t.resolve_all();
    let hasty: Vec<bool> = t
        .named_on_battlefield("Grizzly Bears")
        .into_iter()
        .map(|b| t.obj_now(b).has_keyword(KeywordKind::Haste))
        .collect();
    // Only the next creature spell.
    assert_eq!(hasty.iter().filter(|h| **h).count(), 1, "{hasty:?}");
    assert_eq!(hasty.len(), 2);
}

#[test]
fn the_next_instant_cast_from_hand_gains_rebound() {
    cr!("603.7b", "702.88a");
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset Transcendent");
    t.set_step(P0, Step::PrecombatMain);
    // −2: When you next cast an instant or sorcery spell from your hand this turn, it
    // gains rebound.
    let minus_two = t
        .obj(narset)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.starts_with("-2"))
        .unwrap();
    t.activate(P0, narset, minus_two, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(narset, "loyalty"), 4);
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt"));
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
    // Only the next one.
    bolt(&mut t, P0, P1);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn a_cast_trigger_from_your_hand_means_the_spell_was_cast_from_there() {
    // "Whenever you cast an instant spell from your hand, it gains rebound": the spell is
    // on the stack as the ability triggers (CR 601.2a, 601.2i).
    cr!("601.2a", "601.2i", "702.88a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ojer Pakpatiq, Deepest Epoch // Temple of Cyclical Time");
    t.set_step(P0, Step::PrecombatMain);
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt"));
    // A sorcery isn't an instant spell.
    t.lands(P0, "Mountain", 1);
    let sorcery = t.hand(P0, "Lava Spike");
    t.cast(P0, sorcery).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P0, "Lava Spike"));
}
