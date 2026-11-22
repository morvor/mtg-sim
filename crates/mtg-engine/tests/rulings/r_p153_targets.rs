//! Rulings batch P153 — "scales with mana value": targets that are illegal as the spell
//! or ability resolves (CR 608.2b: the parts that need the illegal target do nothing, and
//! a spell or ability all of whose targets are illegal doesn't resolve), abilities whose
//! source has left the battlefield (CR 113.7a), and the values an effect takes from a
//! permanent that wasn't destroyed.

use crate::r_p153_common::*;
use crate::r_s01_common::{supported, tokens, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s04_common::{next_upkeep, on_stack, top_of_stack};
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s07_common::resolved;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;

use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the ability `id` (an object on the stack) finished resolving this turn.
fn ability_resolved(t: &TestGame, id: ObjectId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::AbilityResolved { ability, .. } if *ability == id))
}

/// P0 casts Orim's Thunder kicked at `artifact` and `creature`; the spell's id.
fn kicked_thunder(t: &mut TestGame, artifact: ObjectId, creature: ObjectId) -> ObjectId {
    let ot = t.hand(P0, "Orim's Thunder");
    mana(t, P0, W, 1);
    mana(t, P0, R, 1);
    mana(t, P0, C, 2);
    t.cast(P0, ot)
        .kicked(true)
        .targets(&[obj(artifact)])
        .targets(&[obj(creature)])
        .go()
}

#[test]
fn orims_thunder_with_an_illegal_target() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Orim's Thunder",
        "If Orim's Thunder is kicked, but the artifact or enchantment is an illegal target as Orim's Thunder tries to resolve, Orim's Thunder won't be able to determine the artifact or enchantment's mana value. No damage will be dealt to the target creature, even if it remained a legal target. If only the creature has become an illegal target, the artifact or enchantment will still be destroyed. (If both targets become illegal, Orim's Thunder won't resolve at all.)"
    );
    supported("Orim's Thunder");
    // The artifact (Hill Giant-sized: Juggernaut, mana value 4) leaves: no damage.
    let mut t = TestGame::new(2);
    let jug = t.battlefield(P1, "Juggernaut");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = kicked_thunder(&mut t, jug, giant);
    move_to(&mut t, jug, Zone::Hand(P1));
    t.resolve();
    assert!(resolved(&t, spell));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
    // The creature leaves: the artifact is still destroyed.
    let mut t = TestGame::new(2);
    let jug = t.battlefield(P1, "Juggernaut");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = kicked_thunder(&mut t, jug, giant);
    move_to(&mut t, giant, Zone::Hand(P1));
    t.resolve();
    assert!(resolved(&t, spell));
    assert_eq!(t.zone(jug), Zone::Graveyard(P1));
    // Both leave: it doesn't resolve.
    let mut t = TestGame::new(2);
    let jug = t.battlefield(P1, "Juggernaut");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = kicked_thunder(&mut t, jug, giant);
    move_to(&mut t, giant, Zone::Hand(P1));
    move_to(&mut t, jug, Zone::Hand(P1));
    t.resolve();
    assert!(!resolved(&t, spell));
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    // Both legal: Juggernaut is destroyed and Hill Giant takes 4.
    let mut t = TestGame::new(2);
    let jug = t.battlefield(P1, "Juggernaut");
    let giant = t.battlefield(P1, "Hill Giant");
    kicked_thunder(&mut t, jug, giant);
    t.resolve();
    assert_eq!(t.zone(jug), Zone::Graveyard(P1));
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
}

#[test]
fn vengeful_rebirth_with_both_targets_illegal_doesnt_resolve_and_isnt_exiled() {
    cr!("608.2b");
    ruling!(
        "Vengeful Rebirth",
        "If both targets are illegal, Vengeful Rebirth doesn’t resolve. It will be put into your graveyard."
    );
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let vr = t.hand(P0, "Vengeful Rebirth");
    mana(&mut t, P0, R, 1);
    mana(&mut t, P0, G, 1);
    mana(&mut t, P0, C, 4);
    let spell = t
        .cast(P0, vr)
        .targets(&[obj(bolt)])
        .targets(&[obj(bears)])
        .go();
    move_to(&mut t, bolt, Zone::Exile);
    destroy(&mut t, bears);
    t.resolve();
    assert!(!resolved(&t, spell));
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    assert!(!t.in_exile("Vengeful Rebirth"));
}

