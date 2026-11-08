//! Rulings batch P069 — leaving a multiplayer game (CR 800.4a) after casting another
//! player's cards from exile: the cards a player owns leave with them, the spells and
//! permanents the leaving player controls but doesn't own are exiled, and cards still
//! exiled stay exiled (face down ones stay face down, CR 406.3).

use crate::r_p069_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use crate::r_s22_common::choose_names_when_offered;
use mtg_engine::decision::Answer;
use mtg_engine::facedown;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Runs a scenario twice: once P0 (who cast P1's card from exile) leaves the game and the
/// spell or permanent is exiled; once P1 (its owner) leaves and takes it along.
fn both_ways(setup: impl Fn() -> (TestGame, ObjectId)) {
    let (mut t, card) = setup();
    if t.zone(card) == Zone::Stack {
        p0_leaves_and_its_spell_is_exiled(&mut t, card);
    } else {
        p0_leaves_and_its_permanent_is_exiled(&mut t, card);
    }
    let (mut t, card) = setup();
    p1_leaves_and_takes_the_card(&mut t, card);
}

#[test]
fn opposition_agent_found_card_leaves_with_its_owner() {
    cr!("800.4a");
    ruling!(
        "Opposition Agent",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well."
    );
    // P1's Rampant Growth finds a Forest, which P1 exiles; P0 plays it. When P1 leaves,
    // the Forest (which P1 owns) leaves the game with them.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Opposition Agent");
    t.g.search_finds_by_default = true;
    let forest = t.library_top(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P1, ManaType::G, 2);
    let rg = t.hand(P1, "Rampant Growth");
    t.cast(P1, rg).go();
    t.resolve_all();
    assert!(exiled(&t, forest));
    main0(&mut t);
    t.play_land(P0, t.g.current(forest)).unwrap();
    assert!(t.on_battlefield(forest));
    assert_eq!(t.obj_now(forest).controller, P0);
    p1_leaves_and_takes_the_card(&mut t, forest);
    assert!(t.named_on_battlefield("Forest").is_empty());
}

#[test]
fn hostage_taker_card_is_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Hostage Taker",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spell or permanent cards you control from Hostage Taker's ability are exiled."
    );
    supported("Hostage Taker");
    // As a permanent, and as a spell on the stack.
    for on_stack in [false, true] {
        both_ways(|| {
            let mut t = TestGame::new(3);
            let bears = t.battlefield(P1, "Grizzly Bears");
            t.answer_targets(P0, &[o(bears)]);
            t.enter(P0, "Hostage Taker");
            t.resolve_all();
            assert!(exiled(&t, bears));
            mana(&mut t, P0, ManaType::U, 2);
            let s = cast_now(&mut t, P0, bears, &[]);
            if !on_stack {
                t.resolve_all();
            }
            (t, s)
        });
    }
}

#[test]
fn nicol_bolas_god_pharaoh_card_is_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Nicol Bolas, God-Pharaoh",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spell or permanent cards you control from Nicol Bolas's first ability are exiled."
    );
    supported("Nicol Bolas, God-Pharaoh");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let bolas = t.battlefield(P0, "Nicol Bolas, God-Pharaoh");
        let bears = stack_library(&mut t, P1, &["Forest", "Grizzly Bears"])[1];
        t.activate(P0, bolas, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert!(exiled(&t, bears));
        cast_free(&mut t, P0, bears, &[]);
        t.resolve_all();
        (t, bears)
    });
}

#[test]
fn ashiok_nightmare_muse_spells_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Ashiok, Nightmare Muse",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells and/or permanents you control from Ashiok's last ability are exiled."
    );
    supported("Ashiok, Nightmare Muse");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let a = t.battlefield(P0, "Ashiok, Nightmare Muse");
        t.g.obj_mut(a).counters.insert("loyalty".into(), 7);
        let bears = t.exile(P1, "Grizzly Bears");
        t.answer_choose(P0, &[o(bears)]);
        t.activate(P0, a, 2, &[]).unwrap();
        t.resolve_all();
        (t, bears)
    });
}

#[test]
fn breeches_cards_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Breeches, Brazen Plunderer",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells and/or permanents you control from Breeches's brazen thievery are exiled."
    );
    supported("Breeches, Brazen Plunderer");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let b = t.battlefield(P0, "Breeches, Brazen Plunderer");
        let bears = t.library_top(P1, "Grizzly Bears");
        t.attack(&[(b, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert!(exiled(&t, bears), "{}", t.dump_log());
        t.advance_to(P0, Step::PostcombatMain);
        // Mana as though it were mana of any color.
        mana(&mut t, P0, ManaType::R, 2);
        cast_now(&mut t, P0, bears, &[]);
        t.resolve_all();
        (t, bears)
    });
}

