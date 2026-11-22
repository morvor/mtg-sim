//! Rulings batch P157 — permanents that an opponent gains control of (Rainbow Vale,
//! Humble Defector, Wishclaw Talisman, Pendant of Prosperity, Akroan Horse, Sky Swallower):
//! delayed triggers at the next end step, who chooses the opponent and when, APNAP order,
//! players leaving the game (CR 800.4a), and control changes that outlast their source.

use crate::r_p157_common::*;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` taps Rainbow Vale for mana; whether it was activated.
fn vale_mana(t: &mut TestGame, p: PlayerId, vale: ObjectId) -> bool {
    tap_for_mana(t, p, vale, "Add one mana")
}

/// Moves to the end step of `p`'s turn and puts what triggers there on the stack.
fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

#[test]
fn rainbow_vale_changes_control_only_after_a_turn_it_was_tapped_and_stays_tapped() {
    cr!("603.7a", "613.1b");
    ruling!(
        "Rainbow Vale",
        "Only changes controller at the end of a turn in which it was tapped for mana. It does not change controller if it is not tapped for mana."
    );
    ruling!(
        "Rainbow Vale",
        "Stays in the same tapped/untapped state it is in when it switches controllers."
    );
    supported("Rainbow Vale");
    // Not tapped: no change at the end step.
    let mut t = TestGame::new(2);
    let vale = t.battlefield(P0, "Rainbow Vale");
    end_step(&mut t);
    assert_eq!(ctl(&t, vale), P0);
    // Tapped: P1 gains control of it at the end step, and it is still tapped.
    let mut t = TestGame::new(2);
    let vale = t.battlefield(P0, "Rainbow Vale");
    assert!(vale_mana(&mut t, P0, vale));
    assert_eq!(ctl(&t, vale), P0, "not before the end step");
    end_step(&mut t);
    assert_eq!(ctl(&t, vale), P1);
    assert!(tapped(&t, vale));
    // Untapped as it changes control: it stays untapped.
    let mut t = TestGame::new(2);
    let vale = t.battlefield(P0, "Rainbow Vale");
    assert!(vale_mana(&mut t, P0, vale));
    untap(&mut t, vale);
    end_step(&mut t);
    assert_eq!(ctl(&t, vale), P1);
    assert!(!tapped(&t, vale));
}

#[test]
fn rainbow_vale_tapped_during_an_end_step_changes_at_the_next_turns_end_step() {
    cr!("603.7a", "603.7c", "513.1");
    ruling!(
        "Rainbow Vale",
        "If Rainbow Vale’s ability is activated during an End step, its “at the beginning of the end step” trigger won’t trigger until the End step of the next turn. If its ability is activated again during that turn, its “at the beginning of the end step” ability will trigger twice at the End step of that turn."
    );
    let mut t = TestGame::new(2);
    let vale = t.battlefield(P0, "Rainbow Vale");
    to_end_step(&mut t, P0);
    assert!(vale_mana(&mut t, P0, vale));
    t.resolve_all();
    assert_eq!(ctl(&t, vale), P0, "the end step had already begun");
    // In P1's turn, P0 taps it again (after an untap effect).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(ctl(&t, vale), P0);
    untap(&mut t, vale);
    assert!(vale_mana(&mut t, P0, vale));
    to_end_step(&mut t, P1);
    assert_eq!(triggers_on_stack(&t, "delayed trigger"), 2);
    t.resolve_all();
    assert_eq!(ctl(&t, vale), P1);
}

#[test]
fn rainbow_vale_the_activating_player_chooses_the_opponent_as_the_trigger_resolves() {
    cr!("603.7e", "603.7a", "608.2d");
    ruling!(
        "Rainbow Vale",
        "If control of Rainbow Vale has somehow changed before the “at end of turn” ability resolves, the player who activated the mana ability (not the current controller of Rainbow Vale) chooses one of their opponents to gain control of it."
    );
    ruling!(
        "Rainbow Vale",
        "The opponent is chosen when the “at end of turn” triggered ability resolves."
    );
    // P0 taps it in P1's turn; then P1 gains control of it with Annex.
    let mut t = TestGame::new(3);
    let vale = t.battlefield(P0, "Rainbow Vale");
    t.set_step(P1, Step::PrecombatMain);
    assert!(vale_mana(&mut t, P0, vale));
    cast_at(&mut t, P1, "Annex", &[obj(vale)]);
    t.resolve_all();
    assert_eq!(ctl(&t, vale), P1);
    to_end_step(&mut t, P1);
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(trig).controller, P0, "the activating player's trigger");
    // No opponent has been chosen yet; P0 chooses among P0's opponents as it resolves.
    let m = mark(&t);
    assert!(t.asked().iter().all(|(_, d)| !matches!(d,
        mtg_engine::decision::Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&pl(P2)) && candidates.contains(&pl(P1)))));
    t.answer_choose(P0, &[pl(P2)]);
    t.resolve_all();
    let asked = choices(&t, P0, m, "");
    assert!(
        asked
            .iter()
            .any(|c| c.contains(&pl(P1)) && c.contains(&pl(P2)) && !c.contains(&pl(P0))),
        "P0 chose among their opponents: {asked:?}"
    );
    assert!(choices(&t, P1, m, "").is_empty(), "not the current controller");
    assert_eq!(ctl(&t, vale), P2);
}

#[test]
fn rainbow_vale_tapped_by_two_players_triggers_twice_in_apnap_order() {
    cr!("603.7a", "101.4", "603.3b");
    ruling!(
        "Rainbow Vale",
        "If more than one player manages to tap it for mana in a given turn, then two control-change abilities will trigger at end of the turn. These abilities resolve using the standard order for such things, so the active player’s triggers are put on the stack first, then the opponents’."
    );
    // P1 taps it in P0's turn, then P0 takes it with Annex and taps it too.
    let mut t = TestGame::new(2);
    let vale = t.battlefield(P1, "Rainbow Vale");
    assert!(vale_mana(&mut t, P1, vale));
    cast_at(&mut t, P0, "Annex", &[obj(vale)]);
    t.resolve_all();
    assert_eq!(ctl(&t, vale), P0);
    untap(&mut t, vale);
    assert!(vale_mana(&mut t, P0, vale));
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "delayed trigger"), 2);
    assert_eq!(
        stack_controllers(&t),
        vec![P0, P1],
        "the active player's trigger first"
    );
    // P1's resolves first (P0 gains control), then P0's (P1 gains control).
    t.resolve_all();
    assert_eq!(ctl(&t, vale), P1);
}

#[test]
fn humble_defector_leaves_the_game_with_its_owner() {
    cr!("800.4a", "613.1b");
    ruling!(
        "Humble Defector",
        "If Humble Defector's owner leaves the game, Humble Defector humbly leaves the game along with that player."
    );
    supported("Humble Defector");
    let mut t = TestGame::new(3);
    let d = t.battlefield(P0, "Humble Defector");
    act(&mut t, P0, d, 0, &[pl(P1)]);
    assert_eq!(ctl(&t, d), P1);
    leave(&mut t, P0);
    assert!(!t.on_battlefield(d));
    assert!(!named_here(&t, "Humble Defector"));
    assert!(!t.in_exile("Humble Defector"));
}

#[test]
fn wishclaw_talisman_without_wish_counters_stays_and_cant_be_activated() {
    cr!("602.2b", "118.3", "122.1");
    ruling!(
        "Wishclaw Talisman",
        "Once Wishclaw Talisman runs out of wish counters, it remains on the battlefield. You can't activate its last ability at all."
    );
    supported("Wishclaw Talisman");
    let mut t = TestGame::new(2);
    let w = t.enter(P0, "Wishclaw Talisman");
    t.settle();
    assert_eq!(t.counters(w, "wish"), 3);
    t.g.objects[w.0 as usize].counters.insert("wish".into(), 0);
    t.settle();
    assert!(t.on_battlefield(w));
    lots(&mut t, P0);
    assert!(t.activate(P0, w, 0, &[]).is_err());
    assert!(t.on_battlefield(w));
    assert_eq!(ctl(&t, w), P0);
}

#[test]
fn wishclaw_talisman_opponent_chosen_on_resolution_control_returns_when_they_leave() {
    cr!("608.2d", "800.4a", "613.1b");
    ruling!(
        "Wishclaw Talisman",
        "You choose which opponent gains control of Wishclaw Talisman while its ability is resolving. If that player later leaves the game, you regain control of Wishclaw Talisman."
    );
    let mut t = TestGame::new(3);
    let w = t.enter(P0, "Wishclaw Talisman");
    t.settle();
    let m = mark(&t);
    // The search (any card), then the opponent.
    t.answer(P0, DecisionKind::Entities, mtg_engine::decision::Answer::Default);
    t.answer_choose(P0, &[pl(P2)]);
    act(&mut t, P0, w, 0, &[]);
    assert!(
        choices(&t, P0, m, "")
            .iter()
            .any(|c| c.contains(&pl(P1)) && c.contains(&pl(P2))),
        "P0 chooses among the opponents"
    );
    assert_eq!(ctl(&t, w), P2);
    assert_eq!(t.counters(w, "wish"), 2);
    leave(&mut t, P2);
    assert!(t.on_battlefield(w));
    assert_eq!(ctl(&t, w), P0);
}

#[test]
fn pendant_of_prosperity_leaving_players() {
    cr!("800.4a");
    ruling!(
        "Pendant of Prosperity",
        "In a multiplayer game, if Pendant of Prosperity's owner leaves the game, Pendant of Prosperity leaves the game with them. If Pendant of Prosperity's controller leaves the game, Pendant of Prosperity is exiled (assuming that player wasn't also Pendant of Prosperity's owner)."
    );
    supported("Pendant of Prosperity");
    // Its owner leaves: it leaves the game too.
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[pl(P1)]);
    let p = t.enter(P0, "Pendant of Prosperity");
    t.settle();
    assert_eq!(ctl(&t, p), P1);
    leave(&mut t, P0);
    assert!(!named_here(&t, "Pendant of Prosperity"));
    assert!(!t.in_exile("Pendant of Prosperity"));
    // Its controller (not its owner) leaves: it's exiled.
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[pl(P1)]);
    let p = t.enter(P0, "Pendant of Prosperity");
    t.settle();
    assert_eq!(ctl(&t, p), P1);
    leave(&mut t, P1);
    assert!(!named_here(&t, "Pendant of Prosperity"));
    assert!(t.in_exile("Pendant of Prosperity"));
}

#[test]
fn pendant_of_prosperity_your_own_draws_twice_and_puts_two_lands() {
    cr!("608.2c", "305.2a");
    ruling!(
        "Pendant of Prosperity",
        "If you control your own Pendant of Prosperity and activate its last ability, you'll draw a card, then you may put a land card onto the battlefield, then you'll draw a second card, then you may once again put a land card onto the battlefield."
    );
    ruling!(
        "Pendant of Prosperity",
        "Pendant of Prosperity's effect doesn't count as playing a land. It can put a land card onto the battlefield even if you've already played your land for the turn, and even if it's not your turn."
    );
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Pendant of Prosperity");
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Forest");
    t.play_land(P0, f1).unwrap();
    assert!(t.on_battlefield(f1));
    // The first draw is the Mountain, the second the Swamp.
    let swamp = t.library_top(P0, "Swamp");
    t.library_top(P0, "Mountain");
    let m = mark(&t);
    t.answer_choose_moved(P0, &[obj(f2)]);
    t.answer_choose_moved(P0, &[obj(swamp)]);
    act(&mut t, P0, p, 0, &[]);
    assert!(t.on_battlefield(f2), "a land put after the land play");
    assert!(named_here(&t, "Swamp"), "the card the second draw drew");
    assert!(t.in_hand(P0, "Mountain"));
    let asked = choices(&t, P0, m, "");
    assert_eq!(asked.len(), 2, "{asked:?}");
    let names = |c: &Vec<Entity>| -> Vec<String> {
        let mut v: Vec<String> = c
            .iter()
            .map(|e| match e {
                Entity::Object(o) => t.g.obj(*o).chars.name.to_string(),
                _ => String::new(),
            })
            .collect();
        v.sort();
        v
    };
    // The first choice comes before the second draw: the Swamp isn't in hand yet.
    assert_eq!(names(&asked[0]), vec!["Forest", "Mountain"]);
    assert_eq!(names(&asked[1]), vec!["Mountain", "Swamp"]);
    // P0's Pendant, controlled by P1: P1 activates it in P1's turn and P0 may put a land
    // onto the battlefield then.
    let mut t = TestGame::new(2);
    t.answer_choose(P0, &[pl(P1)]);
    let p = t.enter(P0, "Pendant of Prosperity");
    t.settle();
    assert_eq!(ctl(&t, p), P1);
    t.set_step(P1, Step::PrecombatMain);
    let f = t.hand(P0, "Forest");
    t.answer_choose(P0, &[obj(f)]);
    act(&mut t, P1, p, 0, &[]);
    assert!(t.on_battlefield(f));
    assert_eq!(ctl(&t, f), P0);
}

#[test]
fn akroan_horse_each_opponent_means_its_controllers_opponents() {
    cr!("109.5", "613.1b");
    ruling!(
        "Akroan Horse",
        "In the last ability, “each opponent” refers to opponents of Akroan Horse's controller. In most situations in two-player games, your opponent will control Akroan Horse and you'll put Soldier tokens onto the battlefield."
    );
    supported("Akroan Horse");
    let mut t = TestGame::new(2);
    let h = t.enter(P0, "Akroan Horse");
    t.resolve_all();
    assert_eq!(ctl(&t, h), P1);
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
    assert!(with_subtype(&t, P1, "Soldier").is_empty());
}

#[test]
fn sky_swallower_permanents_dont_return_when_it_leaves() {
    cr!("611.2a", "613.1b");
    ruling!(
        "Sky Swallower",
        "The permanents don't return to your control when Sky Swallower leaves the battlefield."
    );
    supported("Sky Swallower");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    let s = t.enter(P0, "Sky Swallower");
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P1);
    assert_eq!(ctl(&t, s), P0);
    kill(&mut t, s);
    assert_eq!(ctl(&t, bears), P1);
    pass_turn(&mut t);
    assert_eq!(ctl(&t, bears), P1);
}
