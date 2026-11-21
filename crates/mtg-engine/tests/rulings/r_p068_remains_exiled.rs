//! Rulings batch P068 — what happens to exiled cards that aren't cast: they stay in exile
//! once the permission ends (CR 611.2a, 400.7), a land card can't be played under a
//! permission to cast (CR 305.1, 601.3), and "exile until" effects that run out of cards
//! (CR 701.20, 401).

use crate::r_p068_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Runs the game to P0's next turn's precombat main phase.
fn next_own_turn(t: &mut TestGame) {
    t.g.stack.clear();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
}

/// Asserts that `card` is still in exile and P0 can't cast it (or play it), with plenty of
/// mana in its main phase.
fn stays_exiled_unplayable(t: &mut TestGame, card: ObjectId) {
    assert!(exiled(t, card), "{}", t.dump_log());
    t.set_step(P0, Step::PostcombatMain);
    for ty in [W, U, B, R, G] {
        mana(t, P0, ty, 2);
    }
    mana(t, P0, C, 6);
    assert!(!castable(t, P0, card), "{}", t.dump_log());
    assert!(!playable_land(t, P0, card));
}

// ---------------------------------------------------------------------------
// "Until end of turn": not cast, it stays exiled
// ---------------------------------------------------------------------------

/// `attacker` (P0's) deals combat damage to P1 unblocked; triggers resolve.
fn hit(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
    t.resolve_all();
}

#[test]
fn breeches_thada_adel_and_knacksaw_clique_cards_not_played_stay_exiled() {
    cr!("611.2a");
    ruling!(
        "Breeches, Brazen Plunderer",
        "If you don't play one of the exiled cards, it remains exiled. It won't be available to be played on future turns."
    );
    ruling!(
        "Thada Adel, Acquisitor",
        "If you don't play the card before the turn ends, it simply remains exiled."
    );
    ruling!(
        "Knacksaw Clique",
        "If you don’t play the exiled card that turn, it remains exiled but you can’t play it."
    );
    for name in [
        "Breeches, Brazen Plunderer",
        "Thada Adel, Acquisitor",
        "Knacksaw Clique",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let source = t.battlefield(P0, name);
        let card = match name {
            "Thada Adel, Acquisitor" => {
                let orni = t.library_top(P1, "Ornithopter");
                t.answer_choose(P0, &[o(orni)]);
                hit(&mut t, source);
                orni
            }
            "Knacksaw Clique" => {
                let div = t.library_top(P1, "Divination");
                t.g.obj_mut(source).tapped = true;
                pool(&mut t, P0, &[U, C]);
                t.activate(P0, source, 0, &[Entity::Player(P1)]).unwrap();
                t.resolve_all();
                div
            }
            _ => {
                let div = t.library_top(P1, "Divination");
                hit(&mut t, source);
                div
            }
        };
        assert!(exiled(&t, card), "{name}: {}", t.dump_log());
        // Playable this turn.
        t.set_step(P0, Step::PostcombatMain);
        for ty in [U, R, C] {
            mana(&mut t, P0, ty, 3);
        }
        assert!(castable(&mut t, P0, card), "{name}: {}", t.dump_log());
        next_own_turn(&mut t);
        stays_exiled_unplayable(&mut t, card);
    }
}

#[test]
fn grenzo_card_not_cast_stays_exiled_and_a_land_cant_be_played() {
    cr!("611.2a", "305.1", "601.3");
    ruling!(
        "Grenzo, Havoc Raiser",
        "If you don't cast the card, it will remain in exile."
    );
    ruling!(
        "Grenzo, Havoc Raiser",
        "If you exile a land card this way, you can't play it."
    );
    supported("Grenzo, Havoc Raiser");
    for top in ["Divination", "Forest"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grenzo, Havoc Raiser");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let card = t.library_top(P1, top);
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        hit(&mut t, bears);
        assert!(exiled(&t, card), "{}", t.dump_log());
        t.set_step(P0, Step::PostcombatMain);
        mana(&mut t, P0, B, 3);
        if top == "Forest" {
            assert!(!playable_land(&mut t, P0, card));
            continue;
        }
        assert!(castable(&mut t, P0, card));
        next_own_turn(&mut t);
        stays_exiled_unplayable(&mut t, card);
    }
}

#[test]
fn daxos_land_card_no_life_and_cant_be_played() {
    cr!("202.3a", "305.1", "601.3");
    ruling!(
        "Daxos of Meletis",
        "If the exiled card is a land card, you won't gain any life and you won't be able to play the land."
    );
    supported("Daxos of Meletis");
    let mut t = TestGame::new(2);
    let daxos = t.battlefield(P0, "Daxos of Meletis");
    let forest = t.library_top(P1, "Forest");
    hit(&mut t, daxos);
    assert!(exiled(&t, forest));
    assert_eq!(t.life(P0), 20);
    t.set_step(P0, Step::PostcombatMain);
    assert!(!playable_land(&mut t, P0, forest));
}

