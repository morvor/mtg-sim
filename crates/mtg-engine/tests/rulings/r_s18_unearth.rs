//! Rulings batch S18 — unearth (CR 702.84): "Unearth [cost]" means "[Cost]: Return this
//! card from your graveyard to the battlefield. It gains haste. Exile it at the beginning
//! of the next end step. If it would leave the battlefield, exile it instead of putting it
//! anywhere else. Activate only as a sorcery."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s18_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether the object (followed across zone changes) has haste now.
fn hasty(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::Haste)
}

// ---------------------------------------------------------------------------------------
// The card left the graveyard before the ability resolved.
// ---------------------------------------------------------------------------------------

/// P0 activates the unearth ability of `name`; P1 responds with Coffin Purge ("Exile
/// target card from a graveyard"). The unearth ability resolves and does nothing.
fn unearth_after_the_card_left(name: &str, cost: &str) {
    supported(name);
    supported("Coffin Purge");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, name);
    lands_for(&mut t, P0, cost);
    unearth(&mut t, P0, card).expect("unearth");
    t.lands(P1, "Swamp", 1);
    let purge = t.hand(P1, "Coffin Purge");
    t.cast(P1, purge).target(card).go();
    t.resolve();
    assert_eq!(t.zone(card), Zone::Exile);
    // The unearth ability is still on the stack; it resolves and does nothing.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.named_on_battlefield(name).is_empty());
}

#[test]
fn unearth_does_nothing_if_the_card_left_the_graveyard() {
    cr!("702.84a", "608.2b", "400.7");
    ruling!(
        "Phyrexian Dragon Engine",
        "If you activate a card's unearth ability but that card is removed from your graveyard before the ability resolves, that unearth ability will do nothing as it resolves."
    );
    ruling!(
        "Royal Warden",
        "If you activate a card's unearth ability but that card is removed from your graveyard before the ability resolves, that unearth ability will resolve and do nothing."
    );
    unearth_after_the_card_left("Phyrexian Dragon Engine", "{3}{R}{R}");
    unearth_after_the_card_left("Royal Warden", "{3}{B}");
}

// ---------------------------------------------------------------------------------------
// Haste, and the exile effects that aren't abilities of the permanent.
// ---------------------------------------------------------------------------------------

/// Unearths `name` (with March of the Machines first if it's a noncreature artifact), then
/// Humility makes it lose all its abilities: it's still exiled at the beginning of the next
/// end step, and if it would die, it's exiled instead.
fn exiled_even_without_abilities(name: &str, cost: &str, animate: bool) {
    for dies in [false, true] {
        let mut t = TestGame::new(2);
        if animate {
            t.battlefield(P0, "March of the Machines");
        }
        let id = unearthed(&mut t, P0, name, cost);
        t.battlefield(P1, "Humility");
        t.g.recompute();
        assert!(t.obj_now(id).chars.abilities.is_empty());
        assert!(!hasty(&t, id));
        if dies {
            destroy(&mut t, id);
        } else {
            t.advance_to(P0, Step::End);
            t.resolve_all();
        }
        assert!(t.in_exile(name), "{name} wasn't exiled");
        assert!(!t.in_graveyard(P0, name));
        assert!(t.named_on_battlefield(name).is_empty());
    }
}

