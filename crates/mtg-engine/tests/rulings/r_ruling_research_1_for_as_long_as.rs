//! Research item ruling-research-1: "for as long as" durations of an ability's effect while
//! the ability waits on the stack (CR 611.2b). The duration is checked continually from the
//! moment the ability is activated or triggers (Seasinger ruling); once it has ended, it
//! doesn't begin again before the ability resolves, even if its condition holds again
//! (control regained, the source tapped again), whatever kind of effect it is (a control
//! change, a play permission, a player effect, a rule effect). Also: the trigger time,
//! copies of abilities, the source phasing out (and "until ~ leaves the battlefield", which
//! phasing doesn't end), durations about other objects, durations the resolution sets up,
//! and the resolution itself.

use crate::r_p086_common::*;
use crate::r_s03_common::run_effect;
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::enter_transformed;
use mtg_engine::ability::{
    AbilityKind, Condition, Duration, Effect, Filter, Modification, PlayerRef, Sel, TargetKind,
    Value,
};
use mtg_engine::card::CardDef;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Subtype};
use mtg_engine::*;

// --- Seasinger --------------------------------------------------------------------------------

/// P0's Seasinger ("{T}: Gain control of target creature whose controller controls an Island
/// for as long as you control this creature and this creature remains tapped."), with an
/// Island, is activated targeting P1's `target` (P1 controls an Island too); the ability is
/// left on the stack. Returns Seasinger, the target and P1's Island.
fn seasinger(t: &mut TestGame, target: &str) -> (ObjectId, ObjectId, ObjectId) {
    t.lands(P0, "Island", 1);
    let island = t.lands(P1, "Island", 1)[0];
    let s = t.battlefield(P0, "Seasinger");
    let c = t.battlefield(P1, target);
    activate(t, P0, s, "Gain control", &[obj(c)]);
    assert_eq!(t.stack_len(), 1);
    (s, c, island)
}

#[test]
fn seasinger_lost_or_untapped_while_its_ability_waits_gains_nothing() {
    cr!("611.2b");
    ruling!(
        "Seasinger",
        "Whether you control Seasinger is checked continually, starting when the ability is activated."
    );
    interp!("for-as-long-as-checked-continually-from-activation");
    supported("Seasinger");
    // Nothing happens in between: P0 gains control of the Bears.
    let mut t = TestGame::new(2);
    let (_, bears, _) = seasinger(&mut t, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0);
    // Control of Seasinger lost and regained before the ability resolves: "the ability has
    // no effect — even if you control Seasinger again by the time the ability resolves."
    let mut t = TestGame::new(2);
    let (s, bears, _) = seasinger(&mut t, "Grizzly Bears");
    give_control(&mut t, s, P1);
    give_control(&mut t, s, P0);
    t.resolve_all();
    assert_eq!(controller(&t, s), P0);
    assert!(is_tapped(&t, s));
    assert_eq!(controller(&t, bears), P1);
    // "The same is true regarding whether Seasinger remains tapped": untapped and tapped
    // again.
    let mut t = TestGame::new(2);
    let (s, bears, _) = seasinger(&mut t, "Grizzly Bears");
    untap(&mut t, s);
    tap(&mut t, s);
    t.resolve_all();
    assert!(is_tapped(&t, s));
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn seasinger_keeps_the_creature_until_it_untaps_or_leaves_your_control() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Seasinger",
        "You do not lose control of the stolen creature if it stops being a creature. You only lose control if Seasinger leaves your control or becomes untapped."
    );
    let mut t = TestGame::new(2);
    let (s, bears, _) = seasinger(&mut t, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0);
    // It stops being a creature (Song of the Dryads: "Enchanted permanent is a colorless
    // Forest land."): still P0's.
    attach_new(&mut t, P1, "Song of the Dryads", obj(bears));
    t.g.recompute();
    assert!(!t.obj_now(bears).is(CardType::Creature));
    assert_eq!(controller(&t, bears), P0);
    // Seasinger untaps: back to P1.
    untap(&mut t, s);
    assert_eq!(controller(&t, bears), P1);
    // Seasinger leaves P0's control, staying tapped: back to P1 too.
    let mut t = TestGame::new(2);
    let (s, bears, _) = seasinger(&mut t, "Grizzly Bears");
    t.resolve_all();
    give_control(&mut t, s, P1);
    assert!(is_tapped(&t, s));
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn seasingers_island_requirement_is_checked_only_on_activation_and_resolution() {
    cr!("601.2c", "608.2b");
    ruling!(
        "Seasinger",
        "If a creature’s controller doesn’t control an Island, that creature is an illegal target. Whether that player controls an Island is checked both when Seasinger"
    );
    // Without an Island, P1's creature can't be targeted.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let s = t.battlefield(P0, "Seasinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!ability_targets(&mut t, s, 0).contains(&obj(bears)));
    // P1's Island is destroyed in response and another one arrives before the ability
    // resolves: the moment in between doesn't matter, unlike for the duration.
    let mut t = TestGame::new(2);
    let (_, bears, island) = seasinger(&mut t, "Grizzly Bears");
    destroy(&mut t, island);
    t.lands(P1, "Island", 1);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0);
    // No Island as the ability resolves: its only target is illegal, it doesn't resolve.
    let mut t = TestGame::new(2);
    let (s, bears, island) = seasinger(&mut t, "Grizzly Bears");
    destroy(&mut t, island);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1);
    assert_eq!(t.zone(s), Zone::Battlefield);
}

