//! Rulings batch P204 — crew (CR 702.122) and coven.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn golden_argosy_exiles_only_crew_still_on_the_battlefield() {
    cr!("702.122a", "702.122c", "603.7");
    ruling!(
        "Golden Argosy",
        "Golden Argosy’s triggered ability will only exile creatures that are still on the battlefield as it resolves"
    );
    supported("Golden Argosy");
    // "Whenever Golden Argosy attacks, exile each creature that crewed it this turn.
    // Return them to the battlefield tapped under their owner's control at the beginning
    // of the next end step."
    let mut t = TestGame::new(2);
    let argosy = t.battlefield(P0, "Golden Argosy");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, argosy, &[a, b]));
    t.resolve_all();
    destroy(&mut t, a);
    attack_with(&mut t, &[(argosy, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    assert!(t.obj_now(giant[0]).tapped);
    // The Bears that had left the battlefield aren't returned.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn mindlink_mech_triggers_the_first_time_it_becomes_crewed_each_turn() {
    cr!("702.122a", "702.122d", "707.9b");
    ruling!(
        "Mindlink Mech",
        "Mindlink Mech's second ability triggers as its crew ability resolves for the first time each turn."
    );
    ruling!(
        "Mindlink Mech",
        "Mindlink Mech's triggered ability can target only a creature that was tapped to pay its crew cost this turn."
    );
    supported("Mindlink Mech");
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mindlink Mech");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Savannah Lions");
    assert!(crew(&mut t, P0, mech, &[elves]));
    // Activated and paid, but not resolved yet: no trigger.
    assert_eq!(on_stack(&t, "becomes crewed"), 0);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.resolve();
    t.settle();
    assert_eq!(on_stack(&t, "becomes crewed"), 1);
    // Only the Elves (which crewed it) could be targeted.
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(offered, vec![vec![Entity::Object(elves)]]);
    let _ = (other, theirs);
    // Crewed again by the Bears in response: no second trigger.
    assert!(crew(&mut t, P0, mech, &[bears]));
    t.resolve();
    t.settle();
    assert_eq!(on_stack(&t, "becomes crewed"), 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(mech).chars.name, "Llanowar Elves");
    assert_eq!(t.pt(mech), (4, 3));
}

#[test]
fn mindlink_mech_copying_a_noncreature_permanent_is_a_0_0() {
    cr!("707.9b", "702.122a", "613.2a", "208.3");
    ruling!(
        "Mindlink Mech",
        "If a player crews Mindlink Mech with a nonlegendary permanent that is not normally a creature or Vehicle, the resulting permanent Mindlink Mech becomes is a 0/0 artifact creature"
    );
    supported("Mutavault");
    // Mutavault: "{1}: Until end of turn, this land becomes a 2/2 creature with all
    // creature types." It crews the Mech; the Mech becomes a copy of Mutavault (a land,
    // not a creature in its copiable values), a Vehicle artifact; the crew effect makes it
    // an artifact creature with no defined power and toughness: 0/0.
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mindlink Mech");
    let vault = t.battlefield(P0, "Mutavault");
    add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 1);
    t.activate(P0, vault, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).is_creature());
    t.answer_targets(P0, &[Entity::Object(vault)]);
    assert!(crew(&mut t, P0, mech, &[vault]));
    t.resolve_all();
    assert!(!t.on_battlefield(mech));
    assert!(t.in_graveyard(P0, "Mindlink Mech"));
}

#[test]
fn unlicensed_hearse_crewed_with_nothing_exiled_is_a_0_0() {
    cr!("702.122a", "208.3", "704.5f");
    ruling!(
        "Unlicensed Hearse",
        "If you activated the crew ability of Unlicensed Hearse before exiling any cards with its first ability, its power and toughness will be 0/0."
    );
    supported("Unlicensed Hearse");
    let mut t = TestGame::new(2);
    let hearse = t.battlefield(P0, "Unlicensed Hearse");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, hearse, &[bears]));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Unlicensed Hearse"));
    // With two cards exiled with it, it's a 2/2 creature once crewed.
    let mut t = TestGame::new(2);
    let hearse = t.battlefield(P0, "Unlicensed Hearse");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.graveyard(P1, "Hill Giant");
    let b = t.graveyard(P1, "Savannah Lions");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, hearse, 0, &[]).unwrap();
    t.resolve_all();
    assert!(crew(&mut t, P0, hearse, &[bears]));
    t.resolve_all();
    assert!(t.on_battlefield(hearse));
    assert_eq!(t.pt(hearse), (2, 2));
}

