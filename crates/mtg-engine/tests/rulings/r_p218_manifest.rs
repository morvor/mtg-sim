//! Rulings batch P218 — manifest (CR 701.40) and manifest dread (CR 701.62): turning
//! manifested cards face up, manifesting several cards one at a time, abilities whose
//! source left, hidden information, leaving the game, and double-faced cards.

use crate::r_s01_common::{custom_card, stack_library, supported, triggers_on_stack, watch};
use crate::r_s02_common::destroy;
use crate::r_s04_common::run_with;
use crate::r_s05_common::enter;
use crate::r_s06_common::attach_new;
use crate::r_s11_common::*;
use crate::r_s13_common::commander_game;
use crate::r_s17_common::transform;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::player_control::can_see;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Face-down permanents `p` controls.
fn face_down(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.face_down)
        .map(|o| o.id)
        .collect()
}

/// A custom creature for `p` with "Whenever one or more creatures you control enter, draw
/// a card." (compiled from that text). Returns it.
fn watcher(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let def = custom_card(
        "Entry Watcher",
        "Creature — Human",
        "{1}",
        Some((1, 1)),
        "Whenever one or more creatures you control enter, draw a card.",
    );
    t.custom(p, def, Zone::Battlefield)
}

/// Casts the real spell `name` from P0's hand (with lands for its cost) and resolves it.
fn cast_resolve(t: &mut TestGame, name: &str, x: Option<i64>, targets: &[Entity]) {
    lands_for_cost(t, P0, name);
    if let Some(x) = x {
        t.lands(P0, "Wastes", 2 * x as usize);
    }
    let card = t.hand(P0, name);
    let mut b = t.cast(P0, card).targets(targets);
    if let Some(x) = x {
        b = b.x(x);
    }
    b.go();
    t.resolve_all();
}

#[test]
fn soul_strike_technique_manifest_turns_face_up_as_a_special_action() {
    cr!("701.40b", "116.2b", "708.8");
    ruling!(
        "Soul-Strike Technique",
        "Any time you have priority, you may turn a manifested creature face up by revealing that it’s a creature card (ignoring any type-changing effects that might be applying to it) and paying its mana cost. This is a special action. It doesn’t use the stack and can’t be responded to."
    );
    supported("Soul-Strike Technique");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Soul-Strike Technique", bears);
    let giant = t.library_top(P0, "Hill Giant");
    destroy(&mut t, bears);
    t.resolve_all();
    let m = t.g.current(giant);
    assert!(is_plain_face_down_2_2(&t, m));
    // A type-changing effect makes it a noncreature land: it can still be turned up.
    run_with(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![CardType::Land],
                subtypes: vec![],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(m)],
    );
    assert!(!t.obj(m).chars.is(CardType::Creature));
    t.lands(P0, "Mountain", 4);
    let stack = t.stack_len();
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.stack_len(), stack);
    assert_eq!(t.obj(m).chars.name.as_str(), "Hill Giant");
}

#[test]
fn glitch_interpreter_checks_for_face_down_permanents_twice() {
    cr!("603.4", "701.62a");
    ruling!(
        "Glitch Interpreter",
        "Glitch Interpreter's first ability won't trigger at all if you control one or more face-down permanents when it enters."
    );
    supported("Glitch Interpreter");
    // A face-down permanent as it enters: no trigger.
    let mut t = TestGame::new(2);
    manifest_card(&mut t, P0, "Forest");
    enter(&mut t, P0, "Glitch Interpreter");
    assert_eq!(t.stack_len(), 0);
    // One manifested in response: the ability does nothing.
    let mut t = TestGame::new(2);
    let gi = enter(&mut t, P0, "Glitch Interpreter");
    assert_eq!(t.stack_len(), 1);
    manifest_card(&mut t, P0, "Forest");
    stack_library(&mut t, P0, &["Island", "Swamp"]);
    let library = t.library_size(P0);
    t.resolve_all();
    assert!(t.on_battlefield(gi));
    assert_eq!(face_down(&t, P0).len(), 1);
    assert_eq!(t.library_size(P0), library);
}

