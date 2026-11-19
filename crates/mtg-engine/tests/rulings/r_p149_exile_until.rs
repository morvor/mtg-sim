//! Rulings batch P149 — exile effects: "exile ... until this leaves the battlefield"
//! (CR 610.3), Abuelo's exile-and-return, and permanents returned as 1/1 creatures
//! (Abuelo's Awakening, Excava).

use crate::r_p149_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The four "exile target ... an opponent controls until this leaves the battlefield"
/// permanents of the batch.
const EXILE_UNTIL: [&str; 4] = [
    "Dusk Rose Reliquary",
    "Hardlight Containment",
    "Seal from Existence",
    "Trapped in the Screen",
];

/// P0's `name` enters targeting P1's Grizzly Bears; its trigger is on the stack. (The
/// Aura, Hardlight Containment, enters attached to P0's Ornithopter.)
fn exile_until_setup(t: &mut TestGame, name: &str) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    if name == "Hardlight Containment" {
        let thopter = t.battlefield(P0, "Ornithopter");
        t.answer_choose(P0, &[obj(thopter)]);
    }
    let src = enter_targeting(t, P0, name, &[obj(bears)]);
    assert_eq!(t.stack_len(), 1, "{name}");
    (src, bears)
}

#[test]
fn exile_until_source_leaves_before_its_trigger_resolves_nothing_is_exiled() {
    cr!("610.3b");
    ruling!(
        "Dusk Rose Reliquary",
        "If Dusk Rose Reliquary leaves the battlefield before its last ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Hardlight Containment",
        "If Hardlight Containment leaves the battlefield before its triggered ability resolves, the target creature won’t be exiled."
    );
    ruling!(
        "Seal from Existence",
        "If Seal from Existence leaves the battlefield before its last ability resolves, the target permanent won’t be exiled."
    );
    ruling!(
        "Trapped in the Screen",
        "If Trapped in the Screen leaves the battlefield before its last ability resolves, the target permanent won't be exiled at all."
    );
    for name in EXILE_UNTIL {
        // It stays: the trigger resolves and exiles the Bears until it leaves.
        let mut t = TestGame::new(2);
        let (src, bears) = exile_until_setup(&mut t, name);
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Exile, "{name}");
        kill(&mut t, src);
        assert!(t.on_battlefield(bears), "{name}");
        // It leaves first: the Bears are never exiled.
        let mut t = TestGame::new(2);
        let (src, bears) = exile_until_setup(&mut t, name);
        kill(&mut t, src);
        t.resolve_all();
        assert!(t.on_battlefield(bears), "{name}");
        assert!(t.g.exile.is_empty(), "{name}");
    }
}

