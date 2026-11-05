//! Rulings batch P027 — a permanent that becomes a copy of something is neither entering
//! nor leaving the battlefield (CR 707.2, 603.6a, 603.6c, 614.1c): the copied "enters"
//! abilities don't trigger or apply, nothing else's "whenever a creature enters" triggers,
//! and the form it had doesn't "leave". A triggered ability that may make it a copy
//! targets as it's put on the stack and asks as it resolves (CR 603.3d, 603.5, 608.2).

use crate::r_p020_common::RIFTWATCHER;
use crate::r_p027_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{cast_new, targets_of};
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Asserts `id` is now a copy of Aven Riftwatcher that didn't enter the battlefield: its
/// "enters with" ability (vanishing 3) put no time counters on it, and neither its
/// "enters" trigger (2 life) nor Soul Warden (1 life) gained P0 life since `life`.
fn became_riftwatcher(t: &TestGame, id: ObjectId, life: i32) {
    assert_eq!(name_of(t, id), RIFTWATCHER, "{id:?} isn't a copy");
    assert_eq!(
        t.counters(id, counters::TIME),
        0,
        "it 'entered' with counters"
    );
    assert_eq!(t.life(P0), life, "an enters ability triggered");
    assert_eq!(t.stack_len(), 0);
}

/// P0's Soul Warden ("Whenever another creature enters, you gain 1 life.") and an
/// enchantment with "Whenever a creature leaves the battlefield, you gain 1 life.": a
/// permanent that becomes a copy triggers neither.
fn soul_warden(t: &mut TestGame) {
    supported("Soul Warden");
    t.battlefield(P0, "Soul Warden");
    let watcher = crate::r_s01_common::custom_card(
        "Departure Watch",
        "Enchantment",
        "{1}{W}",
        None,
        "Whenever a creature leaves the battlefield, you gain 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
}

/// Advances to P1's upkeep: P0's "until end of turn" effects have ended (CR 514.2).
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
}

/// The YesNo decisions P0 was asked since `from`.
fn yes_no_asked(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
        .count()
}

/// P0 casts Giant Growth on Artisan of Forms (heroic: "you may have this creature become
/// a copy of target creature, except it has this ability"), the trigger targeting `what`.
/// Returns with the trigger on the stack above Giant Growth.
fn heroic_artisan(t: &mut TestGame, artisan: ObjectId, what: ObjectId) {
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    let artisan = t.g.current(artisan);
    t.cast(P0, gg).target(obj(artisan)).go();
    t.answer_targets(P0, &[obj(what)]);
    t.settle();
}