#[test]
fn stolen_goods_card_not_cast_stays_exiled_and_an_all_land_library_is_exiled() {
    cr!("611.2a", "609.3");
    ruling!(
        "Stolen Goods",
        "If you can't cast the nonland card (perhaps because there are no legal targets available) or if you choose not to cast it, it will remain exiled."
    );
    ruling!(
        "Stolen Goods",
        "If there are no nonland cards left in that library, the opponent will exile all cards from their library, and Stolen Goods will finish resolving."
    );
    supported("Stolen Goods");
    // Grizzly Bears, not cast: exiled after the turn.
    let mut t = TestGame::new(2);
    let bears = t.library_top(P1, "Grizzly Bears");
    let sg = t.hand(P0, "Stolen Goods");
    pool(&mut t, P0, &[U, C, C, C]);
    t.cast(P0, sg).target(P1).go();
    t.resolve_all();
    assert!(exiled(&t, bears));
    next_own_turn(&mut t);
    stays_exiled_unplayable(&mut t, bears);
    // Only lands in P1's library: all exiled; Stolen Goods resolves.
    let mut t = TestGame::new(2);
    t.g.players[1].library.clear();
    let lands = stack_library(&mut t, P1, &["Forest", "Island", "Swamp"]);
    let sg = t.hand(P0, "Stolen Goods");
    pool(&mut t, P0, &[U, C, C, C]);
    t.cast(P0, sg).target(P1).go();
    t.resolve_all();
    assert!(lands.iter().all(|l| exiled(&t, *l)), "{}", t.dump_log());
    assert_eq!(t.library_size(P1), 0);
    assert!(t.in_graveyard(P0, "Stolen Goods"));
    assert!(!t.has_lost(P1));
}

#[test]
fn jace_card_not_cast_stays_exiled() {
    cr!("608.2g", "611.2a");
    ruling!(
        "Jace, Architect of Thought",
        "If you can't cast a card, perhaps because there are no legal targets available, or if you choose not to cast one, it will remain exiled. Jace's ability won't allow you to cast it later."
    );
    supported("Jace, Architect of Thought");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Architect of Thought");
    t.g.obj_mut(jace).counters.insert("loyalty".into(), 8);
    let bears = t.library_top(P0, "Grizzly Bears");
    // Smother ("Destroy target creature with mana value 3 or less"), with no creature.
    let smother = t.library_top(P1, "Smother");
    t.answer_choose(P0, &[o(bears)]);
    t.answer_choose(P0, &[o(smother)]);
    // Bears first: declined; Smother: no legal target.
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.activate(P0, jace, 2, &[]).unwrap();
    t.resolve_all();
    stays_exiled_unplayable(&mut t, bears);
    assert!(exiled(&t, smother));
}

#[test]
fn epic_experiment_cards_not_cast_go_to_the_graveyard() {
    cr!("608.2g", "608.2c");
    ruling!(
        "Epic Experiment",
        "If you can't cast a card, perhaps because there are no legal targets available, or if you choose not to cast it, it will be put into your graveyard."
    );
    supported("Epic Experiment");
    // X = 3: Lightning Bolt (declined), Smother (no creature to target) and Hill Giant.
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Smother", "Hill Giant"]);
    let ee = t.hand(P0, "Epic Experiment");
    pool(&mut t, P0, &[U, R, C, C, C]);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_choose_moved(P0, &[]);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.cast(P0, ee).go();
    t.resolve_all();
    for c in &lib {
        assert_eq!(t.zone(*c), Zone::Graveyard(P0), "{}", t.dump_log());
    }
    assert_eq!(t.life(P1), 20);
}

#[test]
fn grima_card_not_cast_goes_to_the_bottom_with_the_others() {
    cr!("608.2c");
    ruling!(
        "Gríma, Saruman's Footman",
        "If you choose not to cast the card, it is put on the bottom of its owner's library in a random order along with the other exiled cards."
    );
    supported("Gríma, Saruman's Footman");
    let mut t = TestGame::new(2);
    let grima = t.battlefield(P0, "Gríma, Saruman's Footman");
    let lib = stack_library(&mut t, P1, &["Hill Giant", "Divination"]);
    t.answer_yes(P0, false);
    hit(&mut t, grima);
    let n = t.library_size(P1);
    for c in &lib {
        let pos = library_pos(&t, P1, *c).unwrap_or_else(|| panic!("{}", t.dump_log()));
        assert!(pos >= n - 2, "on the bottom");
    }
}