#[test]
fn seal_from_existence_returns_the_card_at_once_without_using_the_stack() {
    cr!("610.3", "610.3c");
    ruling!(
        "Seal from Existence",
        "Seal from Existence’s last ability creates two one-shot effects: one that exiles the permanent when the ability resolves, and another that returns the exiled card to the battlefield immediately after Seal from Existence leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let (seal, bears) = exile_until_setup(&mut t, "Seal from Existence");
    // The first one-shot effect: exiled as the ability resolves.
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.stack_len(), 0);
    // The second: back as soon as the Seal leaves, with nothing put on the stack, under
    // its owner's control.
    let id = t.g.current(seal);
    t.g.destroy(id, None);
    t.g.flush_events();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn seal_from_existence_exile_drops_auras_unattaches_equipment_and_loses_counters() {
    cr!("303.4c", "301.5c", "122.2", "704.5m", "704.5n");
    ruling!(
        "Seal from Existence",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Equipment attached to an exiled creature will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(&mut t, P1, "Rancor", obj(bears));
    let gear = attach_new(&mut t, P1, "Bonesplitter", obj(bears));
    put(&mut t, bears, counters::PLUS1, 2);
    enter_targeting(&mut t, P0, "Seal from Existence", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // Rancor's own "put into a graveyard" trigger returns it to its owner's hand.
    assert!(matches!(t.zone(aura), Zone::Graveyard(_) | Zone::Hand(_)));
    assert_ne!(t.zone(aura), Zone::Battlefield);
    assert!(t.on_battlefield(gear));
    assert_eq!(attached_to(&t, gear), None);
    // Back with no counters.
    let seal = t.named_on_battlefield("Seal from Existence")[0];
    kill(&mut t, seal);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn seal_from_existence_returns_the_card_when_its_owner_leaves_the_game() {
    cr!("800.4a", "610.3");
    ruling!(
        "Seal from Existence",
        "In a multiplayer game, if Seal from Existence’s owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn’t an ability that goes on the stack, it won’t cease to exist along with the leaving player’s spells and abilities on the stack."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    enter_targeting(&mut t, P0, "Seal from Existence", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // P0 leaves with a spell of theirs on the stack.
    t.hand(P0, "Divination");
    lots(&mut t, P0);
    let div = t.g.find_in_zone(Zone::Hand(P0), "Divination")[0];
    t.cast(P0, div).go();
    assert_eq!(t.stack_len(), 1);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.has_lost(P0));
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

/// P0's Abuelo, Ancestral Echo ("{1}{W}{U}: Exile another target creature or artifact
/// you control. Return it to the battlefield under its owner's control at the beginning
/// of the next end step.") exiles `target`.
fn abuelo_exiles(t: &mut TestGame, abuelo: ObjectId, target: ObjectId) {
    supported("Abuelo, Ancestral Echo");
    act(t, P0, abuelo, 0, &[obj(target)]);
}

#[test]
fn abuelo_returns_the_card_at_the_next_end_step_even_without_abuelo() {
    cr!("603.7a", "603.7c", "610.3c");
    ruling!(
        "Abuelo, Ancestral Echo",
        "The exiled card will return to the battlefield at the beginning of the next end step even if Abuelo, Ancestral Echo is no longer on the battlefield at that time."
    );
    let mut t = TestGame::new(2);
    let abuelo = t.battlefield(P0, "Abuelo, Ancestral Echo");
    let bears = t.battlefield(P0, "Grizzly Bears");
    abuelo_exiles(&mut t, abuelo, bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    kill(&mut t, abuelo);
    assert!(!t.on_battlefield(abuelo));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn abuelo_exiling_a_token_it_ceases_to_exist() {
    cr!("111.8", "704.5d");
    ruling!(
        "Abuelo, Ancestral Echo",
        "If a token is exiled this way, it ceases to exist and won't return to the battlefield."
    );
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    let abuelo = t.battlefield(P0, "Abuelo, Ancestral Echo");
    cast_at(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let soldiers = tokens(&t, P0);
    assert_eq!(soldiers.len(), 2);
    abuelo_exiles(&mut t, abuelo, soldiers[0]);
    assert!(!t.g.exile.iter().any(|id| t.g.obj(*id).is_token()));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

/// P0 casts Abuelo's Awakening ("Return target artifact or non-Aura enchantment card from
/// your graveyard to the battlefield with X additional +1/+1 counters on it. It's a 1/1
/// Spirit creature with flying in addition to its other types.") with X = 0 on the card
/// `name` in their graveyard; the permanent it returns.
fn awaken(t: &mut TestGame, name: &str) -> ObjectId {
    supported("Abuelo's Awakening");
    let card = t.graveyard(P0, name);
    lots(t, P0);
    let spell = t.hand(P0, "Abuelo's Awakening");
    t.cast(P0, spell).x(0).target(obj(card)).go();
    t.resolve_all();
    let id = t.g.current(card);
    assert!(t.on_battlefield(id), "{name}");
    id
}

#[test]
fn returned_artifact_and_enchantment_creature_cards_are_base_1_1() {
    cr!("613.4b", "613.1d");
    // Misattached: this is Abuelo's Awakening's ruling (filed under Abuelo, Ancestral
    // Echo, whose own ability sets no power or toughness; docs/RULING_DISCREPANCIES.md).
    ruling!(
        "Abuelo, Ancestral Echo",
        "An artifact creature card or enchantment creature card returned this way will have base power and toughness 1/1."
    );
    // Juggernaut (a 5/3 artifact creature) and Boon Satyr (a 4/2 enchantment creature).
    for name in ["Juggernaut", "Boon Satyr"] {
        let mut t = TestGame::new(2);
        let id = awaken(&mut t, name);
        assert_eq!(t.pt(id), (1, 1), "{name}");
        assert!(t.obj_now(id).chars.has_subtype("Spirit"), "{name}");
    }
}

#[test]
fn excava_returns_artifact_and_enchantment_creature_cards_as_base_1_1() {
    cr!("613.4b", "508.1m");
    ruling!(
        "Excava, the Risen Past",
        "An artifact creature card or enchantment creature card returned this way will have base power and toughness 1/1."
    );
    supported("Excava, the Risen Past");
    // Bronze Sable (a 2/1 artifact creature) and Boon Satyr (a 4/2 enchantment creature),
    // each mana value 3 or less.
    for name in ["Bronze Sable", "Boon Satyr"] {
        let mut t = TestGame::new(2);
        let excava = t.battlefield(P0, "Excava, the Risen Past");
        let card = t.graveyard(P0, name);
        t.answer_targets(P0, &[obj(card)]);
        attack_with(&mut t, &[(excava, pl(P1))]);
        t.resolve_all();
        let id = t.g.current(card);
        assert!(t.on_battlefield(id), "{name}");
        assert_eq!(t.pt(id), (1, 1), "{name}");
        assert_eq!(t.counters(id, "finality"), 1, "{name}");
    }
}

#[test]
fn returned_permanents_take_later_base_pt_but_stay_1_1_when_crewed() {
    cr!("613.4b", "613.7a", "702.122a");
    // Misattached: this is Abuelo's Awakening's ruling (filed under Abuelo, Ancestral
    // Echo, whose own ability makes nothing a creature; docs/RULING_DISCREPANCIES.md).
    ruling!(
        "Abuelo, Ancestral Echo",
        "If another effect causes one of the returned permanents to become a creature and sets its base power and toughness as it does so, that creature will have that power and toughness; it won't be 1/1. Notably, crewing a Vehicle does not set its power and toughness, so a Vehicle will remain a 1/1 if you crew it."
    );
    supported("Ensoul Artifact");
    // Ensoul Artifact ("Enchanted artifact is a creature with base power and toughness 5/5
    // in addition to its other types.") on a returned Bonesplitter: 5/5.
    let mut t = TestGame::new(2);
    let splitter = awaken(&mut t, "Bonesplitter");
    assert_eq!(t.pt(splitter), (1, 1));
    cast_at(&mut t, P0, "Ensoul Artifact", &[obj(splitter)]);
    t.resolve_all();
    assert_eq!(t.pt(splitter), (5, 5));
    // A returned Smuggler's Copter (a 3/3 Vehicle) crewed: still 1/1.
    let mut t = TestGame::new(2);
    let copter = awaken(&mut t, "Smuggler's Copter");
    assert_eq!(t.pt(copter), (1, 1));
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(t.obj_now(copter).is(CardType::Creature));
    assert_eq!(t.pt(copter), (1, 1));
}
