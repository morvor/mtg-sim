//! CR 702.99 Cipher.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_018_026::triggers_on_stack;
use crate::common_k702_052_066::{destroy, run_effect};
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::kw::cipher::{encoded_cards, encoded_on};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Shadow Slice ("Target opponent loses 3 life.") targeting P1 and encodes it on
/// `creature` (or declines, with `None`). Returns the card.
fn cast_shadow_slice(t: &mut TestGame, creature: Option<ObjectId>) -> ObjectId {
    t.lands(P0, "Swamp", 5);
    let c = t.hand(P0, "Shadow Slice");
    t.cast(P0, c).target(P1).go();
    t.answer_choose(
        P0,
        &creature.map(Entity::Object).into_iter().collect::<Vec<_>>(),
    );
    t.resolve_all();
    c
}

/// Attacks P1 with `attacker` (unblocked) and finishes combat.
fn hit(t: &mut TestGame, attacker: ObjectId) {
    let ap = t.g.obj(attacker).controller;
    t.set_step(ap, Step::BeginningOfCombat);
    let defender = if ap == P0 { P1 } else { P0 };
    t.attack(&[(attacker, Entity::Player(defender))], &[]);
    t.resolve_all();
}

#[test]
fn a_spell_with_cipher_is_exiled_encoded_on_a_creature() {
    cr!("702.99", "702.99a", "702.99b");
    ruling!(
        "Hidden Strings",
        "The spell with cipher is encoded on the creature as part of that spell’s resolution, just after the spell’s other effects. That card goes directly from the stack to exile. It never goes to the graveyard."
    );
    ruling!(
        "Hidden Strings",
        "The cipher ability doesn’t target that creature"
    );
    ruling!(
        "Hands of Binding",
        "You can choose only a creature to encode the card onto."
    );
    assert_supported("Shadow Slice");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let saw = t.battlefield(P0, "Bone Saw");
    let c = cast_shadow_slice(&mut t, Some(bears));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(c), Zone::Exile);
    let card = t.g.current(c);
    assert_eq!(encoded_on(&t.g, card), Some(bears));
    assert_eq!(encoded_cards(&t.g, bears), vec![card]);
    // It never went to the graveyard.
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, events::Event::ZoneChange { to: Zone::Graveyard(_), .. })));
    // Only creatures its controller controls could be chosen; nothing was targeted.
    let cands = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("cipher") => Some(candidates),
            _ => None,
        })
        .unwrap();
    assert_eq!(cands, vec![Entity::Object(bears)]);
    assert!(!cands.contains(&Entity::Object(theirs)));
    assert!(!cands.contains(&Entity::Object(saw)));
}

#[test]
fn the_encoded_creature_casts_a_copy_when_it_deals_combat_damage_to_a_player() {
    cr!("702.99a");
    ruling!(
        "Hidden Strings",
        "The copy of the card with cipher is created in and cast from exile."
    );
    ruling!(
        "Hidden Strings",
        "You cast the copy of the card with cipher during the resolution of the triggered ability. Ignore timing restrictions based on the card’s type."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = cast_shadow_slice(&mut t, Some(bears));
    assert_eq!(t.life(P1), 17);
    // Combat damage (2) and a copy of Shadow Slice (3), cast in the combat damage step.
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 12);
    // The card is still exiled and encoded; the copy wasn't encoded.
    let card = t.g.current(c);
    assert_eq!(encoded_on(&t.g, card), Some(bears));
    assert_eq!(t.g.exile.len(), 1);
    // And again next time.
    t.advance_to(P0, Step::PrecombatMain);
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 7);
}

