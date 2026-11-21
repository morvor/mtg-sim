//! Rulings batch P104 — assorted rulings of cards that mill: piles (Curator of Destinies,
//! Fortune's Favor, Steam Augury), graveyard-dependent power and toughness (Carrion Grub,
//! Billowing Shriekmass, Aven Heartstabber, Snooping Newsie), Blossoming Tortoise, Icetill
//! Explorer, Curator Beastie, Valakut Exploration, Sagas, combat (Song of Blood, Ursine
//! Monstrosity, Grisly Anglerfish, Harald Unites the Elves), Impromptu Raid, Wondrous
//! Crucible, Does Machines and Rot Farm Skeleton.

use crate::r_p104_common::*;
use crate::r_p146_common::enter_together;
use crate::r_p189_common::set_library;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_attack, can_play_land, destroy};
use crate::r_s05_common::enter;
use crate::r_s06_common::{damage, give_control, has_kw};
use crate::r_s19_common::{add_lore, gain_level};
use crate::r_s33_common::shuffled_since;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj_lore(t: &mut TestGame, saga: ObjectId, n: u32) {
    t.g.objects[saga.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::LORE.into(), n);
}

// ---------------------------------------------------------------------------
// Piles
// ---------------------------------------------------------------------------

#[test]
fn curator_of_destinies_can_be_targeted_by_a_counterspell() {
    cr!("701.6a", "608.2b");
    ruling!(
        "Curator of Destinies",
        "A spell or ability that counters spells can still target Curator of Destinies. When that spell or ability resolves, Curator of Destinies won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    supported("Curator of Destinies");
    supported("Exclude");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    t.lands(P1, "Island", 3);
    let c = t.hand(P0, "Curator of Destinies");
    let spell = t.cast(P0, c).go();
    let ex = t.hand(P1, "Exclude");
    t.cast(P1, ex).target(obj(spell)).go();
    let hand = t.hand_size(P1);
    t.resolve();
    // Exclude resolved: its draw happened, and Curator is still on the stack.
    assert_eq!(t.hand_size(P1), hand + 1);
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve_all();
    assert!(t.on_battlefield(spell) || !t.named_on_battlefield("Curator of Destinies").is_empty());
}

#[test]
fn curator_of_destinies_face_down_pile_isnt_revealed() {
    cr!("700.3");
    ruling!(
        "Curator of Destinies",
        "If the opponent chooses the face-down pile to be put into your hand, you don't have to reveal the cards in that pile."
    );
    // P0 puts Grizzly Bears and Island in the face-down pile; P1 picks each pile in turn.
    let mut results = vec![];
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        let lib = stack_library(
            &mut t,
            P0,
            &["Grizzly Bears", "Island", "Hill Giant", "Shock", "Forest"],
        );
        t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
        t.answer(P1, DecisionKind::Option, Answer::Index(pick));
        enter(&mut t, P0, "Curator of Destinies");
        t.resolve_all();
        let revealed = crate::r_s31_common::revealed_cards(&t);
        let in_hand = t.in_hand(P0, "Grizzly Bears");
        let down_revealed = revealed.contains(&lib[0]) || revealed.contains(&lib[1]);
        // The face-up pile is seen by everyone.
        assert!(lib[2..].iter().all(|c| revealed.contains(c)), "{revealed:?}");
        results.push((in_hand, down_revealed));
    }
    // When the face-down pile went to the hand, none of its cards was revealed.
    let (_, revealed) = results.iter().find(|(h, _)| *h).expect("a pick took it");
    assert!(!revealed);
    assert!(results.iter().any(|(h, _)| !h));
}

