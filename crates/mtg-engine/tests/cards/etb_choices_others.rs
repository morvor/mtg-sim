//! Replacement effects that modify how other permanents enter (CR 614.1d, 614.12), and
//! "it becomes day as this enters" (CR 731).

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

#[test]
fn opponents_creatures_and_nonbasic_lands_enter_tapped() {
    cr!("614.1d", "614.12");
    assert_supported("Thalia, Heretic Cathar");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thalia, Heretic Cathar");
    let their_bears = t.enter(P1, "Grizzly Bears");
    let their_forest = t.enter(P1, "Forest");
    let their_nonbasic = t.enter(P1, "Reliquary Tower");
    let my_bears = t.enter(P0, "Grizzly Bears");
    assert!(t.obj_now(their_bears).tapped);
    assert!(!t.obj_now(their_forest).tapped, "basic land");
    assert!(t.obj_now(their_nonbasic).tapped, "nonbasic land");
    assert!(!t.obj_now(my_bears).tapped, "your own creature");
}

#[test]
fn opponents_creature_spell_enters_tapped() {
    cr!("614.1d", "608.3a");
    assert_supported("Imposing Sovereign");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imposing Sovereign");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn entering_controller_is_the_one_it_will_have() {
    cr!("614.12");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imposing Sovereign");
    // A card P1 owns put onto the battlefield under P0's control: not an opponent's.
    let id =
        t.g.create_card_object(card("Grizzly Bears"), P1, object::Zone::Nowhere);
    let new =
        t.g.move_object_ev(replacement::MoveEv {
            obj: id,
            to: object::Zone::Battlefield,
            pos: ability::LibraryPosition::Top,
            cause: events::MoveCause::Effect,
            by: Some(P0),
            etb: replacement::EtbInfo {
                controller: Some(P0),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    assert!(!t.g.obj(new).tapped);
}

#[test]
fn general_enter_tapped_effect_doesnt_affect_itself() {
    cr!("614.12");
    assert_supported("Orb of Dreams");
    let mut t = TestGame::new(2);
    let orb = t.enter(P0, "Orb of Dreams");
    assert!(!t.obj_now(orb).tapped);
    let bears = t.enter(P0, "Grizzly Bears");
    assert!(t.obj_now(bears).tapped);
}

// ---------------------------------------------------------------------------
// Day and night
// ---------------------------------------------------------------------------

#[test]
fn it_becomes_day_as_it_enters() {
    cr!("731.1", "731.1a", "614.1c");
    assert_supported("Firmament Sage");
    let mut t = TestGame::new(2);
    assert_eq!(t.g.day, None);
    let hand = t.hand_size(P0);
    t.enter(P0, "Firmament Sage");
    t.resolve_all();
    assert_eq!(t.g.day, Some(true));
    // Becoming day from neither isn't "night becomes day": no card drawn.
    assert_eq!(t.hand_size(P0), hand);
    // Night becomes day triggers it.
    t.g.set_day(false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn it_doesnt_become_day_if_it_is_night() {
    cr!("731.1");
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    t.enter(P0, "Firmament Sage");
    assert_eq!(t.g.day, Some(false));
}

// ---------------------------------------------------------------------------
// "You may have ~ enter as a copy of ..." (CR 707.9)
// ---------------------------------------------------------------------------

#[test]
fn clone_enters_as_a_copy() {
    cr!("707.9", "614.1c", "706.2");
    assert_supported("Clone");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let c = t.enter(P0, "Clone");
    let now = t.g.current(c);
    assert_eq!(t.g.obj(now).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.pt(c), (2, 2));
    assert_eq!(t.g.obj(now).controller, P0);
}

#[test]
fn clone_may_copy_nothing() {
    cr!("707.9", "704.5f");
    ruling!("Clone", "You can choose not to copy anything");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[]);
    let c = t.enter(P0, "Clone");
    t.settle();
    assert!(!t.on_battlefield(c));
    assert!(t.in_graveyard(P0, "Clone"));
}

#[test]
fn clone_uses_the_copied_creatures_as_enters_abilities() {
    cr!("707.9", "614.12", "607.2d");
    ruling!(
        "Clone",
        "Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the chosen creature will also work"
    );
    let mut t = TestGame::new(2);
    let voice = t.battlefield(P1, "Voice of All");
    t.answer_choose(P0, &[Entity::Object(voice)]);
    // Blue is index 1.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let c = t.enter(P0, "Clone");
    let now = t.g.current(c);
    assert_eq!(t.g.obj(now).chars.name.as_str(), "Voice of All");
    assert_eq!(t.g.obj(now).choices.color, Some(types::Color::Blue));
    let blue = t.battlefield(P1, "Coral Merfolk");
    assert!(t.g.protected_from(now, blue));
}

#[test]
fn vesuva_enters_tapped_as_a_copy_of_a_land() {
    cr!("707.9", "614.1c");
    assert_supported("Vesuva");
    let mut t = TestGame::new(2);
    let ground = t.battlefield(P1, "Stomping Ground");
    t.answer_choose(P0, &[Entity::Object(ground)]);
    // Stomping Ground's "you may pay 2 life" applies too; it enters tapped either way.
    t.answer_yes(P0, true);
    let v = t.hand(P0, "Vesuva");
    t.play_land(P0, v).unwrap();
    let now = t.g.current(v);
    assert_eq!(t.g.obj(now).chars.name.as_str(), "Stomping Ground");
    assert!(t.obj_now(v).tapped);
    assert_eq!(t.life(P0), 18);
}

// ---------------------------------------------------------------------------
// Adamant; "if you do or if ..."
// ---------------------------------------------------------------------------

#[test]
fn adamant_counter_needs_three_mana_of_the_color() {
    cr!("614.1c", "207.2c", "601.2h");
    assert_supported("Ardenvale Paladin");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let p = t.hand(P0, "Ardenvale Paladin");
    t.cast(P0, p).go();
    t.resolve();
    assert_eq!(t.counters(p, "+1/+1"), 1);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    let p = t.hand(P0, "Ardenvale Paladin");
    t.cast(P0, p).go();
    t.resolve();
    assert_eq!(t.counters(p, "+1/+1"), 0);
}

#[test]
fn adamant_same_color() {
    cr!("614.1c", "207.2c");
    assert_supported("Henge Walker");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let h = t.hand(P0, "Henge Walker");
    t.cast(P0, h).go();
    t.resolve();
    assert_eq!(t.counters(h, "+1/+1"), 1);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 1);
    let h = t.hand(P0, "Henge Walker");
    t.cast(P0, h).go();
    t.resolve();
    assert_eq!(t.counters(h, "+1/+1"), 0);
}

#[test]
fn reveal_or_control_a_dragon_for_a_counter() {
    cr!("614.1c", "614.12a");
    assert_supported("Dragon's Disciple");
    // Reveal a Dragon card from hand.
    let mut t = TestGame::new(2);
    let dragon = t.hand(P0, "Shivan Dragon");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    let d = t.enter(P0, "Dragon's Disciple");
    assert_eq!(t.counters(d, "+1/+1"), 1);
    // Control a Dragon (and don't reveal).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shivan Dragon");
    let d = t.enter(P0, "Dragon's Disciple");
    assert_eq!(t.counters(d, "+1/+1"), 1);
    // Neither.
    let mut t = TestGame::new(2);
    let d = t.enter(P0, "Dragon's Disciple");
    assert_eq!(t.counters(d, "+1/+1"), 0);
}

// ---------------------------------------------------------------------------
// "Each other [X] you control enters with an additional counter" (CR 614.1d)
// ---------------------------------------------------------------------------

#[test]
fn metallic_mimic_adds_counters_to_the_chosen_type() {
    cr!("614.1d", "614.12", "607.2d", "122.6");
    assert_supported("Metallic Mimic");
    let mut t = TestGame::new(2);
    let i = types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Bear")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let mimic = t.enter(P0, "Metallic Mimic");
    // Not itself ("each other").
    assert_eq!(t.counters(mimic, "+1/+1"), 0);
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    let elves = t.enter(P0, "Llanowar Elves");
    assert_eq!(t.counters(elves, "+1/+1"), 0);
    // Not an opponent's Bear.
    let theirs = t.enter(P1, "Grizzly Bears");
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn additional_counters_for_non_humans() {
    cr!("614.1d", "122.6");
    assert_supported("Grumgully, the Generous");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grumgully, the Generous");
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    let human = t.enter(P0, "Elite Vanguard");
    assert_eq!(t.counters(human, "+1/+1"), 0);
}

#[test]
fn planeswalkers_enter_with_an_additional_loyalty_counter() {
    cr!("614.1d", "306.5b");
    assert_supported("Oath of Gideon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Gideon");
    let jace = t.enter(P0, "Jace Beleren");
    assert_eq!(t.counters(jace, "loyalty"), 4);
}
