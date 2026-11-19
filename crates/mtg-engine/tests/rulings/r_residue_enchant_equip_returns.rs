//! Rulings residue (batches P207–P213, Auras and Equipment): Auras that return things (or
//! themselves) to the battlefield — Mantle of the Ancients, Reins of the Vinesteed,
//! Necrotic Plague, Gift of Immortality, Flickerform, Kaya's Ghostform, Traveling Plague,
//! Shifting Shadow, Nettlevine Blight, Steam Vines, Songbirds' Blessing — and Auras that
//! put cards from libraries onto the battlefield (Curse of Misfortunes, Curse of Unbinding,
//! Call to the Kindred), and Dead Man's Chest.

use crate::r_p212_common::grant;
use crate::r_residue_enchant_equip_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s03_common::respond;
use crate::r_s06_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Goes to the beginning of `p`'s upkeep (from the end of the other player's turn, in a
/// two-player game), with the upkeep triggers on the stack.
fn into_upkeep(t: &mut TestGame, p: PlayerId) {
    t.set_step(if p == P0 { P1 } else { P0 }, Step::End);
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// The candidates of every `ChooseEntities` decision asked since `from`.
fn chosen_among(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

// --- Mantle of the Ancients ---------------------------------------------------------------

const MANTLE: &str = "Mantle of the Ancients";

/// P0 casts Mantle of the Ancients on `host`; only the Aura spell resolves, and its
/// enters trigger targets `cards`. The Mantle.
fn mantle(t: &mut TestGame, host: ObjectId, cards: &[ObjectId]) -> ObjectId {
    supported(MANTLE);
    let m = crate::r_s03_common::in_hand_with_mana(t, P0, MANTLE);
    let targets: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.g.turn.priority = Some(P0);
    t.cast(P0, m).target(host).go();
    t.answer_targets(P0, &targets);
    t.g.resolve_top();
    t.settle();
    assert_eq!(triggers_on_stack(t, "return any number"), 1, "{}", t.dump_log());
    t.g.current(m)
}

#[test]
fn mantle_of_the_ancients_targets_that_cant_be_attached_stay_in_the_graveyard() {
    cr!("303.4i", "608.2b", "301.5c");
    ruling!(
        "Mantle of the Ancients",
        "any targets that couldn't be legally attached to the creature enchanted by Mantle of the Ancients stay in your graveyard."
    );
    // Captured by the Consulate ("Enchant creature you don't control") can't enchant P0's
    // Bears.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.graveyard(P0, "Holy Strength");
    let captured = t.graveyard(P0, "Captured by the Consulate");
    mantle(&mut t, bears, &[strength, captured]);
    t.resolve_all();
    assert_eq!(attached_to(&t, strength), Some(obj(bears)), "{}", t.dump_log());
    assert!(in_zone(&t, captured, Zone::Graveyard(P0)));
    assert_eq!(t.pt(bears), (2 + 1 + 2, 2 + 2 + 2));
}

#[test]
fn mantle_of_the_ancients_uses_the_creature_it_enchants_as_the_trigger_resolves() {
    cr!("608.2h", "303.4i");
    ruling!(
        "Mantle of the Ancients",
        "the creature it's attached to as that ability is resolving is the \"enchanted creature,\" even though you used the original enchanted creature to choose targets."
    );
    // Animate Wall ("Enchant Wall") can enchant the Wall the Mantle was cast on, but not
    // the Bears it's moved to.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.graveyard(P0, "Holy Strength");
    let animate = t.graveyard(P0, "Animate Wall");
    let m = mantle(&mut t, wall, &[strength, animate]);
    assert!(t.g.attach(m, obj(bears)));
    t.g.recompute();
    t.resolve_all();
    assert_eq!(attached_to(&t, strength), Some(obj(bears)), "{}", t.dump_log());
    assert!(in_zone(&t, animate, Zone::Graveyard(P0)));
}

#[test]
fn mantle_of_the_ancients_gone_uses_the_permanent_it_was_last_attached_to() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Mantle of the Ancients",
        "the last permanent it was attached to before leaving the battlefield is the \"enchanted creature\""
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.graveyard(P0, "Holy Strength");
    let m = mantle(&mut t, bears, &[strength]);
    destroy(&mut t, m);
    assert!(!t.on_battlefield(t.g.current(m)));
    t.resolve_all();
    assert_eq!(attached_to(&t, strength), Some(obj(bears)), "{}", t.dump_log());
}

// --- Reins of the Vinesteed ---------------------------------------------------------------

const REINS: &str = "Reins of the Vinesteed";

#[test]
fn reins_of_the_vinesteed_may_return_to_a_creature_sharing_any_type() {
    cr!("603.10a", "608.2h", "205.3m");
    ruling!(
        "Reins of the Vinesteed",
        "If that creature was a Goblin Rogue, for example, Reins of the Vinesteed can be returned to the battlefield attached to either a Goblin creature or a Rogue creature."
    );
    supported(REINS);
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Rummaging Goblin");
    let goblin = t.battlefield(P0, "Goblin Piker");
    let human = t.battlefield(P1, "Bane Alley Blackguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let reins = aura_on(&mut t, P0, REINS, rogue);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(human)]);
    let from = t.asked().len();
    destroy(&mut t, rogue);
    t.resolve_all();
    let offered = chosen_among(&t, from);
    assert_eq!(offered.len(), 1, "{}", t.dump_log());
    assert!(offered[0].contains(&obj(goblin)));
    assert!(offered[0].contains(&obj(human)));
    assert!(!offered[0].contains(&obj(bears)));
    assert_eq!(attached_to(&t, reins), Some(obj(human)));
}

#[test]
fn reins_of_the_vinesteed_triggers_whether_or_not_it_goes_too() {
    cr!("603.10a", "704.5m");
    ruling!(
        "Reins of the Vinesteed",
        "Reins of the Vinesteed's last ability triggers if it and the creature it's enchanting are both put into the graveyard at the same time"
    );
    for together in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let other = t.battlefield(P0, "Grizzly Bears");
        let reins = aura_on(&mut t, P0, REINS, bears);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(other)]);
        if together {
            t.g.destroy_all(vec![bears, reins], None, false);
            t.g.flush_events();
        } else {
            t.g.destroy(bears, None);
            t.g.flush_events();
        }
        t.settle();
        t.resolve_all();
        assert_eq!(attached_to(&t, reins), Some(obj(other)), "together {together}");
    }
}

