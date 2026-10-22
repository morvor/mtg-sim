//! More "as this enters" choices and "enters with" amounts: restricted card names
//! (CR 201.4), card types other than a given one, looking at an opponent's hand,
//! "If this land would enter, instead ..." (CR 614.1c), characteristics chosen as it
//! enters that add a creature type (CR 707.2), and characteristic-defining abilities
//! measured for the chosen player (CR 604.3).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn name_card(t: &mut TestGame, p: PlayerId, name: &str) {
    t.answer(p, DecisionKind::Name, Answer::Text(name.to_string()));
}

#[test]
fn counters_equal_to_greatest_power_among_two_groups() {
    cr!("614.1c", "122.6");
    assert_supported("Ambitious Dragonborn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.graveyard(P0, "Craw Wurm");
    t.battlefield(P1, "Craw Wurm");
    let d = t.enter(P0, "Ambitious Dragonborn");
    assert_eq!(t.counters(d, "+1/+1"), 6);

    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    let d = t.enter(P0, "Ambitious Dragonborn");
    assert_eq!(t.counters(d, "+1/+1"), 3);
}

#[test]
fn counters_equal_to_cards_in_all_hands() {
    cr!("614.1c", "402.1");
    assert_supported("Realm Seekers");
    let mut t = TestGame::new(2);
    let n = t.hand_size(P0) + t.hand_size(P1);
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let s = t.enter(P0, "Realm Seekers");
    assert_eq!(t.counters(s, "+1/+1") as usize, n + 3);
}

#[test]
fn land_sacrifices_others_with_its_name_as_it_enters() {
    cr!("614.1c", "614.12");
    assert_supported("Sheltered Valley");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let old = t.battlefield(P0, "Sheltered Valley");
    let theirs = t.battlefield(P1, "Sheltered Valley");
    let new = t.hand(P0, "Sheltered Valley");
    t.play_land(P0, new).unwrap();
    assert!(!t.on_battlefield(old));
    assert!(t.in_graveyard(P0, "Sheltered Valley"));
    assert!(t.on_battlefield(new));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn becomes_a_wall_in_addition_to_its_other_types() {
    cr!("614.1c", "707.2", "205.1b");
    assert_supported("Primal Clay");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let c = t.enter(P0, "Primal Clay");
    t.settle();
    assert_eq!(t.pt(c), (1, 6));
    let o = t.obj_now(c);
    assert!(o.has_keyword(KeywordKind::Defender));
    assert!(o.chars.has_subtype("Wall"));
    assert!(o.chars.has_subtype("Shapeshifter"));
    assert!(o.chars.is(types::CardType::Artifact));
}

#[test]
fn restricted_card_name_choice() {
    cr!("201.4", "607.2d", "601.2");
    assert_supported("Council of the Absolute");
    // A noncreature, nonland name.
    let mut t = TestGame::new(2);
    name_card(&mut t, P0, "Lightning Bolt");
    t.enter(P0, "Council of the Absolute");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());

    // A creature card's name isn't a legal choice: nothing is named (CR 607.5a).
    let mut t = TestGame::new(2);
    name_card(&mut t, P0, "Grizzly Bears");
    let c = t.enter(P0, "Council of the Absolute");
    assert_eq!(t.obj_now(c).choices.card_name.as_deref(), Some(""));
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    assert!(t.cast(P1, bears).try_go().is_ok());
}

#[test]
fn choose_a_card_type_other_than_creature() {
    cr!("607.2d", "601.2f");
    assert_supported("Arachne, Psionic Weaver");
    let mut t = TestGame::new(2);
    // artifact, battle, enchantment, instant, kindred, land, planeswalker, sorcery
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let a = t.enter(P0, "Arachne, Psionic Weaver");
    assert_eq!(
        t.obj_now(a).choices.card_type,
        Some(types::CardType::Instant)
    );
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    t.lands(P1, "Mountain", 1);
    assert!(t.cast(P1, bolt).target(P0).try_go().is_ok());
}

#[test]
fn look_at_an_opponents_hand_then_name_a_card() {
    cr!("614.12a", "607.2d", "602.5");
    assert_supported("Sorcerous Spyglass");
    let mut t = TestGame::new(2);
    let sorc = t.battlefield(P1, "Prodigal Sorcerer");
    name_card(&mut t, P0, "Prodigal Sorcerer");
    let s = t.enter(P0, "Sorcerous Spyglass");
    assert_eq!(t.obj_now(s).choices.player, Some(P1));
    assert!(t.activate(P1, sorc, 0, &[Entity::Player(P0)]).is_err());
}

#[test]
fn power_from_cards_in_the_chosen_players_graveyard() {
    cr!("604.3", "607.2d", "613.4a");
    assert_supported("Haunting Apparition");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Llanowar Elves");
    t.graveyard(P1, "Raging Goblin");
    t.graveyard(P0, "Grizzly Bears");
    let h = t.enter(P0, "Haunting Apparition");
    assert_eq!(t.pt(h), (3, 2));
}

#[test]
fn power_and_toughness_from_nonbasic_lands_the_chosen_player_controls() {
    cr!("604.3", "607.2d", "613.4a");
    assert_supported("Skyshroud War Beast");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stomping Ground");
    t.battlefield(P1, "Reliquary Tower");
    t.battlefield(P1, "Forest");
    t.battlefield(P0, "Reliquary Tower");
    let b = t.enter(P0, "Skyshroud War Beast");
    assert_eq!(t.pt(b), (2, 2));
}

// ---------------------------------------------------------------------------
// "Spend this mana only to cast a creature spell of the chosen type."
// ---------------------------------------------------------------------------

fn choose_creature_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == ty)
        .expect("creature type");
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn mana_spendable_only_on_creature_spells_of_the_chosen_type() {
    cr!("106.6", "607.2d");
    assert_supported("Unclaimed Territory");
    assert_supported("Pillar of Origins");
    // Colors: W, U, B, R, G.
    let with_mana = |color: usize| {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        choose_creature_type(&mut t, P0, "Elf");
        let land = t.enter(P0, "Unclaimed Territory");
        let now = t.g.current(land);
        t.answer(P0, DecisionKind::Any, Answer::Index(color));
        t.activate(P0, now, 1, &[]).unwrap();
        t
    };
    let mut t = with_mana(4);
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(t.cast(P0, elves).try_go().is_ok());
    // Mana of the right color, but not for a creature spell of the chosen type.
    let mut t = with_mana(4);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    assert!(t.cast(P0, bears).try_go().is_err());
    let mut t = with_mana(3);
    let goblin = t.hand(P0, "Raging Goblin");
    assert!(t.cast(P0, goblin).try_go().is_err());
}