#[test]
fn the_infamous_cruelclaw_card_not_cast_stays_exiled() {
    cr!("608.2g");
    ruling!(
        "The Infamous Cruelclaw",
        "If you choose not to cast the exiled card, it remains in exile."
    );
    supported("The Infamous Cruelclaw");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "The Infamous Cruelclaw");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.hand(P0, "Island");
    t.answer_yes(P0, false);
    hit(&mut t, c);
    stays_exiled_unplayable(&mut t, bears);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn primordial_mist_card_not_played_this_turn_stays_exiled() {
    cr!("611.2a", "708.9");
    ruling!(
        "Primordial Mist",
        "If you activate Primordial Mist's last ability but don't play the exiled card this turn, it remains exiled."
    );
    supported("Primordial Mist");
    let mut t = TestGame::new(2);
    let mist = t.battlefield(P0, "Primordial Mist");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let fd = t.g.current(bears);
    assert!(t.obj_now(fd).face_down, "{}", t.dump_log());
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[o(fd)]);
    t.activate(P0, mist, 0, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, bears));
    mana(&mut t, P0, G, 2);
    assert!(castable(&mut t, P0, bears));
    next_own_turn(&mut t);
    stays_exiled_unplayable(&mut t, bears);
}

#[test]
fn primordial_mist_can_exile_a_face_down_token_which_cant_be_cast() {
    cr!("111.7", "704.5d", "708.9");
    ruling!(
        "Primordial Mist",
        "If you somehow control a face-down token, you may exile it to activate Primordial Mist's last ability, but you won't be able to cast that token."
    );
    let mut t = TestGame::new(2);
    let mist = t.battlefield(P0, "Primordial Mist");
    // Raise the Alarm's Soldier, turned face down by Cyber Conversion.
    let ra = t.hand(P0, "Raise the Alarm");
    pool(&mut t, P0, &[W, C]);
    t.cast(P0, ra).go();
    t.resolve_all();
    let soldier = t.named_on_battlefield("Soldier Token")[0];
    let cc = t.hand(P0, "Cyber Conversion");
    pool(&mut t, P0, &[U, U, C]);
    t.cast(P0, cc).target(soldier).go();
    t.resolve_all();
    assert!(t.obj_now(soldier).face_down, "{}", t.dump_log());
    t.answer_choose(P0, &[o(soldier)]);
    t.activate(P0, mist, 0, &[]).expect("the token can be exiled for the cost");
    t.resolve_all();
    // The token ceased to exist in exile: nothing to cast.
    assert!(!t.g.exile.iter().any(|x| t.g.obj(*x).is_token()), "{}", t.dump_log());
    assert!(t.g.stack.is_empty());
}

// ---------------------------------------------------------------------------
// "Exile until" with nothing to find
// ---------------------------------------------------------------------------

#[test]
fn chaos_wand_without_an_instant_or_sorcery_puts_the_library_back() {
    cr!("609.3");
    ruling!(
        "Chaos Wand",
        "If the opponent exiles their entire library without exiling an instant or sorcery card, that player stops exiling cards, and the exiled cards are put into their library in a random order."
    );
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Chaos Wand");
    let n = t.library_size(P1);
    let bears = t.library_top(P1, "Grizzly Bears");
    mana(&mut t, P0, C, 4);
    t.activate(P0, w, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P1), n + 1, "{}", t.dump_log());
    assert!(in_library(&t, bears, P1));
    assert!(t.g.exile.is_empty());
    assert!(t.g.stack.is_empty());
}

#[test]
fn tibalts_trickery_without_a_different_nonland_card_puts_them_on_the_bottom() {
    cr!("609.3", "701.17a");
    ruling!(
        "Tibalt's Trickery",
        "If there aren't any nonland cards in that library with a different name than the countered spell, the player will exile all cards from their library, revealing all of them, and then they'll put those cards on the bottom of their library in a random order."
    );
    supported("Tibalt's Trickery");
    let mut t = TestGame::new(2);
    t.g.players[1].library.clear();
    stack_library(
        &mut t,
        P1,
        &["Forest", "Forest", "Forest", "Lightning Bolt", "Island"],
    );
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, R, 1);
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, bolt).target(P0).go();
    let tt = t.hand(P0, "Tibalt's Trickery");
    mana(&mut t, P0, R, 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, tt).target(spell).go();
    t.resolve_all();
    // 1 to 3 Forests milled; the rest exiled and put back.
    let milled = t.graveyard_size(P1) - 1;
    assert!((1..=3).contains(&milled), "{}", t.dump_log());
    assert_eq!(t.library_size(P1), 5 - milled);
    assert!(t.g.exile.is_empty());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn unpredictable_cyclone_without_a_shared_type_draws_nothing() {
    cr!("609.3", "702.29a", "614.1a");
    ruling!(
        "Unpredictable Cyclone",
        "If you exile your entire library without exiling a card that shares a card type, you put the library back in a random order and continue. You won’t draw a card."
    );
    supported("Unpredictable Cyclone");
    // Censor (an instant with cycling {U}); the library has no instants.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unpredictable Cyclone");
    let n = t.library_size(P0);
    let censor = t.hand(P0, "Censor");
    mana(&mut t, P0, U, 1);
    t.activate(P0, censor, 0, &[]).expect("cycling");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0, "{}", t.dump_log());
    assert_eq!(t.library_size(P0), n);
    assert!(t.g.exile.is_empty());
}