// --- Other effect kinds: a play permission, a player effect -----------------------------------

fn bolt_p1(t: &mut TestGame, bolt: ObjectId) -> bool {
    let b = t.g.current(bolt);
    let ok = t.cast_with(P0, b, &[Entity::Player(P1)]).is_ok();
    if ok {
        t.resolve_all();
    }
    ok
}

#[test]
fn lightning_lost_and_regained_while_its_trigger_waits_gives_no_permission() {
    cr!("611.2b");
    interp!("for-as-long-as-checked-continually-from-activation");
    supported("Lightning, Security Sergeant");
    // "Whenever Lightning deals combat damage to a player, exile the top card of your
    // library. You may play that card for as long as you control Lightning."
    for regain in [None, Some(false), Some(true)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 1);
        let bolt = t.library_top(P0, "Lightning Bolt");
        let lightning = t.battlefield(P0, "Lightning, Security Sergeant");
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(lightning, Entity::Player(P1))]),
        );
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert_eq!(t.stack_len(), 1, "regain={regain:?}");
        if let Some(back) = regain {
            give_control(&mut t, lightning, P1);
            if back {
                give_control(&mut t, lightning, P0);
            }
        }
        t.resolve_all();
        assert_eq!(t.zone(bolt), Zone::Exile, "regain={regain:?}");
        t.advance_to(P0, Step::PostcombatMain);
        let played = bolt_p1(&mut t, bolt);
        assert_eq!(played, regain.is_none(), "regain={regain:?}");
    }
}

#[test]
fn great_synthesis_lost_and_regained_while_chapter_one_waits_keeps_the_maximum_hand_size() {
    cr!("611.2b");
    interp!("for-as-long-as-checked-continually-from-activation");
    supported("Jin-Gitaxias // The Great Synthesis");
    // Chapter I: "Draw cards equal to the number of cards in your hand. You have no maximum
    // hand size for as long as you control this Saga."
    for regained in [false, true] {
        let mut t = TestGame::new(2);
        let saga = enter_transformed(&mut t, P0, "Jin-Gitaxias // The Great Synthesis");
        assert_eq!(t.stack_len(), 1);
        if regained {
            give_control(&mut t, saga, P1);
            give_control(&mut t, saga, P0);
        }
        t.resolve_all();
        assert_eq!(controller(&t, saga), P0);
        t.g.recompute();
        // Untouched, P0 has no maximum hand size; lost and regained, still seven.
        let want = if regained { Some(7) } else { None };
        assert_eq!(t.g.player(P0).max_hand_size, want, "regained={regained}");
    }
}

// --- When the period starts: the trigger, a copy ----------------------------------------------

/// P0's Tidebinder Mage ("When this creature enters, tap target red or green creature an
/// opponent controls. That creature doesn't untap during its controller's untap step for as
/// long as you control this creature.") enters targeting P1's Raging Goblin; the trigger is
/// left on the stack. Returns the Mage and the Goblin.
fn tidebinder_trigger(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.answer_targets(P0, &[obj(goblin)]);
    let mage = t.enter(P0, "Tidebinder Mage");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (mage, goblin)
}

