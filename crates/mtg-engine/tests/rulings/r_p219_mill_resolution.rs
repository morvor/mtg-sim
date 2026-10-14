//! Rulings batch P219 — mill (CR 701.17) as abilities resolve: last known information,
//! characteristic-defining values that count the milled cards, repeated milling, mana
//! abilities that mill, and "you can't choose to mill more than your library has".

use crate::r_p009_common::{gain, lose};
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::{in_hand_with_mana, respond};
use crate::r_s04_common::crew;
use crate::r_s05_common::{enter, tokens_with_subtype};
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::{empty_library, triggered_from};
use crate::r_s13_common::add;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn neerdiv_uses_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Neerdiv, Devious Diver",
        "If Neerdiv leaves the battlefield while its first ability is still on the stack, use its power as it last existed on the battlefield to determine how many cards the target player mills."
    );
    supported("Neerdiv, Devious Diver");
    let mut t = TestGame::new(2);
    let neerdiv = t.battlefield(P0, "Neerdiv, Devious Diver");
    add(&mut t, neerdiv, "+1/+1", 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(neerdiv, Entity::Player(P1))]);
    assert_eq!(triggered_from(&t, neerdiv), 1);
    destroy(&mut t, neerdiv);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 4);
}

#[test]
fn master_of_keys_mills_twice_x_even_if_gone() {
    cr!("107.3m", "603.10a", "608.2b");
    ruling!(
        "The Master of Keys",
        "If The Master of Keys is no longer on the battlefield when its second ability resolves, you'll still mill twice X cards."
    );
    supported("The Master of Keys");
    let mut t = TestGame::new(2);
    for land in ["Plains", "Island", "Swamp", "Wastes", "Wastes"] {
        t.lands(P0, land, 1);
    }
    let master = t.hand(P0, "The Master of Keys");
    t.cast(P0, master).x(2).go();
    t.resolve();
    let master = t.named_on_battlefield("The Master of Keys")[0];
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, master);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30 - 4);
}

#[test]
fn grist_keeps_making_insects_after_it_leaves() {
    cr!("113.7a", "608.2h", "701.17a");
    ruling!(
        "Grist, the Hunger Tide",
        "If Grist is no longer on the battlefield as its first loyalty ability resolves, you will still create a 1/1 black and green Insect creature token and mill a card. If an Insect card is milled this way, you won't be able to put a loyalty counter on Grist, but you will still repeat the process. More mill for the Grist, as it were."
    );
    supported("Grist, the Hunger Tide");
    let mut t = TestGame::new(2);
    let grist = t.battlefield(P0, "Grist, the Hunger Tide");
    // Moldgraf Millipede is an Insect card.
    stack_library(&mut t, P0, &["Moldgraf Millipede", "Forest", "Hill Giant"]);
    activate_containing(&mut t, P0, grist, "Create a 1/1").expect("activated");
    destroy(&mut t, grist);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Insect").len(), 2);
    assert!(t.in_graveyard(P0, "Moldgraf Millipede"));
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.library_size(P0), 30 + 1);
    let grist = t.g.find_in_zone(Zone::Graveyard(P0), "Grist, the Hunger Tide");
    assert_eq!(t.counters(grist[0], "loyalty"), 0);
}

/// Orders triggered abilities so that the ones whose text contains "mills" are put on the
/// stack last (and resolve first).
fn mills_resolve_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::Order { items, .. } = d else {
        return None;
    };
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by_key(|i| items[*i].contains("mills"));
    Some(Answer::Indices(order))
}

/// Orders triggered abilities so that the ones whose text contains "mills" resolve last.
fn mills_resolve_last(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::Order { items, .. } = d else {
        return None;
    };
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by_key(|i| !items[*i].contains("mills"));
    Some(Answer::Indices(order))
}

