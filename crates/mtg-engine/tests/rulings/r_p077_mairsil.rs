//! Rulings batch P077 — Mairsil, the Pretender: "Mairsil has all activated abilities of
//! all cards you own in exile with cage counters on them. You may activate each of those
//! abilities only once each turn." (CR 113.6, 602.5b, 607.2, 201.5)

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use mtg_engine::ability::AbilityKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts the real card `name` into exile, owned by P0, with a cage counter on it.
fn cage(t: &mut TestGame, name: &str) -> ObjectId {
    let id = t.exile(P0, name);
    t.g.add_counters(Entity::Object(id), "cage", 1, None);
    t.g.recompute();
    id
}

/// The texts of Mairsil's abilities of the given kind.
fn texts(t: &mut TestGame, id: ObjectId, activated: bool) -> Vec<String> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) == activated)
        .map(|a| a.text.to_string())
        .collect()
}

#[test]
fn mairsil_gains_activated_abilities_not_triggered_ones() {
    cr!("602.1", "603.1", "702.6a");
    ruling!(
        "Mairsil, the Pretender",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder text. Mairsil won't gain triggered abilities (which start with \"when,\" \"whenever,\" or \"at\")."
    );
    supported("Mairsil, the Pretender");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    let own = texts(&mut t, m, false).len();
    // Smuggler's Copter: "Whenever this Vehicle attacks or blocks, you may draw a card. If
    // you do, discard a card." and crew 1. Bonesplitter: equip {1}.
    cage(&mut t, "Smuggler's Copter");
    cage(&mut t, "Bonesplitter");
    let act = texts(&mut t, m, true);
    assert!(act.iter().any(|a| a.starts_with("Crew")), "{act:?}");
    assert!(act.iter().any(|a| a.starts_with("Equip")), "{act:?}");
    assert_eq!(
        texts(&mut t, m, false).len(),
        own,
        "no triggered ability gained"
    );
    let hand = t.hand_size(P0);
    t.attack(&[(m, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "no loot");
}

#[test]
fn mairsil_crewed_becomes_an_artifact_creature_and_equip_does_nothing() {
    cr!("702.122a", "702.6a", "301.5c");
    ruling!(
        "Mairsil, the Pretender",
        "If Mairsil has a crew ability, creatures can crew Mairsil. It'll become an artifact creature, but its power and toughness remain unchanged."
    );
    ruling!(
        "Mairsil, the Pretender",
        "If Mairsil has an equip ability, activating it won't cause anything to happen. Mairsil doesn't become attached to a creature."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    cage(&mut t, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, m, &[bears]));
    t.resolve_all();
    let o = t.obj_now(m);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert_eq!(t.pt(m), (4, 4));

    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    cage(&mut t, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, m, "Equip").expect("equip");
    t.resolve_all();
    assert_eq!(crate::r_s06_common::attached_to(&t, m), None);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(t.on_battlefield(m));
}

#[test]
fn mairsil_linked_abilities() {
    cr!("607.2a", "607.5", "113.6");
    ruling!(
        "Mairsil, the Pretender",
        "If Mairsil gains an activated ability that's normally linked to a non-activated ability of the card it came from, the ability Mairsil has isn't linked to any ability. For example, if Mairsil exiles Prototype Portal, the activated ability creates no tokens at all; it doesn't create Prototype Portal tokens."
    );
    // Prototype Portal: "Imprint — When this artifact enters, you may exile an artifact
    // card from your hand. {X}, {T}: Create a token that's a copy of the exiled card. X is
    // the mana value of the exiled card."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    cage(&mut t, "Prototype Portal");
    t.lands(P0, "Wastes", 3);
    let before = t.g.battlefield.len();
    // With no card exiled with Mairsil's instance, X is undefined (Prototype Portal's
    // ruling: "You may not activate the second ability if no card has been exiled with
    // Prototype Portal."): it can't be activated, and certainly makes no Portal token.
    assert!(activate_containing(&mut t, P0, m, "copy of the exiled card").is_err());
    t.resolve_all();
    assert_eq!(t.g.battlefield.len(), before, "no token");
    assert!(t.named_on_battlefield("Prototype Portal").is_empty());
    // The Portal itself, having exiled nothing, can't either; with a card exiled, it can.
    let portal = t.battlefield(P0, "Prototype Portal");
    assert!(activate_containing(&mut t, P0, portal, "copy of the exiled card").is_err());
}

#[test]
fn mairsil_with_izzet_chemister_links_its_two_gained_abilities() {
    cr!("607.2a", "607.5");
    ruling!(
        "Mairsil, the Pretender",
        "If Mairsil gains an activated ability that's normally linked to another activated ability of the card it came from, those two abilities Mairsil gains are linked for as long as Mairsil remains on the battlefield."
    );
    // Izzet Chemister: "{R}, {T}: Exile target instant or sorcery card from your
    // graveyard. {1}{R}, {T}, Sacrifice this creature: Cast any number of cards exiled
    // with this creature without paying their mana costs."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    cage(&mut t, "Izzet Chemister");
    let axe = t.graveyard(P0, "Lava Axe");
    t.lands(P0, "Mountain", 3);
    t.answer_targets(P0, &[Entity::Object(axe)]);
    activate_containing(&mut t, P0, m, "Exile target instant").expect("first ability");
    t.resolve_all();
    assert!(t.in_exile("Lava Axe"));
    // Untapped for the second ability (a new turn's activations).
    let now = t.g.current(m);
    t.g.objects[now.0 as usize].tapped = false;
    t.g.recompute();
    let exiled = t.g.current(axe);
    t.answer_choose(P0, &[Entity::Object(exiled)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, m, "Cast any number").expect("second ability");
    t.resolve_all();
    assert_eq!(t.life(P1), 15, "the Lava Axe exiled with Mairsil was cast");
    assert!(!t.on_battlefield(m));
    assert!(
        t.in_exile("Izzet Chemister"),
        "Chemister itself stays in exile"
    );
}

#[test]
fn mairsil_references_to_the_cards_name_mean_mairsil() {
    cr!("201.5", "602.2");
    ruling!(
        "Mairsil, the Pretender",
        "If an activated ability of a card in exile with a cage counter on it references the card it's printed on by name, treat Mairsil's instance of that ability as though it referenced Mairsil by name instead."
    );
    // Mogg Fanatic: "Sacrifice this creature: It deals 1 damage to any target."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    cage(&mut t, "Mogg Fanatic");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, m, "Sacrifice").expect("the Fanatic's ability");
    assert!(!t.on_battlefield(m), "Mairsil was sacrificed");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.in_exile("Mogg Fanatic"));
}

#[test]
fn mairsil_ability_that_didnt_resolve_still_counts_for_the_turn() {
    cr!("602.5b", "608.2b");
    ruling!(
        "Mairsil, the Pretender",
        "If one of Mairsil's abilities doesn't resolve, most likely because its target became illegal before it resolved, it can't be activated again in the same turn."
    );
    // Prodigal Sorcerer: "{T}: This creature deals 1 damage to any target."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    cage(&mut t, "Prodigal Sorcerer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, m, "damage").unwrap();
    crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    let now = t.g.current(m);
    t.g.objects[now.0 as usize].tapped = false;
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(activate_containing(&mut t, P0, m, "damage").is_err());
    // Next turn it can.
    t.advance_to(P1, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, m, "damage").expect("a new turn");
}

#[test]
fn a_copy_of_mairsil_triggers_and_the_legend_rule_keeps_either() {
    cr!("704.5j", "707.2", "603.6a");
    ruling!(
        "Mairsil, the Pretender",
        "If you have a creature enter the battlefield as a copy of Mairsil, its first ability triggers. You won't be able to activate any abilities before the \"legend rule\" applies, but the triggered ability will still let you exile another card with a cage counter regardless of which Mairsil you keep."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    let sorc = t.hand(P0, "Prodigal Sorcerer");
    t.answer_choose(P0, &[Entity::Object(m)]);
    let clone = t.enter(P0, "Clone");
    // Keep the original.
    t.answer_choose(P0, &[Entity::Object(m)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(sorc)]);
    t.resolve_all();
    assert!(!t.on_battlefield(clone));
    assert!(t.on_battlefield(m));
    let caged = t.g.current(sorc);
    assert_eq!(t.zone(caged), Zone::Exile);
    assert_eq!(t.counters(caged, "cage"), 1);
    assert!(texts(&mut t, m, true).iter().any(|a| a.contains("damage")));
}

#[test]
fn caged_cards_stay_caged_when_mairsil_leaves() {
    cr!("400.7", "122.2");
    ruling!(
        "Mairsil, the Pretender",
        "The exiled cards remain exiled with cage counters when Mairsil leaves the battlefield. If Mairsil returns to the battlefield, it will see all of those exiled cards with cage counters on them."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mairsil, the Pretender");
    let caged = cage(&mut t, "Prodigal Sorcerer");
    assert!(texts(&mut t, m, true).iter().any(|a| a.contains("damage")));
    crate::r_s05_common::move_to(&mut t, m, Zone::Hand(P0));
    assert_eq!(t.zone(caged), Zone::Exile);
    assert_eq!(t.counters(caged, "cage"), 1);
    t.answer_yes(P0, false);
    let back = t.enter(P0, "Mairsil, the Pretender");
    t.resolve_all();
    assert!(texts(&mut t, back, true)
        .iter()
        .any(|a| a.contains("damage")));
}
