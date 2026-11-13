//! Cost modifiers (CR 601.2f, 602.2b, 118.9): additional non-mana costs for spells and
//! activated abilities ("an additional "Sacrifice a land" to activate", "for each black
//! mana symbol in their activation costs"), per-target changes ("{1} less to cast for each
//! target"), the first targeting activation each turn, an alternative to an echo cost, a
//! capped reduction, and reductions conditioned on what the spell targets.

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn untapped(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is(types::CardType::Land) && !o.tapped)
        .count()
}

#[test]
fn brutal_suppression_rebels_sacrifice_a_land_to_activate() {
    cr!("602.2b", "601.2f", "118.3");
    assert_supported("Brutal Suppression");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Brutal Suppression");
    // Amrou Scout: "{4}, {T}: Search your library for a Rebel permanent card ..."
    let scout = t.battlefield(P0, "Amrou Scout");
    t.lands(P0, "Plains", 4);
    let lands_before =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.is(types::CardType::Land))
            .count();
    t.activate(P0, scout, 0, &[]).unwrap();
    let lands_after =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.is(types::CardType::Land))
            .count();
    assert_eq!(lands_after, lands_before - 1);
}

#[test]
fn drought_sacrifices_a_swamp_for_each_black_symbol() {
    cr!("602.2b", "601.2f", "118.3");
    ruling!(
        "Drought",
        "that’s 4 activations with {B} each so you sacrifice 4 Swamps"
    );
    assert_supported("Drought");
    let swamps = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Swamp"))
            .count()
    };
    // Pestilence: "{B}: Pestilence deals 1 damage to each creature and each player."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Drought");
    let pest = t.battlefield(P0, "Pestilence");
    // Black mana but no Swamp: the additional cost can't be paid.
    t.lands(P0, "Sulfurous Springs", 1);
    assert!(t.activate(P0, pest, 0, &[]).is_err());
    // With a Swamp to sacrifice.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Drought");
    let pest = t.battlefield(P0, "Pestilence");
    t.lands(P0, "Sulfurous Springs", 1);
    t.lands(P0, "Swamp", 1);
    t.activate(P0, pest, 0, &[]).unwrap();
    assert_eq!(swamps(&t), 0);
    // A spell with one black symbol: one Swamp.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Drought");
    t.lands(P0, "Swamp", 3);
    let duress = t.hand(P0, "Duress");
    t.cast(P0, duress).target(Entity::Player(P1)).go();
    assert_eq!(swamps(&t), 2);
    // No black symbols: no Swamp needed.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Drought");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn hinata_changes_costs_for_each_target() {
    cr!("601.2f", "601.2c");
    ruling!(
        "Hinata, Dawn-Crowned",
        "count the number of players or objects that are targets"
    );
    assert_supported("Hinata, Dawn-Crowned");
    // Your spell with two targets costs {2} less.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hinata, Dawn-Crowned");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    // Arc Lightning ({2}{R}, divided among one, two, or three targets): {R} with two.
    let arc = t.hand(P0, "Arc Lightning");
    t.cast(P0, arc)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    assert_eq!(untapped(&t, P0), 0);
    // An opponent's one-target spell costs {1} more.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hinata, Dawn-Crowned");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t
        .cast(P1, bolt)
        .target(Entity::Player(P0))
        .try_go()
        .is_err());
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert_eq!(untapped(&t, P1), 0);
}

#[test]
fn hinata_counts_a_creature_targeted_twice_once() {
    cr!("601.2f", "601.2c", "115.3");
    ruling!(
        "Hinata, Dawn-Crowned",
        "{1} less to cast if each instance targeted the same creature"
    );
    // Subtle Strike ({1}{B}), both modes at P0's one creature: {1} more, not {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hinata, Dawn-Crowned");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let strike = t.hand(P1, "Subtle Strike");
    t.cast(P1, strike)
        .modes(&[0, 1])
        .target(bears)
        .target(bears)
        .go();
    assert_eq!(untapped(&t, P1), 0);
    // Two different creatures: {2} more, so three Swamps aren't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hinata, Dawn-Crowned");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let strike = t.hand(P1, "Subtle Strike");
    assert!(t
        .cast(P1, strike)
        .modes(&[0, 1])
        .target(bears)
        .target(giant)
        .try_go()
        .is_err());
}

