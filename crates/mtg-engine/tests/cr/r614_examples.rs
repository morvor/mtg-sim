//! The worked examples of CR 614 (replacement effects), played out as tests. The CR 614.5
//! example is cited by a test in `r609_614_replacement.rs`.

use crate::r600_examples_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The candidates of every "choose entities" decision asked so far.
fn entity_choices(t: &TestGame) -> Vec<Vec<Entity>> {
    t.asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// A Planechase game whose face-up plane is Jund.
fn jund_game() -> TestGame {
    let mut t = TestGame::with_config(
        2,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Planechase,
            ..Default::default()
        },
    );
    let jund = t.command(P0, "Jund");
    t.g.objects[jund.0 as usize].face_down = true;
    t.g.recompute();
    mtg_engine::planechase::set_starting_plane(&mut t.g);
    t.g.recompute();
    t
}

#[test]
fn regenerating_in_response_to_a_destroy_spell_saves_the_creature_but_not_afterward() {
    cr!("614.8", "701.19a");
    cr_example!("614.4", "activate an ability to regenerate a creature in response");
    supported("Drudge Skeletons");
    supported("Murder");
    let mut t = TestGame::new(2);
    let skel = t.battlefield(P0, "Drudge Skeletons");
    t.lands(P0, "Swamp", 2);
    t.lands(P1, "Swamp", 6);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    // In response to Murder, the regeneration shield is created first.
    let murder = t.hand(P1, "Murder");
    t.cast(P1, murder).target(skel).go();
    t.activate(P0, skel, 0, &[]).unwrap();
    t.resolve(); // the regeneration shield
    t.resolve(); // Murder
    assert!(t.on_battlefield(skel));
    assert!(t.obj_now(skel).tapped);
    assert!(t.in_graveyard(P1, "Murder"));
    // Once a destroy spell has resolved, it's too late: the creature is gone, and the
    // regeneration ability can't be activated any more.
    let murder = t.hand(P1, "Murder");
    t.cast(P1, murder).target(skel).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Drudge Skeletons"));
    let dead = t.g.find_in_zone(Zone::Graveyard(P0), "Drudge Skeletons")[0];
    assert!(t.activate(P0, dead, 0, &[]).is_err());
}

#[test]
fn a_token_copy_of_voice_of_all_chooses_its_own_color_as_its_created() {
    cr!("614.12a", "707.2");
    cr_example!("614.12", "An effect creates a token that’s a copy of Voice of All");
    supported("Voice of All");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let voice = t.hand(P0, "Voice of All");
    t.lands(P0, "Plains", 4);
    // White for the original.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, voice).go();
    t.resolve();
    let voice = t.named_on_battlefield("Voice of All")[0];
    assert_eq!(t.obj_now(voice).choices.color, Some(Color::White));
    // Green for the token, chosen by the token's controller as it's created.
    t.lands(P0, "Island", 3);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(voice).go();
    t.resolve();
    let token = t
        .named_on_battlefield("Voice of All")
        .into_iter()
        .find(|v| t.obj_now(*v).is_token())
        .expect("a token copy");
    assert_eq!(t.obj_now(token).choices.color, Some(Color::Green));
    assert_eq!(t.obj_now(voice).choices.color, Some(Color::White));
    let color_choices = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseOption { .. }))
        .count();
    assert_eq!(color_choices, 2);
    // It has protection from the chosen color: a green spell can't target it.
    let growth = t.hand(P1, "Giant Growth");
    assert!(t.g.protected_from(token, growth));
    assert!(!t.g.protected_from(voice, growth));
}

