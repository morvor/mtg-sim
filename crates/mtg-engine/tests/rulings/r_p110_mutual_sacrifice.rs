//! Rulings batch P110 — "each player sacrifices / discards" effects: the player whose turn
//! it is chooses first, then each other player in turn order, knowing the choices made
//! before them (hidden cards stay hidden); then everything happens at the same time
//! (CR 101.4, 101.4a, 101.4b, 701.21a). A player with too few creatures sacrifices all of
//! them; nothing is targeted (CR 115.1); copies keep the modes chosen (CR 707.10).

use crate::r_p110_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameResult;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// What P1 knows of the earlier choices, and how many creatures are on the battlefield,
/// each time P1 is asked to choose.
type Seen = (Vec<(PlayerId, Option<Vec<ObjectId>>)>, usize);

fn watch_p1(t: &mut TestGame) -> crate::r_s01_common::Seen<Seen> {
    watch(
        t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| (g.known_apnap_choices(P1), creatures_on_battlefield(g)),
    )
}

/// P0 (with a Blood Artist among `p0` creatures) and P1 (with `p1` Grizzly Bears) on
/// `active`'s turn: P0 casts `spell` (an instant or sorcery making each player sacrifice
/// `n` creatures) choosing `p0_picks` (indices into P0's creatures, the Blood Artist
/// first). Returns the game, P0's and P1's creatures, what P1 saw, and the decision log
/// index before the spell resolved.
#[allow(clippy::type_complexity)]
fn each_player_sacrifices(
    spell: &str,
    active: PlayerId,
    p0: usize,
    p1: usize,
    p0_picks: &[usize],
) -> (TestGame, Vec<ObjectId>, Vec<ObjectId>, Vec<Seen>, usize) {
    supported(spell);
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    t.set_step(active, Step::PrecombatMain);
    let mut mine = vec![t.battlefield(P0, "Blood Artist")];
    mine.extend(bears(&mut t, P0, p0.saturating_sub(1)));
    let theirs = bears(&mut t, P1, p1);
    let seen = watch_p1(&mut t);
    let picks: Vec<Entity> = p0_picks.iter().map(|i| obj(mine[*i])).collect();
    t.answer_choose(P0, &picks);
    lands_for_cost(&mut t, P0, spell);
    let card = t.hand(P0, spell);
    t.cast(P0, card).go();
    let from = t.asked().len();
    t.resolve();
    let seen = seen.lock().unwrap().clone();
    (t, mine, theirs, seen, from)
}

/// The number of Blood Artist triggers waiting or on the stack ("Whenever this creature
/// or another creature dies").
fn blood_artist_triggers(t: &TestGame) -> usize {
    t.g.pending_triggers.len()
        + t.g
            .stack
            .iter()
            .filter(|id| t.g.obj(**id).stack.is_some())
            .count()
}

#[test]
fn each_player_chooses_in_turn_order_then_all_are_sacrificed_at_once() {
    cr!("101.4", "101.4b", "701.21a", "603.10a");
    ruling!(
        "Taste of Death",
        "As Taste of Death resolves, first the player whose turn it is chooses three creatures they control, then each other player in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    ruling!(
        "Tergrid's Shadow",
        "As Tergrid's Shadow resolves, first the player whose turn it is chooses two creatures they control, then each other player in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    ruling!(
        "Barter in Blood",
        "The active player chooses which creatures will be sacrificed first, then each other player in turn order does the same. Then all creatures are sacrificed simultaneously."
    );
    for (spell, n) in [
        ("Taste of Death", 3usize),
        ("Tergrid's Shadow", 2),
        ("Barter in Blood", 2),
    ] {
        // P0 has four creatures (the Blood Artist among the chosen ones), P1 has four.
        let picks: Vec<usize> = (0..n).collect();
        let (t, mine, _, seen, from) = each_player_sacrifices(spell, P0, 4, 4, &picks);
        assert_eq!(sacrifice_choosers(&t, from), vec![P0, P1], "{spell}");
        // When P1 chose, P0's choice was known and nothing had been sacrificed yet.
        assert_eq!(seen.len(), 1, "{spell}");
        let (known, on_battlefield) = &seen[0];
        let p0_choice: Vec<ObjectId> = picks.iter().map(|i| mine[*i]).collect();
        assert_eq!(known, &vec![(P0, Some(p0_choice))], "{spell}");
        assert_eq!(*on_battlefield, 8, "{spell}");
        // All of them were sacrificed at once: the Blood Artist saw every one die.
        assert_eq!(creatures(&t, P0).len(), 4 - n, "{spell}");
        assert_eq!(creatures(&t, P1).len(), 4 - n, "{spell}");
        assert_eq!(blood_artist_triggers(&t), 2 * n, "{spell}");
    }
    // On P1's turn, P1 chooses first (and P0, who cast the spell, chooses second).
    let (t, ..) = each_player_sacrifices("Tergrid's Shadow", P1, 2, 2, &[0, 1]);
    let from = t
        .asked()
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .unwrap();
    assert_eq!(sacrifice_choosers(&t, from), vec![P1, P0]);
}

