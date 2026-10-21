//! Rulings batch P094 — costs and casting permissions: Emet-Selch's graveyard cost
//! reduction and cast trigger, Licia's reduction and once-a-turn ability, Marshland
//! Bloodcaster's "pay life rather than the mana cost", Demonic Embrace's graveyard
//! permission, Mystic Forge's top card, and Cramped Vents with deathtouch.

use crate::r_s01_common::{custom_card, supported, triggers_on_stack};
use crate::r_s02_common::{can_activate, can_cast, can_play_land};
use mtg_engine::facedown::can_look_at;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
/// The alternative cost an effect offers the next spell (`kw/offered_costs.rs`).
const OFFERED: CastMethod =
    CastMethod::Alternative(mtg_engine::kw::offered_costs::OFFERED_ALT_COST);

#[test]
fn emet_selch_reduces_only_generic_mana_of_graveyard_spells() {
    cr!("601.2f", "702.34a");
    ruling!(
        "Emet-Selch of the Third Seat",
        "The cost reduction applies only to generic mana in the total cost of spells you cast from your graveyard."
    );
    supported("Emet-Selch of the Third Seat");
    // Think Twice: flashback {2}{U} becomes {U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emet-Selch of the Third Seat");
    let tt = t.graveyard(P0, "Think Twice");
    assert!(!can_cast(&mut t, P0, tt, FLASHBACK));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, tt, FLASHBACK));
    // From the hand there's no reduction: {1}{U} needs two lands.
    let hand = t.hand(P0, "Think Twice");
    assert!(!can_cast(&mut t, P0, hand, CastMethod::Normal));
    t.cast(P0, tt).method(FLASHBACK).go();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn emet_selch_increases_then_reductions_mana_value_unchanged() {
    cr!("601.2f", "202.3", "702.34a");
    ruling!(
        "Emet-Selch of the Third Seat",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of Emet-Selch of the Third Seat's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Thalia, Guardian of Thraben");
    supported("Firebolt");
    // Firebolt ({R}, flashback {4}{R}) with Thalia (+{1}) and Emet-Selch (-{2}): {3}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emet-Selch of the Third Seat");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let fb = t.graveyard(P0, "Firebolt");
    t.lands(P0, "Mountain", 3);
    assert!(!can_cast(&mut t, P0, fb, FLASHBACK));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, fb, FLASHBACK));
    let spell = t.cast(P0, fb).method(FLASHBACK).target(Entity::Player(P1)).go();
    assert_eq!(t.g.mana_value_of(spell), 1);
}

#[test]
fn emet_selch_cast_during_resolution_ignoring_timing() {
    cr!("608.2g", "601.2", "117.1a");
    ruling!(
        "Emet-Selch of the Third Seat",
        "You choose whether to cast the target card while Emet-Selch of the Third Seat's second ability is still on the stack. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    for cast in [true, false] {
        // P1's turn: P1 loses life; Divination (a sorcery) in P0's graveyard.
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.battlefield(P0, "Emet-Selch of the Third Seat");
        t.lands(P0, "Island", 1);
        let div = t.graveyard(P0, "Divination");
        t.answer_targets(P0, &[obj(div)]);
        t.answer_yes(P0, cast);
        t.g.lose_life(P1, 1);
        t.g.flush_events();
        t.settle();
        assert_eq!(triggers_on_stack(&t, "cast target instant or sorcery"), 1);
        let hand = t.hand_size(P0);
        t.resolve_all();
        if cast {
            assert_eq!(t.hand_size(P0), hand + 2);
            // "If that spell would be put into your graveyard, exile it instead."
            assert!(t.in_exile("Divination"));
        } else {
            assert_eq!(t.hand_size(P0), hand);
            // It can't be cast later.
            t.set_step(P0, Step::PrecombatMain);
            assert!(!can_cast(&mut t, P0, div, CastMethod::Normal));
        }
    }
}

/// Licia in P0's hand with Mountain, Plains and Swamp after P0 gains `gain` life.
fn licia_castable(gain: u32, land_count: usize) -> bool {
    supported("Licia, Sanguine Tribune");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 30;
    t.g.gain_life(P0, gain);
    t.g.lose_life(P0, 10);
    t.g.flush_events();
    for land in ["Mountain", "Plains", "Swamp"].iter().take(land_count) {
        t.lands(P0, land, 1);
    }
    let licia = t.hand(P0, "Licia, Sanguine Tribune");
    can_cast(&mut t, P0, licia, CastMethod::Normal)
}

#[test]
fn licia_counts_life_gained_and_reduces_only_generic_mana() {
    cr!("601.2f", "118.7");
    ruling!(
        "Licia, Sanguine Tribune",
        "Licia's first ability checks how much life you've gained during the turn, not what your life total is compared to what it was when the turn began. For example, if you start the turn at 30 life, gain 6 life during the turn, then lose 10 life later that turn, Licia still costs {6} less to cast."
    );
    ruling!(
        "Licia, Sanguine Tribune",
        "Licia's first ability reduces only generic mana in its cost. It can't reduce the cost below {R}{W}{B}."
    );
    // At 26 after gaining 6 and losing 10: {R}{W}{B}.
    assert!(licia_castable(6, 3));
    assert!(!licia_castable(4, 3));
    // Gaining more than the generic part can't cut into {R}{W}{B}.
    assert!(!licia_castable(10, 2));
}

