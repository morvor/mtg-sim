//! Structure-coverage tests (round 6): spell-ability structures shared by many cards that
//! no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::{next_turn, target_choices};
use super::structure_seed_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::*;

/// The names of the cards in `p`'s hand.
pub fn hand_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .hand
        .iter()
        .map(|id| t.obj(*id).chars.name.to_string())
        .collect()
}

#[test]
fn psychic_strike_counters_and_mills_the_spells_controller() {
    cr!("608.2", "701.6a", "701.17a");
    // "Counter target spell. Its controller mills two cards."
    supported("Psychic Strike");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P1, "Forest");
        t.library_top(P0, "Island");
    }
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.battlefield(P0, "Island");
    t.battlefield(P0, "Swamp");
    t.battlefield(P0, "Wastes");
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    let s = t.hand(P0, "Psychic Strike");
    t.cast(P0, s).target(Entity::Object(spell)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "the Bolt was countered");
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.library_size(P1), l1 - 2, "its controller milled two");
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.library_size(P0), l0, "the caster mills nothing");
    assert_eq!(t.graveyard_size(P0), 1);
    // "Its controller", not its owner: P0 gains control of P1's Bolt with Commandeer.
    supported("Commandeer");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P1, "Forest");
        t.library_top(P0, "Island");
    }
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.lands(P0, "Island", 7);
    let c = t.hand(P0, "Commandeer");
    t.answer_yes(P0, false); // keep the Bolt's target
    t.cast(P0, c).target(Entity::Object(spell)).go();
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P0);
    assert_eq!(t.obj_now(spell).owner, P1);
    t.battlefield(P1, "Island");
    t.battlefield(P1, "Swamp");
    t.battlefield(P1, "Wastes");
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    let s = t.hand(P1, "Psychic Strike");
    t.cast(P1, s).target(Entity::Object(spell)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "the Bolt was countered");
    assert_eq!(t.library_size(P0), l0 - 2, "its controller milled two");
    assert_eq!(t.library_size(P1), l1, "its owner mills nothing");
    assert!(t.in_graveyard(P1, "Lightning Bolt"), "to its owner's graveyard");
}

#[test]
fn relearn_returns_an_instant_or_sorcery_card() {
    cr!("608.2", "115.1");
    // "Return target instant or sorcery card from your graveyard to your hand."
    supported("Relearn");
    for card in ["Lightning Bolt", "Divination"] {
        let mut t = TestGame::new(2);
        let target = t.graveyard(P0, card);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let theirs = t.graveyard(P1, "Lightning Bolt");
        t.lands(P0, "Island", 3);
        let s = t.hand(P0, "Relearn");
        t.cast(P0, s).target(Entity::Object(target)).go();
        let c = target_choices(&t, P0).pop().unwrap();
        assert!(c.contains(&Entity::Object(target)));
        assert!(!c.contains(&Entity::Object(bears)), "a creature card");
        assert!(!c.contains(&Entity::Object(theirs)), "not your graveyard");
        t.resolve_all();
        assert!(t.in_hand(P0, card));
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        assert!(t.in_graveyard(P1, "Lightning Bolt"));
    }
}

#[test]
fn sandstorm_damages_each_attacking_creature() {
    cr!("608.2", "120.3e");
    // "Sandstorm deals 1 damage to each attacking creature."
    supported("Sandstorm");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Llanowar Elves");
    let defender = t.battlefield(P1, "Llanowar Elves");
    t.lands(P1, "Forest", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(elves, Entity::Player(P1)), (bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let s = t.hand(P1, "Sandstorm");
    t.cast(P1, s).go();
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.obj_now(bears).damage, 1);
    assert!(t.on_battlefield(idle), "not attacking");
    assert!(t.on_battlefield(defender));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn true_ancestry_returns_a_permanent_card_and_creates_a_clue() {
    cr!("608.2", "115.1", "111.10");
    // "Return up to one target permanent card from your graveyard to your hand. Create a
    // Clue token."
    supported("True Ancestry");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let land = t.graveyard(P0, "Forest");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "True Ancestry");
    t.cast(P0, s).target(Entity::Object(bears)).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(land)), "a land is a permanent card");
    assert!(!c.contains(&Entity::Object(bolt)));
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let clues = subtype_ids(&t, P0, "Clue");
    assert_eq!(clues.len(), 1);
    assert!(t.obj_now(clues[0]).is_token());
    assert!(t.obj_now(clues[0]).chars.is(CardType::Artifact));
    assert!(t.obj_now(clues[0]).chars.colors.is_colorless());
    assert!(
        t.obj_now(clues[0]).chars.abilities.iter().any(|a| a.text.contains("Draw a card")),
        "the predefined Clue ability"
    );
    assert_eq!(count_subtype(&t, P1, "Clue"), 0);
    // With no target, it still creates a Clue.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "True Ancestry");
    t.cast(P0, s).targets(&[]).go();
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Clue"), 1);
}

