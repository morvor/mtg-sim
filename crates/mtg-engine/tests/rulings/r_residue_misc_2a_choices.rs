//! Item `rulings-residue-misc-2`, pass A — effects in which each player chooses in turn
//! order and the results happen at once (CR 101.4, 608.2d–e, 701.21a), and other
//! "each player" resolutions: Balance, Restore Balance, Fortunate Few, Deadly Brew,
//! Tragic Arrogance, Smokestack, Warp World, Flood of Tears, Barrin, Decoy Gambit, Trial of
//! Agony and Fight or Flight.

use crate::r_residue_misc_2a_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The `ChooseEntities` decisions asked since `from`: who, the candidates, and the most
/// that could be chosen.
fn entity_asks(t: &TestGame, from: usize) -> Vec<(PlayerId, Vec<Entity>, u32)> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                candidates, max, ..
            } => Some((p, candidates, max)),
            _ => None,
        })
        .collect()
}

/// The number of permanents of the type on the battlefield.
fn count(g: &mtg_engine::game::Game, ty: CardType) -> usize {
    g.permanents().filter(|o| o.is(ty)).count()
}

/// Lands, creatures and cards in hands as `g` is now.
fn census(g: &mtg_engine::game::Game) -> (usize, usize, usize) {
    (
        count(g, CardType::Land),
        count(g, CardType::Creature),
        g.players.iter().map(|p| p.hand.len()).sum(),
    )
}

/// Records the census whenever any of the players makes a `ChooseEntities` decision.
fn watch_choices(t: &mut TestGame, n: usize) -> Vec<Seen<(usize, usize, usize)>> {
    (0..n)
        .map(|i| {
            crate::r_s01_common::watch(
                t,
                PlayerId(i as u8),
                |d| matches!(d, Decision::ChooseEntities { .. }),
                census,
            )
        })
        .collect()
}

type Seen<T> = crate::r_s01_common::Seen<T>;

// --- Balance and Restore Balance ------------------------------------------------------------

/// A four-player game on P1's turn: P3 has the fewest lands (1), creatures (1) and cards
/// in hand (1); P0, P1 and P2 have more of each.
fn balance_table() -> TestGame {
    let mut t = TestGame::new(4);
    t.set_step(P1, Step::PrecombatMain);
    for (p, n) in [(P0, 2), (P1, 3), (P2, 4), (P3, 1)] {
        t.lands(p, "Wastes", n);
        for _ in 0..n {
            t.battlefield(p, "Grizzly Bears");
            t.hand(p, "Island");
        }
    }
    t
}

/// The choices made since `from` (by the players with more than one option), in order,
/// as (player, what was chosen among: 'L'and, 'C'reature, 'H'and).
fn choice_order(t: &TestGame, from: usize) -> Vec<(PlayerId, char)> {
    entity_asks(t, from)
        .into_iter()
        .filter(|(p, c, _)| *p != P3 && c.len() > 1)
        .map(|(p, c, _)| {
            let o = c[0].object().expect("an object");
            let o = t.g.obj(o);
            let kind = if matches!(o.zone, Zone::Hand(_)) {
                'H'
            } else if o.is(CardType::Land) {
                'L'
            } else {
                'C'
            };
            (p, kind)
        })
        .collect()
}

