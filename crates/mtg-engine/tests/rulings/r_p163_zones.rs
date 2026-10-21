//! Rulings batch P163 — sweepers and zone changes: sacrificing isn't destroying and
//! happens simultaneously (CR 701.21a, 603.10a); colors of lands and tokens (CR 105.2c,
//! 202.2, 111.4); "destroyed this way" (CR 701.8a, 614.6); Sagas and control-changing
//! chapters (CR 714.4, 611.2b, 603.10a); delayed triggers that reveal until (CR 603.7);
//! "if you cast it" triggers (CR 603.4, 707.9); Auras on animated permanents (CR
//! 303.4a, 704.5m); spells with two targets (CR 601.2c, 608.2b).

use crate::r_p163_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn all_is_dust_sacrifices_colored_permanents_including_indestructible_ones() {
    cr!("701.21a", "105.2c", "202.2", "111.4", "702.12b");
    ruling!(
        "All Is Dust",
        "All Is Dust doesn't destroy permanents. Rather, it causes them to be sacrificed. Regeneration, umbra armor, and indestructible can't save permanents from All Is Dust."
    );
    ruling!(
        "All Is Dust",
        "A land card that produces colored mana, even a land with a basic land type, normally has no color."
    );
    ruling!(
        "All Is Dust",
        "Lands have no mana cost, so they are colorless unless an effect states otherwise."
    );
    ruling!(
        "All Is Dust",
        "Tokens may have one or more colors. The effect that creates a token states what color it is or whether it's colorless."
    );
    supported("All Is Dust");
    supported("Heliod, God of the Sun");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Heliod, God of the Sun");
    let troll = t.battlefield(P1, "Troll Ascetic");
    let forest = t.battlefield(P1, "Forest");
    t.battlefield(P1, "Dryad Arbor");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Raise the Alarm", &[]);
    let treasure = crate::r_s02_common::create_token(&mut t, P1, "Treasure");
    t.set_step(P0, Step::PrecombatMain);
    // A regeneration shield on the Troll.
    t.lands(P1, "Forest", 1);
    activate_resolve(&mut t, P1, troll, 0, &[]);
    cast_resolve(&mut t, P0, "All Is Dust", &[]);
    assert!(t.in_graveyard(P1, "Heliod, God of the Sun"));
    assert!(t.in_graveyard(P1, "Troll Ascetic"));
    // Dryad Arbor is green (its color indicator); a Forest is colorless.
    assert!(t.in_graveyard(P1, "Dryad Arbor"));
    assert!(t.on_battlefield(forest));
    assert!(t.on_battlefield(thopter));
    // The white Soldier tokens are gone; the colorless Treasure stays.
    assert_eq!(tokens(&t, P1), vec![treasure]);
}

#[test]
fn all_is_dust_sacrifices_everything_at_once() {
    cr!("701.21a", "603.10a", "603.2");
    ruling!(
        "All Is Dust",
        "All permanents with one or more colors are sacrificed at the same time."
    );
    supported("Zulaport Cutthroat");
    // Two Zulaport Cutthroats and a Bears: each Cutthroat sees all three die.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zulaport Cutthroat");
    t.battlefield(P0, "Zulaport Cutthroat");
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "All Is Dust", &[]);
    t.resolve();
    assert_eq!(stacked_triggers(&t), 6);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn ori_counts_what_was_destroyed_wherever_it_went() {
    cr!("701.8a", "614.6", "702.12b");
    ruling!(
        "Ori, Plate Stacker",
        "Artifacts and enchantments destroyed this way count toward the life gained even if they're put into a zone other than a graveyard."
    );
    ruling!(
        "Ori, Plate Stacker",
        "You won't gain life for an artifact or enchantment that isn't actually destroyed (for example, if it has indestructible)."
    );
    supported("Ori, Plate Stacker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Glorious Anthem");
    let relic = t.battlefield(P1, "Darksteel Relic");
    enter(&mut t, P0, "Ori, Plate Stacker");
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    assert!(t.in_exile("Glorious Anthem"));
    assert!(t.on_battlefield(relic));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn elspeth_tirel_ultimate_destroys_your_own_nontoken_nonland_permanents() {
    cr!("701.8a", "608.2h");
    ruling!(
        "Elspeth Tirel",
        "As Elspeth Tirel's last ability resolves, each permanent that isn't a token, a land, or Elspeth Tirel herself is destroyed."
    );
    supported("Elspeth Tirel");
    let mut t = TestGame::new(2);
    let el = t.battlefield(P0, "Elspeth Tirel");
    add_loyalty(&mut t, el, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let forest = t.battlefield(P0, "Forest");
    let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    activate_resolve(&mut t, P0, el, 2, &[]);
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(anthem));
    assert!(t.on_battlefield(forest));
    assert!(t.on_battlefield(token));
    assert!(t.on_battlefield(el));
}

/// The Akroan War enters for P0 and its chapter I takes `target`; resolves the stack.
fn akroan_war(t: &mut TestGame, target: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[obj(target)]);
    let saga = enter(t, P0, "The Akroan War");
    t.resolve_all();
    assert_eq!(t.g.obj(target).controller, P0);
    saga
}

