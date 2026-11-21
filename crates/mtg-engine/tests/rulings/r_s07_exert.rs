//! Rulings batch S07 — exert (CR 701.43): an exerted permanent won't untap during its
//! controller's next untap step. "You may exert [this creature] as it attacks" is an
//! optional cost to attack (CR 508.1g, 701.43d).

use crate::r_s01_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn exerted(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).exerted
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// P1 casts Decision Paralysis ("Tap up to two target creatures. Those creatures don't
/// untap during their controller's next untap step.") on `target`.
fn paralyze(t: &mut TestGame, target: ObjectId) {
    t.lands(P1, "Island", 4);
    let dp = t.hand(P1, "Decision Paralysis");
    t.answer_targets(P1, &[Entity::Object(target)]);
    t.cast(P1, dp).go();
    t.resolve_all();
}

/// The number of "Exert ... as it attacks?" questions asked since decision `from`.
fn exert_questions(t: &TestGame, from: usize) -> usize {
    count_asked(t, from, |d| {
        matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Exert"))
    })
}

#[test]
fn tap_and_freeze_effects_dont_exert() {
    cr!("701.43a", "701.43d");
    ruling!(
        "Glorybringer",
        "You can't exert a creature unless an effect allows you to do so. Similar effects that \"tap and freeze\" a creature (such as that of Decision Paralysis) don't exert that creature."
    );
    supported("Decision Paralysis");
    supported("Trueheart Twins");
    // Trueheart Twins: "Whenever you exert a creature, creatures you control get +1/+0
    // until end of turn."
    let mut t = TestGame::new(2);
    let twins = t.battlefield(P0, "Trueheart Twins");
    let dragon = t.battlefield(P0, "Glorybringer");
    paralyze(&mut t, dragon);
    assert!(tapped(&t, dragon));
    assert!(!exerted(&t, dragon));
    assert_eq!(t.pt(twins), (4, 4));
    // It doesn't untap during P0's next untap step, because of Decision Paralysis.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, dragon));
    // Glorybringer can be exerted only as it attacks: not otherwise.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Glorybringer");
    let from = t.asked().len();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(exert_questions(&t, from), 0);
    assert!(!exerted(&t, dragon));
}

#[test]
fn tap_and_freeze_effects_dont_exert_battlefield_scavenger() {
    cr!("701.43a");
    ruling!(
        "Battlefield Scavenger",
        "You can't exert a creature unless an effect allows you to do so. Similar effects that \"tap and freeze\" a creature (such as that of Decision Paralysis) don't exert that creature."
    );
    supported("Battlefield Scavenger");
    // Battlefield Scavenger: "Whenever you exert a creature, you may discard a card. If you
    // do, draw a card."
    let mut t = TestGame::new(2);
    let scavenger = t.battlefield(P0, "Battlefield Scavenger");
    paralyze(&mut t, scavenger);
    assert!(tapped(&t, scavenger));
    assert!(!exerted(&t, scavenger));
    assert_eq!(triggers_on_stack(&t, "exert"), 0);
    assert!(t.g.stack.is_empty());
}

/// Glory-Bound Initiate's power and toughness.
fn initiate_pt(g: &Game) -> (i32, i32) {
    let id = g.find_in_zone(Zone::Battlefield, "Glory-Bound Initiate")[0];
    (g.obj(id).power(), g.obj(id).toughness())
}

