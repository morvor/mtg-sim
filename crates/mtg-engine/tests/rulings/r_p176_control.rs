//! Rulings batch P176 — control-changing effects: "for as long as you control ~" steals
//! that never begin if the source left or changed controllers first (Dragonlord Silumgar,
//! Mind Flayer, Roil Elemental, Willbreaker, Sower of Temptation, Vedalken Shackles),
//! Merieke Ri Berit's destroy trigger, Captivating Glance's clash, Auras that steal
//! (Enthralling Hold, Enslave, Abduction, Threads of Disloyalty, Illusory Gains,
//! Rootwater Matriarch), exchanges and partially-legal targets (Kitsune, Bond of Passion,
//! Press into Service), Call for Aid's sacrifice restriction and Unholy Indenture.

use crate::r_p176_common::*;
use crate::r_s24_common::enter_together;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------
// "For as long as you control ~" (CR 611.2b): the duration must hold as it begins.
// ---------------------------------------------------------------------------------

/// The ETB trigger of `name` (targeting P1's Grizzly Bears) is on the stack; `meddle`
/// runs before it resolves. Returns (the source, the Bears).
fn etb_steal(name: &str, meddle: fn(&mut TestGame, ObjectId)) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let src = enter_and_trigger(&mut t, P0, name);
    assert_eq!(t.stack_len(), 1, "{name}'s trigger is on the stack");
    meddle(&mut t, src);
    t.resolve_all();
    (t, src, bears)
}

#[test]
fn etb_steals_do_nothing_if_the_source_left_or_changed_controllers_first() {
    cr!("611.2b", "603.2");
    ruling!(
        "Dragonlord Silumgar",
        "If Dragonlord Silumgar ceases to be under your control before its ability resolves, you won't gain control of the creature or planeswalker at all."
    );
    ruling!(
        "Mind Flayer",
        "If Mind Flayer leaves the battlefield or you lose control of it before its Dominate Monster ability resolves, control of the target creature doesn't change at all."
    );
    ruling!(
        "Sower of Temptation",
        "If Sower of Temptation leaves the battlefield before the ability resolves, the ability will have no effect."
    );
    for name in ["Dragonlord Silumgar", "Mind Flayer", "Sower of Temptation"] {
        // Positive control: nothing meddles, P0 takes the Bears.
        let (mut t, _, bears) = etb_steal(name, |_, _| {});
        assert_eq!(controller(&mut t, bears), P0, "{name}");
        // The source leaves the battlefield first.
        let (mut t, _, bears) = etb_steal(name, |t, src| destroy(t, src));
        assert_eq!(controller(&mut t, bears), P1, "{name}: source left");
    }
    for name in ["Dragonlord Silumgar", "Mind Flayer"] {
        // P1 gains control of the source first; even regaining it doesn't help.
        let (mut t, src, bears) = etb_steal(name, |t, src| give_control(t, src, P1));
        assert_eq!(controller(&mut t, bears), P1, "{name}: lost control first");
        give_control(&mut t, src, P0);
        assert_eq!(controller(&mut t, bears), P1, "{name}");
    }
}

#[test]
fn etb_steals_end_when_the_source_leaves_or_changes_controllers() {
    cr!("611.2b");
    ruling!(
        "Dragonlord Silumgar",
        "If Dragonlord Silumgar leaves the battlefield, you no longer control it, and its control-change effect ends."
    );
    ruling!(
        "Dragonlord Silumgar",
        "If another player gains control of Dragonlord Silumgar, its control-change effect ends. Regaining control of Dragonlord Silumgar won't cause you to regain control of the creature or planeswalker."
    );
    for name in ["Dragonlord Silumgar", "Mind Flayer"] {
        let (mut t, src, bears) = etb_steal(name, |_, _| {});
        assert_eq!(controller(&mut t, bears), P0);
        destroy(&mut t, src);
        assert_eq!(controller(&mut t, bears), P1, "{name} left");

        let (mut t, src, bears) = etb_steal(name, |_, _| {});
        assert_eq!(controller(&mut t, bears), P0);
        give_control(&mut t, src, P1);
        assert_eq!(controller(&mut t, bears), P1, "{name} changed controllers");
        give_control(&mut t, src, P0);
        assert_eq!(
            controller(&mut t, bears),
            P1,
            "regaining {name} doesn't help"
        );
    }
}

