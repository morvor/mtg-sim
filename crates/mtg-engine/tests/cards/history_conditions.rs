//! Game-history conditions (`oracle/patterns/history_conditions.rs`, `history_query.rs`,
//! `kw/cast_history.rs`): "if you drew two or more cards this turn", "if an opponent lost
//! 2 or more life this turn", "if no permanents left the battlefield this turn", "if ~
//! attacked or blocked this turn", "if an opponent lost life last turn", "if you haven't
//! been dealt combat damage since your last turn", "if it was cast", "if he was kicked",
//! "if no mana was spent to cast it", "if mana from a Treasure was spent to cast it", "if
//! it's the first combat phase of the turn", "if it's the second creature spell you cast
//! this turn", "if this ability has been activated four or more times this turn".

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn treasure(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let spec = mtg_engine::tokens::predefined("Treasure").expect("Treasure token");
    let tc = mtg_engine::replacement::TokenCreate {
        chars: mtg_engine::tokens::token_characteristics(&spec),
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    t.g.create_tokens(p, tc, 1, None)[0]
}

fn count_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents_controlled_by(p)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.name == name)
        .count()
}

/// Goes to `p`'s end step from their precombat main phase and resolves what triggers.
fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.set_step(p, Step::PostcombatMain);
    t.advance_to(p, Step::End);
    t.resolve_all();
}

// --- [player] [past verb] [objects] this turn -----------------------------------------

#[test]
fn archmage_ascension_counts_the_cards_you_drew_this_turn() {
    cr!("603.4");
    assert_supported("Archmage Ascension");
    let mut t = TestGame::new(2);
    let aa = t.battlefield(P0, "Archmage Ascension");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.answer_yes(P0, true);
    to_end_step(&mut t, P0);
    assert_eq!(t.counters(aa, "quest"), 0, "one card isn't two");
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(aa, "quest"), 1);
}

#[test]
fn bloodchief_ascension_needs_one_opponent_to_lose_two_life() {
    cr!("603.4", "119.3");
    assert_supported("Bloodchief Ascension");
    let mut t = TestGame::new(3);
    let ba = t.battlefield(P0, "Bloodchief Ascension");
    // Two opponents losing 1 life each isn't an opponent losing 2 or more.
    t.g.lose_life(P1, 1);
    t.g.lose_life(P2, 1);
    t.g.flush_events();
    t.answer_yes(P0, true);
    to_end_step(&mut t, P0);
    assert_eq!(t.counters(ba, "quest"), 0);
    t.g.lose_life(P2, 1);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(ba, "quest"), 1);
}

#[test]
fn tinybones_draws_if_an_opponent_discarded_this_turn() {
    cr!("603.4", "701.9a");
    assert_supported("Tinybones, Trinket Thief");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tinybones, Trinket Thief");
    // The controller discarding doesn't count.
    let mine = t.hand(P0, "Island");
    t.g.discard(P0, mine, None);
    t.g.flush_events();
    let hand = t.hand_size(P0);
    to_end_step(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 20);
    let theirs = t.hand(P1, "Island");
    t.g.discard(P1, theirs, None);
    t.g.flush_events();
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn luminarch_ascension_checks_that_you_didnt_lose_life() {
    cr!("603.4", "119.3");
    assert_supported("Luminarch Ascension");
    let mut t = TestGame::new(2);
    let la = t.battlefield(P0, "Luminarch Ascension");
    t.answer_yes(P0, true);
    to_end_step(&mut t, P1);
    assert_eq!(t.counters(la, "quest"), 1);
    t.g.lose_life(P0, 1);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.set_step(P1, Step::PostcombatMain);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(la, "quest"), 1);
}

#[test]
fn lunar_convocation_needs_both_life_gained_and_life_lost() {
    cr!("603.4");
    assert_supported("Lunar Convocation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lunar Convocation");
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    to_end_step(&mut t, P0);
    assert_eq!(count_named(&t, P0, "Bat Token"), 0);
    assert_eq!(t.life(P1), 19, "gained life: each opponent loses 1 life");
    t.g.gain_life(P0, 2);
    t.g.lose_life(P0, 1);
    t.g.flush_events();
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Bat Token"), 1);
}

#[test]
fn sarevok_checks_that_no_permanents_left_the_battlefield() {
    cr!("603.4", "700.4");
    assert_supported("Sarevok, Deathbringer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sarevok, Deathbringer");
    to_end_step(&mut t, P0);
    assert_eq!(t.life(P0), 17, "the active player loses life equal to its power");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.set_step(P1, Step::PostcombatMain);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn eshki_needs_a_creature_spell_and_a_noncreature_spell() {
    cr!("603.4", "601.2i");
    assert_supported("Eshki Dragonclaw");
    let mut t = TestGame::new(2);
    let eshki = t.battlefield(P0, "Eshki Dragonclaw");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(eshki, "+1/+1"), 0, "only a creature spell");
    // Next turn: a creature spell and a noncreature spell.
    let mut t = TestGame::new(2);
    let eshki = t.battlefield(P0, "Eshki Dragonclaw");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(eshki, "+1/+1"), 2);
}