#[test]
fn robber_of_the_rich_cards_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Robber of the Rich",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells and/or permanents you control from Robber of the Rich's effect are exiled."
    );
    supported("Robber of the Rich");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let r = t.battlefield(P0, "Robber of the Rich");
        t.hand(P1, "Island");
        let bears = t.library_top(P1, "Grizzly Bears");
        t.attack(&[(r, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert!(exiled(&t, bears), "{}", t.dump_log());
        t.advance_to(P0, Step::PostcombatMain);
        mana(&mut t, P0, ManaType::R, 2);
        // Left on the stack.
        let s = cast_now(&mut t, P0, bears, &[]);
        (t, s)
    });
}

#[test]
fn abstruse_appropriation_cast_card_is_exiled_and_uncast_one_stays_exiled() {
    cr!("800.4a");
    ruling!(
        "Abstruse Appropriation",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells or permanents you control from Abstruse Appropriation's effect are exiled, and any cards still exiled by it remain exiled."
    );
    supported("Abstruse Appropriation");
    let setup = || {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        for target in [bears, giant] {
            let aa = t.hand(P0, "Abstruse Appropriation");
            mana(&mut t, P0, ManaType::W, 1);
            mana(&mut t, P0, ManaType::B, 1);
            mana(&mut t, P0, ManaType::C, 2);
            t.cast(P0, aa).target(target).go();
            t.resolve_all();
            assert!(exiled(&t, target));
        }
        // Colorless mana as though it were any color.
        mana(&mut t, P0, ManaType::C, 2);
        cast_now(&mut t, P0, bears, &[]);
        t.resolve_all();
        (t, bears, giant)
    };
    both_ways(|| {
        let (t, bears, _) = setup();
        (t, bears)
    });
    let (mut t, _, giant) = setup();
    let exiled_giant = t.g.current(giant);
    concede(&mut t, P0);
    assert_eq!(t.g.current(giant), exiled_giant);
    assert!(exiled(&t, giant));
}

#[test]
fn dead_mans_chest_cards_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Dead Man's Chest",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells or permanents you control from Dead Man's Chest's ability are exiled."
    );
    supported("Dead Man's Chest");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let giant = t.battlefield(P1, "Hill Giant");
        let chest = t.battlefield(P0, "Dead Man's Chest");
        t.g.attach(chest, Entity::Object(giant));
        let bears = t.library_top(P1, "Grizzly Bears");
        t.g.destroy(giant, None);
        t.resolve_all();
        assert!(exiled(&t, bears));
        // Mana of any type.
        mana(&mut t, P0, ManaType::B, 2);
        cast_now(&mut t, P0, bears, &[]);
        t.resolve_all();
        (t, bears)
    });
}

#[test]
fn etali_primal_storm_spells_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Etali, Primal Storm",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells or permanents you control from Etali's ability are exiled."
    );
    supported("Etali, Primal Storm");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let e = t.battlefield(P0, "Etali, Primal Storm");
        let bears = t.library_top(P1, "Grizzly Bears");
        choose_names_when_offered(&mut t, P0, &["Grizzly Bears"]);
        attack_with(&mut t, &[(e, Entity::Player(P1))]);
        t.resolve_all();
        (t, bears)
    });
}

#[test]
fn etali_primal_conqueror_spells_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Etali, Primal Conqueror // Etali, Primal Sickness",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells or permanents you control from Etali’s ability are exiled."
    );
    supported("Etali, Primal Conqueror // Etali, Primal Sickness");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let bears = stack_library(&mut t, P1, &["Forest", "Grizzly Bears"])[1];
        choose_names_when_offered(&mut t, P0, &["Grizzly Bears"]);
        t.enter(P0, "Etali, Primal Conqueror // Etali, Primal Sickness");
        t.resolve_all();
        (t, bears)
    });
}

/// P0 mutates Mindleecher onto its Grizzly Bears in a three-player game: P1's Hill Giant
/// and P2's Serra Angel are exiled face down. Returns (Hill Giant, Serra Angel).
fn mindleecher_3p(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let giant = t.library_top(P1, "Hill Giant");
    let angel = t.library_top(P2, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = t.hand(P0, "Mindleecher");
    mana(t, P0, ManaType::B, 5);
    t.cast(P0, m)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert!(exiled(t, giant) && exiled(t, angel), "{}", t.dump_log());
    assert!(t.obj_now(giant).face_down && t.obj_now(angel).face_down);
    (giant, angel)
}

#[test]
fn mindleecher_spells_are_exiled_and_face_down_cards_stay_face_down_when_you_leave() {
    cr!("800.4a", "406.3");
    ruling!(
        "Mindleecher",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells or permanents you control from Mindleecher are exiled, and any cards remaining in exile remain face down indefinitely. No player may look at them."
    );
    supported("Mindleecher");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let (giant, _) = mindleecher_3p(&mut t);
        mana(&mut t, P0, ManaType::R, 4);
        cast_now(&mut t, P0, giant, &[]);
        t.resolve_all();
        (t, giant)
    });
    // The uncast Serra Angel P2 owns stays exiled face down, and nobody may look at it.
    let mut t = TestGame::new(3);
    let (_, angel) = mindleecher_3p(&mut t);
    let a = t.g.current(angel);
    assert!(facedown::can_look_at(&t.g, P0, a));
    concede(&mut t, P0);
    assert_eq!(t.g.current(angel), a);
    assert!(t.obj_now(angel).face_down);
    for p in [P1, P2] {
        assert!(!facedown::can_look_at(&t.g, p, a));
    }
}