/// Adds a lore counter to the Saga (its next chapter triggers) and resolves the stack up
/// to (not including) the chapter's resolution: the chapter ability is on top.
fn next_chapter(t: &mut TestGame, saga: ObjectId) {
    put_counters(t, saga, counters::LORE, 1);
    assert_eq!(stacked_triggers(t), 1);
}

#[test]
fn akroan_war_you_control_the_creature_while_the_last_chapter_resolves() {
    cr!("714.4", "611.2b", "608.2h");
    ruling!(
        "The Akroan War",
        "Because a Saga isn't sacrificed until after its last ability has resolved, you still control the creature you took with The Akroan War's first ability while the last ability is resolving."
    );
    supported("The Akroan War");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let tapped = t.battlefield(P1, "Hill Giant");
    let saga = akroan_war(&mut t, wurm);
    next_chapter(&mut t, saga);
    t.resolve_all();
    tap(&mut t, tapped);
    next_chapter(&mut t, saga);
    t.g.resolve_top();
    // Chapter III resolved: the tapped Giant dealt damage to itself, the Saga is still
    // there and P0 still controls the Wurm.
    assert_eq!(marked(&t, tapped), 3);
    assert!(t.on_battlefield(saga));
    assert_eq!(t.g.obj(wurm).controller, P0);
    t.settle();
    assert!(!t.on_battlefield(saga));
    assert_eq!(t.g.obj(wurm).controller, P1);
}

#[test]
fn akroan_war_leaving_before_chapter_one_resolves_takes_nothing() {
    cr!("611.2b");
    ruling!(
        "The Akroan War",
        "If The Akroan War leaves the battlefield before its first ability resolves, you won't gain control of the creature at all."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    let saga = enter(&mut t, P0, "The Akroan War");
    assert_eq!(stacked_triggers(&t), 1);
    move_to(&mut t, saga, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.g.obj(wurm).controller, P1);
}

#[test]
fn akroan_war_sacrifice_triggers_of_the_stolen_creature() {
    cr!("603.10a", "603.3a", "714.4");
    ruling!(
        "The Akroan War",
        "If the creature you took with The Akroan War's first ability has an ability that triggers when you sacrifice The Akroan War, you control the resulting trigger. If it has an ability that triggers when The Akroan War is put into a graveyard \"from anywhere,\" check whether that ability should trigger only after the creature's former controller has regained control of it."
    );
    supported("Mazirek, Kraul Death Priest");
    supported("Vulturous Zombie");
    // Mazirek: "Whenever a player sacrifices another permanent, put a +1/+1 counter on
    // each creature you control." P0 controls the trigger: P0's creatures get counters.
    let mut t = TestGame::new(2);
    let maz = t.battlefield(P1, "Mazirek, Kraul Death Priest");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let saga = akroan_war(&mut t, maz);
    next_chapter(&mut t, saga);
    t.resolve_all();
    next_chapter(&mut t, saga);
    t.resolve_all();
    assert!(!t.on_battlefield(saga));
    assert_eq!(t.g.obj(maz).controller, P1);
    assert_eq!(t.counters(mine, counters::PLUS1), 1);
    assert_eq!(t.counters(theirs, counters::PLUS1), 0);
    assert_eq!(t.counters(maz, counters::PLUS1), 0);
    // Vulturous Zombie: "Whenever a card is put into an opponent's graveyard from
    // anywhere, put a +1/+1 counter on this creature." Checked after P1 regained control:
    // P0's graveyard is an opponent's.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P1, "Vulturous Zombie");
    let saga = akroan_war(&mut t, z);
    next_chapter(&mut t, saga);
    t.resolve_all();
    next_chapter(&mut t, saga);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "The Akroan War"));
    assert_eq!(t.g.obj(z).controller, P1);
    assert_eq!(t.counters(z, counters::PLUS1), 1);
}

