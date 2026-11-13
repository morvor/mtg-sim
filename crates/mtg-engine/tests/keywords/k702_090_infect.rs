//! CR 702.90 Infect.

use crate::common_k702_011_017::{assert_supported, attack_with, custom_card};
use crate::common_k702_018_026::declare_blocks;
use crate::common_k702_052_066::{destroy, run_effect};
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::POISON)
}

/// Makes `source` deal `amount` damage to `to` (as an effect would).
fn damage(t: &mut TestGame, source: ObjectId, amount: i32, to: Entity) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(source)], vec![to]];
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

fn grant_infect(t: &mut TestGame, id: ObjectId) {
    run_effect(
        t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Infect))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn infect_damage_to_a_player_gives_poison_counters_instead_of_life_loss() {
    cr!("702.90", "702.90a", "702.90b");
    assert_supported("Glistener Elf");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Glistener Elf");
    attack_with(&mut t, &[(elf, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(poison(&t, P1), 1);
}

#[test]
fn infect_damage_to_a_creature_puts_minus_one_counters_on_it_instead_of_marking_damage() {
    cr!("702.90c");
    ruling!(
        "Nest of Scarabs",
        "If a creature with wither or infect deals damage to a creature, the controller of the creature with wither or infect puts that many -1/-1 counters on the second creature."
    );
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Glistener Elf");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(elf, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(giant, elf)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.g.obj(giant).damage, 0);
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
    assert_eq!(t.pt(giant), (2, 2));
    assert!(t.in_graveyard(P0, "Glistener Elf"));
}

#[test]
fn infect_applies_to_noncombat_damage_and_is_damage_in_all_respects() {
    cr!("702.90b", "702.90c");
    ruling!(
        "Plague Stinger",
        "Infect's effect applies to any damage, not just combat damage."
    );
    ruling!(
        "Plague Stinger",
        "If the source with infect also has lifelink, damage dealt by that source also causes its controller to gain that much life."
    );
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Glistener Elf");
    let bears = t.battlefield(P1, "Grizzly Bears");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Lifelink))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(elf)],
    );
    damage(&mut t, elf, 2, Entity::Player(P1));
    damage(&mut t, elf, 1, Entity::Object(bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(poison(&t, P1), 2);
    assert_eq!(t.counters(bears, counters::MINUS1), 1);
    assert_eq!(t.g.obj(bears).damage, 0);
    // Lifelink still works: 3 damage was dealt.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn prevented_infect_damage_gives_no_counters() {
    cr!("702.90b", "702.90c");
    ruling!(
        "Plague Stinger",
        "If damage from a source with infect that would be dealt to a player is prevented, that player doesn't get poison counters."
    );
    assert_supported("Fog");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Glistener Elf");
    let other = t.battlefield(P0, "Glistener Elf");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(elf, Entity::Player(P1)), (other, Entity::Player(P1))],
    );
    t.lands(P1, "Forest", 1);
    let fog = t.hand(P1, "Fog");
    t.cast(P1, fog).go();
    t.resolve();
    declare_blocks(&mut t, P1, &[(bears, other)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(poison(&t, P1), 0);
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
}

#[test]
fn infect_damage_to_a_planeswalker_removes_loyalty_normally() {
    cr!("702.90b", "120.3c");
    ruling!(
        "Plague Stinger",
        "Damage from a source with infect affects planeswalkers normally."
    );
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Glistener Elf");
    let jace = t.battlefield(P1, "Jace Beleren");
    attack_with(&mut t, &[(elf, Entity::Object(jace))]);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.counters(jace, counters::LOYALTY), 2);
    assert_eq!(poison(&t, P1), 0);
}

#[test]
fn a_source_that_left_uses_its_last_known_information_for_infect() {
    cr!("702.90d", "608.2h");
    ruling!(
        "Warstorm Surge",
        "Since damage is dealt by the creature, abilities like lifelink, deathtouch and infect are taken into account, even if the creature has left the battlefield by the time it deals damage."
    );
    assert_supported("Warstorm Surge");
    assert_supported("Tainted Strike");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Warstorm Surge");
    // "Whenever a creature you control enters, it deals damage equal to its power to
    // any target."
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response the Bears gain infect (3/2), then die.
    t.lands(P0, "Swamp", 1);
    let strike = t.hand(P0, "Tainted Strike");
    t.cast(P0, strike).target(bears).go();
    t.resolve();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Infect));
    destroy(&mut t, bears);
    t.settle();
    // The card in the graveyard doesn't have infect, but the creature did.
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Infect));
    t.resolve_all();
    assert_eq!(poison(&t, P1), 3);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn infect_works_whatever_zone_the_source_deals_damage_from() {
    cr!("702.90e");
    // A creature card with infect that deals damage from its owner's graveyard.
    let mut def = custom_card("Grave Stinger", "Creature — Insect", Some((1, 1)), "Infect");
    let mut act = ActivatedAbility::new(
        Cost::free(),
        Body::simple(
            vec![TargetSpec::player(PlayerFilter::Any, "target player")],
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::c(2),
                to: Sel::Target(0),
            },
        ),
    );
    act.zone = FunctionZone::Graveyard;
    def.faces[0].chars.abilities.push(AbilityDef::new(
        AbilityKind::Activated(act),
        "{0}: ~ deals 2 damage to target player. (graveyard)",
    ));
    let mut t = TestGame::new(2);
    let stinger = t.custom(P0, def, Zone::Graveyard(P0));
    t.activate(P0, stinger, 0, &[Entity::Player(P1)])
        .expect("activate from the graveyard");
    t.resolve();
    assert_eq!(t.zone(stinger), Zone::Graveyard(P0));
    assert_eq!(poison(&t, P1), 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn multiple_instances_of_infect_are_redundant() {
    cr!("702.90f");
    ruling!(
        "Triumph of the Hordes",
        "Multiple instances of infect are redundant."
    );
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Glistener Elf");
    let bears = t.battlefield(P1, "Grizzly Bears");
    grant_infect(&mut t, elf);
    assert_eq!(t.obj_now(elf).chars.keyword_count(KeywordKind::Infect), 2);
    damage(&mut t, elf, 1, Entity::Player(P1));
    damage(&mut t, elf, 1, Entity::Object(bears));
    assert_eq!(poison(&t, P1), 1);
    assert_eq!(t.counters(bears, counters::MINUS1), 1);
}
