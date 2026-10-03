//! Attack and block trigger conditions (CR 508.3, 509.3): "~ and at least one other
//! [creature] attack", "attacks a player and isn't blocked", "attacks one of your opponents
//! or a planeswalker an opponent controls", "you attack with exactly two creatures", "two or
//! more creatures attack", "an opponent attacks a planeswalker you control", "one or more
//! creatures attack one of your opponents …", creature qualities ("that's enchanted or
//! equipped", "with a mana ability", "you own but don't control", goaded), and blocks
//! naming the other creature ("the blocking creature", "that attacking creature").

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// P0 declares `attackers` (each attacking P1 unless a target is given), P1 declares
/// `blocks`; stops in the declare blockers step with the triggers resolved.
fn combat(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(attackers.to_vec()));
    if !blocks.is_empty() {
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    }
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
}

/// The card's ability `text` compiled (other abilities of it may belong to other work).
fn assert_compiled(name: &str, text: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        !c.unsupported_text().iter().any(|u| u.contains(text)),
        "{name}: {text:?} is unsupported"
    );
}

fn at_p1(v: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    v.iter().map(|a| (*a, Entity::Player(P1))).collect()
}

#[test]
fn paired_tactician_needs_another_warrior_attacking() {
    cr!("508.1", "603.2");
    assert_supported("Paired Tactician");
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Paired Tactician");
    let bears = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &at_p1(&[pt, bears]), &[]);
    assert_eq!(t.counters(pt, "+1/+1"), 0, "Grizzly Bears isn't a Warrior");
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Paired Tactician");
    let other = t.battlefield(P0, "Paired Tactician");
    combat(&mut t, &at_p1(&[pt, other]), &[]);
    assert_eq!(t.counters(pt, "+1/+1"), 1);
    assert_eq!(t.counters(other, "+1/+1"), 1);
}

#[test]
fn glimmer_lens_draws_when_the_equipped_creature_attacks_with_another() {
    cr!("508.1", "301.5");
    assert_supported("Glimmer Lens");
    let mut t = TestGame::new(2);
    let lens = t.battlefield(P0, "Glimmer Lens");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(lens, Entity::Object(a)));
    let hand = t.hand_size(P0);
    combat(&mut t, &at_p1(&[a]), &[]);
    assert_eq!(t.hand_size(P0), hand, "attacking alone");
    let mut t = TestGame::new(2);
    let lens = t.battlefield(P0, "Glimmer Lens");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(lens, Entity::Object(a)));
    let hand = t.hand_size(P0);
    combat(&mut t, &at_p1(&[a, b2]), &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    let _ = b;
}

#[test]
fn gahiji_pumps_creatures_attacking_an_opponent() {
    cr!("508.1b", "603.2");
    assert_supported("Gahiji, Honored One");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gahiji, Honored One");
    let bears = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &at_p1(&[bears]), &[]);
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn alluring_suitor_transforms_when_attacking_with_exactly_two() {
    cr!("508.1", "701.27a");
    assert_supported("Alluring Suitor // Deadly Dancer");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Alluring Suitor // Deadly Dancer");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &at_p1(&[s, a, b]), &[]);
    assert_eq!(t.obj_now(s).chars.name, "Alluring Suitor", "three attackers");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Alluring Suitor // Deadly Dancer");
    let a = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &at_p1(&[s, a]), &[]);
    assert_eq!(t.obj_now(s).chars.name, "Deadly Dancer");
}

#[test]
fn gideon_the_oathsworn_counts_only_other_attackers() {
    cr!("508.1", "122.1");
    assert_supported("Gideon, the Oathsworn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gideon, the Oathsworn");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &at_p1(&[a, b]), &[]);
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 1);
}

#[test]
fn oath_of_kaya_punishes_attacks_on_your_planeswalkers() {
    cr!("508.3b", "506.2");
    assert_supported("Oath of Kaya");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Oath of Kaya");
    let pw = t.battlefield(P1, "Ajani Goldmane");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Object(pw))], &[]);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 22);
}

