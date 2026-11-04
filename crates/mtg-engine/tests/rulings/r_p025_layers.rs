//! Rulings batch P025 — copy effects in the layer system: a copy effect (layer 1) leaves
//! the non-copy effects already applied to the permanent in place (CR 613.1a, 707.2),
//! replaces the permanent's own abilities (so a permanent that becomes a copy loses the
//! ability that made it a copy, CR 707.2), lasts as long as its duration says (CR 611.2),
//! and the exceptions it makes are copiable values (CR 707.9b) that later layers can still
//! change (CR 613.1f).

use crate::r_p025_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Ends the turn: advances to P1's upkeep (P0's cleanup step ends "until end of turn"
/// effects).
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
}

// --- Non-copy effects keep applying -------------------------------------------------------

#[test]
fn deepfathom_echo_keeps_its_explore_counter_and_loses_its_trigger() {
    cr!("707.2", "613.1a", "613.4c", "611.2a", "701.44a");
    ruling!(
        "Deepfathom Echo",
        "Non-copy effects that have already applied to Deepfathom Echo will continue to apply to it. For example, +1/+1 counters on it will still affect it."
    );
    ruling!(
        "Deepfathom Echo",
        "Once Deepfathom Echo becomes a copy of another creature, it no longer has its triggered ability until end of turn."
    );
    supported("Deepfathom Echo");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let echo = t.battlefield(P0, "Deepfathom Echo");
    let giant = t.battlefield(P0, "Hill Giant");
    // It explores a nonland card (a +1/+1 counter), then becomes a copy of the Giant.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(name_of(&t, echo), "Hill Giant");
    assert_eq!(t.counters(echo, counters::PLUS1), 1);
    assert_eq!(t.pt(echo), (4, 4));
    assert_eq!(abilities_with(&t, echo, "explores"), 0);
    // Its own ability is back next turn.
    next_turn(&mut t);
    assert_eq!(name_of(&t, echo), "Deepfathom Echo");
    assert_eq!(abilities_with(&t, echo, "explores"), 1);
}

#[test]
fn nanogene_conversion_and_mirrorweave_keep_earlier_pumps() {
    cr!("707.2", "613.1a", "613.4c");
    ruling!(
        "Nanogene Conversion",
        "Non-copy effects that have already applied to the other creatures will continue to apply to them. For example, if Giant Growth had given one of them +3/+3 earlier in the turn, then Nanogene Conversion made it a copy of Bear Cub, it will be a 5/5 Bear Cub."
    );
    ruling!(
        "Mirrorweave",
        "Noncopy effects that have already applied to the other creatures will continue to apply to them. For example, if Giant Growth had given one of them +3/+3 earlier in the turn, then Mirrorweave made it a copy of Grizzly Bears, it will be a 5/5 Grizzly Bears."
    );
    for (spell, cost, model) in [
        ("Nanogene Conversion", "{3}{U}", "Bear Cub"),
        ("Mirrorweave", "{2}{U}{U}", "Grizzly Bears"),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, model);
        let giant = t.battlefield(P0, "Hill Giant");
        giant_growth(&mut t, P0, giant);
        assert_eq!(t.pt(giant), (6, 6));
        cast_for_tokens(&mut t, P0, spell, cost, &[obj(bears)]);
        assert_eq!(name_of(&t, giant), model, "{spell}");
        assert_eq!(t.pt(giant), (5, 5), "{spell}");
    }
}

#[test]
fn renegade_doppelganger_keeps_an_earlier_pump() {
    cr!("707.2", "613.1a", "613.4c");
    ruling!(
        "Renegade Doppelganger",
        "Noncopy effects that have already applied to Renegade Doppelganger will continue to apply to it after it becomes a copy of something else. For example, if Giant Growth had given it +3/+3 earlier in the turn, then it becomes a copy of Runeclaw Bear (a 2/2 creature), it will be a 5/5 Runeclaw Bear."
    );
    supported("Renegade Doppelganger");
    let mut t = TestGame::new(2);
    let dopp = t.battlefield(P0, "Renegade Doppelganger");
    giant_growth(&mut t, P0, dopp);
    t.answer_yes(P0, true);
    t.enter(P0, "Runeclaw Bear");
    t.settle();
    t.resolve_all();
    assert_eq!(name_of(&t, dopp), "Runeclaw Bear");
    assert_eq!(t.pt(dopp), (5, 5));
}

