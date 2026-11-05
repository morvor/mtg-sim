//! Turn history and permissions VII (cards6-history-permissions): permissions over the top
//! card of another player's library (Xanathar, Guild Kingpin), block history anchored on
//! a target (Glyph of Reincarnation, Glyph of Delusion), and damage dealt by one of the
//! sorcery spells a chosen player cast this turn (Backdraft).

use crate::combat2_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn sees(t: &TestGame, p: PlayerId, id: ObjectId) -> bool {
    mtg_engine::zones::can_see_in_library(&t.g, p, t.g.current(id))
}

#[test]
fn xanathar_plays_the_top_card_of_the_target_opponents_library() {
    cr!("401.5", "601.3", "305.2", "609.4b", "611.2a");
    ruling!(
        "Xanathar, Guild Kingpin",
        "You'll still pay all costs for spells cast this way"
    );
    ruling!(
        "Xanathar, Guild Kingpin",
        "you can look at the top card of the chosen opponent's library whenever you want"
    );
    ruling!(
        "Xanathar, Guild Kingpin",
        "You must still follow all normal timing rules and land play limits."
    );
    supported("Xanathar, Guild Kingpin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Xanathar, Guild Kingpin");
    let mountain = t.library_top(P1, "Mountain");
    let forest = t.library_top(P1, "Forest");
    let axe = t.library_top(P1, "Lava Axe");
    let bolt = t.library_top(P1, "Lightning Bolt");
    // (P1 draws this one during their turn before P0's upkeep.)
    t.library_top(P1, "Island");
    let shock = t.hand(P1, "Shock");
    assert!(!sees(&t, P0, bolt));
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // P0 may look at the top card of P1's library; P1 can't cast spells.
    assert!(sees(&t, P0, bolt));
    t.g.players[1].mana_pool.add_type(ManaType::R, 1);
    assert!(t.cast_with(P1, shock, &[Entity::Player(P0)]).is_err());
    t.g.players[1].mana_pool = Default::default();
    // An instant at instant speed, paid with black mana spent as though it were red.
    t.g.players[0].mana_pool.add_type(ManaType::B, 1);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.players[0].mana_pool.total(), 0, "the cost was paid");
    assert!(t.in_graveyard(P1, "Lightning Bolt"), "its owner's graveyard");
    // The new top card: a sorcery, cast only at sorcery speed.
    assert!(sees(&t, P0, axe));
    t.g.players[0].mana_pool.add_type(ManaType::B, 5);
    assert!(t.cast_with(P0, axe, &[Entity::Player(P1)]).is_err());
    t.advance_to(P0, Step::PrecombatMain);
    t.g.players[0].mana_pool.add_type(ManaType::B, 5);
    t.cast_with(P0, axe, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Lands: the land play is P0's, and only one.
    t.play_land(P0, forest).unwrap();
    let f = t.g.current(forest);
    assert!(t.on_battlefield(f));
    assert_eq!(t.obj_now(f).controller, P0);
    assert_eq!(t.obj_now(f).owner, P1);
    assert!(sees(&t, P0, mountain));
    assert!(t.play_land(P0, mountain).is_err(), "one land a turn");
    // Until end of turn only.
    t.advance_to(P1, Step::Upkeep);
    assert!(!sees(&t, P0, mountain));
}

fn mana(t: &mut TestGame, p: PlayerId) {
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        t.g.players[p.idx()].mana_pool.add_type(ty, 3);
    }
}

/// `attacker`'s player attacks with `attackers`; `defender` blocks with `blocks`.
fn attack_and_block(
    t: &mut TestGame,
    attacker: PlayerId,
    attackers: &[ObjectId],
    defender: PlayerId,
    blocks: &[(ObjectId, ObjectId)],
) {
    let decl: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(defender)))
        .collect();
    crate::combat2_common::declare(t, attacker, &decl);
    crate::combat2_common::declare_blocks(t, defender, blocks);
}