/// Checks a Balance-like resolution: the players choose in turn order starting with the
/// active player (P1), one kind after the other in `kinds` order, and each kind's
/// unchosen permanents or cards go only once everyone has chosen.
fn check_balance(
    t: &TestGame,
    from: usize,
    kinds: [char; 3],
    seen: &[Seen<(usize, usize, usize)>],
) {
    let order = choice_order(t, from);
    let expected: Vec<(PlayerId, char)> = kinds
        .iter()
        .flat_map(|k| [P1, P2, P0].into_iter().map(move |p| (p, *k)))
        .collect();
    assert_eq!(order, expected);
    // The census as each of P1, P2 and P0 chose (in that order, for each kind).
    let at = |p: PlayerId, i: usize| {
        let s = seen[p.idx()].lock().unwrap();
        let mine: Vec<_> = s.iter().copied().collect();
        mine[i]
    };
    let initial = (10, 10, 10);
    for (k, kind) in kinds.iter().enumerate() {
        let views: Vec<_> = [P1, P2, P0].iter().map(|p| at(*p, k)).collect();
        assert!(views.windows(2).all(|w| w[0] == w[1]), "{kind}: {views:?}");
        let (l, c, h) = views[0];
        let done: Vec<char> = kinds[..k].to_vec();
        assert_eq!(l, if done.contains(&'L') { 4 } else { initial.0 }, "{kind}");
        assert_eq!(c, if done.contains(&'C') { 4 } else { initial.1 }, "{kind}");
        assert_eq!(h, if done.contains(&'H') { 4 } else { initial.2 }, "{kind}");
    }
    assert_eq!(census(&t.g), (4, 4, 4));
}

#[test]
fn balance_players_choose_in_turn_order_and_lose_the_rest_at_once() {
    cr!("101.4", "101.4a", "608.2e");
    ruling!(
        "Balance",
        "First the player whose turn it is chooses which lands (if any) to keep, then each other player in turn order does the same. Each player will know the choices made by the players who chose before them. All of the unchosen lands are then sacrificed simultaneously. Then the process is repeated for cards in hand, except that no cards are revealed until all players have chosen what to discard, at which point those cards are all discarded simultaneously. Lastly, the process is repeated for creatures, and players will again know earlier choices made when deciding what to sacrifice. All of the unchosen creatures are then sacrificed simultaneously."
    );
    supported("Balance");
    let mut t = balance_table();
    add(&mut t, P1, mtg_engine::mana::ManaType::W, 2);
    let balance = t.hand(P1, "Balance");
    t.cast(P1, balance).go();
    t.settle();
    let seen = watch_choices(&mut t, 4);
    let from = t.asked().len();
    t.resolve_all();
    check_balance(&t, from, ['L', 'H', 'C'], &seen);
}

#[test]
fn restore_balance_lands_then_creatures_then_cards_in_hand() {
    cr!("101.4", "101.4a", "608.2e", "702.62a");
    ruling!(
        "Restore Balance",
        "As Restore Balance resolves, first the player whose turn it is chooses the appropriate number of lands they control, then each other player in turn order does the same knowing the choices made before them. Then all the lands that weren't chosen are sacrificed at the same time. Then, starting with the player whose turn it is again, each player chooses the appropriate number of creatures they control, then sacrifice their other creatures after all choices have been made. Finally, players choose cards in hand without revealing them in the same order and discard the remaining cards."
    );
    supported("Restore Balance");
    let mut t = balance_table();
    let rb = suspended_restore_balance(&mut t, P1);
    let seen = watch_choices(&mut t, 4);
    let from = t.asked().len();
    cast_suspended(&mut t, P1, rb);
    t.resolve_all();
    check_balance(&t, from, ['L', 'C', 'H'], &seen);
}

/// `p`'s Restore Balance, suspended (its {W} paid), with one time counter left.
fn suspended_restore_balance(t: &mut TestGame, p: PlayerId) -> ObjectId {
    use mtg_engine::decision::{Action, SpecialAction};
    add(t, p, mtg_engine::mana::ManaType::W, 1);
    let card = t.hand(p, "Restore Balance");
    t.g.turn.priority = Some(p);
    t.g.perform_action(p, Action::Special(SpecialAction::Suspend { card }))
        .expect("suspended");
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.g.remove_counters(obj(exiled), "time", 5);
    exiled
}

/// Removes the last time counter from the suspended card: its owner casts it.
fn cast_suspended(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    t.g.remove_counters(obj(card), "time", 1);
    t.g.flush_events();
    t.answer_yes(p, true);
    t.settle();
    t.resolve(); // the suspend trigger: cast it
    assert!(t
        .g
        .stack
        .iter()
        .any(|s| t.g.obj(*s).chars.name == "Restore Balance"));
}

