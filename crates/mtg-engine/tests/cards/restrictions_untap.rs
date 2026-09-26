//! "You may choose not to untap ~ during your untap step." (CR 502.3) and effects that
//! last "for as long as ~ remains tapped" (CR 611.2b, 611.2c).

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

/// Moves on to `p`'s next upkeep, through their untap step.
fn next_untap(t: &mut TestGame, p: PlayerId) {
    let other = if p == P0 { P1 } else { P0 };
    t.set_step(other, Step::End);
    t.advance_to(p, Step::Upkeep);
}

fn untap_questions(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("during your untap step")))
        .count()
}

#[test]
fn keeps_control_for_as_long_as_it_remains_tapped() {
    cr!("502.3", "611.2b");
    compiles("Rubinia Soulsinger");
    compiles("Willow Satyr");
    compiles("Helm of Possession");
    let mut t = TestGame::new(2);
    let rubinia = t.battlefield(P0, "Rubinia Soulsinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, rubinia, 0, &[bears.into()]).unwrap();
    t.resolve();
    assert_eq!(controller(&t, bears), P0);
    // Its controller chooses to leave it tapped (the default while its effect lasts).
    next_untap(&mut t, P0);
    assert!(tapped(&t, rubinia));
    assert_eq!(controller(&t, bears), P0);
    assert_eq!(untap_questions(&t, P0), 1);
    // Choosing to untap it ends the effect.
    t.answer_yes(P0, true);
    next_untap(&mut t, P0);
    assert!(!tapped(&t, rubinia));
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn untapped_before_resolution_means_no_effect() {
    cr!("611.2b");
    ruling!(
        "Rubinia Soulsinger",
        "even if it becomes tapped again right away — you won’t gain control"
    );
    let mut t = TestGame::new(2);
    let rubinia = t.battlefield(P0, "Rubinia Soulsinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, rubinia, 0, &[bears.into()]).unwrap();
    t.g.untap(rubinia);
    t.g.tap(rubinia);
    t.resolve();
    assert_eq!(controller(&t, bears), P1);
    // Untapped and not tapped again: likewise.
    let mut t = TestGame::new(2);
    let rubinia = t.battlefield(P0, "Rubinia Soulsinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, rubinia, 0, &[bears.into()]).unwrap();
    t.g.untap(rubinia);
    t.resolve();
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn leaving_the_battlefield_ends_the_effect() {
    cr!("611.2b", "400.7");
    ruling!(
        "Rubinia Soulsinger",
        "If Rubinia Soulsinger leaves the battlefield, you no longer control it"
    );
    let mut t = TestGame::new(2);
    let rubinia = t.battlefield(P0, "Rubinia Soulsinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, rubinia, 0, &[bears.into()]).unwrap();
    t.resolve();
    assert_eq!(controller(&t, bears), P0);
    t.g.destroy(rubinia, None);
    t.g.recompute();
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn tapped_land_doesnt_untap_while_the_source_stays_tapped() {
    cr!("502.3", "611.2b");
    compiles("Mana Leech");
    compiles("Sand Squid");
    compiles("Deserter's Quarters");
    let mut t = TestGame::new(2);
    let leech = t.battlefield(P0, "Mana Leech");
    let forest = t.battlefield(P1, "Forest");
    t.activate(P0, leech, 0, &[forest.into()]).unwrap();
    t.resolve();
    assert!(tapped(&t, forest));
    next_untap(&mut t, P1);
    assert!(tapped(&t, forest));
    // Mana Leech's controller untaps it; the land untaps during its controller's next
    // untap step.
    t.answer_yes(P0, true);
    next_untap(&mut t, P0);
    assert!(!tapped(&t, leech));
    assert!(tapped(&t, forest));
    next_untap(&mut t, P1);
    assert!(!tapped(&t, forest));
}

#[test]
fn affected_permanent_you_control_stays_tapped_the_step_the_source_untaps() {
    cr!("502.3");
    ruling!(
        "Rust Tick",
        "that artifact won’t untap during the untap step in which you choose to untap Rust Tick"
    );
    ruling!(
        "Rust Tick",
        "If the affected artifact is untapped by some other spell or ability, Rust Tick’s effect will not end."
    );
    compiles("Rust Tick");
    let mut t = TestGame::new(2);
    let tick = t.battlefield(P0, "Rust Tick");
    let stone = t.battlefield(P0, "Mind Stone");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, tick, 0, &[stone.into()]).unwrap();
    t.resolve();
    assert!(tapped(&t, stone));
    // Untapped by something else and tapped again: still held.
    t.g.untap(stone);
    t.g.tap(stone);
    next_untap(&mut t, P0);
    assert!(tapped(&t, tick) && tapped(&t, stone));
    t.answer_yes(P0, true);
    next_untap(&mut t, P0);
    assert!(!tapped(&t, tick));
    assert!(tapped(&t, stone));
    next_untap(&mut t, P0);
    assert!(!tapped(&t, stone));
}

#[test]
fn pump_lasts_while_tapped() {
    cr!("611.2b");
    compiles("Zelyon Sword");
    compiles("Everglove Courier");
    compiles("Endoskeleton");
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P0, "Everglove Courier");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 3);
    t.activate(P0, courier, 0, &[elves.into()]).unwrap();
    t.resolve();
    assert_eq!(t.pt(elves), (3, 3));
    assert!(t
        .obj_now(elves)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    t.g.untap(courier);
    t.g.recompute();
    assert_eq!(t.pt(elves), (1, 1));
    assert!(!t
        .obj_now(elves)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn all_creatures_means_those_on_the_battlefield_as_it_resolves() {
    cr!("611.2c");
    compiles("Thran Weaponry");
    let mut t = TestGame::new(2);
    let weaponry = t.battlefield(P0, "Thran Weaponry");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, weaponry, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(mine), (4, 4));
    assert_eq!(t.pt(theirs), (4, 4));
    let later = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(later), (2, 2));
    // The default choice keeps it tapped while its effect lasts.
    next_untap(&mut t, P0);
    assert!(tapped(&t, weaponry));
    assert_eq!(t.pt(mine), (4, 4));
}

#[test]
fn without_the_option_the_source_untaps_as_usual() {
    cr!("502.3");
    ruling!(
        "Kill Switch",
        "Only the artifacts that it tried to tap when the ability resolved are prevented from untapping."
    );
    ruling!(
        "Kill Switch",
        "you can’t choose to leave this card tapped during your untap step"
    );
    compiles("Kill Switch");
    let mut t = TestGame::new(2);
    let switch = t.battlefield(P0, "Kill Switch");
    let stone = t.battlefield(P1, "Mind Stone");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, switch, 0, &[]).unwrap();
    t.resolve();
    assert!(tapped(&t, stone));
    let later = t.battlefield(P1, "Mind Stone");
    t.g.tap(later);
    next_untap(&mut t, P1);
    assert!(tapped(&t, stone));
    assert!(!tapped(&t, later));
    next_untap(&mut t, P0);
    assert!(!tapped(&t, switch));
    assert_eq!(untap_questions(&t, P0), 0);
    next_untap(&mut t, P1);
    assert!(!tapped(&t, stone));
}

#[test]
fn nothing_to_keep_tapped_untaps_by_default() {
    cr!("502.3");
    compiles("Tawnos's Weaponry");
    let mut t = TestGame::new(2);
    let leech = t.battlefield(P0, "Mana Leech");
    t.g.tap(leech);
    next_untap(&mut t, P0);
    assert!(!tapped(&t, leech));
    // It can also be left tapped.
    t.g.tap(leech);
    t.answer_yes(P0, false);
    next_untap(&mut t, P0);
    assert!(tapped(&t, leech));
}

#[test]
fn permanents_untap_during_each_other_players_untap_step() {
    cr!("502.3");
    ruling!(
        "Seedborn Muse",
        "All your permanents untap during each other player's untap step."
    );
    compiles("Seedborn Muse");
    compiles("Prophet of Kruphix");
    compiles("Drumbellower");
    compiles("Ivorytusk Fortress");
    compiles("Urban Burgeoning");
    let mut t = TestGame::new(2);
    let muse = t.battlefield(P0, "Seedborn Muse");
    let forest = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    for id in [muse, forest, theirs] {
        t.g.tap(id);
    }
    next_untap(&mut t, P1);
    assert!(!tapped(&t, muse) && !tapped(&t, forest));
    assert!(!tapped(&t, theirs));
    // Without the Muse, nothing of P0's untaps during P1's untap step.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.g.tap(forest);
    next_untap(&mut t, P1);
    assert!(tapped(&t, forest));
}

#[test]
fn only_the_matching_permanents_untap_and_doesnt_untap_effects_dont_apply() {
    cr!("502.3");
    ruling!(
        "Unwinding Clock",
        "These effects won’t apply and stop the artifact from untapping during another player’s untap step."
    );
    compiles("Unwinding Clock");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unwinding Clock");
    let stone = t.battlefield(P0, "Mind Stone");
    let colossus = t.battlefield(P0, "Colossus of Sardia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let tick = t.battlefield(P1, "Rust Tick");
    t.lands(P1, "Wastes", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, tick, 0, &[stone.into()]).unwrap();
    t.resolve();
    t.g.tap(colossus);
    t.g.tap(bears);
    // P1's untap step: P1 keeps Rust Tick tapped; P0's artifacts untap anyway, Colossus
    // of Sardia ("doesn't untap during your untap step") included; the Bears don't.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert!(tapped(&t, tick));
    assert!(!tapped(&t, stone));
    assert!(!tapped(&t, colossus));
    assert!(tapped(&t, bears));
}

#[test]
fn green_and_or_blue_creatures_untap_during_each_other_players_untap_step() {
    cr!("502.3");
    ruling!(
        "Murkfiend Liege",
        "effects that would otherwise cause your green and/or blue creatures to stay tapped don't apply"
    );
    ruling!(
        "Murkfiend Liege",
        "You can't untap your permanents more than once in a single untap step."
    );
    compiles("Murkfiend Liege");
    compiles("Nettle Sentinel");
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Murkfiend Liege");
    t.battlefield(P0, "Murkfiend Liege");
    let sentinel = t.battlefield(P0, "Nettle Sentinel");
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let forest = t.battlefield(P0, "Forest");
    for id in [liege, sentinel, merfolk, goblin, forest] {
        t.g.tap(id);
    }
    let untaps_before = untap_events(&t, sentinel);
    // P1's untap step: the green and blue creatures untap (Nettle Sentinel too: its
    // "doesn't untap during your untap step" doesn't apply), once each.
    next_untap(&mut t, P1);
    assert!(!tapped(&t, liege) && !tapped(&t, sentinel) && !tapped(&t, merfolk));
    assert!(tapped(&t, goblin));
    assert!(tapped(&t, forest));
    assert_eq!(untap_events(&t, sentinel), untaps_before + 1);
    // P0's own untap step: Nettle Sentinel stays tapped as usual.
    t.g.tap(sentinel);
    next_untap(&mut t, P0);
    assert!(tapped(&t, sentinel));
    assert!(!tapped(&t, goblin));
}

fn untap_events(t: &TestGame, id: ObjectId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, mtg_engine::events::Event::Untapped { obj } if *obj == id))
        .count()
}

#[test]
fn a_restriction_whose_source_untapped_before_resolution_does_nothing() {
    cr!("611.2b");
    let mut t = TestGame::new(2);
    let leech = t.battlefield(P0, "Mana Leech");
    let forest = t.battlefield(P1, "Forest");
    t.activate(P0, leech, 0, &[forest.into()]).unwrap();
    t.g.untap(leech);
    t.g.tap(leech);
    t.resolve();
    // The land is tapped, but its "doesn't untap" effect never began.
    assert!(tapped(&t, forest));
    assert!(tapped(&t, leech));
    next_untap(&mut t, P1);
    assert!(!tapped(&t, forest));
}

#[test]
fn untap_this_during_each_other_players_untap_step() {
    cr!("502.3");
    ruling!(
        "Bender's Waterskin",
        "untaps at the same time as the active player's permanents"
    );
    compiles("Bender's Waterskin");
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, "Bender's Waterskin");
    t.g.tap(skin);
    next_untap(&mut t, P1);
    assert!(!tapped(&t, skin));
    // And during its controller's own untap step, as usual.
    t.g.tap(skin);
    next_untap(&mut t, P0);
    assert!(!tapped(&t, skin));
}

#[test]
fn you_may_tap_or_untap_target_permanent() {
    cr!("701.26a", "701.26b");
    compiles("Jolt");
    compiles("Niblis of the Breath");
    compiles("Teardrop Kami");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.g.tap(forest);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jolt = t.hand(P0, "Jolt");
    t.lands(P0, "Island", 3);
    // Untap P0's land.
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    t.cast(P0, jolt).target(forest).go();
    t.resolve();
    assert!(!tapped(&t, forest));
    // Tap an opponent's creature.
    let kami = t.battlefield(P0, "Teardrop Kami");
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.activate(P0, kami, 0, &[bears.into()]).unwrap();
    t.resolve();
    assert!(tapped(&t, bears));
    // "Another target" after an earlier target (Hidden Strings) isn't handled.
    assert!(!card("Hidden Strings").unsupported_text().is_empty());
}