#[test]
fn an_empty_pile_means_choosing_all_or_nothing() {
    cr!("700.3d");
    ruling!(
        "Fortune's Favor",
        "A pile can have no cards in it. In this case, you’ll choose whether to put all the cards into your hand or into your graveyard."
    );
    ruling!(
        "Steam Augury",
        "If one of the piles is empty, the opponent will choose whether to put the cards from the other pile into your hand or into your graveyard."
    );
    supported("Fortune's Favor");
    supported("Steam Augury");
    // (spell, lands, cards, separator, chooser)
    let cases: [(&str, &[&str], usize, PlayerId, PlayerId); 2] = [
        ("Fortune's Favor", &["Island", "Wastes", "Wastes", "Wastes"], 4, P1, P0),
        ("Steam Augury", &["Island", "Mountain", "Wastes", "Wastes"], 5, P0, P1),
    ];
    for (spell, lands, n, separator, chooser) in cases {
        let mut outcomes = vec![];
        for pick in [0usize, 1] {
            let mut t = TestGame::new(2);
            let names = ["Grizzly Bears", "Island", "Hill Giant", "Shock", "Forest"];
            let lib = stack_library(&mut t, P0, &names[..n]);
            for l in lands {
                t.lands(P0, l, 1);
            }
            let s = t.hand(P0, spell);
            let hand = t.hand_size(P0);
            t.answer_choose(separator, &lib.iter().map(|c| obj(*c)).collect::<Vec<_>>());
            t.answer(chooser, DecisionKind::Option, Answer::Index(pick));
            if spell == "Fortune's Favor" {
                t.cast(P0, s).target(Entity::Player(P1)).go();
            } else {
                t.cast(P0, s).go();
            }
            t.resolve_all();
            outcomes.push((t.hand_size(P0) + 1 - hand, t.graveyard_size(P0) - 1));
        }
        outcomes.sort();
        assert_eq!(outcomes, vec![(0, n), (n, 0)], "{spell}");
    }
}

// ---------------------------------------------------------------------------
// Power and toughness from the graveyard
// ---------------------------------------------------------------------------

#[test]
fn carrion_grub_only_counts_on_the_battlefield_and_sees_printed_power() {
    cr!("611.3a", "400.7", "122.2");
    ruling!(
        "Carrion Grub",
        "Carrion Grub's first ability applies only while it is on the battlefield. In all other zones, its power is 0."
    );
    ruling!(
        "Carrion Grub",
        "Any counters on creatures that die or effects that affect them until end of turn won't apply to those cards in their owner's graveyard."
    );
    supported("Carrion Grub");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    let gy_grub = t.graveyard(P0, "Carrion Grub");
    t.g.recompute();
    assert_eq!(t.obj_now(gy_grub).power(), 0);
    let grub = t.battlefield(P0, "Carrion Grub");
    assert_eq!(t.pt(grub).0, 3);
    // A 5/5 Grizzly Bears (with counters) dies: in the graveyard it's a 2-power card.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), "+1/+1", 3, None);
    t.g.recompute();
    assert_eq!(t.pt(bears).0, 5);
    destroy(&mut t, bears);
    t.g.recompute();
    assert_eq!(t.pt(grub).0, 3);
}

#[test]
fn losing_a_graveyard_bonus_can_make_damage_lethal() {
    cr!("704.5g", "611.3a", "120.6");
    ruling!(
        "Billowing Shriekmass",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Billowing Shriekmass may become lethal if the number of cards in your graveyard falls below seven."
    );
    ruling!(
        "Aven Heartstabber",
        "If you have five or more mana values among cards in your graveyard and then some of them are removed such that you have fewer than five mana values among those cards, Aven Heartstabber loses its bonus to power and toughness. Notably, this may cause it to die if it now has damage marked on it greater than its toughness."
    );
    ruling!(
        "Snooping Newsie",
        "If you have five or more mana values among cards in your graveyard and then some of them are removed such that you have fewer than five mana values among those cards, Snooping Newsie loses its bonus to power and toughness immediately. Notably, this may cause it to die if it now has damage marked on it greater than its toughness."
    );
    let cases: [(&str, &[&str], i32); 3] = [
        (
            "Billowing Shriekmass",
            &["Island", "Island", "Island", "Island", "Island", "Island", "Hill Giant"],
            1,
        ),
        (
            "Aven Heartstabber",
            &["Island", "Llanowar Elves", "Grizzly Bears", "Cancel", "Hill Giant"],
            2,
        ),
        (
            "Snooping Newsie",
            &["Island", "Llanowar Elves", "Grizzly Bears", "Cancel", "Hill Giant"],
            1,
        ),
    ];
    for (name, gy, bonus) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let base = t.pt(c).1;
        let ids: Vec<ObjectId> = gy.iter().map(|n| t.graveyard(P0, n)).collect();
        t.g.recompute();
        assert_eq!(t.pt(c).1, base + bonus, "{name}");
        let src = t.battlefield(P1, "Grizzly Bears");
        damage(&mut t, src, base, c);
        assert!(t.on_battlefield(c), "{name}");
        // Hill Giant (the seventh card / the only mana value 4) leaves the graveyard.
        let giant = *ids.last().unwrap();
        t.g.move_object(giant, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
        t.g.flush_events();
        t.settle();
        assert!(!t.on_battlefield(c), "{name}");
    }
}

