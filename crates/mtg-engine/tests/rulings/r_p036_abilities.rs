//! Rulings batch P036 — countering activated and triggered abilities (CR 701.6b): keyword
//! triggers (echo, flanking, madness, storm, suspend, prowess, afflict), keyword activated
//! abilities (equip, crew, eternalize), delayed triggers (CR 603.7), what isn't on the
//! stack (mana abilities, CR 605.1, 605.3b; special actions, CR 116; turn-based actions,
//! CR 703; replacement effects, CR 614.12), and what countering leaves alone.

use crate::r_p036_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s04_common::top_of_stack;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::resolved;
use crate::r_s21_common::castable;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The legal first-target candidates for the real instant `name` cast by `p` now, which
/// is then cast at `pick` (if given). Returns (candidates, the spell).
fn aim(t: &mut TestGame, p: PlayerId, name: &str, pick: Option<ObjectId>) -> (Vec<Entity>, Option<ObjectId>) {
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    let from = t.asked().len();
    if let Some(x) = pick {
        t.answer_targets(p, &[obj(x)]);
    }
    t.g.turn.priority = Some(p);
    let r = t.g.cast_spell(p, card, mtg_engine::object::CastMethod::Normal);
    t.g.flush_events();
    let c = offered(t, p, from).into_iter().next().unwrap_or_default();
    (c, r.ok())
}

/// Trickbind (or another counter) cast by `p` at the object on top of the stack, and
/// resolved. Returns the countered object.
fn counter_top(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.settle();
    let top = top_of_stack(t);
    let (_, s) = aim(t, p, name, Some(top));
    t.resolve();
    assert!(resolved(t, s.expect("cast")), "{name} didn't resolve");
    assert!(!on_stack(t, top));
    top
}

/// P0 controls Monastery Swiftspear (prowess) and casts Shock at P1; returns (the
/// Swiftspear, its prowess trigger on top of the stack).
fn prowess_trigger(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    cast_new(t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    let trig = top_of_stack(t);
    assert!(t.obj(trig).stack.is_some());
    (spear, trig)
}

// --- Trickbind and keyword triggers ---------------------------------------------------

#[test]
fn trickbind_counters_echo_and_echo_never_triggers_again() {
    cr!("702.30a", "701.6b", "603.4");
    ruling!(
        "Trickbind",
        "Trickbind can be used to counter the echo triggered ability. If so, the echo ability won’t trigger again so the echo cost is never paid."
    );
    supported("Trickbind");
    supported("Avalanche Riders");
    let mut t = TestGame::new(2);
    let riders = t.battlefield(P0, "Avalanche Riders");
    t.lands(P0, "Mountain", 4);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    counter_top(&mut t, P1, "Trickbind");
    t.advance_to(P0, Step::Draw);
    t.settle();
    assert!(t.on_battlefield(riders));
    // Next upkeep: no echo trigger; P0 is never asked to pay.
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::Draw);
    t.settle();
    assert!(t.on_battlefield(riders));
    assert!(!t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    // Control: unanswered, echo makes P0 sacrifice it (P0 declines to pay).
    let mut t = TestGame::new(2);
    let riders = t.battlefield(P0, "Avalanche Riders");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!t.on_battlefield(riders));
}

#[test]
fn trickbind_counters_flanking() {
    cr!("702.25a", "701.6b");
    ruling!(
        "Trickbind",
        "Trickbind can be used to counter the flanking triggered ability."
    );
    supported("Suq'Ata Lancer");
    for counter in [true, false] {
        let mut t = TestGame::new(2);
        let lancer = t.battlefield(P0, "Suq'Ata Lancer");
        let bears = t.battlefield(P1, "Grizzly Bears");
        attack_with(&mut t, &[(lancer, Entity::Player(P1))]);
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, lancer)]));
        t.advance_to(P0, Step::DeclareBlockers);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if counter {
            counter_top(&mut t, P1, "Trickbind");
        } else {
            t.resolve();
        }
        assert_eq!(t.pt(bears), if counter { (2, 2) } else { (1, 1) });
    }
}

