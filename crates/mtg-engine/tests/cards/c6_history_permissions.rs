//! Turn history and permissions VII (cards6-history-permissions): permissions over the top
//! card of another player's library (Xanathar, Guild Kingpin), block history anchored on
//! a target (Glyph of Reincarnation, Glyph of Delusion), and damage dealt by one of the
//! sorcery spells a chosen player cast this turn (Backdraft).

use crate::combat2_common::supported;
use mtg_engine::decision::Action;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

fn sees(t: &TestGame, p: PlayerId, id: ObjectId) -> bool {
    mtg_engine::zones::can_see_in_library(&t.g, p, t.g.current(id))
}

/// An agent that records what `f` observes at each decision it's asked, then answers as
/// the test harness's scripted agent does.
struct Spy {
    inner: ScriptedAgent,
    f: Box<dyn Fn(&Game, &Decision) -> Option<bool> + Send>,
    log: Arc<Mutex<Vec<bool>>>,
}

impl Agent for Spy {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if let Some(x) = (self.f)(g, d) {
            self.log.lock().unwrap().push(x);
        }
        self.inner.decide(g, p, d)
    }
}

/// Replaces `p`'s agent with a [`Spy`]; answers queued on the test game still apply.
fn spy(
    t: &mut TestGame,
    p: PlayerId,
    f: impl Fn(&Game, &Decision) -> Option<bool> + Send + 'static,
) -> Arc<Mutex<Vec<bool>>> {
    let log = Arc::new(Mutex::new(vec![]));
    let agent = Spy {
        inner: ScriptedAgent {
            player: p,
            script: t.script.clone(),
        },
        f: Box::new(f),
        log: log.clone(),
    };
    t.g.set_agent(p, Box::new(agent));
    log
}

