//! Rulings batch S03 — cipher (CR 702.99): "Then you may exile this spell card encoded on a
//! creature you control. Whenever that creature deals combat damage to a player, its
//! controller may cast a copy of the encoded card without paying its mana cost."

use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s03_common::*;
use mtg_engine::events::Event;
use mtg_engine::kw::cipher::{encoded_cards, encoded_on};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts the real cipher card `name` (with the mana for it and `targets`, one per
/// slot) and encodes it on `creature` (or declines with `None`) as it resolves. Returns
/// the card.
fn cast_and_encode(
    t: &mut TestGame,
    name: &str,
    targets: &[Entity],
    creature: Option<ObjectId>,
) -> ObjectId {
    supported(name);
    let c = in_hand_with_mana(t, P0, name);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, c).go();
    t.answer_choose(
        P0,
        &creature.map(Entity::Object).into_iter().collect::<Vec<_>>(),
    );
    t.resolve_all();
    c
}

/// Attacks P1 with `attacker` (unblocked) and finishes combat, resolving what triggers.
fn hit(t: &mut TestGame, attacker: ObjectId) {
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P1))], &[]);
    t.resolve_all();
}

/// Whether the card `card` was ever put into a graveyard.
fn went_to_graveyard(t: &TestGame, card: ObjectId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::ZoneChange { old, to: Zone::Graveyard(_), .. } if *old == card))
}

#[test]
fn damage_dealt_to_two_players_at_once_triggers_once_for_each_player() {
    cr!("702.99a", "510.2", "614.9", "603.2c");
    ruling!(
        "Stolen Identity",
        "If a creature with an encoded card deals combat damage to more than one player simultaneously (perhaps because some of the combat damage was redirected), the triggered ability will trigger once for each player it deals combat damage to. Each ability will create a copy of the exiled card and allow you to cast it."
    );
    supported("Colossal Dreadmaw");
    supported("Sivvi's Valor");
    let mut t = TestGame::new(3);
    let wurm = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Stolen Identity: "Create a token that's a copy of target artifact or creature."
    let card = cast_and_encode(
        &mut t,
        "Stolen Identity",
        &[Entity::Object(bears)],
        Some(wurm),
    );
    assert_eq!(encoded_on(&t.g, t.g.current(card)), Some(wurm));
    assert_eq!(tokens(&t, P0).len(), 1);
    // The 6/6 trampler attacks P1 and is blocked by the Bears; P2 casts Sivvi's Valor
    // ("All damage that would be dealt to target creature this turn is dealt to you
    // instead.") on the blocker.
    to_blockers(&mut t, &[(wurm, Entity::Player(P1))], &[(bears, wurm)]);
    let valor = in_hand_with_mana(&mut t, P2, "Sivvi's Valor");
    t.cast(P2, valor).target(bears).go();
    t.resolve();
    // Two triggers, each casting a copy of Stolen Identity (copying the Bears again).
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    // 2 damage assigned to the Bears was dealt to P2, the 4 trampling over to P1.
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P2), 18);
    assert!(t.on_battlefield(bears));
    let combat_damage_to_players: Vec<_> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage {
                    source,
                    target: Entity::Player(p),
                    combat: true,
                    ..
                } if *source == wurm => Some(*p),
                _ => None,
            })
            .collect();
    assert_eq!(combat_damage_to_players.len(), 2);
    assert_eq!(tokens(&t, P0).len(), 3);
    // The card is still exiled and encoded; the copies ceased to exist.
    assert_eq!(encoded_on(&t.g, t.g.current(card)), Some(wurm));
    assert_eq!(t.g.exile.len(), 1);
}