/// Whether the Goblin stays tapped through P1's next untap step.
fn locked(t: &mut TestGame, goblin: ObjectId) -> bool {
    assert!(is_tapped(t, goblin));
    next_main(t, P1);
    is_tapped(t, goblin)
}

/// `p` gains control of the permanent, without putting triggered abilities on the stack.
fn give_control_quietly(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn control_lost_after_the_trigger_and_regained_before_it_goes_on_the_stack_ends_the_duration() {
    cr!("611.2b", "603.3");
    ruling!(
        "Tidebinder Mage",
        "If you lose control of Tidebinder Mage between when it entered the battlefield and when the ability resolves, the target creature will become tapped."
    );
    interp!("for-as-long-as-checked-continually-from-activation");
    // The period starts as the ability triggers, not as it's put on the stack.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let mage = t.enter(P0, "Tidebinder Mage");
    give_control_quietly(&mut t, mage, P1);
    give_control_quietly(&mut t, mage, P0);
    assert_eq!(t.stack_len(), 0);
    t.answer_targets(P0, &[obj(goblin)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(controller(&t, mage), P0);
    assert!(!locked(&mut t, goblin));
}

/// `resonator` ("{2}, {T}: Copy target triggered ability you control. You may choose new
/// targets for the copy.") copies the topmost ability; the copy is left on the stack.
fn copy_top(t: &mut TestGame, resonator: ObjectId) {
    let top = *t.g.stack.last().expect("nothing to copy");
    t.answer_targets(P0, &[Entity::Object(top)]);
    activate_containing(t, P0, resonator, "Copy target").unwrap();
    t.settle();
    t.resolve();
}

#[test]
fn a_copied_abilitys_period_starts_as_the_copy_is_put_onto_the_stack() {
    cr!("707.10", "611.2b");
    interp!("copied-ability-duration-starts-as-the-copy-is-created");
    supported("Strionic Resonator");
    // Copied, then control of the Mage lost and regained while both wait: neither the copy
    // nor the original locks the Goblin.
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    t.lands(P0, "Wastes", 2);
    let (mage, goblin) = tidebinder_trigger(&mut t);
    copy_top(&mut t, resonator);
    assert_eq!(t.stack_len(), 2);
    give_control(&mut t, mage, P1);
    give_control(&mut t, mage, P0);
    t.resolve_all();
    assert!(!locked(&mut t, goblin));
    // The same with the Mage untouched: the copy and the original lock it.
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    t.lands(P0, "Wastes", 2);
    let (_, goblin) = tidebinder_trigger(&mut t);
    copy_top(&mut t, resonator);
    t.resolve_all();
    assert!(locked(&mut t, goblin));
    // Control lost and regained before the copy is made: the copy's duration starts as the
    // copy is put onto the stack, so the copy locks the Goblin (the original doesn't).
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    t.lands(P0, "Wastes", 2);
    let (mage, goblin) = tidebinder_trigger(&mut t);
    give_control(&mut t, mage, P1);
    give_control(&mut t, mage, P0);
    copy_top(&mut t, resonator);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(locked(&mut t, goblin));
}

#[test]
fn a_copy_of_an_activated_ability_has_a_period_of_its_own_too() {
    cr!("707.10", "611.2b");
    interp!("copied-ability-duration-starts-as-the-copy-is-created");
    supported("Rings of Brighthearth");
    // Rings of Brighthearth: "Whenever you activate an ability, if it isn't a mana ability,
    // you may pay {2}. If you do, copy that ability." Seasinger's ability is copied as the
    // Rings' trigger resolves (`copy`), and the copy resolves before the original.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Case {
        Untouched,
        // Control lost and regained while the copy and the original wait.
        LostAfterCopy,
        // Seasinger untapped before the copy is made and tapped again after: the copy's
        // duration never started ("remains tapped").
        UntappedBeforeCopy,
        // Control lost before the copy is made and regained after: never started either.
        LostBeforeCopy,
    }
    for case in [
        Case::Untouched,
        Case::LostAfterCopy,
        Case::UntappedBeforeCopy,
        Case::LostBeforeCopy,
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Rings of Brighthearth");
        t.lands(P0, "Wastes", 2);
        t.lands(P0, "Island", 1);
        t.lands(P1, "Island", 1);
        let s = t.battlefield(P0, "Seasinger");
        let bears = t.battlefield(P1, "Grizzly Bears");
        activate(&mut t, P0, s, "Gain control", &[obj(bears)]);
        match case {
            Case::UntappedBeforeCopy => untap(&mut t, s),
            Case::LostBeforeCopy => give_control(&mut t, s, P1),
            _ => {}
        }
        t.answer_yes(P0, true);
        t.resolve();
        assert_eq!(t.stack_len(), 2, "{case:?}");
        match case {
            Case::LostAfterCopy => {
                give_control(&mut t, s, P1);
                give_control(&mut t, s, P0);
            }
            Case::UntappedBeforeCopy => tap(&mut t, s),
            Case::LostBeforeCopy => give_control(&mut t, s, P0),
            Case::Untouched => {}
        }
        t.resolve_all();
        let want = if case == Case::Untouched { P0 } else { P1 };
        assert_eq!(controller(&t, bears), want, "{case:?}");
    }
}

// --- Phasing ----------------------------------------------------------------------------------

/// The permanent phases out, and then in again.
fn phase_out_and_in(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![id]);
    t.g.flush_events();
    t.settle();
    assert!(t.g.obj(id).phased_out);
    mtg_engine::keyword_impls::phase_in(&mut t.g, id);
    t.g.flush_events();
    t.settle();
    assert!(!t.g.obj(id).phased_out);
}

#[test]
fn the_source_phasing_out_and_in_while_its_ability_waits_ends_the_duration() {
    cr!("702.26f", "611.2b");
    interp!("phasing-out-ends-a-waiting-for-as-long-as-duration");
    // "for as long as you control this creature" (Tidebinder Mage): only the tap.
    let mut t = TestGame::new(2);
    let (mage, goblin) = tidebinder_trigger(&mut t);
    phase_out_and_in(&mut t, mage);
    t.resolve_all();
    assert_eq!(controller(&t, mage), P0);
    assert!(!locked(&mut t, goblin));
    // "for as long as this creature remains on the battlefield" (Sower of Temptation).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let sower = t.enter(P0, "Sower of Temptation");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    phase_out_and_in(&mut t, sower);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1);
    // "for as long as you control this creature and this creature remains tapped"
    // (Seasinger, still tapped as it phases in).
    let mut t = TestGame::new(2);
    let (s, bears, _) = seasinger(&mut t, "Grizzly Bears");
    phase_out_and_in(&mut t, s);
    assert!(is_tapped(&t, s));
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1);
}