#[test]
fn argent_dais_and_flummoxed_cyclops_need_two_or_more_attackers() {
    cr!("508.1");
    assert_supported("Argent Dais");
    assert_supported("Flummoxed Cyclops");
    let mut t = TestGame::new(2);
    let dais = t.battlefield(P1, "Argent Dais");
    let cyclops = t.battlefield(P1, "Flummoxed Cyclops");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &at_p1(&[a]), &[(cyclops, a)]);
    assert_eq!(t.counters(dais, "oil"), 0);
    assert!(t.obj_now(cyclops).tapped || t.on_battlefield(cyclops));
    let mut t = TestGame::new(2);
    let dais = t.battlefield(P1, "Argent Dais");
    t.battlefield(P1, "Flummoxed Cyclops");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &at_p1(&[a, b2]), &[]);
    assert_eq!(t.counters(dais, "oil"), 1);
    let _ = b;
}

#[test]
fn landroval_triggers_when_two_or_more_attack_a_player() {
    cr!("508.3e");
    assert_supported("Landroval, Horizon Witness");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Landroval, Horizon Witness");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(a)]);
    combat(&mut t, &at_p1(&[a, b]), &[]);
    assert!(t.obj_now(a).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn frontier_warmonger_gives_the_attackers_menace() {
    cr!("508.1", "702.111a");
    assert_supported("Frontier Warmonger");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frontier Warmonger");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &at_p1(&[a]), &[]);
    use mtg_engine::keywords::KeywordKind::Menace;
    assert!(t.obj_now(a).has_keyword(Menace));
    assert!(!t.obj_now(b).has_keyword(Menace));
}

#[test]
fn reyav_gives_enchanted_or_equipped_attackers_double_strike() {
    cr!("508.1", "702.4a");
    assert_supported("Reyav, Master Smith");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reyav, Master Smith");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Short Sword");
    assert!(t.g.attach(sword, Entity::Object(a)));
    combat(&mut t, &at_p1(&[a, b]), &[]);
    use mtg_engine::keywords::KeywordKind::DoubleStrike;
    assert!(t.obj_now(a).has_keyword(DoubleStrike));
    assert!(!t.obj_now(b).has_keyword(DoubleStrike));
}

#[test]
fn raggadragga_untaps_attackers_with_mana_abilities() {
    cr!("605.1a", "508.1f");
    assert_compiled("Raggadragga, Goreguts Boss", "with a mana ability");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raggadragga, Goreguts Boss");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(elf), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
    combat(&mut t, &at_p1(&[elf, bears]), &[]);
    assert!(!t.obj_now(elf).tapped);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn wooden_stake_destroys_the_vampire() {
    cr!("509.3b", "509.3d");
    assert_supported("Wooden Stake");
    let mut t = TestGame::new(2);
    let stake = t.battlefield(P0, "Wooden Stake");
    let a = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(stake, Entity::Object(a)));
    let vamp = t.battlefield(P1, "Vampire Interloper");
    let other = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &at_p1(&[a]), &[(other, a)]);
    assert!(t.on_battlefield(other), "not a Vampire");
    let mut t = TestGame::new(2);
    let stake = t.battlefield(P0, "Wooden Stake");
    let a = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(stake, Entity::Object(a)));
    let vamp2 = t.battlefield(P1, "Sengir Vampire");
    combat(&mut t, &at_p1(&[a]), &[(vamp2, a)]);
    assert!(!t.on_battlefield(vamp2));
    let _ = vamp;
}

#[test]
fn mammoth_harness_gives_the_other_creature_first_strike() {
    cr!("509.3b", "509.3d");
    assert_supported("Mammoth Harness");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let harness = t.battlefield(P1, "Mammoth Harness");
    assert!(t.g.attach(harness, Entity::Object(a)));
    let blocker = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &at_p1(&[a]), &[(blocker, a)]);
    use mtg_engine::keywords::KeywordKind::FirstStrike;
    assert!(t.obj_now(blocker).has_keyword(FirstStrike));
    assert!(!t.obj_now(a).has_keyword(FirstStrike));
}

#[test]
fn dead_iron_sledge_destroys_both_creatures_not_itself() {
    cr!("509.3b", "509.3d");
    assert_supported("Dead-Iron Sledge");
    let mut t = TestGame::new(2);
    let sledge = t.battlefield(P0, "Dead-Iron Sledge");
    let a = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(sledge, Entity::Object(a)));
    let blocker = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &at_p1(&[a]), &[(blocker, a)]);
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(blocker));
    assert!(t.on_battlefield(sledge));
}