#[test]
fn a_copy_not_cast_ceases_to_exist_and_cant_be_cast_later() {
    cr!("702.99a", "704.5e", "707.12", "707.12a", "601.2c");
    ruling!(
        "Voidwalk",
        "If you choose not to cast the copy, or you can’t cast it (perhaps because there are no legal targets available), the copy will cease to exist the next time state-based actions are performed. You won’t get a chance to cast the copy at a later time."
    );
    supported("Lightning Greaves");
    // Voidwalk: "Exile target creature. Return it to the battlefield under its owner's
    // control at the beginning of the next end step." It exiles a token (which won't
    // return).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let token = create_token(&mut t, P1, "Soldier");
    let card = cast_and_encode(&mut t, "Voidwalk", &[Entity::Object(token)], Some(bears));
    let card = t.g.current(card);
    assert!(tokens(&t, P1).is_empty());
    // P0 chooses not to cast the copy (which could have exiled the Hill Giant).
    t.answer_yes(P0, false);
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.g.exile, vec![card]);
    assert_eq!(t.stack_len(), 0);
    // Later in the turn, there's nothing left to cast.
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.g.exile, vec![card]);
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Voidwalk").len(), 1);
    assert!(t.on_battlefield(giant));

    // The copy can't be cast when there's no legal target: the only creature has shroud.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let token = create_token(&mut t, P1, "Soldier");
    let card = cast_and_encode(&mut t, "Voidwalk", &[Entity::Object(token)], Some(bears));
    let card = t.g.current(card);
    let greaves = t.battlefield(P0, "Lightning Greaves");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.activate(P0, greaves, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t
        .obj(bears)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Shroud));
    let from = t.asked().len();
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 18);
    // The copy couldn't be cast (no target could be chosen), and it ceased to exist.
    assert!(target_candidates_since(&t, from)
        .iter()
        .all(|c| c.is_empty()));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.g.exile, vec![card]);
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.g.exile, vec![card]);
}

/// The candidates of every target choice asked since decision `from`.
fn target_candidates_since(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_creature_that_loses_the_ability_doesnt_trigger_but_the_card_stays_encoded() {
    cr!("702.99a", "702.99c", "613.1f", "613.7a");
    ruling!(
        "Last Thoughts",
        "The exiled card with cipher grants a triggered ability to the creature it’s encoded on. If that creature loses that ability and subsequently deals combat damage to a player, the triggered ability won’t trigger. However, the exiled card will continue to be encoded on that creature."
    );
    supported("Humility");
    // Last Thoughts: "Draw a card."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = cast_and_encode(&mut t, "Last Thoughts", &[], Some(bears));
    let card = t.g.current(card);
    let hand = t.hand_size(P0);
    // Humility enters later than the card was encoded: creatures lose all abilities.
    let humility = t.battlefield(P1, "Humility");
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(triggers_on_stack(&t, "Cipher"), 0);
    assert_eq!(encoded_on(&t.g, card), Some(bears));
    assert_eq!(encoded_cards(&t.g, bears), vec![card]);
    // Once Humility is gone, the creature has the ability again.
    destroy_permanent(&mut t, humility);
    t.advance_to(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 17);
    // The turn's draw is already done; the copy of Last Thoughts drew a card.
    assert_eq!(t.hand_size(P0), hand + 1);
}

/// Destroys a permanent as a resolving effect of P1 would.
fn destroy_permanent(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.settle();
}

#[test]
fn a_countered_cipher_spell_goes_to_the_graveyard_unencoded() {
    cr!("702.99a", "701.6a", "608.2b");
    ruling!(
        "Last Thoughts",
        "If the spell with cipher doesn’t resolve, none of its effects will happen, including cipher. The card will go to its owner’s graveyard and won’t be encoded on a creature."
    );
    supported("Last Thoughts");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = in_hand_with_mana(&mut t, P0, "Last Thoughts");
    let spell = t.cast(P0, c).go();
    let hand = t.hand_size(P0);
    let counter = in_hand_with_mana(&mut t, P1, "Counterspell");
    t.cast(P1, counter).target(spell).go();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Last Thoughts"));
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.g.exile.is_empty());
    assert!(encoded_cards(&t.g, bears).is_empty());
    // Nobody was asked to choose a creature to encode it on.
    assert!(choice_candidates(&t, 0, "cipher").is_empty());
}

#[test]
fn the_card_is_encoded_after_the_spells_other_effects_straight_from_the_stack() {
    cr!("702.99a", "608.2c", "608.2n");
    ruling!(
        "Writ of Return",
        "The spell with cipher is encoded on the creature as part of that spell's resolution, just after the spell's other effects. That card goes directly from the stack to exile. It never goes to the graveyard."
    );
    // Writ of Return: "Return target creature card from your graveyard to the battlefield
    // tapped." The creature it returns can be the one it's encoded on.
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P0, "Grizzly Bears");
    let c = in_hand_with_mana(&mut t, P0, "Writ of Return");
    supported("Writ of Return");
    t.cast(P0, c).target(dead).go();
    // The creature is chosen while the spell resolves, after the Bears have returned: P0
    // encodes the card on the only candidate, the returned Bears.
    respond(&mut t, P0, first_cipher_candidate);
    let seen = watch(
        &mut t,
        P0,
        |d| {
            matches!(d, mtg_engine::decision::Decision::ChooseEntities { prompt, .. }
                if prompt.contains("cipher"))
        },
        |g| {
            (
                g.find_in_zone(Zone::Battlefield, "Grizzly Bears"),
                g.stack.len(),
            )
        },
    );
    let spell = t.g.stack[0];
    t.resolve_all();
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].0.len(), 1, "the Bears had returned by then");
    assert_eq!(seen[0].1, 1, "Writ of Return was still resolving");
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(t.obj(bears).tapped);
    let card = t.g.current(spell);
    assert_eq!(t.obj(card).zone, Zone::Exile);
    assert_eq!(encoded_on(&t.g, card), Some(bears));
    assert!(!went_to_graveyard(&t, spell));
    assert!(!t.in_graveyard(P0, "Writ of Return"));
}