#[test]
fn artisan_of_forms_doesnt_enter_or_leave_as_it_changes() {
    cr!("707.2", "603.6a", "603.6c", "614.1c");
    ruling!("Artisan of Forms", "When Artisan of Forms becomes a copy of a creature, it's neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won't trigger.");
    supported("Artisan of Forms");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let artisan = t.battlefield(P0, "Artisan of Forms");
    let rift = t.battlefield(P1, RIFTWATCHER);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let life = t.life(P0);
    heroic_artisan(&mut t, artisan, rift);
    t.answer_yes(P0, true);
    t.resolve_all();
    became_riftwatcher(&t, artisan, life);
    // As a Riftwatcher ("When this creature enters or leaves the battlefield, you gain 2
    // life") with heroic, it becomes a Grizzly Bears: the Riftwatcher didn't leave.
    heroic_artisan(&mut t, artisan, bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(name_of(&t, artisan), "Grizzly Bears");
    assert_eq!(t.life(P0), life);
}

#[test]
fn artisan_of_forms_targets_on_the_stack_and_chooses_on_resolution() {
    cr!("603.3d", "603.5");
    ruling!("Artisan of Forms", "You choose the target for the triggered ability when the ability is put onto the stack. You choose whether or not Artisan of Forms becomes a copy of that creature when the ability resolves.");
    for yes in [false, true] {
        let mut t = TestGame::new(2);
        let artisan = t.battlefield(P0, "Artisan of Forms");
        let giant = t.battlefield(P1, "Hill Giant");
        let from = t.asked().len();
        heroic_artisan(&mut t, artisan, giant);
        // On the stack with its target, and nothing asked about copying yet.
        let trig = *t.g.stack.last().unwrap();
        assert_eq!(targets_of(&t, trig), vec![obj(giant)]);
        assert_eq!(yes_no_asked(&t, from), 0);
        t.answer_yes(P0, yes);
        t.resolve();
        assert_eq!(yes_no_asked(&t, from), 1);
        let want = if yes {
            "Hill Giant"
        } else {
            "Artisan of Forms"
        };
        assert_eq!(name_of(&t, artisan), want);
    }
}

#[test]
fn cryptoplasm_targets_on_the_stack_and_chooses_on_resolution() {
    cr!("603.3d", "603.5", "503.1a");
    ruling!("Cryptoplasm", "You choose the target for the triggered ability when the ability is put onto the stack. You choose whether or not Cryptoplasm becomes a copy of that creature when the ability resolves.");
    // "At the beginning of your upkeep, you may have this creature become a copy of
    // another target creature, except it has this ability."
    supported("Cryptoplasm");
    for yes in [false, true] {
        let mut t = TestGame::new(2);
        let plasm = t.battlefield(P0, "Cryptoplasm");
        let giant = t.battlefield(P1, "Hill Giant");
        t.set_step(P1, Step::End);
        t.answer_targets(P0, &[obj(giant)]);
        let from = t.asked().len();
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        let trig = *t.g.stack.last().expect("the upkeep trigger");
        assert_eq!(targets_of(&t, trig), vec![obj(giant)]);
        assert_eq!(yes_no_asked(&t, from), 0);
        t.answer_yes(P0, yes);
        t.resolve();
        assert_eq!(yes_no_asked(&t, from), 1);
        let want = if yes { "Hill Giant" } else { "Cryptoplasm" };
        assert_eq!(name_of(&t, plasm), want);
    }
}

#[test]
fn fleeting_reflection_target_doesnt_enter_or_leave() {
    cr!("707.2", "603.6a", "603.6c", "614.1c", "514.2");
    ruling!("Fleeting Reflection", "When the first target creature becomes a copy of the other target creature, it’s neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won’t trigger.");
    supported("Fleeting Reflection");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rift = t.battlefield(P1, RIFTWATCHER);
    let life = t.life(P0);
    cast_new(&mut t, P0, "Fleeting Reflection", &[obj(bears), obj(rift)]);
    t.resolve_all();
    became_riftwatcher(&t, bears, life);
    // At the end of the turn it stops being a Riftwatcher without "leaving".
    next_turn(&mut t);
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
    assert_eq!(t.life(P0), life);
}

#[test]
fn lazav_familiar_stranger_doesnt_enter_or_leave() {
    cr!("707.2", "603.6a", "603.6c", "614.1c", "700.13");
    ruling!("Lazav, Familiar Stranger", "When Lazav becomes a copy of an exiled creature card, it's neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won't trigger.");
    // "Whenever you commit a crime, put a +1/+1 counter on Lazav. Then you may exile a
    // card from a graveyard. If a creature card was exiled this way, you may have Lazav
    // become a copy of that card until end of turn."
    supported("Lazav, Familiar Stranger");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let lazav = t.battlefield(P0, "Lazav, Familiar Stranger");
    let card = t.graveyard(P1, RIFTWATCHER);
    let life = t.life(P0);
    // Shock targeting P1 is a crime.
    t.answer_choose_moved(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    became_riftwatcher(&t, lazav, life);
    assert_eq!(t.counters(lazav, counters::PLUS1), 1);
    assert!(t.in_exile(RIFTWATCHER));
    next_turn(&mut t);
    assert_eq!(name_of(&t, lazav), "Lazav, Familiar Stranger");
    assert_eq!(t.life(P0), life);
}

/// P0 casts Shock at P1, triggering Muddle ("Whenever you cast an instant or sorcery
/// spell, Muddle becomes a copy of up to one target nonlegendary creature you control
/// until end of turn, except it has myriad.") targeting `what`.
fn shock_for_muddle(t: &mut TestGame, what: ObjectId) {
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.answer_targets(P0, &[obj(what)]);
    t.settle();
}

#[test]
fn muddle_doesnt_enter_or_leave() {
    cr!("707.2", "603.6a", "603.6c", "614.1c", "514.2");
    ruling!("Muddle, the Ever-Changing", "When Muddle becomes a copy of the target creature, it's neither entering nor leaving the battlefield. Any enters abilities or leaves-the-battlefield abilities won't trigger.");
    supported("Muddle, the Ever-Changing");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let muddle = t.battlefield(P0, "Muddle, the Ever-Changing");
    let rift = t.battlefield(P0, RIFTWATCHER);
    let life = t.life(P0);
    shock_for_muddle(&mut t, rift);
    t.resolve_all();
    became_riftwatcher(&t, muddle, life);
    next_turn(&mut t);
    assert_eq!(name_of(&t, muddle), "Muddle, the Ever-Changing");
    assert_eq!(t.life(P0), life);
}

#[test]
fn muddle_as_a_copy_has_no_trigger_but_its_waiting_triggers_resolve() {
    cr!("707.2", "113.7a", "707.9a");
    ruling!("Muddle, the Ever-Changing", "While Muddle is a copy of something else, it won't have its own triggered ability. However, if Muddle still has abilities on the stack waiting to resolve, they will do so. (It won't get multiple instances of myriad this way.)");
    let mut t = TestGame::new(2);
    let muddle = t.battlefield(P0, "Muddle, the Ever-Changing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    // Two Shocks: two triggers, the second (on top) targeting the Hill Giant.
    shock_for_muddle(&mut t, bears);
    shock_for_muddle(&mut t, giant);
    t.resolve();
    assert_eq!(name_of(&t, muddle), "Hill Giant");
    // Muddle's other trigger still resolves.
    t.resolve_all();
    assert_eq!(name_of(&t, muddle), "Grizzly Bears");
    let myriads = t
        .obj_now(muddle)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Keyword(k) if k.kind == KeywordKind::Myriad))
        .count();
    assert_eq!(myriads, 1, "myriad instances");
    // As a Grizzly Bears it no longer has its own triggered ability.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 1, "Muddle triggered as a copy");
}

/// P0's Silent Hallcreeper attacks P1 unblocked, and its combat damage trigger ("choose
/// one that hasn't been chosen — ... • This creature becomes a copy of another target
/// creature you control.") chooses the copy mode, targeting `what`.
fn hallcreeper_copies(t: &mut TestGame, creeper: ObjectId, what: ObjectId) {
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.answer_targets(P0, &[obj(what)]);
    attack_with(t, &[(creeper, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
}

#[test]
fn silent_hallcreeper_doesnt_enter_or_leave() {
    cr!("707.2", "603.6a", "603.6c", "614.1c");
    ruling!("Silent Hallcreeper", "When Silent Hallcreeper becomes a copy of the target creature, it's neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won't trigger.");
    supported("Silent Hallcreeper");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let creeper = t.battlefield(P0, "Silent Hallcreeper");
    let rift = t.battlefield(P0, RIFTWATCHER);
    let life = t.life(P0);
    hallcreeper_copies(&mut t, creeper, rift);
    assert_eq!(t.life(P1), 19);
    became_riftwatcher(&t, creeper, life);
}

#[test]
fn silent_hallcreeper_as_a_copy_has_none_of_its_previous_abilities() {
    cr!("707.2", "613.1a");
    ruling!("Silent Hallcreeper", "While Silent Hallcreeper is a copy of another creature, it won't have any of its previous abilities.");
    let mut t = TestGame::new(2);
    let creeper = t.battlefield(P0, "Silent Hallcreeper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(abilities_with(&t, creeper, "can't be blocked"), 1);
    hallcreeper_copies(&mut t, creeper, bears);
    assert_eq!(name_of(&t, creeper), "Grizzly Bears");
    assert!(t.obj_now(creeper).chars.abilities.is_empty());
    // Dealing combat damage to a player triggers nothing now: no counters, no card
    // drawn, still a Grizzly Bears.
    let creeper = t.g.current(creeper);
    t.advance_to(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(creeper, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(creeper, counters::PLUS1), 0);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(name_of(&t, creeper), "Grizzly Bears");
    // And it can be blocked: they trade.
    t.advance_to(P0, Step::PrecombatMain);
    let wall = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(creeper, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(wall, creeper)]);
    assert!(!t.on_battlefield(wall));
    assert!(!t.on_battlefield(creeper));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn spirit_of_resilience_doesnt_enter_or_leave() {
    cr!("707.2", "603.6a", "603.6c", "614.1c", "514.2");
    ruling!("Spirit of Resilience", "When Spirit of Resilience becomes a copy of the chosen card, it's neither entering nor leaving the battlefield. Any enters abilities or leaves-the-battlefield abilities won't trigger.");
    // "Whenever one or more cards leave your graveyard, put a +1/+1 counter on this
    // creature, then you may have this creature become a copy of an artifact or creature
    // card from among those cards until end of turn."
    supported("Spirit of Resilience");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let spirit = t.battlefield(P0, "Spirit of Resilience");
    let card = t.graveyard(P0, RIFTWATCHER);
    let life = t.life(P0);
    t.answer_choose_moved(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    t.g.move_object(card, Zone::Exile, MoveCause::Effect, None)
        .unwrap();
    t.g.flush_events();
    t.resolve_all();
    became_riftwatcher(&t, spirit, life);
    assert_eq!(t.counters(spirit, counters::PLUS1), 1);
    next_turn(&mut t);
    assert_eq!(name_of(&t, spirit), "Spirit of Resilience");
    assert_eq!(t.life(P0), life);
}

#[test]
fn the_ever_changing_dane_doesnt_enter() {
    cr!("707.2", "603.6a", "603.6c", "614.1c", "603.10a");
    ruling!("The Ever-Changing 'Dane", "When The Ever-Changing ’Dane becomes a copy of a creature, it’s neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won’t trigger.");
    // "{1}, Sacrifice another creature: The Ever-Changing 'Dane becomes a copy of the
    // sacrificed creature, except it has this ability."
    supported("The Ever-Changing 'Dane");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let dane = t.battlefield(P0, "The Ever-Changing 'Dane");
    let rift = t.battlefield(P0, RIFTWATCHER);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let life = t.life(P0);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(rift)]);
    activate_containing(&mut t, P0, dane, "Sacrifice another").expect("activation");
    t.resolve_all();
    // Only the sacrificed Riftwatcher's own "leaves the battlefield" trigger.
    // (And the watcher: the Riftwatcher left.)
    became_riftwatcher(&t, dane, life + 3);
    // As a Riftwatcher with the 'Dane's ability, it becomes a Grizzly Bears: the
    // Riftwatcher it was doesn't leave.
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, dane, "Sacrifice another").expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, dane), "Grizzly Bears");
    // Only the sacrificed Bears left (the watcher's 1 life); the 'Dane's Riftwatcher form
    // didn't (no 2 life from its own ability, no second 1 from the watcher).
    assert_eq!(t.life(P0), life + 4);
}

#[test]
fn zygon_infiltrator_doesnt_enter_or_leave() {
    cr!("707.2", "603.6a", "603.6c", "614.1c", "611.2b");
    ruling!("Zygon Infiltrator", "When Zygon Infiltrator becomes a copy of a creature, it's neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won't trigger.");
    // "Body-print — {2}{U}: Tap another target creature and put a stun counter on it.
    // This creature becomes a copy of that creature for as long as that creature remains
    // tapped."
    supported("Zygon Infiltrator");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let zygon = t.battlefield(P0, "Zygon Infiltrator");
    let rift = t.battlefield(P1, RIFTWATCHER);
    let life = t.life(P0);
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[obj(rift)]);
    activate_containing(&mut t, P0, zygon, "Tap another").expect("activation");
    t.resolve_all();
    became_riftwatcher(&t, zygon, life);
    // The Riftwatcher untaps (the stun counter goes first): Zygon stops being a copy
    // without "leaving".
    t.g.untap(rift);
    t.g.untap(rift);
    t.g.recompute();
    t.settle();
    assert!(!t.obj_now(rift).tapped);
    assert_eq!(name_of(&t, zygon), "Zygon Infiltrator");
    assert_eq!(t.life(P0), life);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn unstable_shapeshifter_doesnt_enter_as_it_changes_shape() {
    cr!("707.2", "603.6a", "603.6c", "614.1c");
    ruling!("Unstable Shapeshifter", "When it changes \"shape\" to a new creature, any \"enters\" abilities of the creature do not trigger.");
    // "Whenever another creature enters, this creature becomes a copy of that creature,
    // except it has this ability."
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    let (p0, p1) = (t.life(P0), t.life(P1));
    let rift = t.enter(P1, RIFTWATCHER);
    t.resolve_all();
    // The Riftwatcher's own enters trigger: P1 gains 2. The Shapeshifter's doesn't.
    assert_eq!(t.life(P1), p1 + 2);
    assert_eq!(t.counters(rift, counters::TIME), 3);
    assert_eq!(name_of(&t, shifter), RIFTWATCHER);
    assert_eq!(t.counters(shifter, counters::TIME), 0);
    assert_eq!(t.life(P0), p0);
    // The next creature: the Riftwatcher it was doesn't leave either.
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(name_of(&t, shifter), "Grizzly Bears");
    assert_eq!(t.life(P0), p0);
}

#[test]
fn assimilation_aegis_creature_doesnt_enter_or_leave() {
    cr!("707.2", "603.6a", "603.6c", "614.1c", "702.6a");
    ruling!("Assimilation Aegis", "When the creature Assimilation Aegis is attached to becomes a copy of an exiled creature card, it's neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won't trigger.");
    // "When this Equipment enters, exile up to one target creature until this Equipment
    // leaves the battlefield. Whenever this Equipment becomes attached to a creature, for
    // as long as this Equipment remains attached to it, that creature becomes a copy of a
    // creature card exiled with this Equipment. Equip {2}"
    supported("Assimilation Aegis");
    let mut t = TestGame::new(2);
    soul_warden(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let rift = t.battlefield(P1, RIFTWATCHER);
    t.answer_targets(P0, &[obj(rift)]);
    let aegis = t.enter(P0, "Assimilation Aegis");
    t.resolve_all();
    assert!(t.in_exile(RIFTWATCHER));
    let life = t.life(P0);
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, aegis, "Equip").expect("equip");
    t.resolve_all();
    became_riftwatcher(&t, bears, life);
    // Moved to the Hill Giant: the Bears stops being a Riftwatcher without "leaving", and
    // the Giant becomes one without "entering".
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, aegis, "Equip").expect("equip");
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
    became_riftwatcher(&t, giant, life);
}

#[test]
fn vesuvan_doppelganger_switching_isnt_leaving_the_battlefield() {
    cr!("707.2", "603.6c", "610.3");
    ruling!("Vesuvan Doppelganger", "When the Doppelganger switches creatures, the creature it used to be is not considered to have left the battlefield. Such effects will consider the creature to have left the battlefield when the Doppelganger leaves the battlefield.");
    // Banisher Priest: "When this creature enters, exile target creature an opponent
    // controls until this creature leaves the battlefield."
    supported("Vesuvan Doppelganger");
    supported("Banisher Priest");
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P1, "Banisher Priest");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(priest)]);
    t.answer_targets(P0, &[obj(elves)]);
    let dop = t.enter(P0, "Vesuvan Doppelganger");
    t.resolve_all();
    assert_eq!(name_of(&t, dop), "Banisher Priest");
    assert!(t.in_exile("Llanowar Elves"));
    // At P0's upkeep it becomes a Hill Giant: the Banisher Priest it was hasn't left.
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, dop), "Hill Giant");
    assert!(t.in_exile("Llanowar Elves"));
    // It leaves the battlefield: the Elves return.
    kill(&mut t, dop);
    t.resolve_all();
    assert!(!t.in_exile("Llanowar Elves"));
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn vesuvan_doppelganger_copying_isnt_a_shapeshifter() {
    cr!("707.2", "707.9c");
    ruling!("Vesuvan Doppelganger", "When it takes on the characteristics of the other card, it is no longer of creature type Shapeshifter.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    let dop = t.enter(P0, "Vesuvan Doppelganger");
    t.settle();
    assert!(has_subtype(&t, dop, "Bear"));
    assert!(!has_subtype(&t, dop, "Shapeshifter"));
    // It keeps its own color (blue).
    assert_eq!(t.obj_now(dop).chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn vesuvan_doppelganger_targets_on_the_stack_and_chooses_on_resolution() {
    cr!("603.3d", "603.5");
    ruling!("Vesuvan Doppelganger", "When Vesuvan Doppelganger's triggered ability triggers, you must choose a target for it. You determine whether to have the Doppelganger become a copy of that target when the ability resolves.");
    for yes in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_choose(P0, &[obj(bears)]);
        let dop = t.enter(P0, "Vesuvan Doppelganger");
        t.settle();
        t.set_step(P1, Step::End);
        t.answer_targets(P0, &[obj(giant)]);
        let from = t.asked().len();
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        let trig = *t.g.stack.last().expect("the upkeep trigger");
        assert_eq!(targets_of(&t, trig), vec![obj(giant)]);
        assert_eq!(yes_no_asked(&t, from), 0);
        t.answer_yes(P0, yes);
        t.resolve();
        assert_eq!(yes_no_asked(&t, from), 1);
        let want = if yes { "Hill Giant" } else { "Grizzly Bears" };
        assert_eq!(name_of(&t, dop), want);
    }
}

#[test]
fn vesuvan_doppelganger_copying_itself_gets_another_instance() {
    cr!("707.2", "707.9a", "603.3b", "113.7a");
    ruling!("Vesuvan Doppelganger", "Vesuvan Doppelganger's triggered ability may cause it to become a copy of itself. A Vesuvan Doppelganger that becomes a copy of a Vesuvan Doppelganger (either itself or a different one) will gain another instance of its triggered ability. Each instance of that ability will trigger during its controller's upkeep. They'll all be put on the stack and resolve one at a time. The last one to resolve determines the Doppelganger's characteristics for the rest of the turn.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_choose(P0, &[obj(bears)]);
    let dop = t.enter(P0, "Vesuvan Doppelganger");
    t.settle();
    let upkeep_ability = "At the beginning of your upkeep";
    assert_eq!(abilities_with(&t, dop, upkeep_ability), 1);
    // First upkeep: it becomes a copy of itself.
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[obj(t.g.current(dop))]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, dop), "Grizzly Bears");
    assert_eq!(abilities_with(&t, dop, upkeep_ability), 2);
    // Next upkeep: both trigger; the last one to resolve wins.
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(elves)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    let first = name_of(&t, dop);
    t.resolve_all();
    let last = name_of(&t, dop);
    assert_ne!(first, last);
    assert!(["Hill Giant", "Llanowar Elves"].contains(&last.as_str()));
    // It's a copy with just one instance again, and it stays as it is.
    assert_eq!(abilities_with(&t, dop, upkeep_ability), 1);
    t.advance_to(P0, Step::End);
    assert_eq!(name_of(&t, dop), last);
}

#[test]
fn scion_of_the_ur_dragon_loses_its_ability_as_a_copy() {
    cr!("707.2", "613.1a", "514.2");
    ruling!("Scion of the Ur-Dragon", "When Scion of the Ur-Dragon becomes a copy of another Dragon, it loses its copy ability for the rest of the turn.");
    // "{2}: Search your library for a Dragon permanent card and put it into your
    // graveyard. If you do, Scion of the Ur-Dragon becomes a copy of that card until end
    // of turn. Then shuffle."
    supported("Scion of the Ur-Dragon");
    let mut t = TestGame::new(2);
    let scion = t.battlefield(P0, "Scion of the Ur-Dragon");
    let dragon = t.library_top(P0, "Shivan Dragon");
    t.lands(P0, "Wastes", 2);
    let search = "Search your library for a Dragon";
    t.answer_choose_moved(P0, &[obj(dragon)]);
    activate_containing(&mut t, P0, scion, search).expect("activation");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shivan Dragon"));
    assert_eq!(name_of(&t, scion), "Shivan Dragon");
    assert_eq!(abilities_with(&t, scion, search), 0);
    next_turn(&mut t);
    assert_eq!(name_of(&t, scion), "Scion of the Ur-Dragon");
    assert_eq!(abilities_with(&t, scion, search), 1);
}

#[test]
fn renegade_doppelganger_can_decline_and_stay_itself() {
    cr!("603.5", "707.2");
    ruling!("Renegade Doppelganger", "When a creature enters under your control, you can choose not to copy it. In that case, Renegade Doppelganger remains 0/1 and still has its triggered ability.");
    // "Whenever another creature you control enters, you may have this creature become a
    // copy of that creature until end of turn."
    supported("Renegade Doppelganger");
    let mut t = TestGame::new(2);
    let dop = t.battlefield(P0, "Renegade Doppelganger");
    t.answer_yes(P0, false);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(name_of(&t, dop), "Renegade Doppelganger");
    assert_eq!(t.pt(dop), (0, 1));
    assert_eq!(abilities_with(&t, dop, "Whenever another creature"), 1);
    // It still triggers for the next creature.
    t.answer_yes(P0, true);
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(name_of(&t, dop), "Hill Giant");
}

#[test]
fn tilonallis_skinshifter_stays_a_copy_until_damage_wears_off() {
    cr!("514.2", "707.2", "704.5g");
    ruling!("Tilonalli's Skinshifter", "Tilonalli’s Skinshifter remains a copy of the creature through the end step. Damage is removed from it at the same time that it stops being a copy.");
    // "Whenever this creature attacks, it becomes a copy of another target nonlegendary
    // attacking creature until end of turn."
    supported("Tilonalli's Skinshifter");
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, "Tilonalli's Skinshifter");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(
        &mut t,
        &[(skin, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(name_of(&t, skin), "Hill Giant");
    // 2 damage: lethal for a 0/1 Skinshifter, not for a 3/3 Hill Giant.
    let skin_now = t.g.current(skin);
    crate::r_s06_common::damage(&mut t, giant, 2, skin_now);
    t.advance_to(P0, Step::End);
    assert!(t.on_battlefield(skin));
    assert_eq!(name_of(&t, skin), "Hill Giant");
    // Cleanup: the damage and the copy effect end together; it survives as itself.
    next_turn(&mut t);
    assert!(t.on_battlefield(skin));
    assert_eq!(name_of(&t, skin), "Tilonalli's Skinshifter");
    assert_eq!(t.obj_now(skin).damage, 0);
}

#[test]
fn irma_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "115.10");
    ruling!("Irma, Part-Time Mutant", "You don't have to choose a target for Irma's ability. However, if you do and the target creature is an illegal target as Irma's ability tries to resolve, it won't resolve and none of its effects will happen. Irma won't become a copy of anything else, and you won't put a +1/+1 counter on her.");
    // "At the beginning of combat on your turn, Irma becomes a copy of up to one other
    // target creature you control, except her name is Irma, Part-Time Mutant and she has
    // this ability. Then put a +1/+1 counter on her."
    supported("Irma, Part-Time Mutant");
    // No target: she still gets the counter.
    let mut t = TestGame::new(2);
    let irma = t.battlefield(P0, "Irma, Part-Time Mutant");
    t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(irma, counters::PLUS1), 1);
    assert_eq!(t.pt(irma), (2, 2));
    // A legal target: a copy (with her name), and the counter.
    let mut t = TestGame::new(2);
    let irma = t.battlefield(P0, "Irma, Part-Time Mutant");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(irma), (4, 4));
    assert!(has_subtype(&t, irma, "Giant"));
    // The target leaves before it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let irma = t.battlefield(P0, "Irma, Part-Time Mutant");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(irma, counters::PLUS1), 0);
    assert_eq!(t.pt(irma), (1, 1));
    assert!(!has_subtype(&t, irma, "Giant"));
}

#[test]
fn aurora_shifter_targets_with_a_reflexive_trigger() {
    cr!("603.12", "603.3d", "107.14");
    ruling!("Aurora Shifter", "You don't choose a target for Aurora Shifter's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you pay {E}{E} this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal.");
    // "At the beginning of combat on your turn, you may pay {E}{E}. When you do, this
    // creature becomes a copy of another target creature you control, except it has this
    // ability and ..."
    supported("Aurora Shifter");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Aurora Shifter");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.players[0].counters.insert(counters::ENERGY.into(), 2);
    let from = t.asked().len();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    let first = *t.g.stack.last().expect("the beginning of combat trigger");
    assert!(targets_of(&t, first).is_empty());
    let targeted_before = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    assert_eq!(targeted_before, 0);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve();
    // The reflexive trigger is on the stack with its target; players get priority.
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 0);
    assert_eq!(t.stack_len(), 1);
    let reflexive = *t.g.stack.last().unwrap();
    assert_ne!(reflexive, first);
    assert_eq!(targets_of(&t, reflexive), vec![obj(giant)]);
    assert_eq!(name_of(&t, shifter), "Aurora Shifter");
    // P1 responds by removing the target: it does nothing.
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(name_of(&t, shifter), "Aurora Shifter");
}