#[test]
fn with_too_few_creatures_a_player_sacrifices_all_of_them() {
    cr!("701.21a", "101.3");
    ruling!(
        "Taste of Death",
        "If a player controls three or fewer creatures, that player sacrifices each creature they control."
    );
    ruling!(
        "Tergrid's Shadow",
        "If a player controls two or fewer creatures, that player sacrifices each creature they control."
    );
    ruling!(
        "Barter in Blood",
        "If a player controls only one creature, that creature is sacrificed."
    );
    ruling!(
        "By Invitation Only",
        "If a player controls fewer than the chosen number of creatures, they will sacrifice all their creatures."
    );
    for (spell, mine, theirs) in [
        ("Taste of Death", 3usize, 2usize),
        ("Tergrid's Shadow", 2, 1),
        ("Barter in Blood", 1, 2),
    ] {
        let mut t = TestGame::new(2);
        bears(&mut t, P0, mine);
        bears(&mut t, P1, theirs);
        cast_new(&mut t, P0, spell, &[]);
        t.resolve_all();
        assert!(creatures(&t, P0).is_empty(), "{spell}");
        assert!(creatures(&t, P1).is_empty(), "{spell}");
    }
    // By Invitation Only, choosing 5: P0 with three creatures and P1 with six.
    supported("By Invitation Only");
    let mut t = TestGame::new(2);
    bears(&mut t, P0, 3);
    bears(&mut t, P1, 6);
    t.answer(P0, DecisionKind::Number, Answer::Number(5));
    cast_new(&mut t, P0, "By Invitation Only", &[]);
    t.resolve_all();
    assert!(creatures(&t, P0).is_empty());
    assert_eq!(creatures(&t, P1).len(), 1);
}

#[test]
fn by_invitation_only_numbers_include_zero_and_thirteen() {
    cr!("107.1", "701.21a");
    ruling!(
        "By Invitation Only",
        "The numbers you may choose include 0 and 13."
    );
    for (n, left) in [(0i64, 14usize), (13, 1)] {
        let mut t = TestGame::new(2);
        bears(&mut t, P1, 14);
        t.answer(P0, DecisionKind::Number, Answer::Number(n));
        let from = t.asked().len();
        cast_new(&mut t, P0, "By Invitation Only", &[]);
        t.resolve_all();
        let range: Vec<(i64, i64)> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseNumber { min, max, .. } => Some((*min, *max)),
                _ => None,
            })
            .collect();
        assert_eq!(range, vec![(0, 13)]);
        assert_eq!(creatures(&t, P1).len(), left, "chose {n}");
    }
}