/// P0 controls Roil Elemental and plays a Forest: the landfall trigger (targeting P1's
/// Bears, P0 choosing to gain control) goes on the stack; `meddle` runs before it resolves.
fn roil(meddle: fn(&mut TestGame, ObjectId)) -> (TestGame, ObjectId, ObjectId) {
    supported("Roil Elemental");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let roil = t.battlefield(P0, "Roil Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.hand(P0, "Forest");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    t.play_land(P0, forest).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "landfall triggered");
    meddle(&mut t, roil);
    t.resolve_all();
    (t, roil, bears)
}

#[test]
fn roil_elemental_needs_to_stay_under_your_control() {
    cr!("611.2b", "603.2");
    ruling!(
        "Roil Elemental",
        "If Roil Elemental ceases to be under your control before its ability resolves, you won't gain control of the targeted creature at all."
    );
    ruling!(
        "Roil Elemental",
        "If Roil Elemental leaves the battlefield, you no longer control it, so all of its control-change effects end."
    );
    let (mut t, roil_id, bears) = roil(|_, _| {});
    assert_eq!(controller(&mut t, bears), P0);
    destroy(&mut t, roil_id);
    assert_eq!(controller(&mut t, bears), P1, "Roil left: the effect ended");
    let (mut t, _, bears) = roil(|t, r| give_control(t, r, P1));
    assert_eq!(
        controller(&mut t, bears),
        P1,
        "Roil changed controllers first"
    );
}

/// P0 controls Willbreaker and casts Giant Growth at P1's Bears: Willbreaker's trigger is
/// on the stack above it; `meddle` runs before they resolve.
fn willbreaker(meddle: fn(&mut TestGame, ObjectId)) -> (TestGame, ObjectId, ObjectId) {
    supported("Willbreaker");
    let mut t = TestGame::new(2);
    let wb = t.battlefield(P0, "Willbreaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_only(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "Willbreaker triggered");
    meddle(&mut t, wb);
    t.resolve_all();
    (t, wb, bears)
}

#[test]
fn willbreaker_control_lasts_only_while_you_control_it() {
    cr!("611.2b", "603.2");
    ruling!(
        "Willbreaker",
        "If you lose control of Willbreaker before its ability resolves, you won't gain control of the creature at all."
    );
    ruling!(
        "Willbreaker",
        "If Willbreaker leaves the battlefield, you no longer control it, and its control-change effect ends."
    );
    ruling!(
        "Willbreaker",
        "If another player gains control of Willbreaker, its control-change effect ends. Regaining control of Willbreaker won’t cause you to regain control of the creature."
    );
    let (mut t, wb, bears) = willbreaker(|_, _| {});
    assert_eq!(controller(&mut t, bears), P0);
    destroy(&mut t, wb);
    assert_eq!(controller(&mut t, bears), P1, "Willbreaker left");

    let (mut t, wb, bears) = willbreaker(|_, _| {});
    assert_eq!(controller(&mut t, bears), P0);
    give_control(&mut t, wb, P1);
    assert_eq!(
        controller(&mut t, bears),
        P1,
        "Willbreaker changed controllers"
    );
    give_control(&mut t, wb, P0);
    assert_eq!(
        controller(&mut t, bears),
        P1,
        "regaining Willbreaker doesn't help"
    );

    let (mut t, _, bears) = willbreaker(|t, wb| give_control(t, wb, P1));
    assert_eq!(
        controller(&mut t, bears),
        P1,
        "lost Willbreaker before resolution"
    );
}

/// P0 activates Vedalken Shackles (two Islands) at P1's Bears; `meddle` runs before the
/// ability resolves.
fn shackles(meddle: fn(&mut TestGame, ObjectId)) -> (TestGame, ObjectId, ObjectId) {
    supported("Vedalken Shackles");
    let mut t = TestGame::new(2);
    let sh = t.battlefield(P0, "Vedalken Shackles");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, sh, 0, &[obj(bears)]).unwrap();
    meddle(&mut t, sh);
    t.resolve_all();
    (t, sh, bears)
}

#[test]
fn vedalken_shackles_must_stay_tapped() {
    cr!("611.2b", "400.7");
    ruling!(
        "Vedalken Shackles",
        "If Vedalken Shackles becomes untapped before its ability resolves, you won’t gain control of the creature."
    );
    ruling!(
        "Vedalken Shackles",
        "If Vedalken Shackles leaves the battlefield, it’s no longer tapped, so the control-changing effect ends."
    );
    let (mut t, sh, bears) = shackles(|_, _| {});
    assert_eq!(controller(&mut t, bears), P0);
    destroy(&mut t, sh);
    assert_eq!(controller(&mut t, bears), P1, "the Shackles left");
    let (mut t, _, bears) = shackles(|t, sh| {
        t.g.untap(sh);
    });
    assert_eq!(controller(&mut t, bears), P1, "untapped before resolution");
}

// ---------------------------------------------------------------------------------
// Merieke Ri Berit
// ---------------------------------------------------------------------------------

/// P0 activates Merieke Ri Berit at P1's Bears and it resolves.
fn merieke() -> (TestGame, ObjectId, ObjectId) {
    supported("Merieke Ri Berit");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Merieke Ri Berit");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, m, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    (t, m, bears)
}

#[test]
fn merieke_untapping_destroys_the_creature_while_you_still_control_it() {
    cr!("611.2b", "603.2");
    ruling!(
        "Merieke Ri Berit",
        "If Merieke becomes untapped, its “destroy” ability will trigger, but you won’t lose control of the creature you took before it’s destroyed."
    );
    let (mut t, m, bears) = merieke();
    t.g.untap(m);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "the destroy ability triggered");
    assert_eq!(controller(&mut t, bears), P0, "still yours");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1), "destroyed");
}

