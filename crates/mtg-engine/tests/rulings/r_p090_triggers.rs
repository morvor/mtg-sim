//! Rulings batch P090 — triggered abilities: Elesh Norn, Mother of Machines (CR 603.2d,
//! 603.6a), "enter tapped" for an opponent's permanents entering alongside (CR 614.12,
//! 614.1d), untap triggers waiting for the upkeep (CR 502.4, 503.1a), King Macar's
//! controller (CR 603.3a), and Doomed Necromancer's target chosen before its cost is paid
//! (CR 602.2b, 601.2c).

use crate::r_p108_common::two_headed_giant;
use crate::r_s01_common::*;
use crate::r_s04_common::ability_targets;
use crate::r_s06_common::give_control;
use crate::r_s24_common::{choose_creature_type, enter_together};
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn soul_warden_gains(t: &mut TestGame, p: PlayerId, enter_as: PlayerId, name: &str) -> i32 {
    let before = t.life(p);
    t.enter(enter_as, name);
    t.settle();
    t.resolve_all();
    t.life(p) - before
}

#[test]
fn elesh_norn_doesnt_look_at_who_controls_the_entering_permanent() {
    cr!("603.2d", "603.2");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Elesh Norn's abilities don't look at who controls the permanent entering the battlefield, only who controls the permanent that has the triggered ability. For example, if you control Elesh Norn and a permanent with the ability \"Whenever a creature enters the battlefield, you gain 1 life,\" a creature entering the battlefield under an opponent's control will cause that ability to trigger twice."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    t.battlefield(P0, "Soul Warden");
    assert_eq!(soul_warden_gains(&mut t, P0, P1, "Grizzly Bears"), 2);
    assert_eq!(soul_warden_gains(&mut t, P0, P0, "Grizzly Bears"), 2);
}