#[test]
fn the_copy_is_cast_while_the_trigger_resolves_ignoring_sorcery_timing() {
    cr!("702.99a", "707.12", "608.2g");
    ruling!(
        "Writ of Return",
        "You cast the copy of the card with cipher during the resolution of the triggered ability. Ignore timing restrictions based on the card's type."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let first = t.graveyard(P0, "Hill Giant");
    cast_and_encode(
        &mut t,
        "Writ of Return",
        &[Entity::Object(first)],
        Some(bears),
    );
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    let second = t.graveyard(P0, "Llanowar Elves");
    // The Bears deal combat damage: the trigger goes on the stack in the combat damage
    // step.
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::CombatDamage && !g.stack.is_empty()
    });
    assert!(ok);
    assert_eq!(triggers_on_stack(&t, "Cipher"), 1);
    // Resolving the trigger casts the copy (a sorcery) in the combat damage step, without
    // paying its mana cost; it's on the stack when the trigger has finished resolving.
    t.answer_targets(P0, &[Entity::Object(second)]);
    let lands = tapped_lands(&t, P0);
    t.resolve();
    assert_eq!(t.g.turn.step, Step::CombatDamage);
    assert_eq!(t.stack_len(), 1);
    let copy = t.g.stack[0];
    assert!(t.obj(copy).is_spell());
    assert_eq!(t.obj(copy).chars.name, "Writ of Return");
    assert!(t.obj(copy).stack.as_ref().unwrap().cast.was_cast);
    assert_eq!(tapped_lands(&t, P0), lands);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn the_creature_is_chosen_on_resolution_and_isnt_targeted_by_cipher() {
    cr!("702.99a", "115.1", "702.18a");
    ruling!(
        "Stolen Identity",
        "You choose the creature as the spell resolves. The cipher ability doesn't target that creature, although the spell with cipher may target that creature (or a different creature) because of its other abilities."
    );
    // The spell targets the creature it's then encoded on.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = cast_and_encode(
        &mut t,
        "Stolen Identity",
        &[Entity::Object(bears)],
        Some(bears),
    );
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(encoded_on(&t.g, t.g.current(card)), Some(bears));

    // A creature with shroud (it can't be the target of spells or abilities) can be
    // chosen: the spell targets a different creature.
    supported("Argothian Enchantress");
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Argothian Enchantress");
    let theirs = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    let card = cast_and_encode(
        &mut t,
        "Stolen Identity",
        &[Entity::Object(theirs)],
        Some(wolf),
    );
    assert!(!target_candidates_since(&t, from)
        .iter()
        .any(|c| c.contains(&Entity::Object(wolf))));
    assert_eq!(
        choice_candidates(&t, from, "cipher"),
        vec![vec![
            Entity::Object(wolf),
            Entity::Object(tokens(&t, P0)[0])
        ]]
    );
    assert_eq!(encoded_on(&t.g, t.g.current(card)), Some(wolf));
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::BecameTarget { target: Entity::Object(o), .. } if *o == wolf)));
}

#[test]
fn only_a_creature_can_be_chosen_to_encode_the_card_on() {
    cr!("702.99a");
    ruling!(
        "Writ of Return",
        "You can choose only a creature to encode the card onto."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let saw = t.battlefield(P0, "Bone Saw");
    let forest = t.battlefield(P0, "Forest");
    let dead = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    cast_and_encode(
        &mut t,
        "Writ of Return",
        &[Entity::Object(dead)],
        Some(bears),
    );
    let giant = t.named_on_battlefield("Hill Giant")[0];
    let cands = choice_candidates(&t, from, "cipher");
    assert_eq!(cands.len(), 1);
    for c in &cands[0] {
        assert!(t.obj(c.object().unwrap()).is_creature());
    }
    assert!(cands[0].contains(&Entity::Object(bears)));
    assert!(cands[0].contains(&Entity::Object(giant)));
    for x in [copter, saw, forest] {
        assert!(!cands[0].contains(&Entity::Object(x)));
    }
}