#[test]
fn unearth_gives_haste_even_to_a_noncreature_permanent() {
    cr!("702.84a", "702.10b", "302.6");
    ruling!(
        "Mishra's Research Desk",
        "Unearth grants haste to the permanent that's returned to the battlefield (even if it's not a creature card). However, neither of the \"exile\" abilities is granted to that permanent. If that permanent loses all its abilities, it will still be exiled at the beginning of the next end step, and if it would leave the battlefield, it is still exiled instead."
    );
    supported("Mishra's Research Desk");
    supported("March of the Machines");
    supported("Clone");
    // Mishra's Research Desk (an artifact; unearth {1}{R}) has haste, although its
    // reminder text doesn't say so. Made a 1/1 creature by March of the Machines, it can
    // attack the turn it returned; another Desk that just entered can't.
    let mut t = TestGame::new(2);
    let desk = unearthed(&mut t, P0, "Mishra's Research Desk", "{1}{R}");
    assert!(hasty(&t, desk));
    assert!(t.obj_now(desk).summoning_sick);
    let other = t.enter(P0, "Mishra's Research Desk");
    t.battlefield(P0, "March of the Machines");
    t.g.recompute();
    assert_eq!(t.pt(desk), (1, 1));
    // The exile effects aren't abilities of the Desk: a Clone copying it has no haste and
    // isn't exiled at the end step.
    t.answer_choose(P0, &[Entity::Object(desk)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.g.recompute();
    assert_eq!(t.obj_now(clone).chars.name, "Mishra's Research Desk");
    assert!(!hasty(&t, clone));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, desk));
    assert!(!can_attack(&mut t, other));
    assert!(!can_attack(&mut t, clone));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(desk), Zone::Exile);
    assert!(t.on_battlefield(clone));
    assert!(t.on_battlefield(other));
    // Losing all its abilities doesn't stop it from being exiled.
    exiled_even_without_abilities("Mishra's Research Desk", "{1}{R}", true);
}

#[test]
fn unearth_gives_haste_but_not_the_exile_abilities() {
    cr!("702.84a", "302.6", "707.2", "613.1f");
    ruling!(
        "Chronomancer",
        "Unearth grants haste to the permanent that's returned to the battlefield. However, neither of the \"exile\" abilities is granted to that permanent. If that creature loses all its abilities, it will still be exiled at the beginning of the end step, and if it would leave the battlefield, it is still exiled instead."
    );
    supported("Chronomancer");
    supported("Humility");
    // Chronomancer ("{1}, {T}, Sacrifice another artifact: Draw a card.") can use its {T}
    // ability the turn it's unearthed.
    let mut t = TestGame::new(2);
    let chrono = unearthed(&mut t, P0, "Chronomancer", "{2}{B}");
    assert!(hasty(&t, chrono));
    t.battlefield(P0, "Memnite");
    t.battlefield(P0, "Memnite");
    t.lands(P0, "Wastes", 2);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, chrono, "Draw a card").expect("haste");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A Clone copying it gets neither haste nor the exile effects.
    t.answer_choose(P0, &[Entity::Object(chrono)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.g.recompute();
    assert_eq!(t.obj_now(clone).chars.name, "Chronomancer");
    assert!(!hasty(&t, clone));
    assert!(activate_containing(&mut t, P0, clone, "Draw a card").is_err());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(chrono), Zone::Exile);
    assert!(t.on_battlefield(clone));
    exiled_even_without_abilities("Chronomancer", "{2}{B}", false);
}

#[test]
fn an_unearthed_creature_has_haste_but_not_the_exile_abilities() {
    cr!("702.84a", "302.6", "707.2", "613.1f");
    ruling!(
        "Vithian Stinger",
        "Unearth grants haste to the creature that's returned to the battlefield. However, neither of the \"exile\" abilities is granted to that creature. If that creature loses all its abilities, it will still be exiled at the beginning of the end step, and if it would leave the battlefield, it is still exiled instead."
    );
    supported("Vithian Stinger");
    // Vithian Stinger ("{T}: This creature deals 1 damage to any target.") can tap the
    // turn it's unearthed.
    let mut t = TestGame::new(2);
    let stinger = unearthed(&mut t, P0, "Vithian Stinger", "{1}{R}");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, stinger, "damage").expect("haste");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // A copy of it doesn't have haste and stays at the end step.
    t.answer_choose(P0, &[Entity::Object(stinger)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.g.recompute();
    assert!(!hasty(&t, clone));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(activate_containing(&mut t, P0, clone, "damage").is_err());
    t.clear_answers();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(stinger), Zone::Exile);
    assert!(t.on_battlefield(clone));
    exiled_even_without_abilities("Vithian Stinger", "{1}{R}", false);
}

// ---------------------------------------------------------------------------------------
// Leaving the battlefield: exiled instead, unless it's being exiled.
// ---------------------------------------------------------------------------------------