#[test]
fn reins_of_the_vinesteed_doesnt_target_but_must_enchant_legally() {
    cr!("303.4f", "702.18a", "702.16c");
    ruling!(
        "Reins of the Vinesteed",
        "You may return Reins of the Vinesteed to the battlefield attached to a creature that has shroud, for example. However, the creature must be able to be legally enchanted by Reins of the Vinesteed."
    );
    supported("Mirran Crusader");
    // Fugitive Wizard (Human Wizard) dies: Benalish Heralds (Human Soldier) has shroud and
    // can be chosen; Mirran Crusader (Human Knight, protection from green) can't.
    let mut t = TestGame::new(2);
    let wizard = t.battlefield(P0, "Fugitive Wizard");
    let shrouded = t.battlefield(P0, "Benalish Heralds");
    grant(&mut t, shrouded, KeywordKind::Shroud);
    let crusader = t.battlefield(P0, "Mirran Crusader");
    let reins = aura_on(&mut t, P0, REINS, wizard);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(shrouded)]);
    let from = t.asked().len();
    destroy(&mut t, wizard);
    t.resolve_all();
    let offered = chosen_among(&t, from);
    assert!(offered.iter().any(|c| c.contains(&obj(shrouded))), "{offered:?}");
    assert!(!offered.iter().any(|c| c.contains(&obj(crusader))));
    assert_eq!(attached_to(&t, reins), Some(obj(shrouded)));
}

