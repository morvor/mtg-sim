//! Spells cast and lands played "this way" (gap R7; `permissions::PermissionUse`): the
//! permission a card is played with is recorded on the spell or land (CR 601.3, 305.1),
//! and the effect that gave it can find that object (CR 400.7h, 400.7i): a delayed
//! triggered ability that triggers only for a card played with that permission
//! (CR 603.7), "If you cast a creature spell this way" applied to the spell and, by
//! CR 400.7a, to the permanent it becomes.

use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn activated(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.clone())
        .collect()
}

#[test]
fn havengul_lich_gains_the_abilities_when_the_card_is_cast_this_way() {
    cr!("400.7h", "603.7a", "603.7b", "601.3");
    ruling!(
        "Havengul Lich",
        "This triggered ability will go on the stack on top of the creature spell and will resolve before the creature spell will."
    );
    ruling!(
        "Havengul Lich",
        "If you don’t cast the creature card that turn, Havengul Lich won’t gain any of its abilities."
    );
    ruling!(
        "Havengul Lich",
        "gains the activated abilities of the card as it existed in the graveyard while that card is still a spell on the stack"
    );
    assert_supported(&["Havengul Lich"]);
    // Prodigal Sorcerer ("{T}: This creature deals 1 damage to any target.") in P1's
    // graveyard.
    let mut t = TestGame::new(2);
    let lich = t.battlefield(P0, "Havengul Lich");
    let sorcerer = t.graveyard(P1, "Prodigal Sorcerer");
    t.lands(P0, "Island", 4);
    let before = activated(&mut t, lich).len();
    t.activate(P0, lich, 0, &[Entity::Object(sorcerer)]).unwrap();
    t.resolve_all();
    let spell = t.cast(P0, sorcerer).go();
    t.settle();
    // The delayed trigger is on top of the creature spell.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.zone(spell), mtg_engine::object::Zone::Stack);
    assert_eq!(activated(&mut t, lich).len(), before + 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Prodigal Sorcerer").len(), 1);
    // Not cast that turn: no abilities.
    let mut t = TestGame::new(2);
    let lich = t.battlefield(P0, "Havengul Lich");
    let sorcerer = t.graveyard(P1, "Prodigal Sorcerer");
    t.lands(P0, "Island", 1);
    let before = activated(&mut t, lich).len();
    t.activate(P0, lich, 0, &[Entity::Object(sorcerer)]).unwrap();
    t.resolve_all();
    assert_eq!(activated(&mut t, lich).len(), before);
}

#[test]
fn fires_of_mount_doom_deals_damage_only_for_a_card_played_with_that_permission() {
    cr!("400.7i", "603.7a");
    ruling!(
        "Fires of Mount Doom",
        "using that casting permission, Fires of Mount Doom won't deal damage to each player"
    );
    assert_supported(&["Fires of Mount Doom"]);
    // The exiled card is a Mountain: playing it deals 2 damage to each player.
    let mut t = TestGame::new(2);
    let fires = t.battlefield(P0, "Fires of Mount Doom");
    t.lands(P0, "Mountain", 3);
    let land = t.library_top(P0, "Mountain");
    t.activate(P0, fires, 0, &[]).unwrap();
    t.resolve_all();
    t.play_land(P0, t.g.current(land)).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 18);
    // A land played from the hand (not with that permission) doesn't.
    let mut t = TestGame::new(2);
    let fires = t.battlefield(P0, "Fires of Mount Doom");
    t.lands(P0, "Mountain", 3);
    t.library_top(P0, "Lightning Bolt");
    let other = t.hand(P0, "Mountain");
    t.activate(P0, fires, 0, &[]).unwrap();
    t.resolve_all();
    t.play_land(P0, other).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn thundermane_dragon_creature_cast_this_way_keeps_haste() {
    cr!("400.7a", "601.3");
    ruling!(
        "Thundermane Dragon",
        "allows you to cast the top card of your library if the resulting spell is a creature spell with power 4 or greater"
    );
    assert_supported(&["Thundermane Dragon"]);
    // Hill Giant (3/3) can't be cast from the top; Craw Wurm (6/4) can, and has haste.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thundermane Dragon");
    t.lands(P0, "Forest", 6);
    let wurm = t.library_top(P0, "Craw Wurm");
    let spell = t.cast(P0, wurm).go();
    t.g.recompute();
    assert!(t.g.obj(spell).has_keyword(KeywordKind::Haste));
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    assert!(t.obj_now(w[0]).has_keyword(KeywordKind::Haste));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thundermane Dragon");
    t.lands(P0, "Forest", 6);
    let giant = t.library_top(P0, "Hill Giant");
    assert!(t.cast(P0, giant).try_go().is_err());
    // Cast from the hand: no haste.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thundermane Dragon");
    t.lands(P0, "Forest", 6);
    let wurm = t.hand(P0, "Craw Wurm");
    t.cast(P0, wurm).go();
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm");
    assert!(!t.obj_now(w[0]).has_keyword(KeywordKind::Haste));
}

#[test]
fn galea_equipment_cast_this_way_attaches_as_it_enters() {
    cr!("400.7a", "603.2", "601.3");
    ruling!(
        "Galea, Kindler of Hope",
        "You'll still pay all costs for the spell, including additional costs."
    );
    assert_supported(&["Galea, Kindler of Hope"]);
    // Bonesplitter from the top of the library: it enters attached to Grizzly Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galea, Kindler of Hope");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let bs = t.library_top(P0, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, bs).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    // The permission doesn't pay the spell's costs: with no mana it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galea, Kindler of Hope");
    t.battlefield(P0, "Grizzly Bears");
    let bs = t.library_top(P0, "Bonesplitter");
    assert!(t.cast(P0, bs).try_go().is_err());
    // From the hand: it doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galea, Kindler of Hope");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let bs = t.hand(P0, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, bs).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn strago_and_relm_creature_cast_this_way_gains_haste() {
    cr!("400.7a", "608.2g");
    ruling!(
        "Strago and Relm",
        "You cast the card while Strago and Relm's ability is resolving"
    );
    assert_supported(&["Strago and Relm"]);
    let mut t = TestGame::new(2);
    let sr = t.battlefield(P0, "Strago and Relm");
    t.lands(P0, "Mountain", 3);
    t.library_top(P1, "Craw Wurm");
    t.library_top(P1, "Forest");
    t.answer_yes(P0, true);
    t.activate(P0, sr, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    assert_eq!(t.obj_now(w[0]).controller, P0);
    assert!(t.obj_now(w[0]).has_keyword(KeywordKind::Haste));
    // "At the beginning of the end step, sacrifice this creature."
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
}
