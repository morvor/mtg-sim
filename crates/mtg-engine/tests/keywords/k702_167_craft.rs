//! CR 702.167 Craft.

use crate::common_k702_140_152::{activatable, ability_uid, activate_uid};
use crate::common_k702_153_167::*;
use mtg_engine::decision::Decision;
use mtg_engine::kw::craft::parse_materials;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

const LANDMARK: &str = "Oteclan Landmark";

/// Queues the materials `p` chooses for a craft ability.
fn materials(t: &mut TestGame, p: PlayerId, objs: &[ObjectId]) {
    let es: Vec<Entity> = objs.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(p, &es);
}

/// The objects offered as materials for the last craft activation.
fn offered(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.starts_with("Craft") => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn craft_exiles_it_and_its_materials_to_return_it_transformed() {
    cr!("702.167", "702.167a");
    assert_supported(LANDMARK);
    // Oteclan Landmark: "Craft with artifact {2}{W}"; back face Oteclan Levitator, a
    // flying Golem artifact creature.
    let mut t = TestGame::new(2);
    let landmark = t.battlefield(P0, LANDMARK);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Plains", 3);
    let craft = ability_uid(&mut t, landmark, "Craft");
    assert!(activatable(&mut t, P0, landmark, craft));
    materials(&mut t, P0, &[thopter]);
    activate_uid(&mut t, P0, landmark, craft).unwrap();
    // Both were exiled to pay the cost.
    assert_eq!(t.zone(thopter), Zone::Exile);
    assert_eq!(t.zone(landmark), Zone::Exile);
    t.resolve_all();
    // Returned transformed under its owner's control.
    let lev = t.g.current(landmark);
    assert_eq!(t.zone(lev), Zone::Battlefield);
    let o = t.g.obj(lev);
    assert_eq!(o.face, FaceState::Back);
    assert_eq!(o.chars.name, "Oteclan Levitator");
    assert!(o.is(CardType::Creature));
    assert_eq!(o.controller, P0);
}

#[test]
fn craft_is_activated_only_as_a_sorcery_and_only_with_materials() {
    cr!("702.167a");
    // No other artifact: it can't be activated.
    let mut t = TestGame::new(2);
    let landmark = t.battlefield(P0, LANDMARK);
    t.lands(P0, "Plains", 3);
    let craft = ability_uid(&mut t, landmark, "Craft");
    assert!(!activatable(&mut t, P0, landmark, craft));
    assert!(activate_uid(&mut t, P0, landmark, craft).is_err());
    // It can't craft itself.
    t.battlefield(P0, "Ornithopter");
    assert!(activatable(&mut t, P0, landmark, craft));
    // Not during an opponent's turn, or with something on the stack.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!activatable(&mut t, P0, landmark, craft));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!activatable(&mut t, P0, landmark, craft));
}

#[test]
fn materials_are_permanents_you_control_or_cards_in_your_graveyard() {
    cr!("702.167b");
    ruling!(
        "Oteclan Landmark // Oteclan Levitator",
        "you may exile some of them from among permanents you control and the rest from among cards in your graveyard"
    );
    // "artifact": another artifact you control or an artifact card in your graveyard —
    // not one in your hand, an opponent's, or one in an opponent's graveyard.
    let mut t = TestGame::new(2);
    let landmark = t.battlefield(P0, LANDMARK);
    let mine = t.battlefield(P0, "Ornithopter");
    let yard = t.graveyard(P0, "Memnite");
    let hand = t.hand(P0, "Memnite");
    let theirs = t.battlefield(P1, "Ornithopter");
    let their_yard = t.graveyard(P1, "Memnite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let craft = ability_uid(&mut t, landmark, "Craft");
    materials(&mut t, P0, &[yard]);
    activate_uid(&mut t, P0, landmark, craft).unwrap();
    let cands = offered(&t);
    assert!(cands.contains(&Entity::Object(mine)));
    assert!(cands.contains(&Entity::Object(yard)));
    for no in [hand, theirs, their_yard, bears, landmark] {
        assert!(!cands.contains(&Entity::Object(no)));
    }
    assert_eq!(t.zone(yard), Zone::Exile);
    assert!(t.on_battlefield(mine));
    // Described as cards, materials come only from the graveyard.
    let m = parse_materials("four or more red instant and/or sorcery cards").unwrap();
    assert!(m.cards_only);
    assert_eq!((m.min, m.max), (4, None));
    let m = parse_materials("two creatures").unwrap();
    assert!(!m.cards_only);
    assert_eq!((m.min, m.max), (2, Some(2)));
    let def = custom_card(
        "Bone Kiln",
        "Artifact",
        None,
        "Craft with two creature cards {1}",
    );
    let mut t = TestGame::new(2);
    let kiln = t.custom(P0, def, Zone::Battlefield);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 1);
    let craft = ability_uid(&mut t, kiln, "Craft");
    assert!(!activatable(&mut t, P0, kiln, craft), "creatures aren't cards");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    assert!(activatable(&mut t, P0, kiln, craft));
}

#[test]
fn several_materials_may_come_from_both_places_and_must_fit_the_description() {
    cr!("702.167a", "702.167b");
    // Sunbird Standard: "Craft with one or more {5}".
    let mut t = TestGame::new(2);
    let sunbird = t.battlefield(P0, "Sunbird Standard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.graveyard(P0, "Raging Goblin");
    t.lands(P0, "Swamp", 5);
    let craft = ability_uid(&mut t, sunbird, "Craft");
    materials(&mut t, P0, &[bears, goblin]);
    activate_uid(&mut t, P0, sunbird, craft).unwrap();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(goblin), Zone::Exile);
    // "Two that share a card type" (Eye of Ojer Taq, "Craft with two that share a card
    // type {6}"): a choice of two that don't isn't accepted.
    let m = parse_materials("two that share a card type").unwrap();
    assert!(m.share_card_type);
    let mut t = TestGame::new(2);
    let eye = t.battlefield(P0, "Eye of Ojer Taq");
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let plains = t.graveyard(P0, "Plains");
    t.lands(P0, "Swamp", 6);
    let craft = ability_uid(&mut t, eye, "Craft");
    materials(&mut t, P0, &[thopter, plains]);
    activate_uid(&mut t, P0, eye, craft).unwrap();
    // Instead, two that share a card type (creature) were exiled.
    assert_eq!(t.zone(plains), Zone::Graveyard(P0));
    assert_eq!(t.zone(thopter), Zone::Exile);
    assert_eq!(t.zone(bears), Zone::Exile);
    // Lands you control share a card type with a land card in your graveyard.
    let mut t = TestGame::new(2);
    let eye = t.battlefield(P0, "Eye of Ojer Taq");
    t.battlefield(P0, "Glorious Anthem");
    t.graveyard(P0, "Plains");
    t.lands(P0, "Swamp", 6);
    let craft = ability_uid(&mut t, eye, "Craft");
    assert!(activatable(&mut t, P0, eye, craft));
    // With nothing sharing a card type (an enchantment and a creature), it can't be
    // activated.
    let mut t = TestGame::new(2);
    let eye = t.battlefield(P0, "Eye of Ojer Taq");
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P0, "Grizzly Bears");
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::C, 6);
    let craft = ability_uid(&mut t, eye, "Craft");
    assert!(!activatable(&mut t, P0, eye, craft));
    // "a Dinosaur, a Merfolk, a Pirate, and a Vampire": one of each.
    let m = parse_materials("a Dinosaur, a Merfolk, a Pirate, and a Vampire").unwrap();
    assert_eq!(m.slots.len(), 4);
}