#[test]
fn a_creature_is_exerted_as_it_is_declared_as_an_attacker() {
    cr!("508.1g", "701.43d", "508.2", "509.1");
    ruling!(
        "Glory-Bound Initiate",
        "All cards in the Amonkhet set that let you exert a creature let you do so as you declare it as an attacking creature, as do some of the cards in the Hour of Devastation set. You can't do so later in combat, and creatures put onto the battlefield attacking can't be exerted. Any abilities that trigger on exerting an attacking creature will resolve before blockers are declared."
    );
    supported("Glory-Bound Initiate");
    // Glory-Bound Initiate: 3/1, "You may exert this creature as it attacks. When you do, it
    // gets +1/+3 and gains lifelink until end of turn."
    let mut t = TestGame::new(2);
    let gbi = t.battlefield(P0, "Glory-Bound Initiate");
    // P1 has a potential blocker (and doesn't block).
    t.battlefield(P1, "Grizzly Bears");
    let is_blocks = |d: &Decision| matches!(d, Decision::DeclareBlockers { .. });
    let seen = watch(&mut t, P1, is_blocks, initiate_pt);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.attack(&[(gbi, Entity::Player(P1))], &[]);
    assert!(exerted(&t, gbi));
    // The trigger resolved before blockers were declared.
    assert_eq!(*seen.lock().unwrap(), vec![(4, 4)]);
    // P0 was asked once, as it was declared: not again later in combat.
    assert_eq!(exert_questions(&t, from), 1);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    // Declining as it's declared: it can't be exerted later in combat.
    let mut t = TestGame::new(2);
    let gbi = t.battlefield(P0, "Glory-Bound Initiate");
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.attack(&[(gbi, Entity::Player(P1))], &[]);
    assert!(!exerted(&t, gbi));
    assert_eq!(exert_questions(&t, from), 1);
    assert_eq!(t.life(P1), 17);
    // Put onto the battlefield attacking: it can't be exerted.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let from = t.asked().len();
    let card = t.exile(P0, "Glory-Bound Initiate");
    let new = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(P0),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(P0),
                attacking: Some(Entity::Player(P1)),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    t.g.flush_events();
    t.settle();
    assert!(t.g.is_attacking(new));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(exert_questions(&t, from), 0);
    assert!(!exerted(&t, new));
    assert_eq!(t.life(P1), 20 - 2 - 3);
}

#[test]
fn an_exerted_creature_already_untapped_during_the_untap_step() {
    cr!("701.43a", "701.43b", "502.3");
    ruling!(
        "Emberhorn Minotaur",
        "If an exerted creature is already untapped during your next untap step (most likely because it had vigilance or an effect untapped it), exert's effect preventing it from untapping expires without having done anything."
    );
    supported("Emberhorn Minotaur");
    // Emberhorn Minotaur: "You may exert this creature as it attacks. When you do, it gets
    // +1/+1 and gains menace until end of turn."
    let mut t = TestGame::new(2);
    let mino = t.battlefield(P0, "Emberhorn Minotaur");
    t.answer_yes(P0, true);
    t.attack(&[(mino, Entity::Player(P1))], &[]);
    assert!(exerted(&t, mino) && tapped(&t, mino));
    // An effect untaps it after combat.
    run_from(
        &mut t,
        P0,
        None,
        Effect::Untap {
            what: Sel::Target(0),
        },
        &[Entity::Object(mino)],
    );
    assert!(!tapped(&t, mino));
    // P0's next untap step: it's already untapped, and exert's effect expires.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!exerted(&t, mino));
    // Tapped afterward, it untaps as usual during the following untap step.
    t.g.tap(mino);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, mino));
}

#[test]
fn an_exerted_permanent_already_untapped_during_the_untap_step() {
    cr!("701.43a", "701.43b");
    ruling!(
        "Hydra Trainer",
        "If an exerted permanent is already untapped during your next untap step (most likely because an effect untapped it), exert's effect preventing it from untapping expires without having done anything."
    );
    supported("Hydra Trainer");
    // Hydra Trainer: "You may exert this creature as it attacks. When you do, target
    // creature gets +X/+X until end of turn, where X is the number of counters on
    // permanents you control."
    let mut t = TestGame::new(2);
    let trainer = t.battlefield(P0, "Hydra Trainer");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(trainer)]);
    t.attack(&[(trainer, Entity::Player(P1))], &[]);
    assert!(exerted(&t, trainer));
    t.g.untap(trainer);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!exerted(&t, trainer));
    t.g.tap(trainer);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, trainer));
}