#[test]
fn barter_in_blood_doesnt_target_and_can_be_cast_with_few_creatures() {
    cr!("115.1", "601.2c", "701.21a");
    ruling!(
        "Barter in Blood",
        "Barter in Blood doesn’t target any creatures and may be cast even if a player controls fewer than two creatures."
    );
    ruling!(
        "Innocent Blood",
        "You can cast Innocent Blood even if some (or all) players control no creatures. Those players are simply unaffected."
    );
    // Barter in Blood with P1 controlling one creature with protection from black.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Barter in Blood", &[]);
    assert!(
        t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })),
        "no targets are chosen"
    );
    t.resolve_all();
    assert!(!t.on_battlefield(knight));
    // Innocent Blood with no creatures at all, and with only P1 controlling one.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Innocent Blood", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Innocent Blood"));
    let mut t = TestGame::new(3);
    let b = t.battlefield(P1, "Grizzly Bears");
    let kept = t.battlefield(P2, "Grizzly Bears");
    let kept2 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P2, &[obj(kept2)]);
    cast_new(&mut t, P0, "Innocent Blood", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(b) && !t.on_battlefield(kept2));
    assert!(t.on_battlefield(kept));
}

#[test]
fn abyssal_gatekeepers_sacrifice_isnt_targeted() {
    cr!("702.16b", "115.1", "603.10a");
    ruling!(
        "Abyssal Gatekeeper",
        "The \"sacrifice a creature\" effect is not targeted. It can affect creatures with protection from black, for example."
    );
    supported("Abyssal Gatekeeper");
    let mut t = TestGame::new(2);
    let gk = t.battlefield(P0, "Abyssal Gatekeeper");
    let knight = t.battlefield(P1, "White Knight");
    destroy(&mut t, gk);
    t.resolve_all();
    assert!(!t.on_battlefield(knight));
    assert!(t.in_graveyard(P1, "White Knight"));
}

#[test]
fn abyssal_gatekeeper_the_active_player_chooses_first() {
    cr!("101.4", "101.4b", "701.21a");
    ruling!(
        "Abyssal Gatekeeper",
        "The player whose turn it is chooses a creature to sacrifice, then each other player in turn order chooses a creature to sacrifice, then all chosen creatures are sacrificed at the same time."
    );
    // On P1's turn P0's Gatekeeper dies: P1 chooses first, then P0. P0's Blood Artist is
    // sacrificed with P1's Bears and sees it die.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let gk = t.battlefield(P0, "Abyssal Gatekeeper");
    let artist = t.battlefield(P0, "Blood Artist");
    let b = bears(&mut t, P1, 2);
    let seen = watch_p1(&mut t);
    t.answer_choose(P1, &[obj(b[0])]);
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    destroy(&mut t, gk);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(sacrifice_choosers(&t, from), vec![P1, P0]);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(
        seen,
        vec![(vec![], 3)],
        "P1 chose first, before any sacrifice"
    );
    assert!(!t.on_battlefield(artist) && !t.on_battlefield(b[0]));
    assert!(t.on_battlefield(b[1]));
    // Blood Artist: once for the Gatekeeper, once for itself, once for the Bears.
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
}

#[test]
fn rankles_prank_chooses_in_order_and_acts_at_once() {
    cr!("101.4", "101.4a", "101.4b", "700.2");
    ruling!(
        "Rankle's Prank",
        "As the first mode is performed, first the player whose turn it is chooses two cards in hand without revealing them, then each other player in turn order does the same. Then the chosen cards are discarded at the same time."
    );
    ruling!(
        "Rankle's Prank",
        "As the third mode is performed, first the player whose turn it is chooses two creatures they control, then each other player in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    supported("Rankle's Prank");
    // First mode: P0's choice of cards is hidden from P1, and nothing is discarded before
    // P1 chooses.
    let mut t = TestGame::new(2);
    let h0: Vec<ObjectId> = (0..3).map(|_| t.hand(P0, "Grizzly Bears")).collect();
    (0..3).for_each(|_| {
        t.hand(P1, "Grizzly Bears");
    });
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| (g.known_apnap_choices(P1), g.player(P0).graveyard.len()),
    );
    t.answer_choose(P0, &[obj(h0[0]), obj(h0[1])]);
    lands_for_cost(&mut t, P0, "Rankle's Prank");
    let card = t.hand(P0, "Rankle's Prank");
    t.cast(P0, card).modes(&[0]).go();
    let from = t.asked().len();
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen, vec![(vec![(P0, None)], 0)]);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (1, 1));
    assert_eq!(t.zone(h0[2]), Zone::Hand(P0));
    // Third mode: like Barter in Blood.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    bears(&mut t, P0, 1);
    bears(&mut t, P1, 3);
    let seen = watch_p1(&mut t);
    lands_for_cost(&mut t, P0, "Rankle's Prank");
    let card = t.hand(P0, "Rankle's Prank");
    t.cast(P0, card).modes(&[2]).go();
    let from = t.asked().len();
    t.resolve();
    assert_eq!(sacrifice_choosers(&t, from), vec![P0, P1]);
    let seen = seen.lock().unwrap().clone();
    assert!(matches!(&seen[..], [(known, 5)] if known.len() == 1 && known[0].1.is_some()));
    assert_eq!(blood_artist_triggers(&t), 4);
    assert_eq!(creatures(&t, P1).len(), 1);
}