#[test]
fn glitch_interpreter_gone_still_manifests_dread() {
    cr!("608.2b", "701.62a");
    ruling!(
        "Glitch Interpreter",
        "If Glitch Interpreter is no longer on the battlefield when its first ability resolves and you control no face-down permanents, you'll still manifest dread."
    );
    let mut t = TestGame::new(2);
    let gi = enter(&mut t, P0, "Glitch Interpreter");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, gi);
    let cards = stack_library(&mut t, P0, &["Island", "Swamp"]);
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    assert_eq!(face_down(&t, P0).len(), 1);
    assert!(t.in_graveyard(P0, "Glitch Interpreter"));
}

#[test]
fn jeskai_infiltrator_gone_manifests_only_the_top_card() {
    cr!("701.40a", "608.2b");
    ruling!(
        "Jeskai Infiltrator",
        "If Jeskai Infiltrator isn't on the battlefield as its triggered ability resolves, you'll manifest just the top card of your library."
    );
    supported("Jeskai Infiltrator");
    let mut t = TestGame::new(2);
    let infiltrator = t.battlefield(P0, "Jeskai Infiltrator");
    let top = t.library_top(P0, "Hill Giant");
    crate::r_s01_common::attack_with(&mut t, &[(infiltrator, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "exile it and the top card"), 1);
    destroy(&mut t, infiltrator);
    t.resolve_all();
    assert_eq!(face_down(&t, P0), vec![t.g.current(top)]);
    assert!(t.in_graveyard(P0, "Jeskai Infiltrator"));
}

#[test]
fn manifesting_several_cards_manifests_them_one_at_a_time() {
    cr!("701.40e", "603.2c");
    ruling!(
        "Omarthis, Ghostfire Initiate",
        "If Omarthis, Ghostfire Initiate's last ability causes its controller to manifest multiple cards, those cards are manifested one at a time."
    );
    ruling!(
        "Valgavoth's Onslaught",
        "If an effect instructs a player to manifest multiple cards from their library, those cards are manifested one at a time. Players can't take actions in between. However, an ability that triggers \"Whenever one or more creatures enter\" would trigger once for each event."
    );
    supported("Omarthis, Ghostfire Initiate");
    supported("Valgavoth's Onslaught");
    // Omarthis dies with two +1/+1 counters: two manifests, two "one or more" triggers.
    let mut t = TestGame::new(2);
    let w = watcher(&mut t, P0);
    let omarthis = t.battlefield(P0, "Omarthis, Ghostfire Initiate");
    t.g.add_counters(Entity::Object(omarthis), counters::PLUS1, 2, None);
    destroy(&mut t, omarthis);
    t.resolve_all();
    assert_eq!(face_down(&t, P0).len(), 2);
    assert_eq!(triggered_from(&t, w), 2);
    // Valgavoth's Onslaught for X = 2: two manifest dread events.
    let mut t = TestGame::new(2);
    let w = watcher(&mut t, P0);
    cast_resolve(&mut t, "Valgavoth's Onslaught", Some(2), &[]);
    assert_eq!(face_down(&t, P0).len(), 2);
    assert_eq!(triggered_from(&t, w), 2);
}

#[test]
fn valgavoths_onslaught_no_actions_before_the_counters() {
    cr!("608.2c", "117.1");
    ruling!(
        "Valgavoth's Onslaught",
        "Players can't take actions in between the time you manifest dread X times and the time you put +1/+1 counters on them."
    );
    // Whenever anyone gets priority, no face-down creature lacks its counters.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Valgavoth's Onslaught");
    t.lands(P0, "Forest", 4);
    let seen: Vec<_> = [P0, P1]
        .into_iter()
        .map(|p| {
            watch(
                &mut t,
                p,
                |d| matches!(d, Decision::Priority { .. }),
                |g| {
                    g.permanents()
                        .filter(|o| o.face_down && o.counter(counters::PLUS1) == 0)
                        .count()
                },
            )
        })
        .collect();
    let spell = t.hand(P0, "Valgavoth's Onslaught");
    t.cast(P0, spell).x(2).go();
    let ok = t.g.run_until(10_000, |g| g.stack.is_empty());
    assert!(ok);
    let fd = face_down(&t, P0);
    assert_eq!(fd.len(), 2);
    for m in fd {
        assert_eq!(t.obj(m).counter(counters::PLUS1), 2);
        assert_eq!(t.pt(m), (4, 4));
    }
    for s in seen {
        let s = s.lock().unwrap();
        assert!(!s.is_empty() && s.iter().all(|n| *n == 0), "{s:?}");
    }
}

#[test]
fn unnerving_grasp_with_an_illegal_target_doesnt_manifest_dread() {
    cr!("608.2b", "701.62a");
    ruling!(
        "Unnerving Grasp",
        "If the target nonland permanent is an illegal target when Unnerving Grasp tries to resolve, it won't resolve and none of its effects will happen. You won't manifest dread."
    );
    supported("Unnerving Grasp");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Unnerving Grasp");
    let grasp = t.hand(P0, "Unnerving Grasp");
    t.cast(P0, grasp).target(bears).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(face_down(&t, P0).is_empty());
    assert!(t.in_graveyard(P0, "Unnerving Grasp"));
    // With its target still there: it's returned and P0 manifests dread.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, "Unnerving Grasp", None, &[Entity::Object(bears)]);
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(face_down(&t, P0).len(), 1);
}

#[test]
fn wildcall_for_zero_just_manifests_the_top_card() {
    cr!("107.3b", "701.40a");
    ruling!(
        "Wildcall",
        "If you choose 0 for X, you'll just manifest the top card of your library."
    );
    supported("Wildcall");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Hill Giant");
    cast_resolve(&mut t, "Wildcall", Some(0), &[]);
    let m = t.g.current(top);
    assert_eq!(face_down(&t, P0), vec![m]);
    assert!(is_plain_face_down_2_2(&t, m));
    assert_eq!(t.obj(m).counter(counters::PLUS1), 0);
    assert_eq!(t.pt(m), (2, 2));
}

#[test]
fn ethereal_ambush_manifests_the_top_two_cards_one_at_a_time() {
    cr!("701.40e", "401.5", "708.6");
    ruling!(
        "Ethereal Ambush",
        "If you're playing with the top card of your library revealed as Ethereal Ambush resolves (perhaps because you control a card such as Courser of Kruphix), you'll manifest the top card, reveal the next card (now the top card), and then manifest that card."
    );
    ruling!(
        "Ethereal Ambush",
        "The cards are manifested one at a time. It must remain clear which face-down creature was the top card of your library and which one was the second card of your library."
    );
    supported("Ethereal Ambush");
    supported("Courser of Kruphix");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Courser of Kruphix");
    let w = watcher(&mut t, P0);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Forest", "Island"]);
    cast_resolve(&mut t, "Ethereal Ambush", None, &[]);
    // Each manifested permanent is the card it was; the top one entered first.
    let first = t.g.current(cards[0]);
    let second = t.g.current(cards[1]);
    assert!(t.obj(first).face_down && t.obj(second).face_down);
    assert_eq!(face_down(&t, P0).len(), 2);
    assert!(t.obj(first).timestamp < t.obj(second).timestamp);
    assert_eq!(triggered_from(&t, w), 2);
}

#[test]
fn write_into_being_manifests_before_the_other_card_is_placed() {
    cr!("701.40a", "401.5");
    ruling!(
        "Write into Being",
        "If you're playing with the top card of your library revealed, you'll manifest one of the cards, then the other one will be revealed, then you can choose to put that card on the bottom of your library or leave it on top."
    );
    ruling!(
        "Write into Being",
        "Other players won't know whether the card you manifest is the top card or second card of that library."
    );
    supported("Write into Being");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Courser of Kruphix");
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Forest"]);
    // P0 manifests the second card (the Forest).
    t.answer_choose(P0, &[Entity::Object(cards[1])]);
    // When asked where the other card goes, the manifest has happened.
    let seen = watch(
        &mut t,
        P0,
        |d| !matches!(d, Decision::Priority { .. } | Decision::ChooseEntities { .. }),
        |g| g.permanents().filter(|o| o.face_down).count(),
    );
    cast_resolve(&mut t, "Write into Being", None, &[]);
    let m = t.g.current(cards[1]);
    assert_eq!(face_down(&t, P0), vec![m]);
    assert!(!seen.lock().unwrap().is_empty());
    assert!(seen.lock().unwrap().iter().all(|n| *n == 1));
    // P1 can't see which card it is; P0 can.
    assert!(!can_see(&t.g, P1, m));
    assert!(can_see(&t.g, P0, m));
    assert!(t.zone(cards[0]) == Zone::Library(P0));
}

#[test]
fn thieving_amalgam_manifests_are_exiled_when_their_controller_leaves() {
    cr!("800.4a", "701.40a");
    ruling!(
        "Thieving Amalgam",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, the creatures you manifested with Thieving Amalgam's triggered ability are exiled."
    );
    supported("Thieving Amalgam");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Thieving Amalgam");
    let top = t.library_top(P1, "Hill Giant");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let m = t.g.current(top);
    assert!(t.obj(m).face_down && t.obj(m).controller == P0 && t.on_battlefield(m));
    t.g.player_loses(P0);
    t.settle();
    assert_eq!(t.zone(m), Zone::Exile);
    assert_eq!(t.obj_now(m).owner, P1);
}

#[test]
fn thieving_amalgam_commander_can_turn_up_off_color_cards() {
    cr!("903.4", "701.40b", "106.1b");
    ruling!(
        "Thieving Amalgam",
        "In the Commander variant, you can produce mana that isn't of your commander's color identity if an effect lets you produce mana of that color or mana of any color"
    );
    supported("Lotus Petal");
    // P0's commander is Thieving Amalgam (black). P0 manifests P1's Llanowar Elves and
    // turns it face up with green mana from Lotus Petal.
    let mut t = commander_game();
    crate::r_s13_common::commander(&mut t, P0, "Thieving Amalgam");
    t.battlefield(P0, "Thieving Amalgam");
    t.battlefield(P0, "Lotus Petal");
    let elves = t.library_top(P1, "Llanowar Elves");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let m = t.g.current(elves);
    assert!(t.obj(m).face_down && t.obj(m).controller == P0);
    assert!(can_turn_face_up(&mut t, P0, m));
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Llanowar Elves");
    assert!(t.in_graveyard(P0, "Lotus Petal"));
}

#[test]
fn under_the_skin_may_return_the_card_milled_by_dread() {
    cr!("701.62a", "608.2c");
    ruling!(
        "Under the Skin",
        "The permanent card can be one you milled while manifesting dread, but it doesn't have to be."
    );
    supported("Under the Skin");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Forest"]);
    // Manifest the Forest; the Hill Giant goes to the graveyard and is returned.
    t.answer_choose(P0, &[Entity::Object(cards[1])]);
    t.answer_yes(P0, true);
    cast_resolve(&mut t, "Under the Skin", None, &[]);
    assert_eq!(face_down(&t, P0), vec![t.g.current(cards[1])]);
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn orochi_soul_reaver_owner_cant_look_at_the_manifested_card() {
    cr!("708.5", "701.40a");
    ruling!(
        "Orochi Soul-Reaver",
        "Your opponents can’t look at the card they own that you manifested."
    );
    supported("Orochi Soul-Reaver");
    let mut t = TestGame::new(2);
    let orochi = t.battlefield(P0, "Orochi Soul-Reaver");
    let top = t.library_top(P1, "Hill Giant");
    t.attack(&[(orochi, Entity::Player(P1))], &[]);
    let m = t.g.current(top);
    assert!(t.obj(m).face_down && t.on_battlefield(m) && t.obj(m).controller == P0);
    assert_eq!(t.obj(m).owner, P1);
    assert!(!can_see(&t.g, P1, m));
    assert!(can_see(&t.g, P0, m));
}

#[test]
fn a_manifested_double_faced_card_cant_transform_and_turns_up_front_face() {
    cr!("712.15", "712.15a", "701.40b");
    ruling!(
        "Omarthis, Ghostfire Initiate",
        "If a double-faced card is manifested, it will be put onto the battlefield face down. While face down, a transforming double-faced card can't transform or convert."
    );
    let mut t = TestGame::new(2);
    let m = manifest_card(&mut t, P0, "Delver of Secrets // Insectile Aberration");
    assert!(is_plain_face_down_2_2(&t, m));
    transform(&mut t, m);
    assert!(is_plain_face_down_2_2(&t, m));
    t.lands(P0, "Island", 1);
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Delver of Secrets");
    assert_eq!(t.pt(m), (1, 1));
}
