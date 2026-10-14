//! Rulings batch P177 — hand disruption ("target player reveals their hand ..."): Brain
//! Maggot's exile-until, Deceit's mana-spent checks, Tourach's Canticle with madness, Lost
//! Hours, Perish the Thought, Doomfall, Nightsnare, Encroach, Hollow Specter, Corpse
//! Traders, Brainbite, Thought-Stalker Warlock, Sin Collector, and revealing your own hand.

use crate::r_p177_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::mana::ManaType;
use mtg_engine::*;

/// P0's Brain Maggot enters targeting P1, whose hand is a Lightning Bolt; its trigger is
/// on the stack, with P0 choosing the Bolt. Returns (Maggot, Bolt).
fn maggot_enters(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Brain Maggot");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[obj(bolt)]);
    let maggot = t.enter(P0, "Brain Maggot");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (maggot, bolt)
}

#[test]
fn brain_maggot_exiles_the_card_until_it_leaves() {
    cr!("610.3", "610.3c");
    ruling!(
        "Brain Maggot",
        "Brain Maggot has a single ability that creates two one-shot effects: one that exiles the nonland card when the ability resolves, and another that returns the exiled card to its owner's hand immediately after Brain Maggot leaves the battlefield."
    );
    ruling!(
        "Brain Maggot",
        "The exiled card returns to its owner's hand immediately after Brain Maggot leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    let mut t = TestGame::new(2);
    let (maggot, bolt) = maggot_enters(&mut t);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    // Destroyed: the Bolt is back as the event is processed, before state-based actions
    // are checked or anything else happens.
    t.g.destroy(maggot, None);
    t.g.flush_events();
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert_eq!(t.stack_len(), 0, "not a triggered ability");
}

#[test]
fn brain_maggot_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3c");
    ruling!(
        "Brain Maggot",
        "If Brain Maggot leaves the battlefield before its enters-the-battlefield ability resolves, the opponent will reveal their hand, but no card will be exiled."
    );
    let mut t = TestGame::new(2);
    let (maggot, bolt) = maggot_enters(&mut t);
    destroy(&mut t, maggot);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
    assert!(!t.in_exile("Lightning Bolt"));
}

#[test]
fn brain_maggots_card_returns_when_its_owner_leaves_the_game() {
    cr!("800.4a", "610.3c");
    ruling!(
        "Brain Maggot",
        "In a multiplayer game, if Brain Maggot's owner leaves the game, the exiled card will return to its owner's hand."
    );
    let mut t = ffa(3);
    let (_, bolt) = maggot_enters(&mut t);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    concede(&mut t, P0);
    assert!(t.in_hand(P1, "Lightning Bolt"));
}

/// Casts Deceit from P0's hand with exactly `mana` in P0's mana pool; P1 holds Lightning
/// Bolt and Shock and controls Grizzly Bears (the bounce target).
fn cast_deceit(mana: &[(ManaType, u32)]) -> (TestGame, ObjectId) {
    supported("Deceit");
    let mut t = TestGame::new(2);
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Shock");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for (ty, n) in mana {
        crate::r_s04_common::add_mana(&mut t, P0, *ty, *n);
    }
    let deceit = t.hand(P0, "Deceit");
    t.cast(P0, deceit).go();
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.on_battlefield(deceit));
    (t, bears)
}

#[test]
fn deceit_counts_all_mana_spent_on_its_total_cost() {
    cr!("601.2h", "603.4");
    ruling!(
        "Deceit",
        "Deceit's first and second abilities care about what mana was spent to pay its total cost, not just what mana was spent to pay the hybrid mana symbols in its cost."
    );
    // {U}{U}{B}{B}{C}{C}: the hybrid symbols can't both be paid with blue and black, but
    // both abilities trigger.
    let (t, bears) = cast_deceit(&[
        (ManaType::U, 2),
        (ManaType::B, 2),
        (ManaType::C, 2),
    ]);
    assert!(t.in_hand(P1, "Grizzly Bears"), "bounced");
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.graveyard_size(P1), 1, "one card discarded");
}

