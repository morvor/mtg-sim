//! Rulings batch P015 — triggered and activated abilities of creatures with
//! power/toughness-defining abilities: intervening "if" clauses, numbers counted as the
//! ability resolves, chosen opponents, and tokens put onto the battlefield attacking.

use crate::r_p015_common::*;
use crate::r_p016_common::{cast_at, lots};
use crate::r_s01_common::supported;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::turn::Step;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn sage_dies_to_an_empty_hand_before_its_draw_trigger_resolves() {
    cr!("704.5f", "603.3", "604.3");
    ruling!(
        "Sage of Ancient Lore // Werewolf of Ancient Hunger",
        "If you have no cards in hand after Sage of Ancient Lore enters the battlefield, it will be put into your graveyard before its first triggered ability resolves."
    );
    supported("Sage of Ancient Lore // Werewolf of Ancient Hunger");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    let hand = t.hand_size(P0);
    assert_eq!(hand, 0);
    let sage = enter(&mut t, P0, "Sage of Ancient Lore // Werewolf of Ancient Hunger");
    assert!(!on_bf(&t, sage), "a 0/0 is put into the graveyard");
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), 1, "you still draw");
    assert!(!on_bf(&t, sage));
}

/// Kiyomaro attacks P1 unblocked with `hand` cards in P0's hand; returns at the combat
/// damage step with its trigger (if any) on the stack.
fn kiyomaro_hits(hand: usize) -> (TestGame, ObjectId) {
    supported("Kiyomaro, First to Stand");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..hand {
        t.hand(P0, "Forest");
    }
    let k = t.battlefield(P0, "Kiyomaro, First to Stand");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(k, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    (t, k)
}

#[test]
fn kiyomaro_rechecks_seven_cards_as_its_trigger_resolves() {
    cr!("603.4");
    ruling!(
        "Kiyomaro, First to Stand",
        "If you have seven or more cards in hand when Kiyomaro's triggered ability goes on the stack but less than seven when the ability resolves, the ability does nothing."
    );
    let (mut t, _) = kiyomaro_hits(7);
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.life(P0), 27);

    let (mut t, _) = kiyomaro_hits(7);
    assert_eq!(t.stack_len(), 1);
    let card = t.g.players[0].hand[0];
    t.g.discard(P0, card, None);
    t.resolve();
    assert_eq!(t.life(P0), 20, "six cards as it resolves: nothing");

    let (t, _) = kiyomaro_hits(6);
    assert_eq!(t.stack_len(), 0, "six cards: it doesn't trigger");
}

#[test]
fn kiyomaro_vigilance_depends_on_cards_in_hand_not_its_power() {
    cr!("604.3", "613.1f");
    ruling!(
        "Kiyomaro, First to Stand",
        "Kiyomaro's power and toughness don't affect whether it has vigilance or the life-gain ability."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..3 {
        t.hand(P0, "Forest");
    }
    let k = t.battlefield(P0, "Kiyomaro, First to Stand");
    // Giant Growth makes it 6/6; three cards in hand: no vigilance.
    cast_at(&mut t, P0, "Giant Growth", &[Entity::Object(k)]);
    t.resolve();
    assert_eq!(t.pt(k), (6, 6));
    assert!(!t.obj_now(k).has_keyword(KeywordKind::Vigilance));
    t.hand(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(k), (7, 7));
    assert!(t.obj_now(k).has_keyword(KeywordKind::Vigilance));
    // A 7-power Kiyomaro with four cards in hand: no life gain on damage.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(k, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
}

/// P0 controls a full party (Archpriest of Iona is the Cleric) and Grizzly Bears, the
/// target of the beginning-of-combat trigger. Returns (bears, the Warrior).
fn full_party() -> (TestGame, ObjectId, ObjectId) {
    supported("Archpriest of Iona");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Archpriest of Iona");
    let warrior = t.battlefield(P0, "Goblin Piker");
    t.battlefield(P0, "Merfolk Looter");
    t.battlefield(P0, "Prodigal Sorcerer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    (t, bears, warrior)
}

#[test]
fn archpriest_checks_a_full_party_on_trigger_and_resolution_and_the_bonus_stays() {
    cr!("603.4", "611.2a", "700.8");
    ruling!(
        "Archpriest of Iona",
        "Once the target creature gets +1/+1 and gains flying, it keeps those bonuses even if you no longer have a full party later in the turn."
    );
    ruling!(
        "Archpriest of Iona",
        "The last ability won't trigger at all if you don't have a full party as your combat phase begins."
    );
    let (mut t, bears, warrior) = full_party();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
    t.g.destroy(warrior, None);
    t.settle();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));

    // Losing the full party between trigger and resolution: nothing.
    let (mut t, bears, warrior) = full_party();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(warrior, None);
    t.settle();
    t.resolve();
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));

    // No full party as combat begins: no trigger.
    let (mut t, _, warrior) = full_party();
    t.g.destroy(warrior, None);
    t.settle();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn tishana_counts_creatures_as_its_trigger_resolves_including_itself() {
    cr!("608.2h");
    ruling!(
        "Tishana, Voice of Thunder",
        "The number of creatures you control is counted only as Tishana's last ability resolves. If Tishana is still on the battlefield, it'll count itself."
    );
    supported("Tishana, Voice of Thunder");
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Forest");
    }
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    enter(&mut t, P0, "Tishana, Voice of Thunder");
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(bears, None);
    t.settle();
    t.resolve();
    // Only Tishana itself is left.
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn robobrain_counts_artifact_creatures_as_its_trigger_resolves() {
    cr!("608.2h");
    ruling!(
        "Robobrain War Mind",
        "Use the number of artifact creatures you control when Robobrain War Mind's second ability resolves, including Robobrain War Mind itself if it's still on the battlefield"
    );
    supported("Robobrain War Mind");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    enter(&mut t, P0, "Robobrain War Mind");
    t.g.destroy(thopter, None);
    t.settle();
    t.resolve();
    assert_eq!(energy(&t, P0), 2, "one Ornithopter and itself");

    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    let r = enter(&mut t, P0, "Robobrain War Mind");
    move_to(&mut t, r, mtg_engine::object::Zone::Hand(P0));
    t.resolve();
    assert_eq!(energy(&t, P0), 1, "it's gone: only the Ornithopter");
}