#[test]
fn rankles_prank_modes_may_affect_no_one() {
    cr!("700.2", "601.2b");
    ruling!(
        "Rankle's Prank",
        "You can choose a mode even if some or all players will be entirely or partially unaffected."
    );
    // No cards in hand and no creatures: all three modes; only the life loss happens.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Rankle's Prank");
    let card = t.hand(P0, "Rankle's Prank");
    t.cast(P0, card).modes(&[0, 1, 2]).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (16, 16));
    // Only P1 has a creature and a card: P1 alone sacrifices and discards.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Rankle's Prank");
    let card = t.hand(P0, "Rankle's Prank");
    t.cast(P0, card).modes(&[0, 2]).go();
    t.resolve_all();
    assert!(!t.on_battlefield(b));
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn necrotic_hex_sacrifices_then_creates_six_zombies() {
    cr!("101.4", "701.21a", "608.2c");
    ruling!(
        "Necrotic Hex",
        "As Necrotc Hex resolves, first the player whose turn it is (likely you) chooses which six creatures they'll sacrifice, then each other player in turn order does the same, knowing the choices of players before them. Then all creatures are sacrificed simultaneously. Finally, you'll create the six tapped Zombies."
    );
    ruling!(
        "Necrotic Hex",
        "You'll create six tapped Zombies no matter how many creatures you sacrifice."
    );
    supported("Necrotic Hex");
    // P0 controls seven creatures, P1 eight (with Blood Artist).
    let (t, mine, _, seen, from) =
        each_player_sacrifices("Necrotic Hex", P0, 7, 8, &[1, 2, 3, 4, 5, 6]);
    assert_eq!(sacrifice_choosers(&t, from), vec![P0, P1]);
    assert!(matches!(&seen[..], [(known, 15)] if known.len() == 1));
    assert!(t.on_battlefield(mine[0]), "the Blood Artist was kept");
    assert_eq!(blood_artist_triggers(&t), 12);
    assert_eq!(creatures(&t, P1).len(), 2);
    // The six Zombies came after the sacrifices: none was sacrificed, all tapped.
    let zombies: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(zombies.len(), 6);
    assert!(zombies.iter().all(|z| t.obj(*z).tapped));
    // With no creatures to sacrifice at all: still six Zombies.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Necrotic Hex", &[]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 6);
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn doomsday_confluence_copies_keep_the_modes() {
    cr!("707.10", "700.2h");
    ruling!(
        "Doomsday Confluence",
        "If an effect copies Doomsday Confluence, the copy will have the same modes chosen the same number of times."
    );
    supported("Doomsday Confluence");
    // X = 2, the Dalek mode twice; Twincast copies it: four Daleks.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Doomsday Confluence");
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Doomsday Confluence");
    let spell = t.cast(P0, card).x(2).modes(&[1, 1]).go();
    t.lands(P0, "Island", 2);
    let tc = t.hand(P0, "Twincast");
    t.cast(P0, tc).target(spell).go();
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 2, "the copy made two Daleks");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 4);
}