#[test]
fn reins_of_the_vinesteed_on_another_players_creature_stays_yours_and_retriggers() {
    cr!("303.4f", "603.10a", "110.2");
    ruling!(
        "Reins of the Vinesteed",
        "You can choose to return Reins of the Vinesteed to the battlefield attached to a creature another player controls. You still control Reins of the Vinesteed."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let reins = aura_on(&mut t, P0, REINS, bears);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(theirs)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(attached_to(&t, reins), Some(obj(theirs)));
    assert_eq!(t.obj_now(reins).controller, P0);
    assert_eq!(t.pt(theirs), (4, 4));
    // That creature dies: P0 moves the Reins again.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(mine)]);
    destroy(&mut t, theirs);
    t.resolve_all();
    assert_eq!(attached_to(&t, reins), Some(obj(mine)));
    assert_eq!(t.obj_now(reins).controller, P0);
}

// --- Necrotic Plague ----------------------------------------------------------------------

const PLAGUE: &str = "Necrotic Plague";

#[test]
fn necrotic_plague_sacrifice_happens_in_the_creature_controllers_upkeep() {
    cr!("113.6", "503.1a", "109.5");
    ruling!(
        "Necrotic Plague",
        "it triggers at the beginning of the upkeep of the creature's controller (not necessarily Necrotic Plague's controller)."
    );
    supported(PLAGUE);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    aura_on(&mut t, P0, PLAGUE, bears);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
}

#[test]
fn necrotic_plague_returns_whenever_the_creature_dies() {
    cr!("603.10a", "704.5m", "603.6c");
    ruling!(
        "Necrotic Plague",
        "Necrotic Plague's last ability triggers when the enchanted creature is put into a graveyard for any reason, not just when it's sacrificed due to the ability it grants to that creature."
    );
    for together in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        let plague = aura_on(&mut t, P0, PLAGUE, bears);
        t.answer_targets(P1, &[obj(giant)]);
        if together {
            t.g.destroy_all(vec![bears, plague], None, false);
        } else {
            t.g.destroy(bears, None);
        }
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        assert_eq!(attached_to(&t, plague), Some(obj(giant)), "together {together}");
    }
}

#[test]
fn necrotic_plague_returns_under_its_controller_when_the_creature_died() {
    cr!("603.3a", "110.2", "608.2c");
    ruling!(
        "Necrotic Plague",
        "The player that controls Necrotic Plague at the time the enchanted creature is put into a graveyard is the player who controls the last ability (even though that creature's controller is the one who chooses its target)."
    );
    // P1 owns the Plague but P0 controls it; it enchants P1's Bears. P1 (the Bears'
    // controller) chooses a creature one of P1's opponents controls; P0 returns the Plague
    // from P1's graveyard under P0's control.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Hill Giant");
    let plague = aura_on(&mut t, P1, PLAGUE, bears);
    give_control(&mut t, plague, P0);
    assert_eq!(t.obj_now(plague).controller, P0);
    t.answer_targets(P1, &[obj(giant)]);
    let from = t.asked().len();
    destroy(&mut t, bears);
    t.resolve_all();
    let asked: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P1]);
    assert_eq!(attached_to(&t, plague), Some(obj(giant)), "{}", t.dump_log());
    assert_eq!(t.obj_now(plague).controller, P0);
    assert_eq!(t.obj_now(plague).owner, P1);
}

// --- Gift of Immortality, Flickerform, Kaya's Ghostform, Traveling Plague ------------------

#[test]
fn gift_of_immortality_on_a_token_returns_nothing() {
    cr!("111.7", "111.8", "704.5d");
    ruling!(
        "Gift of Immortality",
        "If the enchanted creature is a token, neither it nor Gift of Immortality will return to the battlefield."
    );
    supported("Gift of Immortality");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tok = create_token(&mut t, P0, "Soldier");
    let gift = aura_on(&mut t, P0, "Gift of Immortality", tok);
    destroy(&mut t, tok);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Soldier").is_empty());
    assert!(in_zone(&t, gift, Zone::Graveyard(P0)), "{}", t.dump_log());
}

const FLICKERFORM: &str = "Flickerform";

/// P0 activates the Flickerform `ff` and the game goes to the end step, where the cards
/// return.
fn flicker(t: &mut TestGame, ff: ObjectId) {
    supported(FLICKERFORM);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 4);
    activate_containing(t, P0, ff, "Exile enchanted").expect("activate");
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

#[test]
fn flickerform_on_a_token_leaves_the_auras_in_exile() {
    cr!("111.7", "704.5d", "610.3c");
    ruling!(
        "Flickerform",
        "If the enchanted creature was a token, it ceased to exist when it was exiled. Any Auras that were attached to it (including Flickerform) remain exiled."
    );
    let mut t = TestGame::new(2);
    let tok = create_token(&mut t, P0, "Soldier");
    let ff = aura_on(&mut t, P0, FLICKERFORM, tok);
    let strength = aura_on(&mut t, P0, "Holy Strength", tok);
    flicker(&mut t, ff);
    assert!(with_subtype(&t, P0, "Soldier").is_empty());
    assert!(in_zone(&t, ff, Zone::Exile), "{}", t.dump_log());
    assert!(in_zone(&t, strength, Zone::Exile));
}

