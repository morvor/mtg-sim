//! Where the mana spent came from, round 4 (cards4-costs-mana-replacements:
//! `oracle/patterns/mana_5.rs`, `costs_5.rs`, `kw/cast_history.rs`): spells that have a
//! keyword if mana from an artifact or a Treasure was spent to cast them, snow mana of a
//! spell's colors, mana spent to activate a triggering ability, spend restrictions by
//! source for one way of casting, and sources of your choice that share a color with the
//! mana spent on an activation cost or with an exiled card.

use mtg_engine::ability::{AbilityKind, CostChange, StaticEffect};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
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

fn has(t: &mut TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.g.recompute();
    t.g.obj(id).chars.keywords().any(|x| x.kind == k)
}

#[test]
fn shadow_gives_split_second_to_spells_cast_with_artifact_mana() {
    cr!("106.3", "601.2h", "702.61a");
    assert_supported(&["Shadow the Hedgehog"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow the Hedgehog");
    // Paid with lands: no split second.
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    assert!(!has(&mut t, spell, KeywordKind::SplitSecond));
    t.resolve_all();
    // A Treasure (an artifact) pays part of it: split second.
    t.battlefield(P0, "Forest");
    treasure(&mut t, P0);
    let bears2 = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears2).go();
    assert!(has(&mut t, spell, KeywordKind::SplitSecond));
    // An opponent's spell paid with an artifact isn't "a spell you cast".
    t.resolve_all();
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow the Hedgehog");
    t.battlefield(P1, "Forest");
    treasure(&mut t, P1);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    assert!(!has(&mut t, spell, KeywordKind::SplitSecond));
}

#[test]
fn rain_of_riches_gives_cascade_to_the_first_spell_cast_with_treasure_mana() {
    cr!("106.3", "702.85a");
    assert_supported(&["Rain of Riches"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rain of Riches");
    // A spell paid with lands first: no cascade, and it doesn't use up the turn's first.
    t.lands(P0, "Forest", 2);
    let b1 = t.hand(P0, "Grizzly Bears");
    let s1 = t.cast(P0, b1).go();
    assert!(!has(&mut t, s1, KeywordKind::Cascade));
    t.resolve_all();
    // The first spell with Treasure mana: cascade.
    t.battlefield(P0, "Forest");
    treasure(&mut t, P0);
    let b2 = t.hand(P0, "Grizzly Bears");
    let s2 = t.cast(P0, b2).go();
    assert!(has(&mut t, s2, KeywordKind::Cascade));
    // Granted as the spell becomes cast, so its cascade triggers (CR 702.85a).
    assert_eq!(t.stack_len(), 2, "the spell and its cascade trigger");
    t.resolve_all();
    // The second one this turn: none.
    t.battlefield(P0, "Forest");
    treasure(&mut t, P0);
    let b3 = t.hand(P0, "Grizzly Bears");
    let s3 = t.cast(P0, b3).go();
    assert!(!has(&mut t, s3, KeywordKind::Cascade));
}

#[test]
fn vazi_triggers_on_treasure_mana_spent_to_activate_an_ability() {
    cr!("106.3", "602.2b", "603.4");
    assert_supported(&["Vazi, Keen Negotiator"]);
    let mut t = TestGame::new(2);
    let vazi = t.battlefield(P0, "Vazi, Keen Negotiator");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    let hand = t.hand_size(P0);
    // Firebreathing paid with a Mountain: no trigger.
    t.battlefield(P1, "Mountain");
    t.activate(P1, dragon, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Paid with a Treasure: a +1/+1 counter on target creature, then draw.
    treasure(&mut t, P1);
    t.answer_targets(P0, &[Entity::Object(vazi)]);
    t.activate(P1, dragon, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(vazi, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    // A spell cast with Treasure mana still counts.
    treasure(&mut t, P1);
    t.battlefield(P1, "Forest");
    let bears = t.hand(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(vazi)]);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.cast(P1, bears).go();
    t.resolve_all();
    assert_eq!(t.counters(vazi, "+1/+1"), 2);
}

#[test]
fn boreal_outrider_needs_snow_mana_of_one_of_the_spells_colors() {
    cr!("106.3", "107.4h", "603.4");
    ruling!("Boreal Outrider", "cares about the color of that snow mana");
    assert_supported(&["Boreal Outrider"]);
    // Snow green mana for a green creature: an additional counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boreal Outrider");
    t.battlefield(P0, "Snow-Covered Forest");
    t.battlefield(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(bears), "+1/+1"), 1);
    // Snow blue mana (on the generic part) for a green creature: none.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boreal Outrider");
    t.battlefield(P0, "Snow-Covered Island");
    t.battlefield(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(bears), "+1/+1"), 0);
}

fn alternative_cost_uid(t: &TestGame, id: ObjectId) -> u64 {
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .find(|a| {
            matches!(&a.kind, AbilityKind::Static(s) if matches!(&s.effect,
                StaticEffect::CostModifier(m) if matches!(m.change, CostChange::AlternativeCost(_))))
        })
        .expect("alternative cost")
        .uid
}

#[test]
fn security_rhox_alternative_cost_takes_only_treasure_mana() {
    cr!("106.3", "118.9", "601.2h");
    assert_supported(&["Security Rhox"]);
    // With only lands, it can't be cast for {R}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mountain");
    t.battlefield(P0, "Forest");
    let rhox = t.hand(P0, "Security Rhox");
    let uid = alternative_cost_uid(&t, rhox);
    assert!(t
        .cast(P0, rhox)
        .method(CastMethod::Alternative(uid))
        .try_go()
        .is_err());
    // With two Treasures (and the lands still there), the Treasures pay it.
    let a = treasure(&mut t, P0);
    let b = treasure(&mut t, P0);
    t.cast(P0, rhox).method(CastMethod::Alternative(uid)).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(rhox)));
    assert!(
        !t.g.is_live(a) && !t.g.is_live(b),
        "the Treasures were sacrificed"
    );
    let lands_untapped =
        t.g.permanents_controlled_by(P0)
            .into_iter()
            .filter(|o| t.g.obj(*o).chars.is(mtg_engine::types::CardType::Land))
            .all(|o| !t.g.obj(o).tapped);
    assert!(lands_untapped);
    // Cast normally, any mana pays its mana cost.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Forest", 2);
    let rhox = t.hand(P0, "Security Rhox");
    t.cast(P0, rhox).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(rhox)));
}

