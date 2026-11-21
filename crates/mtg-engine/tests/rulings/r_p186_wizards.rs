//! Rulings batch P186 — typal Wizard cards: Naban's and Wizard's Staff's extra triggers
//! (CR 603.2d, 603.6a, 614.12), conditional granted abilities and "dies" look-back (CR
//! 603.10a), intervening "if" clauses (CR 603.4), effects locked in on resolution (CR
//! 611.2c), cost increases and reductions (CR 601.2f, 202.3), and Two-Headed Giant life
//! loss (CR 810.8a).

use crate::r_p076_common::mana;
use crate::r_p185_common::*;
use crate::r_p186_common::*;
use crate::r_p189_common::stack_triggers_from;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{attach_new, damage};
use crate::r_s24_common::{choose_creature_type, enter_together};
use crate::r_s25_common::cast_new;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
fn two_headed_giant() -> TestGame {
    crate::r_s33_common::two_headed_giant()
}

#[test]
fn naban_doesnt_affect_as_enters_choices() {
    cr!("603.2d", "614.12");
    ruling!(
        "Naban, Dean of Iteration",
        "Abilities that apply \"as [this creature] enters the battlefield,\" such as choosing a card name with Meddling Mage, are unaffected."
    );
    supported("Naban, Dean of Iteration");
    supported("Meddling Mage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naban, Dean of Iteration");
    t.battlefield(P0, "Soul Warden");
    let before = names_asked(&t);
    crate::r_s05_common::enter(&mut t, P0, "Meddling Mage");
    t.resolve_all();
    assert_eq!(names_asked(&t) - before, 1, "one name is chosen");
    // Positive control: the Wizard entering does double Soul Warden's trigger.
    assert_eq!(t.life(P0), 22);
}

#[test]
fn naban_doubles_triggers_of_wizards_entering_with_it() {
    cr!("603.2d", "603.6a");
    ruling!(
        "Naban, Dean of Iteration",
        "If a Wizard entering the battlefield at the same time as Naban (including Naban itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    ruling!(
        "Naban, Dean of Iteration",
        "Naban affects a Wizard's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that Wizard enters the battlefield."
    );
    ruling!(
        "Naban, Dean of Iteration",
        "The trigger event doesn't have to specifically refer to \"Wizards.\" For example, an ability that triggers \"whenever a creature enters the battlefield under your control\" would trigger twice if the entering creature is a Wizard."
    );
    supported("Ghitu Journeymage");
    supported("Soul Warden");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    enter_together(
        &mut t,
        &[(P0, "Naban, Dean of Iteration"), (P0, "Ghitu Journeymage")],
    );
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    // Soul Warden: twice for Naban, twice for Ghitu; Ghitu's own trigger: twice.
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.life(P1), 16);
    // A non-Wizard entering triggers Soul Warden once.
    t.battlefield(P0, "Naban, Dean of Iteration");
    crate::r_s05_common::enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn two_nabans_trigger_three_times() {
    cr!("603.2d");
    ruling!(
        "Naban, Dean of Iteration",
        "If you somehow control two Nabans, a Wizard entering the battlefield causes abilities to trigger three times, not four."
    );
    let naban = card_text("Naban, Dean of Iteration");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naban, Dean of Iteration");
    // A second, nonlegendary Naban.
    let second = custom_card("Naban Again", "Creature — Human Wizard", "{1}{U}", Some((2, 1)), &naban);
    t.custom(P0, second, mtg_engine::object::Zone::Battlefield);
    crate::r_s05_common::enter(&mut t, P0, "Ghitu Journeymage");
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "Ghitu's trigger three times");
}

/// The oracle text of a real card's front face.
fn card_text(name: &str) -> String {
    mtg_engine::card::card(name).front().chars.rules_text.to_string()
}

#[test]
fn naban_looks_at_entering_permanents_with_continuous_effects() {
    cr!("603.2d", "603.6a", "613.1d");
    ruling!(
        "Naban, Dean of Iteration",
        "Look at each permanent as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger multiple times."
    );
    supported("Arcane Adaptation");
    supported("Runeclaw Bear");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naban, Dean of Iteration");
    t.battlefield(P0, "Soul Warden");
    choose_creature_type(&mut t, P0, "Wizard");
    crate::r_s05_common::enter(&mut t, P0, "Arcane Adaptation");
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    crate::r_s05_common::enter(&mut t, P0, "Runeclaw Bear");
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "the Bear is a Wizard as it enters");
}