#[test]
fn identity_thief_keeps_woodcutters_grit_bonuses() {
    cr!("707.2", "613.1a", "613.1f", "613.4c");
    ruling!(
        "Identity Thief",
        "Noncopy effects that have already applied to Identity Thief will continue to apply to it. For example, if Woodcutter's Grit gave Identity Thief +3/+3 and hexproof earlier in the turn, Identity Thief will still have those bonuses after copying a creature."
    );
    supported("Identity Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Identity Thief");
    lands_for(&mut t, P0, "{2}{G}");
    let grit = t.hand(P0, "Woodcutter's Grit");
    t.cast(P0, grit).target(obj(thief)).go();
    t.resolve_all();
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.attack(&[(thief, Entity::Player(P1))], &[]);
    assert_eq!(name_of(&t, thief), "Hill Giant");
    assert_eq!(t.pt(thief), (6, 6));
    assert!(has_kw(&t, thief, KeywordKind::Hexproof));
}

#[test]
fn the_myriad_pools_target_keeps_its_counters() {
    cr!("707.2", "613.1a", "613.4c");
    ruling!(
        "The Everflowing Well // The Myriad Pools",
        "Non-copy effects that have already applied to the target permanent will continue to apply to it. For example, if it's a creature, +1/+1 counters on it will still affect it."
    );
    supported("The Everflowing Well // The Myriad Pools");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let well = t.battlefield(P0, "The Everflowing Well // The Myriad Pools");
    assert!(mtg_engine::dfc::transform(&mut t.g, well));
    t.g.recompute();
    let giant = t.battlefield(P0, "Hill Giant");
    plus1(&mut t, giant, 2);
    t.lands(P0, "Island", 1);
    // Bronze Sable: a 2/1 artifact creature for {2}, paid with the Pools' mana.
    let sable = t.hand(P0, "Bronze Sable");
    t.answer_targets(P0, &[obj(giant)]);
    t.cast(P0, sable).go();
    t.settle();
    t.resolve();
    t.g.recompute();
    assert_eq!(name_of(&t, giant), "Bronze Sable");
    assert_eq!(t.pt(giant), (4, 3));
}

#[test]
fn the_ever_changing_dane_keeps_its_counters_but_not_the_sacrificed_creatures() {
    cr!("707.2", "707.3", "613.1a", "608.2h");
    ruling!(
        "The Ever-Changing 'Dane",
        "The Ever-Changing ’Dane copies the printed values of the creature plus any copy effects that have been applied to it. It won’t copy any other effects that have changed that creature’s power, toughness, color, and so on. The Ever-Changing ’Dane won’t copy any counters on the creature, but The Ever-Changing ’Dane will retain any counters it already had on it."
    );
    supported("The Ever-Changing 'Dane");
    let mut t = TestGame::new(2);
    let dane = t.battlefield(P0, "The Ever-Changing 'Dane");
    plus1(&mut t, dane, 1);
    // A Clone copying a Hill Giant, dressed up, is sacrificed.
    let giant = t.battlefield(P0, "Hill Giant");
    let clone = clone_of(&mut t, P0, giant);
    dress(&mut t, P0, clone);
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(clone)]);
    activate_containing(&mut t, P0, dane, "becomes a copy").expect("activation");
    t.resolve_all();
    assert!(!t.on_battlefield(clone));
    let o = t.obj_now(dane);
    assert_eq!(o.chars.name, "Hill Giant");
    assert_eq!(o.chars.colors, red());
    assert_eq!(t.counters(dane, counters::PLUS1), 1);
    assert_eq!(t.pt(dane), (4, 4));
    assert_eq!(abilities_with(&t, dane, "Sacrifice another creature"), 1);
}

// --- Losing the ability that made it a copy --------------------------------------------------