#[test]
fn x_is_determined_as_the_ability_resolves() {
    cr!("608.2h", "107.3a");
    ruling!(
        "Kagemaro, First to Suffer",
        "The value of X is determined as the ability resolves."
    );
    ruling!(
        "Scourge of Fleets",
        "The value of X is determined as the ability resolves."
    );
    // Kagemaro: sacrificed with two cards in hand; a third arrives before resolution.
    supported("Kagemaro, First to Suffer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Forest");
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    let giant = t.battlefield(P1, "Hill Giant");
    let k = t.battlefield(P0, "Kagemaro, First to Suffer");
    lots(&mut t, P0);
    t.activate(P0, k, 0, &[]).unwrap();
    assert!(!on_bf(&t, k));
    t.g.draw_cards(P0, 1);
    t.resolve();
    assert!(!on_bf(&t, giant), "-3/-3 kills a 3/3");

    // Scourge of Fleets: two Islands as it enters; a third before its trigger resolves.
    supported("Scourge of Fleets");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    enter(&mut t, P0, "Scourge of Fleets");
    t.battlefield(P0, "Island");
    t.resolve();
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(!on_bf(&t, giant) && !on_bf(&t, bears));
}

/// P0's Entropic Specter, having chosen P1 (who has two cards in hand), attacks P1.
fn specter_attacks(holy_day: bool) -> TestGame {
    supported("Entropic Specter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.hand(P1, "Forest");
    t.hand(P1, "Forest");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let s = enter(&mut t, P0, "Entropic Specter");
    let cur = t.g.current(s);
    t.g.objects[cur.0 as usize].summoning_sick = false;
    if holy_day {
        cast_at(&mut t, P1, "Holy Day", &[]);
        t.resolve();
    }
    assert_eq!(t.pt(s), (2, 2));
    t.attack(&[(s, Entity::Player(P1))], &[]);
    t
}

#[test]
fn entropic_specter_prevented_damage_means_no_discard() {
    cr!("615.1", "603.2");
    ruling!(
        "Entropic Specter",
        "The opponent does not discard a card if all the damage is prevented."
    );
    let t = specter_attacks(false);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P1), 1);
    let t = specter_attacks(true);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn the_chosen_opponent_stays_chosen_through_control_changes_and_leaving() {
    cr!("607.2d", "800.4a", "704.5f");
    ruling!(
        "Entropic Specter",
        "You choose one opposing player during resolution and it only affects that one player. This choice is not changed even if this card changes controllers. It becomes 0/0 if the chosen player leaves the game."
    );
    ruling!(
        "Haunting Apparition",
        "You choose one opposing player when it enters and it only affects that one player. This choice is not changed even if this card changes controllers. This card becomes useless but stays on the battlefield if the chosen player leaves the game."
    );
    supported("Entropic Specter");
    supported("Haunting Apparition");
    let mut t = TestGame::new(3);
    t.hand(P1, "Forest");
    t.hand(P1, "Forest");
    t.hand(P2, "Forest");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P2, "Grizzly Bears");
    t.graveyard(P2, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let s = enter(&mut t, P0, "Entropic Specter");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let a = enter(&mut t, P0, "Haunting Apparition");
    assert_eq!(t.pt(s), (2, 2));
    assert_eq!(t.pt(a), (2, 2));
    // P2 gains control of both: they still look at P1.
    gain_control(&mut t, P2, s);
    gain_control(&mut t, P2, a);
    t.settle();
    assert_eq!(t.obj_now(s).controller, P2);
    assert_eq!(t.pt(s), (2, 2));
    assert_eq!(t.pt(a), (2, 2));
    // P1 leaves the game: the Specter is 0/0 and dies; the Apparition stays as a 1/2.
    t.g.lose_game(P1);
    t.settle();
    assert!(!on_bf(&t, s));
    assert!(on_bf(&t, a));
    assert_eq!(t.pt(a), (1, 2));
}

