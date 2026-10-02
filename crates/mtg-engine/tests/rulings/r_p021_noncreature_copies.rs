//! Rulings batch P021 — copying a permanent that's a creature only because of a non-copy
//! effect (an animated land, Saga or planeswalker) copies a noncreature: a creature
//! subtype or power/toughness the copy effect adds doesn't stick to a noncreature, even
//! if it later becomes a creature (CR 205.3d, 208.3, 707.2, 707.9b), and Machine God's
//! Effigy, an artifact with the copied abilities, isn't a Saga, a land, or a planeswalker
//! (CR 205.1a, 305.6, 306.5b, 714.3).

use crate::r_p021_common::*;
use crate::r_p023_common::enter_copying;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s26_common::new_tokens;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s Forest, made a 2/2 creature until end of turn (still a land).
fn animated_forest(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let f = t.battlefield(p, "Forest");
    animate_land(t, f);
    f
}

#[test]
fn glasspool_mimic_copying_an_animated_land_isnt_a_shapeshifter_rogue() {
    cr!("707.9b", "205.3d");
    ruling!(
        "Glasspool Mimic // Glasspool Shore",
        "If Glasspool Mimic isn't a creature, most likely because it copied a creature that was only temporarily a creature, it won't be a Shapeshifter Rogue, even if it becomes a creature later."
    );
    supported("Glasspool Mimic // Glasspool Shore");
    let mut t = TestGame::new(2);
    let forest = animated_forest(&mut t, P0);
    let mimic = enter_copying(&mut t, P0, "Glasspool Mimic // Glasspool Shore", forest);
    assert_eq!(t.obj_now(mimic).chars.name, "Forest");
    assert!(!t.obj_now(mimic).is(CardType::Creature));
    assert!(!has_subtype(&t, mimic, "Shapeshifter") && !has_subtype(&t, mimic, "Rogue"));
    animate_land(&mut t, mimic);
    assert!(has_subtype(&t, mimic, "Forest"));
    assert!(!has_subtype(&t, mimic, "Shapeshifter") && !has_subtype(&t, mimic, "Rogue"));
}

#[test]
fn pirated_copy_copying_an_animated_land_isnt_a_pirate_but_has_the_ability() {
    cr!("707.9b", "205.3d");
    ruling!(
        "Pirated Copy",
        "If Pirated Copy doesn't enter the battlefield as a creature (such as by copying an artifact or land that became a creature), it doesn't become a Pirate, though it still has the granted triggered ability."
    );
    supported("Pirated Copy");
    let mut t = TestGame::new(2);
    let forest = animated_forest(&mut t, P1);
    let pc = enter_copying(&mut t, P0, "Pirated Copy", forest);
    assert_eq!(t.obj_now(pc).chars.name, "Forest");
    assert!(!t.obj_now(pc).is(CardType::Creature));
    assert!(!has_subtype(&t, pc, "Pirate"));
    assert_eq!(abilities_with(&t, pc, "deals combat damage to a player"), 1);
    animate_land(&mut t, pc);
    assert!(!has_subtype(&t, pc, "Pirate"));
}

#[test]
fn quicksilver_gargantuan_copying_an_animated_land_has_no_power_or_toughness() {
    cr!("707.9b", "208.3");
    ruling!(
        "Quicksilver Gargantuan",
        "If Quicksilver Gargantuan is not a creature (for example, if it entered as a copy of an animated land), it will not have the characteristics of power or toughness at all"
    );
    supported("Quicksilver Gargantuan");
    let mut t = TestGame::new(2);
    let forest = animated_forest(&mut t, P1);
    let qg = enter_copying(&mut t, P0, "Quicksilver Gargantuan", forest);
    assert_eq!(t.obj_now(qg).chars.name, "Forest");
    assert!(!t.obj_now(qg).is(CardType::Creature));
    assert_eq!(t.obj_now(qg).chars.power, None);
    assert_eq!(t.obj_now(qg).chars.toughness, None);
    // Becoming a creature later: the animating effect sets its power and toughness.
    animate_land(&mut t, qg);
    assert_eq!(t.pt(qg), (2, 2));
}

#[test]
fn phantom_steed_copying_an_exiled_land_makes_a_tapped_nonattacking_land_token() {
    cr!("707.9b", "205.3d", "506.3");
    ruling!(
        "Phantom Steed",
        "If Phantom Steed somehow exiles a noncreature card, such as an animated land, it will still create a copy of that card when it attacks."
    );
    supported("Phantom Steed");
    let mut t = TestGame::new(2);
    let forest = animated_forest(&mut t, P0);
    t.answer_targets(P0, &[obj(forest)]);
    let steed = t.enter(P0, "Phantom Steed");
    t.resolve_all();
    assert!(t.in_exile("Forest"));
    crate::r_p023_common::unsick(&mut t, steed);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(steed, Entity::Player(P1))]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    let o = t.obj_now(tok);
    assert_eq!(o.chars.name, "Forest");
    assert!(o.is(CardType::Land) && !o.is(CardType::Creature));
    assert!(o.tapped);
    assert!(!has_subtype(&t, tok, "Illusion"));
    let attackers =
        t.g.combat
            .as_ref()
            .map(|c| c.attackers.clone())
            .unwrap_or_default();
    assert!(
        attackers.iter().all(|a| a.id != tok),
        "the land token is attacking"
    );
    // It's sacrificed at end of combat.
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
}