#[test]
fn light_up_the_night_x_cant_be_0_only_with_flashback() {
    cr!("107.3a", "702.34a");
    assert_supported(&["Light Up the Night"]);
    let x_min = |t: &TestGame| {
        t.asked()
            .into_iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseX { min, .. } => Some(min),
                _ => None,
            })
            .last()
    };
    // From the hand: X may be 0.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Light Up the Night");
    t.cast(P0, spell).target(P1).x(0).go();
    assert_eq!(x_min(&t), Some(0));
    // With flashback: X can't be 0.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    t.battlefield(P0, "Chandra, Torch of Defiance");
    let spell = t.graveyard(P0, "Light Up the Night");
    t.cast(P0, spell)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(P1)
        .x(2)
        .go();
    assert_eq!(x_min(&t), Some(1));
}

#[test]
fn protective_sphere_chooses_a_source_sharing_a_color_with_the_mana_spent() {
    cr!("609.7a", "609.7b", "615.1a");
    interp!("shared-color-source-locked");
    assert_supported(&["Protective Sphere"]);
    let mut t = TestGame::new(2);
    let sphere = t.battlefield(P0, "Protective Sphere");
    t.battlefield(P0, "Mountain");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    // Red mana: the red Bolt can be chosen (the green Bears can't).
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, sphere, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 19, "paid 1 life");
    t.resolve();
    let candidates = t
        .asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if q == P0 && prompt.contains("source") => Some(candidates),
            _ => None,
        })
        .last()
        .expect("a source choice");
    assert_eq!(candidates, vec![Entity::Object(bolt)]);
    t.resolve();
    assert_eq!(t.life(P0), 19, "the Bolt's damage was prevented");
    // White mana: nothing red can be chosen, the Bolt deals its damage.
    let mut t = TestGame::new(2);
    let sphere = t.battlefield(P0, "Protective Sphere");
    t.battlefield(P0, "Plains");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, sphere, 0, &[]).unwrap();
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn mourners_shield_prevents_damage_from_a_source_sharing_a_color_with_the_exiled_card() {
    cr!("609.7a", "615.1a", "607.2a");
    assert_supported(&["Mourner's Shield"]);
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P1, "Lightning Bolt");
    t.lands(P0, "Plains", 6);
    let shield = t.hand(P0, "Mourner's Shield");
    t.answer_targets(P0, &[Entity::Object(dead)]);
    t.answer_yes(P0, true);
    t.cast(P0, shield).go();
    t.resolve_all();
    let shield = t.g.current(shield);
    assert!(t.in_exile("Lightning Bolt"));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(bears).go();
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, shield, 0, &[]).unwrap();
    t.resolve_all();
    // Damage to anything from the chosen red source is prevented.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.g.obj(bears).damage, 0);
}
