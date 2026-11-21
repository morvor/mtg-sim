//! Rulings batch P005 — activated abilities: a cost without {T} (CR 602.1), several
//! instances of a {T} ability (CR 113.2c), abilities that resolve independently of their
//! source (CR 113.7a), several equip abilities (CR 702.6d), "doesn't untap during its next
//! untap step" (CR 502.3), searching by basic land type (CR 205.3i), casting permissions
//! that keep their timing (CR 302.1, 601.3) and don't apply to lands (CR 305.9), and
//! flashback costs equal to a mana cost (CR 702.34a, 118.6, 107.3b).

use crate::r_p006_common::{choices_since, in_graveyard, mark, obj, pl};
use crate::r_p044_common::kill;
use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s02_common::{can_cast, can_play_land};
use crate::r_s21_common::castable;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the activated ability of `source` whose text contains `needle`.
fn activate_containing(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
    nth: usize,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let source = t.g.current(source);
    let uid =
        t.g.obj(source)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
            .nth(nth)
            .map(|a| a.uid)
            .unwrap_or_else(|| panic!("no activated ability containing {needle:?}"));
    t.g.turn.priority = Some(p);
    let r = t.g.activate_ability(p, source, uid);
    t.g.flush_events();
    r
}

#[test]
fn guardian_of_new_benalia_activates_while_tapped_and_repeatedly() {
    cr!("602.1", "602.2");
    ruling!(
        "Guardian of New Benalia",
        "You may activate Guardian of New Benalia's last ability even if it's already tapped. You may also activate that ability multiple times in the same turn, even if Guardian of New Benalia already has indestructible."
    );
    supported("Guardian of New Benalia");
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P0, "Guardian of New Benalia");
    t.g.tap(guardian);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(a)]);
    t.activate(P0, guardian, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(guardian).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj_now(guardian).tapped);
    // Again, already indestructible (and tapped).
    t.answer_choose(P0, &[obj(b)]);
    t.activate(P0, guardian, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn lorehold_apprentice_grants_instances_each_needing_the_tap() {
    cr!("113.2c", "602.2b");
    ruling!(
        "Lorehold Apprentice",
        "Lorehold Apprentice’s ability can give your Spirit creatures multiple instances of “{T}: This creature deals 1 damage to each opponent.” However, each Spirit can only activate one of these abilities at a time, and each ability will still only deal 1 damage."
    );
    supported("Lorehold Apprentice");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lorehold Apprentice");
    // Spectral Sailor is a Spirit.
    let spirit = t.battlefield(P0, "Spectral Sailor");
    t.lands(P0, "Island", 2);
    for _ in 0..2 {
        let opt = t.hand(P0, "Opt");
        t.cast(P0, opt).go();
        t.resolve_all();
    }
    let granted = |t: &TestGame| {
        t.obj_now(spirit)
            .chars
            .abilities
            .iter()
            .filter(|a| a.text.contains("1 damage to each opponent"))
            .count()
    };
    assert_eq!(granted(&t), 2);
    activate_containing(&mut t, P0, spirit, "1 damage to each opponent", 0).unwrap();
    // Tapped: the other instance can't be activated now.
    assert!(activate_containing(&mut t, P0, spirit, "1 damage to each opponent", 1).is_err());
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

/// P0 activates the equip ability of `equipment` whose cost is {`n`}.
fn equip_for(
    t: &mut TestGame,
    equipment: ObjectId,
    n: u32,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let uid =
        t.g.obj(equipment)
            .chars
            .abilities
            .iter()
            .find(|a| match &a.kind {
                AbilityKind::Activated(act) => {
                    a.text == "Equip" && act.cost.mana.as_ref().map_or(0, |m| m.mana_value()) == n
                }
                _ => false,
            })
            .map(|a| a.uid)
            .expect("no such equip ability");
    t.g.turn.priority = Some(P0);
    let r = t.g.activate_ability(P0, equipment, uid);
    t.g.flush_events();
    r
}

#[test]
fn puresteel_paladins_equip_resolves_without_it() {
    cr!("113.7a", "702.6a");
    ruling!(
        "Puresteel Paladin",
        "Once the equip {0} ability is activated, causing Puresteel Paladin to leave the battlefield or causing its controller to control fewer than three artifacts won't stop the equip ability from resolving."
    );
    supported("Puresteel Paladin");
    for leave in ["the Paladin", "an artifact"] {
        let mut t = TestGame::new(2);
        let paladin = t.battlefield(P0, "Puresteel Paladin");
        let splitter = t.battlefield(P0, "Bonesplitter");
        let orni = t.battlefield(P0, "Ornithopter");
        t.battlefield(P0, "Ornithopter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        // No mana: the equip {0} ability.
        t.answer_targets(P0, &[obj(bears)]);
        equip_for(&mut t, splitter, 0).unwrap();
        assert_eq!(t.stack_len(), 1);
        kill(
            &mut t,
            if leave == "the Paladin" {
                paladin
            } else {
                orni
            },
        );
        t.resolve_all();
        assert_eq!(t.obj_now(splitter).attached_to, Some(obj(bears)), "{leave}");
        assert_eq!(t.pt(bears), (4, 2));
    }
}

#[test]
fn puresteel_paladin_leaves_the_other_equip_abilities() {
    cr!("702.6a", "702.6d");
    ruling!(
        "Puresteel Paladin",
        "You may still activate the Equipment's other equip abilities if you wish."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puresteel Paladin");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    // Bonesplitter's own equip {1}.
    t.answer_targets(P0, &[obj(bears)]);
    equip_for(&mut t, splitter, 1).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(splitter).attached_to, Some(obj(bears)));
    assert!(t
        .g
        .permanents()
        .any(|o| o.tapped && o.chars.name == "Wastes"));
}

#[test]
fn house_guildmage_doesnt_tap_and_is_used_up_by_the_next_untap_step() {
    cr!("502.3", "608.2c");
    ruling!(
        "House Guildmage",
        "House Guildmage's first ability doesn't tap the creature. It can target any creature, tapped or untapped. If that creature is already untapped at the beginning of its controller's next untap step, the effect won't do anything."
    );
    supported("House Guildmage");
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        let mage = t.battlefield(P0, "House Guildmage");
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 1);
        let bears = t.battlefield(P1, "Grizzly Bears");
        if tapped {
            t.g.tap(bears);
        }
        t.activate(P0, mage, 0, &[obj(bears)]).unwrap();
        t.resolve_all();
        assert_eq!(t.obj_now(bears).tapped, tapped, "not tapped by it");
        t.advance_to(P1, Step::Upkeep);
        // A tapped creature stays tapped; an untapped one was unaffected.
        assert_eq!(t.obj_now(bears).tapped, tapped);
        // Either way the effect is over: tapped later, it untaps in the next untap step.
        t.g.tap(bears);
        t.advance_to(P0, Step::Upkeep);
        t.advance_to(P1, Step::Upkeep);
        assert!(!t.obj_now(bears).tapped, "{tapped}");
    }
}

#[test]
fn sprouting_goblin_finds_any_land_with_a_basic_land_type() {
    cr!("205.3i", "701.23a");
    ruling!(
        "Sprouting Goblin",
        "You may search for any land card with one of the basic land types, even if that land isn't a basic land. The basic land types are Plains, Island, Swamp, Mountain, and Forest."
    );
    supported("Sprouting Goblin");
    let mut t = TestGame::new(2);
    let pool = t.library_top(P0, "Breeding Pool");
    let wastes = t.library_top(P0, "Wastes");
    let island = t.library_top(P0, "Island");
    let bears = t.library_top(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Sprouting Goblin");
    t.lands(P0, "Forest", 1);
    let goblin = t.hand(P0, "Sprouting Goblin");
    let from = mark(&t);
    t.answer_choose(P0, &[obj(pool)]);
    t.cast(P0, goblin).kicked(true).go();
    t.resolve_all();
    let choices = choices_since(&t, P0, from);
    let candidates = &choices.last().expect("searched").0;
    assert!(candidates.contains(&obj(pool)) && candidates.contains(&obj(island)));
    assert!(
        !candidates.contains(&obj(wastes)),
        "Wastes has no basic land type"
    );
    assert!(!candidates.contains(&obj(bears)));
    assert!(t.in_hand(P0, "Breeding Pool"));
}

/// P0 activates Osteomancer Adept (which resolves); P0's graveyard has `card` and three
/// other cards to forage with. Returns the card.
fn adept_permission(t: &mut TestGame, card: &str) -> ObjectId {
    supported("Osteomancer Adept");
    let adept = t.battlefield(P0, "Osteomancer Adept");
    let c = t.graveyard(P0, card);
    for _ in 0..3 {
        t.graveyard(P0, "Forest");
    }
    activate_containing(t, P0, adept, "Until end of turn", 0).unwrap();
    t.resolve_all();
    c
}

#[test]
fn osteomancer_adept_doesnt_let_dryad_arbor_be_played() {
    cr!("305.9", "601.3");
    ruling!(
        "Osteomancer Adept",
        "If Dryad Arbor (the only card that's both a creature and a land) is in your graveyard, you can't play it this way. Dryad Arbor can't be cast as a spell."
    );
    let mut t = TestGame::new(2);
    let arbor = adept_permission(&mut t, "Dryad Arbor");
    assert!(!castable(&mut t, P0, arbor));
    assert!(!can_play_land(&mut t, P0, arbor));
    assert!(t.play_land(P0, arbor).is_err());
    // A creature card there can be cast this way.
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(castable(&mut t, P0, bears));
}

#[test]
fn osteomancer_adept_keeps_creature_timing() {
    cr!("302.1", "601.3");
    ruling!(
        "Osteomancer Adept",
        "You must still follow timing restrictions and permissions for creature spells you cast with the permission granted by Osteomancer Adept's last ability. Normally, you'll be able to cast them only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let bears = adept_permission(&mut t, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    assert!(castable(&mut t, P0, bears));
    // Not with something on the stack.
    let growth = t.hand(P0, "Giant Growth");
    let target = obj(t.battlefield(P1, "Grizzly Bears"));
    let spell = t.cast(P0, growth).target(target).go();
    assert!(t.g.stack.contains(&spell));
    assert!(!castable(&mut t, P0, bears));
    t.resolve_all();
    // Not in the combat phase.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, bears));
    // A creature card with flash may be cast at instant speed: Spectral Sailor.
    let sailor = t.graveyard(P0, "Spectral Sailor");
    t.lands(P0, "Island", 1);
    assert!(castable(&mut t, P0, sailor));
    t.set_step(P0, Step::PostcombatMain);
    assert!(castable(&mut t, P0, bears));
}

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

/// Archmage's Newt (saddled or not) deals combat damage to P1, giving `card` in P0's
/// graveyard flashback; the game moves on to P0's postcombat main phase.
fn newt_hits(t: &mut TestGame, card: ObjectId, saddled: bool) {
    supported("Archmage's Newt");
    let newt = t.battlefield(P0, "Archmage's Newt");
    if saddled {
        let giant = t.battlefield(P0, "Hill Giant");
        t.answer_choose(P0, &[obj(giant)]);
        activate_containing(t, P0, newt, "Saddle", 0).unwrap();
        t.resolve_all();
    }
    t.answer_targets(P0, &[obj(card)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(newt, pl(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.obj_now(card).has_keyword(KeywordKind::Flashback));
}

#[test]
fn archmages_newt_gives_a_card_with_no_mana_cost_no_flashback_cost() {
    cr!("702.34a", "118.6");
    ruling!(
        "Archmage's Newt",
        "If a card with no mana cost gains flashback equal to its mana cost, it has no flashback cost. It can't be cast this way."
    );
    // Ancestral Vision has no mana cost.
    supported("Ancestral Vision");
    let mut t = TestGame::new(2);
    let vision = t.graveyard(P0, "Ancestral Vision");
    newt_hits(&mut t, vision, false);
    t.lands(P0, "Island", 5);
    let vision = t.g.current(vision);
    assert!(!can_cast(&mut t, P0, vision, FLASHBACK));
    assert!(t.cast(P0, vision).method(FLASHBACK).try_go().is_err());
    assert!(in_graveyard(&t, vision));
}

#[test]
fn archmages_newt_flashback_x_chooses_x_unless_its_flashback_0() {
    cr!("702.34a", "107.3", "107.3b");
    ruling!(
        "Archmage's Newt",
        "If you cast an instant or sorcery with {X} in its mana cost and a flashback cost equal to its mana cost, you still choose the value of X as part of casting the spell and paying that cost. If that spell has flashback {0}, you won't be able to choose the value of X; the value of X will simply be 0."
    );
    supported("Blaze");
    // Blaze ({X}{R}): "Blaze deals X damage to any target."
    for saddled in [false, true] {
        let mut t = TestGame::new(2);
        let blaze = t.graveyard(P0, "Blaze");
        newt_hits(&mut t, blaze, saddled);
        let life = t.life(P1);
        t.lands(P0, "Mountain", 4);
        let blaze = t.g.current(blaze);
        t.cast(P0, blaze).method(FLASHBACK).x(3).target(pl(P1)).go();
        t.resolve_all();
        let untapped =
            t.g.permanents()
                .filter(|o| o.chars.name == "Mountain" && !o.tapped)
                .count();
        if saddled {
            assert_eq!(t.life(P1), life, "X is 0");
            assert_eq!(untapped, 4);
        } else {
            assert_eq!(t.life(P1), life - 3);
            assert_eq!(untapped, 0);
        }
        assert!(t.in_exile("Blaze"));
    }
}
