//! Rulings batch P212 — Equipment (CR 301.5, 702.6): equip abilities (costs paid on
//! activation, CR 601.2h, 602.2; targets checked on resolution, CR 608.2b), Equipment
//! that attaches itself with a triggered ability (CR 301.5b), attaching to a creature it
//! can't legally equip (CR 301.5c, 301.5e, 701.3b), phasing (CR 702.26), and the timing
//! of "while attached" effects.

use crate::r_p212_common::*;
use crate::r_p213_common::equip;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use crate::r_s16_common::add_charge;
use mtg_engine::ability::{Effect, Filter, Sel};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn main_phase(p: PlayerId) -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(p, Step::PrecombatMain);
    t
}

#[test]
fn conquerors_flail_stays_attached_while_its_equip_ability_is_on_the_stack() {
    cr!("702.6a", "601.2h");
    ruling!(
        "Conqueror's Flail",
        "If Conquerer's Flail is attached to a creature you control and you activate its equip ability targeting another creature you control, there's no time in between activating that ability and the resolution of that ability during which it's not attached to a creature. Assuming it's your turn, your opponents won't be able to cast spells."
    );
    supported("Conqueror's Flail");
    for attached in [true, false] {
        let mut t = main_phase(P0);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let lion = t.battlefield(P0, "Silvercoat Lion");
        let flail = if attached {
            attach_new(&mut t, P0, "Conqueror's Flail", bears)
        } else {
            t.battlefield(P0, "Conqueror's Flail")
        };
        t.lands(P0, "Wastes", 2);
        equip(&mut t, P0, flail, lion).unwrap();
        assert_eq!(t.stack_len(), 1);
        t.lands(P1, "Mountain", 1);
        let bolt = t.hand(P1, "Lightning Bolt");
        assert_eq!(can_cast(&mut t, P1, bolt, CastMethod::Normal), !attached);
        t.resolve_all();
        assert!(on(&t, flail, lion));
    }
}

#[test]
fn sigil_of_distinction_equip_cost_shrinks_the_first_creature_at_once() {
    cr!("602.2b", "601.2h", "702.6a");
    ruling!(
        "Sigil of Distinction",
        "If Sigil of Distinction is attached to a creature when its equip ability is activated targeting a different creature, the first creature will immediately get a little smaller because a charge counter has been removed from Sigil of Distinction as a cost. Then, when the equip ability resolves, Sigil of Distinction will move to the new creature and the first creature will lose the entire bonus the Equipment was giving it."
    );
    ruling!(
        "Sigil of Distinction",
        "Removing a charge counter from Sigil of Distinction is the only way to pay for its equip ability. If it has no charge counters on it, its equip ability can't be activated."
    );
    supported("Sigil of Distinction");
    let mut t = main_phase(P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let sigil = attach_new(&mut t, P0, "Sigil of Distinction", bears);
    add_charge(&mut t, sigil, 2);
    assert_eq!(t.pt(bears), (4, 4));
    t.lands(P0, "Wastes", 5);
    equip(&mut t, P0, sigil, lion).unwrap();
    assert_eq!(t.pt(bears), (3, 3));
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(lion), (3, 3));
    // With no charge counters (and mana available), it can't be activated.
    move_to_counters(&mut t, sigil, 0);
    assert!(equip(&mut t, P0, sigil, bears).is_err());
    assert!(on(&t, sigil, lion));
}

/// Removes every charge counter from the permanent but `keep`.
fn move_to_counters(t: &mut TestGame, id: ObjectId, keep: u32) {
    let n = t.counters(id, counters::CHARGE);
    t.g.remove_counters(Entity::Object(id), counters::CHARGE, n - keep);
    t.g.recompute();
    assert_eq!(t.counters(id, counters::CHARGE), keep);
}