// ---------------------------------------------------------------------------
// Lands
// ---------------------------------------------------------------------------

#[test]
fn blossoming_tortoise_reduces_activated_abilities_of_lands_on_the_battlefield() {
    cr!("602.1", "601.2f", "113.3b");
    ruling!(
        "Blossoming Tortoise",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities are activated abilities and will have colons in their reminder text. Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected by the cost reduction ability of Blossoming Tortoise."
    );
    ruling!(
        "Blossoming Tortoise",
        "Blossoming Tortoise's second ability affects only abilities of lands you control on the battlefield. The costs of activated abilities of land cards that work in other zones (such as cycling) won't be reduced."
    );
    supported("Blossoming Tortoise");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blossoming Tortoise");
    // Mishra's Factory's "{1}: ..." costs nothing; it's then a 3/3 land creature.
    let factory = t.battlefield(P0, "Mishra's Factory");
    t.g.objects[factory.0 as usize].tapped = true;
    assert!(can_activate(&mut t, P0, factory));
    crate::r_s06_common::activate_containing(&mut t, P0, factory, "becomes").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(factory), (3, 3));
    // Ash Barrens' basic landcycling {1} in hand isn't reduced: no mana, no cycling.
    let barrens = t.hand(P0, "Ash Barrens");
    assert!(!can_activate(&mut t, P0, barrens));
}

#[test]
fn icetill_explorer_doesnt_change_land_timing_or_allow_graveyard_abilities() {
    cr!("305.2", "305.3", "602.1");
    ruling!(
        "Icetill Explorer",
        "Icetill Explorer’s second ability doesn’t change the times when you can play those land cards. You can still play lands only during your main phase when you have priority and the stack is empty."
    );
    ruling!(
        "Icetill Explorer",
        "Icetill Explorer’s second ability doesn’t allow you to activate abilities (such as cycling) of land cards in your graveyard."
    );
    supported("Icetill Explorer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Icetill Explorer");
    let thicket = t.graveyard(P0, "Tranquil Thicket");
    t.lands(P0, "Forest", 1);
    assert!(can_play_land(&mut t, P0, thicket));
    assert!(!can_activate(&mut t, P0, thicket));
    // Not with something on the stack.
    let bolt = t.hand(P0, "Giant Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, bolt).target(obj(bears)).go();
    assert!(!can_play_land(&mut t, P0, thicket));
    t.resolve_all();
    // Not outside a main phase.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, thicket));
}

#[test]
fn curator_beastie_doesnt_affect_creatures_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Curator Beastie",
        "If Curator Beastie enters under your control at the same time as one or more colorless creatures, those colorless creatures won't enter with additional +1/+1 counters. This is because a replacement effect created by an object entering can only apply to that object."
    );
    supported("Curator Beastie");
    let mut t = TestGame::new(2);
    let ids = enter_together(&mut t, P0, &["Curator Beastie", "Ornithopter"]);
    assert_eq!(t.counters(ids[1], "+1/+1"), 0);
    t.resolve_all();
    let later = enter(&mut t, P0, "Ornithopter");
    assert_eq!(t.counters(later, "+1/+1"), 2);
}

#[test]
fn milling_more_cards_than_the_library_has() {
    cr!("701.17b");
    ruling!(
        "Splinterfright",
        "If Splinterfright’s controller has only one card in their library when its triggered ability resolves, they put that card into their graveyard."
    );
    ruling!(
        "Armored Skaab",
        "If you have fewer than four cards in your library when Armored Skaab enters, you'll put all of them into your graveyard."
    );
    supported("Splinterfright");
    supported("Armored Skaab");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    t.battlefield(P0, "Splinterfright");
    t.advance_to(P1, Step::Upkeep);
    let lib = set_library(&mut t, P0, &["Grizzly Bears"]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Graveyard(P0));
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Grizzly Bears", "Island"]);
    enter(&mut t, P0, "Armored Skaab");
    t.resolve_all();
    for c in lib {
        assert_eq!(t.zone(c), Zone::Graveyard(P0));
    }
}

