//! CR 111.10: predefined tokens.

use super::r105_util::*;
use super::r111_tokens::run_text;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::{ManaRestriction, ManaType};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

use ManaType::*;

/// Asserts a colorless artifact token with one subtype, named "<subtype> Token".
fn assert_artifact_token(t: &TestGame, id: ObjectId, subtype: &str) {
    let c = &t.obj(id).chars;
    assert!(t.obj(id).is_token());
    assert_eq!(c.name.as_str(), format!("{subtype} Token"));
    assert_eq!(c.colors, ColorSet::NONE);
    assert_eq!(c.card_types, [CardType::Artifact].into_iter().collect());
    assert!(c.has_subtype(subtype));
    assert_eq!(c.subtypes.len(), 1);
    assert!(c.power.is_none() && c.toughness.is_none());
}

/// The oracle text of each ability of an object.
fn ability_texts(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj(id)
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect()
}

/// A role token attached to `creature` that `p` controls.
fn role_on(t: &TestGame, p: PlayerId, creature: ObjectId) -> ObjectId {
    let creature = t.g.current(creature);
    *tokens_of(t, p)
        .iter()
        .find(|x| t.obj(**x).attached_to == Some(Entity::Object(creature)))
        .expect("no role attached")
}

fn assert_role(t: &TestGame, role: ObjectId, name: &str) {
    let c = &t.obj(role).chars;
    assert_eq!(c.name.as_str(), name);
    assert_eq!(c.colors, ColorSet::NONE);
    assert_eq!(c.card_types, [CardType::Enchantment].into_iter().collect());
    assert!(c.has_subtype("Aura") && c.has_subtype("Role"));
    assert!(t.obj(role).has_keyword(KeywordKind::Enchant));
}