#[test]
fn the_exiled_cards_used_to_craft_it() {
    cr!("702.167c");
    ruling!(
        "Oteclan Landmark // Oteclan Levitator",
        "Those cards are considered to be \"used to craft\" that permanent as long as they remain exiled and the permanent remains on the battlefield"
    );
    // Sunbird Effigy: "power and toughness are each equal to the number of colors among
    // the exiled cards used to craft it."
    let mut t = TestGame::new(2);
    let sunbird = t.battlefield(P0, "Sunbird Standard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.graveyard(P0, "Raging Goblin");
    let other = t.graveyard(P0, "Llanowar Elves");
    // An exiled card that wasn't used to craft it doesn't count.
    let lions = t.exile(P0, "Savannah Lions");
    t.lands(P0, "Swamp", 5);
    let craft = ability_uid(&mut t, sunbird, "Craft");
    materials(&mut t, P0, &[bears, goblin]);
    activate_uid(&mut t, P0, sunbird, craft).unwrap();
    t.resolve_all();
    let effigy = t.g.current(sunbird);
    assert_eq!(t.obj(effigy).chars.name, "Sunbird Effigy");
    // Green and red.
    assert_eq!(t.pt(effigy), (2, 2));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    let _ = (other, lions);
    // Once one of them leaves exile, it no longer counts.
    let b = t.g.current(bears);
    t.g.move_object(
        b,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.g.recompute();
    assert_eq!(t.pt(effigy), (1, 1));
}