#[test]
fn merieke_leaving_returns_the_creature_then_destroys_it() {
    cr!("611.2b", "603.6c");
    ruling!(
        "Merieke Ri Berit",
        "If Merieke leaves the battlefield, you will immediately lose control of the creature you took and Merieke’s “destroy” ability will trigger. The affected creature will go back under the control of the appropriate player, then it will be destroyed."
    );
    let (mut t, m, bears) = merieke();
    destroy(&mut t, m);
    assert_eq!(t.stack_len(), 1, "the destroy ability triggered");
    assert_eq!(controller(&mut t, bears), P1, "back under P1's control");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1), "destroyed");
}

#[test]
fn merieke_changing_controllers_returns_the_creature_without_a_trigger() {
    cr!("611.2b");
    ruling!(
        "Merieke Ri Berit",
        "If another player takes control of Merieke, you will immediately lose control of the creature you took, but Merieke’s “destroy” ability won’t trigger."
    );
    let (mut t, m, bears) = merieke();
    give_control(&mut t, m, P1);
    assert_eq!(controller(&mut t, bears), P1);
    assert_eq!(t.stack_len(), 0, "no destroy trigger");
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------
// Captivating Glance
// ---------------------------------------------------------------------------------

/// P0 controls Captivating Glance on P1's Bears; the clash at P0's end step is won by P0
/// if `p0_wins` (P0 reveals Hill Giant, P1 an Island) and lost otherwise (the reverse).
fn glance(p0_wins: bool) -> (TestGame, ObjectId, ObjectId) {
    supported("Captivating Glance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let glance = attach_new(&mut t, P0, "Captivating Glance", bears);
    let (mine, theirs) = if p0_wins {
        ("Hill Giant", "Island")
    } else {
        ("Island", "Hill Giant")
    };
    t.library_top(P0, mine);
    t.library_top(P1, theirs);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    (t, glance, bears)
}

#[test]
fn captivating_glance_control_outlasts_it_and_never_changes_its_controller() {
    cr!("701.30a", "611.2a");
    ruling!(
        "Captivating Glance",
        "Captivating Glance's control-change effect continues to apply even if Captivating Glance leaves the battlefield."
    );
    ruling!(
        "Captivating Glance",
        "Captivating Glance's controller never changes as a result of this card."
    );
    ruling!(
        "Captivating Glance",
        "Captivating Glance may cause a player to gain control of a creature they already control."
    );
    // P0 wins: P0 gains control of the Bears, which stays after the Glance is destroyed.
    let (mut t, g, bears) = glance(true);
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(controller(&mut t, g), P0);
    destroy(&mut t, g);
    assert_eq!(controller(&mut t, bears), P0, "outlasts the Aura");
    // P0 loses: P1 "gains control" of the Bears it already controls; P0 keeps the Glance.
    let (mut t, g, bears) = glance(false);
    assert_eq!(controller(&mut t, bears), P1);
    assert_eq!(controller(&mut t, g), P0);
    assert!(t.on_battlefield(g));
}

// ---------------------------------------------------------------------------------
// Auras that steal
// ---------------------------------------------------------------------------------

/// P1's tapped Grizzly Bears wears P1's Bonesplitter and Spirit Link; P0 enchants it with
/// the stealing Aura `name`. Returns (game, Bears, Bonesplitter, Spirit Link).
fn steal_equipped(name: &str) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let splitter = t.battlefield(P1, "Bonesplitter");
    t.g.attach(splitter, Entity::Object(bears));
    let link = attach_new(&mut t, P1, "Spirit Link", bears);
    t.g.tap(bears);
    cast_resolve(&mut t, P0, name, &[obj(bears)]);
    assert_eq!(controller(&mut t, bears), P0, "{name} stole the Bears");
    (t, bears, splitter, link)
}

#[test]
fn stealing_a_creature_doesnt_steal_its_auras_or_equipment() {
    cr!("301.5d", "303.4e", "702.6a");
    ruling!(
        "Enthralling Hold",
        "Gaining control of a permanent doesn't cause you to gain control of any Auras or Equipment attached to it."
    );
    ruling!(
        "Enslave",
        "Gaining control of a permanent doesn't cause you to gain control of any Auras or Equipment attached to it. They'll remain attached, but their abilities that refer to \"you\" still refer to its controller rather than you."
    );
    for name in ["Enthralling Hold", "Enslave"] {
        let (mut t, bears, splitter, link) = steal_equipped(name);
        assert_eq!(controller(&mut t, splitter), P1, "{name}");
        assert_eq!(controller(&mut t, link), P1, "{name}");
        assert_eq!(attached_to(&t, splitter), Some(obj(bears)));
        assert_eq!(attached_to(&t, link), Some(obj(bears)));
        assert_eq!(t.pt(bears), (4, 2), "still equipped");
        // Spirit Link's "you" is still P1, its controller.
        let life = (t.life(P0), t.life(P1));
        let giant = t.battlefield(P1, "Hill Giant");
        crate::r_s06_common::damage(&mut t, bears, 2, giant);
        t.resolve_all();
        assert_eq!(t.life(P1), life.1 + 2, "{name}: P1 gains the life");
        assert_eq!(t.life(P0), life.0);
        // P1 moves the Bonesplitter back to one of their creatures in their main phase.
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Wastes", 1);
        t.answer_targets(P1, &[obj(giant)]);
        let r = activate_containing(&mut t, P1, splitter, "Equip");
        assert!(r.is_ok(), "{name}: {r:?}");
        t.resolve_all();
        assert_eq!(attached_to(&t, splitter), Some(obj(giant)), "{name}");
    }
}

#[test]
fn abduction_can_target_an_untapped_creature() {
    cr!("303.4a", "115.1");
    ruling!(
        "Abduction",
        "Abduction may be cast targeting an untapped creature."
    );
    supported("Abduction");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!tapped(&t, bears));
    cast_resolve(&mut t, P0, "Abduction", &[obj(bears)]);
    assert_eq!(controller(&mut t, bears), P0);
    assert!(!tapped(&t, bears));
}