#[test]
fn dissection_tools_sacrificing_the_target_leaves_the_equipment_where_it_was() {
    cr!("608.2b", "602.2b");
    ruling!(
        "Dissection Tools",
        "If you sacrifice the creature targeted by the equip ability, then the ability won't resolve and the Equipment will remain attached to whatever it was attached to (if anything)."
    );
    supported("Dissection Tools");
    let mut t = main_phase(P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let tools = attach_new(&mut t, P0, "Dissection Tools", bears);
    t.answer_choose(P0, &[Entity::Object(lion)]);
    equip(&mut t, P0, tools, lion).unwrap();
    assert!(t.in_graveyard(P0, "Silvercoat Lion"));
    t.resolve_all();
    assert!(on(&t, tools, bears));
}

#[test]
fn robe_of_stars_phased_out_in_response_isnt_moved() {
    cr!("702.26d", "702.26b", "608.2b");
    ruling!(
        "Robe of Stars",
        "Notably, this means you cannot equip Robe of Stars to another creature while it is phased out. If you activate the Astral Projection ability of an attached Robe of Stars in response to the equip ability, Robe of Stars will be phased out before the equip ability resolves and will not be moved."
    );
    supported("Robe of Stars");
    let mut t = main_phase(P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let robe = attach_new(&mut t, P0, "Robe of Stars", bears);
    t.lands(P0, "Plains", 4);
    equip(&mut t, P0, robe, lion).unwrap();
    activate_containing(&mut t, P0, robe, "phases out").unwrap();
    t.resolve_all();
    assert!(t.obj_now(robe).phased_out);
    assert!(t.obj_now(bears).phased_out);
    assert!(!on(&t, robe, lion));
    assert_eq!(t.pt(lion), (2, 2));
    // While phased out, it can't be equipped.
    assert!(equip(&mut t, P0, robe, lion).is_err());
}

#[test]
fn o_naginata_power_three_or_greater() {
    cr!("301.5c", "704.5n", "608.2b");
    ruling!(
        "O-Naginata",
        "O-Naginata becomes unattached if the equipped creature's power is reduced so that its power is 2 or less, including the +3/+0 from O-Naginata's effect, as state-based actions are checked."
    );
    ruling!(
        "O-Naginata",
        "O-Naginata's equip ability can target any creature you control—but if the creature's power is 2 or less, it won't become attached as it resolves."
    );
    supported("O-Naginata");
    // Hill Giant 3/3 → 6/3: -3/-0 leaves 3 (still attached), another -1/-0 makes it 2.
    let mut t = main_phase(P0);
    let giant = t.battlefield(P0, "Hill Giant");
    let nag = attach_new(&mut t, P0, "O-Naginata", giant);
    assert_eq!(t.pt(giant), (6, 3));
    boost(&mut t, giant, -3, 0);
    t.settle();
    assert!(on(&t, nag, giant));
    boost(&mut t, giant, -1, 0);
    t.settle();
    assert_eq!(attached_to(&t, nag), None);
    assert_eq!(t.pt(giant), (-1, 3));
    // Equip can target the 2-power Bears, but doesn't attach.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    equip(&mut t, P0, nag, bears).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, nag), None);
}

#[test]
fn stormrider_rig_doesnt_move_to_a_creature_that_left_or_cant_be_equipped() {
    cr!("608.2b", "301.5c", "702.16c");
    ruling!(
        "Stormrider Rig",
        "If the creature that entered the battlefield is no longer on the battlefield as Stormrider Rig's second ability resolves, Stormrider Rig doesn't move. This is also true if that creature can't legally be equipped by Stormrider Rig."
    );
    supported("Stormrider Rig");
    for (name, leaves) in [
        ("Silvercoat Lion", false),
        ("Silvercoat Lion", true),
        ("Tel-Jilad Chosen", false),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let rig = attach_new(&mut t, P0, "Stormrider Rig", bears);
        t.answer_yes(P0, true);
        let c = crate::r_s05_common::enter(&mut t, P0, name);
        assert_eq!(t.stack_len(), 1);
        if leaves {
            move_to(&mut t, c, Zone::Hand(P0));
        }
        t.resolve_all();
        let moved = name == "Silvercoat Lion" && !leaves;
        assert_eq!(on(&t, rig, c), moved, "{name} leaves={leaves}");
        assert_eq!(on(&t, rig, bears), !moved, "{name} leaves={leaves}");
    }
}

#[test]
fn deathrender_creature_with_protection_enters_but_stays_unequipped() {
    cr!("301.5c", "702.16c");
    ruling!(
        "Deathrender",
        "If the creature you put onto the battlefield can't be equipped by Deathrender (due to protection from artifacts, for example), the creature enters but Deathrender remains unattached."
    );
    supported("Deathrender");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dr = attach_new(&mut t, P0, "Deathrender", bears);
    let chosen = t.hand(P0, "Tel-Jilad Chosen");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(chosen)]);
    destroy(&mut t, bears);
    t.resolve_all();
    let chosen = t.g.current(chosen);
    assert_eq!(t.zone(chosen), Zone::Battlefield);
    assert_eq!(attached_to(&t, dr), None);
}