/// The card, returned by `exile_and_return` (which exiles the unearthed permanent and
/// returns the card to the battlefield), is a new object: no haste, not exiled at the end
/// step, and it goes to the graveyard when it dies.
fn returned_card_is_a_new_object(
    name: &str,
    cost: &str,
    exile_and_return: fn(&mut TestGame, ObjectId),
) {
    let mut t = TestGame::new(2);
    let id = unearthed(&mut t, P0, name, cost);
    exile_and_return(&mut t, id);
    let back = t.g.current(id);
    assert_ne!(back, id);
    assert!(t.on_battlefield(back), "{name} didn't return");
    assert_eq!(t.obj_now(back).controller, P0);
    assert!(!hasty(&t, back));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(back));
    destroy(&mut t, back);
    assert!(t.in_graveyard(P0, name));
    assert!(!t.in_exile(name));
}

/// The unearthed permanent is exiled instead of going to its owner's hand (Unsummon) or
/// graveyard (Murder).
fn exiled_instead_of_leaving(name: &str, cost: &str) {
    for spell in ["Unsummon", "Murder"] {
        let mut t = TestGame::new(2);
        let id = unearthed(&mut t, P0, name, cost);
        give_mana_for(&mut t, P1, spell);
        let s = t.hand(P1, spell);
        t.cast(P1, s).target(id).go();
        t.resolve_all();
        assert!(t.in_exile(name), "{spell}: {name} wasn't exiled");
        assert!(!t.in_hand(P0, name));
        assert!(!t.in_graveyard(P0, name));
    }
}

/// P1's enchantment `name` enters and exiles `target` with its enters ability.
fn exile_with(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    t.answer_targets(P1, &[Entity::Object(target)]);
    let e = t.enter(P1, name);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(target), Zone::Exile, "{name} didn't exile it");
    e
}

#[test]
fn an_unearthed_permanent_exiled_by_static_net_returns_as_a_new_object() {
    cr!("702.84a", "400.7", "610.3");
    ruling!(
        "Combat Courier",
        "If a permanent returned to the battlefield with unearth would leave the battlefield for any reason, it's exiled instead—unless the spell or ability that's causing the permanent to leave the battlefield is actually trying to exile it! In that case, it succeeds at exiling it. If that spell or ability later returns the card to the battlefield (as Static Net might, for example), the permanent card will return to the battlefield as a new object with no relation to its previous existence. The unearth effects will no longer apply to it."
    );
    supported("Combat Courier");
    supported("Static Net");
    // Combat Courier ("{2}, Sacrifice this creature: Draw a card."; unearth {U}):
    // sacrificed, it's exiled instead.
    let mut t = TestGame::new(2);
    let courier = unearthed(&mut t, P0, "Combat Courier", "{U}");
    t.lands(P0, "Wastes", 2);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, courier, "Draw a card").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_exile("Combat Courier"));
    assert!(!t.in_graveyard(P0, "Combat Courier"));
    exiled_instead_of_leaving("Combat Courier", "{U}");
    // Static Net exiles it, and returns it when Static Net leaves the battlefield.
    returned_card_is_a_new_object("Combat Courier", "{U}", |t, id| {
        let net = exile_with(t, "Static Net", id);
        destroy(t, net);
        t.resolve_all();
    });
}

#[test]
fn an_unearthed_permanent_exiled_by_static_prison_returns_as_a_new_object() {
    cr!("702.84a", "400.7", "610.3");
    ruling!(
        "Molten Gatekeeper",
        "If that spell or ability later returns the card to the battlefield (as Static Prison might, for example), the permanent card will return to the battlefield as a new object with no relation to its previous existence. The unearth effects will no longer apply to it."
    );
    supported("Molten Gatekeeper");
    supported("Static Prison");
    exiled_instead_of_leaving("Molten Gatekeeper", "{R}");
    returned_card_is_a_new_object("Molten Gatekeeper", "{R}", |t, id| {
        let prison = exile_with(t, "Static Prison", id);
        destroy(t, prison);
        t.resolve_all();
    });
}