#[test]
fn rootwater_matriarch_can_target_an_unenchanted_creature_to_no_effect() {
    cr!("611.2b", "115.1");
    ruling!(
        "Rootwater Matriarch",
        "Can target an unenchanted creature but has no effect on that creature."
    );
    supported("Rootwater Matriarch");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Rootwater Matriarch");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, m, 0, &[obj(bears)]).expect("legal target");
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P1, "no effect");
    // Positive control: an enchanted creature is stolen.
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P1, "Holy Strength", giant);
    t.g.untap(m);
    t.activate(P0, m, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, giant), P0);
}

#[test]
fn threads_of_disloyalty_counts_x_as_zero_on_the_battlefield() {
    cr!("202.3e", "303.4a");
    ruling!(
        "Threads of Disloyalty",
        "Creatures on the battlefield treat {X} in their mana cost as 0 for the purposes of calculating mana values."
    );
    supported("Threads of Disloyalty");
    // P1 casts Hydroid Krasis ({X}{G}{U}) with X = 3: on the battlefield its mana value is 2.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Island", 1);
    t.lands(P1, "Wastes", 3);
    let krasis = t.hand(P1, "Hydroid Krasis");
    t.cast(P1, krasis).x(3).go();
    t.resolve_all();
    let krasis = t.g.current(krasis);
    assert!(t.on_battlefield(krasis));
    assert_eq!(t.counters(krasis, "+1/+1"), 3);
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Threads of Disloyalty", &[obj(krasis)]);
    assert_eq!(controller(&mut t, krasis), P0);
}