#[test]
fn dancing_sword_declined_stays_an_equipment() {
    cr!("603.5", "301.5");
    ruling!(
        "Dancing Sword",
        "If you choose not to have Dancing Sword become a creature as its triggered ability resolves, it will remain an Equipment and can be equipped to another creature as normal."
    );
    supported("Dancing Sword");
    let mut t = main_phase(P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let sword = attach_new(&mut t, P0, "Dancing Sword", bears);
    t.answer_yes(P0, false);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(!t.obj_now(sword).chars.is_creature());
    assert!(t.obj_now(sword).chars.has_subtype("Equipment"));
    t.lands(P0, "Wastes", 1);
    equip(&mut t, P0, sword, lion).unwrap();
    t.resolve_all();
    assert!(on(&t, sword, lion));
    assert_eq!(t.pt(lion), (4, 3));
}

/// Equipment with "When this Equipment enters, attach it to target creature you control"
/// (or "target legendary creature you control"): it enters unattached, then the trigger
/// attaches it; with no creature, it can still be cast.
#[test]
fn equipment_that_attaches_on_entering_doesnt_enter_attached() {
    cr!("301.5b", "603.3d");
    ruling!(
        "Javelin of Lightning",
        "Javelin of Lightning doesn't enter the battlefield attached to a creature. Instead, the Equipment enters the battlefield and then a triggered ability attaches it to a creature. You may cast Javelin of Lightning even if you don't control any creatures."
    );
    ruling!(
        "Mithril Coat",
        "Mithril Coat doesn't enter the battlefield attached to a creature. Instead, the Equipment enters the battlefield and then a triggered ability attaches it to a creature. You may cast Mithril Coat even if you don't control any creatures."
    );
    ruling!(
        "Quick-Draw Dagger",
        "Quick-Draw Dagger doesn't enter the battlefield attached to a creature. Instead, the Equipment enters the battlefield and then a triggered ability attaches it to a creature. You may cast Quick-Draw Dagger even if you don't control any creatures."
    );
    for (eq, creature) in [
        ("Javelin of Lightning", "Grizzly Bears"),
        ("Mithril Coat", "Isamaru, Hound of Konda"),
        ("Quick-Draw Dagger", "Grizzly Bears"),
    ] {
        supported(eq);
        // No creatures: castable; it resolves and stays unattached.
        let mut t = main_phase(P0);
        give_mana_for(&mut t, P0, eq);
        let card = t.hand(P0, eq);
        assert!(can_cast(&mut t, P0, card, CastMethod::Normal), "{eq}");
        let spell = t.cast(P0, card).go();
        t.resolve_all();
        let id = t.g.current(spell);
        assert_eq!(t.zone(id), Zone::Battlefield, "{eq}");
        assert_eq!(attached_to(&t, id), None, "{eq}");
        // With a creature: it enters unattached and the trigger attaches it.
        let mut t = main_phase(P0);
        let c = t.battlefield(P0, creature);
        give_mana_for(&mut t, P0, eq);
        let card = t.hand(P0, eq);
        t.answer_targets(P0, &[Entity::Object(c)]);
        let spell = t.cast(P0, card).go();
        t.resolve();
        let id = t.g.current(spell);
        assert_eq!(t.zone(id), Zone::Battlefield, "{eq}");
        assert_eq!(attached_to(&t, id), None, "{eq}");
        assert_eq!(t.stack_len(), 1, "{eq}");
        t.resolve_all();
        assert!(on(&t, id, c), "{eq}");
    }
}

#[test]
fn galadhrim_bow_illegal_target_nothing_happens() {
    cr!("608.2b");
    ruling!(
        "Galadhrim Bow",
        "If the target creature becomes an illegal target, the Equipment remains on the battlefield unattached, and no creatures are untapped."
    );
    supported("Galadhrim Bow");
    for illegal in [false, true] {
        let mut t = main_phase(P0);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.objects[bears.0 as usize].tapped = true;
        give_mana_for(&mut t, P0, "Galadhrim Bow");
        let card = t.hand(P0, "Galadhrim Bow");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        let spell = t.cast(P0, card).go();
        t.resolve();
        assert_eq!(t.stack_len(), 1);
        if illegal {
            // No longer a creature P0 controls.
            give_control(&mut t, bears, P1);
        }
        t.resolve_all();
        let bow = t.g.current(spell);
        assert!(t.on_battlefield(bow));
        assert_eq!(on(&t, bow, bears), !illegal);
        assert_eq!(t.obj_now(bears).tapped, illegal);
    }
}

#[test]
fn hammer_of_nazahn_triggers_for_each_equipment_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Hammer of Nazahn",
        "If Hammer of Nazahn enters the battlefield at the same time as other Equipment you control, its ability will trigger for each of those Equipment."
    );
    supported("Hammer of Nazahn");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cards = [
        t.hand(P0, "Hammer of Nazahn"),
        t.hand(P0, "Bonesplitter"),
        t.hand(P0, "Short Sword"),
    ];
    for _ in 0..3 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Object(bears)]);
    }
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::Move {
            what: Sel::All(Filter::Objects(cards.to_vec())),
            to: mtg_engine::ability::Destination::battlefield(),
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    for c in cards {
        assert!(on(&t, c, bears));
    }
    // Hammer +2/+0, Bonesplitter +2/+0, Short Sword +1/+1.
    assert_eq!(t.pt(bears), (7, 3));
}