#[test]
fn an_unearthed_permanent_exiled_by_oblivion_ring_returns_as_a_new_object() {
    cr!("702.84a", "400.7", "610.3");
    ruling!(
        "Skorpekh Lord",
        "If a permanent returned to the battlefield by an unearth ability would leave it for any reason, it's exiled instead—unless the spell or ability that's causing it to leave the battlefield is actually trying to exile it. In that case, the spell or ability succeeds at exiling the permanent. If the spell or ability later returns the card to the battlefield, it will return as a new object with no relation to its previous existence. The unearth effect will no longer apply to it."
    );
    supported("Skorpekh Lord");
    supported("Oblivion Ring");
    exiled_instead_of_leaving("Skorpekh Lord", "{2}{B}");
    returned_card_is_a_new_object("Skorpekh Lord", "{2}{B}", |t, id| {
        let ring = exile_with(t, "Oblivion Ring", id);
        destroy(t, ring);
        t.resolve_all();
    });
}

#[test]
fn an_unearthed_creature_flickered_by_flickerwisp_returns_as_a_new_object() {
    cr!("702.84a", "400.7", "603.7c");
    ruling!(
        "Vithian Stinger",
        "If a creature returned to the battlefield with unearth would leave the battlefield for any reason, it's exiled instead — unless the spell or ability that's causing the creature to leave the battlefield is actually trying to exile it! In that case, it succeeds at exiling it. If it later returns the creature card to the battlefield (as Oblivion Ring or Flickerwisp might, for example), the creature card will return to the battlefield as a new object with no relation to its previous existence. The unearth effect will no longer apply to it."
    );
    supported("Flickerwisp");
    exiled_instead_of_leaving("Vithian Stinger", "{1}{R}");
    // Flickerwisp exiles the Stinger; it returns at the beginning of the end step — the
    // same end step at which unearth would have exiled it — as a new object that stays.
    let mut t = TestGame::new(2);
    let stinger = unearthed(&mut t, P0, "Vithian Stinger", "{1}{R}");
    t.answer_targets(P0, &[Entity::Object(stinger)]);
    t.enter(P0, "Flickerwisp");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(stinger), Zone::Exile);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.g.current(stinger);
    assert!(t.on_battlefield(back));
    assert!(!hasty(&t, back));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(back));
    destroy(&mut t, back);
    assert!(t.in_graveyard(P0, "Vithian Stinger"));
    // With Oblivion Ring too.
    returned_card_is_a_new_object("Vithian Stinger", "{1}{R}", |t, id| {
        let ring = exile_with(t, "Oblivion Ring", id);
        destroy(t, ring);
        t.resolve_all();
    });
}

#[test]
fn an_unearthed_creature_blinked_by_ephemerate_is_a_new_object() {
    cr!("702.84a", "400.7");
    ruling!(
        "Priest of Fell Rites",
        "If the spell or ability later returns the creature card to the battlefield (as Ephemerate might, for example), the creature card will return as a new object with no relation to its previous existence. The unearth effect will no longer apply to it."
    );
    supported("Priest of Fell Rites");
    supported("Ephemerate");
    exiled_instead_of_leaving("Priest of Fell Rites", "{3}{W}{B}");
    returned_card_is_a_new_object("Priest of Fell Rites", "{3}{W}{B}", |t, id| {
        t.lands(P0, "Plains", 1);
        let e = t.hand(P0, "Ephemerate");
        t.cast(P0, e).target(id).go();
        t.resolve_all();
    });
}

// ---------------------------------------------------------------------------------------
// Activating unearth isn't casting the card.
// ---------------------------------------------------------------------------------------