#[test]
fn deepway_navigator_counts_the_merfolk_you_attacked_with() {
    cr!("508.1", "611.3a");
    assert_supported("Deepway Navigator");
    let mut t = TestGame::new(2);
    let nav = t.battlefield(P0, "Deepway Navigator");
    let a = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let b = t.battlefield(P0, "Merfolk of the Pearl Trident");
    assert_eq!(t.pt(nav), (2, 2));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[
            (nav, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
        &[],
    );
    assert_eq!(t.pt(nav), (3, 2));
    assert_eq!(t.pt(a), (2, 1));
}

#[test]
fn deepway_navigator_two_merfolk_attacking_isnt_enough() {
    cr!("508.1");
    let mut t = TestGame::new(2);
    let nav = t.battlefield(P0, "Deepway Navigator");
    let a = t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(nav, Entity::Player(P1)), (a, Entity::Player(P1))], &[]);
    assert_eq!(t.pt(nav), (2, 2));
}

#[test]
fn mercadian_atlas_draws_only_if_you_didnt_play_a_land() {
    cr!("603.4", "305.1");
    assert_supported("Mercadian Atlas");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mercadian Atlas");
    let land = t.hand(P0, "Island");
    t.play_land(P0, land).expect("plays a land");
    t.g.flush_events();
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    to_end_step(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mercadian Atlas");
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    to_end_step(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn ertais_scorn_costs_less_after_an_opponent_cast_two_spells() {
    cr!("601.2f", "117.7");
    assert_supported("Ertai's Scorn");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 2);
    let b1 = t.hand(P1, "Lightning Bolt");
    let b2 = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, b1).target(P0).go();
    t.resolve();
    t.lands(P0, "Island", 2);
    let scorn = t.hand(P0, "Ertai's Scorn");
    t.cast(P1, b2).target(P0).go();
    // Two spells cast by the opponent: {1}{U} is enough.
    t.cast(P0, scorn).target(b2).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 17);
}

// --- [objects] [past verb] this turn --------------------------------------------------

#[test]
fn inferno_hellion_is_shuffled_away_after_it_attacked() {
    cr!("603.4", "508.1");
    assert_supported("Inferno Hellion");
    let mut t = TestGame::new(2);
    let hellion = t.battlefield(P0, "Inferno Hellion");
    to_end_step(&mut t, P0);
    assert!(t.on_battlefield(hellion), "it didn't attack");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(hellion, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(hellion));
    assert_eq!(t.named_on_battlefield("Inferno Hellion").len(), 0);
}

#[test]
fn vizier_of_deferment_asks_about_the_target_not_itself() {
    cr!("603.4", "608.2b");
    assert_supported("Vizier of Deferment");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let vizier = t.hand(P0, "Vizier of Deferment");
    t.cast(P0, vizier).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "the target didn't attack or block");
}

#[test]
fn vizier_of_deferment_exiles_a_creature_that_attacked() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    t.lands(P0, "Plains", 3);
    let vizier = t.hand(P0, "Vizier of Deferment");
    t.g.turn.priority = Some(P0);
    t.cast(P0, vizier).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn erg_raiders_and_mad_dog_creatures_that_came_under_your_control() {
    cr!("603.4");
    ruling!(
        "Erg Raiders",
        "comes under your control\" if it enters the battlefield under your control"
    );
    assert_supported("Erg Raiders");
    assert_supported("Mad Dog");
    // Entered this turn: no damage.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let raiders = t.hand(P0, "Erg Raiders");
    t.cast(P0, raiders).go();
    t.resolve();
    to_end_step(&mut t, P0);
    assert_eq!(t.life(P0), 20);
    // On the battlefield since an earlier turn, didn't attack: 2 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erg Raiders");
    let dog = t.battlefield(P0, "Mad Dog");
    to_end_step(&mut t, P0);
    assert_eq!(t.life(P0), 18);
    assert!(!t.on_battlefield(dog), "Mad Dog is sacrificed");
}