#[test]
fn professor_hojo_reduces_the_first_targeting_activation_each_of_your_turns() {
    cr!("602.2b", "601.2f");
    ruling!("Professor Hojo", "reduces only the amount of generic mana");
    assert_supported("Professor Hojo");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Professor Hojo");
    // Equip {2}: free with Hojo for the first targeting activation.
    let sword = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, sword, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    // The second costs {1} again.
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(t.activate(P0, sword, 0, &[Entity::Object(giant)]).is_err());
    t.clear_answers();
    t.lands(P0, "Plains", 1);
    t.activate(P0, sword, 0, &[Entity::Object(giant)]).unwrap();
    assert_eq!(untapped(&t, P0), 0);
}

#[test]
fn thick_skinned_goblin_pays_zero_for_echo() {
    cr!("702.30a");
    ruling!(
        "Thick-Skinned Goblin",
        "you may pay {0}, pay its echo cost, or choose not to pay"
    );
    assert_supported("Thick-Skinned Goblin");
    // Without the Goblin and without mana, the echo creature is sacrificed.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let riders = t.enter(P0, "Avalanche Riders");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(riders));
    // With it, {0} is paid instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thick-Skinned Goblin");
    t.set_step(P1, Step::End);
    let riders = t.enter(P0, "Avalanche Riders");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(riders));
}

#[test]
fn valiant_changeling_reduction_is_capped_at_five() {
    cr!("601.2f");
    assert_supported("Valiant Changeling");
    let mut t = TestGame::new(2);
    // A changeling creature has every creature type: the reduction would be huge.
    t.battlefield(P0, "Changeling Outcast");
    t.lands(P0, "Plains", 2);
    let vc = t.hand(P0, "Valiant Changeling");
    // {5}{W}{W} reduced by at most {5}: {W}{W}.
    t.cast(P0, vc).go();
    assert_eq!(untapped(&t, P0), 0);
    // One creature type: {1} less only.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let vc = t.hand(P0, "Valiant Changeling");
    assert!(t.cast(P0, vc).try_go().is_err());
}

#[test]
fn lullmages_domination_costs_less_targeting_a_creature_of_a_player_with_eight_cards() {
    cr!("601.2f", "601.2c");
    ruling!("Lullmage's Domination", "subtracting {3} if appropriate");
    assert_supported("Lullmage's Domination");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    for _ in 0..7 {
        t.graveyard(P1, "Island");
    }
    // X = 2 with seven cards: {2}{U}{U}{U} is more than four Islands.
    let ld = t.hand(P0, "Lullmage's Domination");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    assert!(t.cast(P0, ld).target(bears).try_go().is_err());
    t.clear_answers();
    // Eight cards: {3} less, {U}{U}{U}.
    t.graveyard(P1, "Island");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, ld).target(bears).go();
    assert_eq!(untapped(&t, P0), 1);
    t.resolve();
    assert_eq!(t.g.obj(bears).controller, P0);
}

#[test]
fn monastery_siege_dragons_taxes_spells_that_target_you_or_your_permanents() {
    cr!("601.2f", "115.9b");
    ruling!("Monastery Siege", "adds {2} per spell, not per target");
    assert_supported("Monastery Siege");
    let siege_game = || {
        let mut t = TestGame::new(2);
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        t.enter(P0, "Monastery Siege");
        t.set_step(P1, Step::PrecombatMain);
        t
    };
    // A Lightning Bolt at P0: {R} plus {2}.
    let mut t = siege_game();
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t
        .cast(P1, bolt)
        .target(Entity::Player(P0))
        .try_go()
        .is_err());
    t.clear_answers();
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert_eq!(untapped(&t, P1), 0);
    // At P0's creature and P0 at once: still {2} more, once.
    let mut t = siege_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 3);
    let fork = t.hand(P1, "Forked Bolt");
    t.cast(P1, fork)
        .targets(&[Entity::Player(P0), Entity::Object(bears)])
        .go();
    assert_eq!(untapped(&t, P1), 0);
    // At P1's own creature: no tax.
    let mut t = siege_game();
    let own = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(own).go();
    assert_eq!(untapped(&t, P1), 0);
}

#[test]
fn not_of_this_world_is_free_against_a_spell_targeting_your_big_creature() {
    cr!("601.2f", "115.9b");
    ruling!(
        "Not of This World",
        "This check is made as you cast Not of This World"
    );
    assert_supported("Not of This World");
    let mut t = TestGame::new(2);
    let bronto = t.battlefield(P0, "Ancient Brontodon");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    let spell = t.cast(P1, murder).target(bronto).go();
    // P0 has no mana: {7} less makes it free.
    let notw = t.hand(P0, "Not of This World");
    t.cast(P0, notw).target(spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(bronto));
    // Against a smaller creature, it costs {7}.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    let spell = t.cast(P1, murder).target(bears).go();
    let notw = t.hand(P0, "Not of This World");
    assert!(t.cast(P0, notw).target(spell).try_go().is_err());
}