/// P0 controls Xanathar, Guild Kingpin; its upkeep ability, targeting P1, has resolved in
/// P0's upkeep (P1 drew the Island it puts on top of P1's library, in P1's turn before).
fn xanathar_upkeep(t: &mut TestGame) {
    t.battlefield(P0, "Xanathar, Guild Kingpin");
    t.library_top(P1, "Island");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn xanathar_plays_the_top_card_of_the_target_opponents_library() {
    cr!("401.5", "601.3", "305.2", "609.4b", "611.2a");
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
    let mountain = t.library_top(P1, "Mountain");
    let forest = t.library_top(P1, "Forest");
    let axe = t.library_top(P1, "Lava Axe");
    let bolt = t.library_top(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    assert!(!sees(&t, P0, bolt));
    xanathar_upkeep(&mut t);
    // P0 may look at the top card of P1's library (P1 itself may not); P1 can't cast
    // spells.
    assert!(sees(&t, P0, bolt));
    assert!(!sees(&t, P1, bolt));
    t.g.players[1].mana_pool.add_type(ManaType::R, 1);
    assert!(t.cast_with(P1, shock, &[Entity::Player(P0)]).is_err());
    t.clear_answers();
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
    t.clear_answers();
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
    // Until end of turn only: P0 can't look at or cast P1's top card any more, and P1 can
    // cast spells again.
    t.advance_to(P1, Step::Upkeep);
    assert!(!sees(&t, P0, mountain));
    let late = t.library_top(P1, "Lightning Bolt");
    assert!(!sees(&t, P0, late));
    t.g.players[0].mana_pool.add_type(ManaType::R, 1);
    assert!(t.cast_with(P0, late, &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    t.g.players[1].mana_pool.add_type(ManaType::R, 1);
    t.cast_with(P1, shock, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn xanathar_does_nothing_if_the_opponent_has_become_an_illegal_target() {
    cr!("608.2b", "702.18a");
    ruling!(
        "Xanathar, Guild Kingpin",
        "The target opponent may still cast spells in response"
    );
    ruling!(
        "Xanathar, Guild Kingpin",
        "If that player has somehow become an illegal target for the ability as it tries to resolve, the ability does nothing."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Xanathar, Guild Kingpin");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.library_top(P1, "Island");
    let light = t.hand(P1, "Gilded Light");
    let shock = t.hand(P1, "Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1, "Xanathar's ability, targeting P1");
    // In response, P1 casts a spell: they gain shroud.
    t.g.players[1].mana_pool.add_type(ManaType::W, 2);
    t.cast_with(P1, light, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    // The ability did nothing: P0 can't look at or play P1's top card, and P1 may cast
    // spells.
    assert!(!sees(&t, P0, bolt));
    t.g.players[0].mana_pool.add_type(ManaType::R, 1);
    assert!(t.cast_with(P0, bolt, &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    t.g.players[0].mana_pool = Default::default();
    t.g.players[1].mana_pool.add_type(ManaType::R, 1);
    t.cast_with(P1, shock, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn xanathar_cant_look_at_the_new_top_card_until_the_land_or_spell_is_finished() {
    cr!("401.5", "116.2a", "601.2i");
    ruling!(
        "Xanathar, Guild Kingpin",
        "you can't look at the new top card until you finish doing so"
    );
    let mut t = TestGame::new(2);
    let giant = t.library_top(P1, "Hill Giant");
    let bolt = t.library_top(P1, "Lightning Bolt");
    let vents = t.library_top(P1, "Steam Vents");
    xanathar_upkeep(&mut t);
    assert!(sees(&t, P0, vents));
    assert!(!sees(&t, P0, bolt));
    t.advance_to(P0, Step::PrecombatMain);
    // "As this land enters, you may pay 2 life" is asked while the land is being played:
    // the card below it can't be looked at yet.
    let log = spy(&mut t, P0, move |g, d| {
        (!matches!(d, Decision::Priority { .. }))
            .then(|| mtg_engine::zones::can_see_in_library(g, P0, bolt))
    });
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::PlayLand { card: vents })
        .unwrap();
    let seen = log.lock().unwrap().clone();
    assert!(!seen.is_empty() && seen.iter().all(|x| !x), "{seen:?}");
    assert!(t.on_battlefield(t.g.current(vents)));
    assert!(sees(&t, P0, bolt), "once the land is played");
    // Casting the Bolt from the top of P1's library: while its target is chosen, the
    // card below can't be looked at.
    let log = spy(&mut t, P0, move |g, d| {
        matches!(d, Decision::ChooseTargets { .. })
            .then(|| mtg_engine::zones::can_see_in_library(g, P0, giant))
    });
    t.g.players[0].mana_pool.add_type(ManaType::R, 1);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    assert_eq!(*log.lock().unwrap(), vec![false]);
    assert!(sees(&t, P0, giant), "once the spell is cast");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn xanathar_spells_cast_this_way_have_their_additional_and_alternative_costs() {
    cr!("118.8", "118.9", "601.2f");
    ruling!(
        "Xanathar, Guild Kingpin",
        "You'll still pay all costs for spells cast this way, including additional costs. You may also pay alternative costs if any are available."
    );
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P0, "Grizzly Bears");
    let mountains = t.lands(P0, "Mountain", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    let fireblast = t.library_top(P1, "Fireblast");
    let splinters = t.library_top(P1, "Bone Splinters");
    xanathar_upkeep(&mut t);
    t.advance_to(P0, Step::PrecombatMain);
    // Bone Splinters: its additional cost, sacrificing a creature, is paid.
    t.g.players[0].mana_pool.add_type(ManaType::B, 1);
    t.answer_choose(P0, &[Entity::Object(victim)]);
    t.cast_with(P0, splinters, &[Entity::Object(giant)])
        .unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "sacrificed as it was cast");
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // Fireblast, now on top: cast for its alternative cost, sacrificing two Mountains.
    assert!(sees(&t, P0, fireblast));
    let alt = t
        .g
        .cast_options(P0, fireblast)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("its alternative cost is offered");
    t.cast(P0, fireblast)
        .method(alt)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(mountains.iter().all(|m| !t.on_battlefield(*m)));
    assert_eq!(t.g.player(P0).graveyard.len(), 3, "the Bears and both Mountains");
}

/// A card compiled from `text`, which must compile completely.
fn compiled_card(name: &str, type_line: &str, cost: &str, text: &str) -> CardDef {
    let mut def = crate::basic_effects_common::oracle_card(name, type_line, text);
    def.faces[0].chars.mana_cost = mtg_engine::mana::ManaCost::parse(cost);
    def
}

#[test]
fn a_permission_over_each_opponents_library_top_card() {
    cr!("601.3");
    // "Each opponent's library": the top card of any opponent's library, not of your own.
    let mut t = TestGame::new(3);
    let lens = compiled_card(
        "Probe Lens",
        "Enchantment",
        "{2}",
        "At the beginning of your upkeep, until end of turn, you may play the top card of each opponent's library.",
    );
    t.custom(P0, lens, mtg_engine::object::Zone::Battlefield);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let forest = t.library_top(P2, "Forest");
    // (P1 and P2 draw these Islands in their turns before P0's.)
    t.library_top(P1, "Island");
    t.library_top(P2, "Island");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    let own = t.library_top(P0, "Lightning Bolt");
    t.g.players[0].mana_pool.add_type(ManaType::R, 2);
    assert!(
        t.cast_with(P0, own, &[Entity::Player(P1)]).is_err(),
        "not P0's own library"
    );
    t.clear_answers();
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.play_land(P0, forest).unwrap();
    let f = t.g.current(forest);
    assert!(t.on_battlefield(f));
    assert_eq!(t.obj_now(f).controller, P0);
    // The permission doesn't let P0 look at those cards.
    let next = t.library_top(P2, "Hill Giant");
    assert!(!sees(&t, P0, next));
}

#[test]
fn a_permission_to_cast_spells_from_the_top_of_their_library_doesnt_play_lands() {
    cr!("601.3", "305.1");
    let mut t = TestGame::new(2);
    let probe = compiled_card(
        "Probe Eye",
        "Enchantment",
        "{2}",
        "At the beginning of your upkeep, choose target opponent. Until end of turn, you may cast spells from the top of their library.",
    );
    t.custom(P0, probe, mtg_engine::object::Zone::Battlefield);
    let forest = t.library_top(P1, "Forest");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.library_top(P1, "Island");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    t.g.players[0].mana_pool.add_type(ManaType::R, 1);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.play_land(P0, forest).is_err(), "only spells");
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
fn glyph_of_reincarnation_uses_the_controller_at_the_block_not_the_owner() {
    cr!("509.1", "400.3");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.graveyard(P1, "Craw Wurm");
    let giant = t.graveyard(P0, "Hill Giant");
    // P1 takes P0's Bears and attacks with them; P0's Wall blocks them.
    t.advance_to(P1, Step::PrecombatMain);
    let threaten = t.hand(P1, "Threaten");
    mana(&mut t, P1);
    t.cast_with(P1, threaten, &[Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[bears], P0, &[(wall, bears)]);
    t.advance_to(P1, Step::PostcombatMain);
    let glyph = t.hand(P0, "Glyph of Reincarnation");
    mana(&mut t, P0);
    t.cast_with(P0, glyph, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    // The Bears died into their owner's graveyard, P0's; the card came from the
    // graveyard of P1, who controlled them when the Wall blocked them.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let w = t.g.current(wurm);
    assert!(t.on_battlefield(w));
    assert_eq!(t.obj_now(w).controller, P1);
    assert_eq!(t.g.current(giant), giant, "P0's graveyard isn't that one");
}

/// P1 attacks with Benalish Hero and Grizzly Bears in a band, and P0's Wall of Stone
/// blocks the Hero: the Bears become blocked by the Wall too (CR 702.22h). The game stops
/// in the declare blockers step.
fn band_blocked_by_a_wall(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let wall = t.battlefield(P0, "Wall of Stone");
    let hero = t.battlefield(P1, "Benalish Hero");
    let bears = t.battlefield(P1, "Grizzly Bears");
    ready(t, hero);
    ready(t, bears);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    attack_and_block(t, P1, &[hero, bears], P0, &[(wall, hero)]);
    let blockers =
        t.g.combat
            .as_ref()
            .map(|c| c.blockers_of(bears))
            .unwrap_or_default();
    assert_eq!(blockers, vec![wall], "the band was blocked");
    (wall, hero, bears)
}

#[test]
fn glyph_of_reincarnation_puts_the_cards_for_a_blocked_band_onto_the_battlefield_together() {
    cr!("702.22h", "608.2f", "614.12a");
    ruling!(
        "Glyph of Reincarnation",
        "First place all the affected creatures in the graveyard, then choose the appropriate number of creatures from the attacker’s graveyard and put them onto the battlefield."
    );
    ruling!(
        "Clone",
        "If Clone somehow enters at the same time as another creature, Clone can't become a copy of that creature."
    );
    let mut t = TestGame::new(2);
    let clone = t.graveyard(P1, "Clone");
    let (wall, _hero, bears) = band_blocked_by_a_wall(&mut t);
    t.advance_to(P1, Step::PostcombatMain);
    let glyph = t.hand(P0, "Glyph of Reincarnation");
    mana(&mut t, P0);
    // Both creatures of the band are destroyed; then a card is chosen for each (the
    // Bears just destroyed, then Clone) and both are put onto the battlefield at once.
    t.answer_choose_moved(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(clone)]);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(wall)]);
    let from = t.asked().len();
    t.cast_with(P0, glyph, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    let b = t.g.current(bears);
    let c = t.g.current(clone);
    assert!(t.on_battlefield(b) && t.on_battlefield(c));
    assert_eq!(t.obj_now(b).controller, P1);
    assert_eq!(t.obj_now(c).controller, P1);
    assert!(t.in_graveyard(P1, "Benalish Hero"));
    let asked = t.asked()[from..].to_vec();
    let offered = |who: PlayerId| -> Vec<Vec<Entity>> {
        asked
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseEntities { candidates, .. } if *p == who => {
                    Some(candidates.clone())
                }
                _ => None,
            })
            .collect()
    };
    // The second card was chosen from the cards not chosen yet, before anything moved.
    let bears_card = t.g.obj(b).prev.expect("the Bears' card in the graveyard");
    let chosen = offered(P0);
    assert_eq!(chosen.len(), 2);
    assert!(chosen[0].contains(&Entity::Object(bears_card)));
    assert!(chosen[1].contains(&Entity::Object(clone)));
    assert!(!chosen[1].contains(&Entity::Object(bears_card)));
    // The Clone entered with the Bears: it could copy the Wall, not them.
    let copies = offered(P1);
    assert!(copies.iter().any(|v| v.contains(&Entity::Object(wall))), "{copies:?}");
    assert!(copies.iter().all(|v| !v.contains(&Entity::Object(b))), "{copies:?}");
    assert_eq!(t.obj_now(c).chars.name.as_str(), "Wall of Stone");
}

#[test]
fn glyph_of_delusion_targets_a_creature_blocked_by_the_wall_as_part_of_a_band() {
    cr!("702.22h", "115.1");
    ruling!(
        "Glyph of Delusion",
        "or that was blocked by the wall due to being in a band"
    );
    let mut t = TestGame::new(2);
    let (wall, _hero, bears) = band_blocked_by_a_wall(&mut t);
    // The Wall was declared blocking the Hero; the Bears, in its band, were blocked by it.
    let glyph = t.hand(P0, "Glyph of Delusion");
    mana(&mut t, P0);
    t.cast_with(P0, glyph, &[Entity::Object(wall), Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "glyph"), 2);
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
    // The Wall is chosen first; then only the creature it blocked can be, announced with
    // the card's words.
    assert_eq!(
        target_candidates(&t, "target creature that target wall blocked this turn"),
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
fn glyph_of_delusion_still_affects_the_creature_once_the_wall_has_left() {
    cr!("608.2b");
    interp!("anchor-target-chosen-first");
    interp!("dependent-target-checked-against-an-illegal-target");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    ready(&mut t, bears);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[bears], P0, &[(wall, bears)]);
    let glyph = t.hand(P0, "Glyph of Delusion");
    mana(&mut t, P0);
    t.cast_with(P0, glyph, &[Entity::Object(wall), Entity::Object(bears)])
        .unwrap();
    // In response the Wall is returned to its owner's hand: that target is illegal, but
    // the Bears are still a creature the Wall (as chosen) blocked this turn.
    let unsummon = t.hand(P1, "Unsummon");
    mana(&mut t, P1);
    t.cast_with(P1, unsummon, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Wall of Stone"));
    assert_eq!(t.counters(bears, "glyph"), 2);
}

#[test]
fn a_wall_put_onto_the_battlefield_blocking_never_blocked() {
    cr!("509.4");
    interp!("entered-blocker-never-blocked");
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Aetherplasm");
    let wall_card = t.hand(P0, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    ready(&mut t, bears);
    t.advance_to(P1, Step::BeginningOfCombat);
    // Aetherplasm blocks the Bears, returns to P0's hand and puts Wall of Stone onto the
    // battlefield blocking them.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(wall_card)]);
    attack_and_block(&mut t, P1, &[bears], P0, &[(plasm, bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Aetherplasm"));
    let wall = t.named_on_battlefield("Wall of Stone");
    assert_eq!(wall.len(), 1);
    let wall = wall[0];
    let blockers =
        t.g.combat
            .as_ref()
            .map(|c| c.blockers_of(bears))
            .unwrap_or_default();
    assert_eq!(blockers, vec![wall], "the Wall is blocking the Bears");
    // But it never blocked them: Glyph of Delusion has no creature to target with it ...
    let delusion = t.hand(P0, "Glyph of Delusion");
    mana(&mut t, P0);
    assert!(t
        .cast_with(P0, delusion, &[Entity::Object(wall), Entity::Object(bears)])
        .is_err());
    t.clear_answers();
    // ... and Glyph of Reincarnation targeting it destroys nothing.
    t.advance_to(P1, Step::PostcombatMain);
    let glyph = t.hand(P0, "Glyph of Reincarnation");
    mana(&mut t, P0);
    t.cast_with(P0, glyph, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(wall), "a legal target as it resolved");
}

#[test]
fn glyph_of_delusion_targets_creatures_switched_into_or_out_of_the_walls_block() {
    cr!("509.1", "506.4", "115.1");
    // The third case of the ruling: a block an ability switches (Sorrow's Path) counts, both
    // for the attacker switched into the Wall's block and the one switched out of it. This
    // is the case its "switched into or out of the block by an ability" is about, not a
    // creature put onto the battlefield blocking, which never "blocked" (CR 509.4).
    ruling!(
        "Glyph of Delusion",
        "or by being switched into or out of the block by an ability"
    );
    interp!("entered-blocker-never-blocked");
    supported("Sorrow's Path");
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    let stone = t.battlefield(P1, "Wall of Stone");
    let wood = t.battlefield(P1, "Wall of Wood");
    ready(&mut t, wurm);
    ready(&mut t, giant);
    // Wall of Stone blocks the Craw Wurm, Wall of Wood the Hill Giant; Sorrow's Path then
    // switches them (and deals 2 damage to P0 and each creature P0 controls).
    attack_and_block(&mut t, P0, &[wurm, giant], P1, &[(stone, wurm), (wood, giant)]);
    t.answer_targets(P0, &[Entity::Object(stone), Entity::Object(wood)]);
    t.activate(P0, path, 0, &[]).unwrap();
    t.resolve_all();
    let blocking = |t: &TestGame, b: ObjectId| {
        t.g.combat
            .as_ref()
            .map(|c| c.blocking(b))
            .unwrap_or_default()
    };
    assert_eq!(blocking(&t, stone), vec![giant]);
    assert_eq!(blocking(&t, wood), vec![wurm]);
    // Wall of Stone blocked both this turn: the Giant switched into its block, the Wurm
    // switched out of it.
    let glyph = t.hand(P1, "Glyph of Delusion");
    mana(&mut t, P1);
    t.cast_with(P1, glyph, &[Entity::Object(stone), Entity::Object(giant)])
        .unwrap();
    let mut offered =
        target_candidates(&t, "target creature that target wall blocked this turn").unwrap();
    offered.sort_by_key(|e| format!("{e:?}"));
    let mut both = vec![Entity::Object(wurm), Entity::Object(giant)];
    both.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(offered, both);
    t.resolve_all();
    assert_eq!(t.counters(giant, "glyph"), 3, "X is the Hill Giant's power");
}

#[test]
fn glyph_of_delusion_needs_a_creature_that_wall_blocked() {
    cr!("115.1", "601.2c");
    interp!("anchor-target-chosen-first");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let other = t.battlefield(P0, "Wall of Wood");
    let bears = t.battlefield(P1, "Grizzly Bears");
    ready(&mut t, bears);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[bears], P0, &[(other, bears)]);
    t.advance_to(P1, Step::PostcombatMain);
    // Wall of Stone blocked nothing: no creature to target with it, so it isn't offered
    // as the Wall (the targets are chosen as a set, CR 601.2c); Wall of Wood is, with the
    // Bears it blocked.
    let glyph = t.hand(P0, "Glyph of Delusion");
    mana(&mut t, P0);
    let from = t.asked().len();
    t.cast_with(P0, glyph, &[Entity::Object(other), Entity::Object(bears)])
        .unwrap();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 2, "{offered:?}");
    assert_eq!(offered[0], vec![Entity::Object(other)]);
    assert_eq!(offered[1], vec![Entity::Object(bears)]);
    assert!(!offered.iter().flatten().any(|e| *e == Entity::Object(wall)));
    t.resolve_all();
    assert_eq!(t.counters(bears, "glyph"), 2);
    // No Wall blocked anything this turn: the spell has no legal targets and can't be cast.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    ready(&mut t, bears);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_and_block(&mut t, P1, &[bears], P0, &[]);
    t.advance_to(P1, Step::PostcombatMain);
    let glyph = t.hand(P0, "Glyph of Delusion");
    mana(&mut t, P0);
    assert!(t
        .cast_with(P0, glyph, &[Entity::Object(wall), Entity::Object(bears)])
        .is_err());
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
fn backdraft_counts_all_the_damage_the_sorcery_dealt() {
    cr!("120.1", "107.1a");
    interp!("backdraft-controller-chooses-the-spell");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    let rift = t.hand(P1, "Flame Rift");
    mana(&mut t, P1);
    t.cast_with(P1, rift, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (16, 16));
    let bd = t.hand(P0, "Backdraft");
    mana(&mut t, P0);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.cast_with(P0, bd, &[]).unwrap();
    t.resolve_all();
    // Half of the 8 damage Flame Rift dealt to the two players.
    assert_eq!((t.life(P0), t.life(P1)), (16, 12));
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