#[test]
fn elesh_norn_triggers_twice_with_separate_choices() {
    cr!("603.2d", "603.3d", "608.2d");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Elesh Norn's first ability doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Fire Imp");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    // Fire Imp: "When this creature enters, it deals 2 damage to target creature." One
    // instance at each creature.
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.enter(P0, "Fire Imp");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(
        t.g.stack
            .iter()
            .all(|s| !crate::r_s04_common::is_spell_copy(&t, *s)),
        "not copies"
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(crate::r_s07_common::damage_on(&t, b), 2);
    // A choice on resolution, made for each instance: Reclamation Sage ("you may destroy
    // target artifact or enchantment"): no once, yes once.
    supported("Reclamation Sage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    let x = t.battlefield(P1, "Ornithopter");
    let y = t.battlefield(P1, "Glorious Anthem");
    t.answer_targets(P0, &[Entity::Object(x)]);
    t.answer_targets(P0, &[Entity::Object(y)]);
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.enter(P0, "Reclamation Sage");
    t.settle();
    t.resolve_all();
    // The top instance (targeting the Anthem) resolved first: declined; then yes.
    let destroyed = usize::from(!t.on_battlefield(x)) + usize::from(!t.on_battlefield(y));
    assert_eq!(destroyed, 1, "{:?}\n{}", t.asked(), t.dump_log());
}

#[test]
fn elesh_norn_and_permanents_entering_with_it() {
    cr!("603.6a", "603.2d");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "If a permanent entering the battlefield at the same time as Elesh Norn (including Elesh Norn itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    // Elesh Norn and Grizzly Bears enter together: Soul Warden triggers twice for each.
    enter_together(
        &mut t,
        &[
            (P0, "Elesh Norn, Mother of Machines"),
            (P0, "Grizzly Bears"),
        ],
    );
    assert_eq!(t.stack_len(), 4, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn two_elesh_norns_trigger_three_times() {
    cr!("603.2d");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "If you somehow control two Elesh Norns, a permanent entering the battlefield causes abilities to trigger three times, not four. A third Elesh Norn causes abilities to trigger four times, a fourth causes abilities to trigger five times, and so on."
    );
    // (Mirror Gallery: "The \"legend rule\" doesn't apply.")
    supported("Mirror Gallery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    t.battlefield(P0, "Soul Warden");
    assert_eq!(soul_warden_gains(&mut t, P0, P0, "Grizzly Bears"), 3);
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    assert_eq!(soul_warden_gains(&mut t, P0, P0, "Grizzly Bears"), 4);
    assert_eq!(
        t.named_on_battlefield("Elesh Norn, Mother of Machines")
            .len(),
        3
    );
}

#[test]
fn elesh_norn_doesnt_affect_a_teammates_triggers() {
    cr!("603.2d", "102.3");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "In a Two-Headed Giant game, Elesh Norn does not affect triggered abilities of permanents your teammate controls."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    t.battlefield(P1, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    // One trigger: not doubled (a teammate's), and not stopped (not an opponent's).
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    // An opponent's (P2's) creature entering: P1's Soul Warden still triggers once.
    t.resolve_all();
    t.enter(P2, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn elesh_norn_leaves_as_enters_choices_and_replacements_alone() {
    cr!("614.12", "614.1c", "603.2d");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Abilities that apply \"as [this permanent] enters the battlefield,\" such as choosing a card type with Stenn, Paranoid Partisan, are also unaffected."
    );
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Replacement effects are unaffected by Elesh Norn's abilities. For example, a creature that enters the battlefield under your control with one +1/+1 counter on it won't receive an additional +1/+1 counter."
    );
    supported("Festercreep");
    for (norn, enters) in [(P0, P0), (P1, P0)] {
        let mut t = TestGame::new(2);
        t.battlefield(norn, "Elesh Norn, Mother of Machines");
        // Brass Herald: one choice, applied (the Bears get +1/+1).
        let bears = t.battlefield(enters, "Grizzly Bears");
        let from = t.asked().len();
        choose_creature_type(&mut t, enters, "Bear");
        t.enter(enters, "Brass Herald");
        t.g.recompute();
        let choices = crate::r_s35_common::creature_type_options(&t, enters, from).len();
        assert_eq!(choices, 1);
        assert_eq!(t.pt(bears), (3, 3));
        // Festercreep enters with one +1/+1 counter, not two.
        let f = t.enter(enters, "Festercreep");
        assert_eq!(t.counters(f, "+1/+1"), 1);
    }
}

#[test]
fn kinjallis_sunwing_and_manglehorn_dont_tap_what_enters_alongside() {
    cr!("614.12", "614.1d");
    ruling!(
        "Kinjalli's Sunwing",
        "If a creature an opponent controls enters the battlefield at the same time that Kinjalli's Sunwing enters the battlefield under your control, Kinjalli's Sunwing's effect doesn't apply to your opponent's creature."
    );
    ruling!(
        "Manglehorn",
        "If an artifact enters the battlefield under an opponent’s control at the same time that Manglehorn enters the battlefield under your control, that artifact won’t enter tapped."
    );
    for (name, other) in [
        ("Kinjalli's Sunwing", "Grizzly Bears"),
        ("Manglehorn", "Ornithopter"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let ids = enter_together(&mut t, &[(P0, name), (P1, other)]);
        assert!(!t.obj_now(ids[1]).tapped, "{name}");
        // Positive control: one entering afterwards is tapped.
        let later = t.enter(P1, other);
        assert!(t.obj_now(later).tapped, "{name}");
    }
}

#[test]
fn wake_thrasher_untap_step_triggers_wait_for_upkeep() {
    cr!("502.4", "503.1a", "603.3b");
    ruling!(
        "Wake Thrasher",
        "If permanents you control become untapped during your untap step, Wake Thresher’s ability will trigger that many times. However, since no player gets priority during the untap step, those abilities wait to be put on the stack until your upkeep starts. At that time, all your “beginning of upkeep” triggers will also trigger. You can put them and Wake Thresher’s abilities on the stack in any order."
    );
    ruling!(
        "Well Rested",
        "If the ability granted by Well Rested triggers in an untap step, it goes onto the stack at the beginning of the next step (usually the upkeep)."
    );
    supported("Wake Thrasher");
    supported("Well Rested");
    supported("Phyrexian Arena");
    let mut t = TestGame::new(2);
    let thrasher = t.battlefield(P0, "Wake Thrasher");
    t.battlefield(P0, "Phyrexian Arena");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Well Rested", bears);
    for l in t.lands(P0, "Forest", 2) {
        t.g.tap(l);
    }
    t.g.tap(bears);
    t.advance_to(P1, Step::End);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.stack_len(), 0, "nothing on the stack in the untap step");
    t.settle();
    // Three Wake Thrasher triggers (two Forests, the Bears), Well Rested's, Phyrexian
    // Arena's.
    assert_eq!(t.stack_len(), 5, "{}", t.dump_log());
    let orders: Vec<_> = asked_since(&t, from)
        .into_iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. }))
        .collect();
    assert!(!orders.is_empty(), "P0 orders the triggers");
    let hand = t.hand_size(P0);
    t.resolve_all();
    // Wake Thrasher got +1/+1 three times, once for each permanent untapped.
    assert_eq!(t.pt(thrasher), (4, 4));
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn king_macars_token_goes_to_the_controller_of_the_trigger() {
    cr!("603.3a", "111.2");
    ruling!(
        "King Macar, the Gold-Cursed",
        "The player who controlled King Macar when its ability triggered will control the artifact token, not the player who controlled the creature that was exiled."
    );
    supported("King Macar, the Gold-Cursed");
    let mut t = TestGame::new(2);
    let macar = t.battlefield(P0, "King Macar, the Gold-Cursed");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(macar);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.g.untap(macar);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // P1 gains control of King Macar before the ability resolves.
    give_control(&mut t, macar, P1);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile, "{}", t.dump_log());
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Gold").len(),
        1
    );
    assert!(crate::r_s05_common::tokens_with_subtype(&t, P1, "Gold").is_empty());
}

#[test]
fn doomed_necromancer_cant_target_itself() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Doomed Necromancer",
        "Because targets are chosen before costs are paid, Doomed Necromancer can't be the target of its own ability."
    );
    supported("Doomed Necromancer");
    let mut t = TestGame::new(2);
    let nec = t.battlefield(P0, "Doomed Necromancer");
    // No creature card in the graveyard: it can't be activated at all.
    t.lands(P0, "Swamp", 1);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, nec));
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(crate::r_s02_common::can_activate(&mut t, P0, nec));
    let cands = ability_targets(&mut t, nec, 0);
    assert_eq!(cands, vec![Entity::Object(bears)]);
}