#[test]
fn acrobatic_leap_pumps_grants_flying_and_untaps() {
    cr!("608.2", "611.2a", "701.26a", "514.2");
    // "Target creature gets +1/+3 and gains flying until end of turn. Untap it."
    supported("Acrobatic Leap");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.g.tap(bears);
    t.g.tap(other);
    t.lands(P0, "Plains", 1);
    let s = t.hand(P0, "Acrobatic Leap");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(other).tapped);
    assert_eq!(t.pt(bears), (3, 5));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(other), (3, 3));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Flying));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn angelic_edict_exiles_a_creature_or_enchantment() {
    cr!("608.2", "115.1", "701.13a");
    // "Exile target creature or enchantment."
    supported("Angelic Edict");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let art = t.battlefield(P1, "Sol Ring");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Plains", 5);
    let s = t.hand(P0, "Angelic Edict");
    t.cast(P0, s).target(anthem).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Object(anthem)));
    assert!(!c.contains(&Entity::Object(art)) && !c.contains(&Entity::Object(land)));
    t.resolve_all();
    assert!(t.in_exile("Glorious Anthem"));
    assert!(t.on_battlefield(bears));
    // A creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let s = t.hand(P0, "Angelic Edict");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn battle_rage_blessing_grants_deathtouch_and_indestructible() {
    cr!("608.2", "611.2a", "702.2b", "702.12b", "514.2");
    // "Target creature gains deathtouch and indestructible until end of turn."
    supported("Battle-Rage Blessing");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let other = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Battle-Rage Blessing");
    t.cast(P0, s).target(elves).go();
    t.resolve_all();
    assert!(t.obj_now(elves).has_keyword(KeywordKind::Deathtouch));
    assert!(t.obj_now(elves).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Deathtouch));
    // Deathtouch: 1 damage destroys the 3/3. Indestructible: 3 damage doesn't kill it.
    t.g.deal_damage(elves, Entity::Object(giant), 1, false);
    t.g.deal_damage(giant, Entity::Object(elves), 3, false);
    t.settle();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(elves));
    assert!(!t.g.destroy(elves, None));
    next_turn(&mut t);
    assert!(!t.obj_now(elves).has_keyword(KeywordKind::Deathtouch));
    assert!(!t.obj_now(elves).has_keyword(KeywordKind::Indestructible));
    assert!(t.g.destroy(elves, None));
}

#[test]
fn seething_song_adds_five_red_mana() {
    cr!("608.2", "106.4");
    // "Add {R}{R}{R}{R}{R}."
    supported("Seething Song");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Seething Song");
    t.cast(P0, s).go();
    t.resolve_all();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.count(ManaType::R), 5);
    assert_eq!(pool.mana.len(), 5);
    assert_eq!(t.g.player(P1).mana_pool.mana.len(), 0);
}

#[test]
fn blazing_volley_damages_each_creature_your_opponents_control() {
    cr!("608.2", "120.3e");
    // "Blazing Volley deals 1 damage to each creature your opponents control."
    supported("Blazing Volley");
    let mut t = TestGame::new(3);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let p2 = t.battlefield(PlayerId(2), "Llanowar Elves");
    let mine = t.battlefield(P0, "Llanowar Elves");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Blazing Volley");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(!t.on_battlefield(elves) && !t.on_battlefield(p2));
    assert_eq!(t.obj_now(bears).damage, 1);
    assert!(t.on_battlefield(mine));
    assert_eq!(t.counters(garruk, "loyalty"), 3, "not a creature");
    assert_eq!(t.life(P1), 20);
}

#[test]
fn withstand_death_grants_indestructible_until_end_of_turn() {
    cr!("608.2", "611.2a", "702.12b", "514.2");
    // "Target creature gains indestructible until end of turn."
    supported("Withstand Death");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let s = t.hand(P0, "Withstand Death");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(!t.g.destroy(bears, None));
    assert!(t.on_battlefield(bears));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Indestructible));
    next_turn(&mut t);
    assert!(t.g.destroy(bears, None));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn sizzle_damages_each_opponent() {
    cr!("608.2", "120.3a");
    // "Sizzle deals 3 damage to each opponent."
    supported("Sizzle");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Sizzle");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(PlayerId(2)), 17);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn burst_of_strength_puts_a_counter_and_untaps() {
    cr!("608.2", "122.1", "701.26a");
    // "Put a +1/+1 counter on target creature and untap it."
    supported("Burst of Strength");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.g.tap(other);
    t.lands(P0, "Forest", 1);
    let s = t.hand(P0, "Burst of Strength");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(other).tapped);
    assert_eq!(t.counters(other, "+1/+1"), 0);
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (3, 3), "a counter is permanent");
}