/// P0's upkeep with Puca's Mischief: the trigger is on the stack with its targets chosen
/// (`mine`, `theirs`), and returned.
fn pucas_upkeep(t: &mut TestGame, mine: ObjectId, theirs: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(theirs)]);
    next_upkeep(t, P0);
    assert_eq!(triggers_on_stack(t, "exchange control"), 1);
    top_of_stack(t)
}

#[test]
fn pucas_mischief_with_an_illegal_target() {
    cr!("608.2b", "701.12a");
    ruling!(
        "Puca's Mischief",
        "If either one of the targets becomes illegal (because it leaves the battlefield or for any other reason) by the time the ability resolves, the exchange won't happen. If both targets become illegal, the ability doesn't resolve."
    );
    supported("Puca's Mischief");
    // One target leaves: no exchange.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puca's Mischief");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let trig = pucas_upkeep(&mut t, mine, theirs);
    move_to(&mut t, theirs, Zone::Hand(P1));
    t.answer_yes(P0, true);
    t.resolve();
    assert!(ability_resolved(&t, trig));
    assert_eq!(t.obj_now(mine).controller, P0);
    // Both leave: it doesn't resolve.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puca's Mischief");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let trig = pucas_upkeep(&mut t, mine, theirs);
    move_to(&mut t, theirs, Zone::Hand(P1));
    move_to(&mut t, mine, Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!ability_resolved(&t, trig));
    // Both legal: the exchange happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puca's Mischief");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    pucas_upkeep(&mut t, mine, theirs);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.obj_now(mine).controller, P1);
    assert_eq!(t.obj_now(theirs).controller, P0);
}

#[test]
fn pucas_mischief_must_target_when_it_can_and_targets_nothing_otherwise() {
    cr!("603.3d", "601.2c", "115.1");
    ruling!(
        "Puca's Mischief",
        "Even if you don't plan on exchanging control of any permanents, if you can choose two legal targets when the ability triggers, you must do so. If you can't, then you don't choose any targets."
    );
    // Both targets available: each is a required choice (at least one).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puca's Mischief");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    next_upkeep(&mut t, P0);
    let mins: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { min, .. } if *p == P0 => Some(*min),
            _ => None,
        })
        .collect();
    assert_eq!(mins, vec![1, 1]);
    assert_eq!(triggers_on_stack(&t, "exchange control"), 1);
    // The opponent controls no nonland permanent with mana value 4 or less (Puca's
    // Mischief itself is the greatest P0 could offer): no targets are chosen and the
    // ability is removed from the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puca's Mischief");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Craw Wurm");
    t.lands(P1, "Forest", 2);
    let from = t.asked().len();
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "exchange control"), 0);
    // No target is asked for at all: there's no pair to choose.
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.is_empty(), "{cands:?}");
    // The targets are chosen as a set that meets every condition (CR 601.2c): Memnite (0)
    // can't be the first target, since no permanent of the opponent's has mana value 0 or
    // less; Grizzly Bears (2) and Puca's Mischief (4) can, and with the Bears chosen only
    // Llanowar Elves (1) is offered for the second, not Craw Wurm (6).
    let mut t = TestGame::new(2);
    let mischief = t.battlefield(P0, "Puca's Mischief");
    let memnite = t.battlefield(P0, "Memnite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(elves)]);
    next_upkeep(&mut t, P0);
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 2, "{cands:?}");
    assert!(cands[0].contains(&obj(bears)) && cands[0].contains(&obj(mischief)), "{cands:?}");
    assert!(!cands[0].contains(&obj(memnite)), "{cands:?}");
    assert_eq!(cands[1], vec![obj(elves)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.obj_now(elves).controller, P0);
    assert_eq!(t.obj_now(wurm).controller, P1);
}

#[test]
fn jade_seedstones_counters_for_an_illegal_target_are_lost() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Jade Seedstones // Jadeheart Attendant",
        "If some of the creatures are illegal targets as the first ability of Jade Seedstones tries to resolve, the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost. They won't be put instead on a legal target."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    crate::r_s05_common::enter(&mut t, P0, "Jade Seedstones // Jadeheart Attendant");
    assert_eq!(triggers_on_stack(&t, "distribute three"), 1);
    destroy(&mut t, b);
    t.resolve();
    assert_eq!(t.counters(a, "+1/+1"), 1);
}