#[test]
fn restore_balance_triggers_wait_and_sacrificed_creatures_see_only_the_lands() {
    cr!("608.2d", "603.10a", "603.3");
    ruling!(
        "Restore Balance",
        "Any abilities of the creatures sacrificed may modify or trigger on the lands being sacrificed, but won't modify or trigger on the cards being discarded. Abilities of the sacrificed lands won't modify or trigger on the creatures being sacrificed or the cards being discarded."
    );
    ruling!(
        "Restore Balance",
        "Any abilities that trigger during this process will wait to be put onto the stack until Restore Balance is done resolving."
    );
    supported("Titania, Protector of Argoth");
    supported("Bone Miser");
    let mut t = TestGame::new(2);
    // P0: three lands, Titania, Bone Miser and a Bears, three cards in hand; P1: one land,
    // one creature, no cards.
    t.lands(P0, "Forest", 3);
    t.battlefield(P0, "Titania, Protector of Argoth");
    t.battlefield(P0, "Bone Miser");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Wastes");
    t.battlefield(P1, "Savannah Lions");
    let rb = suspended_restore_balance(&mut t, P0);
    for _ in 0..2 {
        t.hand(P0, "Hill Giant");
    }
    t.hand(P0, "Swamp");
    let forests: Vec<Entity> =
        t.g.permanents()
            .filter(|o| o.chars.name == "Forest")
            .map(|o| obj(o.id))
            .collect();
    t.answer_choose(P0, &forests[..1]);
    t.answer_choose(P0, &[obj(bears)]);
    cast_suspended(&mut t, P0, rb);
    t.g.resolve_top();
    t.g.flush_events();
    // Nothing went on the stack while it resolved; Titania's two triggers (for the two
    // Forests) are put on the stack now.
    let titania = |t: &TestGame| crate::r_s01_common::triggers_on_stack(t, "Elemental");
    assert_eq!(titania(&t), 0);
    t.settle();
    assert_eq!(titania(&t), 2, "{}", t.dump_log());
    t.resolve_all();
    // Two 5/3 Elementals, created after Restore Balance finished (not sacrificed); Bone
    // Miser, gone before the discards, made no Zombies, mana or cards.
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().all(|id| t.pt(*id) == (5, 3)));
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.g.player(P0).mana_pool.is_empty());
}

// --- Fortunate Few ----------------------------------------------------------------------------------

#[test]
fn fortunate_few_choices_in_turn_order_without_actions_and_indestructible_may_be_chosen() {
    cr!("101.4", "608.2e", "702.12b");
    ruling!(
        "Fortunate Few",
        "A permanent with indestructible can be chosen this way even though it wouldn’t be destroyed otherwise."
    );
    ruling!(
        "Fortunate Few",
        "After you choose a nonland permanent, each other player does so in turn order starting from the player whose turn it is. Each player will know the choice made by previous players."
    );
    ruling!(
        "Fortunate Few",
        "Once Fortunate Few begins resolving, no player may take any actions other than those specified by Fortunate Few until it’s done resolving."
    );
    supported("Fortunate Few");
    // P0 casts it: P0 chooses first, then P1 and P2 in turn order.
    let mut t = TestGame::new(3);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let giant = t.battlefield(P2, "Hill Giant");
    let wurm = t.battlefield(P2, "Craw Wurm");
    t.answer_choose(P0, &[obj(myr)]);
    t.answer_choose(P1, &[obj(lions)]);
    t.answer_choose(P2, &[obj(bears)]);
    let ff = cast_new(&mut t, P0, "Fortunate Few", &[]);
    t.settle();
    let from = t.asked().len();
    t.resolve_all();
    let asks = entity_asks(&t, from);
    let who: Vec<PlayerId> = asks.iter().map(|a| a.0).collect();
    assert_eq!(who, vec![P0, P1, P2]);
    // P0 could choose the indestructible Myr; P1 can't choose it again.
    assert!(asks[0].1.contains(&obj(myr)));
    assert!(!asks[1].1.contains(&obj(myr)));
    assert!(!asks[2].1.contains(&obj(lions)));
    // Nobody got priority while it resolved.
    assert!(
        asked_since(&t, from)
            .iter()
            .take_while(|(_, d)| !matches!(d, Decision::Priority { .. }))
            .count()
            >= 3
    );
    assert!(t.on_battlefield(myr) && t.on_battlefield(lions) && t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant) && !t.on_battlefield(wurm));
    let _ = ff;
}