#[test]
fn bladehold_war_whip_doesnt_reduce_reconfigure() {
    cr!("702.151a", "702.6a", "601.2f");
    ruling!(
        "Bladehold War-Whip",
        "Some Equipment creature cards in other sets have reconfigure, a different ability that attaches them to a creature. Reconfigure is not an equip ability, and reconfigure costs are not reduced by Bladehold War-Whip's second ability."
    );
    supported("Bladehold War-Whip");
    supported("Lizard Blades");
    let mut t = main_phase(P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Bladehold War-Whip");
    let blades = t.battlefield(P0, "Lizard Blades");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Wastes", 1);
    // Reconfigure {2} isn't reduced: one land isn't enough.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert!(crate::r_s04_common::activate_named(&mut t, P0, blades, "Reconfigure", 0).is_err());
    t.clear_answers();
    // Equip {1} is reduced to {0}: no mana spent.
    equip(&mut t, P0, splitter, bears).unwrap();
    t.resolve_all();
    assert!(on(&t, splitter, bears));
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 1);
}

// ---------------------------------------------------------------------------------------
// Halvar, God of Battle: "At the beginning of each combat, you may attach target Aura or
// Equipment attached to a creature you control to target creature you control."
// ---------------------------------------------------------------------------------------

/// Halvar's trigger on the stack at the beginning of P0's combat, targeting `eq` and `to`.
fn halvar_trigger(t: &mut TestGame, eq: ObjectId, to: ObjectId) {
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(eq)]);
    t.answer_targets(P0, &[Entity::Object(to)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
}

#[test]
fn halvar_targets_illegal_or_already_attached_nothing_happens() {
    cr!("608.2b", "701.3b");
    ruling!(
        "Halvar, God of Battle // Sword of the Realms",
        "If either target of the triggered ability is an illegal target as the ability resolves, the ability won't do anything. If both targets are illegal, the ability won't resolve. If the Aura or Equipment is already attached to the target creature, nothing happens."
    );
    supported("Halvar, God of Battle // Sword of the Realms");
    // 0: legal (control); 1: the Equipment becomes illegal (moved to a creature P0
    // doesn't control); 2: the creature becomes illegal; 3: both.
    for case in 0..4 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Halvar, God of Battle // Sword of the Realms");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let lion = t.battlefield(P0, "Silvercoat Lion");
        let elves = t.battlefield(P1, "Llanowar Elves");
        let eq = attach_new(&mut t, P0, "Bonesplitter", bears);
        halvar_trigger(&mut t, eq, lion);
        assert_eq!(t.stack_len(), 1, "case {case}");
        if case == 1 || case == 3 {
            move_eq(&mut t, eq, elves);
        }
        if case >= 2 {
            move_to(&mut t, lion, Zone::Hand(P0));
        }
        t.resolve_all();
        match case {
            0 => assert!(on(&t, eq, lion)),
            1 | 3 => assert!(on(&t, eq, elves), "case {case}"),
            _ => assert!(on(&t, eq, bears)),
        }
    }
    // Already attached to the target creature: it stays (no new timestamp needed to
    // observe; it's simply still attached).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Halvar, God of Battle // Sword of the Realms");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let eq = attach_new(&mut t, P0, "Bonesplitter", bears);
    halvar_trigger(&mut t, eq, bears);
    t.resolve_all();
    assert!(on(&t, eq, bears));
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn halvar_can_move_an_equipment_you_dont_control() {
    cr!("115.1", "301.5d");
    ruling!(
        "Halvar, God of Battle // Sword of the Realms",
        "Similarly, you don't have to control the Aura or Equipment that you choose as a target of Halvar's triggered ability, but it must be attached to a creature you control."
    );
    // P1's Bonesplitter on P0's Bears: P0 moves it to P0's Lion; P1 still controls it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Halvar, God of Battle // Sword of the Realms");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let eq = attach_new(&mut t, P1, "Bonesplitter", bears);
    halvar_trigger(&mut t, eq, lion);
    t.resolve_all();
    assert!(on(&t, eq, lion));
    assert_eq!(t.obj_now(eq).controller, P1);
    // An Equipment attached to a creature P0 doesn't control can't be targeted: the
    // trigger has no legal first target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Halvar, God of Battle // Sword of the Realms");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let eq = attach_new(&mut t, P0, "Bonesplitter", elves);
    halvar_trigger(&mut t, eq, lion);
    t.resolve_all();
    assert!(on(&t, eq, elves));
}