#[test]
fn trickbind_counters_madness_and_the_card_stays_exiled() {
    cr!("702.35a", "701.6b");
    ruling!(
        "Trickbind",
        "Trickbind can be used to counter the madness triggered ability. The affected card will remain exiled."
    );
    supported("Fiery Temper");
    let mut t = TestGame::new(2);
    let temper = t.hand(P0, "Fiery Temper");
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.discard(P0, temper, None);
    t.g.flush_events();
    let exiled = t.g.current(temper);
    assert_eq!(t.zone(exiled), Zone::Exile);
    counter_top(&mut t, P1, "Trickbind");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(temper)), Zone::Exile);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn trickbind_counters_storm_but_not_the_original() {
    cr!("702.40a", "701.6b");
    ruling!(
        "Trickbind",
        "Trickbind can be used to counter the storm triggered ability. Doing so prevents the copies from being created, but doesn’t counter the original storm spell."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve();
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve();
    let gs = cast_new(&mut t, P0, "Grapeshot", &[Entity::Player(P1)]);
    counter_top(&mut t, P1, "Trickbind");
    assert!(crate::r_s25_common::spell_copies(&t).is_empty());
    assert!(on_stack(&t, gs));
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 3 - 1);
}

/// P0 suspends Rift Bolt (suspend 1—{R}); returns the card in exile.
fn suspend_rift_bolt(t: &mut TestGame, time: u32) -> ObjectId {
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Rift Bolt");
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Suspend { card: bolt }))
        .expect("suspended");
    let exiled = t.g.current(bolt);
    if time > 1 {
        crate::r_s13_common::add(t, obj(exiled), "time", time - 1);
    }
    exiled
}

#[test]
fn trickbind_and_suspend() {
    cr!("702.62a", "702.62b", "701.6b", "116.2f");
    ruling!(
        "Trickbind",
        "Trickbind can be used to counter the suspend “remove a counter” ability. If so, the counter won’t get removed, but the ability will trigger again as normal later on. Trickbind can also be used to counter the suspend “play this card” ability. If so, the card stays exiled (the ability won’t trigger again). Using suspend to exile a card in your hand is neither an activated nor a triggered ability, so it can’t be countered this way."
    );
    supported("Rift Bolt");
    // Suspending is a special action: nothing goes on the stack to counter.
    let mut t = TestGame::new(2);
    let bolt = suspend_rift_bolt(&mut t, 2);
    assert!(t.g.stack.is_empty());
    assert_eq!(t.counters(bolt, "time"), 2);
    let tb = t.hand(P1, "Trickbind");
    lands_for_cost(&mut t, P1, "Trickbind");
    assert!(!castable(&mut t, P1, tb));
    // The "remove a counter" trigger is countered: the counter stays...
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    counter_top(&mut t, P1, "Trickbind");
    assert_eq!(t.counters(bolt, "time"), 2);
    // ...and it triggers again next upkeep.
    t.advance_to(P0, Step::Draw);
    t.settle();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.counters(bolt, "time"), 1);
    // The cast trigger is countered: the card stays exiled for good.
    t.advance_to(P0, Step::Draw);
    t.settle();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve(); // the last counter is removed
    assert_eq!(t.counters(bolt, "time"), 0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    counter_top(&mut t, P1, "Trickbind");
    t.advance_to(P0, Step::Draw);
    t.settle();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert_eq!(t.life(P1), 20);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
}

#[test]
fn trickbind_cant_stop_turning_a_morph_face_up() {
    cr!("116.2b", "702.37e", "701.6b");
    ruling!(
        "Trickbind",
        "Trickbind can’t counter or prevent paying the morph cost to turn a face-down creature face up, since that action doesn’t use the stack."
    );
    let mut t = TestGame::new(2);
    let fd = crate::r_s12_common::morph(&mut t, P0, "Den Protector");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, fd));
    // Nothing went on the stack; Trickbind has nothing to target.
    assert!(t.g.stack.is_empty());
    assert!(!t.obj_now(fd).face_down);
    let tb = t.hand(P1, "Trickbind");
    lands_for_cost(&mut t, P1, "Trickbind");
    assert!(!castable(&mut t, P1, tb));
}