#[test]
fn master_of_lake_town_you_order_its_two_triggers() {
    cr!("603.3b", "603.10a", "701.17a");
    ruling!(
        "The Master of Lake-town",
        "If The Master of Lake-town dies at the same time that a player loses life (for example, if it dies in combat and another creature dealt combat damage to a player), you choose the order for The Master of Lake-town's two triggered abilities. If the mill effect resolves first, those cards—and also The Master of Lake-town—will be in graveyards before you determine how many cards to draw with its last ability."
    );
    supported("The Master of Lake-town");
    for (mill_first, drawn) in [(true, 1), (false, 0)] {
        let mut t = TestGame::new(2);
        let master = t.battlefield(P0, "The Master of Lake-town");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        for _ in 0..4 {
            t.graveyard(P1, "Forest");
        }
        respond(
            &mut t,
            P0,
            if mill_first {
                mills_resolve_first
            } else {
                mills_resolve_last
            },
        );
        attack_with(
            &mut t,
            &[(master, Entity::Player(P1)), (bears, Entity::Player(P1))],
        );
        block_and_finish(&mut t, P1, &[(giant, master)]);
        // P1 lost 2 life and milled 2: four Forests, the Giant and the milled cards.
        assert_eq!(t.life(P1), 18);
        assert_eq!(t.graveyard_size(P1), 7);
        assert!(t.in_graveyard(P0, "The Master of Lake-town"));
        assert_eq!(t.hand_size(P0), drawn, "mill first: {mill_first}");
    }
}

#[test]
fn war_leech_counts_the_cards_it_milled_as_it_entered() {
    cr!("614.1c", "614.12", "604.3");
    ruling!(
        "Monstrous War-Leech",
        "If Monstrous War-Leech was kicked, it enters the battlefield with power and toughness each equal to the highest mana value among cards in your graveyard, including cards that were milled due to its replacement effect. For example, if you have no cards in your graveyard as it enters and you mill three land cards and a card with mana value 4, Monstrous War-Leech is a 4/4 creature entering the battlefield."
    );
    supported("Monstrous War-Leech");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp", "Hill Giant"]);
    let leech = in_hand_with_mana(&mut t, P0, "Monstrous War-Leech");
    t.lands(P0, "Island", 1);
    t.cast(P0, leech).kicked(true).go();
    t.resolve_all();
    let leech = t.named_on_battlefield("Monstrous War-Leech")[0];
    assert_eq!(t.pt(leech), (4, 4));
    assert_eq!(t.graveyard_size(P0), 4);
}

#[test]
fn urborg_lhurgoyf_mills_as_it_enters_without_using_the_stack() {
    cr!("614.1c", "614.12", "702.33c");
    ruling!(
        "Urborg Lhurgoyf",
        "If it was kicked, Urborg Lhurgoyf enters the battlefield with a power and toughness determined by the number of creature cards in your graveyard after you have milled cards for each time it was kicked. For example, if you have one creature card in your graveyard before it enters the battlefield and you mill two more creature cards with its ability, it enters the battlefield as a 3/4 creature."
    );
    ruling!(
        "Urborg Lhurgoyf",
        "Urborg Lhurgoyf's second ability isn't a triggered ability and doesn't use the stack. Players can't cast spells or take other actions after it enters the battlefield but before its controller mills cards."
    );
    supported("Urborg Lhurgoyf");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Hill Giant", "Forest", "Gray Ogre", "Island"]);
    let goyf = in_hand_with_mana(&mut t, P0, "Urborg Lhurgoyf");
    t.lands(P0, "Island", 1);
    // Kicked once ({U}), not twice.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, goyf).go();
    t.resolve();
    let goyf = t.named_on_battlefield("Urborg Lhurgoyf")[0];
    // It's on the battlefield with the cards already milled, and nothing on the stack.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(triggered_from(&t, goyf), 0);
    assert_eq!(t.library_size(P0), 30 + 1);
    assert_eq!(t.pt(goyf), (3, 4));
}

#[test]
fn moldgraf_millipede_counts_every_creature_card_in_your_graveyard() {
    cr!("608.2c", "701.17a");
    ruling!(
        "Moldgraf Millipede",
        "Moldgraf Millipede gets a +1/+1 counter for every creature card in your graveyard, including any creature cards that were already there before you milled three cards."
    );
    supported("Moldgraf Millipede");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Hill Giant", "Forest", "Island"]);
    let millipede = enter(&mut t, P0, "Moldgraf Millipede");
    t.resolve_all();
    assert_eq!(t.counters(millipede, "+1/+1"), 2);
}

#[test]
fn halving_twice_mills_half_then_half_of_the_rest() {
    cr!("701.17a", "107.1a");
    ruling!(
        "Terisian Mindbreaker",
        "If you attack the same player with two Terisian Mindbreakers, that player will mill half their library as the first Mindbreaker's ability resolves. Then they will mill half of what remains in their library as the second Mindbreaker's ability resolves. They do not mill their entire library unless their library only had three or fewer cards to begin with."
    );
    supported("Terisian Mindbreaker");
    // (library, left after both triggers): "mills half their library, rounded up".
    for (library, left) in [(30, 7), (3, 0)] {
        let mut t = TestGame::new(2);
        empty_library(&mut t, P1);
        for _ in 0..library {
            t.library_top(P1, "Forest");
        }
        let a = t.battlefield(P0, "Terisian Mindbreaker");
        let b = t.battlefield(P0, "Terisian Mindbreaker");
        attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(t.library_size(P1), left, "{library}");
    }
}