#[test]
fn flickerform_bestowed_auras_return_as_creatures() {
    cr!("702.103f", "303.4h", "610.3c");
    ruling!(
        "Flickerform",
        "If the enchanted creature was enchanted by any Auras with bestow or any Licids, those cards will return to the battlefield unattached as creatures."
    );
    supported("Nyxborn Rollicker");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ff = aura_on(&mut t, P0, FLICKERFORM, bears);
    t.lands(P0, "Mountain", 2);
    let rollicker = t.hand(P0, "Nyxborn Rollicker");
    t.cast(P0, rollicker)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(attached_to(&t, rollicker), Some(obj(bears)));
    flicker(&mut t, ff);
    let bears_now = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(attached_to(&t, ff), Some(obj(bears_now)), "{}", t.dump_log());
    let r = t.g.current(rollicker);
    assert!(t.on_battlefield(r));
    assert_eq!(attached_to(&t, r), None);
    assert!(t.obj_now(r).is(CardType::Creature));
}

#[test]
fn flickerform_returns_the_card_first_then_the_auras_that_can_enchant_it() {
    cr!("303.4f", "303.4g", "610.3c");
    ruling!(
        "Flickerform",
        "The card that was enchanted comes back onto the battlefield first, regardless of whether it’s still a creature. Then any Auras exiled that can legally enchant that card come back. Any Auras that can’t enchant that permanent remain exiled."
    );
    supported("Forbidden Lore");
    // An animated Treetop Village comes back as a land: Forbidden Lore ("Enchant land")
    // comes back attached to it, Flickerform ("Enchant creature") stays in exile.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let village = t.battlefield(P0, "Treetop Village");
    t.lands(P0, "Forest", 2);
    activate_containing(&mut t, P0, village, "becomes").expect("animate");
    t.resolve_all();
    let ff = aura_on(&mut t, P0, FLICKERFORM, village);
    let lore = aura_on(&mut t, P0, "Forbidden Lore", village);
    flicker(&mut t, ff);
    let v = t.g.current(village);
    assert!(t.on_battlefield(v), "{}", t.dump_log());
    assert!(!t.obj_now(v).is(CardType::Creature));
    assert_eq!(attached_to(&t, lore), Some(obj(v)));
    assert!(in_zone(&t, ff, Zone::Exile));
}

#[test]
fn flickerform_copy_enchantment_chooses_again_as_it_returns() {
    cr!("707.2", "303.4f", "614.12");
    ruling!(
        "Flickerform",
        "As Copy Enchantment returns to the battlefield, its controller may choose any enchantment on the battlefield for it to copy. It can’t copy Flickerform or any other Aura returning to the battlefield at the same time. If it copies a non-Aura enchantment, it returns to the battlefield unattached instead."
    );
    supported("Copy Enchantment");
    for copy_aura in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let strength = aura_on(&mut t, P1, "Holy Strength", giant);
        let anthem = t.battlefield(P1, "Glorious Anthem");
        let ff = aura_on(&mut t, P0, FLICKERFORM, bears);
        // Copy Enchantment, a copy of Holy Strength, on the Bears.
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(strength)]);
        t.answer_choose(P0, &[obj(bears)]);
        let ce = t.enter(P0, "Copy Enchantment");
        t.settle();
        assert_eq!(name(&t, ce), "Holy Strength");
        assert_eq!(attached_to(&t, ce), Some(obj(bears)));
        // As it returns, P0 chooses Holy Strength again, or Glorious Anthem.
        t.clear_answers();
        let pick = if copy_aura { strength } else { anthem };
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(pick)]);
        let from = t.asked().len();
        flicker(&mut t, ff);
        let ce = t.g.current(ce);
        let bears_now = t.named_on_battlefield("Grizzly Bears")[0];
        assert!(t.on_battlefield(ce), "{}", t.dump_log());
        for c in chosen_among(&t, from) {
            assert!(!c.contains(&obj(t.g.current(ff))), "{c:?}");
        }
        if copy_aura {
            assert_eq!(name(&t, ce), "Holy Strength");
            assert_eq!(attached_to(&t, ce), Some(obj(bears_now)));
        } else {
            assert_eq!(name(&t, ce), "Glorious Anthem");
            assert_eq!(attached_to(&t, ce), None);
        }
    }
}

const GHOSTFORM: &str = "Kaya's Ghostform";