#[test]
fn machine_gods_effigy_copying_an_animated_forest_has_only_its_own_mana_ability() {
    cr!("707.9b", "305.6", "205.3d");
    ruling!(
        "Machine God's Effigy",
        "If Machine God’s Effigy copies a land with a basic land type that has become a creature due to an effect, it will not have any mana abilities that land had due to its land types"
    );
    supported("Machine God's Effigy");
    let mut t = TestGame::new(2);
    let forest = animated_forest(&mut t, P1);
    let effigy = enter_copying(&mut t, P0, "Machine God's Effigy", forest);
    let o = t.obj_now(effigy);
    assert_eq!(o.chars.name, "Forest");
    assert!(o.is(CardType::Artifact) && !o.is(CardType::Land));
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(!has_subtype(&t, effigy, "Forest"));
    assert_eq!(abilities_with(&t, effigy, "Add {G}"), 0);
    assert_eq!(abilities_with(&t, effigy, "Add {"), 1);
    activate_containing(&mut t, P0, effigy, "Add {U}").expect("mana ability");
    assert_eq!(
        t.g.player(P0)
            .mana_pool
            .count(mtg_engine::mana::ManaType::U),
        1
    );
    assert_eq!(
        t.g.player(P0)
            .mana_pool
            .count(mtg_engine::mana::ManaType::G),
        0
    );
}

#[test]
fn machine_gods_effigy_copying_an_animated_saga_isnt_a_saga() {
    cr!("707.9b", "714.3a", "714.2b", "714.4");
    ruling!(
        "Machine God's Effigy",
        "If Machine God’s Effigy copies a Saga that has become a creature due to an effect, it will have that Saga’s chapter abilities, but it won’t get a lore counter every turn"
    );
    supported("Machine God's Effigy");
    supported("History of Benalia");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P1, "History of Benalia");
    modify(
        &mut t,
        saga,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(3)), Some(Value::c(3))),
        ],
    );
    let effigy = enter_copying(&mut t, P0, "Machine God's Effigy", saga);
    assert_eq!(t.obj_now(effigy).chars.name, "History of Benalia");
    assert!(!has_subtype(&t, effigy, "Saga"));
    // No lore counter as it enters, nor after its controller's draw step.
    assert_eq!(t.counters(effigy, counters::LORE), 0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(effigy, counters::LORE), 0);
    assert!(new_tokens(&t, P0, &[]).is_empty());
    // Lore counters put on it some other way trigger its chapter abilities, and it isn't
    // sacrificed after the last one.
    for n in 1..=3u32 {
        t.g.add_counters(obj(effigy), counters::LORE, 1, None);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.counters(effigy, counters::LORE), n);
    }
    let knights =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Knight"))
            .count();
    assert_eq!(knights, 2);
    assert!(t.on_battlefield(effigy));
}

#[test]
fn machine_gods_effigy_copying_an_animated_planeswalker_isnt_a_planeswalker() {
    cr!("707.9b", "306.5b", "606.3", "704.5i");
    ruling!(
        "Machine God's Effigy",
        "If Machine God’s Effigy copies a planeswalker that became a creature due to an effect, Machine God’s Effigy will have that planeswalker’s loyalty abilities but its only card type is artifact"
    );
    supported("Machine God's Effigy");
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    modify(
        &mut t,
        jace,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(3)), Some(Value::c(3))),
        ],
    );
    let effigy = enter_copying(&mut t, P0, "Machine God's Effigy", jace);
    let o = t.obj_now(effigy);
    assert_eq!(o.chars.name, "Jace Beleren");
    assert!(o.is(CardType::Artifact) && !o.is(CardType::Planeswalker));
    assert!(legendary(&t, effigy));
    // No loyalty counters, and it doesn't die for having none.
    assert_eq!(t.counters(effigy, counters::LOYALTY), 0);
    t.settle();
    assert!(t.on_battlefield(effigy));
    // It can't be attacked.
    t.set_step(P1, Step::BeginningOfCombat);
    let options = mtg_engine::combat::attack_options(&t.g);
    assert!(options
        .iter()
        .all(|(_, es)| !es.contains(&Entity::Object(effigy))));
    // Its loyalty abilities: only one per turn.
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, effigy, "+2").expect("loyalty ability");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(effigy, counters::LOYALTY), 2);
    assert!(activate_containing(&mut t, P0, effigy, "+2").is_err());
}