#[test]
fn illusory_gains_moves_for_each_creature_and_must_move() {
    cr!("603.3b", "701.3a");
    ruling!(
        "Illusory Gains",
        "If multiple creatures enter the battlefield under an opponent's control at the same time, Illusory Gains will trigger for each of them."
    );
    ruling!(
        "Illusory Gains",
        "Illusory Gains's last ability is mandatory. You can't choose to have Illusory Gains not move."
    );
    supported("Illusory Gains");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gains = attach_new(&mut t, P0, "Illusory Gains", bears);
    assert_eq!(controller(&mut t, bears), P0);
    t.answer_yes(P0, false);
    let from = t.asked().len();
    let ids = enter_together(&mut t, &[(P1, "Hill Giant"), (P1, "Serra Angel")]);
    assert_eq!(t.stack_len(), 2, "one trigger per creature");
    t.resolve_all();
    assert!(
        !t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })),
        "not optional"
    );
    let on = attached_to(&t, gains).expect("attached");
    let mine: Vec<ObjectId> = ids
        .iter()
        .copied()
        .filter(|id| controller(&mut t, *id) == P0)
        .collect();
    assert_eq!(mine.len(), 1, "P0 ends up with exactly one of them");
    assert_eq!(on, obj(mine[0]));
    assert_eq!(controller(&mut t, bears), P1);
}

// ---------------------------------------------------------------------------------
// Exchanges and partially legal targets (CR 608.2b)
// ---------------------------------------------------------------------------------

#[test]
fn kitsune_exchange_needs_both_targets() {
    cr!("701.12b", "608.2b");
    ruling!(
        "Kitsune, Dragon's Daughter",
        "If one of the targets is an illegal target when Kitsune's last ability resolves, the exchange won't happen."
    );
    supported("Kitsune, Dragon's Daughter");
    for kill in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let angel = t.battlefield(P1, "Serra Angel");
        t.answer_targets(P0, &[obj(bears), obj(giant)]);
        t.answer_yes(P0, true);
        enter_and_trigger(&mut t, P0, "Kitsune, Dragon's Daughter");
        assert_eq!(t.stack_len(), 1);
        if kill {
            destroy(&mut t, giant);
        }
        t.resolve_all();
        let _ = angel;
        if kill {
            assert_eq!(controller(&mut t, bears), P0, "no exchange");
        } else {
            assert_eq!(controller(&mut t, bears), P1, "exchanged");
            assert_eq!(controller(&mut t, giant), P0, "exchanged");
        }
    }
}