/// Synthetic Destiny for P0 with `mine` creatures to exile and the library stacked
/// `top_first`; advances to the end step and resolves its delayed trigger.
fn synthetic_destiny(t: &mut TestGame, top_first: &[&str]) {
    stack_library(t, P0, top_first);
    cast_resolve(t, P0, "Synthetic Destiny", &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

#[test]
fn synthetic_destiny_shuffles_even_with_no_noncreature_cards_and_keeps_the_exiled() {
    cr!("603.7", "701.24a", "406.3");
    ruling!(
        "Synthetic Destiny",
        "If you don't reveal any noncreature cards this way (perhaps because all the creature cards you needed were on top of your library), you still shuffle your library."
    );
    ruling!(
        "Synthetic Destiny",
        "The cards you exile will stay in exile. They're not part of the group you shuffle into your library."
    );
    supported("Synthetic Destiny");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let lib = t.library_size(P0);
    synthetic_destiny(&mut t, &["Craw Wurm", "Savannah Lions", "Forest"]);
    assert_eq!(shuffles(&t, P0), 1);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert_eq!(t.named_on_battlefield("Savannah Lions").len(), 1);
    assert_eq!(t.library_size(P0), lib + 1);
}

#[test]
fn synthetic_destiny_creatures_enter_simultaneously() {
    cr!("603.7", "603.6a", "603.10a");
    ruling!(
        "Synthetic Destiny",
        "The creature cards will enter the battlefield simultaneously."
    );
    supported("Soul Warden");
    // Two Soul Wardens ("Whenever another creature enters, you gain 1 life."): each sees
    // the other enter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    synthetic_destiny(&mut t, &["Soul Warden", "Forest", "Soul Warden"]);
    assert_eq!(t.named_on_battlefield("Soul Warden").len(), 2);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn synthetic_destiny_counts_an_exiled_commander() {
    cr!("903.9a", "704.6d", "608.2h");
    ruling!(
        "Synthetic Destiny",
        "If you exile your commander as Synthetic Destiny resolves, you may not put it into the command zone until after Synthetic Destiny is finished resolving. It will still count toward the number of creature cards you put onto the battlefield."
    );
    let mut t = crate::r_s13_common::commander_game();
    let cmd = crate::r_s13_common::commander(&mut t, P0, "Grizzly Bears");
    move_to(&mut t, cmd, Zone::Battlefield);
    t.battlefield(P0, "Hill Giant");
    yes(&mut t, P0);
    synthetic_destiny(&mut t, &["Craw Wurm", "Savannah Lions"]);
    assert_eq!(t.g.obj(t.g.current(cmd)).zone, Zone::Command);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert_eq!(t.named_on_battlefield("Savannah Lions").len(), 1);
}

#[test]
fn cast_it_enter_triggers() {
    cr!("603.4", "601.2a");
    ruling!(
        "Cresting Mosasaurus",
        "Cresting Mosasaurus's last ability triggers if you cast it from any zone. It doesn't trigger if you put Cresting Mosasaurus onto the battlefield without casting it."
    );
    ruling!(
        "Wakening Sun's Avatar",
        "If you put Wakening Sun's Avatar onto the battlefield from your hand without casting it, its ability won't trigger."
    );
    ruling!(
        "Wakening Sun's Avatar",
        "If you cast a creature spell that enters the battlefield as a copy of Wakening Sun's Avatar, such as Clone, the enters-the-battlefield ability will trigger."
    );
    supported("Cresting Mosasaurus");
    supported("Wakening Sun's Avatar");
    supported("Future Sight");
    // Mosasaurus cast from the top of the library (Future Sight): it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Cresting Mosasaurus");
    let m = t.library_top(P0, "Cresting Mosasaurus");
    t.cast(P0, m).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Put onto the battlefield without casting: no trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    enter(&mut t, P0, "Cresting Mosasaurus");
    assert_eq!(stacked_triggers(&t), 0);
    assert!(t.on_battlefield(bears));
    // Wakening Sun's Avatar put onto the battlefield from the hand: no trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let avatar = t.hand(P0, "Wakening Sun's Avatar");
    move_to(&mut t, avatar, Zone::Battlefield);
    assert!(t.on_battlefield(avatar));
    assert_eq!(stacked_triggers(&t), 0);
    assert!(t.on_battlefield(bears));
    // A Clone cast from the hand copying it: the trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wakening Sun's Avatar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    yes(&mut t, P0);
    let avatar = t.named_on_battlefield("Wakening Sun's Avatar")[0];
    t.answer_choose(P0, &[obj(avatar)]);
    cast_new(&mut t, P0, "Clone", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn cacophony_unleashed_auras_need_it_to_be_a_creature() {
    cr!("303.4a", "704.5m", "613.1d");
    ruling!(
        "Cacophony Unleashed",
        "You can't cast an Aura with \"enchant creature\" targeting Cacophony Unleashed unless it's already a creature before you cast that Aura.  When Cacophony Unleashed stops being a creature, any such auras will be put into their owners' graveyards."
    );
    supported("Cacophony Unleashed");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cacophony Unleashed");
    lands_for_cost(&mut t, P0, "Holy Strength");
    let hs = t.hand(P0, "Holy Strength");
    assert!(!can_cast(&mut t, P0, hs, CastMethod::Normal));
    // Another enchantment enters: Cacophony becomes a 6/6 creature until end of turn.
    enter(&mut t, P0, "Glorious Anthem");
    t.resolve_all();
    assert!(t.obj(c).is(CardType::Creature));
    t.cast(P0, hs).target(obj(c)).go();
    t.resolve_all();
    assert_eq!(t.obj(t.g.current(hs)).attached_to, Some(obj(c)));
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj(c).is(CardType::Creature));
    assert!(t.in_graveyard(P0, "Holy Strength"));
}

#[test]
fn divergent_transformations_needs_two_targets_and_each_is_separate() {
    cr!("601.2c", "608.2b", "115.1");
    ruling!(
        "Divergent Transformations",
        "You can't cast Divergent Transformations without two target creatures. If a targeted creature becomes an illegal target for Divergent Transformations after it's been cast, its controller won't exile or replace that creature, but the other creature is still affected."
    );
    supported("Divergent Transformations");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Divergent Transformations");
    let dt = t.hand(P0, "Divergent Transformations");
    assert!(!can_cast(&mut t, P0, dt, CastMethod::Normal));
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(can_cast(&mut t, P0, dt, CastMethod::Normal));
    stack_library(&mut t, P0, &["Craw Wurm"]);
    stack_library(&mut t, P1, &["Forest", "Savannah Lions"]);
    t.cast(P0, dt).targets(&[obj(mine), obj(theirs)]).go();
    move_to(&mut t, mine, Zone::Hand(P0));
    t.resolve_all();
    // P0's creature was bounced: P0 reveals nothing.
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    // P1's Bears were exiled and replaced.
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Savannah Lions").len(), 1);
}
