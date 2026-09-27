//! CR 702.169 Solved, on the Cases whose "to solve" conditions look back over the turn
//! (CR 719.3a) and whose solved abilities are replacement effects or play permissions.

use crate::common_k702_011_017::{assert_supported, give_mana_for};
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::cases;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Advances to `P0`'s next end step and resolves the "to solve" trigger, if any.
fn end_step(t: &mut TestGame) {
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

/// Moves on to `P0`'s next precombat main phase.
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
}

#[test]
fn case_of_the_gateway_express_is_solved_if_three_creatures_attacked() {
    cr!("702.169b", "719.3a");
    ruling!(
        "Case of the Gateway Express",
        "\"To Solve — [condition]\" means \"At the beginning of your end step, if [condition] and this Case is not solved, it becomes solved.\""
    );
    assert_supported("Case of the Gateway Express");
    // "To solve — Three or more creatures attacked this turn." "Solved — Creatures you
    // control get +1/+0."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let case = t.battlefield(P0, "Case of the Gateway Express");
    let bears: Vec<_> = (0..3)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    // Two attackers: not solved.
    let two: Vec<_> = bears[..2]
        .iter()
        .map(|b| (*b, Entity::Player(P1)))
        .collect();
    t.attack(&two, &[]);
    end_step(&mut t);
    assert!(!cases::is_solved(&t.g, case));
    // Three attackers: solved, and creatures get +1/+0.
    next_turn(&mut t);
    let three: Vec<_> = bears.iter().map(|b| (*b, Entity::Player(P1))).collect();
    t.attack(&three, &[]);
    end_step(&mut t);
    assert!(cases::is_solved(&t.g, case));
    assert_eq!(t.pt(bears[0]), (3, 2));
}

#[test]
fn case_of_the_gorgons_kiss_counts_creature_cards_put_into_graveyards() {
    cr!("702.169b", "719.3a");
    ruling!(
        "Case of the Gorgon's Kiss",
        "Tokens are not cards and, as such, do not count toward Case of the Gorgon's Kiss"
    );
    assert_supported("Case of the Gorgon's Kiss");
    // "To solve — Three or more creature cards were put into graveyards from anywhere this
    // turn." "Solved — This Case is a 4/4 Gorgon creature with deathtouch and lifelink in
    // addition to its other types."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let case = t.battlefield(P0, "Case of the Gorgon's Kiss");
    // Two creature cards (one of each player's) and a creature token: not solved.
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(a, None);
    t.g.destroy(b, None);
    // Raise the Alarm: "Create two 1/1 white Soldier creature tokens."
    let alarm = t.hand(P0, "Raise the Alarm");
    give_mana_for(&mut t, P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve_all();
    for tok in tokens_of_subtype(&t, P0, "Soldier") {
        t.g.destroy(tok, None);
    }
    t.g.settle();
    end_step(&mut t);
    assert!(!cases::is_solved(&t.g, case));
    // Next turn: a creature card destroyed, one discarded, and one milled: solved.
    next_turn(&mut t);
    let c = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(c, None);
    let d = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, d, None);
    t.library_top(P1, "Grizzly Bears");
    t.g.mill(P1, 1);
    t.g.settle();
    end_step(&mut t);
    assert!(cases::is_solved(&t.g, case));
    t.g.recompute();
    let o = t.obj(case);
    assert!(o.chars.is(CardType::Creature) && o.chars.is(CardType::Enchantment));
    assert!(o.chars.has_subtype("Gorgon"));
    assert_eq!(t.pt(case), (4, 4));
    assert!(has_kw(&t, case, KeywordKind::Deathtouch));
    assert!(has_kw(&t, case, KeywordKind::Lifelink));
}