fn ready(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = false;
    t.g.objects[id.0 as usize].summoning_sick = false;
}

#[test]
fn glyph_of_reincarnation_returns_cards_from_the_graveyard_of_the_controller_at_the_block() {
    cr!("509.1", "608.2c", "115.1");
    ruling!(
        "Glyph of Reincarnation",
        "First place all the affected creatures in the graveyard"
    );
    supported("Glyph of Reincarnation");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    ready(&mut t, bears);
    ready(&mut t, giant);
    let wurm = t.graveyard(P1, "Craw Wurm");
    let mine = t.graveyard(P0, "Hill Giant");
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[bears, giant], P0, &[(wall, bears)]);
    t.advance_to(P1, Step::PostcombatMain);
    // P0 takes the Bears: P1 controlled them when the Wall blocked them.
    let ray = t.hand(P0, "Ray of Command");
    mana(&mut t, P0);
    t.cast_with(P0, ray, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    let glyph = t.hand(P0, "Glyph of Reincarnation");
    mana(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.cast_with(P0, glyph, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    // Only the creature the Wall blocked was destroyed.
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    // The card came from P1's graveyard, not from P0's (the Bears' controller now).
    let w = t.g.current(wurm);
    assert!(t.on_battlefield(w));
    assert_eq!(t.obj_now(w).controller, P1, "under its owner's control");
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.g.current(mine), mine);
}

#[test]
fn glyph_of_reincarnation_on_an_opponents_wall_may_return_the_destroyed_creature() {
    cr!("509.1", "608.2c");
    ruling!(
        "Glyph of Reincarnation",
        "you can cast this targeting your opponent’s Walls"
    );
    ruling!(
        "Glyph of Reincarnation",
        "Some or all of them may be the same creatures which were just destroyed."
    );
    supported("Glyph of Reincarnation");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    ready(&mut t, bears);
    let glyph = t.hand(P0, "Glyph of Reincarnation");
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_and_block(&mut t, P0, &[bears], P1, &[(wall, bears)]);
    // "Cast this spell only after combat."
    mana(&mut t, P0);
    assert!(t.cast_with(P0, glyph, &[Entity::Object(wall)]).is_err());
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0);
    // The only creature card in P0's graveyard is the Bears just destroyed.
    t.cast_with(P0, glyph, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    let b = t.g.current(bears);
    assert_ne!(b, bears);
    assert!(t.on_battlefield(b), "the same card returned");
    assert_eq!(t.obj_now(b).controller, P0);
    assert!(t.on_battlefield(wall));
}

#[test]
fn glyph_of_delusion_targets_a_creature_the_target_wall_blocked() {
    cr!("509.1", "115.1", "601.2c");
    ruling!(
        "Glyph of Delusion",
        "Can be cast at any time after the wall blocked a creature"
    );
    interp!("anchor-target-chosen-first");
    supported("Glyph of Delusion");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    ready(&mut t, bears);
    ready(&mut t, giant);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[bears, giant], P0, &[(wall, bears)]);
    // The Giant wasn't blocked by the Wall: not a legal target with it.
    let glyph = t.hand(P0, "Glyph of Delusion");
    mana(&mut t, P0);
    let spell = t
        .cast_with(P0, glyph, &[Entity::Object(wall), Entity::Object(bears)])
        .unwrap();
    let targets = t.g.obj(spell).stack.as_ref().unwrap().chosen[0].targets.clone();
    assert_eq!(
        targets,
        vec![vec![Entity::Object(wall)], vec![Entity::Object(bears)]]
    );
    // The Wall is chosen first; then only the creature it blocked can be.
    assert_eq!(
        target_candidates(&t, "target creature"),
        Some(vec![Entity::Object(bears)])
    );
    t.resolve_all();
    assert_eq!(t.counters(bears, "glyph"), 2, "X is the Bears' power");
    assert_eq!(t.counters(giant, "glyph"), 0);
    // The Bears don't untap while they have a glyph counter; one is removed in each of
    // their controller's upkeeps.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(giant).tapped);
    assert_eq!(t.counters(bears, "glyph"), 1);
}