/// P0 activates the unearth ability of `name`: the card stays in the graveyard, and the
/// ability on the stack can't be targeted by the spell-only counterspell `spell_counter`
/// but can by `ability_counter` (cast in mode `mode`), which counters it: the card stays
/// in the graveyard.
fn unearth_is_an_activated_ability(
    name: &str,
    cost: &str,
    spell_counter: &str,
    ability_counter: &str,
    mode: Option<usize>,
) {
    supported(name);
    supported(spell_counter);
    supported(ability_counter);
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, name);
    lands_for(&mut t, P0, cost);
    unearth(&mut t, P0, card).expect("unearth");
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    let ability = top_of_stack(&t);
    assert!(matches!(
        t.g.obj(ability).stack.as_deref().map(|si| &si.kind),
        Some(StackKind::Activated { .. })
    ));
    assert!(!t.g.obj(ability).is_spell());
    let a = Entity::Object(ability);
    assert!(!spell_targets(&mut t, P1, spell_counter).contains(&a));
    if mode.is_none() {
        assert!(spell_targets(&mut t, P1, ability_counter).contains(&a));
    }
    give_mana_for(&mut t, P1, ability_counter);
    let c = t.hand(P1, ability_counter);
    let mut cast = t.cast(P1, c).target(a);
    if let Some(m) = mode {
        cast = cast.modes(&[m]);
    }
    cast.go();
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    assert!(t.named_on_battlefield(name).is_empty());
    // Cast as a spell, the card can be targeted by the spell counter.
    t.set_step(P0, Step::PrecombatMain);
    give_mana_for(&mut t, P0, name);
    let again = t.hand(P0, name);
    let spell = t.cast(P0, again).go();
    assert!(spell_targets(&mut t, P1, spell_counter).contains(&Entity::Object(spell)));
}

#[test]
fn activating_unearth_isnt_casting_defabricate_and_scatter_ray() {
    cr!("702.84a", "602.2", "112.1", "113.3b");
    ruling!(
        "Phyrexian Dragon Engine",
        "Activating a card's unearth ability isn't the same as casting that card. The unearth ability is put on the stack, but the card is not. Spells and abilities that interact with activated abilities (such as Defabricate's second mode) will interact with unearth, but spells and abilities that interact with spells (such as Scatter Ray) will not."
    );
    unearth_is_an_activated_ability(
        "Phyrexian Dragon Engine",
        "{3}{R}{R}",
        "Scatter Ray",
        "Defabricate",
        Some(1),
    );
}

#[test]
fn activating_unearth_isnt_casting_stifle_and_cancel() {
    cr!("702.84a", "602.2", "112.1", "113.3b");
    ruling!(
        "Royal Warden",
        "Activating a permanent card's unearth ability isn't the same as casting it as a spell. The unearth ability is put on the stack, but the card is not. Spells and abilities that interact with activated abilities (such as Stifle) will interact with unearth, but spells and abilities that interact with spells (such as Cancel) will not."
    );
    ruling!(
        "Priest of Fell Rites",
        "Activating a creature card's unearth ability isn't the same as casting the creature card. The unearth ability is put on the stack, but the creature card is not. Spells and abilities that interact with activated abilities (such as Stifle) will interact with unearth, but spells and abilities that interact with spells (such as Cancel) will not."
    );
    unearth_is_an_activated_ability("Royal Warden", "{3}{B}", "Cancel", "Stifle", None);
    unearth_is_an_activated_ability("Priest of Fell Rites", "{3}{W}{B}", "Cancel", "Stifle", None);
}

#[test]
fn activating_unearth_isnt_casting_stifle_and_remove_soul() {
    cr!("702.84a", "602.2", "112.1", "113.3b");
    ruling!(
        "Vithian Stinger",
        "Spells and abilities that interact with activated abilities (such as Stifle) will interact with unearth, but spells and abilities that interact with spells (such as Remove Soul) will not."
    );
    unearth_is_an_activated_ability("Vithian Stinger", "{1}{R}", "Remove Soul", "Stifle", None);
}

#[test]
fn activating_unearth_isnt_casting_flare_of_denial() {
    cr!("702.84a", "602.2", "112.1", "113.3b");
    ruling!(
        "Molten Gatekeeper",
        "Activating a card's unearth ability isn't the same as casting that card. The unearth ability is put on the stack, but the card is not. Spells and abilities that interact with activated abilities will interact with unearth, but spells and abilities that interact with spells (such as Flare of Denial) will not."
    );
    unearth_is_an_activated_ability("Molten Gatekeeper", "{R}", "Flare of Denial", "Stifle", None);
}