// --- Deadly Brew -----------------------------------------------------------------------------------

#[test]
fn deadly_brew_sacrifices_chosen_in_turn_order_then_the_return_choice() {
    cr!("101.4", "608.2e", "701.21a");
    ruling!(
        "Deadly Brew",
        "First the active player chooses which creature or planeswalker they'll sacrifice, then each other player in turn order does the same, knowing choices made before their choice. Then all those permanents are sacrificed simultaneously."
    );
    ruling!(
        "Deadly Brew",
        "If you cast Deadly Brew and control any creatures or planeswalkers as it resolves, you must sacrifice one of them."
    );
    ruling!(
        "Deadly Brew",
        "You choose which permanent card you're returning to your hand, if any, as Deadly Brew resolves, after permanents are sacrificed."
    );
    supported("Deadly Brew");
    // P1's turn; P1 casts it: P1 chooses first, then P2, then P0.
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let mine = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Savannah Lions");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P2, "Llanowar Elves");
    t.battlefield(P2, "Air Elemental");
    let old = t.graveyard(P1, "Wall of Omens");
    let seen = watch_choices(&mut t, 3);
    // P1 queues no choice of what to sacrifice (it must sacrifice one anyway); it returns
    // the Wall.
    lands_for_cost(&mut t, P1, "Deadly Brew");
    let brew = t.hand(P1, "Deadly Brew");
    t.g.turn.priority = Some(P1);
    t.g.cast_spell(P1, brew, mtg_engine::object::CastMethod::Normal)
        .expect("cast");
    t.g.flush_events();
    t.answer_yes(P1, true);
    let from = t.asked().len();
    // The return choice: P1 picks the Wall (queued after the sacrifices are chosen by
    // default).
    let order_check = from;
    t.answer(P1, DecisionKind::Entities, Answer::Default);
    t.answer(P1, DecisionKind::Entities, Answer::Entities(vec![obj(old)]));
    t.resolve_all();
    let asks = entity_asks(&t, order_check);
    let sac: Vec<PlayerId> = asks.iter().take(3).map(|a| a.0).collect();
    assert_eq!(sac, vec![P1, P2, P0], "{asks:?}");
    // Six creatures on the battlefield while each chose.
    for p in [P0, P1, P2] {
        let s = seen[p.idx()].lock().unwrap();
        assert_eq!(s[0].1, 6, "{p:?}");
    }
    // One each was sacrificed (P1 too, with no choice queued); the Wall came back.
    assert_eq!(crate::r_s01_common::creatures(&t, P1).len(), 1);
    assert_eq!(crate::r_s01_common::creatures(&t, P0).len(), 1);
    assert_eq!(crate::r_s01_common::creatures(&t, P2).len(), 1);
    assert!(t.in_hand(P1, "Wall of Omens"), "{}", t.dump_log());
    // The return choice came after the sacrifices: the sacrificed card was in the
    // graveyard by then and wasn't offered ("another permanent card").
    let gone = t.g.player(P1).graveyard.iter().copied().find(|id| {
        let n = &t.g.obj(*id).chars.name;
        n == "Grizzly Bears" || n == "Savannah Lions"
    });
    let gone = gone.expect("P1 sacrificed one");
    let last = asks.last().unwrap();
    assert!(last.1.contains(&obj(old)), "{asks:?}");
    assert!(!last.1.contains(&obj(gone)));
    let _ = mine;
}

