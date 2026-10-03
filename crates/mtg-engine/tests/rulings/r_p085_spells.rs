//! Rulings batch P085 — hosers of instants and nonbasic lands: Teferi, Mage of Zhalfir's
//! sorcery-speed restriction, Dryad Militant, Goblin Flectomancer, Archon of Valor's
//! Reach, Niv-Mizzet, Supreme, Ravnica at War, Archon of Emeria, Thalia, Heretic Cathar,
//! Alpine Moon, Burning Earth, and Mercadia's Downfall.

use crate::r_p085_common::*;
use mtg_engine::ability::LibraryPosition;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn enter_together(t: &mut TestGame, cards: &[(&str, PlayerId)]) -> Vec<ObjectId> {
    let moves: Vec<MoveEv> = cards
        .iter()
        .map(|(n, p)| MoveEv {
            obj: t.g.create_card_object(card(n), *p, Zone::Nowhere),
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: events::MoveCause::Effect,
            by: Some(*p),
            etb: EtbInfo {
                controller: Some(*p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let ids =
        t.g.move_objects(moves)
            .into_iter()
            .map(|x| x.unwrap())
            .collect();
    t.g.flush_events();
    ids
}

/// The objects on the stack, bottom first.
fn stack(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack.clone()
}

/// The chosen targets of a spell or ability on the stack.
fn stack_targets(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect()
}

// ---------------------------------------------------------------------------------------
// Teferi, Mage of Zhalfir
// ---------------------------------------------------------------------------------------

#[test]
fn teferi_opponents_only_at_sorcery_speed() {
    cr!("307.1", "117.1a");
    ruling!(
        "Teferi, Mage of Zhalfir",
        "Teferi's last ability means that in order for an opponent to cast a spell, it must be that opponent's turn, during a main phase, and the stack must be empty."
    );
    ruling!(
        "Teferi, Mage of Zhalfir",
        "If an effect allows an opponent to cast spells any time they could cast an instant (for example, if your opponent also controls Teferi), the restriction of Teferi's last ability takes precedence over that permission."
    );
    supported("Teferi, Mage of Zhalfir");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi, Mage of Zhalfir");
    t.battlefield(P1, "Teferi, Mage of Zhalfir");
    // P0's turn: P1 can't cast an instant, nor a creature its own Teferi gives flash.
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bears = t.hand(P1, "Grizzly Bears");
    pool(&mut t, P1, &[(ManaType::R, 1), (ManaType::G, 2)]);
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    assert!(t.cast(P1, bears).try_go().is_err());
    // P1, in its own main phase with an empty stack, can cast.
    t.set_step(P1, Step::PrecombatMain);
    t.clear_answers();
    let bolt = t.g.current(bolt);
    pool(&mut t, P1, &[(ManaType::R, 1)]);
    t.cast(P1, bolt).target(P0).go();
    // ... but not while that spell is on the stack.
    let bears = t.g.current(bears);
    pool(&mut t, P1, &[(ManaType::G, 2)]);
    assert!(t.cast(P1, bears).try_go().is_err());
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // P1's upkeep: no.
    t.set_step(P1, Step::Upkeep);
    let bears = t.g.current(bears);
    assert!(t.cast(P1, bears).try_go().is_err());
}

#[test]
fn teferi_split_second_stops_suspending() {
    cr!("702.62a", "702.61a", "116.2f");
    ruling!(
        "Teferi, Mage of Zhalfir",
        "If a spell on the stack has split second while you control Teferi, you can't suspend creature cards"
    );
    supported("Durkwood Baloth");
    supported("Sudden Shock");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi, Mage of Zhalfir");
    let baloth = t.hand(P0, "Durkwood Baloth");
    t.set_step(P1, Step::PrecombatMain);
    let shock = t.hand(P1, "Sudden Shock");
    pool(&mut t, P1, &[(ManaType::R, 2)]);
    t.cast(P1, shock).target(P0).go();
    pool(&mut t, P0, &[(ManaType::G, 1)]);
    let suspend = Action::Special(SpecialAction::Suspend { card: baloth });
    t.g.turn.priority = Some(P0);
    assert!(t.g.perform_action(P0, suspend.clone()).is_err());
    assert!(t.in_hand(P0, "Durkwood Baloth"));
    t.resolve_all();
    // Without split second, on P1's turn, it can be suspended (it has flash).
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, suspend).unwrap();
    assert_eq!(t.zone(baloth), Zone::Exile);
}

#[test]
fn teferi_stops_casting_during_resolution() {
    cr!("702.88a", "608.2g");
    ruling!(
        "Teferi, Mage of Zhalfir",
        "If a spell or ability lets an opponent cast a spell as part of its effect (such as suspend and rebound do), that opponent can't cast that spell since the resolving ability is still on the stack."
    );
    supported("Distortion Strike");
    for teferi in [false, true] {
        let mut t = TestGame::new(2);
        if teferi {
            t.battlefield(P0, "Teferi, Mage of Zhalfir");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P1, Step::PrecombatMain);
        let strike = t.hand(P1, "Distortion Strike");
        pool(&mut t, P1, &[(ManaType::U, 1)]);
        t.cast(P1, strike).target(bears).go();
        t.resolve_all();
        assert_eq!(t.zone(strike), Zone::Exile);
        // P1's next upkeep: the rebound trigger offers to cast it.
        t.answer_yes(P1, true);
        t.answer_targets(P1, &[Entity::Object(bears)]);
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        assert_eq!(
            t.pt(bears),
            if teferi { (2, 2) } else { (3, 2) },
            "teferi={teferi}"
        );
        assert_eq!(
            t.zone(strike),
            if teferi {
                Zone::Exile
            } else {
                Zone::Graveyard(P1)
            }
        );
    }
}

// ---------------------------------------------------------------------------------------
// Dryad Militant
// ---------------------------------------------------------------------------------------

#[test]
fn dryad_militant_discards_and_madness() {
    cr!("614.1a", "702.35a");
    ruling!(
        "Dryad Militant",
        "If an instant or sorcery card is discarded while Dryad Militant is on the battlefield, abilities that function when a card is discarded (such as madness) still work"
    );
    supported("Dryad Militant");
    supported("Fiery Temper");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Dryad Militant");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.g.discard(P0, bolt, None);
    t.g.flush_events();
    assert_eq!(t.zone(bolt), Zone::Exile);
    // Madness: Fiery Temper is exiled and can be cast for its madness cost.
    let temper = t.hand(P0, "Fiery Temper");
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.discard(P0, temper, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Then the spell goes to exile instead of the graveyard.
    assert_eq!(t.zone(temper), Zone::Exile);
}

#[test]
fn dryad_militant_murder_vs_lethal_damage() {
    cr!("704.3", "608.2n", "614.1a");
    ruling!(
        "Dryad Militant",
        "If an instant or sorcery spell destroys Dryad Militant directly (like Murder does), that instant or sorcery card will be put into its owner's graveyard."
    );
    let mut t = TestGame::new(2);
    let militant = t.battlefield(P1, "Dryad Militant");
    let murder = t.hand(P0, "Murder");
    pool(&mut t, P0, &[(ManaType::B, 3)]);
    t.cast(P0, murder).target(militant).go();
    t.resolve_all();
    assert_eq!(t.zone(murder), Zone::Graveyard(P0));
    let mut t = TestGame::new(2);
    let militant = t.battlefield(P1, "Dryad Militant");
    let bolt = t.hand(P0, "Lightning Bolt");
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    t.cast(P0, bolt).target(militant).go();
    t.resolve_all();
    assert!(!t.on_battlefield(militant));
    assert_eq!(t.zone(bolt), Zone::Exile);
}

// ---------------------------------------------------------------------------------------
// Goblin Flectomancer
// ---------------------------------------------------------------------------------------

#[test]
fn goblin_flectomancer_changes_all_or_none() {
    cr!("115.7a", "115.7e");
    ruling!(
        "Goblin Flectomancer",
        "If the spell has multiple targets, you may either change all the targets or none of them."
    );
    ruling!(
        "Goblin Flectomancer",
        "The ability can target any instant or sorcery spell, even if it has no targets."
    );
    supported("Goblin Flectomancer");
    supported("Seeds of Strength");
    supported("Prey Upon");
    // Seeds of Strength targeting X, X, Y: changed to Y, Z, X.
    let mut t = TestGame::new(2);
    let flecto = t.battlefield(P0, "Goblin Flectomancer");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Hill Giant");
    let z = t.battlefield(P1, "Llanowar Elves");
    let seeds = t.hand(P1, "Seeds of Strength");
    pool(&mut t, P1, &[(ManaType::G, 1), (ManaType::W, 1)]);
    let seeds = t.cast(P1, seeds).target(x).target(x).target(y).go();
    t.answer_targets(P0, &[Entity::Object(seeds)]);
    t.answer_yes(P0, true);
    for e in [y, z, x] {
        t.answer_targets(P0, &[Entity::Object(e)]);
    }
    t.activate(P0, flecto, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(
        stack_targets(&t, seeds),
        vec![Entity::Object(y), Entity::Object(z), Entity::Object(x)]
    );
    // Prey Upon: P1's only creature can't be changed, so neither target is.
    let mut t = TestGame::new(2);
    let flecto = t.battlefield(P0, "Goblin Flectomancer");
    let mine = t.battlefield(P1, "Hill Giant");
    let a = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.set_step(P1, Step::PrecombatMain);
    let prey = t.hand(P1, "Prey Upon");
    pool(&mut t, P1, &[(ManaType::G, 1)]);
    let prey = t.cast(P1, prey).target(mine).target(a).go();
    t.answer_targets(P0, &[Entity::Object(prey)]);
    t.answer_yes(P0, true);
    t.activate(P0, flecto, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(
        stack_targets(&t, prey),
        vec![Entity::Object(mine), Entity::Object(a)]
    );
    // A spell with no targets can be targeted.
    let mut t = TestGame::new(2);
    let flecto = t.battlefield(P0, "Goblin Flectomancer");
    t.set_step(P1, Step::PrecombatMain);
    let div = t.hand(P1, "Divination");
    pool(&mut t, P1, &[(ManaType::U, 3)]);
    let div = t.cast(P1, div).go();
    t.answer_targets(P0, &[Entity::Object(div)]);
    let ab = t.activate(P0, flecto, 0, &[]).unwrap().unwrap();
    assert_eq!(stack_targets(&t, ab), vec![Entity::Object(div)]);
}

// ---------------------------------------------------------------------------------------
// Archon of Valor's Reach
// ---------------------------------------------------------------------------------------

/// P0's Archon of Valor's Reach enters naming sorcery.
fn archon_sorcery(t: &mut TestGame) -> ObjectId {
    supported("Archon of Valor's Reach");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.enter(P0, "Archon of Valor's Reach")
}

#[test]
fn archon_of_valors_reach_choice_is_part_of_entering() {
    cr!("614.12", "614.1c");
    ruling!(
        "Archon of Valor's Reach",
        "The enters-the-battlefield effect of Archon of Valor’s Reach is a replacement effect that doesn’t use the stack."
    );
    let mut t = TestGame::new(2);
    archon_sorcery(&mut t);
    // Nothing went on the stack; sorceries can't be cast right away.
    assert_eq!(t.stack_len(), 0);
    let div = t.hand(P0, "Divination");
    pool(&mut t, P0, &[(ManaType::U, 3)]);
    assert!(t.cast(P0, div).try_go().is_err());
    let bolt = t.hand(P0, "Lightning Bolt");
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    t.cast(P0, bolt).target(P1).go();
}

#[test]
fn archon_of_valors_reach_checked_before_costs() {
    cr!("601.2", "601.3");
    ruling!(
        "Archon of Valor's Reach",
        "Whether it’s legal to cast a spell is determined before its costs are paid."
    );
    supported("Morbid Curiosity");
    let mut t = TestGame::new(2);
    let archon = archon_sorcery(&mut t);
    let mc = t.hand(P0, "Morbid Curiosity");
    pool(&mut t, P0, &[(ManaType::B, 3)]);
    t.answer_choose(P0, &[Entity::Object(archon)]);
    assert!(t.cast(P0, mc).try_go().is_err());
    assert!(t.on_battlefield(archon));
    assert!(t.in_hand(P0, "Morbid Curiosity"));
}

// ---------------------------------------------------------------------------------------
// Niv-Mizzet, Supreme
// ---------------------------------------------------------------------------------------

#[test]
fn niv_mizzet_supreme_jump_start() {
    cr!("702.133a", "117.3b", "307.1");
    ruling!(
        "Niv-Mizzet, Supreme",
        "If an instant or sorcery card is put into your graveyard during your turn, you'll be able to cast it right away if it's legal to do so"
    );
    ruling!(
        "Niv-Mizzet, Supreme",
        "You must still follow any timing restrictions and permissions, including those based on the card's type."
    );
    supported("Niv-Mizzet, Supreme");
    supported("Electrolyze");
    supported("Blightning");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niv-Mizzet, Supreme");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    let e = t.hand(P0, "Electrolyze");
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::R, 2)]);
    t.cast(P0, e).targets(&[Entity::Player(P1)]).go();
    t.resolve();
    let e = t.g.current(e);
    assert_eq!(t.g.obj(e).zone, Zone::Graveyard(P0));
    // P0 gets priority first and casts it again with jump-start.
    assert_eq!(t.g.turn.priority, Some(P0));
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::R, 2)]);
    t.cast(P0, e)
        .method(CastMethod::Keyword(KeywordKind::JumpStart))
        .targets(&[Entity::Player(P1)])
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // A sorcery (Blightning) only at sorcery speed.
    let b = t.graveyard(P0, "Blightning");
    t.set_step(P1, Step::PrecombatMain);
    pool(&mut t, P0, &[(ManaType::B, 1), (ManaType::R, 2)]);
    t.hand(P0, "Island");
    assert!(t
        .cast(P0, b)
        .method(CastMethod::Keyword(KeywordKind::JumpStart))
        .target(P1)
        .try_go()
        .is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.clear_answers();
    t.cast(P0, b)
        .method(CastMethod::Keyword(KeywordKind::JumpStart))
        .target(P1)
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
}

#[test]
fn ravnica_at_war_lands_are_colorless() {
    cr!("105.2c", "305.2");
    ruling!(
        "Ravnica at War",
        "A land normally has no color, even if it can produce multiple colors of mana."
    );
    supported("Ravnica at War");
    let mut t = TestGame::new(2);
    let chancery = t.battlefield(P1, "Azorius Chancery");
    let wolf = t.battlefield(P1, "Watchwolf");
    let spell = t.hand(P0, "Ravnica at War");
    pool(&mut t, P0, &[(ManaType::W, 4)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(chancery));
    assert_eq!(t.zone(wolf), Zone::Exile);
}

// ---------------------------------------------------------------------------------------
// Archon of Emeria
// ---------------------------------------------------------------------------------------

#[test]
fn archon_of_emeria_whole_turn() {
    cr!("101.2", "601.2");
    ruling!(
        "Archon of Emeria",
        "Archon of Emeria looks at the entire turn to see if a player has cast a spell"
    );
    ruling!(
        "Archon of Emeria",
        "If you cast a spell that was countered, you can't cast another spell during the same turn."
    );
    supported("Archon of Emeria");
    // Casting the Archon itself counts.
    let mut t = TestGame::new(2);
    let archon = t.hand(P0, "Archon of Emeria");
    pool(&mut t, P0, &[(ManaType::W, 3)]);
    t.cast(P0, archon).go();
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    // P1 cast a spell before the Archon arrived.
    let mut t2 = TestGame::new(2);
    let b1 = t2.hand(P1, "Lightning Bolt");
    pool(&mut t2, P1, &[(ManaType::R, 1)]);
    t2.cast(P1, b1).target(P0).go();
    t2.resolve_all();
    t2.enter(P0, "Archon of Emeria");
    let b2 = t2.hand(P1, "Lightning Bolt");
    pool(&mut t2, P1, &[(ManaType::R, 1)]);
    assert!(t2.cast(P1, b2).target(P0).try_go().is_err());
    // A countered spell counts.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archon of Emeria");
    let b1 = t.hand(P0, "Lightning Bolt");
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    let b1 = t.cast(P0, b1).target(P1).go();
    let cs = t.hand(P1, "Counterspell");
    pool(&mut t, P1, &[(ManaType::U, 2)]);
    t.cast(P1, cs).target(b1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    let b2 = t.hand(P0, "Lightning Bolt");
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    assert!(t.cast(P0, b2).target(P1).try_go().is_err());
}

/// P1 plays `land` in its main phase (with `in_hand` cards in hand and `controls` lands
/// on the battlefield) while P0 controls `hoser`. Whether it entered tapped.
fn enters_tapped(
    hoser: Option<&str>,
    land: &str,
    in_hand: &[&str],
    controls: &[&str],
    answers: impl FnOnce(&mut TestGame),
) -> bool {
    let mut t = TestGame::new(2);
    if let Some(h) = hoser {
        t.battlefield(P0, h);
    }
    for c in controls {
        t.battlefield(P1, c);
    }
    for c in in_hand {
        t.hand(P1, c);
    }
    t.set_step(P1, Step::PrecombatMain);
    let l = t.hand(P1, land);
    answers(&mut t);
    t.play_land(P1, l).unwrap();
    t.resolve_all();
    t.obj_now(l).tapped
}

#[test]
fn enters_tapped_unless_still_tapped() {
    cr!("614.1c", "614.12");
    ruling!(
        "Archon of Emeria",
        "If an effect says that a land enters tapped unless a condition is met or a cost is paid, that land enters tapped even if that condition is met or that cost is paid"
    );
    ruling!(
        "Thalia, Heretic Cathar",
        "If an effect states that a creature or land enters the battlefield tapped unless a condition is met, Thalia’s last ability has it enter tapped even if that condition is true."
    );
    supported("Thalia, Heretic Cathar");
    for hoser in [
        None,
        Some("Archon of Emeria"),
        Some("Thalia, Heretic Cathar"),
    ] {
        let tapped = enters_tapped(hoser, "Glacial Fortress", &[], &["Island"], |_| {});
        assert_eq!(tapped, hoser.is_some(), "{hoser:?} Glacial Fortress");
        let tapped = enters_tapped(hoser, "Port Town", &["Plains"], &[], |t| {
            t.answer_yes(P1, true);
            t.answer_choose(P1, &[]);
        });
        assert_eq!(tapped, hoser.is_some(), "{hoser:?} Port Town");
    }
}

#[test]
fn thalia_entering_together() {
    cr!("614.12", "603.6a");
    ruling!(
        "Thalia, Heretic Cathar",
        "If Thalia enters the battlefield at the same time as an opponent’s creatures or nonbasic lands, those creatures and lands aren’t affected by Thalia’s last ability."
    );
    let mut t = TestGame::new(2);
    let ids = enter_together(
        &mut t,
        &[
            ("Thalia, Heretic Cathar", P0),
            ("Grizzly Bears", P1),
            ("Tundra", P1),
        ],
    );
    assert!(!t.obj_now(ids[1]).tapped);
    assert!(!t.obj_now(ids[2]).tapped);
    // Afterwards, they enter tapped.
    let bears = t.enter(P1, "Grizzly Bears");
    assert!(t.obj_now(bears).tapped);
}

/// P0's Alpine Moon naming `name`.
fn alpine_moon(t: &mut TestGame, name: &str) {
    supported("Alpine Moon");
    t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
    t.enter(P0, "Alpine Moon");
    t.resolve_all();
}

#[test]
fn alpine_moon_removes_entering_abilities() {
    cr!("614.12", "613.1f", "603.6a");
    ruling!(
        "Alpine Moon",
        "If an affected land has an ability that causes it to enter the battlefield tapped, it will lose that ability before it applies."
    );
    ruling!(
        "Alpine Moon",
        "If an affected land has an ability that triggers “when” it enters the battlefield, it will lose that ability before it triggers."
    );
    supported("Radiant Fountain");
    let mut t = TestGame::new(2);
    alpine_moon(&mut t, "Glacial Fortress");
    t.set_step(P1, Step::PrecombatMain);
    let gf = t.hand(P1, "Glacial Fortress");
    t.play_land(P1, gf).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(gf).tapped);
    let mut t = TestGame::new(2);
    alpine_moon(&mut t, "Radiant Fountain");
    t.set_step(P1, Step::PrecombatMain);
    let rf = t.hand(P1, "Radiant Fountain");
    t.play_land(P1, rf).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // P0's own Radiant Fountain isn't affected.
    t.enter(P0, "Radiant Fountain");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

// ---------------------------------------------------------------------------------------
// Burning Earth
// ---------------------------------------------------------------------------------------

#[test]
fn burning_earth_triggers_wait_for_casting() {
    cr!("605.4a", "603.3", "601.2i");
    ruling!(
        "Burning Earth",
        "If any nonbasic lands are tapped for mana while a player is casting a spell or activating an ability, Burning Earth’s ability will trigger that many times and wait."
    );
    ruling!(
        "Burning Earth",
        "The ability will trigger each time a nonbasic land is tapped for mana. Each of these abilities goes on the stack and resolves separately."
    );
    supported("Burning Earth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Burning Earth");
    t.lands(P1, "Tundra", 3);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.hand(P1, "Divination");
    let div = t.cast(P1, div).go();
    let st = stack(&t);
    t.settle();
    let st2 = stack(&t);
    assert_eq!(st, vec![div]);
    assert_eq!(st2.len(), 4);
    assert_eq!(st2[0], div);
    // Each trigger resolves separately, before Divination.
    let hand = t.hand_size(P1);
    for k in 1..=3 {
        t.resolve();
        assert_eq!(t.life(P1), 20 - k);
        assert_eq!(t.hand_size(P1), hand);
    }
    t.resolve();
    assert_eq!(t.hand_size(P1), hand + 2);
}

#[test]
fn burning_earth_respond_with_the_mana() {
    cr!("605.3a", "603.3");
    ruling!(
        "Burning Earth",
        "On the other hand, a player can tap nonbasic lands for mana, put the Burning Earth triggered abilities on the stack, and then respond to those abilities"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Burning Earth");
    let tundra = t.battlefield(P1, "Tundra");
    // Tap for {U} (the Tundra's second mana ability).
    t.activate(P1, tundra, 1, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let opt = t.hand(P1, "Opt");
    let opt = t.cast(P1, opt).go();
    assert_eq!(stack(&t).last(), Some(&opt));
    let hand = t.hand_size(P1);
    t.resolve();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn mercadias_downfall_attackers_fixed_on_resolution() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Mercadia's Downfall",
        "The creatures that are attacking is determined when this resolves and is not changed later"
    );
    supported("Mercadia's Downfall");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Tundra", 2);
    t.battlefield(P1, "Island");
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    let spell = t.hand(P0, "Mercadia's Downfall");
    pool(&mut t, P0, &[(ManaType::R, 3)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(giant), (3, 3));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.life(P1), 16);
}