#[test]
fn chosen_type_anthem_for_each_charge_counter() {
    cr!("607.2d", "613.4c");
    assert_supported("Door of Destinies");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    choose_creature_type(&mut t, P0, "Elf");
    let door = t.enter(P0, "Door of Destinies");
    assert_eq!(t.pt(elf), (1, 1));
    // Casting an Elf spell puts a charge counter on it.
    t.lands(P0, "Forest", 1);
    let e2 = t.hand(P0, "Llanowar Elves");
    t.cast(P0, e2).go();
    t.resolve_all();
    assert_eq!(t.counters(door, "charge"), 1);
    assert_eq!(t.pt(elf), (2, 2));
    assert_eq!(t.pt(e2), (2, 2));
    let now = t.g.current(door);
    t.g.add_counters(Entity::Object(now), "charge", 2, None);
    t.settle();
    assert_eq!(t.pt(elf), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn gets_smaller_for_each_card_in_the_chosen_players_hand() {
    cr!("607.2d", "613.4c");
    assert_supported("Nyxathid");
    let mut t = TestGame::new(2);
    let n = t.hand_size(P1) as i32;
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let x = t.enter(P0, "Nyxathid");
    assert_eq!(t.pt(x), (7 - n - 2, 7 - n - 2));
}

#[test]
fn restricted_mana_from_the_chosen_type_ability() {
    cr!("106.6", "607.2d", "607.5a");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Goblin");
    let pillar = t.enter(P0, "Pillar of Origins");
    let now = t.g.current(pillar);
    // Choose red for "any color".
    t.answer(P0, DecisionKind::Any, Answer::Index(3));
    t.activate(P0, now, 0, &[]).unwrap();
    let pool = &t.g.players[0].mana_pool;
    assert_eq!(pool.total(), 1);
    let m = &pool.mana[0];
    assert_eq!(
        m.restriction,
        Some(mana::ManaRestriction::SpellWithSubtype("Goblin".into()))
    );
}