// --- Tragic Arrogance ----------------------------------------------------------------------------------

#[test]
fn tragic_arrogance_one_permanent_may_be_chosen_for_two_types() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Tragic Arrogance",
        "If a permanent has more than one of the affected types, it can count for any of them."
    );
    supported("Tragic Arrogance");
    let mut t = TestGame::new(2);
    let satyr = t.battlefield(P1, "Boon Satyr");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    // P0 has nothing; for P1, the Satyr is chosen as the creature and as the enchantment.
    for _ in 0..4 {
        t.answer_choose(P0, &[obj(satyr)]);
    }
    cast_new(&mut t, P0, "Tragic Arrogance", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(satyr), "{}", t.dump_log());
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(anthem));
}

// --- Smokestack ---------------------------------------------------------------------------------------------

#[test]
fn smokestack_a_player_without_enough_permanents_sacrifices_them_all() {
    cr!("701.21a", "603.2");
    ruling!(
        "Smokestack",
        "If a player does not have enough permanents, they sacrifice all the ones they have."
    );
    supported("Smokestack");
    let mut t = TestGame::new(2);
    let stack = t.battlefield(P0, "Smokestack");
    t.g.add_counters(obj(stack), "soot", 3, None);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Forest");
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.permanents().filter(|o| o.controller == P1).count(), 0);
    assert!(t.on_battlefield(stack));
}

// --- Warp World ------------------------------------------------------------------------------------------------

/// Replaces `p`'s library with `n` copies of `name`.
fn library_of(t: &mut TestGame, p: PlayerId, name: &str, n: usize) {
    t.g.players[p.idx()].library.clear();
    for _ in 0..n {
        t.library_top(p, name);
    }
}

/// P0 casts Warp World with mana from its pool (no lands of its own), and it resolves.
fn warp_world(t: &mut TestGame) {
    add(t, P0, mtg_engine::mana::ManaType::R, 8);
    let ww = t.hand(P0, "Warp World");
    t.cast(P0, ww).go();
    t.resolve_all();
}