// --- Delayed triggers ------------------------------------------------------------------

/// P0 casts Otherworldly Journey ("Exile target creature. At the beginning of the next end
/// step, return that card to the battlefield under its owner's control with a +1/+1
/// counter on it.") at its Grizzly Bears; returns the Bears.
fn journey(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(t, P0, "Otherworldly Journey", &[obj(bears)]);
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    bears
}

#[test]
fn countered_next_end_step_delayed_trigger_doesnt_trigger_again() {
    cr!("603.7a", "603.7b", "603.7c", "701.6b");
    ruling!(
        "Tale's End",
        "If you counter a delayed triggered ability that triggered at the beginning of the \"next\" occurrence of a specified step or phase, that ability won't trigger again the following time that step or phase occurs."
    );
    ruling!(
        "Stifle",
        "It can target delayed triggered abilities. For example, a card that says \"Whenever a player cycles a card, you may exile target creature. If you do, return that creature to the battlefield at the beginning of the end step.\" triggers when a player cycles a card and creates a delayed trigger that happens at the beginning of the end step. You can choose to target the \"at the beginning of the end step\" trigger when it is placed on the stack at end of turn."
    );
    supported("Otherworldly Journey");
    for name in ["Tale's End", "Stifle"] {
        supported(name);
        let mut t = TestGame::new(2);
        journey(&mut t);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        counter_top(&mut t, P1, name);
        // The following end step (P1's turn): it doesn't trigger again.
        t.advance_to(P1, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}");
        t.advance_to(P0, Step::End);
        t.settle();
        assert!(t.in_exile("Grizzly Bears"), "{name}");
        assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    }
    // Control: uncountered, the Bears return.
    let mut t = TestGame::new(2);
    journey(&mut t);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.counters(b[0], "+1/+1"), 1);
}

// --- Not on the stack: turn-based actions, special actions -------------------------------

#[test]
fn stifle_cant_target_the_draw_combat_damage_or_turning_face_up() {
    cr!("703.4c", "510.2", "116.2b", "115.1");
    ruling!(
        "Stifle",
        "Turn-based actions and special actions like the normal card draw, combat damage, or turning a face-down creature face up can't be targeted."
    );
    supported("Stifle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let stifle = t.hand(P0, "Stifle");
    lands_for_cost(&mut t, P0, "Stifle");
    // The normal draw (P1's draw step): nothing on the stack.
    let lib = t.library_size(P1);
    t.advance_to(P1, Step::Draw);
    t.settle();
    assert_eq!(t.library_size(P1), lib - 1);
    assert!(t.g.stack.is_empty());
    assert!(!castable(&mut t, P0, stifle));
    // Combat damage: nothing on the stack while it's dealt or after.
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P0), 18);
    assert!(t.g.stack.is_empty());
    assert!(!castable(&mut t, P0, stifle));
    // Turning a face-down creature face up (P1's Den Protector).
    t.advance_to(P1, Step::PostcombatMain);
    t.settle();
    let fd = crate::r_s12_common::morph(&mut t, P1, "Den Protector");
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Wastes", 1);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P1, fd));
    assert!(t.g.stack.is_empty());
    assert!(!castable(&mut t, P0, stifle));
    // Control: a triggered ability is a legal target.
    t.battlefield(P1, "Soul Warden");
    t.enter(P1, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(castable(&mut t, P0, stifle));
}

// --- What an activated or triggered ability is ----------------------------------------

