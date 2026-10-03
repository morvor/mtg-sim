//! CR 702.89 Umbra armor.

use crate::common_k702_011_017::{assert_supported, attack_with, custom_card};
use crate::common_k702_018_026::declare_blocks;
use crate::common_k702_052_066::{choose_replacement, destroy, run_effect};
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts the real Aura `name` onto the battlefield under `p`'s control attached to `to`.
fn enchant(t: &mut TestGame, p: PlayerId, name: &str, to: ObjectId) -> ObjectId {
    let aura = t.battlefield(p, name);
    assert!(t.g.attach(aura, Entity::Object(to)));
    t.g.recompute();
    aura
}

fn deal(t: &mut TestGame, source: ObjectId, amount: i32, to: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(source)], vec![Entity::Object(to)]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::Target(0),
            amount: Value::c(amount),
            to: Sel::Target(1),
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn umbra_armor_destroys_the_aura_instead_of_the_enchanted_creature() {
    cr!("702.89", "702.89a");
    ruling!(
        "Hyena Umbra",
        "Umbra armor's effect is applied no matter why the enchanted creature would be destroyed"
    );
    ruling!(
        "Hyena Umbra",
        "Umbra armor's effect is mandatory. If the enchanted creature would be destroyed, you must remove all damage from it (if it has any) and destroy the Aura that has umbra armor instead."
    );
    assert_supported("Hyena Umbra");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = enchant(&mut t, P0, "Hyena Umbra", bears);
    assert_eq!(t.pt(bears), (3, 3));
    destroy(&mut t, bears);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(umbra));
    assert!(t.in_graveyard(P0, "Hyena Umbra"));
    assert_eq!(t.pt(bears), (2, 2));
    // Mandatory: nobody was asked.
    assert!(!t.asked().iter().any(|(_, d)| matches!(
        d,
        Decision::YesNo { .. } | Decision::ChooseReplacement { .. }
    )));
}

#[test]
fn umbra_armor_removes_all_damage_from_a_creature_dealt_lethal_damage() {
    cr!("702.89a", "704.5g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    enchant(&mut t, P0, "Spider Umbra", bears);
    deal(&mut t, giant, 3, bears);
    assert_eq!(t.g.obj(bears).damage, 3);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.g.obj(bears).damage, 0);
    assert!(t.in_graveyard(P0, "Spider Umbra"));
}

#[test]
fn umbra_armor_replaces_every_destruction_from_lethal_and_deathtouch_damage() {
    cr!("702.89a", "704.5h");
    ruling!(
        "Hyena Umbra",
        "umbra armor's effect will replace all of them and save the creature"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let snake = t.battlefield(P1, "Deadly Recluse");
    enchant(&mut t, P0, "Spider Umbra", bears);
    // 3 damage from a deathtouch source: both lethal damage and deathtouch.
    deal(&mut t, snake, 3, bears);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.g.obj(bears).damage, 0);
    assert!(!t.g.obj(bears).deathtouch_damage);
    assert!(t.in_graveyard(P0, "Spider Umbra"));
}

#[test]
fn umbra_armor_isnt_regeneration() {
    cr!("702.89a");
    ruling!(
        "Hyena Umbra",
        "Umbra armor's effect is not regeneration. Specifically, if umbra armor's effect is applied, the enchanted creature does not become tapped and is not removed from combat as a result."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, P0, "Hyena Umbra", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    // Destroyed during combat (a "can't be regenerated" destruction).
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: true,
        },
        &[Entity::Object(bears)],
    );
    t.settle();
    assert!(t.on_battlefield(bears));
    assert!(t.g.is_attacking(bears));
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn umbra_armor_doesnt_apply_to_other_ways_of_dying() {
    cr!("702.89a");
    ruling!(
        "Hyena Umbra",
        "Umbra armor has no effect if the enchanted creature is put into a graveyard for any other reason, such as if it's sacrificed"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, P0, "Hyena Umbra", bears);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::SacrificeObjects {
            what: Sel::Target(0),
        },
        &[Entity::Object(bears)],
    );
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Toughness 0 or less: put into the graveyard, not destroyed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, P0, "Hyena Umbra", bears);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: counters::MINUS1.into(),
            n: Value::c(3),
        },
        &[Entity::Object(bears)],
    );
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn an_indestructible_creature_doesnt_need_umbra_armor() {
    cr!("702.89a");
    ruling!(
        "Hyena Umbra",
        "Umbra armor won't do anything because it won't have to."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = enchant(&mut t, P0, "Hyena Umbra", bears);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(
                KeywordKind::Indestructible,
            ))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    destroy(&mut t, bears);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(umbra));
}

