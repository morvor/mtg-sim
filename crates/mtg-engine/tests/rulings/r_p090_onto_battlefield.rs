//! Rulings batch P090 — cards put onto the battlefield from among the top cards of a
//! library (Saheeli's Directive, Glimpse of Tomorrow, Mitotic Manipulation, Clone Shell,
//! Muxus): simultaneous entry (CR 603.6a, 603.3b), counting (CR 608.2h) and information
//! (CR 406.3, 401.4).

use crate::r_s01_common::*;
use crate::r_s06_common::give_control;
use crate::r_s11_common::empty_library;
use crate::r_s13_common::{commander, commander_game};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::reveal::REVEALED;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The cards revealed this turn.
fn revealed(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == REVEALED => *obj,
            _ => None,
        })
        .collect()
}

#[test]
fn saheelis_directive_artifacts_enter_together_with_their_own_mana_values() {
    cr!("603.6a", "603.2", "202.3");
    ruling!(
        "Saheeli's Directive",
        "All of the artifacts put onto the battlefield this way enter at the same time. If any have triggered abilities that trigger on something else entering the battlefield, they'll see each other."
    );
    ruling!(
        "Saheeli's Directive",
        "Saheeli's Directive considers the individual mana values of the artifact cards you reveal. The sum of their mana values may be greater than X."
    );
    supported("Saheeli's Directive");
    supported("Glint Hawk Idol");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 2);
    // X = 2: two Glint Hawk Idols (mana value 2 each, 4 in all).
    let ids = stack_library(&mut t, P0, &["Glint Hawk Idol", "Glint Hawk Idol"]);
    t.answer_choose(P0, &[Entity::Object(ids[0]), Entity::Object(ids[1])]);
    let spell = t.hand(P0, "Saheeli's Directive");
    t.cast(P0, spell).x(2).go();
    t.resolve();
    let idols = t.named_on_battlefield("Glint Hawk Idol");
    assert_eq!(idols.len(), 2, "{}", t.dump_log());
    // Each saw the other: "Whenever another artifact you control enters" twice.
    assert_eq!(
        triggers_on_stack(&t, "another artifact"),
        2,
        "{}",
        t.dump_log()
    );
}

/// Puts Omniscience onto P0's battlefield and casts Glimpse of Tomorrow from P0's hand
/// without paying its mana cost.
fn cast_glimpse(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Omniscience");
    let spell = t.hand(P0, "Glimpse of Tomorrow");
    t.g.turn.priority = Some(P0);
    t.cast(P0, spell)
        .method(mtg_engine::object::CastMethod::Free)
        .go();
    spell
}

#[test]
fn glimpse_of_tomorrow_triggers_wait_and_go_on_in_apnap_order() {
    cr!("603.3b", "101.4", "603.6a");
    ruling!(
        "Glimpse of Tomorrow",
        "Any abilities that trigger during the resolution of Glimpse of Tomorrow will wait to be put on the stack until everything is put onto the battlefield and resolution is complete. The player whose turn it is puts their triggered abilities on the stack in any order, then each other player in turn order does the same."
    );
    supported("Glimpse of Tomorrow");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    // P0: Soul Warden and Grizzly Bears (and Omniscience); P1: Soul Warden.
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Soul Warden");
    // Nothing goes on the stack while Glimpse of Tomorrow is still resolving.
    let seen = watch(
        &mut t,
        P0,
        |d| {
            matches!(
                d,
                mtg_engine::decision::Decision::ChooseTargets { .. }
                    | mtg_engine::decision::Decision::Order { .. }
            )
        },
        |g| g.stack.len(),
    );
    cast_glimpse(&mut t);
    t.g.resolve_top();
    // (Glimpse of Tomorrow itself is still on the stack while it resolves.)
    assert!(
        seen.lock().unwrap().iter().all(|n| *n <= 1),
        "{:?}",
        seen.lock().unwrap()
    );
    assert_eq!(
        t.named_on_battlefield("Grizzly Bears").len(),
        1,
        "{}",
        t.dump_log()
    );
    assert_eq!(t.stack_len(), 0, "nothing on the stack before the settle");
    t.settle();
    // P0's Soul Warden saw the Bears (and Omniscience isn't a creature): one trigger;
    // P1's saw the Soul Warden and the Bears: two, on top (P1 put theirs on last).
    let ctl: Vec<PlayerId> = t.g.stack.iter().map(|s| t.g.obj(*s).controller).collect();
    assert_eq!(ctl, vec![P0, P1, P1], "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 22);
}

#[test]
fn glimpse_of_tomorrow_counts_a_commander_that_went_to_the_command_zone() {
    cr!("903.9b", "608.2h");
    ruling!(
        "Glimpse of Tomorrow",
        "If a permanent leaves the battlefield this way but ends up in a zone other than a library (most likely because it's a player's commander), it's still counted to determine how many cards to reveal."
    );
    let mut t = commander_game();
    empty_library(&mut t, P0);
    let cmd = commander(&mut t, P0, "Grizzly Bears");
    crate::r_s05_common::move_to(&mut t, cmd, Zone::Battlefield);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.library_top(P0, "Hill Giant");
    // Two permanents (the commander and Omniscience) leave: two cards revealed, though
    // only Omniscience joins the Hill Giant in the library.
    t.answer_yes(P0, true);
    cast_glimpse(&mut t);
    t.resolve();
    assert!(t.g.player(P0).library.is_empty(), "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Omniscience").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.zone(t.g.current(cmd)), Zone::Command);
}

