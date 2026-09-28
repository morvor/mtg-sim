//! Rulings batch S24 — abilities that refer to permanents of a creature type you control:
//! Slivers that affect only Sliver creatures you control (CR 205.3, 613.1d), and outlaws
//! (CR 700.12): Assassins, Mercenaries, Pirates, Rogues and Warlocks.

use crate::r_s01_common::{supported, with_subtype};
use crate::r_s04_common::spell_targets;
use crate::r_s05_common::enter;
use crate::r_s06_common::{attach_new, has_kw};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_sliver_that_stops_being_a_sliver_isnt_affected_by_its_own_ability() {
    cr!("613.1d", "613.1f", "613.8a");
    ruling!(
        "Galerider Sliver",
        "If you change the creature type of a Sliver you control so it’s no longer a Sliver, it will no longer be affected by its own ability. Its ability will continue to affect other Sliver creatures you control."
    );
    supported("Galerider Sliver");
    supported("Amoeboid Changeling");
    // "Sliver creatures you control have flying."
    let mut t = TestGame::new(2);
    let galerider = t.battlefield(P0, "Galerider Sliver");
    let venom = t.battlefield(P0, "Venom Sliver");
    assert!(has_kw(&t, galerider, KeywordKind::Flying));
    assert!(has_kw(&t, venom, KeywordKind::Flying));
    // Amoeboid Changeling: "{T}: Target creature loses all creature types until end of
    // turn."
    let amoeboid = t.battlefield(P0, "Amoeboid Changeling");
    t.activate(P0, amoeboid, 1, &[Entity::Object(galerider)])
        .expect("activate");
    t.resolve_all();
    assert!(!t.obj_now(galerider).chars.has_subtype("Sliver"));
    assert!(!has_kw(&t, galerider, KeywordKind::Flying));
    assert!(has_kw(&t, venom, KeywordKind::Flying));
}

#[test]
fn these_slivers_affect_only_sliver_creatures_you_control() {
    cr!("611.3a", "109.4");
    ruling!(
        "Venom Sliver",
        "Slivers in this set affect only Sliver creatures you control. They don’t grant bonuses to your opponents’ Slivers."
    );
    supported("Venom Sliver");
    // "Sliver creatures you control have deathtouch."
    let mut t = TestGame::new(2);
    let venom = t.battlefield(P0, "Venom Sliver");
    let mine = t.battlefield(P0, "Galerider Sliver");
    let theirs = t.battlefield(P1, "Galerider Sliver");
    assert!(has_kw(&t, venom, KeywordKind::Deathtouch));
    assert!(has_kw(&t, mine, KeywordKind::Deathtouch));
    assert!(!has_kw(&t, theirs, KeywordKind::Deathtouch));
    // And the opponent's Galerider gives flying only to their own Slivers.
    assert!(has_kw(&t, theirs, KeywordKind::Flying));
    assert!(t.obj_now(mine).has_keyword(KeywordKind::Flying));
}

#[test]
fn outlaw_means_an_outlaw_permanent_but_an_outlaw_spell_is_on_the_stack() {
    cr!("700.12");
    ruling!(
        "Hellspur Posse Boss",
        "If an ability refers to an outlaw or whether a player controls an outlaw, it’s referring only to permanents with one or more of the creature types specified above. Notably, it’s not referring to any spell or card not on the battlefield. However, other abilities may refer to an “outlaw spell” or “outlaw card” in a zone other than the battlefield."
    );
    supported("Hellspur Posse Boss");
    supported("Shoot the Sheriff");
    supported("Discreet Retreat");
    // "Other outlaws you control have haste." A Rogue permanent has haste; a Bear doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellspur Posse Boss");
    let rogue = t.battlefield_sick(P0, "Bane Alley Blackguard");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    assert!(has_kw(&t, rogue, KeywordKind::Haste));
    assert!(!has_kw(&t, bears, KeywordKind::Haste));
    // Shoot the Sheriff: "Destroy target non-outlaw creature."
    let targets = spell_targets(&mut t, P1, "Shoot the Sheriff");
    assert!(targets.contains(&Entity::Object(bears)));
    assert!(!targets.contains(&Entity::Object(rogue)));
    // Discreet Retreat: "Whenever you cast your first outlaw spell each turn, you draw a
    // card and you lose 1 life." A Rogue spell on the stack is an outlaw spell.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let forests = t.lands(P0, "Forest", 2);
    attach_new(&mut t, P0, "Discreet Retreat", forests[0]);
    let bears = t.hand(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), hand - 1);
    let rogue = t.hand(P0, "Bane Alley Blackguard");
    t.cast(P0, rogue).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn another_outlaw_you_control_is_a_permanent_and_an_outlaw_card_is_in_another_zone() {
    cr!("700.12", "603.4");
    ruling!(
        "Mine Raider",
        "If an ability refers to an outlaw or whether a player controls an outlaw, it's referring only to permanents with one or more of the creature types specified above. Notably, it's not referring to any spell or card not on the battlefield."
    );
    ruling!(
        "Charred Graverobber",
        "However, other abilities may refer to an \"outlaw spell\" or \"outlaw card\" in a zone other than the battlefield. Those abilities refer to spells and cards with one or more of the specified creature types."
    );
    supported("Mine Raider");
    supported("Charred Graverobber");
    // Mine Raider: "When this creature enters, if you control another outlaw, create a
    // Treasure token." Outlaw cards in the hand and graveyard don't count.
    let mut t = TestGame::new(2);
    t.hand(P0, "Bane Alley Blackguard");
    t.graveyard(P0, "Bane Alley Blackguard");
    enter(&mut t, P0, "Mine Raider");
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Treasure").is_empty());
    t.battlefield(P0, "Bane Alley Blackguard");
    enter(&mut t, P0, "Mine Raider");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    // Charred Graverobber: "return target outlaw card from your graveyard to your hand".
    let mut t = TestGame::new(2);
    let rogue_card = t.graveyard(P0, "Bane Alley Blackguard");
    let bears_card = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(rogue_card)]);
    enter(&mut t, P0, "Charred Graverobber");
    let asked = t.asked();
    let candidates = asked
        .iter()
        .find_map(|(p, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } if *p == P0 => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .expect("targets were chosen");
    assert!(candidates.contains(&Entity::Object(rogue_card)));
    assert!(!candidates.contains(&Entity::Object(bears_card)));
    t.resolve_all();
    assert!(t.in_hand(P0, "Bane Alley Blackguard"));
}