#[test]
fn case_of_the_burning_masks_counts_each_source_you_controlled_once() {
    cr!("702.169d", "719.3a");
    ruling!(
        "Case of the Burning Masks",
        "A single creature that deals combat damage multiple times in a turn (due to double strike or additional combats) still counts as only one source."
    );
    ruling!(
        "Case of the Burning Masks",
        "You need to control the sources that dealt damage only at the times when they dealt damage."
    );
    ruling!(
        "Case of the Burning Masks",
        "Something that changes zones becomes a new object in that zone and will therefore be a new source."
    );
    assert_supported("Case of the Burning Masks");
    // "To solve — Three or more sources you controlled dealt damage this turn."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let case = t.battlefield(P0, "Case of the Burning Masks");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // One source twice, another once, and one the opponent controls: two sources.
    t.g.deal_damage(a, Entity::Player(P1), 1, false);
    t.g.deal_damage(a, Entity::Player(P1), 1, false);
    t.g.deal_damage(b, Entity::Player(P1), 1, false);
    t.g.deal_damage(theirs, Entity::Player(P0), 1, false);
    t.g.settle();
    end_step(&mut t);
    assert!(!cases::is_solved(&t.g, case));
    // Next turn: a source that dealt damage and then died still counts, and a permanent
    // that left and returned is a new source.
    next_turn(&mut t);
    t.g.deal_damage(a, Entity::Player(P1), 1, false);
    let a2 = flicker(&mut t, a);
    t.g.deal_damage(a2, Entity::Player(P1), 1, false);
    t.g.deal_damage(b, Entity::Player(P1), 1, false);
    t.g.destroy(b, None);
    t.g.settle();
    end_step(&mut t);
    assert!(cases::is_solved(&t.g, case));
}

#[test]
fn case_of_the_burning_masks_lets_you_play_the_chosen_card_this_turn() {
    cr!("702.169d");
    ruling!(
        "Case of the Burning Masks",
        "You pay all costs and follow all normal timing rules for the card played from exile"
    );
    // "Solved — Sacrifice this Case: Exile the top three cards of your library. Choose one
    // of them. You may play that card this turn."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let case = t.battlefield(P0, "Case of the Burning Masks");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let bears = t.library_top(P0, "Grizzly Bears");
    let mountain = t.library_top(P0, "Mountain");
    // Unsolved, the Case has no such ability.
    t.g.recompute();
    assert!(!t
        .obj(case)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.starts_with("Sacrifice ~")));
    cases::solve(&mut t.g, case);
    let uid = ability_uid(&mut t, case, "Sacrifice ~");
    // Activated in the beginning of combat step; the player chooses one of the three
    // exiled cards (by default the first, the former top card: the Mountain).
    activate_uid(&mut t, P0, case, uid).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Case of the Burning Masks"));
    let now = |t: &TestGame, id| t.g.current(id);
    for c in [mountain, bolt, bears] {
        assert_eq!(t.zone(now(&t, c)), Zone::Exile);
    }
    let chosen = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("chose one of them");
    assert_eq!(chosen.len(), 3);
    let m = now(&t, mountain);
    assert_eq!(chosen[0], Entity::Object(m));
    let can_play_land = |t: &mut TestGame| {
        t.g.turn.priority = Some(P0);
        t.g.legal_actions(P0)
            .contains(&Action::PlayLand { card: m })
    };
    // A land may be played only in a main phase.
    assert!(!can_play_land(&mut t));
    // Only the chosen card may be played.
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(cast_methods_now(&mut t, P0, bolt).is_empty());
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t));
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(cast_methods_now(&mut t, P0, bears).is_empty());
    t.play_land(P0, m).unwrap();
    assert_eq!(t.named_on_battlefield("Mountain").len(), 1);
}