#[test]
fn treasure_tokens() {
    cr!("111.10", "111.10a");
    let mut t = TestGame::new(2);
    let strike = t.hand(P0, "Strike It Rich");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, strike).go();
    t.resolve();
    let tr = tokens_of(&t, P0);
    assert_eq!(tr.len(), 1);
    assert_artifact_token(&t, tr[0], "Treasure");
    assert_eq!(
        ability_texts(&t, tr[0]),
        vec!["{T}, Sacrifice this token: Add one mana of any color."]
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, tr[0], 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, B), 1);
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn food_tokens() {
    cr!("111.10", "111.10b");
    let mut t = TestGame::new(2);
    let food = run_text(&mut t, P0, "Create a Food token.")[0];
    assert_artifact_token(&t, food, "Food");
    t.lands(P0, "Plains", 2);
    t.activate(P0, food, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 23);
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn a_predefined_token_can_be_given_a_different_name() {
    cr!("111.10");
    let mut t = TestGame::new(2);
    // Urza's Hot Dog Stand: "Create two Food tokens named Hot Dog."
    let dogs = run_text(&mut t, P0, "Create two Food tokens named Hot Dog.");
    assert_eq!(dogs.len(), 2);
    for d in &dogs {
        let c = &t.obj(*d).chars;
        assert_eq!(c.name.as_str(), "Hot Dog");
        assert!(c.has_subtype("Food"));
        assert!(c.card_types.contains(CardType::Artifact));
    }
    t.lands(P0, "Plains", 2);
    t.activate(P0, dogs[0], 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn gold_tokens() {
    cr!("111.10", "111.10c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gild = t.hand(P0, "Gild");
    t.lands(P0, "Swamp", 4);
    t.cast(P0, gild).target(bears).go();
    t.resolve();
    let gold = tokens_of(&t, P0)[0];
    assert_artifact_token(&t, gold, "Gold");
    // It needs no {T}: it can be sacrificed for mana even while tapped.
    t.g.objects[gold.0 as usize].tapped = true;
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, gold, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, G), 1);
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn walker_tokens() {
    cr!("111.10", "111.10d");
    let mut t = TestGame::new(2);
    let w = run_text(&mut t, P0, "Create a Walker token.")[0];
    let c = &t.obj(w).chars;
    assert_eq!(c.name.as_str(), "Walker");
    assert_eq!(c.colors, cs("B"));
    assert_eq!(c.card_types, [CardType::Creature].into_iter().collect());
    assert!(c.has_subtype("Zombie"));
    assert_eq!(t.pt(w), (2, 2));
}

#[test]
fn shard_tokens() {
    cr!("111.10", "111.10e");
    let mut t = TestGame::new(2);
    let s = run_text(&mut t, P0, "Create a Shard token.")[0];
    let c = &t.obj(s).chars;
    assert_eq!(c.name.as_str(), "Shard Token");
    assert_eq!(c.colors, ColorSet::NONE);
    assert_eq!(c.card_types, [CardType::Enchantment].into_iter().collect());
    assert!(c.has_subtype("Shard"));
    let hand = t.hand_size(P0);
    t.lands(P0, "Island", 2);
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve();
    // Scry 1, then draw a card.
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::Scry { .. })));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn clue_tokens() {
    cr!("111.10", "111.10f");
    let mut t = TestGame::new(2);
    let clue = run_text(&mut t, P0, "Investigate.")[0];
    assert_artifact_token(&t, clue, "Clue");
    let hand = t.hand_size(P0);
    t.lands(P0, "Island", 2);
    t.activate(P0, clue, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn blood_tokens() {
    cr!("111.10", "111.10g");
    let mut t = TestGame::new(2);
    let blood = run_text(&mut t, P0, "Create a Blood token.")[0];
    assert_artifact_token(&t, blood, "Blood");
    let junk = t.hand(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.lands(P0, "Mountain", 1);
    t.answer_choose(P0, &[Entity::Object(junk)]);
    t.activate(P0, blood, 0, &[]).unwrap();
    t.resolve();
    // Discarded the Giant, drew a card.
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand);
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn powerstone_tokens() {
    cr!("111.10", "111.10h");
    let mut t = TestGame::new(2);
    let ps = run_text(&mut t, P0, "Create a tapped Powerstone token.")[0];
    assert_artifact_token(&t, ps, "Powerstone");
    assert!(t.obj(ps).tapped);
    t.g.objects[ps.0 as usize].tapped = false;
    t.activate(P0, ps, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, C), 1);
    let m = &t.g.player(P0).mana_pool.mana[0];
    assert_eq!(m.restriction, Some(ManaRestriction::NotNonartifactSpell));
    // That mana can't pay for Grizzly Bears (a nonartifact spell) ...
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // ... but can pay for an artifact spell.
    let spellbomb = t.hand(P0, "Bonesplitter");
    t.cast(P0, spellbomb).go();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn incubator_tokens_are_double_faced() {
    cr!("111.10", "111.10i");
    let mut t = TestGame::new(2);
    let inc = run_text(&mut t, P0, "Create an Incubator token.")[0];
    assert_artifact_token(&t, inc, "Incubator");
    assert_eq!(ability_texts(&t, inc), vec!["{2}: Transform this token."]);
    t.g.objects[inc.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 2);
    t.lands(P0, "Plains", 2);
    t.activate(P0, inc, 0, &[]).unwrap();
    t.resolve();
    t.g.recompute();
    let c = &t.obj(inc).chars;
    assert_eq!(c.name.as_str(), "Phyrexian Token");
    assert_eq!(c.colors, ColorSet::NONE);
    assert_eq!(
        c.card_types,
        [CardType::Artifact, CardType::Creature].into_iter().collect()
    );
    assert!(c.has_subtype("Phyrexian"));
    // A 0/0 with two +1/+1 counters.
    assert_eq!(t.pt(inc), (2, 2));
    t.settle();
    assert!(t.on_battlefield(inc));
}

#[test]
fn cursed_role_tokens() {
    cr!("111.10", "111.10j");
    let mut t = TestGame::new(2);
    let courtier = t.hand(P0, "Cursed Courtier");
    t.lands(P0, "Plains", 3);
    t.cast(P0, courtier).go();
    t.resolve();
    t.resolve();
    let c = t.named_on_battlefield("Cursed Courtier")[0];
    let role = role_on(&t, P0, c);
    assert_role(&t, role, "Cursed");
    // "Enchanted creature has base power and toughness 1/1."
    assert_eq!(t.pt(c), (1, 1));
}

#[test]
fn monster_role_tokens() {
    cr!("111.10", "111.10k");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let curse = t.hand(P0, "Curse of the Werefox");
    t.lands(P0, "Forest", 3);
    t.cast(P0, curse).target(bears).go();
    t.resolve_all();
    let role = role_on(&t, P0, bears);
    assert_role(&t, role, "Monster");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj(bears).has_keyword(KeywordKind::Trample));
}

#[test]
fn royal_role_tokens() {
    cr!("111.10", "111.10m");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let royal = run_text(
        &mut t,
        P0,
        "Create a Royal Role token attached to target creature you control.",
    );
    let _ = royal;
    let role = role_on(&t, P0, bears);
    assert_role(&t, role, "Royal");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj(bears).has_keyword(KeywordKind::Ward));
}

#[test]
fn sorcerer_role_tokens() {
    cr!("111.10", "111.10n");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_text(
        &mut t,
        P0,
        "Create a Sorcerer Role token attached to target creature you control.",
    );
    let role = role_on(&t, P0, bears);
    assert_role(&t, role, "Sorcerer");
    assert_eq!(t.pt(bears), (3, 3));
    // "Whenever this creature attacks, scry 1."
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::Scry { .. })));
}

