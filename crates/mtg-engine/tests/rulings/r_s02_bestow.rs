//! Rulings batch S02 — bestow (CR 702.103): "You may cast this card by paying [cost] rather
//! than its mana cost. If you do, it's an Aura spell with enchant creature; it becomes a
//! creature again if it's not attached."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::ability::Duration;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::bestow::is_bestowed;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

const BESTOW: CastMethod = CastMethod::Keyword(KeywordKind::Bestow);

/// P0 casts the real card `name` bestowed on `target` (with lands for its bestow cost
/// `lands`), returning (card, spell).
fn bestow(
    t: &mut TestGame,
    name: &str,
    lands: &[(&str, usize)],
    target: ObjectId,
) -> (ObjectId, ObjectId) {
    supported(name);
    for (land, n) in lands {
        t.lands(P0, land, *n);
    }
    let c = t.hand(P0, name);
    let spell = t.cast(P0, c).method(BESTOW).target(target).go();
    (c, spell)
}

#[test]
fn a_bestow_spell_is_an_enchantment_spell_either_way() {
    cr!("702.103a", "702.103b");
    ruling!(
        "Leafcrown Dryad",
        "On the stack, a spell with bestow is either a creature spell or an Aura spell. It's never both, although it's an enchantment spell in either case."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Leafcrown Dryad ({1}{G} 2/2 enchantment creature, bestow {3}{G}): "Enchanted
    // creature gets +2/+2 and has reach."
    let (_, spell) = bestow(&mut t, "Leafcrown Dryad", &[("Forest", 4)], bears);
    let o = t.obj(spell);
    assert!(o.chars.is(CardType::Enchantment));
    assert!(o.chars.has_subtype("Aura"));
    assert!(!o.chars.is(CardType::Creature));
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    // Cast normally, it's an enchantment creature spell, not an Aura spell.
    t.lands(P0, "Forest", 2);
    let c = t.hand(P0, "Leafcrown Dryad");
    let spell = t.cast(P0, c).go();
    let o = t.obj(spell);
    assert!(o.chars.is(CardType::Enchantment));
    assert!(o.chars.is(CardType::Creature));
    assert!(!o.chars.has_subtype("Aura"));
}

#[test]
fn a_bestow_aura_spell_with_an_illegal_target_resolves_as_an_enchantment_creature() {
    cr!("702.103e", "608.3b");
    ruling!(
        "Leafcrown Dryad",
        "Unlike other Aura spells, an Aura spell with bestow isn't countered if its target is illegal as it begins to resolve. Rather, the effect making it an Aura spell ends, it loses enchant creature, it returns to being an enchantment creature spell, and it resolves and enters the battlefield as an enchantment creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (c, spell) = bestow(&mut t, "Leafcrown Dryad", &[("Forest", 4)], bears);
    destroy(&mut t, bears);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(c));
    let dryad = t.g.current(c);
    let o = t.obj(dryad);
    assert!(o.chars.is(CardType::Enchantment) && o.is_creature());
    assert!(!o.chars.has_subtype("Aura"));
    assert!(!o.chars.has_keyword(KeywordKind::Enchant));
    assert!(o.chars.has_keyword(KeywordKind::Reach));
    assert!(!is_bestowed(&t.g, dryad));
    assert_eq!(t.pt(dryad), (2, 2));
}

/// P0's `name` bestowed on P0's Bears during P0's turn 1; on P0's next turn the Bears
/// leave the battlefield by `remove`. The permanent stays as an enchantment creature that
/// can attack that turn.
fn unattached_bestow_creature_can_attack(
    name: &str,
    lands: &[(&str, usize)],
    remove: fn(&mut TestGame, ObjectId),
) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (c, _) = bestow(&mut t, name, lands, bears);
    t.resolve_all();
    let aura = t.g.current(c);
    assert!(is_bestowed(&t.g, aura));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    remove(&mut t, bears);
    t.settle();
    let now = t.g.current(c);
    // The same object, now an enchantment creature (not put into the graveyard).
    assert_eq!(now, aura);
    assert!(t.on_battlefield(c));
    assert!(!t.in_graveyard(P0, name));
    let o = t.obj(now);
    assert!(o.is_creature() && o.chars.is(CardType::Enchantment));
    assert!(!o.chars.has_subtype("Aura"));
    assert_eq!(o.attached_to, None);
    assert!(!o.tapped);
    // It's been under P0's control continuously since the turn began: it can attack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, now));
    attack_with(&mut t, &[(now, Entity::Player(P1))]);
    assert!(t.g.is_attacking(now));
}

#[test]
fn an_unattached_bestow_aura_stays_as_a_creature_that_can_attack() {
    cr!("702.103f", "302.6");
    ruling!(
        "Hopeful Eidolon",
        "Unlike other Auras, an Aura with bestow isn't put into its owner's graveyard if it becomes unattached. Rather, the effect making it an Aura ends, it loses enchant creature, and it remains on the battlefield as an enchantment creature. It can attack (and its {T} abilities can be activated, if it has any) on the turn it becomes unattached if it's been under your control continuously, even as an Aura, since your most recent turn began."
    );
    // Hopeful Eidolon ({W} 1/1 lifelink, bestow {3}{W}); the Bears are destroyed.
    unattached_bestow_creature_can_attack("Hopeful Eidolon", &[("Plains", 4)], |t, b| {
        destroy(t, b)
    });
}