/// Whether the land has the land type `ty` now.
fn has_land_type(t: &mut TestGame, land: ObjectId, ty: &str) -> bool {
    t.g.recompute();
    t.obj_now(land)
        .chars
        .subtypes
        .iter()
        .any(|s| *s == Subtype::from(ty))
}

#[test]
fn until_the_source_leaves_the_battlefield_isnt_ended_by_phasing() {
    cr!("702.26d", "611.2a");
    interp!("phasing-out-ends-a-waiting-for-as-long-as-duration");
    supported("Gaea's Liege");
    supported("Graceful Antelope");
    // Phasing out isn't leaving the battlefield (CR 702.26d), unlike "for as long as this
    // creature remains on the battlefield" (Sower of Temptation above), which can't see a
    // phased-out permanent (CR 702.26f). Gaea's Liege: "{T}: Target land becomes a Forest
    // until this creature leaves the battlefield." It phases out and in while its ability
    // waits, and again once the land is a Forest; then it leaves.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2); // the Liege's toughness is the number of P0's Forests
    let liege = t.battlefield(P0, "Gaea's Liege");
    let land = t.lands(P1, "Mountain", 1)[0];
    activate(&mut t, P0, liege, "becomes a Forest", &[obj(land)]);
    phase_out_and_in(&mut t, liege);
    t.resolve_all();
    assert!(has_land_type(&mut t, land, "Forest"));
    phase_out_and_in(&mut t, liege);
    assert!(has_land_type(&mut t, land, "Forest"));
    destroy(&mut t, liege);
    assert!(has_land_type(&mut t, land, "Mountain"));
    assert!(!has_land_type(&mut t, land, "Forest"));
    // Graceful Antelope: "Whenever this creature deals combat damage to a player, you may
    // have target land become a Plains until this creature leaves the battlefield."
    let mut t = TestGame::new(2);
    let land = t.lands(P1, "Mountain", 1)[0];
    let antelope = t.battlefield(P0, "Graceful Antelope");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(antelope, Entity::Player(P1))]),
    );
    t.answer_targets(P0, &[obj(land)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    phase_out_and_in(&mut t, antelope);
    t.resolve_all();
    assert!(has_land_type(&mut t, land, "Plains"));
    phase_out_and_in(&mut t, antelope);
    assert!(has_land_type(&mut t, land, "Plains"));
}

// --- Durations about other objects ------------------------------------------------------------

/// The creature gets +`n`/+0 until end of turn.
fn pump(t: &mut TestGame, id: ObjectId, n: i32) {
    run_effect(
        t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::Const(n), Value::Const(0))],
            duration: Duration::EndOfTurn,
        },
        &[obj(id)],
    );
}