#[test]
fn licia_ability_countered_still_counts_for_the_turn() {
    cr!("602.5b", "701.6b");
    ruling!(
        "Licia, Sanguine Tribune",
        "If Licia's last ability doesn't resolve once activated, it can't be activated again in the same turn."
    );
    supported("Stifle");
    let mut t = TestGame::new(2);
    let licia = t.battlefield(P0, "Licia, Sanguine Tribune");
    assert!(can_activate(&mut t, P0, licia));
    t.activate(P0, licia, 0, &[]).unwrap();
    let ability = t.g.stack[0];
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.g.turn.priority = Some(P1);
    t.cast(P1, stifle).target(obj(ability)).go();
    t.resolve_all();
    assert_eq!(t.counters(licia, "+1/+1"), 0);
    assert_eq!(t.life(P0), 15);
    assert!(!can_activate(&mut t, P0, licia));
}

/// P0 has Marshland Bloodcaster with two Swamps for its ability.
fn bloodcaster(t: &mut TestGame) -> ObjectId {
    supported("Marshland Bloodcaster");
    let mb = t.battlefield(P0, "Marshland Bloodcaster");
    t.lands(P0, "Swamp", 2);
    mb
}

#[test]
fn marshland_bloodcaster_affects_spells_after_it_resolves() {
    cr!("611.2f", "118.9");
    ruling!(
        "Marshland Bloodcaster",
        "Marshland Bloodcaster's ability affects the next spell you cast after the ability has resolved. If you cast any spells in response to the ability, those spells are cast as normal."
    );
    ruling!(
        "Marshland Bloodcaster",
        "Even after the ability resolves, you may choose to not pay life rather than pay mana cost of the next spell you cast that turn. However, if you do this, you won't have the opportunity to pay life for any other spells you cast that turn (unless you activate the ability again)."
    );
    let mut t = TestGame::new(2);
    let mb = bloodcaster(&mut t);
    t.lands(P0, "Mountain", 1);
    t.activate(P0, mb, 0, &[]).unwrap();
    // In response: Lightning Bolt can't be cast for life; it's cast for its mana.
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!can_cast(&mut t, P0, bolt, OFFERED));
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Now the next spell may be cast for life ...
    let giant = t.hand(P0, "Hill Giant");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, giant, OFFERED));
    // ... but P0 casts Memnite normally instead: the chance is gone.
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.resolve_all();
    assert!(!can_cast(&mut t, P0, giant, OFFERED));
    assert!(!can_cast(&mut t, P0, bears, OFFERED));
}

#[test]
fn marshland_bloodcaster_x_is_0() {
    cr!("107.3", "118.9", "601.2b");
    ruling!(
        "Marshland Bloodcaster",
        "If the next spell you cast has {X} in its mana cost, you must choose 0 as the value of X if you pay life rather than pay its mana cost."
    );
    supported("Endless One");
    let mut t = TestGame::new(2);
    let mb = bloodcaster(&mut t);
    t.activate(P0, mb, 0, &[]).unwrap();
    t.resolve_all();
    let one = t.hand(P0, "Endless One");
    // X = 3 is refused or ignored; X is 0 either way.
    let spell = match t.cast(P0, one).method(OFFERED).x(3).try_go() {
        Ok(s) => s,
        Err(_) => t.cast(P0, one).method(OFFERED).x(0).go(),
    };
    assert_eq!(t.obj_now(spell).stack.as_ref().unwrap().x.unwrap_or(0), 0);
    // Mana value 0: no life paid.
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    // A 0/0 with no counters.
    assert!(t.in_graveyard(P0, "Endless One"));
}