#[test]
fn an_unattached_bestow_aura_stays_as_a_creature_that_can_attack_after_an_exile() {
    cr!("702.103f", "302.6");
    ruling!(
        "Indebted Spirit",
        "Unlike other Auras, an Aura with bestow isn't put into its owner's graveyard if it becomes unattached. Rather, the effect making it an Aura ends, it loses enchant creature, and it remains on the battlefield as an enchantment creature."
    );
    // Indebted Spirit ({W} 1/1 afterlife 1, bestow {2}{W}); the Bears are exiled.
    unattached_bestow_creature_can_attack("Indebted Spirit", &[("Plains", 3)], |t, b| {
        let b = t.g.current(b);
        t.g.move_object(
            b,
            Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            Some(P1),
        );
    });
}

#[test]
fn you_control_a_bestow_aura_on_an_opponents_creature_and_the_creature_it_becomes() {
    cr!("303.4e", "702.103f");
    ruling!(
        "Gnarled Scarhide",
        "You still control the Aura, even if it's enchanting a creature controlled by another player."
    );
    ruling!(
        "Gnarled Scarhide",
        "If the enchanted creature leaves the battlefield, the Aura stops being an Aura and remains on the battlefield. Control of that permanent doesn't change; you'll control the resulting enchantment creature."
    );
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    // Gnarled Scarhide ({B} 2/1 bestow {3}{B}): "This creature can't block. Enchanted
    // creature gets +2/+1 and can't block."
    let (c, _) = bestow(&mut t, "Gnarled Scarhide", &[("Swamp", 4)], theirs);
    t.resolve_all();
    let aura = t.g.current(c);
    assert_eq!(t.obj(aura).attached_to, Some(Entity::Object(theirs)));
    assert_eq!(t.obj(aura).controller, P0);
    assert_eq!(t.obj(theirs).controller, P1);
    assert_eq!(t.pt(theirs), (5, 4));
    // The Aura's controller's opponent's creature can't block.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!t.g.can_block(theirs, bears));
    // The Giant leaves: the Scarhide stays, an enchantment creature P0 controls.
    destroy(&mut t, theirs);
    let now = t.g.current(c);
    assert_eq!(now, aura);
    assert!(t.on_battlefield(c));
    assert!(t.obj(now).is_creature());
    assert_eq!(t.obj(now).controller, P0);
    assert_eq!(t.pt(now), (2, 1));
    assert!(creatures(&t, P0).contains(&now));
}

#[test]
fn a_bestow_spell_targeting_an_opponents_creature_that_becomes_illegal_enters_under_your_control(
) {
    cr!("702.103e", "608.3b");
    ruling!(
        "Gnarled Scarhide",
        "Similarly, if you cast an Aura spell with bestow targeting a creature controlled by another player, and that creature is an illegal target when the spell tries to resolve, it will finish resolving as an enchantment creature spell. It will enter the battlefield under your control."
    );
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    let (c, _) = bestow(&mut t, "Gnarled Scarhide", &[("Swamp", 4)], theirs);
    destroy(&mut t, theirs);
    t.resolve_all();
    assert!(t.on_battlefield(c));
    let now = t.g.current(c);
    assert!(t.obj(now).is_creature());
    assert_eq!(t.obj(now).controller, P0);
    assert_eq!(t.obj(now).attached_to, None);
    assert_eq!(t.pt(now), (2, 1));
}

#[test]
fn bestow_cant_be_combined_with_casting_without_paying_the_mana_cost() {
    cr!("702.103a", "118.9a");
    ruling!(
        "Celestial Archon",
        "Bestow is an alternative cost to cast the spell with bestow. It can't be combined with other alternative costs, such as casting a spell \"without paying its mana cost.\""
    );
    supported("Celestial Archon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, "Celestial Archon");
    // P0 may cast it without paying its mana cost.
    t.g.play_grants.push(mtg_engine::casting::PlayGrant {
        player: P0,
        object: c,
        duration: Duration::EndOfTurn,
        free: true,
        source: None,
        turn: 1,
    });
    assert!(can_cast(&mut t, P0, c, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, c, BESTOW));
    // Bestowing it means paying its bestow cost ({5}{W}{W}).
    t.lands(P0, "Plains", 7);
    assert!(can_cast(&mut t, P0, c, BESTOW));
    // Cast for free, it's a creature spell.
    let spell = t.cast(P0, c).method(CastMethod::Free).go();
    assert_eq!(tapped_lands(&t, P0), 0);
    assert!(t.obj(spell).chars.is(CardType::Creature));
    assert!(!t.obj(spell).chars.has_subtype("Aura"));
}