#[test]
fn warp_world_counts_permanents_owned_tokens_included_and_reveals_that_many() {
    cr!("608.2c", "111.7", "111.8");
    ruling!(
        "Warp World",
        "1) Each player counts the number of permanents they own."
    );
    ruling!(
        "Warp World",
        "2) Each player shuffles those permanents into their library."
    );
    ruling!(
        "Warp World",
        "3) Each player reveals cards from the top of their library equal to the number that player counted."
    );
    ruling!(
        "Warp World",
        "Tokens are permanents but not cards. They'll count toward the number of permanents shuffled into your library, so you'll get a card back for each token you owned."
    );
    ruling!(
        "Warp World",
        "6) Each player puts all of their other revealed cards (instants, sorceries, planeswalkers, and Auras that can't enchant anything) on the bottom of their library in any order."
    );
    supported("Warp World");
    let mut t = TestGame::new(2);
    // P0 owns a Forest, a Bears and a token (one it controls); its library: Forests.
    library_of(&mut t, P0, "Forest", 8);
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    // P1 owns only a token; its library: Shocks.
    library_of(&mut t, P1, "Shock", 6);
    crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    warp_world(&mut t);
    t.resolve_all();
    // P0 counted 3 (the token too): three cards came back, none of them tokens.
    let p0: Vec<_> =
        t.g.permanents()
            .filter(|o| o.owner == P0)
            .map(|o| o.id)
            .collect();
    assert_eq!(p0.len(), 3, "{}", t.dump_log());
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(t.library_size(P0), 8 + 2 - 3);
    // P1 counted 1, revealed a Shock and put it on the bottom: nothing on the battlefield,
    // all six still in its library.
    assert_eq!(t.g.permanents().filter(|o| o.owner == P1).count(), 0);
    assert_eq!(t.library_size(P1), 6);
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn warp_world_counts_a_commander_that_went_to_the_command_zone() {
    cr!("903.9b", "608.2c");
    ruling!(
        "Warp World",
        "If a permanent leaves the battlefield this way but ends up in a zone other than a library (most likely because it's a player's commander in the Commander variant), it's still counted to determine how many cards to reveal."
    );
    let mut t = crate::r_p114_common::commander_game_n(2);
    library_of(&mut t, P0, "Forest", 8);
    t.battlefield(P0, "Forest");
    let cmd = crate::r_p114_common::commander_on_battlefield(&mut t, P0, "Isamaru, Hound of Konda");
    t.answer_yes(P0, true);
    warp_world(&mut t);
    t.resolve_all();
    assert_eq!(t.zone(cmd), Zone::Command, "{}", t.dump_log());
    // Two counted: two Forests came back.
    let forests =
        t.g.permanents()
            .filter(|o| o.owner == P0 && o.chars.name == "Forest")
            .count();
    assert_eq!(forests, 2);
}

// --- Flood of Tears --------------------------------------------------------------------------------------------

/// P0 controls four nontoken nonland permanents (and casts Flood of Tears).
fn flood() -> (TestGame, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let mine = vec![
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Savannah Lions"),
        t.battlefield(P0, "Bonesplitter"),
        t.battlefield(P0, "Glorious Anthem"),
    ];
    (t, mine)
}

#[test]
fn flood_of_tears_may_put_back_one_of_the_returned_cards() {
    cr!("608.2c", "400.7");
    ruling!(
        "Flood of Tears",
        "If you return four or more nontoken permanents, the permanent card you put onto the battlefield may be one of the cards returned to your hand this way."
    );
    supported("Flood of Tears");
    let (mut t, mine) = flood();
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(mine[0])]);
    add(&mut t, P0, mtg_engine::mana::ManaType::U, 6);
    let flood = t.hand(P0, "Flood of Tears");
    t.cast(P0, flood).go();
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Grizzly Bears").len(),
        1,
        "{}",
        t.dump_log()
    );
    assert!(t.in_hand(P0, "Savannah Lions"));
}

#[test]
fn flood_of_tears_counts_a_commander_that_went_to_the_command_zone() {
    cr!("903.9b", "608.2c");
    ruling!(
        "Flood of Tears",
        "If a permanent would be returned to your hand but ends up in another zone (most likely because an effect exiled it instead or because it's a player's commander in the Commander variant) Flood of Tears counts it when determining whether four or more of your nontoken permanents were returned."
    );
    let mut t = crate::r_p114_common::commander_game_n(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Savannah Lions");
    t.battlefield(P0, "Bonesplitter");
    let cmd = crate::r_p114_common::commander_on_battlefield(&mut t, P0, "Isamaru, Hound of Konda");
    // The commander goes to the command zone; then the extra permanent is put in.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.hand(P0, "Craw Wurm");
    let wurm: Vec<Entity> =
        t.g.find_in_zone(Zone::Hand(P0), "Craw Wurm")
            .into_iter()
            .map(obj)
            .collect();
    t.answer_choose(P0, &wurm);
    add(&mut t, P0, mtg_engine::mana::ManaType::U, 6);
    let flood = t.hand(P0, "Flood of Tears");
    t.cast(P0, flood).go();
    t.resolve_all();
    assert_eq!(t.zone(cmd), Zone::Command);
    assert_eq!(
        t.named_on_battlefield("Craw Wurm").len(),
        1,
        "{}",
        t.dump_log()
    );
}

#[test]
fn flood_of_tears_leave_and_enter_triggers_go_on_the_stack_together() {
    cr!("603.3", "603.3b", "603.10a");
    ruling!(
        "Flood of Tears",
        "Any abilities that trigger as the nonland permanents are returned are put onto the stack at the same time as any abilities that trigger as the permanent card is put onto the battlefield (if applicable)."
    );
    supported("Faceless Butcher");
    let (mut t, mine) = flood();
    // P1's Faceless Butcher had exiled P1's Hill Giant (its leave trigger returns it); P0
    // puts Mulldrifter onto the battlefield (its enter trigger draws two).
    let butcher = t.battlefield(P1, "Faceless Butcher");
    let _ = mine;
    t.hand(P0, "Mulldrifter");
    let drifter: Vec<Entity> =
        t.g.find_in_zone(Zone::Hand(P0), "Mulldrifter")
            .into_iter()
            .map(obj)
            .collect();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &drifter);
    add(&mut t, P0, mtg_engine::mana::ManaType::U, 6);
    let flood = t.hand(P0, "Flood of Tears");
    t.cast(P0, flood).go();
    t.g.settle();
    t.g.resolve_top();
    t.g.flush_events();
    assert!(t.g.stack.is_empty());
    t.settle();
    // Both triggers wait for the spell to finish and go on the stack together.
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    let _ = butcher;
}

