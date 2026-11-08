//! Rulings batch P077 — "creatures you control get ... and gain vigilance until end of
//! turn": the affected set is locked in as the ability resolves (CR 611.2c), and values
//! are counted then (CR 608.2h).

use crate::r_p077_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// After the pump (`p`/`t` and vigilance) resolved, a creature that comes under P0's
/// control and a land that becomes a creature are both unaffected.
fn later_creatures_unaffected(t: &mut TestGame) {
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    assert!(!vig(t, late));
    let vault = animate_mutavault(t, P0);
    assert_eq!(t.pt(vault), (2, 2));
    assert!(!vig(t, vault));
}

#[test]
fn enters_pumps_lock_in_creatures_as_they_resolve() {
    cr!("611.2c", "603.6a");
    ruling!(
        "Stagecoach Security",
        "Stagecoach Security’s triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won’t get +1/+1 or vigilance."
    );
    ruling!(
        "Dawnfeather Eagle",
        "The set of creatures affected by Dawnfeather Eagle’s triggered ability is determined as the ability resolves. Creatures you begin to control later in the turn and permanents you control that become creatures later in the turn won’t get +1/+1 or gain vigilance."
    );
    ruling!(
        "Trial of Solidarity",
        "Trial of Solidarity’s triggered ability affects only creatures you control at the time it resolves. It won’t affect creatures that come under your control later in the turn."
    );
    // (card, bonus the bears get)
    let cases = [
        ("Stagecoach Security", (3, 3)),
        ("Dawnfeather Eagle", (3, 3)),
        ("Trial of Solidarity", (4, 3)),
    ];
    for (name, bears_pt) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.enter(P0, name);
        t.resolve_all();
        assert_eq!(t.pt(bears), bears_pt, "{name}");
        assert!(vig(&t, bears), "{name}");
        later_creatures_unaffected(&mut t);
    }
}

#[test]
fn kamahl_combat_pump_locks_in_creatures_as_it_resolves() {
    cr!("611.2c", "507.1");
    ruling!(
        "Kamahl, Heart of Krosa",
        "Only creatures you control at the time Kamahl's triggered ability resolves will get +3/+3 and gain trample. Creatures that come under your control later in the turn and noncreature permanents that become creatures later in the turn won't get the bonuses."
    );
    supported("Kamahl, Heart of Krosa");
    let mut t = TestGame::new(2);
    let kamahl = t.battlefield(P0, "Kamahl, Heart of Krosa");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Forest", 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    assert!(!has_kw(&t, late, KeywordKind::Trample));
    // Kamahl's own land animation, later in the turn.
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_containing(&mut t, P0, kamahl, "target land").expect("activates");
    t.resolve_all();
    assert!(crate::r_s04_common::is_creature(&t, forest));
    assert_eq!(t.pt(forest), (1, 1));
    assert!(!has_kw(&t, forest, KeywordKind::Trample));
}

#[test]
fn three_blind_mice_chapter_four_locks_in_creatures() {
    cr!("611.2c", "714.2b");
    ruling!(
        "Three Blind Mice",
        "Only creatures you control at the time the fourth chapter ability resolves will get +1/+1 and gain vigilance. Creatures you begin to control later in the turn won't be affected."
    );
    supported("Three Blind Mice");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Three Blind Mice");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(vig(&t, bears));
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    assert!(!vig(&t, late));
}

#[test]
fn angelic_skirmisher_grants_to_creatures_present_and_chooses_before_attacks() {
    cr!("611.2c", "608.2d", "507.1", "702.20b");
    ruling!(
        "Angelic Skirmisher",
        "Only creatures you control when the ability resolves will gain the chosen ability. Creatures that come under your control later in the turn will not gain the ability."
    );
    ruling!(
        "Angelic Skirmisher",
        "You choose which ability creatures you control will gain when Angelic Skirmisher’s ability resolves. This happens before attacking creatures are declared."
    );
    supported("Angelic Skirmisher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Skirmisher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    // Vigilance (the second choice).
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    // The choice was made as the ability resolved, in the beginning of combat step.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::ChooseOption { .. })));
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert!(vig(&t, bears));
    let late = t.battlefield(P0, "Hill Giant");
    assert!(!vig(&t, late));
    // Bears attack with the vigilance chosen before attackers were declared.
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!crate::r_s08_common::is_tapped(&t, bears));
}