#[test]
fn kayas_ghostform_card_that_left_its_new_zone_stays() {
    cr!("400.7", "608.2b", "603.10a");
    ruling!(
        "Kaya's Ghostform",
        "If the enchanted permanent is put into a graveyard or exile but leaves that zone before the last ability of Kaya’s Ghostform resolves, that card stays in its new zone"
    );
    // The Bears die, then are exiled from the graveyard before the trigger resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    aura_on(&mut t, P0, GHOSTFORM, bears);
    destroy(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, "return that card"), 1);
    let gy = t.g.current(bears);
    t.g.move_object(gy, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn kayas_ghostform_on_a_token_and_itself_dont_return() {
    cr!("111.7", "111.8", "303.4c");
    ruling!(
        "Kaya's Ghostform",
        "Kaya’s Ghostform can enchant a token, but its last ability won’t return the token to the battlefield."
    );
    ruling!(
        "Kaya's Ghostform",
        "Kaya’s Ghostform is put into your graveyard once the enchanted permanent leaves the battlefield. It doesn’t return to the battlefield with that permanent."
    );
    let mut t = TestGame::new(2);
    let tok = create_token(&mut t, P0, "Soldier");
    let g = aura_on(&mut t, P0, GHOSTFORM, tok);
    destroy(&mut t, tok);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Soldier").is_empty());
    assert!(in_zone(&t, g, Zone::Graveyard(P0)));
    // A nontoken creature returns; the Ghostform stays in the graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let g = aura_on(&mut t, P0, GHOSTFORM, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(in_zone(&t, g, Zone::Graveyard(P0)));
}

#[test]
fn traveling_plague_returns_with_no_counters() {
    cr!("704.5m", "122.2", "400.7");
    ruling!(
        "Traveling Plague",
        "The triggered ability returns this card to the battlefield with no counters on it."
    );
    supported("Traveling Plague");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let plague = aura_on(&mut t, P0, "Traveling Plague", wurm);
    t.g.add_counters(obj(plague), "plague", 2, None);
    t.g.recompute();
    assert_eq!(t.pt(wurm), (4, 2));
    t.answer_choose(P1, &[obj(giant)]);
    destroy(&mut t, wurm);
    assert!(in_zone(&t, plague, Zone::Graveyard(P0)) || t.on_battlefield(t.g.current(plague)));
    t.resolve_all();
    let p = t.g.current(plague);
    assert!(t.on_battlefield(p), "{}", t.dump_log());
    assert_eq!(t.counters(p, "plague"), 0);
    assert_eq!(attached_to(&t, p), Some(obj(giant)));
    assert_eq!(t.pt(giant), (3, 3));
}

// --- Shifting Shadow, Nettlevine Blight, Steam Vines ---------------------------------------

#[test]
fn shifting_shadow_gone_before_its_granted_trigger_resolves() {
    cr!("113.7a", "608.2h", "603.10");
    ruling!(
        "Shifting Shadow",
        "you'll destroy the creature it enchanted before it left the battlefield, then reveal cards and put a creature card onto the battlefield. Shifting Shadow won't become attached to it."
    );
    supported("Shifting Shadow");
    for shadow_leaves in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let shadow = aura_on(&mut t, P0, "Shifting Shadow", bears);
        stack_library(&mut t, P0, &["Island", "Hill Giant", "Forest"]);
        into_upkeep(&mut t, P0);
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        if shadow_leaves {
            destroy(&mut t, shadow);
        } else {
            t.g.move_object(
                bears,
                Zone::Exile,
                mtg_engine::events::MoveCause::Effect,
                None,
            );
            t.g.flush_events();
            t.settle();
        }
        t.resolve_all();
        let giant = t.named_on_battlefield("Hill Giant");
        assert_eq!(giant.len(), 1, "{}", t.dump_log());
        assert_eq!(attached_to(&t, shadow), None);
        assert!(in_zone(&t, shadow, Zone::Graveyard(P0)));
        if shadow_leaves {
            assert!(t.in_graveyard(P0, "Grizzly Bears"));
        } else {
            assert!(t.in_exile("Grizzly Bears"));
        }
    }
}