// --- Barrin, Tolarian Archmage -------------------------------------------------------------------------

#[test]
fn barrin_draws_one_card_however_many_permanents_went_to_hand() {
    cr!("603.4", "513.1");
    ruling!(
        "Barrin, Tolarian Archmage",
        "The permanent that was returned to your hand doesn't have to still be there."
    );
    ruling!(
        "Barrin, Tolarian Archmage",
        "You draw only one card, no matter how many permanents were put into your hand from the battlefield during that turn."
    );
    supported("Barrin, Tolarian Archmage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Barrin, Tolarian Archmage");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    for x in [a, b] {
        cast_new(&mut t, P0, "Unsummon", &[obj(x)]);
        t.resolve_all();
    }
    // One of them is cast again, the other discarded: neither is in hand any more.
    t.lands(P0, "Plains", 1);
    let lions = t.g.find_in_zone(Zone::Hand(P0), "Savannah Lions")[0];
    t.cast(P0, lions).go();
    t.resolve_all();
    let bears = t.g.find_in_zone(Zone::Hand(P0), "Grizzly Bears")[0];
    t.g.discard(P0, bears, None);
    t.g.flush_events();
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// --- Decoy Gambit -----------------------------------------------------------------------------------------

#[test]
fn decoy_gambit_opponents_choose_from_the_active_or_next_opponent() {
    cr!("101.4", "608.2e");
    ruling!(
        "Decoy Gambit",
        "As the Decoy Gambit resolves, first the next opponent in turn order (or, if it’s an opponent’s turn, the opponent whose turn it is) chooses whether you’ll draw a card or return their creature that was targeted to its owner’s hand, then each other opponent in turn order does so knowing the choices made before them. After all choices are made, you draw the appropriate number of cards. After you’ve drawn, the appropriate creatures are all simultaneously returned to their owners’ hands."
    );
    for (active, order) in [(P0, [P1, P2, P3]), (P2, [P2, P3, P1])] {
        let mut t = TestGame::new(4);
        t.set_step(active, Step::PrecombatMain);
        // (The target slots are in the order the opponents choose.)
        let targets: Vec<ObjectId> = order
            .iter()
            .map(|p| t.battlefield(*p, "Grizzly Bears"))
            .collect();
        lands_for_cost(&mut t, P0, "Decoy Gambit");
        let gambit = t.hand(P0, "Decoy Gambit");
        for x in &targets {
            t.answer_targets(P0, &[obj(*x)]);
        }
        t.g.turn.priority = Some(P0);
        t.g.cast_spell(P0, gambit, mtg_engine::object::CastMethod::Normal)
            .expect("cast");
        t.g.flush_events();
        let from = t.asked().len();
        let hand = t.hand_size(P0);
        t.resolve_all();
        let who: Vec<PlayerId> = asked_since(&t, from)
            .into_iter()
            .filter(|(p, d)| *p != P0 && !matches!(d, Decision::Priority { .. }))
            .map(|(p, _)| p)
            .collect();
        assert_eq!(who, order.to_vec(), "active {active:?}");
        // By default nobody lets P0 draw: all three Bears are returned.
        assert_eq!(t.hand_size(P0), hand);
        assert!(
            targets.iter().all(|x| !t.on_battlefield(*x)),
            "{}",
            t.dump_log()
        );
    }
}

// --- Trial of Agony ---------------------------------------------------------------------------------------

#[test]
fn trial_of_agony_with_one_illegal_target_the_other_takes_the_damage() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Trial of Agony",
        "If one of the two target creatures is an illegal target when Trial of Agony resolves, the remaining creature's controller must choose the remaining creature. Trial of Agony will deal 5 damage to that creature."
    );
    supported("Trial of Agony");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let trial = cast_new(&mut t, P0, "Trial of Agony", &[obj(wurm), obj(giant)]);
    let _ = trial;
    // The Giant leaves; P1 would rather choose it, but must choose the Wurm.
    cast_new(&mut t, P1, "Unsummon", &[obj(giant)]);
    t.resolve();
    t.answer_choose(P1, &[obj(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"), "{}", t.dump_log());
}

// --- Fight or Flight --------------------------------------------------------------------------------------

/// P1's turn's beginning of combat with P0's `n` Fight or Flights; P0 separates P1's
/// creatures with `piles` (the first pile of each separation) and P1 picks `picks`.
fn fight_or_flight(t: &mut TestGame, n: usize, piles: &[Vec<ObjectId>], picks: &[usize]) {
    for _ in 0..n {
        t.battlefield(P0, "Fight or Flight");
    }
    for p in piles {
        let es: Vec<Entity> = p.iter().map(|x| obj(*x)).collect();
        t.answer_choose(P0, &es);
    }
    for i in picks {
        t.answer(P1, DecisionKind::Option, Answer::Index(*i));
    }
    t.set_step(P1, Step::PrecombatMain);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.resolve_all();
}

#[test]
fn fight_or_flight_twice_only_creatures_in_both_chosen_piles_attack() {
    cr!("700.3", "508.1c");
    ruling!(
        "Fight or Flight",
        "If you have two of these, then you will separate the creatures into piles twice, and the opponent will choose a pile each time. Only creatures that were in both of the chosen piles can attack."
    );
    ruling!(
        "Fight or Flight",
        "The effect is best understood if you read it as \"creatures not in the chosen pile can't attack.\""
    );
    supported("Fight or Flight");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Savannah Lions");
    // First: {a, b} / {c}, P1 picks {a, b}; second: {b, c} / {a}, P1 picks {b, c}.
    fight_or_flight(&mut t, 2, &[vec![a, b], vec![b, c]], &[0, 0]);
    let p0 = pl(P0);
    use crate::r_s09_common::legal_attack;
    assert!(legal_attack(&mut t, &[(b, p0)]));
    assert!(!legal_attack(&mut t, &[(a, p0)]));
    assert!(!legal_attack(&mut t, &[(c, p0)]));
}

#[test]
fn fight_or_flight_creatures_arriving_later_are_in_no_pile() {
    cr!("700.3", "508.1c");
    ruling!(
        "Fight or Flight",
        "Creatures which enter (or cards which become creatures) after the beginning of combat ability resolves are not placed in either pile. Since they weren't in the chosen pile, they won't be able to attack."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    fight_or_flight(&mut t, 1, &[vec![a]], &[0]);
    // A hasty creature arriving after the ability resolved can't attack.
    let late = t.battlefield(P1, "Raging Goblin");
    let p0 = pl(P0);
    use crate::r_s09_common::legal_attack;
    assert!(legal_attack(&mut t, &[(a, p0)]));
    assert!(!legal_attack(&mut t, &[(b, p0)]));
    assert!(!legal_attack(&mut t, &[(late, p0)]));
}