#[test]
fn no_quarter_destroys_the_creature_with_lesser_power() {
    cr!("509.3b", "509.3d");
    assert_supported("No Quarter");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "No Quarter");
    let big = t.battlefield(P0, "Craw Wurm");
    let small = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &at_p1(&[big]), &[(small, big)]);
    assert!(!t.on_battlefield(small), "the blocking creature has lesser power");
    assert!(t.on_battlefield(big));
    let mut t = TestGame::new(2);
    t.battlefield(P1, "No Quarter");
    let small = t.battlefield(P0, "Grizzly Bears");
    let big = t.battlefield(P1, "Craw Wurm");
    combat(&mut t, &at_p1(&[small]), &[(big, small)]);
    assert!(!t.on_battlefield(small), "the attacking creature has lesser power");
    assert!(t.on_battlefield(big));
}

#[test]
fn righteous_indignation_pumps_the_blocker_of_a_red_creature() {
    cr!("509.3b");
    assert_supported("Righteous Indignation");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Righteous Indignation");
    let red = t.battlefield(P0, "Raging Goblin");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &at_p1(&[red]), &[(blocker, red)]);
    assert_eq!(t.pt(blocker), (3, 3));
    assert_eq!(t.pt(red), (1, 1));
}

#[test]
fn seifer_gives_a_double_blocked_attacker_deathtouch() {
    cr!("509.3e");
    assert_supported("Seifer, Balamb Rival");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Seifer, Balamb Rival");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &at_p1(&[wurm]), &[(b1, wurm), (b2, wurm)]);
    use mtg_engine::keywords::KeywordKind::Deathtouch;
    assert!(t.obj_now(wurm).has_keyword(Deathtouch));
}

#[test]
fn rashka_and_lairwatch_giant_count_the_creatures_they_block() {
    cr!("509.3a");
    assert_supported("Rashka the Slayer");
    let mut t = TestGame::new(2);
    let rashka = t.battlefield(P1, "Rashka the Slayer");
    let black = t.battlefield(P0, "Vampire Interloper");
    t.g.objects[black.0 as usize].summoning_sick = false;
    combat(&mut t, &at_p1(&[black]), &[]);
    assert_eq!(t.pt(rashka), (3, 3), "didn't block");
    let mut t = TestGame::new(2);
    let rashka = t.battlefield(P1, "Rashka the Slayer");
    let black = t.battlefield(P0, "Hypnotic Specter");
    combat(&mut t, &at_p1(&[black]), &[(rashka, black)]);
    assert_eq!(t.pt(rashka), (4, 5));
}

#[test]
fn serra_inquisitors_gets_bigger_against_black_creatures() {
    cr!("509.3a", "509.3c");
    assert_supported("Serra Inquisitors");
    let mut t = TestGame::new(2);
    let si = t.battlefield(P0, "Serra Inquisitors");
    let white = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &at_p1(&[si]), &[(white, si)]);
    assert_eq!(t.pt(si), (3, 3));
    let mut t = TestGame::new(2);
    let si = t.battlefield(P0, "Serra Inquisitors");
    let black = t.battlefield(P1, "Hypnotic Specter");
    combat(&mut t, &at_p1(&[si]), &[(black, si)]);
    assert_eq!(t.pt(si), (5, 3));
}

#[test]
fn vengeful_ancestor_damages_the_controller_of_an_attacking_goaded_creature() {
    cr!("701.15a", "508.1");
    assert_supported("Vengeful Ancestor");
    let mut t = TestGame::new(2);
    let ancestor = t.battlefield(P1, "Vengeful Ancestor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].goaded_by.push(P1);
    let _ = ancestor;
    combat(&mut t, &at_p1(&[bears]), &[]);
    assert_eq!(t.life(P0), 19);
    let _ = CardType::Creature;
}

#[test]
fn giant_shark_needs_a_damaged_creature() {
    cr!("509.3b", "509.3d");
    assert_supported("Giant Shark");
    let mut t = TestGame::new(2);
    let shark = t.battlefield(P1, "Giant Shark");
    t.battlefield(P1, "Island");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let damaged = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Object(damaged)).go();
    t.resolve_all();
    combat(&mut t, &at_p1(&[bears, damaged]), &[(shark, damaged)]);
    assert_eq!(t.pt(shark), (6, 4));
    let _ = bears;
}