#[test]
fn nettlevine_blight_is_controlled_by_the_enchanted_permanents_controller() {
    cr!("113.6", "109.5", "701.3a");
    ruling!(
        "Nettlevine Blight",
        "Nettlevine Blight grants the triggered ability to the enchanted permanent, so “you” refers to that permanent’s controller. The ability will trigger at the end of that player’s turn, and that player chooses the new creature or land to attach Nettlevine Blight to. The player must choose a creature or land they control."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    let blight = aura_on(&mut t, P0, "Nettlevine Blight", bears);
    // P0's end step: nothing.
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // P1's end step: P1 sacrifices the Bears and chooses among what P1 controls.
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(theirs)]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(attached_to(&t, blight), Some(obj(theirs)), "{}", t.dump_log());
    assert_eq!(t.obj_now(blight).controller, P0);
    let asked: Vec<(PlayerId, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some((*p, candidates.clone())),
            _ => None,
        })
        .collect();
    assert!(!asked.is_empty());
    for (p, c) in asked {
        assert_eq!(p, P1);
        for e in c {
            let Entity::Object(o) = e else { panic!() };
            assert_eq!(t.g.obj(o).controller, P1);
        }
    }
}

#[test]
fn steam_vines_with_no_other_land_goes_to_the_graveyard() {
    cr!("704.5m", "701.3b");
    ruling!(
        "Steam Vines",
        "If there are no other lands to move it to, this card goes to the graveyard as would any Aura that lost the thing it enchants."
    );
    supported("Steam Vines");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Mountain");
    let vines = aura_on(&mut t, P0, "Steam Vines", land);
    crate::r_p209_common::tap(&mut t, land);
    t.resolve_all();
    assert!(!t.on_battlefield(land));
    assert_eq!(t.life(P1), 19);
    assert!(in_zone(&t, vines, Zone::Graveyard(P0)), "{}", t.dump_log());
}

// --- Songbirds' Blessing ------------------------------------------------------------------

#[test]
fn songbirds_blessing_aura_doesnt_target_but_must_be_able_to_enchant() {
    cr!("303.4f", "303.4g", "702.11b");
    ruling!(
        "Songbirds' Blessing",
        "An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, you won't be able to put it onto the battlefield, and it will be put into your hand by the next part of the effect."
    );
    supported("Songbirds' Blessing");
    // Holy Strength goes onto P1's hexproof Hill Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    grant(&mut t, giant, KeywordKind::Hexproof);
    aura_on(&mut t, P0, "Songbirds' Blessing", bears);
    let strength = t.library_top(P0, "Holy Strength");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(attached_to(&t, strength), Some(obj(giant)), "{}", t.dump_log());
    // Animate Wall can't enchant anything: it goes to the hand.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    aura_on(&mut t, P0, "Songbirds' Blessing", bears);
    let wall = t.library_top(P0, "Animate Wall");
    t.answer_yes(P0, true);
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert!(
        t.in_hand(P0, "Animate Wall"),
        "{:?} {}",
        t.zone(t.g.current(wall)),
        t.dump_log()
    );
}

// --- Curse of Misfortunes, Curse of Unbinding, Call to the Kindred -------------------------

#[test]
fn curse_of_misfortunes_name_check_is_on_resolution_and_sees_earlier_resolutions() {
    cr!("608.2c", "303.4f");
    ruling!(
        "Curse of Misfortunes",
        "The Curse card you put onto the battlefield must have a different name than each other Curse enchanting the enchanted player when the triggered ability resolves. If multiple of these abilities trigger, the Curse card you put onto the battlefield when the first resolves may impact your choices for each subsequent one."
    );
    supported("Curse of Misfortunes");
    let mut t = TestGame::new(2);
    aura_on(&mut t, P0, "Curse of Misfortunes", P1);
    aura_on(&mut t, P0, "Curse of Misfortunes", P1);
    t.library_top(P0, "Curse of Hospitality");
    t.library_top(P0, "Curse of Hospitality");
    t.library_top(P0, "Curse of Misfortunes");
    respond(&mut t, P0, |g, d| match d {
        Decision::YesNo { .. } => Some(Answer::Bool(true)),
        Decision::ChooseEntities { candidates, .. } => candidates
            .iter()
            .copied()
            .find(|e| matches!(e, Entity::Object(o) if g.obj(*o).chars.name == "Curse of Hospitality"))
            .map(|e| Answer::Entities(vec![e])),
        _ => None,
    });
    let from = t.asked().len();
    into_upkeep(&mut t, P0);
    t.resolve_all();
    let hosp = t.named_on_battlefield("Curse of Hospitality");
    assert_eq!(hosp.len(), 1, "{}", t.dump_log());
    assert_eq!(attached_to(&t, hosp[0]), Some(Entity::Player(P1)));
    assert_eq!(t.named_on_battlefield("Curse of Misfortunes").len(), 2);
    // No Curse of Misfortunes was ever offered; the second search offered no Curse of
    // Hospitality.
    let offered = chosen_among(&t, from);
    for c in &offered {
        for e in c {
            let Entity::Object(o) = e else { panic!() };
            assert_ne!(t.g.obj(*o).chars.name.as_str(), "Curse of Misfortunes");
        }
    }
    if let Some(last) = offered.get(1) {
        assert!(last.is_empty(), "{last:?}");
    }
}