#[test]
fn rot_farm_skeleton_needs_four_cards_to_mill() {
    cr!("118.3", "602.2b");
    ruling!(
        "Rot Farm Skeleton",
        "If you have three or fewer cards in your library, you can't activate Rot Farm Skeleton's last ability."
    );
    supported("Rot Farm Skeleton");
    let mut t = TestGame::new(2);
    let s = t.graveyard(P0, "Rot Farm Skeleton");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    set_library(&mut t, P0, &["Island", "Island", "Island"]);
    assert!(!can_activate(&mut t, P0, s));
    t.library_top(P0, "Island");
    assert!(can_activate(&mut t, P0, s));
}

// ---------------------------------------------------------------------------
// Valakut Exploration
// ---------------------------------------------------------------------------

#[test]
fn valakut_exploration_leaving_keeps_cards_playable_and_exiled() {
    cr!("610.3", "603.4");
    ruling!(
        "Valakut Exploration",
        "If Valakut Exploration leaves the battlefield, any cards exiled with it remain exiled and can be played. The end step triggered ability won't trigger, and the cards won't be put into their owner's graveyard."
    );
    supported("Valakut Exploration");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Valakut Exploration");
    let island = t.library_top(P0, "Island");
    enter(&mut t, P0, "Forest");
    t.resolve_all();
    assert_eq!(t.zone(island), Zone::Exile);
    destroy(&mut t, v);
    assert!(can_play_land(&mut t, P0, island));
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(island), Zone::Exile);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn valakut_exploration_card_played_in_response_isnt_counted() {
    cr!("603.4", "608.2h");
    ruling!(
        "Valakut Exploration",
        "If you play the card in response to the end step triggered ability, that card won't be put into your graveyard and won't be counted when determining how much damage to deal."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valakut Exploration");
    t.lands(P0, "Mountain", 1);
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Grizzly Bears"]);
    enter(&mut t, P0, "Forest");
    t.resolve_all();
    enter(&mut t, P0, "Forest");
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    assert_eq!(t.zone(lib[1]), Zone::Exile);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let bolt = t.g.current(lib[0]);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 1);
    assert_eq!(t.zone(lib[1]), Zone::Graveyard(P0));
}

// ---------------------------------------------------------------------------
// Sagas
// ---------------------------------------------------------------------------

#[test]
fn tymaret_scry_more_than_the_library_still_gains_full_life() {
    cr!("701.22a", "714.2b");
    ruling!(
        "Tymaret Calls the Dead",
        "If X is greater than the number of cards in your library while resolving the third chapter ability, you look at your entire library and put it back in any order. You still gain life equal to the full value of X."
    );
    supported("Tymaret Calls the Dead");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Tymaret Calls the Dead");
    obj_lore(&mut t, saga, 2);
    for _ in 0..3 {
        t.battlefield(P0, "Armored Skaab");
    }
    let lib = set_library(&mut t, P0, &["Island"]);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.library_size(P0), 1);
    assert!(t.zone(lib[0]) == Zone::Library(P0) || t.zone(lib[0]) == Zone::Graveyard(P0));
}

#[test]
fn three_seasons_doesnt_shuffle_a_library_with_no_graveyard_cards_chosen() {
    cr!("701.24a", "714.2b");
    ruling!(
        "The Three Seasons",
        "For the chapter III ability, if a player has no cards in their graveyard, they don’t shuffle their library."
    );
    supported("The Three Seasons");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Three Seasons");
    obj_lore(&mut t, saga, 2);
    for _ in 0..3 {
        t.graveyard(P0, "Island");
    }
    let from = t.g.turn_events.len();
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert!(shuffled_since(&t, from, P0));
    assert!(!shuffled_since(&t, from, P1));
}

// ---------------------------------------------------------------------------
// Combat
// ---------------------------------------------------------------------------