#[test]
fn case_of_the_ransacked_lab_counts_the_spells_you_cast_this_turn() {
    cr!("702.169c", "719.3a");
    assert_supported("Case of the Ransacked Lab");
    // "To solve — You've cast four or more instant and sorcery spells this turn."
    for spells in [3, 4] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let case = t.battlefield(P0, "Case of the Ransacked Lab");
        // A creature spell doesn't count.
        let bears = t.hand(P0, "Grizzly Bears");
        give_mana_for(&mut t, P0, "Grizzly Bears");
        t.cast(P0, bears).go();
        t.resolve_all();
        for _ in 0..spells {
            let opt = t.hand(P0, "Opt");
            add_mana(&mut t, P0, ManaType::U, 1);
            t.cast(P0, opt).go();
            t.resolve_all();
        }
        end_step(&mut t);
        assert_eq!(cases::is_solved(&t.g, case), spells == 4);
    }
}

#[test]
fn case_of_the_locked_hothouse_lets_you_play_from_the_top_of_your_library() {
    cr!("702.169b");
    assert_supported("Case of the Locked Hothouse");
    // "Solved — You may look at the top card of your library any time, and you may play
    // lands and cast creature and enchantment spells from the top of your library."
    for solved in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let case = t.battlefield(P0, "Case of the Locked Hothouse");
        if solved {
            cases::solve(&mut t.g, case);
        }
        t.g.recompute();
        let forest = t.library_top(P0, "Forest");
        t.g.turn.priority = Some(P0);
        let can_play = t
            .g
            .legal_actions(P0)
            .contains(&Action::PlayLand { card: forest });
        assert_eq!(can_play, solved);
        if solved {
            t.play_land(P0, forest).unwrap();
        }
        // A creature spell on top may be cast; an instant may not.
        let t2 = t.library_top(P0, "Grizzly Bears");
        add_mana(&mut t, P0, ManaType::G, 2);
        assert_eq!(!cast_methods_now(&mut t, P0, t2).is_empty(), solved);
        let bolt = t.library_top(P0, "Lightning Bolt");
        add_mana(&mut t, P0, ManaType::R, 1);
        assert!(cast_methods_now(&mut t, P0, bolt).is_empty());
    }
}

#[test]
fn case_of_the_pilfered_proof_adds_a_clue_to_tokens_you_create() {
    cr!("702.169b", "614.1a");
    assert_supported("Case of the Pilfered Proof");
    // "Solved — If one or more tokens would be created under your control, those tokens
    // plus a Clue token are created instead."
    for solved in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let case = t.battlefield(P0, "Case of the Pilfered Proof");
        if solved {
            cases::solve(&mut t.g, case);
        }
        // Raise the Alarm: "Create two 1/1 white Soldier creature tokens."
        let spell = t.hand(P0, "Raise the Alarm");
        give_mana_for(&mut t, P0, "Raise the Alarm");
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(tokens_of_subtype(&t, P0, "Soldier").len(), 2);
        // One Clue for the event, not one per token; the Clue doesn't make another.
        assert_eq!(tokens_of_subtype(&t, P0, "Clue").len(), solved as usize);
        // Tokens an opponent creates aren't affected.
        let theirs = t.hand(P1, "Raise the Alarm");
        give_mana_for(&mut t, P1, "Raise the Alarm");
        t.cast(P1, theirs).go();
        t.resolve_all();
        assert!(tokens_of_subtype(&t, P1, "Clue").is_empty());
    }
}

#[test]
fn case_of_the_stashed_skeleton_is_solved_if_you_control_no_suspected_skeletons() {
    cr!("719.3a");
    assert_supported("Case of the Stashed Skeleton");
    // "When this Case enters, create a 2/1 black Skeleton creature token and suspect it."
    // "To solve — You control no suspected Skeletons."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let case = t.enter(P0, "Case of the Stashed Skeleton");
    t.resolve_all();
    let skeletons = tokens_of_subtype(&t, P0, "Skeleton");
    assert_eq!(skeletons.len(), 1);
    // A suspected Skeleton: not solved.
    end_step(&mut t);
    assert!(!cases::is_solved(&t.g, case));
    // The Skeleton is gone: solved.
    next_turn(&mut t);
    t.g.destroy(skeletons[0], None);
    t.g.settle();
    end_step(&mut t);
    assert!(cases::is_solved(&t.g, case));
}