#[test]
fn curse_of_unbinding_creature_enters_under_the_curse_controllers_control() {
    cr!("110.2a", "701.20a");
    ruling!(
        "Curse of Unbinding",
        "The controller of Curse of Unbinding puts the creature card onto the battlefield under their control. This may not be the same player as the enchanted player."
    );
    let mut t = TestGame::new(2);
    aura_on(&mut t, P0, "Curse of Unbinding", P1);
    stack_library(&mut t, P1, &["Island", "Hill Giant", "Forest"]);
    into_upkeep(&mut t, P1);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(giant[0]).controller, P0);
    assert_eq!(t.obj_now(giant[0]).owner, P1);
    assert!(t.in_graveyard(P1, "Island"));
}

#[test]
fn call_to_the_kindred_needs_only_one_shared_creature_type() {
    cr!("205.3m", "700.13");
    ruling!(
        "Call to the Kindred",
        "The creature need only share one creature type with enchanted creature, not necessarily all of them."
    );
    supported("Call to the Kindred");
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Alert Shu Infantry");
    aura_on(&mut t, P0, "Call to the Kindred", soldier);
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Fugitive Wizard",
            "Hearthfire Hobgoblin",
            "Affa Protector",
            "Grizzly Bears",
            "Island",
        ],
    );
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[1])]);
    let from = t.asked().len();
    into_upkeep(&mut t, P0);
    t.resolve_all();
    let offered = chosen_among(&t, from);
    let c = offered
        .iter()
        .find(|c| c.contains(&obj(lib[0])))
        .unwrap_or_else(|| panic!("{offered:?} {}", t.dump_log()));
    assert!(c.contains(&obj(lib[1])));
    assert!(c.contains(&obj(lib[2])));
    assert!(!c.contains(&obj(lib[3])));
    assert_eq!(t.named_on_battlefield("Hearthfire Hobgoblin").len(), 1);
}

// --- Dead Man's Chest ---------------------------------------------------------------------

const CHEST: &str = "Dead Man's Chest";

#[test]
fn dead_mans_chest_triggers_when_both_go_and_uses_last_known_power() {
    cr!("603.10a", "608.2h", "603.6c");
    ruling!(
        "Dead Man's Chest",
        "If Dead Man's Chest and the enchanted creature are both put into graveyards at the same time, the last ability of Dead Man's Chest still triggers."
    );
    ruling!(
        "Dead Man's Chest",
        "To determine how many cards to exile, use the enchanted creature's power as it last existed on the battlefield."
    );
    supported(CHEST);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p212_common::boost(&mut t, bears, 2, 0);
    let chest = aura_on(&mut t, P0, CHEST, bears);
    let lib = t.library_size(P1);
    t.g.destroy_all(vec![bears, chest], None, false);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 4, "{}", t.dump_log());
}

#[test]
fn dead_mans_chest_falls_off_when_you_gain_control_of_the_creature() {
    cr!("303.4c", "704.5m");
    ruling!(
        "Dead Man's Chest",
        "If you gain control of the enchanted creature, Dead Man's Chest will be put into your graveyard."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let chest = aura_on(&mut t, P0, CHEST, bears);
    give_control(&mut t, bears, P0);
    assert!(in_zone(&t, chest, Zone::Graveyard(P0)));
}

#[test]
fn dead_mans_chest_exiles_from_the_owners_library() {
    cr!("108.3", "111.2");
    ruling!(
        "Dead Man's Chest",
        "If Dead Man's Chest enchants a creature that you own but an opponent controls, you'll exile cards from your library."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    aura_on(&mut t, P0, CHEST, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.library_size(P0), l0 - 2);
    assert_eq!(t.library_size(P1), l1);
}