#[test]
fn scarwood_treefolk_returned_from_a_yixlid_jailer_graveyard_enters_tapped() {
    cr!("614.12");
    cr_example!("614.12", "Yixlid Jailer says");
    supported("Yixlid Jailer");
    supported("Scarwood Treefolk");
    supported("Zombify");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yixlid Jailer");
    let tf = t.graveyard(P0, "Scarwood Treefolk");
    t.recompute();
    // In the graveyard, the card has no abilities ...
    assert!(t.obj_now(tf).chars.abilities.is_empty());
    t.lands(P0, "Swamp", 4);
    let z = t.hand(P0, "Zombify");
    t.cast(P0, z).target(tf).go();
    t.resolve();
    // ... but "enters tapped" is checked as it would exist on the battlefield.
    let tf = t.named_on_battlefield("Scarwood Treefolk")[0];
    assert!(t.obj_now(tf).tapped);
}

#[test]
fn orb_of_dreams_doesnt_make_itself_enter_tapped() {
    cr!("614.12", "614.1d");
    cr_example!("614.12", "Orb of Dreams is an artifact");
    supported("Orb of Dreams");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let orb = t.hand(P0, "Orb of Dreams");
    t.cast(P0, orb).go();
    t.resolve();
    let orb = t.named_on_battlefield("Orb of Dreams")[0];
    assert!(!t.obj_now(orb).tapped);
    // Other permanents do enter tapped.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn sutured_ghoul_cant_exile_a_card_entering_the_battlefield_with_it() {
    cr!("614.13");
    cr_example!("614.13a", "Sutured Ghoul says");
    supported("Sutured Ghoul");
    let mut t = TestGame::new(2);
    let ghoul = t.graveyard(P0, "Sutured Ghoul");
    let bear = t.graveyard(P0, "Runeclaw Bear");
    let other = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let raise = t.custom(
        P0,
        custom_costed(
            "Double Raise",
            "Sorcery",
            "{3}{B}",
            "Return up to two target creature cards from your graveyard to the battlefield.",
        ),
        Zone::Hand(P0),
    );
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.cast(P0, raise)
        .targets(&[Entity::Object(ghoul), Entity::Object(bear)])
        .go();
    t.resolve();
    // Runeclaw Bear (and the Ghoul itself) weren't among the cards that could be chosen.
    let choices = entity_choices(&t);
    assert_eq!(choices, vec![vec![Entity::Object(other)]]);
    assert_eq!(t.named_on_battlefield("Runeclaw Bear").len(), 1);
    let ghoul = t.named_on_battlefield("Sutured Ghoul")[0];
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.pt(ghoul), (3, 3));
}

#[test]
fn thunder_thrash_elder_with_jund_can_devour_runeclaw_bear_only_once() {
    cr!("614.13", "702.82a", "616.1");
    cr_example!("614.13b", "Thunder-Thrash Elder");
    supported("Jund");
    supported("Thunder-Thrash Elder");
    // (which devour applies first: 0 = devour 3, 1 = devour 5; sacrifice the bear for
    // the first or the second one applied, or neither; expected +1/+1 counters)
    for (first, sac, expected) in [
        (0, Some(0), 3),
        (0, Some(1), 5),
        (1, Some(0), 5),
        (1, Some(1), 3),
        (0, None, 0),
    ] {
        let mut t = jund_game();
        let bear = t.battlefield(P0, "Runeclaw Bear");
        t.lands(P0, "Mountain", 3);
        let elder = t.hand(P0, "Thunder-Thrash Elder");
        t.cast(P0, elder).go();
        t.resolve(); // Jund's trigger: the spell gains devour 5.
        assert_eq!(t.stack_len(), 1);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(first));
        for i in 0..2 {
            let pick: &[Entity] = if sac == Some(i) {
                &[Entity::Object(bear)]
            } else {
                &[]
            };
            t.answer_choose(P0, pick);
        }
        t.resolve();
        let elder = t.named_on_battlefield("Thunder-Thrash Elder")[0];
        assert_eq!(t.counters(elder, counters::PLUS1), expected);
        assert_eq!(t.on_battlefield(bear), sac.is_none());
        // Once sacrificed for one devour effect, the bear isn't offered to the other.
        let offered = entity_choices(&t)
            .iter()
            .filter(|c| c.contains(&Entity::Object(bear)))
            .count();
        assert_eq!(offered, if sac == Some(0) { 1 } else { 2 });
    }
}