#[test]
fn fraying_omnipotence_follows_its_instructions_in_order() {
    cr!("608.2c", "101.4", "101.4a", "107.1a");
    ruling!(
        "Fraying Omnipotence",
        "When Fraying Omnipotence resolves, first each player loses the appropriate amount of life at once. Next, the player whose turn it is chooses an appropriate number of cards in hand without revealing them, then each other player in turn order does the same, and then those cards are all discarded simultaneously. Then, the player whose turn it is chooses an appropriate number of creatures they control. Each other player in turn order does the same, knowing choices made by previous players. Finally, all of the chosen creatures are sacrificed simultaneously."
    );
    supported("Fraying Omnipotence");
    // P0 at 20 with no cards left in hand and three creatures; P1 at 15 with three cards
    // and three creatures.
    let mut t = TestGame::new(2);
    t.g.players[1].life = 15;
    bears(&mut t, P0, 3);
    bears(&mut t, P1, 3);
    (0..3).for_each(|_| {
        t.hand(P1, "Grizzly Bears");
    });
    // At each of P1's choices: both life totals, P1's graveyard, the creatures.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            (
                g.player(P0).life,
                g.player(P1).life,
                g.player(P1).graveyard.len(),
                creatures_on_battlefield(g),
                g.known_apnap_choices(P1).len(),
            )
        },
    );
    cast_new(&mut t, P0, "Fraying Omnipotence", &[]);
    t.resolve_all();
    let seen = seen.lock().unwrap().clone();
    // Discarding: all life already lost (halves rounded up: 10 and 8), P0 has chosen (no
    // cards). Sacrificing: P1's two cards already discarded, P0's choice known, no
    // creature sacrificed yet.
    assert_eq!(seen, vec![(10, 7, 0, 6, 1), (10, 7, 2, 6, 1)]);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!((creatures(&t, P0).len(), creatures(&t, P1).len()), (1, 1));
}

#[test]
fn fraying_omnipotence_triggers_resolve_afterwards() {
    cr!("608.2c", "603.3", "603.10a");
    ruling!(
        "Fraying Omnipotence",
        "Because the instructions are followed in order, abilities of the creatures a player will sacrifice may modify or trigger on losing life, discarding cards, and/or sacrificing creatures. Abilities that trigger this way will resolve after Fraying Omnipotence is done resolving."
    );
    supported("Bone Miser");
    // P0's only creature is Bone Miser ("Whenever you discard a creature card, create a
    // 2/2 black Zombie creature token."), and P0 discards a creature card before
    // sacrificing it.
    let mut t = TestGame::new(2);
    let miser = t.battlefield(P0, "Bone Miser");
    t.hand(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Fraying Omnipotence", &[]);
    t.resolve();
    assert!(!t.on_battlefield(miser), "sacrificed by the spell");
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(tokens(&t, P0).is_empty(), "the trigger hasn't resolved yet");
    assert_eq!(t.life(P0), 10);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "the Zombie came after, and stays");
}

#[test]
fn fraying_omnipotence_in_two_headed_giant_is_a_draw() {
    cr!("810.9", "810.9a", "810.8c", "104.4a");
    ruling!(
        "Fraying Omnipotence",
        "In a Two-Headed Giant game, Fraying Omnipotence causes each player to lose half of their team’s life, rounded up. This means that teams at an even life total will have 0 life, and teams at an odd life total will have -1 life. The game then ends in a draw."
    );
    let mut t = crate::r_p108_common::two_headed_giant();
    for (i, life) in [30, 30, 29, 29].into_iter().enumerate() {
        t.g.players[i].life = life;
    }
    // Before the game ends, both teams' life totals are 0 and -1.
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| (g.player(P0).life, g.player(P2).life),
    );
    t.hand(P2, "Grizzly Bears");
    cast_new(&mut t, P0, "Fraying Omnipotence", &[]);
    t.resolve_all();
    assert_eq!(seen.lock().unwrap().clone(), vec![(0, -1)]);
    assert!(matches!(t.g.result, Some(GameResult::Draw)));
}