#[test]
fn destroying_both_the_aura_and_the_creature_saves_the_creature() {
    cr!("702.89a");
    ruling!(
        "Hyena Umbra",
        "If a spell or ability would destroy both an Aura with umbra armor and the creature it's enchanting at the same time, umbra armor's effect will save the enchanted creature from being destroyed."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = enchant(&mut t, P0, "Hyena Umbra", bears);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::AllTargets,
            no_regen: false,
        },
        &[Entity::Object(bears), Entity::Object(umbra)],
    );
    t.settle();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Hyena Umbra"));
}

#[test]
fn the_enchanted_creatures_controller_chooses_which_umbra_is_destroyed() {
    cr!("702.89a", "616.1");
    ruling!(
        "Hyena Umbra",
        "one of those Auras is destroyed instead—but only one of them. You choose which one because you control the enchanted creature."
    );
    let mut survivors = Vec::new();
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let a = enchant(&mut t, P0, "Hyena Umbra", bears);
        // An Aura controlled by the opponent works for the creature's controller too.
        let b = enchant(&mut t, P1, "Spider Umbra", bears);
        choose_replacement(&mut t, P0, pick);
        destroy(&mut t, bears);
        t.settle();
        assert!(t.on_battlefield(bears));
        let asked = t
            .asked()
            .into_iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
            .collect::<Vec<_>>();
        assert_eq!(asked.len(), 1);
        assert_eq!(asked[0].0, P0);
        // Exactly one of them was destroyed: the one chosen.
        assert_ne!(t.on_battlefield(a), t.on_battlefield(b));
        survivors.push(t.on_battlefield(a));
    }
    assert_ne!(survivors[0], survivors[1]);
}

#[test]
fn totem_armor_is_umbra_armor() {
    cr!("702.89b");
    ruling!(
        "Hyena Umbra",
        "Some printings of this card refer to the ability \"totem armor\"."
    );
    // An Aura printed with the old wording.
    let def = custom_card(
        "Old Umbra",
        "Enchantment — Aura",
        None,
        "Enchant creature\nTotem armor",
    );
    assert!(def
        .front()
        .chars
        .abilities
        .iter()
        .any(|a| a.keyword().is_some_and(|k| k.kind == KeywordKind::UmbraArmor)));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.custom(P0, def, mtg_engine::object::Zone::Battlefield);
    assert!(t.g.attach(aura, Entity::Object(bears)));
    t.g.recompute();
    destroy(&mut t, bears);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Old Umbra"));
}

#[test]
fn umbra_mystic_gives_auras_on_your_permanents_umbra_armor() {
    cr!("702.89a");
    ruling!(
        "Umbra Mystic",
        "Umbra Mystic grants umbra armor to Auras attached to permanents you control, regardless of who controls those Auras. Conversely, it doesn't grant umbra armor to Auras you control that are attached to permanents controlled by other players."
    );
    assert_supported("Umbra Mystic");
    assert_supported("Holy Strength");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Umbra Mystic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An opponent's Aura on your creature gets umbra armor.
    let theirs = enchant(&mut t, P1, "Holy Strength", bears);
    assert!(t.obj_now(theirs).has_keyword(KeywordKind::UmbraArmor));
    destroy(&mut t, bears);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Holy Strength"));
    // Your Aura on an opponent's creature doesn't.
    let giant = t.battlefield(P1, "Hill Giant");
    let mine = enchant(&mut t, P0, "Holy Strength", giant);
    assert!(!t.obj_now(mine).has_keyword(KeywordKind::UmbraArmor));
    destroy(&mut t, giant);
    t.settle();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn umbra_armor_protects_any_enchanted_permanent() {
    cr!("702.89a");
    ruling!(
        "Umbra Mystic",
        "The ability works the same way even if the Aura is enchanting a land, an artifact, or any other permanent."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Umbra Mystic");
    let land = t.battlefield(P0, "Forest");
    // "Enchant land" (Wild Growth).
    let aura = enchant(&mut t, P0, "Wild Growth", land);
    assert!(t.obj_now(aura).has_keyword(KeywordKind::UmbraArmor));
    destroy(&mut t, land);
    t.settle();
    assert!(t.on_battlefield(land));
    assert!(t.in_graveyard(P0, "Wild Growth"));
}