#[test]
fn mirrors_lose_their_copy_ability_once_it_resolves() {
    cr!("707.2", "613.1a");
    ruling!(
        "Mirage Mirror",
        "Once Mirage Mirror's ability resolves, it no longer has that ability."
    );
    ruling!(
        "Mirror of the Forebears",
        "Once the second ability of Mirror of the Forebears resolves, it no longer has that ability."
    );
    supported("Mirage Mirror");
    supported("Mirror of the Forebears");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let mirage = t.battlefield(P0, "Mirage Mirror");
    lands_for(&mut t, P0, "{2}");
    t.activate(P0, mirage, 0, &[obj(giant)]).expect("activate");
    t.resolve_all();
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Giant");
    let forebears = t.enter(P0, "Mirror of the Forebears");
    t.settle();
    lands_for(&mut t, P0, "{1}");
    t.activate(P0, forebears, 0, &[obj(giant)]).expect("activate");
    t.resolve_all();
    for m in [mirage, forebears] {
        assert_eq!(name_of(&t, m), "Hill Giant");
        assert_eq!(abilities_with(&t, m, "becomes a copy"), 0);
    }
}

#[test]
fn vesuvan_drifter_copying_a_card_loses_its_own_abilities() {
    cr!("707.2", "613.1a");
    ruling!(
        "Vesuvan Drifter",
        "Notably, while Vesuvan Drifter is a copy of another card, it doesn't have its own abilities. You won't be able to look at the top card of your library unless another effect allows you to do so."
    );
    supported("Vesuvan Drifter");
    let mut t = TestGame::new(2);
    let drifter = t.battlefield(P0, "Vesuvan Drifter");
    assert_eq!(abilities_with(&t, drifter, "look at the top card"), 1);
    t.library_top(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(name_of(&t, drifter), "Hill Giant");
    assert!(has_kw(&t, drifter, KeywordKind::Flying));
    assert_eq!(abilities_with(&t, drifter, "look at the top card"), 0);
    assert_eq!(abilities_with(&t, drifter, "reveal the top card"), 0);
}

#[test]
fn unstable_shapeshifters_granted_ability_is_removed_by_an_earlier_lose_all_abilities_effect() {
    cr!("707.9a", "613.1a", "613.1f", "613.7");
    ruling!(
        "Unstable Shapeshifter",
        "The ability is granted as part of the copy effect, which means it will be removed by any effects that would remove abilities regardless of when the effect was created."
    );
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    t.enter(P1, "Hill Giant");
    t.settle();
    assert_eq!(triggers_on_stack(&t).len(), 1);
    // In response, the Shapeshifter loses all abilities (an earlier timestamp than the
    // copy effect it's about to get).
    modify(
        &mut t,
        shifter,
        vec![Modification::RemoveAllAbilities],
    );
    t.resolve_all();
    assert_eq!(name_of(&t, shifter), "Hill Giant");
    // The granted "this ability" is removed too: another creature entering doesn't make
    // it a copy.
    assert_eq!(abilities_with(&t, shifter, "becomes a copy"), 0);
    t.enter(P1, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(name_of(&t, shifter), "Hill Giant");
}

#[test]
fn protean_thaumaturge_stays_a_copy_indefinitely() {
    cr!("707.2", "611.2a", "514.2");
    ruling!(
        "Protean Thaumaturge",
        "Protean Thaumaturge’s ability causes it to become a copy indefinitely. It doesn’t expire during the cleanup step or if the target creature leaves the battlefield after the ability has resolved."
    );
    supported("Protean Thaumaturge");
    let mut t = TestGame::new(2);
    let thaum = t.battlefield(P0, "Protean Thaumaturge");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Leyline of Anticipation");
    t.settle();
    t.resolve_all();
    assert_eq!(name_of(&t, thaum), "Hill Giant");
    t.g.destroy(giant, None);
    t.settle();
    next_turn(&mut t);
    assert_eq!(name_of(&t, thaum), "Hill Giant");
    assert_eq!(t.pt(thaum), (3, 3));
}

/// P0's Tilonalli's Skinshifter attacks alongside `other` and becomes a copy of it.
fn skinshifter_attacks_with(t: &mut TestGame, other: ObjectId) -> ObjectId {
    supported("Tilonalli's Skinshifter");
    let skin = t.battlefield(P0, "Tilonalli's Skinshifter");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(other)]);
    t.attack(
        &[(skin, Entity::Player(P1)), (other, Entity::Player(P1))],
        &[],
    );
    skin
}