#[test]
fn naban_extra_triggers_make_their_choices_separately() {
    cr!("603.2d", "603.3d", "608.2d");
    ruling!(
        "Naban, Dean of Iteration",
        "Naban's effect doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Voidwielder");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naban, Dean of Iteration");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    let from = t.asked().len();
    let v = crate::r_s05_common::enter(&mut t, P0, "Voidwielder");
    assert_eq!(target_asks(&t, P0, from), 2);
    let trig = stack_triggers_from(&t, v);
    assert_eq!(trig.len(), 2);
    assert!(!crate::r_s25_common::spell_copies(&t).contains(&trig[0]));
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    let bounced = [a, b].iter().filter(|o| !t.on_battlefield(**o)).count();
    assert_eq!(bounced, 1, "one yes, one no");
}

#[test]
fn naban_and_wizards_staff_dont_affect_replacement_effects() {
    cr!("603.2d", "614.1c", "615.1");
    ruling!(
        "Naban, Dean of Iteration",
        "Replacement effects are unaffected by Naban's first ability. For example, a Wizard that enters the battlefield with one +1/+1 counter on it won't receive an additional +1/+1 counter."
    );
    ruling!(
        "Wizard's Staff",
        "Replacement effects are unaffected by Wizard's Staff's second ability."
    );
    ruling!(
        "Unbreathing Horde",
        "Only one +1/+1 counter will be removed, no matter how much damage is prevented."
    );
    supported("Star Pupil");
    supported("Wizard's Staff");
    supported("Unbreathing Horde");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naban, Dean of Iteration");
    let pupil = crate::r_s05_common::enter(&mut t, P0, "Star Pupil");
    t.resolve_all();
    assert_eq!(t.counters(pupil, counters::PLUS1), 1);
    // Wizard's Staff on Unbreathing Horde: one counter removed for prevented damage, while
    // the granted prowess does trigger twice.
    let mut t = TestGame::new(2);
    let horde = t.battlefield(P0, "Unbreathing Horde");
    crate::r_s29_common::put_counters(&mut t, horde, counters::PLUS1, 3);
    attach_new(&mut t, P0, "Wizard's Staff", horde);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 2, horde);
    assert_eq!(t.counters(horde, counters::PLUS1), 2);
    assert_eq!(marked(&t, horde), 0);
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.pt(horde), (4, 4), "prowess twice on a 2/2");
}

#[test]
fn wizards_staff_extra_trigger_makes_its_choices_separately() {
    cr!("603.2d", "603.3d", "608.2d");
    ruling!(
        "Wizard's Staff",
        "The second ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kalastria Highborn");
    attach_new(&mut t, P0, "Wizard's Staff", k);
    let v = t.battlefield(P0, "Arrogant Vampire");
    mana(&mut t, P0, ManaType::B, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    destroy(&mut t, v);
    assert_eq!(target_asks(&t, P0, from), 2);
    assert_eq!(stack_triggers_from(&t, k).len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "paid for one instance only");
    assert_eq!(crate::r_s24_common::pool(&t, P0, ManaType::B), 1);
}

#[test]
fn adeliz_pumps_only_the_wizards_there_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Adeliz, the Cinder Wind",
        "Adeliz's last ability affects only Wizards you control at the time it resolves, including Adeliz itself. Creatures you begin to control later in the turn or that become Wizards later in the turn won't get +1/+1."
    );
    supported("Adeliz, the Cinder Wind");
    let mut t = TestGame::new(2);
    let adeliz = t.battlefield(P0, "Adeliz, the Cinder Wind");
    let other = t.battlefield(P0, "Fugitive Wizard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.pt(adeliz), (3, 3));
    assert_eq!(t.pt(other), (2, 2));
    add_types(&mut t, bears, &["Wizard"]);
    assert_eq!(t.pt(bears), (2, 2));
    let late = t.battlefield(P0, "Fugitive Wizard");
    assert_eq!(t.pt(late), (1, 1));
}