#[test]
fn deceits_ability_isnt_multiplied_by_more_mana_of_that_color() {
    cr!("603.4");
    ruling!(
        "Deceit",
        "It doesn't matter if more than two mana of that color was spent; the effect isn't multiplied."
    );
    let (t, bears) = cast_deceit(&[(ManaType::B, 6)]);
    assert!(t.on_battlefield(bears), "no blue spent");
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn tourachs_canticle_two_madness_triggers_in_either_order() {
    cr!("702.35a", "603.3b");
    ruling!(
        "Tourach's Canticle",
        "If a player discards two cards with madness due to Tourach's Canticle, the madness triggered abilities may be put on the stack in either order."
    );
    supported("Tourach's Canticle");
    let mut t = TestGame::new(2);
    let temper = t.hand(P1, "Fiery Temper");
    t.hand(P1, "Basking Rootwalla");
    t.answer_choose(P0, &[obj(temper)]);
    t.answer(P1, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    let from = t.asked().len();
    cast_targeting(&mut t, P0, "Tourach's Canticle", &[Entity::Player(P1)]);
    t.resolve();
    let orders = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P1 && matches!(d, Decision::Order { .. }))
        .count();
    assert_eq!(orders, 1, "P1 orders the two madness triggers");
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn lost_hours_with_zero_or_one_card_in_library_goes_to_the_bottom() {
    cr!("401.4");
    ruling!(
        "Lost Hours",
        "If the player has zero or one card left in their library, the card is put on the bottom of that library."
    );
    supported("Lost Hours");
    for n in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.g.players[P1.idx()].library.clear();
        if n == 1 {
            t.library_top(P1, "Island");
        }
        let bolt = t.hand(P1, "Lightning Bolt");
        t.answer_choose(P0, &[obj(bolt)]);
        cast_resolve(&mut t, P0, "Lost Hours", &[Entity::Player(P1)]);
        let lib = t.g.player(P1).library.clone();
        assert_eq!(lib.len(), n + 1);
        // The library's first element is its bottom.
        assert_eq!(t.g.obj(lib[0]).chars.name.as_str(), "Lightning Bolt", "{n}");
    }
}

#[test]
fn perish_the_thought_with_an_empty_hand_doesnt_shuffle() {
    cr!("701.24a");
    ruling!(
        "Perish the Thought",
        "If the targeted player’s hand is empty, you can’t choose a card for that player to shuffle into their library. No shuffle occurs."
    );
    supported("Perish the Thought");
    let mut t = TestGame::new(2);
    t.g.players[P1.idx()].library.clear();
    for name in [
        "Island",
        "Forest",
        "Swamp",
        "Mountain",
        "Plains",
        "Grizzly Bears",
        "Hill Giant",
        "Shock",
        "Lightning Bolt",
        "Wastes",
    ] {
        t.library_top(P1, name);
    }
    let before = t.g.player(P1).library.clone();
    cast_resolve(&mut t, P0, "Perish the Thought", &[Entity::Player(P1)]);
    assert_eq!(t.g.player(P1).library, before);
    // With a card in hand, the chosen card is shuffled in.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_choose(P0, &[obj(bolt)]);
    cast_resolve(&mut t, P0, "Perish the Thought", &[Entity::Player(P1)]);
    assert_eq!(t.zone(bolt), Zone::Library(P1));
    assert_eq!(t.library_size(P1), 11);
}

#[test]
fn doomfalls_first_mode_can_exile_a_hexproof_creature() {
    cr!("702.11b", "115.1");
    ruling!(
        "Doomfall",
        "If you choose Doomfall’s first mode, the target opponent chooses which creature to exile. That creature isn’t targeted, so a creature with hexproof can be exiled this way."
    );
    supported("Doomfall");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1 chooses the Scout.
    t.answer_choose(P1, &[obj(scout)]);
    give_mana_for(&mut t, P0, "Doomfall");
    let doomfall = t.hand(P0, "Doomfall");
    t.cast(P0, doomfall)
        .modes(&[0])
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.zone(scout), Zone::Exile);
    assert!(t.on_battlefield(bears));
}