#[test]
fn rushing_tide_zubera_counts_the_damage_dealt_to_it() {
    cr!("603.4", "603.10a", "120.3");
    ruling!("Rushing-Tide Zubera", "counts all damage dealt to it this turn");
    assert_supported("Rushing-Tide Zubera");
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Rushing-Tide Zubera");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Object(z), 2, false);
    t.g.deal_damage(src, Entity::Object(z), 2, false);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(!t.on_battlefield(z));
    assert_eq!(t.hand_size(P0), hand + 3);
    // Destroyed with less damage: no cards.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Rushing-Tide Zubera");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Object(z), 3, false);
    t.g.destroy(z, None);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

// --- Other windows: this combat, last turn, since your last turn ----------------------

#[test]
fn gnoll_hunter_pack_tactics_totals_the_power_of_attackers_this_combat() {
    cr!("508.1", "603.4");
    assert_supported("Gnoll Hunter");
    let mut t = TestGame::new(2);
    let gnoll = t.battlefield(P0, "Gnoll Hunter");
    let ogre = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    // 2 + 3 power: not enough.
    t.attack(&[(gnoll, Entity::Player(P1)), (ogre, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(gnoll, "+1/+1"), 0);
    let mut t = TestGame::new(2);
    let gnoll = t.battlefield(P0, "Gnoll Hunter");
    let ogre = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[
            (gnoll, Entity::Player(P1)),
            (ogre, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
        &[],
    );
    assert_eq!(t.counters(gnoll, "+1/+1"), 1);
}

#[test]
fn feast_on_the_fallen_looks_at_last_turn() {
    cr!("603.4", "500.1");
    assert_supported("Feast on the Fallen");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feast on the Fallen");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.lose_life(P1, 1);
    t.g.flush_events();
    // P1's upkeep: P1 lost life during the last turn (P0's).
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // P0's upkeep: nobody lost life during P1's turn.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn marchesa_draws_if_you_werent_dealt_combat_damage_since_your_last_turn() {
    cr!("603.4", "510.2");
    let name = "Invasion of Fiora // Marchesa, Resolute Monarch";
    let c = card(name);
    assert!(!c.faces[1].unsupported.iter().any(|u| u.contains("since your last turn")));
    // The condition is about the opponents' turns since yours.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.custom(P0, CardDef::custom(c.faces[1].chars.clone()), Zone::Battlefield);
    for p in [P0, P1] {
        for _ in 0..4 {
            t.library_top(p, "Island");
        }
    }
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Draw);
    // Dealt combat damage during P1's turn: no card from Marchesa (only the draw step's).
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 18);
    // Next upkeep: nothing happened since.
    t.advance_to(P1, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 17);
}

// --- How a spell was cast ---------------------------------------------------------------

#[test]
fn josu_vess_makes_zombies_only_if_he_was_kicked() {
    cr!("702.33d", "603.4");
    assert_supported("Josu Vess, Lich Knight");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let josu = t.hand(P0, "Josu Vess, Lich Knight");
    t.cast(P0, josu).kicked(false).go();
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Zombie Knight Token"), 0);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 10);
    let josu = t.hand(P0, "Josu Vess, Lich Knight");
    t.cast(P0, josu).kicked(true).go();
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Zombie Knight Token"), 8);
}

#[test]
fn anti_venom_returns_a_creature_only_if_he_was_cast() {
    cr!("603.4", "601.2");
    assert_supported("Anti-Venom, Horrifying Healer");
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(dead)]);
    t.enter(P0, "Anti-Venom, Horrifying Healer");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "put onto the battlefield, not cast");
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let av = t.hand(P0, "Anti-Venom, Horrifying Healer");
    t.cast(P0, av).go();
    t.answer_targets(P0, &[Entity::Object(dead)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn rapid_augmenter_counts_creatures_that_werent_cast() {
    cr!("603.4");
    assert_supported("Rapid Augmenter");
    let mut t = TestGame::new(2);
    let ra = t.battlefield(P0, "Rapid Augmenter");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.counters(ra, "+1/+1"), 0);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(ra, "+1/+1"), 1);
}

#[test]
fn twinned_vision_draws_two_when_cast_from_the_graveyard() {
    cr!("702.34a", "601.2");
    assert_supported("Twinned Vision");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let tv = t.hand(P0, "Twinned Vision");
    let hand = t.hand_size(P0);
    t.cast(P0, tv).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    t.lands(P0, "Island", 3);
    let other = t.hand(P0, "Island");
    let tv = t.graveyard(P0, "Twinned Vision");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.cast(P0, tv)
        .method(CastMethod::Keyword(keywords::KeywordKind::Flashback))
        .go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn rock_jockey_stops_land_plays_the_turn_it_was_cast() {
    cr!("305.2", "601.2");
    assert_supported("Rock Jockey");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let rj = t.hand(P0, "Rock Jockey");
    t.cast(P0, rj).go();
    t.resolve();
    let land = t.hand(P0, "Mountain");
    assert!(t.play_land(P0, land).is_err());
    // Put onto the battlefield without being cast: lands can be played.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rock Jockey");
    let land = t.hand(P0, "Mountain");
    assert!(t.play_land(P0, land).is_ok());
}

#[test]
fn kotis_counts_creatures_that_entered_from_a_graveyard() {
    cr!("603.4", "400.7");
    assert_supported("Kotis, Sibsig Champion");
    let mut t = TestGame::new(2);
    let kotis = t.battlefield(P0, "Kotis, Sibsig Champion");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(kotis, "+1/+1"), 0);
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.g.move_object(dead, Zone::Battlefield, events::MoveCause::Effect, Some(P0));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(kotis, "+1/+1"), 2);
}