#[test]
fn adeliz_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Adeliz, the Cinder Wind",
        "Adeliz's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    let adeliz = t.battlefield(P0, "Adeliz, the Cinder Wind");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(*t.g.stack.first().unwrap(), bolt, "the Bolt is below the trigger");
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(bolt)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "countered");
    assert_eq!(t.pt(adeliz), (3, 3));
}

#[test]
fn expedition_diviner_has_one_trigger_however_many_wizards() {
    cr!("603.2", "603.10a");
    ruling!(
        "Expedition Diviner",
        "Expedition Diviner has just one instance of its triggered ability, no matter how many other Wizards you control beyond the first."
    );
    supported("Expedition Diviner");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Expedition Diviner");
    t.battlefield(P0, "Fugitive Wizard");
    t.battlefield(P0, "Fugitive Wizard");
    let hand = t.hand_size(P0);
    destroy(&mut t, d);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn expedition_diviners_dying_together_both_trigger() {
    cr!("603.10a", "704.3");
    ruling!(
        "Expedition Diviner",
        "If Expedition Diviner dies at the same time that each other Wizard you control leaves the battlefield, its ability triggers. For example, if the only creatures you control are two Expedition Diviners and both die at the same time, their abilities each trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Expedition Diviner");
    t.battlefield(P0, "Expedition Diviner");
    let hand = t.hand_size(P0);
    crate::r_p044_common::wrath(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Alone, a Diviner doesn't have the ability.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Expedition Diviner");
    let hand = t.hand_size(P0);
    destroy(&mut t, d);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn ghitu_journeymage_needs_another_wizard_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Ghitu Journeymage",
        "Ghitu Journeymage’s ability doesn’t trigger if you don’t control another Wizard immediately after it enters the battlefield. If it does trigger but you don’t control another Wizard as it resolves, it does nothing."
    );
    ruling!(
        "Ghitu Journeymage",
        "Ghitu Journeymage’s triggered ability doesn’t deal more damage if you control more than one other Wizard."
    );
    let mut t = TestGame::new(2);
    let g = crate::r_s05_common::enter(&mut t, P0, "Ghitu Journeymage");
    assert_eq!(stack_triggers_from(&t, g).len(), 0);
    // Triggers, then the other Wizard leaves.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Fugitive Wizard");
    let g = crate::r_s05_common::enter(&mut t, P0, "Ghitu Journeymage");
    assert_eq!(stack_triggers_from(&t, g).len(), 1);
    destroy(&mut t, w);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Two other Wizards: still 2 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fugitive Wizard");
    t.battlefield(P0, "Fugitive Wizard");
    crate::r_s05_common::enter(&mut t, P0, "Ghitu Journeymage");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn ghitu_journeymage_two_headed_giant_team_loses_four() {
    cr!("810.8a", "810.9");
    ruling!(
        "Ghitu Journeymage",
        "In a Two-Headed Giant game, Ghitu Journeymage’s ability causes the opposing team to lose 4 life."
    );
    let mut t = two_headed_giant();
    let team = t.life(P2);
    t.battlefield(P0, "Fugitive Wizard");
    crate::r_s05_common::enter(&mut t, P0, "Ghitu Journeymage");
    t.resolve_all();
    assert_eq!(t.life(P2), team - 4);
    assert_eq!(t.life(P3), team - 4);
}

#[test]
fn kaza_reduction_applies_after_increases_and_mana_value_is_unchanged() {
    cr!("601.2f", "202.3");
    ruling!(
        "Kaza, Roil Chaser",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Kaza's activated ability). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Kaza, Roil Chaser");
    supported("Thalia, Guardian of Thraben");
    let mut t = TestGame::new(2);
    let kaza = t.battlefield(P0, "Kaza, Roil Chaser");
    t.battlefield(P0, "Fugitive Wizard");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.activate(P0, kaza, 0, &[]).unwrap();
    t.resolve_all();
    // Divination {2}{U}, +{1} from Thalia, -{2} from Kaza: {1}{U}.
    let div = t.hand(P0, "Divination");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, div).go();
    assert_eq!(crate::r_s24_common::pool(&t, P0, ManaType::U), 0);
    assert_eq!(crate::r_s26_common::mv(&mut t, spell), 3);
}