#[test]
fn doomfalls_second_mode_with_only_lands_does_nothing() {
    cr!("609.3");
    ruling!(
        "Doomfall",
        "If you choose Doomfall’s second mode, you must exile a nonland card from that player’s hand if able."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Island");
    give_mana_for(&mut t, P0, "Doomfall");
    let doomfall = t.hand(P0, "Doomfall");
    t.cast(P0, doomfall)
        .modes(&[1])
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_hand(P1, "Island"));
    // With a nonland card, it must be exiled even if P0 declines to choose.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_choose(P0, &[]);
    give_mana_for(&mut t, P0, "Doomfall");
    let doomfall = t.hand(P0, "Doomfall");
    t.cast(P0, doomfall)
        .modes(&[1])
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
}

#[test]
fn nightsnare_without_a_choice_the_player_discards_two_of_their_choice() {
    cr!("701.9b");
    ruling!(
        "Nightsnare",
        "If you don’t choose a nonland card for the player to discard, that player chooses which two cards they will discard."
    );
    supported("Nightsnare");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    let island = t.hand(P1, "Island");
    t.answer_choose(P0, &[]);
    t.answer_choose(P1, &[obj(island), obj(shock)]);
    cast_resolve(&mut t, P0, "Nightsnare", &[Entity::Player(P1)]);
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
    assert!(t.in_graveyard(P1, "Island"));
    assert!(t.in_graveyard(P1, "Shock"));
}