#[test]
fn bond_of_passion_affects_the_target_that_is_still_legal() {
    cr!("608.2b");
    ruling!(
        "Bond of Passion",
        "If either target becomes illegal after Bond of Passion is cast but before it resolves, the other is still affected as appropriate."
    );
    supported("Bond of Passion");
    // The creature becomes illegal: the damage is still dealt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_only(
        &mut t,
        P0,
        "Bond of Passion",
        &[obj(bears), Entity::Player(P1)],
    );
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // The damage target becomes illegal: P0 still gains control of the creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(bears);
    cast_only(&mut t, P0, "Bond of Passion", &[obj(bears), obj(giant)]);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    assert!(!tapped(&t, bears), "untapped");
}

#[test]
fn press_into_service_affects_only_its_legal_targets() {
    cr!("608.2b", "701.41a");
    ruling!(
        "Press into Service",
        "As Press into Service resolves, if at least one of its targets is still legal, it will resolve, affecting only targets that are still legal at that time."
    );
    supported("Press into Service");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    give_mana_for(&mut t, P0, "Press into Service");
    let card = t.hand(P0, "Press into Service");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(angel)]);
    t.g.turn.priority = Some(P0);
    t.cast(P0, card).go();
    // The creature to steal becomes illegal: the support counter is still put on.
    destroy(&mut t, angel);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(controller(&mut t, giant), P1);
    assert!(t.in_graveyard(P0, "Press into Service"));
}

// ---------------------------------------------------------------------------------
// Call for Aid
// ---------------------------------------------------------------------------------

#[test]
fn call_for_aid_creatures_cant_be_sacrificed_so_you_sacrifice_your_own() {
    cr!("701.21a", "101.3");
    ruling!(
        "Call for Aid",
        "If an effect instructs you to sacrifice a creature, and you control any creatures other than the ones you gained control of with Call for Aid this turn, you must sacrifice one of them."
    );
    supported("Call for Aid");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Call for Aid", &[Entity::Player(P1)]);
    assert_eq!(controller(&mut t, giant), P0);
    // P1's Diabolic Edict: P0 tries to sacrifice the Giant.
    t.answer_choose(P0, &[obj(giant)]);
    cast_resolve(&mut t, P1, "Diabolic Edict", &[Entity::Player(P0)]);
    assert!(
        t.in_graveyard(P0, "Grizzly Bears"),
        "P0's own creature was sacrificed"
    );
    assert!(t.on_battlefield(giant));
    // With only the stolen creature left, another Edict does nothing.
    cast_resolve(&mut t, P1, "Diabolic Edict", &[Entity::Player(P0)]);
    assert!(t.on_battlefield(giant));
    let _ = bears;
}

// ---------------------------------------------------------------------------------
// Unholy Indenture
// ---------------------------------------------------------------------------------

#[test]
fn unholy_indenture_two_auras_last_player_in_turn_order_gets_it() {
    cr!("101.4", "603.3b", "400.7");
    ruling!(
        "Unholy Indenture",
        "If multiple players each control an Unholy Indenture attached to one creature, the player who will control the returned creature is the last player in turn order from the player whose turn it is at the time the creature dies."
    );
    supported("Unholy Indenture");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Unholy Indenture", bears);
    attach_new(&mut t, P1, "Unholy Indenture", bears);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "returned once");
    assert_eq!(controller(&mut t, back[0]), P1, "the non-active player");
    assert_eq!(t.counters(back[0], "+1/+1"), 1);
}

#[test]
fn unholy_indenture_adds_its_counter_to_the_creatures_own() {
    cr!("614.1c", "122.6");
    ruling!(
        "Unholy Indenture",
        "If the returned creature enters with any other counters, it enters with those counters plus an additional +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let feeder = t.battlefield(P1, "Spike Feeder");
    attach_new(&mut t, P0, "Unholy Indenture", feeder);
    destroy(&mut t, feeder);
    t.resolve_all();
    let back = t.named_on_battlefield("Spike Feeder");
    assert_eq!(back.len(), 1);
    assert_eq!(controller(&mut t, back[0]), P0);
    assert_eq!(t.counters(back[0], "+1/+1"), 3);
}