#[test]
fn old_man_of_the_sea_untapped_or_outpowered_while_its_ability_waits_gains_nothing() {
    cr!("611.2b");
    ruling!(
        "Old Man of the Sea",
        "If Old Man of the Sea stops being tapped before its ability resolves — even if it becomes tapped again right away — you won’t gain control of the targeted creature at all. The same is true if the targeted creature’s power becomes greater than Old Man of the Sea’s power before the ability resolves."
    );
    interp!("duration-about-another-object-ends-while-the-ability-waits");
    supported("Old Man of the Sea");
    // "{T}: Gain control of target creature with power less than or equal to this
    // creature's power for as long as this creature remains tapped and that creature's
    // power remains less than or equal to this creature's power." (a 2/3).
    let start = |t: &mut TestGame| {
        let old = t.battlefield(P0, "Old Man of the Sea");
        let bears = t.battlefield(P1, "Grizzly Bears");
        activate(t, P0, old, "Gain control", &[obj(bears)]);
        (old, bears)
    };
    let mut t = TestGame::new(2);
    let (_, bears) = start(&mut t);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0);
    // Untapped, and tapped again right away.
    let mut t = TestGame::new(2);
    let (old, bears) = start(&mut t);
    untap(&mut t, old);
    tap(&mut t, old);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1);
    // The Bears' power becomes greater than the Old Man's (3 > 2), then the Old Man's
    // grows too (3 <= 3): the target is legal as the ability resolves, the duration over.
    let mut t = TestGame::new(2);
    let (old, bears) = start(&mut t);
    pump(&mut t, bears, 1);
    pump(&mut t, old, 1);
    assert_eq!(t.obj_now(bears).chars.power, Some(3));
    assert_eq!(t.obj_now(old).chars.power, Some(3));
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn garland_opponent_stops_and_becomes_the_monarch_again_while_its_trigger_waits() {
    cr!("611.2b", "725.3");
    interp!("duration-about-another-object-ends-while-the-ability-waits");
    supported("Garland, Royal Kidnapper");
    // "Whenever an opponent becomes the monarch, gain control of target creature that
    // player controls for as long as they're the monarch."
    let become_monarch = |t: &mut TestGame, p: PlayerId| {
        run_effect(
            t,
            None,
            p,
            Effect::BecomeMonarch {
                who: PlayerRef::You,
            },
            &[],
        );
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garland, Royal Kidnapper");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.answer_targets(P0, &[obj(bears)]);
    become_monarch(&mut t, P1);
    assert_eq!(t.stack_len(), 1);
    // P1 stops being the monarch, then becomes it again (a new trigger, for the Goblin).
    become_monarch(&mut t, P0);
    t.answer_targets(P0, &[obj(goblin)]);
    become_monarch(&mut t, P1);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(controller(&t, goblin), P0);
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn rootwater_matriarch_needs_the_creature_enchanted_from_activation_to_resolution() {
    cr!("611.2b");
    // Read with the period: "an unenchanted creature" is one unenchanted as it's targeted.
    ruling!(
        "Rootwater Matriarch",
        "Can target an unenchanted creature but has no effect on that creature."
    );
    interp!("duration-about-another-object-ends-while-the-ability-waits");
    supported("Rootwater Matriarch");
    // "{T}: Gain control of target creature for as long as that creature is enchanted."
    // (Untouched, an enchanted creature is stolen and an unenchanted one isn't: r_p176.)
    // Enchanted as the ability is activated, its Aura destroyed and another one attached
    // while the ability waits: the duration ended. Unenchanted as the ability is activated
    // and enchanted while it waits: the duration never started (it didn't begin during the
    // resolution). Either way the creature is enchanted as the ability resolves.
    for enchanted_first in [true, false] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Rootwater Matriarch");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let aura = enchanted_first.then(|| attach_new(&mut t, P1, "Pacifism", obj(bears)));
        activate(&mut t, P0, m, "Gain control", &[obj(bears)]);
        if let Some(a) = aura {
            destroy(&mut t, a);
        }
        attach_new(&mut t, P1, "Pacifism", obj(bears));
        t.resolve_all();
        assert_eq!(controller(&t, bears), P1, "enchanted_first={enchanted_first}");
    }
}

/// Old Man of the Sea ("{T}: Gain control of target creature with power less than or equal
/// to this creature's power for as long as this creature remains tapped and that creature's
/// power remains less than or equal to this creature's power.", a 2/3) able to target any
/// creature, whose ability first gives it +1/+0 until end of turn as it resolves.
fn old_man_growing_first() -> CardDef {
    let mut def = (*card("Old Man of the Sea")).clone();
    def.name = "Old Man of the Sea Probe".into();
    let chars = &mut def.faces[0].chars;
    chars.name = "Old Man of the Sea Probe".into();
    for a in chars.abilities.iter_mut() {
        let a = std::sync::Arc::make_mut(a);
        if let AbilityKind::Activated(act) = &mut a.kind {
            act.body.targets[0].what = TargetKind::Object(Filter::Type(CardType::Creature));
            let grow = Effect::Modify {
                what: Sel::This,
                mods: vec![Modification::ModifyPT(Value::Const(1), Value::Const(0))],
                duration: Duration::EndOfTurn,
            };
            act.body.effect = Effect::Seq(vec![grow, act.body.effect.clone()]);
        }
    }
    def
}

#[test]
fn a_condition_about_another_object_can_begin_during_the_resolution() {
    cr!("611.2b");
    interp!("duration-about-another-object-ends-while-the-ability-waits");
    // The probe (2/3) targets a Hill Giant (3/3): the power condition doesn't hold as the
    // ability is activated. Its resolution makes it hold (the probe becomes 3/3 first): the
    // duration begins during the resolution, and P0 gains control. If another effect makes
    // it hold while the ability waits instead, it doesn't begin during the resolution:
    // nothing, though it holds as the ability resolves (the probe 4/3 by then).
    for pumped_while_waiting in [false, true] {
        let mut t = TestGame::new(2);
        let old = t.custom(P0, old_man_growing_first(), Zone::Battlefield);
        let giant = t.battlefield(P1, "Hill Giant");
        activate(&mut t, P0, old, "Gain control", &[obj(giant)]);
        if pumped_while_waiting {
            pump(&mut t, old, 1);
        }
        t.resolve_all();
        let want = if pumped_while_waiting { P1 } else { P0 };
        assert_eq!(
            controller(&t, giant),
            want,
            "pumped_while_waiting={pumped_while_waiting}"
        );
    }
}

#[test]
fn a_duration_the_resolution_sets_up_is_checked_as_the_effect_begins() {
    cr!("611.2b");
    interp!("duration-the-resolution-sets-up-is-checked-as-the-effect-begins");
    supported("Zygon Infiltrator");
    // Zygon Infiltrator: "{2}{U}: Tap another target creature and put a stun counter on it.
    // This creature becomes a copy of that creature for as long as that creature remains
    // tapped." The target is tapped as the ability is activated and becomes untapped while
    // it waits. The resolution taps it again (the duration begins during the resolution,
    // CR 611.2b), or another effect has tapped it again before the resolution (what happened
    // while the ability waited doesn't matter): either way the Infiltrator becomes a copy.
    for retapped in [false, true] {
        let mut t = TestGame::new(2);
        let zygon = t.battlefield(P0, "Zygon Infiltrator");
        let bears = t.battlefield(P1, "Grizzly Bears");
        tap(&mut t, bears);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 2);
        activate(&mut t, P0, zygon, "Tap another target", &[obj(bears)]);
        assert_eq!(t.stack_len(), 1);
        untap(&mut t, bears);
        if retapped {
            tap(&mut t, bears);
        }
        t.resolve_all();
        assert!(is_tapped(&t, bears));
        assert_eq!(
            t.obj_now(zygon).chars.name.as_str(),
            "Grizzly Bears",
            "retapped={retapped}"
        );
    }
}