#[test]
fn elspeths_talent_locks_in_creatures_and_resolves_before_the_loyalty_ability() {
    cr!("611.2c", "606.3", "603.3", "405.5");
    ruling!(
        "Elspeth's Talent",
        "The set of creatures affected by Elspeth's Talent's last ability is determined as the ability resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won't get +2/+2 and gain vigilance."
    );
    ruling!(
        "Elspeth's Talent",
        "The last ability will resolve before the loyalty ability that caused it to trigger. Notably, if that loyalty ability creates creature tokens, those creatures won't be on the battlefield yet to get +2/+2 and gain vigilance."
    );
    supported("Elspeth's Talent");
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P0, "Ajani Goldmane");
    crate::r_s06_common::attach_new(&mut t, P0, "Elspeth's Talent", pw);
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, pw, "Soldier").expect("the granted loyalty ability");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(vig(&t, bears));
    let soldiers = crate::r_s01_common::with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 3);
    for s in soldiers {
        assert_eq!(t.pt(s), (1, 1), "the Soldier tokens weren't there yet");
        assert!(!vig(&t, s));
    }
    later_creatures_unaffected(&mut t);
}

#[test]
fn elspeths_talent_granted_ability_shares_the_one_loyalty_activation() {
    cr!("606.3", "613.1f");
    ruling!(
        "Elspeth's Talent",
        "You can activate one loyalty ability of each planeswalker you control during each of your turns, whether that ability is one it always has or the one it gains because of Elspeth's Talent."
    );
    supported("Elspeth's Talent");
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P0, "Ajani Goldmane");
    crate::r_s06_common::attach_new(&mut t, P0, "Elspeth's Talent", pw);
    activate_containing(&mut t, P0, pw, "Soldier").expect("the granted loyalty ability");
    t.resolve_all();
    assert!(activate_containing(&mut t, P0, pw, "gain 2 life").is_err());
    // And the other way round, on a new turn.
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P0, "Ajani Goldmane");
    crate::r_s06_common::attach_new(&mut t, P0, "Elspeth's Talent", pw);
    activate_containing(&mut t, P0, pw, "gain 2 life").expect("its own loyalty ability");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(activate_containing(&mut t, P0, pw, "Soldier").is_err());
}

#[test]
fn treeguard_duo_counts_creatures_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Treeguard Duo",
        "The value of X is determined only as Treeguard Duo’s triggered ability resolves, potentially including Treeguard Duo itself. It won’t change later in the turn if the number of creatures you control changes."
    );
    supported("Treeguard Duo");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Treeguard Duo");
    t.settle();
    // A creature that enters with the trigger on the stack counts.
    t.battlefield(P0, "Hill Giant");
    t.resolve_all();
    // Bears, Hill Giant and Treeguard Duo itself: X = 3.
    assert_eq!(t.pt(bears), (5, 5));
    assert!(vig(&t, bears));
    t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn murmuration_counts_spells_as_its_ability_resolves() {
    cr!("608.2h", "513.1a");
    ruling!(
        "Murmuration",
        "The number of spells you've cast is counted only as Murmuration's last ability resolves. For example, say you've cast two spells this turn, and with Murmuration's last ability on the stack, you cast a Blizzard Strix"
    );
    supported("Murmuration");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Murmuration");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_new(&mut t, P0, "Blizzard Strix", &[]);
    t.resolve_all();
    let crows = t.named_on_battlefield("Storm Crow");
    assert_eq!(crows.len(), 3);
    // Storm Crows are Birds: 2/3 with Murmuration's bonus.
    assert_eq!(t.pt(crows[0]), (2, 3));
    assert!(vig(&t, crows[0]));
}
