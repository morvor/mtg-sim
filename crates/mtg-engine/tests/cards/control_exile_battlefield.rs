//! Putting cards onto the battlefield and what they are as they enter (patterns in
//! `src/oracle/patterns/control_exile_battlefield.rs`).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn battlefield_cards_compile() {
    assert_compiles(&[
        "Reanimate",
        "Rise from the Grave",
        "Debtors' Knell",
        "Ashiok, Sculptor of Fears",
        "Enduring Vitality",
        "Enduring Curiosity",
        "Enduring Tenacity",
        "Enduring Innocence",
        "Enduring Courage",
    ]);
}

#[test]
fn put_target_creature_card_from_a_graveyard_onto_the_battlefield() {
    cr!("110.2", "110.2a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let giant = t.graveyard(P1, "Hill Giant");
    let spell = t.hand(P0, "Reanimate");
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // Under the control of the player who put it there; its owner is unchanged.
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(giant).owner, P1);
    // "You lose life equal to its mana value."
    assert_eq!(t.life(P0), 16);
}

#[test]
fn that_creature_is_a_black_zombie_in_addition() {
    cr!("611.2e", "205.1b", "105.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    // "Whenever another Zombie you control enters, each opponent loses 1 life and you gain
    // 1 life": the creature is a Zombie as it enters, not only afterwards.
    t.battlefield(P0, "Wayward Servant");
    let bear = t.graveyard(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Rise from the Grave");
    t.cast(P0, spell).target(bear).go();
    t.resolve_all();
    let o = t.obj_now(bear);
    assert_eq!(o.zone, Zone::Battlefield);
    assert_eq!(o.controller, P0);
    assert!(o.chars.has_subtype("Zombie"));
    assert!(o.chars.has_subtype("Bear"));
    assert!(o.chars.colors.contains(Color::Black));
    assert!(o.chars.colors.contains(Color::Green));
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn reanimate_at_the_beginning_of_upkeep() {
    cr!("110.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Debtors' Knell");
    let bear = t.graveyard(P1, "Grizzly Bears");
    t.set_step(P0, Step::Untap);
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.advance_to(P0, Step::Draw);
    assert!(t.on_battlefield(bear));
    assert_eq!(t.obj_now(bear).controller, P0);
}

#[test]
fn enduring_returns_as_a_noncreature_enchantment_once() {
    cr!("611.2e", "603.4", "603.10a", "205.3d");
    ruling!(
        "Enduring Vitality",
        "Elk and Glimmer are both creature types"
    );
    let mut t = TestGame::new(2);
    // P0 controls a Vitality P1 owns: it returns under its owner's control.
    let vitality = t.battlefield(P1, "Enduring Vitality");
    t.g.objects[vitality.0 as usize].controller = P0;
    t.g.objects[vitality.0 as usize].base_controller = P0;
    t.g.recompute();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(vitality).go();
    t.resolve_all();
    let o = t.obj_now(vitality);
    assert_eq!(o.zone, Zone::Battlefield);
    assert_eq!(o.controller, P1);
    assert!(o.is(CardType::Enchantment));
    assert!(!o.is(CardType::Creature));
    assert!(!o.chars.has_subtype("Elk"));
    assert!(!o.chars.has_subtype("Glimmer"));
    // Its other abilities still work: "Creatures you control have '{T}: Add one mana of
    // any color.'" (no creature here to check), and it isn't a creature any more, so when
    // it's put into a graveyard again it doesn't return.
    let now = t.g.current(vitality);
    t.lands(P0, "Forest", 2);
    let naturalize = t.hand(P0, "Naturalize");
    t.cast(P0, naturalize).target(now).go();
    t.resolve_all();
    assert_eq!(t.zone(vitality), Zone::Graveyard(P1));
}

#[test]
fn then_that_player_mills_is_the_graveyards_owner() {
    cr!("107.3", "110.2");
    ruling!(
        "Geth, Lord of the Vault",
        "The target card must have mana value exactly X"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let geth = t.battlefield(P0, "Geth, Lord of the Vault");
    let bear = t.graveyard(P1, "Grizzly Bears");
    let lib0 = t.library_size(P0);
    let lib1 = t.library_size(P1);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.activate(P0, geth, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    let o = t.obj_now(bear);
    assert_eq!(o.zone, Zone::Battlefield);
    assert_eq!(o.controller, P0);
    assert!(o.tapped);
    // "Then that player mills X cards": the opponent, not you.
    assert_eq!(t.library_size(P1), lib1 - 2);
    assert_eq!(t.library_size(P0), lib0);

    // With X = 3, a card with mana value 2 isn't a legal target: X can't be overpaid to
    // mill more cards.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let geth = t.battlefield(P0, "Geth, Lord of the Vault");
    let bear = t.graveyard(P1, "Grizzly Bears");
    let lib1 = t.library_size(P1);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(3));
    let _ = t.activate(P0, geth, 0, &[Entity::Object(bear)]);
    t.resolve_all();
    assert_eq!(t.zone(bear), Zone::Graveyard(P1));
    assert_eq!(t.library_size(P1), lib1);
}

#[test]
fn that_creature_deals_damage_to_each_other_creature() {
    cr!("110.2", "120.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    t.lands(P0, "Mountain", 1);
    let giant = t.graveyard(P1, "Hill Giant");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Gray Ogre");
    let spell = t.hand(P0, "Too Greedily, Too Deep");
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    // The 3/3 deals 3 damage to each other creature, but not to itself.
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert!(!t.on_battlefield(bear));
    assert!(!t.on_battlefield(ogre));
}