/// Runs one combat of P0's turn in which `attackers` attack P1 (unblocked), ending in
/// the end of combat step.
fn one_combat(t: &mut TestGame, attackers: &[ObjectId]) {
    let a: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|c| (t.g.current(*c), Entity::Player(P1)))
        .collect();
    attack_with(t, &a);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
}

#[test]
fn song_of_blood_bonus_applies_each_time_it_attacks() {
    cr!("508.1m", "506.1");
    ruling!(
        "Song of Blood",
        "If a creature attacks more than once in a turn, it gets the bonus each time."
    );
    supported("Song of Blood");
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Hill Giant", "Llanowar Elves", "Island", "Shock"]);
    t.lands(P0, "Mountain", 6);
    let song = t.hand(P0, "Song of Blood");
    t.cast(P0, song).go();
    t.resolve_all();
    one_combat(&mut t, &[bears]);
    assert_eq!(t.pt(bears).0, 4);
    assert_eq!(t.life(P1), 16);
    // Relentless Assault untaps it and adds a combat after this main phase.
    t.advance_to(P0, Step::PostcombatMain);
    let ra = t.hand(P0, "Relentless Assault");
    t.cast(P0, ra).go();
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    one_combat(&mut t, &[bears]);
    assert_eq!(t.pt(bears).0, 6);
    assert_eq!(t.life(P1), 10);
}

#[test]
fn ursine_monstrosity_triggers_each_combat_and_isnt_forced_by_a_cost() {
    cr!("508.1d", "508.1h", "506.1");
    ruling!(
        "Ursine Monstrosity",
        "If your turn has multiple combat phases, Ursine Monstrosity's last ability triggers at the beginning of each of them. Ignore any choices made for its last ability during previous combat phases."
    );
    ruling!(
        "Ursine Monstrosity",
        "If Ursine Monstrosity isn't forced to attack the chosen player for one of the above reasons but can still attack, you may choose to have it attack another player, a planeswalker, a battle, or nothing at all."
    );
    supported("Ursine Monstrosity");
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Ursine Monstrosity");
    t.lands(P0, "Mountain", 4);
    let ra = t.hand(P0, "Relentless Assault");
    t.cast(P0, ra).go();
    t.resolve_all();
    let mut triggered = 0;
    for _ in 0..2 {
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        triggered += triggers_on_stack(&t, "attacks that player this combat");
        one_combat(&mut t, &[u]);
    }
    assert_eq!(triggered, 2);
    // With an attack cost (Propaganda), it isn't forced to attack: it may stay home.
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Ursine Monstrosity");
    t.battlefield(P1, "Propaganda");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(can_attack(&mut t, u));
    attack_with(&mut t, &[]);
    assert!(!crate::r_s10_common::attacking(&t, u));
    // Without it, it attacks even if not declared.
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Ursine Monstrosity");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    attack_with(&mut t, &[]);
    assert!(crate::r_s10_common::attacking(&t, u));
}

/// Grisly Anglerfish on P0's battlefield, activated during P1's turn (before combat).
fn anglerfish(t: &mut TestGame) {
    supported("Grizzled Angler // Grisly Anglerfish");
    let fish = crate::r_p079_common::back_face(t, P0, "Grizzled Angler // Grisly Anglerfish");
    t.lands(P0, "Wastes", 6);
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P0, fish, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn grisly_anglerfish_creatures_choose_what_they_attack() {
    cr!("508.1d", "506.4");
    ruling!(
        "Grizzled Angler // Grisly Anglerfish",
        "Each creature’s controller still chooses the player, planeswalker, or battle it attacks. The creatures that have to attack don’t necessarily have to attack you if there are other options."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    anglerfish(&mut t);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    assert!(crate::r_s10_common::attacking(&t, bears));
    let c = t.g.combat.as_ref().unwrap();
    assert!(c
        .attackers
        .iter()
        .any(|a| a.id == bears && a.target == Some(Entity::Player(P2))));
}

#[test]
fn grisly_anglerfish_doesnt_force_creatures_that_cant_or_must_pay() {
    cr!("508.1d", "508.1h", "508.1a");
    ruling!(
        "Grizzled Angler // Grisly Anglerfish",
        "If a creature affected by Grisly Anglerfish’s effect hasn’t been under its controller’s control since the turn began, is tapped, or is affected by a spell or ability that says it can’t attack, then it doesn’t attack. If there’s a cost associated with having that creature attack, its controller isn’t forced to pay that cost, so it doesn’t have to attack in that case either."
    );
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[tapped.0 as usize].tapped = true;
    let sick = t.battlefield_sick(P1, "Grizzly Bears");
    let pacified = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Pacifism", obj(pacified));
    let ready = t.battlefield(P1, "Hill Giant");
    anglerfish(&mut t);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[]);
    for c in [tapped, sick, pacified] {
        assert!(!crate::r_s10_common::attacking(&t, c));
    }
    assert!(crate::r_s10_common::attacking(&t, ready));
    // With an attack cost, nothing is forced.
    let mut t = TestGame::new(2);
    let ready = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Propaganda");
    anglerfish(&mut t);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[]);
    assert!(!crate::r_s10_common::attacking(&t, ready));
}