#[test]
fn razor_hippogriff_with_an_illegal_target_gains_no_life() {
    cr!("608.2b");
    ruling!(
        "Razor Hippogriff",
        "If the artifact card in your graveyard is an illegal target by the time the ability resolves, the ability won’t resolve. You won’t gain any life."
    );
    supported("Razor Hippogriff");
    let mut t = TestGame::new(2);
    let jug = t.graveyard(P0, "Juggernaut");
    t.answer_targets(P0, &[obj(jug)]);
    crate::r_s05_common::enter(&mut t, P0, "Razor Hippogriff");
    let trig = top_of_stack(&t);
    move_to(&mut t, jug, Zone::Exile);
    t.resolve();
    assert!(!ability_resolved(&t, trig));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.zone(jug), Zone::Exile);
}

#[test]
fn imps_mischief_cant_target_a_spell_with_two_targets_even_if_one_is_illegal() {
    cr!("115.1");
    ruling!(
        "Imp's Mischief",
        "If a spell targets multiple things, you can't target it with Imp's Mischief, even if all but one of those targets have become illegal."
    );
    supported("Arc Trail");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    // P1's Arc Trail: 2 damage to P0, 1 damage to P0's Grizzly Bears; the Bears leave.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let arc = t.hand(P1, "Arc Trail");
    mana(&mut t, P1, R, 1);
    mana(&mut t, P1, C, 1);
    t.g.turn.priority = Some(P1);
    let spell = t
        .cast(P1, arc)
        .targets(&[Entity::Player(P0)])
        .targets(&[obj(bears)])
        .go();
    move_to(&mut t, bears, Zone::Hand(P0));
    let cands = crate::r_s04_common::spell_targets(&mut t, P0, "Imp's Mischief");
    assert!(!cands.contains(&obj(spell)), "{cands:?}");
    // A spell with a single target can be targeted.
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, R, 1);
    t.g.turn.priority = Some(P1);
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    let cands = crate::r_s04_common::spell_targets(&mut t, P0, "Imp's Mischief");
    assert!(cands.contains(&obj(bolt)) && !cands.contains(&obj(spell)), "{cands:?}");
}

#[test]
fn draining_whelk_counters_the_spell_even_after_leaving_the_battlefield() {
    cr!("113.7a", "608.2b");
    ruling!(
        "Draining Whelk",
        "Draining Whelk's ability counters the target spell even if you can't put counters on Draining Whelk, most likely because Draining Whelk left the battlefield before the ability resolved."
    );
    supported("Draining Whelk");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let bears = t.hand(P1, "Grizzly Bears");
    mana(&mut t, P1, G, 2);
    let spell = t.cast(P1, bears).go();
    let whelk = t.hand(P0, "Draining Whelk");
    mana(&mut t, P0, U, 2);
    mana(&mut t, P0, C, 4);
    t.g.turn.priority = Some(P0);
    let whelk = t.cast(P0, whelk).go();
    t.resolve();
    t.answer_targets(P0, &[obj(spell)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "counter target spell"), 1);
    destroy(&mut t, whelk);
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Graveyard(P1));
    assert_eq!(t.counters(whelk, "+1/+1"), 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn pyreswipe_hawk_leaving_before_its_trigger_resolves_changes_no_control() {
    cr!("611.2b", "113.7a", "700.14");
    ruling!(
        "Pyreswipe Hawk",
        "If Pyreswipe Hawk leaves the battlefield before its last ability resolves, control of the target artifact won't change at all."
    );
    supported("Pyreswipe Hawk");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Pyreswipe Hawk");
    let ring = t.battlefield(P1, "Sol Ring");
    // Six mana spent on spells this turn: Hill Giant and Grizzly Bears.
    cast_simple(&mut t, P0, "Hill Giant");
    t.resolve_all();
    t.answer_targets(P0, &[obj(ring)]);
    cast_simple(&mut t, P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "gain control"), 1, "{}", t.dump_log());
    destroy(&mut t, hawk);
    t.resolve_all();
    assert_eq!(t.obj_now(ring).controller, P1);
    // With the Hawk still there, control changes.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyreswipe Hawk");
    let ring = t.battlefield(P1, "Sol Ring");
    cast_simple(&mut t, P0, "Hill Giant");
    t.resolve_all();
    t.answer_targets(P0, &[obj(ring)]);
    cast_simple(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.obj_now(ring).controller, P0);
}