#[test]
fn casting_the_copy_is_optional() {
    cr!("702.99a");
    ruling!(
        "Hands of Binding",
        "If you choose not to cast the copy, or you can’t cast it (perhaps because there are no legal targets available), the copy will cease to exist the next time state-based actions are performed."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_shadow_slice(&mut t, Some(bears));
    t.answer_yes(P0, false);
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.g.exile.len(), 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn its_controller_may_decline_to_encode_it() {
    cr!("702.99a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let c = cast_shadow_slice(&mut t, None);
    assert!(t.in_graveyard(P0, "Shadow Slice"));
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
}

#[test]
fn a_spell_that_doesnt_resolve_isnt_encoded() {
    cr!("702.99a");
    ruling!(
        "Hands of Binding",
        "If the spell with cipher doesn't resolve, none of its effects will happen, including cipher. The card will go to its owner's graveyard and won't be encoded on a creature."
    );
    assert_supported("Hands of Binding");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let c = t.hand(P0, "Hands of Binding");
    t.cast(P0, c).target(theirs).go();
    destroy(&mut t, theirs);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hands of Binding"));
    assert!(t.g.exile.is_empty());
}

#[test]
fn a_card_stays_exiled_but_isnt_encoded_once_the_creature_leaves() {
    cr!("702.99c");
    ruling!(
        "Hands of Binding",
        "If the creature leaves the battlefield, the exiled card will no longer be encoded on any creature. It will stay exiled."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = cast_shadow_slice(&mut t, Some(bears));
    destroy(&mut t, bears);
    let card = t.g.current(c);
    assert_eq!(t.zone(c), Zone::Exile);
    assert_eq!(encoded_on(&t.g, card), None);
    // The creature, returned to the battlefield, is a new object without the ability.
    let back = t.g.current(bears);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &[Entity::Object(back)],
    );
    let back = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(encoded_on(&t.g, card), None);
    assert!(!t
        .obj(back)
        .chars
        .abilities
        .iter()
        .any(|a| a.text == "Cipher"));
}

#[test]
fn a_card_stays_encoded_when_the_creature_changes_controller_or_type() {
    cr!("702.99c");
    ruling!(
        "Hands of Binding",
        "If another player gains control of the creature, that player will control the triggered ability. That player will create a copy of the encoded card and may cast it."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = cast_shadow_slice(&mut t, Some(bears));
    let card = t.g.current(c);
    // P1 gains control of the Bears and attacks P0 with them.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(bears)],
    );
    assert_eq!(t.obj(bears).controller, P1);
    assert_eq!(encoded_on(&t.g, card), Some(bears));
    t.set_step(P1, Step::BeginningOfCombat);
    t.g.objects[bears.0 as usize].summoning_sick = false;
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    t.resolve_all();
    // 2 combat damage, then P1's copy of Shadow Slice targets P0.
    assert_eq!(t.life(P0), 15);
    // It stops being a creature: still encoded.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveTypes(vec![types::CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(!t.obj(bears).chars.is(types::CardType::Creature));
    assert_eq!(encoded_on(&t.g, card), Some(bears));
}

#[test]
fn a_creature_that_loses_the_ability_stays_encoded_but_doesnt_trigger() {
    cr!("702.99a", "702.99c");
    ruling!(
        "Hands of Binding",
        "If that creature loses that ability and subsequently deals combat damage to a player, the triggered ability won't trigger. However, the exiled card will continue to be encoded on that creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = cast_shadow_slice(&mut t, Some(bears));
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 15);
    assert_eq!(triggers_on_stack(&t, "Cipher"), 0);
    assert_eq!(encoded_on(&t.g, t.g.current(c)), Some(bears));
}

#[test]
fn several_cards_can_be_encoded_on_one_creature() {
    cr!("702.99a", "702.99b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_shadow_slice(&mut t, Some(bears));
    cast_shadow_slice(&mut t, Some(bears));
    assert_eq!(encoded_cards(&t.g, bears).len(), 2);
    assert_eq!(t.life(P1), 14);
    hit(&mut t, bears);
    // Two triggers: 2 combat damage and two copies.
    assert_eq!(t.life(P1), 6);
}