#[test]
fn harald_chapter_three_triggers_for_elves_gained_later() {
    cr!("603.7a", "603.7c");
    ruling!(
        "Harald Unites the Elves",
        "Conversely, the triggered ability created by the chapter III ability will trigger whenever any Elf you control attacks that turn, even if you didn’t control that Elf as the chapter III ability resolved."
    );
    supported("Harald Unites the Elves");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Harald Unites the Elves");
    obj_lore(&mut t, saga, 2);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(elf, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
}

// ---------------------------------------------------------------------------
// Impromptu Raid, Wondrous Crucible, Does Machines
// ---------------------------------------------------------------------------

#[test]
fn impromptu_raid_in_the_end_step_sacrifices_next_end_step() {
    cr!("603.7b", "513.1");
    ruling!(
        "Impromptu Raid",
        "If the ability is activated after the current turn’s End step has begun, the creature won’t be sacrificed until the next turn’s End step."
    );
    supported("Impromptu Raid");
    let mut t = TestGame::new(2);
    let raid = t.battlefield(P0, "Impromptu Raid");
    t.lands(P0, "Mountain", 3);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.activate(P0, raid, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(bears));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn impromptu_raid_creature_taken_by_another_player_isnt_sacrificed() {
    cr!("701.21a", "611.2c");
    ruling!(
        "Impromptu Raid",
        "If you put a creature onto the battlefield and a different player takes control of it before the end of the turn, you can’t sacrifice it at end of turn. It will remain on the battlefield. It will have haste as long as it’s on the battlefield."
    );
    let mut t = TestGame::new(2);
    let raid = t.battlefield(P0, "Impromptu Raid");
    t.lands(P0, "Mountain", 3);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, raid, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    give_control(&mut t, bears, P1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.on_battlefield(bears));
    assert!(has_kw(&t, bears, KeywordKind::Haste));
}

#[test]
fn wondrous_crucible_copy_not_cast_ceases_to_exist() {
    cr!("707.10a");
    ruling!(
        "Wondrous Crucible",
        "If you don’t cast the copy (perhaps because there are no legal targets available or you don’t want to), the copy will cease to exist."
    );
    supported("Wondrous Crucible");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wondrous Crucible");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    stack_library(&mut t, P0, &["Island", "Forest"]);
    t.advance_to(P0, Step::End);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.stack_len(), 0);
    let bolts = t
        .g
        .objects
        .iter()
        .filter(|o| o.chars.name == "Lightning Bolt" && t.g.is_live(o.id))
        .count();
    assert_eq!(bolts, 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn does_machines_level_three_keeps_the_artifacts_abilities() {
    cr!("613.1d", "613.1f", "716.2a");
    ruling!(
        "Does Machines",
        "Does Machines's level 3 ability doesn't remove any abilities the target artifact has."
    );
    supported("Does Machines");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Does Machines");
    t.lands(P0, "Island", 7);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    gain_level(&mut t, P0, class, 3).unwrap();
    t.resolve_all();
    let stone = t.battlefield(P0, "Mind Stone");
    let abilities = t.obj_now(stone).chars.abilities.len();
    t.answer_targets(P0, &[obj(stone)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(stone).is(mtg_engine::types::CardType::Creature));
    assert_eq!(t.pt(stone), (3, 3));
    assert_eq!(t.obj_now(stone).chars.abilities.len(), abilities);
}
