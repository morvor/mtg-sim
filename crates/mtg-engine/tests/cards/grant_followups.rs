//! Grants that follow up on an earlier instruction or name an unusual group
//! (`src/oracle/patterns/grant_grammar.rs`, `r113_spell_ability_word_followup.rs`,
//! `keywords_702_11_17.rs`, `counters_resources_life.rs`, `statics.rs`, `phrases.rs`):
//! "That creature also gains trample ... if [condition]" in an ability-word paragraph
//! (CR 207.2c, 608.2c), "Those creatures gain ..." after counting them, "Players gain
//! hexproof" (CR 702.11c), "You gain that much life", "Eldrazi you control have ..." and
//! "Eldrazi Spawn creatures you control get ...".

use mtg_engine::ability::AbilityKind;
use mtg_engine::eval::Ctx;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// A custom card compiled from oracle text with the real oracle compiler.
fn custom_card(name: &str, type_line: &str, pt: Option<(i32, i32)>, text: &str) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(compiled.unsupported.is_empty(), "{:?}", compiled.unsupported);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities: compiled.abilities,
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

fn mana_abilities(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
        .count()
}

#[test]
fn temur_battle_rage_trample_only_with_a_ferocious_creature() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Temur Battle Rage",
        "checks whether you control a creature with power 4 or greater as it resolves"
    );
    assert_supported(&["Temur Battle Rage", "Roar of Challenge"]);
    for ferocious in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if ferocious {
            t.battlefield(P0, "Craw Wurm");
        }
        t.lands(P0, "Mountain", 2);
        let s = t.hand(P0, "Temur Battle Rage");
        t.cast(P0, s).target(bears).go();
        t.resolve();
        let o = t.obj_now(bears);
        assert!(o.has_keyword(KeywordKind::DoubleStrike));
        assert_eq!(o.has_keyword(KeywordKind::Trample), ferocious);
    }
}

#[test]
fn roar_of_challenge_indestructible_only_with_a_ferocious_creature() {
    cr!("608.2c");
    for ferocious in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if ferocious {
            t.battlefield(P0, "Craw Wurm");
        }
        t.lands(P0, "Forest", 3);
        let s = t.hand(P0, "Roar of Challenge");
        t.cast(P0, s).target(bears).go();
        t.resolve();
        assert_eq!(
            t.obj_now(bears).has_keyword(KeywordKind::Indestructible),
            ferocious
        );
    }
}

#[test]
fn inspiring_call_counted_creatures_gain_indestructible() {
    cr!("608.2c");
    ruling!(
        "Inspiring Call",
        "have +1/+1 counters put on them after Inspiring Call resolves won't gain indestructible"
    );
    assert_supported(&["Inspiring Call"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(a), "+1/+1", 1, None);
    t.g.add_counters(Entity::Object(b), "+1/+1", 2, None);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Forest", 3);
    let s = t.hand(P0, "Inspiring Call");
    let hand = t.hand_size(P0);
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert!(t.obj_now(a).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj_now(b).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(c).has_keyword(KeywordKind::Indestructible));
    // A counter put on afterwards doesn't grant it.
    t.g.add_counters(Entity::Object(c), "+1/+1", 1, None);
    t.settle();
    assert!(!t.obj_now(c).has_keyword(KeywordKind::Indestructible));
}

#[test]
fn players_gain_hexproof_until_end_of_turn() {
    cr!("702.11c");
    let mut def = custom_card(
        "Hexproof For All",
        "Instant",
        None,
        "Players gain hexproof until end of turn.",
    );
    def.faces[0].chars.mana_cost = Some(mtg_engine::mana::ManaCost::generic(0));
    let mut t = TestGame::new(2);
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    // An opponent can't target either player; a player can still target themself.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.recompute();
    let spec = t.g.spell_body(bolt).targets[0].clone();
    let targets = t.g.legal_target_candidates(&spec, &Ctx::new(Some(bolt), P1), bolt);
    assert!(!targets.contains(&Entity::Player(P0)));
    assert!(targets.contains(&Entity::Player(P1)));
}

#[test]
fn foul_tongue_shriek_you_gain_that_much_life() {
    cr!("119.3");
    assert_supported(&["Foul-Tongue Shriek"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ]),
    );
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareAttackers && g.turn.stage == mtg_engine::turn::Stage::Priority
    });
    assert!(ok);
    let s = t.hand(P0, "Foul-Tongue Shriek");
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn path_of_annihilation_eldrazi_you_control_have_a_mana_ability() {
    cr!("613.1f");
    assert_supported(&["Path of Annihilation"]);
    let mut t = TestGame::new(2);
    let eldrazi = t.custom(
        P0,
        custom_card("Test Eldrazi", "Creature — Eldrazi", Some((3, 3)), ""),
        Zone::Battlefield,
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.custom(
        P1,
        custom_card("Test Eldrazi", "Creature — Eldrazi", Some((3, 3)), ""),
        Zone::Battlefield,
    );
    t.battlefield(P0, "Path of Annihilation");
    t.settle();
    assert_eq!(mana_abilities(&t, eldrazi), 1);
    assert_eq!(mana_abilities(&t, bears), 0);
    assert_eq!(mana_abilities(&t, theirs), 0);
}

#[test]
fn broodwarden_pumps_only_eldrazi_spawn_creatures() {
    cr!("205.3m", "613.4c");
    assert_supported(&["Broodwarden"]);
    let mut t = TestGame::new(2);
    let spawn = t.custom(
        P0,
        custom_card("Test Spawn", "Creature — Eldrazi Spawn", Some((0, 1)), ""),
        Zone::Battlefield,
    );
    let eldrazi = t.custom(
        P0,
        custom_card("Test Eldrazi", "Creature — Eldrazi", Some((3, 3)), ""),
        Zone::Battlefield,
    );
    t.battlefield(P0, "Broodwarden");
    t.settle();
    assert_eq!(t.pt(spawn), (2, 2));
    assert_eq!(t.pt(eldrazi), (3, 3));
}