#[test]
fn burner_rocket_cant_be_crewed_in_time_to_target_itself() {
    cr!("603.3d", "702.122a", "115.1");
    ruling!(
        "Burner Rocket",
        "This Vehicle’s triggered ability requires a target at the time you put it on the stack, which means you can’t crew this Vehicle in time to make it a legal target for its own triggered ability."
    );
    supported("Burner Rocket");
    // "When this Vehicle enters, target creature you control gets +2/+0 and gains trample
    // until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Burner Rocket");
    let from = t.asked().len();
    t.cast(P0, card).go();
    t.resolve();
    t.settle();
    let rocket = t.named_on_battlefield("Burner Rocket")[0];
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(offered, vec![vec![Entity::Object(bears)]]);
    assert!(!offered[0].contains(&Entity::Object(rocket)));
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn harvesttide_sentrys_restriction_is_checked_only_as_blockers_are_declared() {
    cr!("509.1b", "506.4");
    ruling!(
        "Harvesttide Sentry",
        "The blocking restriction is applied only when blockers are declared."
    );
    supported("Harvesttide Sentry");
    // Harvesttide Sentry (3/1): "Coven — At the beginning of combat on your turn, if you
    // control three or more creatures with different powers, this creature can't be
    // blocked by creatures with power 2 or less this turn." Powers 3, 2, 1.
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Harvesttide Sentry");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the coven trigger");
    t.resolve_all();
    attack_with(&mut t, &[(sentry, Entity::Player(P1))]);
    // The 2-power Bears can't block it (P1's attempt is rejected and nothing blocks).
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, sentry)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(bears) && t.on_battlefield(giant));
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Harvesttide Sentry");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    let giant2 = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    attack_with(&mut t, &[(sentry, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant2, sentry)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    // Reducing the Giant's power to 1 afterward doesn't make the Sentry unblocked.
    t.lands(P0, "Swamp", 1);
    let slick = t.hand(P0, "Disfigure");
    t.cast(P0, slick).target(giant2).go();
    t.resolve_all();
    assert_eq!(t.pt(giant2), (1, 1));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn coven_is_checked_as_it_triggers_and_as_it_resolves() {
    cr!("603.4", "207.2c");
    ruling!(
        "Redemption Choir",
        "You must control three or more creatures with different powers at the time the coven ability triggers and at the time the ability tries to resolve. They do not, however, need to be the same set of creatures in both cases."
    );
    supported("Redemption Choir");
    // Redemption Choir (3/3): "Coven — Whenever this creature enters or attacks, if you
    // control three or more creatures with different powers, return target permanent card
    // with mana value 3 or less from your graveyard to the battlefield."
    // Choir (3), Bears (2), Ornithopter (0). Before the trigger resolves the Bears is
    // destroyed (only two powers: nothing returns), or destroyed and replaced by Llanowar
    // Elves (1): a different set of three powers still counts.
    for (swap, kill_only, returns) in [(false, false, true), (true, false, true), (false, true, false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let ornithopter = t.battlefield(P0, "Ornithopter");
        let target = t.graveyard(P0, "Mind Stone");
        let card = t.hand(P0, "Redemption Choir");
        t.lands(P0, "Plains", 4);
        t.answer_targets(P0, &[Entity::Object(target)]);
        t.cast(P0, card).go();
        t.resolve();
        t.settle();
        assert_eq!(t.stack_len(), 1, "the coven trigger");
        if swap || kill_only {
            destroy(&mut t, bears);
        }
        if swap {
            t.battlefield(P0, "Llanowar Elves");
        }
        let _ = ornithopter;
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Mind Stone").len(),
            returns as usize,
            "{swap} {kill_only}"
        );
    }
    // Not true as it would trigger: it doesn't trigger at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let target = t.graveyard(P0, "Mind Stone");
    let card = t.hand(P0, "Redemption Choir");
    t.lands(P0, "Plains", 4);
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.cast(P0, card).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

