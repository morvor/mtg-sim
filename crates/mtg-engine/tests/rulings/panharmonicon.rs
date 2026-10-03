//! Rulings on Panharmonicon and Yarok, the Desecrated: "If an artifact or creature (a
//! permanent) entering causes a triggered ability of a permanent you control to trigger,
//! that ability triggers an additional time." (CR 603.2d, 603.6a).

use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn panharmonicon_doubles_a_permanents_own_and_other_enters_triggers() {
    cr!("603.2d", "603.6a");
    ruling!(
        "Panharmonicon",
        "Panharmonicon affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield."
    );
    supported("Panharmonicon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Panharmonicon");
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    let (hand, life) = (t.hand_size(P0), t.life(P0));
    // Elvish Visionary: "When this creature enters, draw a card."
    t.enter(P0, "Elvish Visionary");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), life + 2);
}

#[test]
fn panharmonicon_applies_to_permanents_entering_under_an_opponents_control() {
    cr!("603.2d");
    ruling!(
        "Panharmonicon",
        "You don't need to control the permanent entering the battlefield, only the permanent that has the triggered ability."
    );
    supported("Panharmonicon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Panharmonicon");
    t.battlefield(P0, "Soul Warden");
    // P1's Soul Warden isn't affected: P1 doesn't control Panharmonicon.
    t.battlefield(P1, "Soul Warden");
    let (life0, life1) = (t.life(P0), t.life(P1));
    t.enter(P1, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), life0 + 2);
    assert_eq!(t.life(P1), life1 + 1);
}

#[test]
fn two_panharmonicons_make_abilities_trigger_three_times() {
    cr!("603.2d");
    ruling!(
        "Panharmonicon",
        "If you control two Panharmonicons, an artifact or creature entering the battlefield causes abilities to trigger three times, not four."
    );
    supported("Panharmonicon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Panharmonicon");
    t.battlefield(P0, "Panharmonicon");
    t.battlefield(P0, "Soul Warden");
    let life = t.life(P0);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), life + 3);
}

#[test]
fn panharmonicon_applies_to_itself_entering() {
    cr!("603.2d", "603.6a");
    ruling!(
        "Panharmonicon",
        "If an artifact or creature entering the battlefield at the same time as Panharmonicon (including Panharmonicon itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    supported("Panharmonicon");
    supported("Reckless Fireweaver");
    let mut t = TestGame::new(2);
    // Reckless Fireweaver: "Whenever an artifact you control enters, this creature deals
    // 1 damage to each opponent."
    t.battlefield(P0, "Reckless Fireweaver");
    t.enter(P0, "Panharmonicon");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn panharmonicon_doesnt_affect_as_enters_abilities_or_enchantments_entering() {
    cr!("603.2d", "614.12");
    ruling!(
        "Panharmonicon",
        "Abilities that apply \"as [this artifact or creature] enters the battlefield,\" such as choosing a color with Gauntlet of Power, are also unaffected."
    );
    supported("Panharmonicon");
    supported("Meddling Mage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Panharmonicon");
    // Meddling Mage: "As this creature enters, choose a nonland card name." One choice.
    let from = t.asked().len();
    t.enter(P0, "Meddling Mage");
    t.settle();
    let names = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::NameCard { .. }))
        .count();
    assert_eq!(names, 1);
    // An enchantment entering isn't an artifact or creature entering: Entity Tracker
    // ("Whenever an enchantment you control enters ..., draw a card.") draws one card.
    t.battlefield(P0, "Entity Tracker");
    let hand = t.hand_size(P0);
    t.enter(P0, "Glorious Anthem");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn yarok_doubles_abilities_triggered_by_any_permanent_entering() {
    cr!("603.2d", "603.6a");
    ruling!(
        "Yarok, the Desecrated",
        "Yarok affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield."
    );
    supported("Yarok, the Desecrated");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yarok, the Desecrated");
    // An enchantment entering: Entity Tracker draws two cards.
    t.battlefield(P0, "Entity Tracker");
    let hand = t.hand_size(P0);
    t.enter(P0, "Glorious Anthem");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // A creature's own enters ability.
    let hand = t.hand_size(P0);
    t.enter(P0, "Elvish Visionary");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}