// ---------------------------------------------------------------------------------------
// The delayed triggered ability can be countered.
// ---------------------------------------------------------------------------------------

/// Unearths `name`; at the beginning of the end step, P0 counters the delayed triggered
/// ability with `counter` (in mode `mode`): the permanent stays, isn't exiled at the next
/// end step, and is still exiled if it later dies.
fn delayed_trigger_countered(name: &str, cost: &str, counter: &str, mode: Option<usize>) {
    supported(name);
    supported(counter);
    let mut t = TestGame::new(2);
    let id = unearthed(&mut t, P0, name, cost);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trigger = top_of_stack(&t);
    assert!(matches!(
        t.g.obj(trigger).stack.as_deref().map(|si| &si.kind),
        Some(StackKind::Triggered { .. })
    ));
    give_mana_for(&mut t, P0, counter);
    let c = t.hand(P0, counter);
    let mut cast = t.cast(P0, c).target(Entity::Object(trigger));
    if let Some(m) = mode {
        cast = cast.modes(&[m]);
    }
    cast.go();
    t.resolve_all();
    assert!(t.on_battlefield(id), "{name} was exiled");
    // It doesn't trigger again.
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(id));
    // The replacement effect still applies.
    destroy(&mut t, id);
    assert!(t.in_exile(name));
    assert!(!t.in_graveyard(P0, name));
}

#[test]
fn the_delayed_trigger_can_be_countered_by_defabricate() {
    cr!("702.84a", "603.7", "701.6b");
    ruling!(
        "Phyrexian Dragon Engine",
        "At the beginning of the next end step, a permanent returned to the battlefield with unearth is exiled. This is a delayed triggered ability, and it can be countered by effects such as Defabricate that counter triggered abilities. If the ability is countered, the permanent will stay on the battlefield and the delayed triggered ability won't trigger again. However, the replacement effect will still exile the permanent if it eventually leaves the battlefield."
    );
    delayed_trigger_countered("Phyrexian Dragon Engine", "{3}{R}{R}", "Defabricate", Some(1));
}

#[test]
fn the_delayed_trigger_can_be_countered() {
    cr!("702.84a", "603.7", "701.6b");
    ruling!(
        "Royal Warden",
        "At the beginning of the end step, a permanent returned to the battlefield with unearth is exiled. This is a delayed triggered ability, and it can be countered by effects that counter triggered abilities. If the ability is countered, the permanent will stay on the battlefield and the delayed trigger won't trigger again. However, the replacement effect will still exile it if it eventually leaves the battlefield."
    );
    ruling!(
        "Molten Gatekeeper",
        "At the beginning of the next end step, a permanent returned to the battlefield with unearth is exiled. This is a delayed triggered ability, and it can be countered by effects that counter triggered abilities. If the ability is countered, the permanent will stay on the battlefield and the delayed triggered ability won't trigger again. However, the replacement effect will still exile the permanent if it eventually leaves the battlefield."
    );
    delayed_trigger_countered("Royal Warden", "{3}{B}", "Stifle", None);
    delayed_trigger_countered("Molten Gatekeeper", "{R}", "Stifle", None);
}

#[test]
fn the_delayed_trigger_can_be_countered_by_stifle_or_voidslime() {
    cr!("702.84a", "603.7", "701.6b");
    ruling!(
        "Vithian Stinger",
        "At the beginning of the end step, a creature returned to the battlefield with unearth is exiled. This is a delayed triggered ability, and it can be countered by effects such as Stifle or Voidslime that counter triggered abilities. If the ability is countered, the creature will stay on the battlefield and the delayed trigger won't trigger again. However, the replacement effect will still exile the creature when it eventually leaves the battlefield."
    );
    delayed_trigger_countered("Vithian Stinger", "{1}{R}", "Stifle", None);
    delayed_trigger_countered("Vithian Stinger", "{1}{R}", "Voidslime", None);
}