// --- The resolution itself; "and" conditions --------------------------------------------------

/// Willow Satyr ("{T}: Gain control of target legendary creature for as long as you control
/// this creature and this creature remains tapped.") whose ability does `first` as it
/// resolves, before it gains control.
fn satyr_doing_first(first: Vec<Effect>) -> CardDef {
    let mut def = (*card("Willow Satyr")).clone();
    def.name = "Willow Satyr Probe".into();
    let chars = &mut def.faces[0].chars;
    chars.name = "Willow Satyr Probe".into();
    for a in chars.abilities.iter_mut() {
        let a = std::sync::Arc::make_mut(a);
        if let AbilityKind::Activated(act) = &mut a.kind {
            let mut v = first.clone();
            v.push(act.body.effect.clone());
            act.body.effect = Effect::Seq(v);
        }
    }
    def
}

/// `who` gains control of the ability's source.
fn source_to(who: PlayerRef) -> Effect {
    Effect::GainControl {
        what: Sel::This,
        who,
        duration: Duration::Permanent,
    }
}

#[test]
fn control_of_the_source_lost_and_regained_during_the_resolution_lets_the_effect_begin() {
    cr!("611.2b");
    interp!("for-as-long-as-checked-continually-from-activation");
    let to_p1 = source_to(PlayerRef::Player(P1));
    let to_you = source_to(PlayerRef::You);
    for (first, regained, gained) in [
        // Lost and regained during the resolution: the duration "begins again during that
        // spell or ability's resolution", so the effect begins.
        (vec![to_p1.clone(), to_you.clone()], false, true),
        // Lost during the resolution and not regained: it does nothing.
        (vec![to_p1.clone()], false, false),
        // Lost while the ability waits, regained by the resolution itself: the duration
        // begins again during the resolution (CR 611.2b).
        (vec![to_you.clone()], true, true),
    ] {
        let mut t = TestGame::new(2);
        let satyr = t.custom(P0, satyr_doing_first(first), Zone::Battlefield);
        let ayumi = t.battlefield(P1, "Ayumi, the Last Visitor");
        activate(&mut t, P0, satyr, "Gain control", &[obj(ayumi)]);
        if regained {
            give_control(&mut t, satyr, P1);
        }
        t.resolve_all();
        let want = if gained { P0 } else { P1 };
        assert_eq!(
            controller(&t, ayumi),
            want,
            "regained={regained} gained={gained}"
        );
    }
}