#[test]
fn keyword_activated_and_triggered_abilities_can_be_countered() {
    cr!("602.1", "603.1", "702.6a", "702.122a", "702.108a", "701.6b");
    ruling!(
        "Stifle",
        "An activated ability has a \"Cost: Effect\" format. Look for the colon. A triggered ability starts with \"when\", \"whenever\", or \"at\"."
    );
    ruling!(
        "Tale's End",
        "Activated abilities are written in the form \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder texts, if applicable."
    );
    ruling!(
        "Disallow",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities, such as prowess and fabricate, are triggered abilities and will have \"when,\" \"whenever,\" or \"at\" in their reminder text."
    );
    ruling!(
        "Overcharged Amalgam",
        "Activated abilities are written in the form “[Cost]: [Effect].” Some keyword abilities, such as equip and crew, are activated abilities and will have colons in their reminder texts."
    );
    // Stifle: Prodigal Pyromancer's "{T}: This creature deals 1 damage to any target." and
    // Soul Warden's "Whenever another creature enters, you gain 1 life."
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    activate_containing(&mut t, P1, pyro, "damage").unwrap();
    counter_top(&mut t, P0, "Stifle");
    assert_eq!(t.life(P0), 20);
    t.battlefield(P1, "Soul Warden");
    t.enter(P1, "Hill Giant");
    counter_top(&mut t, P0, "Stifle");
    assert_eq!(t.life(P1), 20);
    // Tale's End: equip.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bs = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, bs, "Equip").unwrap();
    counter_top(&mut t, P1, "Tale's End");
    assert_eq!(t.pt(bears), (2, 2));
    // Disallow: prowess and fabricate (Angel of Invention's "Fabricate 2").
    let mut t = TestGame::new(2);
    let (spear, _) = prowess_trigger(&mut t);
    counter_top(&mut t, P1, "Disallow");
    t.resolve_all();
    assert_eq!(t.pt(spear), (1, 2));
    let mut t = TestGame::new(2);
    let angel = t.enter(P0, "Angel of Invention");
    counter_top(&mut t, P1, "Disallow");
    assert_eq!(t.counters(angel, "+1/+1"), 0);
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    // Overcharged Amalgam: its exploit trigger counters a crew activation.
    let mut t = TestGame::new(2);
    let crewman = t.battlefield(P1, "Grizzly Bears");
    let copter = t.battlefield(P1, "Smuggler's Copter");
    t.answer_choose(P1, &[obj(crewman)]);
    let crew = activate_containing(&mut t, P1, copter, "Crew").unwrap().unwrap();
    amalgam_at(&mut t, crew);
    assert!(!on_stack(&t, crew));
    assert!(!t.obj_now(copter).is(mtg_engine::types::CardType::Creature));
    // An equip activation too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bs = t.battlefield(P1, "Bonesplitter");
    turn_of(&mut t, P1);
    t.lands(P1, "Wastes", 1);
    t.answer_targets(P1, &[obj(bears)]);
    let equip = activate_containing(&mut t, P1, bs, "Equip").unwrap().unwrap();
    amalgam_at(&mut t, equip);
    assert!(!on_stack(&t, equip));
    assert_eq!(t.pt(bears), (2, 2));
}

/// P0 casts Overcharged Amalgam (flash), exploits a Grizzly Bears, and its "When this
/// creature exploits a creature, counter target spell, activated ability, or triggered
/// ability." trigger is aimed at `target` and resolves. Returns the candidates offered.
fn amalgam_at(t: &mut TestGame, target: ObjectId) -> Vec<Entity> {
    supported("Overcharged Amalgam");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let card = cast_new(t, P0, "Overcharged Amalgam", &[]);
    let _ = card;
    t.resolve(); // Amalgam enters; exploit triggers
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(fodder)]);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(target)]);
    t.resolve(); // exploit: sacrifice the Bears; the counter trigger goes on the stack
    assert!(!t.on_battlefield(fodder));
    let c = offered(t, P0, from).into_iter().next().unwrap_or_default();
    t.resolve();
    c
}