#[test]
fn encroach_chooses_a_nonbasic_land_card() {
    cr!("205.4c");
    ruling!("Encroach", "You choose a land card that is not a basic land.");
    supported("Encroach");
    let mut t = TestGame::new(2);
    t.hand(P1, "Plains");
    t.hand(P1, "Grizzly Bears");
    let trop = t.hand(P1, "Tropical Island");
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Encroach", &[Entity::Player(P1)]);
    for c in entity_choices_since(&t, P0, from) {
        assert_eq!(c, vec![obj(trop)]);
    }
    assert!(t.in_graveyard(P1, "Tropical Island"));
    assert!(t.in_hand(P1, "Plains"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

/// P0's Hollow Specter deals combat damage to P1 (who holds Bolt, Shock and an Island);
/// P0 pays X = `x` with untapped Swamps.
fn hollow_specter_hits(x: i64) -> TestGame {
    supported("Hollow Specter");
    let mut t = TestGame::new(2);
    let specter = t.battlefield(P0, "Hollow Specter");
    t.lands(P0, "Swamp", 3);
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Shock");
    t.hand(P1, "Island");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.attack(&[(specter, Entity::Player(P1))], &[]);
    t.resolve_all();
    t
}

#[test]
fn hollow_specter_x_is_chosen_and_paid_on_resolution_and_can_be_zero() {
    cr!("107.3", "608.2");
    ruling!(
        "Hollow Specter",
        "You decide on the value of X and pay {X} during resolution."
    );
    ruling!(
        "Hollow Specter",
        "X can be zero, but then that player discards nothing."
    );
    let t = hollow_specter_hits(0);
    assert_eq!(t.hand_size(P1), 3);
    let t = hollow_specter_hits(2);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn corpse_traders_can_sacrifice_itself() {
    cr!("602.2b", "118.3");
    ruling!(
        "Corpse Traders",
        "You can sacrifice Corpse Traders to pay the cost of its ability."
    );
    supported("Corpse Traders");
    let mut t = TestGame::new(2);
    let traders = t.battlefield(P0, "Corpse Traders");
    t.lands(P0, "Swamp", 3);
    t.hand(P1, "Lightning Bolt");
    t.answer_choose(P0, &[obj(traders)]);
    t.activate(P0, traders, 0, &[Entity::Player(P1)]).unwrap();
    assert!(!t.on_battlefield(traders));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn brainbite_can_target_an_empty_hand_and_still_draws() {
    cr!("115.1", "608.2b");
    ruling!(
        "Brainbite",
        "When Brainbite resolves, you’ll draw a card even if the targeted opponent has no cards in hand."
    );
    ruling!("Brainbite", "You may target an opponent who has no cards in hand.");
    supported("Brainbite");
    let mut t = TestGame::new(2);
    assert_eq!(t.hand_size(P1), 0);
    let before = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Brainbite", &[Entity::Player(P1)]);
    assert_eq!(t.hand_size(P0), before + 1);
}

#[test]
fn thought_stalker_warlock_checks_life_lost_not_net_change() {
    cr!("119.3");
    ruling!(
        "Thought-Stalker Warlock",
        "Thought-Stalker Warlock’s triggered ability cares whether an opponent lost life this turn, not how their life total changed."
    );
    supported("Thought-Stalker Warlock");
    let mut t = TestGame::new(2);
    t.g.gain_life(P1, 2);
    t.g.lose_life(P1, 1);
    assert_eq!(t.life(P1), 21);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Island");
    // P0 chooses the card: the "lost life" branch.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[obj(bolt)]);
    let from = t.asked().len();
    t.enter(P0, "Thought-Stalker Warlock");
    t.resolve_all();
    assert!(!entity_choices_since(&t, P0, from).is_empty(), "P0 chose");
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(t.in_hand(P1, "Island"));
}

#[test]
fn sin_collector_exiles_the_chosen_card_at_once() {
    cr!("608.2c");
    ruling!(
        "Sin Collector",
        "No player can do anything between you choosing a card and that card being exiled."
    );
    supported("Sin Collector");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[obj(bolt)]);
    t.enter(P0, "Sin Collector");
    t.settle();
    let before = t.asked().len();
    t.g.resolve_top();
    assert_eq!(t.zone(bolt), Zone::Exile);
    // P1 got no priority during the resolution.
    assert!(t.asked()[before..]
        .iter()
        .all(|(p, d)| !(*p == P1 && matches!(d, Decision::Priority { .. }))));
}

#[test]
fn targeting_yourself_reveals_your_whole_hand() {
    cr!("701.20a");
    ruling!(
        "Distress",
        "If you target yourself with this spell, you must reveal your entire hand to the other players just as any other player would."
    );
    for name in ["Distress", "Night Terrors", "Unmask", "Encroach"] {
        supported(name);
        let mut t = TestGame::new(2);
        for card in ["Lightning Bolt", "Tropical Island", "Grizzly Bears"] {
            t.hand(P0, card);
        }
        let seen = crate::r_s01_common::watch(
            &mut t,
            P0,
            |d| matches!(d, Decision::ChooseEntities { .. }),
            |g| {
                g.player(P0)
                    .hand
                    .iter()
                    .filter(|c| mtg_engine::reveal::is_revealed(g, **c))
                    .count()
            },
        );
        give_mana_for(&mut t, P0, name);
        let spell = t.hand(P0, name);
        t.cast_with(P0, spell, &[Entity::Player(P0)]).unwrap();
        t.resolve_all();
        let seen = seen.lock().unwrap().clone();
        assert!(!seen.is_empty() && seen.iter().all(|n| *n == 3), "{name}: {seen:?}");
    }
}

#[test]
fn deep_cavern_bat_exiles_until_it_leaves_and_nothing_if_it_left_first() {
    cr!("610.3", "610.3c");
    ruling!(
        "Deep-Cavern Bat",
        "If Deep-Cavern Bat leaves the battlefield before its last ability resolves, you'll still look at the target opponent's hand, but you won't exile any cards from it."
    );
    supported("Deep-Cavern Bat");
    // It resolves: the Bolt is exiled until the Bat leaves.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[obj(bolt)]);
    let bat = t.enter(P0, "Deep-Cavern Bat");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    destroy(&mut t, bat);
    assert!(t.in_hand(P1, "Lightning Bolt"));
    // The Bat leaves first: nothing is exiled.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[obj(bolt)]);
    let bat = t.enter(P0, "Deep-Cavern Bat");
    t.settle();
    destroy(&mut t, bat);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
}