#[test]
fn glyph_of_delusion_needs_a_creature_that_wall_blocked() {
    cr!("115.1", "601.2c");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let other = t.battlefield(P0, "Wall of Wood");
    let bears = t.battlefield(P1, "Grizzly Bears");
    ready(&mut t, bears);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[bears], P0, &[(other, bears)]);
    t.advance_to(P1, Step::PostcombatMain);
    // Wall of Stone blocked nothing: no creature to target with it.
    let glyph = t.hand(P0, "Glyph of Delusion");
    mana(&mut t, P0);
    assert!(t
        .cast_with(P0, glyph, &[Entity::Object(wall), Entity::Object(bears)])
        .is_err());
    // With the Wall that blocked them, the Bears.
    t.clear_answers();
    mana(&mut t, P0);
    t.cast_with(P0, glyph, &[Entity::Object(other), Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "glyph"), 2);
}

/// The candidates of the latest target choice whose text starts with `text`.
fn target_candidates(t: &TestGame, text: &str) -> Option<Vec<Entity>> {
    t.asked().into_iter().rev().find_map(|(_, d)| match d {
        Decision::ChooseTargets {
            text: x, candidates, ..
        } if x.starts_with(text) => Some(candidates),
        _ => None,
    })
}

#[test]
fn backdraft_deals_half_the_damage_one_of_the_players_sorceries_dealt() {
    cr!("120.1", "107.1a", "608.2d");
    interp!("backdraft-controller-chooses-the-spell");
    supported("Backdraft");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    let axe = t.hand(P1, "Lava Axe");
    let sign = t.hand(P1, "Sign in Blood");
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1);
    t.cast_with(P1, axe, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    mana(&mut t, P1);
    t.cast_with(P1, sign, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    mana(&mut t, P1);
    t.cast_with(P1, bolt, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 12);
    assert_eq!(t.life(P1), 18);
    // P0 chooses P1 (the only player who cast a sorcery), then Lava Axe of P1's two.
    let bd = t.hand(P0, "Backdraft");
    mana(&mut t, P0);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast_with(P0, bd, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "half of 5, rounded down");
    let players: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(players.last(), Some(&vec![Entity::Player(P1)]));
    // Choosing Sign in Blood, which dealt no damage: none.
    let bd = t.hand(P0, "Backdraft");
    mana(&mut t, P0);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast_with(P0, bd, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn backdraft_does_nothing_without_a_sorcery_cast_this_turn() {
    cr!("608.2d");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1);
    t.cast_with(P1, bolt, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    let bd = t.hand(P0, "Backdraft");
    mana(&mut t, P0);
    t.cast_with(P0, bd, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn glyph_of_reincarnation_creatures_cant_be_regenerated() {
    cr!("701.19c");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let boa = t.battlefield(P1, "River Boa");
    ready(&mut t, boa);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[boa], P0, &[(wall, boa)]);
    t.advance_to(P1, Step::PostcombatMain);
    mana(&mut t, P1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve_all();
    let glyph = t.hand(P0, "Glyph of Reincarnation");
    mana(&mut t, P0);
    t.cast_with(P0, glyph, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    // It died (the shield doesn't apply) and came back as a new object, the only
    // creature card in P1's graveyard.
    assert_ne!(t.g.current(boa), boa, "the shield doesn't apply");
    assert!(t.on_battlefield(t.g.current(boa)));
}

/// Whether `text` compiles fully for a card named `name` with the type line `type_line`.
fn compiles(name: &str, type_line: &str, text: &str) -> bool {
    use mtg_engine::card::Layout;
    use mtg_engine::oracle::{self, CompileContext};
    let tl = mtg_engine::types::TypeLine::parse(type_line);
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    oracle::compile(text, &ctx).unsupported.is_empty()
}

#[test]
fn anchored_history_and_permission_wordings_compile() {
    for text in [
        "Destroy all creatures that were blocked by target Wall this turn. They can't be regenerated.",
        "Destroy all creatures blocked by target Wall this turn.",
        "Destroy all creatures that were blocked by target creature this turn.",
        "Destroy target creature that was blocked by target Wall this turn.",
        "Put X glyph counters on target creature blocked by target Wall this turn, where X is its power.",
        "Tap all creatures that blocked target creature this turn.",
        "Choose a player who cast one or more sorcery spells this turn. Backdraft deals 2 damage to that player.",
        "Choose an opponent who cast one or more sorcery spells this turn. Backdraft deals 2 damage to that player.",
        "Choose a player who cast one or more instant spells this turn. Backdraft deals damage to that player equal to the damage dealt by one of those instant spells this turn.",
    ] {
        assert!(compiles("Backdraft", "Instant", text), "{text}");
    }
    for text in [
        "At the beginning of your upkeep, choose target opponent. Until end of turn, you may play the top card of their library.",
        "At the beginning of your upkeep, choose target opponent. Until end of turn, you may look at the top card of their library any time.",
        "At the beginning of your upkeep, choose target opponent. Until end of turn, you may play the top card of that player's library.",
        "At the beginning of your upkeep, choose target opponent. Until end of turn, you may look at the top card of their library any time and you may play the top card of their library.",
        "At the beginning of your upkeep, choose target opponent. Until end of turn, you may cast spells from the top of their library.",
    ] {
        assert!(
            compiles("Xanathar, Guild Kingpin", "Legendary Creature — Beholder", text),
            "{text}"
        );
    }
    // A spending rider needs a permission it's about ("this way").
    assert!(!compiles(
        "Xanathar, Guild Kingpin",
        "Legendary Creature — Beholder",
        "At the beginning of your upkeep, choose target opponent. Until end of turn, you may look at the top card of their library any time and you may spend mana as though it were mana of any color to cast spells this way."
    ));
    // "Half" needs its rounding.
    assert!(!compiles(
        "Backdraft",
        "Instant",
        "Choose a player who cast one or more sorcery spells this turn. Backdraft deals damage to that player equal to half the damage dealt by one of those sorcery spells this turn."
    ));
}

#[test]
fn tap_all_creatures_that_blocked_target_creature_this_turn() {
    cr!("509.1", "115.1");
    use mtg_engine::card::{CardDef, Layout};
    use mtg_engine::object::Characteristics;
    use mtg_engine::oracle::{self, CompileContext};
    let text = "Tap all creatures that blocked target creature this turn.";
    let tl = mtg_engine::types::TypeLine::parse("Instant");
    let ctx = CompileContext {
        card_name: "Probe",
        full_name: "Probe",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(compiled.unsupported.is_empty());
    let def = CardDef::custom(Characteristics {
        name: "Probe".into(),
        mana_cost: mtg_engine::mana::ManaCost::parse("{U}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        ..Default::default()
    });
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    ready(&mut t, bears);
    ready(&mut t, giant);
    let wall = t.battlefield(P0, "Wall of Stone");
    let wood = t.battlefield(P0, "Wall of Wood");
    let idle = t.battlefield(P0, "Wall of Wood");
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[bears, giant], P0, &[(wall, bears), (wood, giant)]);
    t.advance_to(P1, Step::PostcombatMain);
    let probe = t.custom(P0, def, mtg_engine::object::Zone::Hand(P0));
    mana(&mut t, P0);
    t.cast_with(P0, probe, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    // Only the creature that blocked the target, after combat.
    assert!(t.obj_now(wall).tapped);
    assert!(!t.obj_now(wood).tapped);
    assert!(!t.obj_now(idle).tapped);
}
