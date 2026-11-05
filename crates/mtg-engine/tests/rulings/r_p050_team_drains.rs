//! Rulings batch P050 (second round) — "each opponent loses N life and you gain N life"
//! in Two-Headed Giant and other multiplayer games: each opponent is affected
//! individually and the results are applied to the team's shared life total (CR 810.9),
//! while you gain life once. Also Doom Foretold, whose "each player's upkeep" ability
//! triggers once for each player of the active team (CR 805.4d).

use crate::r_p050_common::*;
use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s05_common::tokens_with_subtype;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Runs `go` in a fresh 2HG game and checks P0's team gained `gain` and the opposing team
/// lost `loss`.
fn drain_2hg(name: &str, gain: i32, loss: i32, go: impl FnOnce(&mut TestGame)) {
    supported(name);
    let mut t = two_headed_giant();
    let before = team_lives(&t);
    assert_eq!(before, (30, 30));
    go(&mut t);
    assert_team_change(&t, before, gain, loss, name);
}

/// Moves to the beginning of P0's precombat main phase (its "first main phase").
fn into_first_main(t: &mut TestGame) {
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
}

/// Moves to the beginning of P0's end step.
fn into_end_step(t: &mut TestGame) {
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
}

#[test]
fn hooded_blightfang_attack_trigger_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Hooded Blightfang",
        "In a Two-Headed Giant game, Hooded Blightfang's middle ability causes the opposing team to lose 2 life and you to gain 1 life."
    );
    drain_2hg("Hooded Blightfang", 1, 2, |t| {
        let b = t.battlefield(P0, "Hooded Blightfang");
        attack_with(t, &[(b, Entity::Player(P2))]);
        t.resolve_all();
        // Only the trigger so far: no combat damage has been dealt.
        assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    });
}

#[test]
fn klothys_last_ability_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Klothys, God of Destiny",
        "In a Two-Headed Giant game, Klothys’s last ability causes the opposing team to lose 4 life and you gain 2 life."
    );
    drain_2hg("Klothys, God of Destiny", 2, 4, |t| {
        t.battlefield(P0, "Klothys, God of Destiny");
        let bears = t.graveyard(P2, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        into_first_main(t);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
    });
}

#[test]
fn klothys_nonland_card_gains_only_2_life_however_many_opponents() {
    cr!("120.3a");
    ruling!(
        "Klothys, God of Destiny",
        "If you exile a nonland card, you gain only 2 life, no matter how many opponents are dealt damage."
    );
    supported("Klothys, God of Destiny");
    // A four-player free-for-all game: three opponents.
    let mut t = TestGame::new(4);
    t.battlefield(P0, "Klothys, God of Destiny");
    let bears = t.graveyard(P3, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    into_first_main(&mut t);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P0), 22);
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (18, 18, 18));
}

#[test]
fn liliana_untouched_by_death_first_ability_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Liliana, Untouched by Death",
        "In a Two-Headed Giant game, Liliana’s first ability causes the opposing team to lose 4 life and you gain 2 life."
    );
    drain_2hg("Liliana, Untouched by Death", 2, 4, |t| {
        let l = t.battlefield(P0, "Liliana, Untouched by Death");
        t.library_top(P0, "Diregraf Ghoul");
        t.activate(P0, l, 0, &[]).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Diregraf Ghoul"));
    });
}

#[test]
fn obelisk_spider_last_ability_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Obelisk Spider",
        "In a Two-Headed Giant game, Obelisk Spider’s last ability causes the opposing team to lose 2 life and you gain 1 life."
    );
    drain_2hg("Obelisk Spider", 1, 2, |t| {
        let s = t.battlefield(P0, "Obelisk Spider");
        let bears = t.battlefield(P2, "Hill Giant");
        t.attack(&[(s, Entity::Player(P2))], &[(bears, s)]);
        t.resolve_all();
        assert_eq!(t.counters(bears, "-1/-1"), 1);
    });
}

#[test]
fn sol_kanar_second_mode_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Sol'Kanar the Tainted",
        "In a Two-Headed Giant game, Sol'kanar the Tainted's second mode causes the opposing team to lose 4 life. Its controller will still gain only 2 life."
    );
    drain_2hg("Sol'Kanar the Tainted", 2, 4, |t| {
        t.battlefield(P0, "Sol'Kanar the Tainted");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        into_end_step(t);
        t.resolve_all();
    });
}

#[test]
fn tezzeret_master_of_the_bridge_first_loyalty_ability_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Tezzeret, Master of the Bridge",
        "In a Two-Headed Giant game, Tezzeret's first loyalty ability causes the opposing team to lose twice X life and you gain X life."
    );
    // X = 2 artifacts.
    drain_2hg("Tezzeret, Master of the Bridge", 2, 4, |t| {
        let tz = t.battlefield(P0, "Tezzeret, Master of the Bridge");
        t.battlefield(P0, "Ornithopter");
        t.battlefield(P0, "Ornithopter");
        t.activate(P0, tz, 0, &[]).unwrap();
        t.resolve_all();
    });
    // X = 3 artifacts.
    drain_2hg("Tezzeret, Master of the Bridge", 3, 6, |t| {
        let tz = t.battlefield(P0, "Tezzeret, Master of the Bridge");
        for _ in 0..3 {
            t.battlefield(P0, "Ornithopter");
        }
        t.activate(P0, tz, 0, &[]).unwrap();
        t.resolve_all();
    });
}

