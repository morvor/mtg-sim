//! Additional costs: "behold a Kithkin and exile it" (CR 701.4a, 607.2q) and two costs
//! with their own verbs ("discard a card and sacrifice a creature", CR 601.2b).

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn champion_of_the_clachan_exiles_the_beheld_kithkin_and_returns_it() {
    cr!("701.4a", "607.2q");
    assert_supported("Champion of the Clachan");
    let mut t = TestGame::new(2);
    let harrier = t.battlefield(P0, "Goldmeadow Harrier");
    t.lands(P0, "Plains", 4);
    let champ = t.hand(P0, "Champion of the Clachan");
    t.answer_choose(P0, &[Entity::Object(harrier)]);
    t.cast(P0, champ).go();
    // The cost exiled the Harrier.
    assert!(t.in_exile("Goldmeadow Harrier"));
    t.resolve();
    let champ = t.named_on_battlefield("Champion of the Clachan")[0];
    t.g.destroy(champ, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Goldmeadow Harrier"));
}

#[test]
fn ruthless_disposal_discards_and_sacrifices() {
    cr!("601.2b", "601.2h");
    assert_supported("Ruthless Disposal");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Craw Wurm");
    t.hand(P0, "Island");
    t.lands(P0, "Swamp", 5);
    let rd = t.hand(P0, "Ruthless Disposal");
    t.cast(P0, rd)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    assert!(!t.on_battlefield(mine));
    assert!(t.in_graveyard(P0, "Island"));
    t.resolve();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn transmogrants_crown_equips_for_either_cost() {
    cr!("702.6a");
    assert_supported("Transmogrant's Crown");
    let mut t = TestGame::new(2);
    let crown = t.battlefield(P0, "Transmogrant's Crown");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    // The {B} equip ability (the second one).
    t.activate(P0, crown, 1, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn my_precious_equip_costs_mana_and_life() {
    cr!("702.6a");
    assert_supported("My Precious // Allure of Power");
    let mut t = TestGame::new(2);
    let mp = t.battlefield(P0, "My Precious // Allure of Power");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.activate(P0, mp, 0, &[Entity::Object(bears)]).unwrap();
    assert_eq!(t.life(P0), 18);
    t.resolve();
    assert_eq!(t.obj_now(mp).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn sauron_ward_demands_a_legendary_artifact_or_creature() {
    cr!("702.21a");
    assert_supported("Sauron, the Dark Lord");
    for has_legend in [false, true] {
        let mut t = TestGame::new(2);
        let sauron = t.battlefield(P0, "Sauron, the Dark Lord");
        if has_legend {
            t.battlefield(P1, "Isamaru, Hound of Konda");
        }
        t.lands(P1, "Mountain", 1);
        let shock = t.hand(P1, "Shock");
        t.cast(P1, shock).target(sauron).go();
        t.answer_yes(P1, true);
        t.resolve_all();
        assert_eq!(t.obj_now(sauron).damage, if has_legend { 2 } else { 0 });
        assert_eq!(t.in_graveyard(P1, "Isamaru, Hound of Konda"), has_legend);
    }
}

#[test]
fn hulks_thunderclap_needs_its_last_target_only_if_it_beheld() {
    cr!("601.2c", "701.4a");
    assert_supported("Hulk's Thunderclap");
    // Not beheld: castable with no noncreature artifact or enchantment to target.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 2);
    let clap = t.hand(P0, "Hulk's Thunderclap");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, clap).target(bears).target(giant).go();
    t.resolve();
    assert_eq!(t.g.obj(giant).damage, 2);
    // Beheld: it also destroys the third target.
    let mut t = TestGame::new(2);
    let brawn = t.battlefield(P0, "Brawn, Amadeus Cho");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let stone = t.battlefield(P1, "Mind Stone");
    t.lands(P0, "Forest", 2);
    let clap = t.hand(P0, "Hulk's Thunderclap");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(brawn)]);
    t.cast(P0, clap)
        .target(bears)
        .target(giant)
        .target(stone)
        .go();
    t.resolve();
    assert!(!t.on_battlefield(stone));
}