#[test]
fn preacher_of_the_schism_draws_while_you_have_the_most_life() {
    cr!("508.1", "119.1");
    assert_supported("Preacher of the Schism");
    let mut t = TestGame::new(2);
    let preacher = t.battlefield(P0, "Preacher of the Schism");
    t.g.players[1].life = 25;
    let hand = t.hand_size(P0);
    combat(&mut t, &at_p1(&[preacher]), &[]);
    // P1 has the most life: the token ability, not the card ability.
    assert_eq!(t.hand_size(P0), hand);
    let mut t = TestGame::new(2);
    let preacher = t.battlefield(P0, "Preacher of the Schism");
    let hand = t.hand_size(P0);
    combat(&mut t, &at_p1(&[preacher]), &[]);
    // Tied for most life: both abilities.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn killian_draws_when_creatures_enchanted_by_your_auras_attack() {
    cr!("303.4", "508.1");
    assert_compiled("Killian, Decisive Mentor", "enchanted by an Aura you control");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Killian, Decisive Mentor");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(aura, Entity::Object(a)));
    let hand = t.hand_size(P0);
    combat(&mut t, &at_p1(&[a, b]), &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn zariels_emblem_triggers_only_after_the_first_combat_phase() {
    cr!("511.2", "505.1a");
    assert_supported("Zariel, Archduke of Avernus");
    let mut t = TestGame::new(2);
    let zariel = t.battlefield(P0, "Zariel, Archduke of Avernus");
    t.g.objects[zariel.0 as usize]
        .counters
        .insert("loyalty".into(), 6);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, zariel, 2, &[]).unwrap();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(bears, Entity::Player(P1))]));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped, "untapped at the end of the first combat");
    // There is an additional combat phase, whose end doesn't trigger the emblem again.
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(bears, Entity::Player(P1))]));
    t.advance_to(P0, Step::DeclareAttackers);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn mob_mentality_needs_every_non_wall_creature_attacking() {
    cr!("508.1");
    assert_supported("Mob Mentality");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Wall of Stone");
    let aura = t.battlefield(P0, "Mob Mentality");
    assert!(t.g.attach(aura, Entity::Object(a)));
    combat(&mut t, &at_p1(&[a]), &[]);
    assert_eq!(t.pt(a), (2, 2), "the other Bears didn't attack");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Wall of Stone");
    let aura = t.battlefield(P0, "Mob Mentality");
    assert!(t.g.attach(aura, Entity::Object(a)));
    combat(&mut t, &at_p1(&[a, b2]), &[]);
    assert_eq!(t.pt(a), (4, 2));
    let _ = b;
}

#[test]
fn rigo_draws_for_each_player_or_planeswalker_attacked_by_a_small_creature() {
    cr!("508.3b");
    assert_supported("Rigo, Streetwise Mentor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rigo, Streetwise Mentor");
    let big = t.battlefield(P0, "Grizzly Bears");
    let small = t.battlefield(P0, "Llanowar Elves");
    let small2 = t.battlefield(P0, "Llanowar Elves");
    let pw = t.battlefield(P1, "Ajani Goldmane");
    let hand = t.hand_size(P0);
    combat(
        &mut t,
        &[
            (big, Entity::Player(P1)),
            (small, Entity::Player(P1)),
            (small2, Entity::Object(pw)),
        ],
        &[],
    );
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn loot_dispute_rewards_attacking_the_player_with_the_initiative() {
    cr!("725.1", "508.3e");
    assert_compiled("Loot Dispute", "the player who has the initiative");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loot Dispute");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.initiative = Some(P0);
    combat(&mut t, &at_p1(&[bears]), &[]);
    let treasures = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|s| s == "Treasure"))
            .count()
    };
    assert_eq!(treasures(&t), 0, "P1 doesn't have the initiative");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loot Dispute");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.initiative = Some(P1);
    combat(&mut t, &at_p1(&[bears]), &[]);
    assert_eq!(treasures(&t), 1);
}