#[test]
fn virtuous_role_tokens() {
    cr!("111.10", "111.10p");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_text(
        &mut t,
        P0,
        "Create a Virtuous Role token attached to target creature you control.",
    );
    let role = role_on(&t, P0, bears);
    assert_role(&t, role, "Virtuous");
    // +1/+1 for each enchantment you control: just the Role.
    assert_eq!(t.pt(bears), (3, 3));
    t.battlefield(P0, "Glorious Anthem");
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn wicked_role_tokens() {
    cr!("111.10", "111.10q");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mark = t.hand(P0, "Witch's Mark");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, false);
    t.cast(P0, mark).target(bears).go();
    t.resolve();
    let role = role_on(&t, P0, bears);
    assert_role(&t, role, "Wicked");
    assert_eq!(t.pt(bears), (3, 3));
    // When the Role is put into a graveyard from the battlefield, each opponent loses 1
    // life.
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, murder).target(bears).go();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn young_hero_role_tokens() {
    cr!("111.10", "111.10r");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_text(
        &mut t,
        P0,
        "Create a Young Hero Role token attached to target creature you control.",
    );
    let role = role_on(&t, P0, bears);
    assert_role(&t, role, "Young Hero");
    assert_eq!(t.pt(bears), (2, 2));
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn map_tokens() {
    cr!("111.10", "111.10s");
    let mut t = TestGame::new(2);
    let map = run_text(&mut t, P0, "Create a Map token.")[0];
    assert_artifact_token(&t, map, "Map");
    assert_eq!(
        ability_texts(&t, map),
        vec!["{1}, {T}, Sacrifice this token: Target creature you control explores. Activate only as a sorcery."]
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    // Activate only as a sorcery.
    t.g.turn.step = Step::BeginningOfCombat;
    assert!(t.activate(P0, map, 0, &[Entity::Object(bears)]).is_err());
    t.set_step(P0, Step::PrecombatMain);
    // Only a creature you control can be the target.
    let uid = activated_uid(&t, map, 0);
    let AbilityKind::Activated(act) = &t
        .obj(map)
        .chars
        .abilities
        .iter()
        .find(|a| a.uid == uid)
        .unwrap()
        .kind
    else {
        unreachable!()
    };
    let spec = act.body.targets[0].clone();
    let cands = t.g.legal_target_candidates(
        &spec,
        &mtg_engine::eval::Ctx::new(Some(map), P0),
        map,
    );
    assert!(cands.contains(&Entity::Object(bears)));
    assert!(!cands.contains(&Entity::Object(theirs)));
    t.activate(P0, map, 0, &[Entity::Object(bears)]).unwrap();
    // The token was sacrificed as a cost.
    assert!(tokens_of(&t, P0).is_empty());
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn junk_tokens() {
    cr!("111.10", "111.10t");
    let mut t = TestGame::new(2);
    let junk = run_text(&mut t, P0, "Create a Junk token.")[0];
    assert_artifact_token(&t, junk, "Junk");
    let top = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, junk, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.zone(top), Zone::Exile);
    // You may play that card this turn.
    t.lands(P0, "Forest", 2);
    let exiled = t.g.current(top);
    t.cast(P0, exiled).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn lander_tokens() {
    cr!("111.10", "111.10u");
    let mut t = TestGame::new(2);
    let lithobraking = t.hand(P0, "Lithobraking");
    t.lands(P0, "Mountain", 3);
    t.answer_yes(P0, false);
    t.cast(P0, lithobraking).go();
    t.resolve();
    let lander = tokens_of(&t, P0)[0];
    assert_artifact_token(&t, lander, "Lander");
    let forest = t.library_top(P0, "Forest");
    t.lands(P0, "Plains", 2);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.activate(P0, lander, 0, &[]).unwrap();
    t.resolve();
    let f = t.named_on_battlefield("Forest");
    assert_eq!(f.len(), 1);
    assert!(t.obj(f[0]).tapped);
}

#[test]
fn mutagen_tokens() {
    cr!("111.10", "111.10v");
    let mut t = TestGame::new(2);
    let m = run_text(&mut t, P0, "Create a Mutagen token.")[0];
    assert_artifact_token(&t, m, "Mutagen");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.activate(P0, m, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn vibranium_tokens() {
    cr!("111.10", "111.10w");
    let mut t = TestGame::new(2);
    let v = run_text(&mut t, P0, "Create a Vibranium token.")[0];
    assert_artifact_token(&t, v, "Vibranium");
    assert!(t.obj(v).has_keyword(KeywordKind::Indestructible));
    t.activate(P0, v, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, C), 1);
    assert_eq!(
        t.g.player(P0).mana_pool.mana[0].restriction,
        Some(ManaRestriction::NotNonartifactSpell)
    );
}

#[test]
fn heartwood_tokens() {
    cr!("111.10", "111.10x");
    let mut t = TestGame::new(2);
    let h = run_text(&mut t, P0, "Create a Heartwood token.")[0];
    let c = &t.obj(h).chars;
    assert_eq!(c.name.as_str(), "Heartwood Token");
    assert_eq!(c.colors, cs("RG"));
    assert_eq!(c.card_types, [CardType::Artifact].into_iter().collect());
    assert!(c.has_subtype("Heartwood"));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, h, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, G), 1);
}