#[test]
fn cut_your_losses_and_its_copy_halve_twice() {
    cr!("702.153a", "707.10", "701.17a");
    ruling!(
        "Cut Your Losses",
        "If you cast Cut Your Losses and copy it targeting the same player, that player will mill half their library, then mill half of what remains after that. They do not mill their entire library."
    );
    supported("Cut Your Losses");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let spell = in_hand_with_mana(&mut t, P0, "Cut Your Losses");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // 30, then 15 (rounded down), then 8.
    assert_eq!(t.library_size(P1), 8);
}

#[test]
fn persistent_petitioners_each_taps_for_one_ability() {
    cr!("602.2b", "118.3");
    ruling!(
        "Persistent Petitioners",
        "It's not possible to tap one Persistent Petitioners to activate both its first and second ability, or to activate its second ability and that of another Persistent Petitioners. This means that if you tap four Persistent Petitioners, the target player mills the top twelve cards of their library, not the top forty-eight."
    );
    let mut t = TestGame::new(2);
    let pets = t.lands(P0, "Persistent Petitioners", 4);
    t.lands(P0, "Island", 4);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, pets[0], "Tap four untapped").expect("activated");
    assert!(pets.iter().all(|p| t.obj_now(*p).tapped));
    // Neither ability of any of them can be activated again.
    for p in &pets {
        assert!(!crate::r_s02_common::can_activate(&mut t, P0, *p));
    }
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 12);
}

#[test]
fn pestilent_cauldron_counts_life_gained_ignoring_life_lost() {
    cr!("119.3", "701.17a");
    ruling!(
        "Pestilent Cauldron // Restorative Burst",
        "Pestilent Cauldron’s second ability counts the total amount of life gained without considering any life you lost during that turn. For example, if you lost 2 life and gained 2 life earlier in the turn, your opponents will mill two cards."
    );
    supported("Pestilent Cauldron // Restorative Burst");
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Pestilent Cauldron // Restorative Burst");
    t.lands(P0, "Swamp", 1);
    lose(&mut t, P0, 2);
    gain(&mut t, P0, 2);
    assert_eq!(t.life(P0), 20);
    activate_containing(&mut t, P0, cauldron, "Each opponent mills").expect("activated");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn shaun_and_rebecca_mill_trigger_waits_until_the_spell_is_cast() {
    cr!("605.3b", "603.12", "601.2i");
    ruling!(
        "Shaun & Rebecca, Agents",
        "If you activate the mana ability of Shaun & Rebecca, Agents while in the process of casting a spell, activating an ability, or during a window to activate mana abilities when you're asked to pay a cost during the resolution of a spell or ability, you won't mill cards right away. Instead, a second \"reflexive\" ability triggers whenever you activate that ability. That ability won't go onto the stack until you're done casting, activating, or resolving that spell or ability, as appropriate. Each player may respond to that triggered ability as normal."
    );
    supported("Shaun & Rebecca, Agents");
    let mut t = TestGame::new(2);
    let sr = t.battlefield(P0, "Shaun & Rebecca, Agents");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(t.obj_now(sr).tapped);
    // The spell is on the stack, the mill trigger above it; nothing was milled yet.
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.library_size(P0), 30);
    t.resolve();
    assert_eq!(t.library_size(P0), 28);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn millikin_is_not_a_mana_ability() {
    cr!("605.1a", "602.2");
    ruling!(
        "Millikin",
        "Millikin’s ability is not a mana ability. Players can respond to it, and it can’t be activated in the middle of paying for a cost. A recent update to rule 605.1a has made any abilities that move cards to or from libraries no longer mana abilities."
    );
    supported("Millikin");
    supported("Deranged Assistant");
    // Deranged Assistant ("{T}, Mill a card: Add {C}.") is the same.
    for name in ["Millikin", "Deranged Assistant"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        activate_containing(&mut t, P0, c, "Add").expect("activated");
        // The card was milled as a cost; the ability is on the stack.
        assert_eq!(t.graveyard_size(P0), 1, "{name}");
        assert_eq!(t.stack_len(), 1, "{name}");
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 0, "{name}");
        t.resolve();
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1, "{name}");
        // It isn't used to pay for a spell.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let ogre = t.hand(P0, "Gray Ogre");
        t.lands(P0, "Mountain", 1);
        assert!(t.cast(P0, ogre).try_go().is_err(), "{name}");
        assert_eq!(t.graveyard_size(P0), 0, "{name}");
    }
}

