//! CR 114: emblems.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts a planeswalker with `loyalty` loyalty counters onto the battlefield.
fn walker(t: &mut TestGame, p: PlayerId, name: &str, loyalty: u32) -> ObjectId {
    let pw = t.battlefield(p, name);
    t.g.objects[pw.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), loyalty);
    t.g.recompute();
    pw
}

/// The index (among its activated abilities) of the ability of `pw` that creates an
/// emblem.
fn emblem_ability(t: &TestGame, pw: ObjectId) -> usize {
    t.obj(pw)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(act) => Some(act),
            _ => None,
        })
        .position(|act| format!("{:?}", act.body.effect).contains("CreateEmblem"))
        .expect("no emblem ability")
}

fn emblems(t: &TestGame) -> Vec<ObjectId> {
    t.g.command
        .iter()
        .copied()
        .filter(|id| t.obj(*id).kind == ObjKind::Emblem)
        .collect()
}

#[test]
fn a_player_gets_an_emblem_in_the_command_zone_and_owns_and_controls_it() {
    cr!("114.1", "114.2");
    ruling!(
        "Ob Nixilis Reignited",
        "is both owned and controlled by the target opponent"
    );
    let mut t = TestGame::new(2);
    let ob = walker(&mut t, P0, "Ob Nixilis Reignited", 8);
    t.activate(P0, ob, emblem_ability(&t, ob), &[Entity::Player(P1)]).unwrap();
    t.resolve();
    let e = emblems(&t);
    assert_eq!(e.len(), 1);
    let emblem = t.obj(e[0]);
    assert_eq!(emblem.zone, Zone::Command);
    assert_eq!(emblem.owner, P1);
    assert_eq!(emblem.controller, P1);
    // "Whenever a player draws a card, you lose 2 life": "you" is the emblem's owner.
    t.g.draw_cards(P0, 1);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn each_opponent_gets_an_emblem_that_triggers_on_their_own_upkeep() {
    cr!("114.2", "114.4");
    ruling!(
        "Chandra, Awakened Inferno",
        "triggers at the beginning of the upkeep of the player who gets the emblem"
    );
    ruling!(
        "Chandra, Awakened Inferno",
        "A player may have more than one of Chandra's emblems. Each one's ability triggers separately."
    );
    let mut t = TestGame::new(3);
    let chandra = walker(&mut t, P0, "Chandra, Awakened Inferno", 6);
    t.activate(P0, chandra, emblem_ability(&t, chandra), &[]).unwrap();
    t.resolve();
    let owners: Vec<PlayerId> = emblems(&t).iter().map(|e| t.obj(*e).owner).collect();
    assert_eq!(owners.len(), 2);
    assert!(owners.contains(&P1) && owners.contains(&P2));
    // A second emblem for each opponent (next turn's activation).
    t.g.objects[chandra.0 as usize]
        .activations_this_turn
        .clear();
    t.activate(P0, chandra, emblem_ability(&t, chandra), &[]).unwrap();
    t.resolve();
    // P0's own upkeep: nothing.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 20);
}

#[test]
fn an_emblem_has_no_characteristics_other_than_its_abilities() {
    cr!("114.3");
    ruling!("Koth, Fire of Resistance", "Koth's emblem is colorless");
    let mut t = TestGame::new(2);
    let koth = walker(&mut t, P0, "Koth, Fire of Resistance", 7);
    let ult = emblem_ability(&t, koth);
    t.activate(P0, koth, ult, &[]).unwrap();
    t.resolve();
    let e = emblems(&t)[0];
    let c = &t.obj(e).chars;
    assert!(c.name.is_empty());
    assert!(c.card_types.is_empty());
    assert!(c.subtypes.is_empty());
    assert!(c.mana_cost.is_none());
    assert_eq!(c.colors, ColorSet::NONE);
    assert_eq!(c.abilities.len(), 1);
    // A creature with protection from red can be the target of its ability.
    let knight = t.battlefield(P1, "Paladin en-Vec");
    t.answer_targets(P0, &[Entity::Object(knight)]);
    t.enter(P0, "Mountain");
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Paladin en-Vec"));
}

#[test]
fn abilities_of_emblems_function_in_the_command_zone() {
    cr!("114.4");
    ruling!(
        "Sorin, Lord of Innistrad",
        "The emblems are cumulative. If you get two of them, creatures you control will get +2/+0."
    );
    let mut t = TestGame::new(2);
    let sorin = walker(&mut t, P0, "Sorin, Lord of Innistrad", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, sorin, emblem_ability(&t, sorin), &[]).unwrap();
    t.resolve();
    // Sorin is gone (no loyalty left); the emblem keeps working from the command zone.
    t.settle();
    assert!(!t.on_battlefield(sorin));
    assert_eq!(t.pt(bears), (3, 2));
    let sorin2 = walker(&mut t, P0, "Sorin, Lord of Innistrad", 2);
    t.activate(P0, sorin2, emblem_ability(&t, sorin2), &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn an_emblem_is_neither_a_card_nor_a_permanent() {
    cr!("114.5");
    ruling!(
        "Chandra, Awakened Inferno",
        "Emblems aren't permanents and can't be exiled or destroyed."
    );
    let mut t = TestGame::new(2);
    let sorin = walker(&mut t, P0, "Sorin, Lord of Innistrad", 5);
    t.activate(P0, sorin, emblem_ability(&t, sorin), &[]).unwrap();
    t.resolve();
    let e = emblems(&t)[0];
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(!t.g.matches(e, &Filter::Permanent, &ctx));
    assert!(!t.g.matches(e, &Filter::Card, &ctx));
    // It can't be chosen as the target of "destroy target permanent".
    let vindicate = t.hand(P0, "Vindicate");
    t.lands(P0, "Plains", 3);
    assert!(t.cast(P0, vindicate).target(e).try_go().is_err());
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Destroy all nonland permanents" leaves the emblem.
    t.lands(P0, "Plains", 6);
    let cleansing = t.hand(P0, "Planar Cleansing");
    t.cast(P0, cleansing).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.g.is_live(e));
    assert_eq!(t.obj(e).zone, Zone::Command);
}