#[test]
fn skyclave_apparition_leaving_before_its_first_ability_resolves_still_exiles() {
    cr!("113.7a", "603.10a");
    ruling!(
        "Skyclave Apparition",
        "If Skyclave Apparition leaves the battlefield before its first ability resolves, the ability still exiles the target permanent."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let app = crate::r_s05_common::enter(&mut t, P0, "Skyclave Apparition");
    assert_eq!(triggers_on_stack(&t, "exile up to one"), 1);
    destroy(&mut t, app);
    // The leaves trigger is above the enters trigger: nothing is exiled yet, so no token.
    assert_eq!(on_stack(&t, "creates an X/X"), 1);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(tokens(&t, P1).is_empty(), "{}", t.dump_log());
}

#[test]
fn skyclave_apparition_token_sums_the_mana_values_of_all_the_exiled_cards() {
    cr!("603.2d", "608.2h");
    ruling!(
        "Skyclave Apparition",
        "If Skyclave Apparition's first ability exiled more than one card owned by a single player, that player creates a token with power and toughness equal to the sum of those cards' mana values. If the first ability exiled cards owned by more than one player, each of those players creates a token with power and toughness equal to the sum of the mana values of all cards exiled by the first ability."
    );
    supported("Panharmonicon");
    // Panharmonicon: the enters ability triggers twice and exiles P1's Grizzly Bears (2)
    // and Hill Giant (4); when the Apparition leaves, P1 creates one 6/6.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Panharmonicon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    let app = crate::r_s05_common::enter(&mut t, P0, "Skyclave Apparition");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    destroy(&mut t, app);
    t.resolve_all();
    let toks = tokens(&t, P1);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(toks[0]), (6, 6));
    // Cards owned by two players: each creates a 6/6.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Panharmonicon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    let app = crate::r_s05_common::enter(&mut t, P0, "Skyclave Apparition");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    destroy(&mut t, app);
    t.resolve_all();
    let (t1, t2) = (tokens(&t, P1), tokens(&t, P2));
    assert_eq!((t1.len(), t2.len()), (1, 1), "{}", t.dump_log());
    assert_eq!(t.pt(t1[0]), (6, 6));
    assert_eq!(t.pt(t2[0]), (6, 6));
}

#[test]
fn eriette_gains_no_control_if_the_aura_is_gone_as_the_trigger_resolves() {
    cr!("611.2b");
    ruling!(
        "Eriette, the Beguiler",
        "If the Aura you control becomes unattached from a nonland permanent (most likely because the Aura left the battlefield) before Eriette’s triggered ability resolves, you won’t gain control of that nonland permanent at all."
    );
    supported("Eriette, the Beguiler");
    supported("Pacifism");
    for aura_leaves in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Eriette, the Beguiler");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let pac = t.hand(P0, "Pacifism");
        mana(&mut t, P0, W, 1);
        mana(&mut t, P0, C, 1);
        let pac = t.cast(P0, pac).target(bears).go();
        t.resolve();
        assert_eq!(triggers_on_stack(&t, "gain control"), 1, "{}", t.dump_log());
        if aura_leaves {
            destroy(&mut t, pac);
        }
        t.resolve_all();
        let expected = if aura_leaves { P1 } else { P0 };
        assert_eq!(t.obj_now(bears).controller, expected, "aura leaves: {aura_leaves}");
    }
}

#[test]
fn light_paws_leaves_an_aura_that_cant_be_attached_to_it_in_the_library() {
    cr!("701.3b", "303.4i");
    ruling!(
        "Light-Paws, Emperor's Voice",
        "If the Aura card you find can't be attached to Light-Paws for any reason, it stays in your library and does not enter the battlefield."
    );
    supported("Light-Paws, Emperor's Voice");
    supported("Spirit Link");
    supported("Wild Growth");
    let mut t = TestGame::new(2);
    let paws = t.battlefield(P0, "Light-Paws, Emperor's Voice");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Wild Growth (enchant land, mana value 1) can't be attached to Light-Paws.
    let wild = t.library_top(P0, "Wild Growth");
    let link = t.hand(P0, "Spirit Link");
    mana(&mut t, P0, W, 1);
    mana(&mut t, P0, C, 1);
    t.cast(P0, link).target(bears).go();
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "search your library"), 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(wild)]);
    t.resolve_all();
    assert_eq!(t.zone(wild), Zone::Library(P0), "{}", t.dump_log());
    assert_eq!(t.library_size(P0), 31);
    assert_eq!(attached_to(&t, paws), None);
}