#[test]
fn fateful_tempest_counts_x_as_zero_in_milled_cards() {
    cr!("202.3e", "701.38a");
    ruling!(
        "Fateful Tempest",
        "If a milled card has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Fateful Tempest");
    let mut t = TestGame::new(2);
    // Blaze ({X}{R}) has mana value 1; Hill Giant 4.
    stack_library(&mut t, P0, &["Blaze", "Hill Giant"]);
    let tempest = in_hand_with_mana(&mut t, P0, "Fateful Tempest");
    // Both players vote past.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, tempest).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blaze") && t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn glowing_one_triggers_for_each_nonland_card_milled() {
    cr!("701.17a", "603.2c", "728.1");
    ruling!(
        "Glowing One",
        "If a player is instructed to mill more than one card, Glowing One’s last ability will trigger once for each nonland card they mill during that instruction. For example, a player with four rad counters will mill four cards at the beginning of their precombat main phase. If all four of those cards are nonland cards, Glowing One’s ability triggers four times."
    );
    supported("Glowing One");
    let mut t = TestGame::new(2);
    let one = t.battlefield(P0, "Glowing One");
    // P1 draws the first in their draw step and mills the other four.
    stack_library(
        &mut t,
        P1,
        &["Shock", "Grizzly Bears", "Hill Giant", "Opt", "Gray Ogre"],
    );
    add(&mut t, Entity::Player(P1), "rad", 4);
    t.advance_to(P1, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 4);
    assert_eq!(triggered_from(&t, one), 4);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn old_rutstein_dryad_arbor_is_both_a_land_and_a_creature() {
    cr!("205.2a", "701.17a");
    ruling!(
        "Old Rutstein",
        "If you mill Dryad Arbor, a card that's both a creature and a land, you get both a Treasure token and a 1/1 green Insect token."
    );
    supported("Old Rutstein");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Dryad Arbor");
    enter(&mut t, P0, "Old Rutstein");
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Treasure").len(), 1);
    assert_eq!(tokens_with_subtype(&t, P0, "Insect").len(), 1);
    assert!(tokens_with_subtype(&t, P0, "Blood").is_empty());
}

#[test]
fn soaring_thought_thief_mills_every_opponent() {
    cr!("701.17a", "102.2");
    ruling!(
        "Soaring Thought-Thief",
        "Soaring Thought-Thief’s fourth ability causes each opponent to mill two cards, not just those being attacked by Rogues you control."
    );
    supported("Soaring Thought-Thief");
    let mut t = TestGame::new(3);
    let thief = t.battlefield(P0, "Soaring Thought-Thief");
    attack_with(&mut t, &[(thief, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
    assert_eq!(t.graveyard_size(P2), 2);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn ursine_monstrosity_does_the_rest_with_an_empty_library() {
    cr!("701.17b", "608.2c");
    ruling!(
        "Ursine Monstrosity",
        "If you can't mill a card, you'll still follow the rest of the instructions of Ursine Monstrosity's last ability."
    );
    supported("Ursine Monstrosity");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Ursine Monstrosity");
    empty_library(&mut t, P0);
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bear), (5, 5));
    assert!(t
        .obj_now(bear)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
}

#[test]
fn mind_drain_mills_and_drains_even_without_two_cards_to_discard() {
    cr!("701.9b", "608.2c");
    ruling!(
        "Mind Drain",
        "If the target opponent can discard only one card or can’t discard any cards, they still mill a card and lose 1 life, and you gain 1 life."
    );
    supported("Mind Drain");
    for hand in [1, 0] {
        let mut t = TestGame::new(2);
        for _ in 0..hand {
            t.hand(P1, "Grizzly Bears");
        }
        let drain = in_hand_with_mana(&mut t, P0, "Mind Drain");
        t.cast(P0, drain).target(Entity::Player(P1)).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P1), 0);
        assert_eq!(t.graveyard_size(P1), 1 + hand);
        assert_eq!(t.library_size(P1), 29);
        assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    }
}

#[test]
fn demogorgons_clutches_discards_before_milling() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Demogorgon's Clutches",
        "Demogorgon's Clutches requires its target to discard cards before milling cards. This means they do not get to know what cards are milled when they choose what cards to discard."
    );
    supported("Demogorgon's Clutches");
    let mut t = TestGame::new(2);
    for name in ["Grizzly Bears", "Hill Giant", "Forest"] {
        t.hand(P1, name);
    }
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.player(P1).library.len(),
    );
    let clutches = in_hand_with_mana(&mut t, P0, "Demogorgon's Clutches");
    t.cast(P0, clutches).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![30]);
    assert_eq!(t.library_size(P1), 28);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn glacial_grasp_illegal_target_or_already_tapped() {
    cr!("608.2b", "701.26a", "608.2c");
    ruling!(
        "Glacial Grasp",
        "If the target creature is an illegal target by the time Glacial Grasp tries to resolve, the spell doesn’t resolve. No player mills or draws any cards, and the creature won’t be stopped from untapping. If the target is legal but Glacial Grasp doesn’t tap it (most likely because it’s already tapped), the rest of Glacial Grasp’s effects still happen: that creature won’t untap during its controller’s next untap step, that player mills two cards, and you draw a card."
    );
    supported("Glacial Grasp");
    // The target is gone: nothing happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let grasp = in_hand_with_mana(&mut t, P0, "Glacial Grasp");
    t.cast(P0, grasp).target(Entity::Object(bears)).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 30);
    assert_eq!(t.hand_size(P0), 0);
    // Already tapped: the rest happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    let grasp = in_hand_with_mana(&mut t, P0, "Glacial Grasp");
    t.cast(P0, grasp).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
    assert_eq!(t.hand_size(P0), 1);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn zulaport_duelist_nothing_is_milled_if_the_target_is_gone() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Zulaport Duelist",
        "If the target creature is an illegal target by the time Zulaport Duelist’s ability tries to resolve, the ability doesn’t resolve. No player mills two cards."
    );
    supported("Zulaport Duelist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    enter(&mut t, P0, "Zulaport Duelist");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 30);
    // With the target there, its controller mills two.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    enter(&mut t, P0, "Zulaport Duelist");
    t.resolve_all();
    assert_eq!(t.library_size(P1), 28);
}