#[test]
fn glimpse_of_tomorrow_counts_tokens_which_dont_affect_the_reveal() {
    cr!("111.8", "704.5d", "608.2h");
    ruling!(
        "Glimpse of Tomorrow",
        "If you own any token permanents, they will also be shuffled into your library. They'll count toward the number of cards you reveal. Even though the tokens will technically be in your library until Glimpse of Tomorrow is finished resolving, they won't affect the shuffle or the cards you reveal."
    );
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    // Two tokens and Omniscience: three cards revealed, from a library of Omniscience,
    // a Hill Giant and a Grizzly Bears (the tokens aren't among them).
    for _ in 0..2 {
        crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    }
    assert_eq!(tokens(&t, P0).len(), 2);
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Grizzly Bears");
    cast_glimpse(&mut t);
    t.resolve();
    assert_eq!(
        t.named_on_battlefield("Hill Giant").len(),
        1,
        "{}",
        t.dump_log()
    );
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Omniscience").len(), 1);
    assert!(tokens(&t, P0).is_empty());
    assert!(t.g.player(P0).library.is_empty());
}

#[test]
fn clone_shell_controlled_by_another_player() {
    cr!("406.3", "708.5", "608.2c");
    ruling!(
        "Clone Shell",
        "If you gain control of another player's Clone Shell, you can't look at the face-down exiled card. However, if Clone Shell dies, you turn that card face up and, if it's a creature card, you put it onto the battlefield under your control."
    );
    supported("Clone Shell");
    let mut t = TestGame::new(2);
    let ids = stack_library(&mut t, P1, &["Hill Giant", "Forest", "Forest", "Forest"]);
    t.answer_choose(P1, &[Entity::Object(ids[0])]);
    let shell = t.enter(P1, "Clone Shell");
    t.settle();
    t.resolve_all();
    let exiled = t.g.current(ids[0]);
    assert_eq!(t.zone(exiled), Zone::Exile, "{}", t.dump_log());
    assert!(t.obj_now(exiled).face_down);
    assert!(mtg_engine::facedown::can_look_at(&t.g, P1, exiled));
    give_control(&mut t, shell, P0);
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, exiled));
    crate::r_s02_common::destroy(&mut t, shell);
    t.settle();
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(giant[0]).controller, P0);
    assert_eq!(t.obj_now(giant[0]).owner, P1);
}

#[test]
fn mitotic_manipulation_one_card_named_like_a_permanent_now() {
    cr!("608.2c", "201.2a", "401.4");
    ruling!(
        "Mitotic Manipulation",
        "No matter how many of the seven cards have the same name as a permanent, you can put only one of them onto the battlefield."
    );
    ruling!(
        "Mitotic Manipulation",
        "“The same name as a permanent” means the same name as a permanent currently on the battlefield as Mitotic Manipulation resolves."
    );
    ruling!(
        "Mitotic Manipulation",
        "You don’t reveal any of the other cards or the order in which they go to the bottom of your library."
    );
    supported("Mitotic Manipulation");
    // Grizzly Bears and an opponent's Hill Giant on the battlefield; a Serra Angel in a
    // graveyard (not a permanent); an Ornithopter that leaves the battlefield in
    // response.
    for pick in ["both", "Serra Angel", "Ornithopter"] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 3);
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Hill Giant");
        t.graveyard(P1, "Serra Angel");
        let thopter = t.battlefield(P1, "Ornithopter");
        let ids = stack_library(
            &mut t,
            P0,
            &[
                "Grizzly Bears",
                "Hill Giant",
                "Serra Angel",
                "Ornithopter",
                "Shock",
                "Shock",
                "Shock",
            ],
        );
        let spell = t.hand(P0, "Mitotic Manipulation");
        let choice = match pick {
            "both" => vec![Entity::Object(ids[0]), Entity::Object(ids[1])],
            "Serra Angel" => vec![Entity::Object(ids[2])],
            _ => vec![Entity::Object(ids[3])],
        };
        t.answer_choose(P0, &choice);
        t.cast(P0, spell).go();
        crate::r_s02_common::destroy(&mut t, thopter);
        t.resolve();
        let bears = t.named_on_battlefield("Grizzly Bears").len();
        let giants = t.named_on_battlefield("Hill Giant").len();
        match pick {
            // Only one of the two matching cards.
            "both" => assert_eq!(bears + giants, 3, "{}", t.dump_log()),
            // A card in a graveyard, or a permanent that's gone by now, doesn't count.
            "Serra Angel" => assert!(t.named_on_battlefield("Serra Angel").is_empty()),
            _ => assert!(
                t.named_on_battlefield("Ornithopter").is_empty(),
                "{}",
                t.dump_log()
            ),
        }
        // Six or seven cards went to the bottom, none of them revealed.
        assert!(revealed(&t).is_empty(), "{:?}", revealed(&t));
        assert!(t.library_size(P0) >= 6);
    }
}

#[test]
fn muxus_bonus_is_locked_in_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Muxus, Goblin Grandee",
        "The bonus Muxus gets is determined only as its last ability resolves. Once that happens, the bonus won't change later in the turn even if the number of Goblins you control changes."
    );
    supported("Muxus, Goblin Grandee");
    let mut t = TestGame::new(2);
    let muxus = t.battlefield(P0, "Muxus, Goblin Grandee");
    let g1 = t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Raging Goblin");
    crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(muxus, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(muxus), (6, 6));
    crate::r_s02_common::destroy(&mut t, g1);
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Raging Goblin");
    t.g.recompute();
    assert_eq!(t.pt(muxus), (6, 6));
}