#[test]
fn silverwing_squadron_doesnt_count_opponents_who_left() {
    cr!("800.4a", "102.2");
    ruling!(
        "Silverwing Squadron",
        "In a multiplayer game, opponents who have left the game aren't counted by Silverwing Squadron's triggered ability."
    );
    supported("Silverwing Squadron");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let s = t.battlefield(P0, "Silverwing Squadron");
    t.g.lose_game(P2);
    t.settle();
    t.attack(&[(s, Entity::Player(P1))], &[]);
    assert_eq!(tokens(&t, P0, "Knight").len(), 1);
}

#[test]
fn gaeas_avenger_counts_artifacts_of_every_opponent() {
    cr!("604.3", "102.2");
    ruling!(
        "Gaea's Avenger",
        "In multi-player games it counts artifacts of all opponents."
    );
    supported("Gaea's Avenger");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P2, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    let g = t.battlefield(P0, "Gaea's Avenger");
    t.g.recompute();
    assert_eq!(t.pt(g), (3, 3));
}

#[test]
fn attack_tokens_are_created_for_each_opponent_not_just_the_attacked_one() {
    cr!("508.4", "506.2");
    ruling!(
        "Adeline, Resplendent Cathar",
        "Tokens will be created for each of your opponents, not just opponents that you attacked."
    );
    for (name, subtype) in [
        ("Adeline, Resplendent Cathar", "Human"),
        ("Endless Foot Assault", "Ninja"),
    ] {
        supported(name);
        let mut t = TestGame::new(3);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, name);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(bears, Entity::Player(P1))]),
        );
        t.advance_to(P0, Step::DeclareAttackers);
        t.resolve_all();
        let toks = tokens(&t, P0, subtype);
        assert_eq!(toks.len(), 2, "{name}");
        let mut at: Vec<_> = toks.iter().map(|id| attacking(&t, *id)).collect();
        at.sort_by_key(|e| format!("{e:?}"));
        assert_eq!(
            at,
            vec![Some(Entity::Player(P1)), Some(Entity::Player(P2))],
            "{name}"
        );
    }
}

#[test]
fn suki_token_attacks_whichever_opponent_you_choose() {
    cr!("508.4");
    ruling!(
        "Suki, Kyoshi Warrior",
        "You choose which opponent, planeswalker, or battle the token is attacking as it enters."
    );
    supported("Suki, Kyoshi Warrior");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let suki = t.battlefield(P0, "Suki, Kyoshi Warrior");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(suki, Entity::Player(P1))]),
    );
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let toks = tokens(&t, P0, "Ally");
    assert_eq!(toks.len(), 1);
    assert_eq!(attacking(&t, toks[0]), Some(Entity::Player(P2)));
    assert_eq!(attacking(&t, suki), Some(Entity::Player(P1)));
}

#[test]
fn scourge_of_the_skyclaves_in_two_headed_giant_is_a_draw() {
    cr!("810.9", "810.9a", "810.8c", "104.4a");
    ruling!(
        "Scourge of the Skyclaves",
        "In a Two-Headed Giant game, Scourge of the Skyclaves causes each team to lose twice half their life rounded up, which is their full life total or, if it's odd, that much life plus 1. This causes the game to end in a draw."
    );
    supported("Scourge of the Skyclaves");
    let mut t = TestGame::with_config(
        4,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.set_step(P0, Step::PrecombatMain);
    // An odd team life total: 29 for P2 and P3's team.
    t.g.lose_life(P2, 1);
    assert_eq!(t.life(P3), 29);
    lots(&mut t, P0);
    let c = t.hand(P0, "Scourge of the Skyclaves");
    t.cast(P0, c).kicked(true).go();
    t.resolve();
    // 30 - 15 - 15 and 29 - 15 - 15.
    assert_eq!(t.life(P0), 0);
    assert_eq!(t.life(P2), -1);
    assert_eq!(
        t.g.result,
        Some(mtg_engine::game::GameResult::Draw),
        "{:?}",
        t.g.result
    );
}