#[test]
fn you_cant_choose_to_mill_more_than_your_library_has() {
    cr!("701.17b");
    ruling!(
        "Daggerfang Duo",
        "If you have one or fewer cards in your library, you can’t choose to mill two cards."
    );
    ruling!(
        "Lasyd Prowler",
        "If you don’t have at least as many cards in your library as lands you control when Lasyd Prowler’s first ability resolves, you can’t choose to mill that many cards."
    );
    ruling!(
        "Hedge Shredder",
        "If your library contains one or fewer cards when Hedge Shredder's first ability begins to resolve, you can't choose to mill two cards."
    );
    ruling!(
        "Rampant Frogantua",
        "If the number of cards remaining in your library is less than the amount of combat damage dealt to a player by Rampant Frogantua, you can't choose to mill that many cards with Rampant Frogantua's last ability."
    );
    for name in [
        "Daggerfang Duo",
        "Lasyd Prowler",
        "Hedge Shredder",
        "Rampant Frogantua",
    ] {
        supported(name);
        for (library, mills) in [(1, false), (3, true)] {
            let mut t = TestGame::new(2);
            empty_library(&mut t, P0);
            for _ in 0..library {
                t.library_top(P0, "Hill Giant");
            }
            t.answer_yes(P0, true);
            match name {
                "Daggerfang Duo" => {
                    enter(&mut t, P0, name);
                }
                "Lasyd Prowler" => {
                    // Lasyd Prowler: three lands, so "mill three cards".
                    t.lands(P0, "Forest", if library == 1 { 2 } else { 3 });
                    enter(&mut t, P0, name);
                }
                "Hedge Shredder" => {
                    let shredder = t.battlefield(P0, name);
                    let bears = t.battlefield(P0, "Grizzly Bears");
                    assert!(crew(&mut t, P0, shredder, &[bears]));
                    t.resolve_all();
                    attack_with(&mut t, &[(shredder, Entity::Player(P1))]);
                }
                _ => {
                    // 3 combat damage: "you may mill that many cards".
                    let frog = t.battlefield(P0, name);
                    attack_with(&mut t, &[(frog, Entity::Player(P1))]);
                    block_and_finish(&mut t, P1, &[]);
                }
            }
            t.resolve_all();
            let milled = t.library_size(P0) < library;
            assert_eq!(milled, mills, "{name} with {library}");
        }
    }
}