#[test]
fn sorin_grim_nemesis_highest_life_total_is_a_teams() {
    cr!("810.9a");
    ruling!(
        "Sorin, Grim Nemesis",
        "In a Two-Headed Giant game, the highest life total among all players is the highest life total among all teams."
    );
    supported("Sorin, Grim Nemesis");
    let mut t = two_headed_giant();
    // P3 (the opposing team) casts Renewed Faith ("You gain 6 life."): that team has 36.
    crate::r_s29_common::cast_and_resolve(&mut t, P3, "Renewed Faith", &[]);
    assert_eq!(team_lives(&t), (30, 36));
    let s = crate::r_p007_common::pw(&mut t, P0, "Sorin, Grim Nemesis", 9);
    t.activate(P0, s, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Vampire").len(), 36);
}

/// A 2HG game in which P0 controls Doom Foretold and P2's team (P2 and P3) begins its
/// upkeep, P0 answering the order of its two triggers with `order` (first = bottom of the
/// stack). P3 has only a land, and a card in hand; P2 has `p2` on the battlefield.
fn doom_2hg(order: Option<Vec<usize>>, p2: Option<&str>) -> (TestGame, ObjectId) {
    let mut t = two_headed_giant();
    let doom = t.battlefield(P0, "Doom Foretold");
    if let Some(name) = p2 {
        t.battlefield(P2, name);
    }
    t.hand(P2, "Grizzly Bears");
    t.battlefield(P3, "Swamp");
    t.hand(P3, "Grizzly Bears");
    if let Some(o) = order {
        t.answer(P0, DecisionKind::Order, Answer::Indices(o));
    }
    let from = t.asked().len();
    t.advance_to(P2, Step::Upkeep);
    t.settle();
    let orders = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. }))
        .count();
    assert_eq!(orders, 1, "P0 orders its two triggers");
    assert_eq!(t.stack_len(), 2, "one trigger for each player of the team");
    (t, doom)
}

#[test]
fn doom_foretold_triggers_for_each_player_of_the_team() {
    cr!("805.4d", "810.9", "603.3b");
    ruling!(
        "Doom Foretold",
        "In a Two-Headed Giant game, Doom Foretold's ability triggers twice during a team's upkeep. In any order you'd like, each of those players has to sacrifice a permanent. The full consequences happen if that player can't, even if you sacrifice Doom Foretold as the first triggered ability is resolving."
    );
    supported("Doom Foretold");
    // P3's trigger on top: P3 can't sacrifice, so P3 discards, the team loses 2, P0 draws,
    // gains 2, creates a Knight and sacrifices Doom Foretold. Then P2's trigger still makes
    // P2 sacrifice its Hill Giant.
    let (mut t, doom) = doom_2hg(Some(vec![0, 1]), Some("Hill Giant"));
    let hand0 = t.hand_size(P0);
    t.resolve();
    assert!(t.in_graveyard(P3, "Grizzly Bears"), "P3 discarded");
    assert!(t.in_graveyard(P0, "Doom Foretold"));
    assert!(!t.on_battlefield(doom));
    assert_eq!(team_lives(&t), (32, 28));
    assert_eq!(t.hand_size(P0), hand0 + 1);
    assert_eq!(tokens_with_subtype(&t, P0, "Knight").len(), 1);
    assert!(t.named_on_battlefield("Hill Giant").len() == 1);
    t.resolve();
    assert!(t.in_graveyard(P2, "Hill Giant"), "P2 still sacrifices");
    assert!(!t.in_graveyard(P2, "Grizzly Bears"), "P2 didn't discard");
    assert_eq!(team_lives(&t), (32, 28));
    assert_eq!(t.hand_size(P0), hand0 + 1);
    assert_eq!(tokens_with_subtype(&t, P0, "Knight").len(), 1);

    // The other order: P2's trigger on top, so P2 sacrifices first.
    let (mut t, _) = doom_2hg(Some(vec![1, 0]), Some("Hill Giant"));
    t.resolve();
    assert!(t.in_graveyard(P2, "Hill Giant"));
    assert!(t.on_battlefield(t.named_on_battlefield("Doom Foretold")[0]));
    assert_eq!(team_lives(&t), (30, 30));
    t.resolve();
    assert!(t.in_graveyard(P3, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Doom Foretold"));
    assert_eq!(team_lives(&t), (32, 28));

    // Neither player can sacrifice: both triggers have their full consequences, the
    // second one even though Doom Foretold is already gone.
    let (mut t, _) = doom_2hg(None, None);
    let hand0 = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P2, "Grizzly Bears") && t.in_graveyard(P3, "Grizzly Bears"));
    assert_eq!(team_lives(&t), (34, 26));
    assert_eq!(t.hand_size(P0), hand0 + 2);
    assert_eq!(tokens_with_subtype(&t, P0, "Knight").len(), 2);
    assert!(t.in_graveyard(P0, "Doom Foretold"));
}

#[test]
fn doom_foretold_a_player_who_can_sacrifice_must() {
    cr!("701.21a", "603.3b");
    ruling!(
        "Doom Foretold",
        "If the player can sacrifice a nonland, nontoken permanent, that player must do so. They can't choose to rush toward their doom."
    );
    supported("Doom Foretold");
    // P1's upkeep: P1 controls Grizzly Bears and a Swamp. P1 tries to choose nothing; the
    // engine offers only the Bears and requires one, so none of the consequences happen.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doom Foretold");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Swamp");
    t.hand(P1, "Grizzly Bears");
    let hand0 = t.hand_size(P0);
    t.answer(P1, DecisionKind::Entities, Answer::Entities(vec![]));
    let from = t.asked().len();
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let asked: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                candidates,
                min,
                max,
                ..
            } if *p == P1 => Some((candidates.clone(), *min, *max)),
            _ => None,
        })
        .collect();
    assert_eq!(asked, vec![(vec![Entity::Object(bears)], 1, 1)]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P1), 1, "no discard");
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(t.hand_size(P0), hand0);
    assert!(t.named_on_battlefield("Doom Foretold").len() == 1);
}