#[test]
fn overcharged_amalgam_cant_target_replacement_effects() {
    cr!("614.1c", "614.12", "115.1");
    ruling!(
        "Overcharged Amalgam",
        "Abilities that create replacement effects, such as a permanent entering tapped or with counters on it, can’t be targeted. Abilities that apply “as [this creature] enters” are also replacement effects and can’t be targeted."
    );
    supported("Pterafractyl");
    // P1's Pterafractyl ("This creature enters with X +1/+1 counters on it. When this
    // creature enters, you gain 2 life.") with X = 2 resolves on P1's turn.
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    t.lands(P1, "Wastes", 2);
    lands_for_cost(&mut t, P1, "Pterafractyl");
    let card = t.hand(P1, "Pterafractyl");
    t.cast(P1, card).x(2).go();
    t.resolve();
    let ptera = t.g.current(card);
    assert_eq!(t.counters(ptera, "+1/+1"), 2);
    let trigger = top_of_stack(&t);
    let c = amalgam_at(&mut t, trigger);
    // The only ability on the stack to choose is the enters trigger.
    assert_eq!(c, vec![obj(trigger)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(ptera, "+1/+1"), 2);
}

#[test]
fn nimble_obstructionist_counters_keyword_abilities_of_others() {
    cr!("702.29a", "602.1", "603.1", "702.128a", "702.130a", "701.6b");
    ruling!(
        "Nimble Obstructionist",
        "Activated abilities are written in the form \"Cost: Effect.\" Some keyword abilities, such as equip and eternalize, are activated abilities and will have colons in their reminder texts."
    );
    ruling!(
        "Nimble Obstructionist",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities, such as prowess and afflict, are triggered abilities and will have \"when,\" \"whenever,\" or \"at\" in their reminder text."
    );
    supported("Nimble Obstructionist");
    supported("Earthshaker Khenra");
    supported("Ammit Eternal");
    // Eternalize (P1's Earthshaker Khenra from its graveyard, on P1's turn).
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let khenra = t.graveyard(P1, "Earthshaker Khenra");
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Wastes", 4);
    let ab = activate_containing(&mut t, P1, khenra, "Eternalize").unwrap().unwrap();
    obstruct(&mut t, ab);
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
    assert_eq!(t.zone(t.g.current(khenra)), Zone::Exile);
    // Afflict (P1's Ammit Eternal becomes blocked by P0's Bears).
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let ammit = t.battlefield(P1, "Ammit Eternal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(ammit, Entity::Player(P0))]);
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(bears, ammit)]));
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    let trig = top_of_stack(&t);
    obstruct(&mut t, trig);
    assert_eq!(t.life(P0), 20);
    // Prowess (P1's Swiftspear) and equip (P1's Bonesplitter).
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let spear = t.battlefield(P1, "Monastery Swiftspear");
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.settle();
    let trig = top_of_stack(&t);
    obstruct(&mut t, trig);
    t.resolve_all();
    assert_eq!(t.pt(spear), (1, 2));
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bs = t.battlefield(P1, "Bonesplitter");
    t.lands(P1, "Wastes", 1);
    t.answer_targets(P1, &[obj(bears)]);
    let ab = activate_containing(&mut t, P1, bs, "Equip").unwrap().unwrap();
    obstruct(&mut t, ab);
    assert_eq!(t.pt(bears), (2, 2));
}

/// P0 cycles Nimble Obstructionist and its "When you cycle this card, counter target
/// activated or triggered ability you don't control." trigger is aimed at `target` and
/// resolves. Returns the candidates offered.
fn obstruct(t: &mut TestGame, target: ObjectId) -> Vec<Entity> {
    let card = t.hand(P0, "Nimble Obstructionist");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(target)]);
    crate::r_s04_common::cycle(t, P0, card, 0).expect("cycled");
    t.settle();
    let c = offered(t, P0, from).into_iter().next().unwrap_or_default();
    // The trigger resolves (above the cycling ability's draw).
    t.resolve();
    assert!(!on_stack(t, target));
    c
}