// --- How much mana was spent, and from what -----------------------------------------------

#[test]
fn vexing_bauble_counters_spells_cast_without_mana() {
    cr!("601.2h", "603.4");
    assert_supported("Vexing Bauble");
    assert_supported("Nix");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Vexing Bauble");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "mana was spent");
    // Nix counters a spell no mana was spent on, and not one that had mana spent.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Island", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    let nix = t.hand(P1, "Nix");
    t.g.turn.priority = Some(P1);
    t.cast(P1, nix).target(bolt).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn hired_hexblade_checks_for_treasure_mana() {
    cr!("601.2h", "603.4");
    assert_supported("Hired Hexblade");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let hh = t.hand(P0, "Hired Hexblade");
    let hand = t.hand_size(P0);
    t.cast(P0, hh).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    treasure(&mut t, P0);
    let hh = t.hand(P0, "Hired Hexblade");
    let hand = t.hand_size(P0);
    t.cast(P0, hh).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn jetmirs_fixer_checks_for_treasure_mana_spent_to_activate() {
    cr!("602.2b", "608.2c");
    assert_supported("Jetmir's Fixer");
    let mut t = TestGame::new(2);
    let fixer = t.battlefield(P0, "Jetmir's Fixer");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.activate(P0, fixer, 0, &[]).expect("activates");
    t.resolve();
    assert_eq!(t.counters(fixer, "+1/+1"), 0);
    assert_eq!(t.pt(fixer), (3, 3));
    t.lands(P0, "Mountain", 1);
    treasure(&mut t, P0);
    t.activate(P0, fixer, 0, &[]).expect("activates");
    t.resolve();
    assert_eq!(t.counters(fixer, "+1/+1"), 1);
    assert_eq!(t.pt(fixer), (4, 4));
}

#[test]
fn prompto_argentum_needs_four_mana_spent_on_the_spell() {
    cr!("601.2h", "603.4");
    assert_supported("Prompto Argentum");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prompto Argentum");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Treasure Token"), 0);
    t.lands(P0, "Mountain", 5);
    let fireball = t.hand(P0, "Fireball");
    t.cast(P0, fireball).x(3).target(P1).go();
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Treasure Token"), 1);
}

// --- Ordinals and activation counts ---------------------------------------------------

#[test]
fn karlach_adds_a_combat_only_in_the_first_combat_phase() {
    cr!("506.1", "500.8");
    assert_supported("Karlach, Fury of Avernus");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Karlach, Fury of Avernus");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(k, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 15);
    // A second combat phase follows; attacking in it doesn't add a third.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(k, Entity::Player(P1))]),
    );
    t.advance_to_step(Step::PostcombatMain);
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.g.turn.combat_phases, 2);
}

#[test]
fn vengevine_returns_on_the_second_creature_spell() {
    cr!("603.4", "601.2i");
    assert_supported("Vengevine");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Vengevine");
    t.lands(P0, "Forest", 4);
    let b1 = t.hand(P0, "Grizzly Bears");
    let b2 = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.cast(P0, b1).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Vengevine").len(), 0);
    t.answer_yes(P0, true);
    t.cast(P0, b2).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Vengevine").len(), 1);
}

#[test]
fn nalathni_dragon_is_sacrificed_after_four_activations() {
    cr!("602.2", "603.7");
    assert_supported("Nalathni Dragon");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Nalathni Dragon");
    t.lands(P0, "Mountain", 4);
    for _ in 0..3 {
        t.activate(P0, d, 0, &[]).expect("activates");
        t.resolve();
    }
    to_end_step(&mut t, P0);
    assert!(t.on_battlefield(d), "three activations");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Nalathni Dragon");
    t.lands(P0, "Mountain", 4);
    for _ in 0..4 {
        t.activate(P0, d, 0, &[]).expect("activates");
        t.resolve();
    }
    assert_eq!(t.pt(d), (5, 1));
    to_end_step(&mut t, P0);
    assert!(!t.on_battlefield(d));
}

#[test]
fn omnath_does_something_different_each_time() {
    cr!("603.4", "608.2c");
    assert_supported("Omnath, Locus of Creation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnath, Locus of Creation");
    for _ in 0..3 {
        t.enter(P0, "Forest");
        t.resolve_all();
    }
    assert_eq!(t.life(P0), 24, "4 life the first time only");
    assert_eq!(t.g.player(P0).mana_pool.total(), 4, "the second time: RGWU");
    assert_eq!(t.life(P1), 16, "the third time: 4 damage to each opponent");
}