#[test]
fn codie_without_a_lesser_instant_or_sorcery_returns_the_library_and_ends() {
    cr!("609.3", "603.7");
    ruling!(
        "Codie, Vociferous Codex",
        "If you exile your entire library without exiling an instant or sorcery card with a lesser mana value, you will randomize the exiled cards and they again become your library, ending the effect. You will not continue to exile and shuffle your library forever."
    );
    supported("Codie, Vociferous Codex");
    let mut t = TestGame::new(2);
    let codie = t.battlefield(P0, "Codie, Vociferous Codex");
    let n = t.library_size(P0);
    mana(&mut t, P0, C, 4);
    t.activate(P0, codie, 0, &[]).unwrap();
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.library_size(P0), n, "{}", t.dump_log());
    assert!(t.g.exile.is_empty());
    assert_eq!(t.life(P1), 17);
}

#[test]
fn emergent_ultimatum_finding_one_card_casts_nothing() {
    cr!("701.23", "608.2c");
    ruling!(
        "Emergent Ultimatum",
        "If you find only one card, you’ll shuffle it into your library and exile Emergent Ultimatum without casting anything."
    );
    supported("Emergent Ultimatum");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let eu = t.hand(P0, "Emergent Ultimatum");
    pool(&mut t, P0, &[B, B, G, G, G, U, U]);
    t.answer_choose(P0, &[o(bears)]);
    t.cast(P0, eu).go();
    t.resolve_all();
    assert!(in_library(&t, bears, P0), "{}", t.dump_log());
    assert!(t.in_exile("Emergent Ultimatum"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn invasion_of_alara_with_only_one_card_casts_it_or_puts_it_into_hand() {
    cr!("609.3");
    ruling!(
        "Invasion of Alara // Awaken the Maelstrom",
        "If you exile only one nonland card with mana value 4 or less, you'll have the option to cast it. If you don't, you'll put it into your hand."
    );
    supported("Invasion of Alara // Awaken the Maelstrom");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        // The only nonland card of mana value 4 or less is Grizzly Bears (Craw Wurm: 6).
        t.g.players[0].library.clear();
        let lib = stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Craw Wurm"]);
        let cast: Vec<Entity> = if yes { vec![o(lib[1])] } else { vec![] };
        t.answer_choose(P0, &[Entity::Player(P1)]);
        t.answer_choose_moved(P0, &cast);
        t.enter(P0, "Invasion of Alara // Awaken the Maelstrom");
        t.resolve_all();
        let bears = t.g.current(lib[1]);
        if yes {
            assert!(t.on_battlefield(bears), "{}", t.dump_log());
        } else {
            assert_eq!(t.zone(bears), Zone::Hand(P0), "{}", t.dump_log());
        }
        assert!(in_library(&t, lib[0], P0) && in_library(&t, lib[2], P0));
    }
}

#[test]
fn black_cat_looks_at_all_of_a_short_library() {
    cr!("609.3");
    ruling!(
        "Black Cat, Cunning Thief",
        "If there are fewer than nine cards left in the opponent's library as the ability resolves, you look at all of the remaining cards."
    );
    supported("Black Cat, Cunning Thief");
    let mut t = TestGame::new(2);
    t.g.players[1].library.clear();
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant", "Divination"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[o(lib[0]), o(lib[2])]);
    let from = t.asked().len();
    t.enter(P0, "Black Cat, Cunning Thief");
    t.resolve_all();
    for c in &lib {
        assert!(was_offered(&t, from, *c), "{}", t.dump_log());
    }
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[2]));
    assert!(in_library(&t, lib[1], P1));
}

#[test]
fn planeswalkers_mischief_with_an_empty_hand_does_nothing() {
    cr!("608.2c");
    ruling!(
        "Planeswalker's Mischief",
        "If the opponent has no cards in hand, then the ability does nothing."
    );
    supported("Planeswalker's Mischief");
    let mut t = TestGame::new(2);
    let pm = t.battlefield(P0, "Planeswalker's Mischief");
    pool(&mut t, P0, &[U, C, C, C]);
    t.activate(P0, pm, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.g.exile.is_empty());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.g.exile.is_empty());
}