#[test]
fn queens_commission_creates_two_lifelink_vampires() {
    cr!("608.2", "111.1", "702.15b");
    // "Create two 1/1 white Vampire creature tokens with lifelink."
    supported("Queen's Commission");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Queen's Commission");
    t.cast(P0, s).go();
    t.resolve_all();
    let v = subtype_ids(&t, P0, "Vampire");
    assert_eq!(v.len(), 2);
    assert_eq!(count_subtype(&t, P1, "Vampire"), 0);
    for id in &v {
        let o = t.obj_now(*id);
        assert!(o.is_token() && o.chars.is_creature());
        assert_eq!(o.chars.colors, ColorSet::single(Color::White));
        assert!(o.has_keyword(KeywordKind::Lifelink));
        assert_eq!(t.pt(*id), (1, 1));
    }
    // Lifelink works.
    t.g.deal_damage(v[0], Entity::Player(P1), 1, false);
    t.settle();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn desperate_ravings_draws_two_then_discards_at_random() {
    cr!("608.2", "121.1", "701.9a", "701.9b");
    // "Draw two cards, then discard a card at random."
    supported("Desperate Ravings");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Island");
    t.library_top(P0, "Swamp");
    t.lands(P0, "Mountain", 2);
    let lib = t.library_size(P0);
    let s = t.hand(P0, "Desperate Ravings");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib - 2);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 2, "the spell and the discarded card");
    let kept = hand_names(&t, P0).pop().unwrap();
    assert!(kept == "Island" || kept == "Swamp");
    let other = if kept == "Island" { "Swamp" } else { "Island" };
    assert!(t.in_graveyard(P0, other));
    assert!(
        !t.asked().iter().any(|(p, d)| *p == P0
            && matches!(d, Decision::ChooseEntities { .. })),
        "a random discard: nobody chooses"
    );
}

#[test]
fn risky_research_surveils_draws_and_loses_life() {
    cr!("608.2", "701.25a", "121.1", "119.3");
    // "Surveil 2, then draw two cards. You lose 2 life."
    supported("Risky Research");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Plains");
    t.library_top(P0, "Forest");
    let b = t.library_top(P0, "Island");
    let a = t.library_top(P0, "Swamp");
    t.lands(P0, "Swamp", 3);
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![b], vec![a]));
    let s = t.hand(P0, "Risky Research");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(surveils(&t), vec![2]);
    assert!(t.in_graveyard(P0, "Swamp"), "surveilled away");
    let mut h = hand_names(&t, P0);
    h.sort();
    assert_eq!(h, vec!["Forest", "Island"], "then drew the next two");
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn dauntless_onslaught_pumps_up_to_two_creatures() {
    cr!("608.2", "115.1", "611.2a", "514.2");
    // "Up to two target creatures each get +2/+2 until end of turn."
    supported("Dauntless Onslaught");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Dauntless Onslaught");
    t.cast(P0, s)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(b), (3, 3));
    assert_eq!(t.pt(c), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(t.pt(b), (1, 1));
    // One target.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Dauntless Onslaught");
    t.cast(P0, s).targets(&[Entity::Object(a)]).go();
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(c), (2, 2));
}

#[test]
fn distress_makes_a_player_discard_a_nonland_card_you_choose() {
    cr!("608.2", "115.1", "701.9a", "701.9b");
    // "Target player reveals their hand. You choose a nonland card from it. That player
    // discards that card."
    supported("Distress");
    let mut t = TestGame::new(2);
    let forest = t.hand(P1, "Forest");
    let bolt = t.hand(P1, "Lightning Bolt");
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let s = t.hand(P0, "Distress");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    let c = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .last()
        .expect("P0 chooses the card");
    assert!(c.contains(&Entity::Object(bolt)) && c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(forest)), "a land");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Lightning Bolt") && t.in_hand(P1, "Forest"));
    assert_eq!(t.hand_size(P1), 2);
    // Only lands in hand: nothing is discarded.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    t.hand(P1, "Island");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Distress");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.graveyard_size(P1), 0);
}
