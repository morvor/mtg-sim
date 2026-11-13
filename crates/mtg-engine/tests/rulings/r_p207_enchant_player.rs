//! Rulings batch P207 — enchant player / enchant opponent (CR 303.4, 702.5d): Grievous
//! Wound's damage trigger, Psychic Possession's targeting and attachment, and Curse of
//! Exhaustion's spell limit, which looks at the whole turn.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s06_common::*;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's `name` attached to `p`.
fn curse(t: &mut TestGame, name: &str, p: PlayerId) -> ObjectId {
    supported(name);
    attach_new(t, P0, name, Entity::Player(p))
}

fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}

// ---------------------------------------------------------------------------------------
// Grievous Wound
// ---------------------------------------------------------------------------------------

#[test]
fn grievous_wound_triggers_after_the_damage_is_dealt() {
    cr!("603.2", "120.3a", "107.1a");
    ruling!(
        "Grievous Wound",
        "Grievous Wound's last ability triggers and resolves after the damage is dealt. For example, if the enchanted player has 10 life and is dealt 1 damage, that damage will reduce their life total to 9. Then Grievous Wound's ability will cause that player to lose 5 life, leaving their life total at 4."
    );
    let mut t = TestGame::new(2);
    curse(&mut t, "Grievous Wound", P1);
    set_life(&mut t, P1, 10);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    t.g.resolve_top();
    t.g.flush_events();
    assert_eq!(t.life(P1), 9);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "half their life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 4);
    // P0 isn't enchanted: damage to P0 doesn't trigger it.
    t.g.untap(pyro);
    t.activate(P0, pyro, 0, &[Entity::Player(P0)]).unwrap();
    t.g.resolve_top();
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "half their life"), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 4);
    // (Its first ability: the enchanted player can't gain life; P0 can.)
    t.g.gain_life(P1, 3);
    t.g.gain_life(P0, 3);
    assert_eq!(t.life(P1), 4);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn grievous_wound_triggers_once_for_simultaneous_combat_damage() {
    cr!("603.2c", "510.2", "120.3a");
    ruling!(
        "Grievous Wound",
        "Grievous Wound's last ability triggers only once whenever enchanted player is dealt combat damage, no matter how many creatures deal combat damage to them at the same time."
    );
    let mut t = TestGame::new(2);
    curse(&mut t, "Grievous Wound", P1);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    t.advance_to_step(mtg_engine::turn::Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 16);
    assert_eq!(triggers_on_stack(&t, "half their life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 8);
}

// ---------------------------------------------------------------------------------------
// Psychic Possession
// ---------------------------------------------------------------------------------------

#[test]
fn an_enchant_opponent_aura_cant_be_attached_to_a_permanent() {
    cr!("303.4a", "303.4d", "702.5d", "704.5m");
    ruling!(
        "Psychic Possession",
        "An Aura with enchant opponent can't be attached to a permanent."
    );
    supported("Psychic Possession");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let possession = curse(&mut t, "Psychic Possession", P1);
    // Moving it onto a permanent is illegal: it's put into its owner's graveyard.
    t.g.objects[possession.0 as usize].attached_to = Some(Entity::Object(bears));
    t.g.recompute();
    t.settle();
    assert!(!t.on_battlefield(possession));
    assert!(t.in_graveyard(P0, "Psychic Possession"));
    // As a spell, it can target only an opponent.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let card = in_hand_with_mana(&mut t, P0, "Psychic Possession");
    let from = t.asked().len();
    t.cast(P0, card).target(Entity::Player(P1)).go();
    assert_eq!(
        target_candidates(&t, P0, from),
        vec![vec![Entity::Player(P1)]]
    );
}

#[test]
fn psychic_possession_targets_the_opponent_only_as_a_spell() {
    cr!("303.4a", "702.5d", "702.11c", "115.1");
    ruling!(
        "Psychic Possession",
        "Enchanting an opponent works very much like enchanting a permanent. The Aura spell targets the opponent. When it resolves, it enters \"attached\" to that player. Once it's on the battlefield, it no longer targets that player."
    );
    supported("Leyline of Sanctity");
    let mut t = TestGame::new(2);
    let card = in_hand_with_mana(&mut t, P0, "Psychic Possession");
    t.cast(P0, card).target(Entity::Player(P1)).go();
    t.resolve_all();
    let possession = t.g.current(card);
    assert_eq!(attached_to(&t, possession), Some(Entity::Player(P1)));
    // P1 gets hexproof (Leyline of Sanctity): it stays attached.
    t.battlefield(P1, "Leyline of Sanctity");
    t.g.recompute();
    t.settle();
    assert_eq!(attached_to(&t, possession), Some(Entity::Player(P1)));
    // Whenever P1 draws a card, P0 may draw a card.
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.g.draw_cards(P1, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A hexproof player can't be targeted by a new one.
    let mut t2 = TestGame::new(2);
    t2.battlefield(P1, "Leyline of Sanctity");
    let card = in_hand_with_mana(&mut t2, P0, "Psychic Possession");
    assert!(!can_cast(&mut t2, P0, card, CastMethod::Normal));
}

// ---------------------------------------------------------------------------------------
// Curse of Exhaustion
// ---------------------------------------------------------------------------------------

#[test]
fn curse_of_exhaustion_counts_spells_cast_before_it_entered() {
    cr!("101.2", "604.1", "303.4");
    ruling!(
        "Curse of Exhaustion",
        "Curse of Exhaustion will look at the entire turn to see if the enchanted player has cast a spell yet that turn, even if Curse of Exhaustion wasn’t on the battlefield when that spell was cast."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    let gg = t.hand(P1, "Giant Growth");
    t.g.turn.priority = Some(P1);
    t.cast(P1, gg).target(bears).go();
    t.resolve_all();
    curse(&mut t, "Curse of Exhaustion", P1);
    let gg2 = t.hand(P1, "Giant Growth");
    assert!(!can_cast(&mut t, P1, gg2, CastMethod::Normal));
    // P0 isn't limited.
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    assert!(can_cast(&mut t, P0, bolt, CastMethod::Normal));
    // Next turn, P1 can cast one spell.
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    assert!(can_cast(&mut t, P1, gg2, CastMethod::Normal));
}

// ---------------------------------------------------------------------------------------
// Fraying Sanity
// ---------------------------------------------------------------------------------------

#[test]
fn fraying_sanity_counts_every_card_put_into_the_graveyard_this_turn() {
    cr!("404.1", "701.17a", "111.1");
    ruling!(
        "Fraying Sanity",
        "Fraying Sanity's triggered ability counts the number of cards that were put into the enchanted player's graveyard during the turn, even if Fraying Sanity wasn't on the battlefield at the time those cards were put there, and even if those cards have left that graveyard."
    );
    let mut t = TestGame::new(2);
    // Before the Curse: two of P1's cards are put into their graveyard (one is then
    // exiled), and a token of P1's dies (not a card).
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    destroy(&mut t, bears);
    destroy(&mut t, giant);
    crate::r_s05_common::move_to(&mut t, giant, mtg_engine::object::Zone::Exile);
    let token = create_token(&mut t, P1, "Soldier");
    destroy(&mut t, token);
    // A card put into P0's graveyard doesn't count.
    let mine = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, mine);
    curse(&mut t, "Fraying Sanity", P1);
    let library = t.library_size(P1);
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.resolve_all();
    assert_eq!(t.library_size(P1), library - 2);
}