/// P0 casts Act of Treason on P1's `name`, attacks with it (exerting it), and the game
/// advances to P1's upkeep. Returns the creature.
fn steal_and_exert(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.battlefield(P1, name);
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Act of Treason");
    t.answer_targets(P0, &[Entity::Object(c)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(c).controller, P0);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(c)]);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    assert!(exerted(t, c) && tapped(t, c));
    t.advance_to(P1, Step::Upkeep);
    c
}

#[test]
fn a_stolen_creature_you_exert_untaps_during_its_owners_untap_step() {
    cr!("701.43a", "502.3");
    ruling!(
        "Ahn-Crop Crasher",
        "If you gain control of another player's creature until end of turn and exert it, it will untap during that player's untap step."
    );
    supported("Ahn-Crop Crasher");
    supported("Act of Treason");
    // Ahn-Crop Crasher: "You may exert this creature as it attacks. When you do, target
    // creature can't block this turn."
    let mut t = TestGame::new(2);
    let c = steal_and_exert(&mut t, "Ahn-Crop Crasher");
    assert_eq!(t.obj_now(c).controller, P1);
    assert!(!tapped(&t, c));
}

#[test]
fn a_stolen_permanent_you_exert_untaps_during_its_owners_untap_step() {
    cr!("701.43a", "502.3");
    ruling!(
        "Hydra Trainer",
        "If you gain control of another player's permanent until end of turn and exert it, and then that player regains control of it, it will untap during that player's untap step."
    );
    let mut t = TestGame::new(2);
    let c = steal_and_exert(&mut t, "Hydra Trainer");
    assert_eq!(t.obj_now(c).controller, P1);
    assert!(!tapped(&t, c));
}

#[test]
fn a_creature_can_be_exerted_to_pay_a_cost_again() {
    cr!("701.43a", "701.43b", "602.2b");
    ruling!(
        "Hope Tender",
        "Some cards in the Hour of Devastation set let you exert a creature as a cost to activate one of its abilities. You can exert it to pay that cost even if you've already exerted it earlier in the turn. Exerting it multiple times will keep it tapped only during your next untap step."
    );
    supported("Hope Tender");
    // Hope Tender: "{1}, {T}, Exert this creature: Untap two target lands."
    let mut t = TestGame::new(2);
    let tender = t.battlefield(P0, "Hope Tender");
    let lands = t.lands(P0, "Forest", 4);
    for l in &lands[2..] {
        t.g.tap(*l);
    }
    let targets = [Entity::Object(lands[2]), Entity::Object(lands[3])];
    t.answer_targets(P0, &targets);
    t.activate(P0, tender, 1, &[]).expect("activate");
    t.resolve_all();
    assert!(exerted(&t, tender));
    // Untapped by an effect, it's exerted again to activate the ability again.
    t.g.untap(tender);
    t.answer_targets(P0, &targets);
    t.activate(P0, tender, 1, &[]).expect("activate again");
    t.resolve_all();
    assert!(tapped(&t, tender));
    // It stays tapped during P0's next untap step only.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, tender));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, tender));
}

#[test]
fn a_creature_can_be_exerted_without_a_legal_target_for_its_trigger() {
    cr!("701.43d", "603.3d");
    ruling!(
        "Devoted Crop-Mate",
        "If a creature has a targeted triggered ability that triggers when you exert it, you can exert it even if there isn't a legal target for that triggered ability."
    );
    supported("Devoted Crop-Mate");
    // Devoted Crop-Mate: "You may exert this creature as it attacks. When you do, return
    // target creature card with mana value 2 or less from your graveyard to the
    // battlefield." P0's graveyard is empty.
    let mut t = TestGame::new(2);
    let mate = t.battlefield(P0, "Devoted Crop-Mate");
    t.answer_yes(P0, true);
    t.attack(&[(mate, Entity::Player(P1))], &[]);
    assert!(exerted(&t, mate));
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, mate));
    // With a legal target, the card returns.
    let mut t = TestGame::new(2);
    let mate = t.battlefield(P0, "Devoted Crop-Mate");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.attack(&[(mate, Entity::Player(P1))], &[]);
    assert!(t.on_battlefield(bears));
}
