//! CR 702.103 Bestow.

use crate::common_k702_011_017::{assert_supported, attack_with};
use crate::common_k702_027_037::{can_cast, untapped_lands};
use crate::common_k702_052_066::{destroy, run_effect};
use crate::k702_001_010_common::can_attack;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::bestow::is_bestowed;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

const BESTOW: CastMethod = CastMethod::Keyword(KeywordKind::Bestow);

/// P0 casts Nyxborn Rollicker ({R} 1/1, bestow {1}{R}, "Enchanted creature gets +1/+1.")
/// bestowed on `target`, and returns the spell.
fn bestow_rollicker(t: &mut TestGame, target: ObjectId) -> (ObjectId, ObjectId) {
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Nyxborn Rollicker");
    let spell = t.cast(P0, c).method(BESTOW).target(target).go();
    (c, spell)
}

#[test]
fn a_spell_cast_bestowed_is_an_aura_spell_that_enchants_a_creature() {
    cr!("702.103", "702.103a", "702.103b");
    ruling!(
        "Boon Satyr",
        "A spell with bestow is either a creature spell or an Aura spell. It's never both. Similarly, a permanent with bestow is either a creature or an Aura, but not both."
    );
    ruling!(
        "Boon Satyr",
        "The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    assert_supported("Nyxborn Rollicker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (c, spell) = bestow_rollicker(&mut t, bears);
    // It paid its bestow cost rather than its mana cost.
    assert_eq!(untapped_lands(&t, P0), 0);
    let o = t.obj(spell);
    assert!(o.chars.is(CardType::Enchantment));
    assert!(!o.chars.is(CardType::Creature));
    assert!(o.chars.has_subtype("Aura"));
    assert!(!o.chars.has_subtype("Satyr"));
    assert!(o.chars.has_keyword(KeywordKind::Enchant));
    assert_eq!(t.g.mana_value_of(spell), 1);
    t.resolve_all();
    let aura = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert_eq!(t.obj(aura).attached_to, Some(Entity::Object(bears)));
    assert!(is_bestowed(&t.g, aura));
    assert!(!t.obj(aura).is_creature());
    assert!(t.obj(aura).chars.has_subtype("Aura"));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn a_bestowed_aura_spell_targets_a_creature() {
    cr!("702.103b");
    let mut t = TestGame::new(2);
    let saw = t.battlefield(P1, "Bone Saw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Nyxborn Rollicker");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, c).method(BESTOW).go();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            decision::Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert!(offered.contains(&Entity::Object(bears)));
    assert!(!offered.contains(&Entity::Object(saw)));
    assert!(!offered.contains(&Entity::Player(P1)));
}

#[test]
fn cast_normally_or_put_onto_the_battlefield_it_is_a_creature() {
    cr!("702.103a");
    ruling!(
        "Boon Satyr",
        "If a permanent with bestow enters the battlefield by any method other than being cast, it will be an enchantment creature."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let c = t.hand(P0, "Nyxborn Rollicker");
    let spell = t.cast(P0, c).go();
    assert!(t.obj(spell).chars.is(CardType::Creature));
    assert!(!t.obj(spell).chars.has_subtype("Aura"));
    t.resolve_all();
    let r = t.g.current(c);
    assert!(t.obj(r).is_creature() && t.obj(r).chars.is(CardType::Enchantment));
    assert!(!is_bestowed(&t.g, r));
    let other = t.enter(P0, "Nyxborn Rollicker");
    assert!(t.obj(other).is_creature());
    assert_eq!(t.obj(other).attached_to, None);
}

#[test]
fn a_bestowed_aura_spell_with_an_illegal_target_resolves_as_a_creature() {
    cr!("702.103e");
    ruling!(
        "Boon Satyr",
        "Unlike other Aura spells, an Aura spell with bestow still resolves if its target is illegal."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let (c, _) = bestow_rollicker(&mut t, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    let r = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert!(t.obj(r).is_creature());
    assert!(!t.obj(r).chars.has_subtype("Aura"));
    assert!(!t.obj(r).chars.has_keyword(KeywordKind::Enchant));
    assert_eq!(t.obj(r).attached_to, None);
    assert!(!is_bestowed(&t.g, r));
    assert_eq!(t.pt(r), (1, 1));
}

#[test]
fn a_bestowed_aura_whose_creature_leaves_becomes_a_creature() {
    cr!("702.103f");
    ruling!(
        "Boon Satyr",
        "Unlike other Auras, an Aura with bestow isn't put into its owner's graveyard if the enchanted creature leaves the battlefield or becomes an illegal creature for the Aura to enchant."
    );
    ruling!(
        "Boon Satyr",
        "It can attack (and its abilities can be activated, if it has any) on the turn it becomes unattached if it's been under your control continuously, even as an Aura, since your most recent turn began."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (c, _) = bestow_rollicker(&mut t, bears);
    t.resolve_all();
    // A turn later, the Bears die.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    destroy(&mut t, bears);
    t.settle();
    let r = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert!(t.obj(r).is_creature());
    assert_eq!(t.obj(r).attached_to, None);
    assert!(!is_bestowed(&t.g, r));
    // It's been under P0's control since the turn began: it can attack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, r));
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    assert!(t.g.is_attacking(r));
}

#[test]
fn a_bestowed_aura_attached_illegally_becomes_unattached_and_a_creature() {
    cr!("702.103f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (c, _) = bestow_rollicker(&mut t, bears);
    t.resolve_all();
    // The Bears stop being a creature: enchant creature can't enchant them.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    t.settle();
    let r = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj(r).attached_to, None);
    assert!(t.obj(r).is_creature());
    // An ordinary Aura would have been put into its owner's graveyard.
    assert!(!t.in_graveyard(P0, "Nyxborn Rollicker"));
}

#[test]
fn a_bestowed_aura_that_phases_in_unattached_becomes_a_creature() {
    cr!("702.103g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (c, _) = bestow_rollicker(&mut t, bears);
    t.resolve_all();
    let r = t.g.current(c);
    // The Aura phases out on its own; the Bears leave the battlefield meanwhile.
    keyword_impls::phase_out(&mut t.g, vec![r]);
    t.settle();
    assert!(t.obj(r).phased_out);
    destroy(&mut t, bears);
    t.settle();
    assert!(is_bestowed(&t.g, r));
    keyword_impls::phase_in(&mut t.g, r);
    t.settle();
    assert!(t.on_battlefield(c));
    assert!(!is_bestowed(&t.g, r));
    assert!(t.obj(r).is_creature());
    assert_eq!(t.obj(r).attached_to, None);
}

#[test]
fn only_the_bestowed_characteristics_are_checked_when_casting_it_bestowed() {
    cr!("702.103d");
    let mut t = TestGame::new(2);
    // Steel Golem: "You can't cast creature spells."
    t.battlefield(P0, "Steel Golem");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Nyxborn Rollicker");
    assert!(!can_cast(&mut t, P0, c, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, c, BESTOW));
    t.cast(P0, c).method(BESTOW).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn a_permission_to_cast_creature_spells_doesnt_allow_casting_it_bestowed() {
    cr!("702.103d");
    let mut t = TestGame::new(2);
    // Garruk's Horde: "You may cast creature spells from the top of your library."
    t.battlefield(P0, "Garruk's Horde");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let c = t.library_top(P0, "Nyxborn Rollicker");
    assert!(can_cast(&mut t, P0, c, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, c, BESTOW));
}

#[test]
fn a_bestowed_aura_with_flash_can_be_cast_at_instant_speed() {
    cr!("702.103a", "702.103b");
    assert_supported("Boon Satyr");
    let mut t = TestGame::new(2);
    // Boon Satyr: flash, bestow {3}{G}{G}, "Enchanted creature gets +4/+2."
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let c = t.hand(P0, "Boon Satyr");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(can_cast(&mut t, P0, c, BESTOW));
    t.cast(P0, c).method(BESTOW).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 4));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn a_copy_of_a_bestowed_aura_spell_is_bestowed() {
    cr!("702.103c");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    let (_, spell) = bestow_rollicker(&mut t, bears);
    // "{4}, {T}: Copy target permanent spell you control. (The copy becomes a token.)"
    t.activate(P0, engine, 1, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert!(t.obj(copy).chars.has_subtype("Aura"));
    assert!(!t.obj(copy).chars.is(CardType::Creature));
    t.resolve_all();
    // Two bestowed Auras (one a token) on the Bears.
    assert_eq!(t.pt(bears), (4, 4));
    let token = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|o| t.g.obj(*o).is_token())
        .unwrap();
    assert!(is_bestowed(&t.g, token));
    assert_eq!(t.obj(token).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn a_card_may_let_itself_be_cast_bestowed_from_a_graveyard() {
    cr!("702.103a");
    assert_supported("Detective's Phoenix");
    let mut t = TestGame::new(2);
    // Detective's Phoenix: "Bestow—{R}, Collect evidence 6." and "You may cast this card
    // from your graveyard using its bestow ability."
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let c = t.graveyard(P0, "Detective's Phoenix");
    // Evidence to collect: cards in the graveyard with total mana value 6 or more.
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Hill Giant");
    assert!(can_cast(&mut t, P0, c, BESTOW));
    assert!(!can_cast(&mut t, P0, c, CastMethod::Normal));
    t.cast(P0, c).method(BESTOW).target(bears).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0, "evidence was collected");
    assert_eq!(t.zone(c), Zone::Battlefield);
    assert_eq!(t.pt(bears), (4, 4));
}