#[test]
fn tilonallis_skinshifter_keeps_its_new_characteristics() {
    cr!("707.2", "611.2c");
    ruling!(
        "Tilonalli's Skinshifter",
        "Once the ability of Tilonalli’s Skinshifter resolves, its new characteristics don’t change if the characteristics of the copied creature change or if the copied creature leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let skin = skinshifter_attacks_with(&mut t, giant);
    t.resolve_all();
    assert_eq!(name_of(&t, skin), "Hill Giant");
    // The Giant becomes a blue 1/1 Frog with no abilities, then leaves.
    modify(
        &mut t,
        giant,
        vec![
            Modification::SetPT(Some(mtg_engine::ability::Value::c(1)), Some(mtg_engine::ability::Value::c(1))),
            Modification::SetColors(ColorSet::single(Color::Blue)),
        ],
    );
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(t.pt(skin), (3, 3));
    assert_eq!(t.obj_now(skin).chars.colors, red());
    t.g.destroy(giant, None);
    t.settle();
    assert_eq!(name_of(&t, skin), "Hill Giant");
    assert_eq!(t.pt(skin), (3, 3));
}

#[test]
fn tilonallis_skinshifter_copying_an_animated_land_is_removed_from_combat() {
    cr!("707.2", "613.1a", "506.4");
    ruling!(
        "Tilonalli's Skinshifter",
        "Notably, Tilonalli’s Skinshifter won’t copy effects that make a noncreature permanent become a creature. If this causes Tilonalli’s Skinshifter to stop being a creature, it’s removed from combat."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    animate_land(&mut t, land);
    let skin = skinshifter_attacks_with(&mut t, land);
    t.resolve_all();
    assert_eq!(name_of(&t, skin), "Forest");
    assert!(!is_creature(&t, skin));
    let combat = t.g.combat.as_ref().expect("combat");
    assert!(!combat.attackers.iter().any(|a| a.id == t.g.current(skin)));
}

// --- Gigantoplasm's base power and toughness ---------------------------------------------------

#[test]
fn gigantoplasms_ability_sets_base_power_and_toughness_in_timestamp_order() {
    cr!("707.9a", "613.4b", "613.4c", "613.7");
    ruling!(
        "Gigantoplasm",
        "The activated ability overwrites all previous effects that set Gigantoplasm's base power and/or toughness to specific values. Other effects that set its base power and/or toughness that start to apply after the ability resolves (including ones created by subsequent activations of the ability) will overwrite the effect of the ability."
    );
    ruling!(
        "Gigantoplasm",
        "The activated ability that Gigantoplasm gives itself doesn't have a duration. It lasts until Gigantoplasm leaves the battlefield or another effect overwrites it (for example, if you activate the ability again once you have more mana available). Each time the ability resolves, it will overwrite any previous effects that set its power and toughness, including previous activations of the ability. Effects that otherwise modify Gigantoplasm's power and/or toughness, including +1/+1 counters, will still apply no matter when they took effect."
    );
    supported("Gigantoplasm");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let g = enter_copying(&mut t, P0, "Gigantoplasm", giant);
    let x = |t: &mut TestGame, n: usize| {
        t.lands(P0, "Wastes", n);
        t.answer(P0, DecisionKind::X, Answer::Number(n as i64));
        activate_containing(t, P0, g, "base power and toughness").expect("activation");
        t.resolve_all();
    };
    // An earlier effect sets its base power and toughness to 1/1; X = 5 overwrites it.
    modify(
        &mut t,
        g,
        vec![Modification::SetPT(
            Some(mtg_engine::ability::Value::c(1)),
            Some(mtg_engine::ability::Value::c(1)),
        )],
    );
    assert_eq!(t.pt(g), (1, 1));
    x(&mut t, 5);
    assert_eq!(t.pt(g), (5, 5));
    // A +1/+1 counter and a pump apply whenever they started.
    plus1(&mut t, g, 1);
    giant_growth(&mut t, P0, g);
    assert_eq!(t.pt(g), (9, 9));
    // A later activation with X = 2 overwrites the first.
    x(&mut t, 2);
    assert_eq!(t.pt(g), (6, 6));
    // A later effect setting it to 0/4 overwrites the ability.
    modify(
        &mut t,
        g,
        vec![Modification::SetPT(
            Some(mtg_engine::ability::Value::c(0)),
            Some(mtg_engine::ability::Value::c(4)),
        )],
    );
    assert_eq!(t.pt(g), (4, 8));
    // It lasts past the end of the turn (the 0/4 and the pump don't).
    next_turn(&mut t);
    assert_eq!(t.pt(g), (3, 3));
}