#[test]
fn divine_offering_gains_life_even_if_the_artifact_survives() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Divine Offering",
        "If Divine Offering resolves, but the artifact regenerates or has indestructible, you'll still gain the life."
    );
    supported("Darksteel Ingot");
    let mut t = TestGame::new(2);
    let ingot = t.battlefield(P1, "Darksteel Ingot");
    let d = t.hand(P0, "Divine Offering");
    mana(&mut t, P0, W, 1);
    mana(&mut t, P0, C, 1);
    t.cast(P0, d).target(ingot).go();
    t.resolve();
    assert!(t.on_battlefield(ingot));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn hoard_smelter_dragon_gets_the_bonus_even_if_the_artifact_survives() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Hoard-Smelter Dragon",
        "If the activated ability resolves but the targeted artifact isn't destroyed (perhaps because it has indestructible or it regenerates), Hoard-Smelter Dragon will still get the bonus."
    );
    supported("Hoard-Smelter Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Hoard-Smelter Dragon");
    let ingot = t.battlefield(P1, "Darksteel Ingot");
    mana(&mut t, P0, R, 1);
    mana(&mut t, P0, C, 3);
    t.answer_targets(P0, &[obj(ingot)]);
    crate::r_s06_common::activate_containing(&mut t, P0, dragon, "Destroy target artifact")
        .unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(ingot));
    assert_eq!(t.pt(dragon), (8, 5));
}

#[test]
fn riveteers_ascendancy_returns_a_creature_for_only_one_of_several_instances() {
    cr!("603.2", "608.2c");
    ruling!(
        "Riveteers Ascendancy",
        "If multiple instances of the ability are on the stack, you will be able to return a creature to the battlefield for only one of those instances. Once you do that, other instances will do nothing as they resolve."
    );
    supported("Riveteers Ascendancy");
    supported("Viscera Seer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riveteers Ascendancy");
    let seer = t.battlefield(P0, "Viscera Seer");
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    let m1 = t.graveyard(P0, "Memnite");
    let m2 = t.graveyard(P0, "Memnite");
    // Two sacrifices in a row, both triggers waiting on the stack.
    for (bear, memnite) in [(b1, m1), (b2, m2)] {
        t.answer_choose(P0, &[obj(bear)]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(memnite)]);
        crate::r_s06_common::activate_containing(&mut t, P0, seer, "Sacrifice a creature")
            .unwrap();
        t.settle();
    }
    assert_eq!(triggers_on_stack(&t, "return target creature card"), 2);
    t.resolve_all();
    let returned = [m1, m2].iter().filter(|m| t.on_battlefield(**m)).count();
    assert_eq!(returned, 1, "{}", t.dump_log());
}

#[test]
fn tetsuo_triggers_only_if_equipped_and_does_nothing_once_unequipped() {
    cr!("603.4", "608.2b");
    ruling!(
        "Tetsuo, Imperial Champion",
        "If Tetsuo isn't equipped when it's declared as an attacking creature, the ability won't trigger. If Tetsuo isn't equipped when the ability resolves, the ability won't do anything."
    );
    supported("Tetsuo, Imperial Champion");
    // Not equipped: no trigger.
    let mut t = TestGame::new(2);
    let tetsuo = t.battlefield(P0, "Tetsuo, Imperial Champion");
    crate::r_s01_common::attack_with(&mut t, &[(tetsuo, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "choose one"), 0);
    // Equipped with Bonesplitter (mana value 1); it's destroyed in response: no damage.
    let mut t = TestGame::new(2);
    let tetsuo = t.battlefield(P0, "Tetsuo, Imperial Champion");
    let axe = attach_new(&mut t, P0, "Bonesplitter", tetsuo);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s01_common::attack_with(&mut t, &[(tetsuo, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "deals damage"), 1, "{}", t.dump_log());
    destroy(&mut t, axe);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Still equipped as it resolves: 1 damage.
    let mut t = TestGame::new(2);
    let tetsuo = t.battlefield(P0, "Tetsuo, Imperial Champion");
    attach_new(&mut t, P0, "Bonesplitter", tetsuo);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s01_common::attack_with(&mut t, &[(tetsuo, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}