#[test]
fn mnemonic_betrayal_spells_are_exiled_and_cards_stay_exiled_when_you_leave() {
    cr!("800.4a", "603.7");
    ruling!(
        "Mnemonic Betrayal",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells or permanents you control from Mnemonic Betrayal’s effect are exiled. If you leave the game before Mnemonic Betrayal’s delayed triggered ability resolves, the cards remain exiled."
    );
    supported("Mnemonic Betrayal");
    let setup = || {
        let mut t = TestGame::new(3);
        let bears = t.graveyard(P1, "Grizzly Bears");
        let giant = t.graveyard(P1, "Hill Giant");
        let mb = t.hand(P0, "Mnemonic Betrayal");
        mana(&mut t, P0, ManaType::U, 1);
        mana(&mut t, P0, ManaType::B, 2);
        t.cast(P0, mb).go();
        t.resolve_all();
        assert!(exiled(&t, bears) && exiled(&t, giant));
        mana(&mut t, P0, ManaType::U, 2);
        cast_now(&mut t, P0, bears, &[]);
        t.resolve_all();
        (t, bears, giant)
    };
    both_ways(|| {
        let (t, bears, _) = setup();
        (t, bears)
    });
    // P0 leaves with the delayed trigger on the stack: it ceases to exist, and the Hill
    // Giant stays exiled.
    let (mut t, _, giant) = setup();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    concede(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(exiled(&t, giant));
    assert!(!t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn thief_of_sanity_spells_are_exiled_and_face_down_cards_stay_face_down_when_you_leave() {
    cr!("800.4a", "406.3");
    ruling!(
        "Thief of Sanity",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells or permanents you control from Thief of Sanity's effect are exiled, and any of the face-down exiled cards remain face down indefinitely. No player may look at them."
    );
    supported("Thief of Sanity");
    // Thief of Sanity hits P1: P0 exiles Grizzly Bears face down from among the top three.
    let hit = || {
        let mut t = TestGame::new(3);
        let thief = t.battlefield(P0, "Thief of Sanity");
        let bears = stack_library(&mut t, P1, &["Hill Giant", "Grizzly Bears", "Forest"])[1];
        t.answer_choose_moved(P0, &[o(bears)]);
        attack_with(&mut t, &[(thief, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[]);
        t.resolve_all();
        assert!(exiled(&t, bears), "{}", t.dump_log());
        assert!(t.obj_now(bears).face_down);
        (t, bears)
    };
    both_ways(|| {
        let (mut t, bears) = hit();
        t.advance_to(P0, Step::PostcombatMain);
        mana(&mut t, P0, ManaType::U, 2);
        cast_now(&mut t, P0, bears, &[]);
        t.resolve_all();
        (t, bears)
    });
    let (mut t, bears) = hit();
    let b = t.g.current(bears);
    assert!(facedown::can_look_at(&t.g, P0, b));
    concede(&mut t, P0);
    assert_eq!(t.g.current(bears), b);
    assert!(t.obj_now(bears).face_down);
    for p in [P1, P2] {
        assert!(!facedown::can_look_at(&t.g, p, b));
    }
}

#[test]
fn haldan_cards_pako_fetched_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Haldan, Avid Arcanist",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any spells or permanents you control that Pako fetched are exiled."
    );
    supported("Haldan, Avid Arcanist");
    supported("Pako, Arcane Retriever");
    both_ways(|| {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Haldan, Avid Arcanist");
        let pako = t.battlefield(P0, "Pako, Arcane Retriever");
        let stone = t.library_top(P1, "Mind Stone");
        t.attack(&[(pako, Entity::Player(P1))], &[]);
        assert!(exiled(&t, stone), "{}", t.dump_log());
        assert_eq!(t.counters(stone, "fetch"), 1);
        t.advance_to(P0, Step::PostcombatMain);
        mana(&mut t, P0, ManaType::R, 2);
        cast_now(&mut t, P0, stone, &[]);
        t.resolve_all();
        (t, stone)
    });
}

#[test]
fn covetous_urge_card_is_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Covetous Urge",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, the spell or permanent you control from Covetous Urge's effect is exiled."
    );
    supported("Covetous Urge");
    both_ways(|| {
        let mut t = TestGame::new(3);
        let bears = t.hand(P1, "Grizzly Bears");
        let urge = t.hand(P0, "Covetous Urge");
        mana(&mut t, P0, ManaType::U, 4);
        t.answer_choose_moved(P0, &[o(bears)]);
        t.cast(P0, urge).target(P1).go();
        t.resolve_all();
        assert!(exiled(&t, bears));
        mana(&mut t, P0, ManaType::B, 2);
        cast_now(&mut t, P0, bears, &[]);
        t.resolve_all();
        (t, bears)
    });
}