#[test]
fn marshland_bloodcaster_mandatory_additional_costs_are_still_paid() {
    cr!("118.8", "118.9", "601.2f");
    ruling!(
        "Marshland Bloodcaster",
        "If you cast a spell for another cost \"rather than pay its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Draconic Intervention, those must be paid to cast the card."
    );
    supported("Bone Splinters");
    // Bone Splinters ({B}, "As an additional cost to cast this spell, sacrifice a
    // creature."): with no creature to sacrifice, it can't be cast for life either.
    let mut t = TestGame::new(2);
    let mb = bloodcaster(&mut t);
    t.activate(P0, mb, 0, &[]).unwrap();
    t.resolve_all();
    let target = t.battlefield(P1, "Grizzly Bears");
    let bs = t.hand(P0, "Bone Splinters");
    // Marshland Bloodcaster itself is a creature: move it out of the way.
    crate::r_s05_common::move_to(&mut t, mb, Zone::Hand(P0));
    assert!(!can_cast(&mut t, P0, bs, OFFERED));
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(can_cast(&mut t, P0, bs, OFFERED));
    t.answer_choose(P0, &[obj(elves)]);
    t.cast(P0, bs).method(OFFERED).target(obj(target)).go();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.life(P0), 19);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn demonic_embrace_graveyard_casting_keeps_its_timing() {
    cr!("601.3", "307.1", "117.1a");
    ruling!(
        "Demonic Embrace",
        "Casting Demonic Embrace with the permission granted by its last ability doesn't change when you can cast it."
    );
    supported("Demonic Embrace");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Memnite");
    t.lands(P0, "Swamp", 3);
    let e = t.graveyard(P0, "Demonic Embrace");
    let methods = crate::r_s08_common::legal_cast_methods(&mut t, P0, e);
    assert_eq!(methods.len(), 1);
    // On P1's turn it can't be cast from there.
    t.set_step(P1, Step::PrecombatMain);
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, e).is_empty());
    // Nor with something on the stack.
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, e).is_empty());
    t.resolve_all();
    assert_eq!(crate::r_s08_common::legal_cast_methods(&mut t, P0, e), methods);
}

#[test]
fn demonic_embrace_other_permission_has_no_additional_cost() {
    cr!("601.2f", "118.8");
    ruling!(
        "Demonic Embrace",
        "If another effect gives you permission to cast Demonic Embrace from your graveyard, you don't pay the additional cost imposed by its last ability if you use that permission."
    );
    supported("Yawgmoth's Will");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Memnite");
    t.lands(P0, "Swamp", 6);
    let e = t.graveyard(P0, "Demonic Embrace");
    let yw = t.hand(P0, "Yawgmoth's Will");
    t.cast(P0, yw).go();
    t.resolve_all();
    assert!(can_cast(&mut t, P0, e, CastMethod::Normal));
    t.cast(P0, e).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_hand(P0, "Memnite"));
    assert_eq!(t.pt(bears), (5, 3));
}

#[test]
fn mystic_forge_top_card_rulings() {
    cr!("305.9", "702.37c", "401.5", "708.4");
    ruling!(
        "Mystic Forge",
        "Because you never \"cast\" a land card, Mystic Forge doesn't allow you to play an artifact land from the top of your library."
    );
    ruling!(
        "Mystic Forge",
        "If the top card of your library has a morph ability, you can cast it face down from the top of your library, even if it's normally not a colorless card."
    );
    ruling!(
        "Mystic Forge",
        "Mystic Forge lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    supported("Seat of the Synod");
    supported("Akroma, Angel of Fury");
    let mut t = TestGame::new(2);
    let seat = t.library_top(P0, "Seat of the Synod");
    assert!(!can_look_at(&t.g, P0, seat));
    t.battlefield(P0, "Mystic Forge");
    assert!(can_look_at(&t.g, P0, seat));
    assert!(!can_look_at(&t.g, P1, seat));
    // Even when it isn't P0's priority.
    t.g.turn.priority = Some(P1);
    assert!(can_look_at(&t.g, P0, seat));
    assert!(!can_play_land(&mut t, P0, seat));
    // A red card with morph: not normally, but face down.
    let akroma = t.library_top(P0, "Akroma, Angel of Fury");
    t.lands(P0, "Mountain", 3);
    assert!(!can_cast(&mut t, P0, akroma, CastMethod::Normal));
    let fd = CastMethod::FaceDown(KeywordKind::Morph);
    assert!(can_cast(&mut t, P0, akroma, fd.clone()));
    let spell = t.cast(P0, akroma).method(fd).go();
    assert!(t.obj_now(spell).face_down);
}

#[test]
fn cramped_vents_with_deathtouch_deals_more_excess_damage() {
    cr!("120.10", "702.2c", "709.5");
    ruling!(
        "Cramped Vents // Access Maze",
        "In some unusual cases, Cramped Vents may have deathtouch. Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage, so any amount greater than that will cause excess damage to be dealt, even if the total amount of damage isn't greater than the creature's toughness."
    );
    supported("Cramped Vents // Access Maze");
    for deathtouch in [false, true] {
        let mut t = TestGame::new(2);
        if deathtouch {
            let def = custom_card(
                "Deathly Aura",
                "Enchantment",
                "{B}",
                None,
                "Enchantments you control have deathtouch.",
            );
            t.custom(P0, def, Zone::Battlefield);
        }
        let giant = t.battlefield(P1, "Hill Giant");
        let room = t.battlefield(P0, "Cramped Vents // Access Maze");
        t.answer_targets(P0, &[obj(giant)]);
        rooms::unlock(&mut t.g, room, 0, P0);
        t.g.flush_events();
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Hill Giant"));
        // 6 damage to a 3/3: 3 excess; with deathtouch 1 is lethal, so 5.
        assert_eq!(t.life(P0), if deathtouch { 25 } else { 23 });
    }
}