#[test]
fn a_restriction_lasting_while_the_source_remains_tapped_and_more_is_checked_while_waiting() {
    cr!("611.2b");
    interp!("for-as-long-as-checked-continually-from-activation");
    // Rust Tick ("{1}, {T}: Tap target artifact. It doesn't untap during its controller's
    // untap step for as long as this creature remains tapped.") with its duration written
    // as "for as long as this creature remains tapped and [a condition that always holds]":
    // untapped and tapped again while the ability waits, the artifact untaps as usual.
    fn with_and(e: &mut Effect) {
        match e {
            Effect::Seq(v) => v.iter_mut().for_each(with_and),
            Effect::AddRestriction { duration, .. } => {
                if let Duration::WhileCondition(c) = duration {
                    *c = Condition::And(vec![c.clone(), Condition::Always]);
                }
            }
            _ => {}
        }
    }
    let mut def = (*card("Rust Tick")).clone();
    def.name = "Rust Tick Probe".into();
    def.faces[0].chars.name = "Rust Tick Probe".into();
    for a in def.faces[0].chars.abilities.iter_mut() {
        if let AbilityKind::Activated(act) = &mut std::sync::Arc::make_mut(a).kind {
            with_and(&mut act.body.effect);
        }
    }
    for retapped in [false, true] {
        let mut t = TestGame::new(2);
        let tick = t.custom(P0, def.clone(), Zone::Battlefield);
        let thopter = t.battlefield(P1, "Ornithopter");
        t.lands(P0, "Wastes", 1);
        activate(&mut t, P0, tick, "Tap target artifact", &[obj(thopter)]);
        if retapped {
            untap(&mut t, tick);
            tap(&mut t, tick);
        }
        t.resolve_all();
        assert!(is_tapped(&t, thopter));
        next_main(&mut t, P1);
        assert_eq!(is_tapped(&t, thopter), !retapped, "retapped={retapped}");
    }
}
