//! CR 701.54: the Ring tempts you.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::kwa::ring::{is_ring_bearer, ring_bearer, the_ring, RING_TEMPTS};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn tempt(t: &mut TestGame, p: PlayerId) {
    run(t, p, None, ka(KeywordAction::TheRingTemptsYou, Sel::None, 1), &[]);
}

#[test]
fn the_ring_tempts_you_and_you_choose_your_ring_bearer() {
    cr!("701.54a");
    ruling!(
        "Birthday Escape",
        "Each time the Ring tempts you, you must choose a creature if you control one."
    );
    supported("Birthday Escape");
    // "Draw a card. The Ring tempts you."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    let spell = t.hand(P0, "Birthday Escape");
    choose(&mut t, P0, &[giant]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(ring_bearer(&t.g, P0), Some(giant));
    // Until another creature becomes the Ring-bearer.
    choose(&mut t, P0, &[bears]);
    tempt(&mut t, P0);
    assert_eq!(ring_bearer(&t.g, P0), Some(bears));
    assert!(!is_ring_bearer(&t.g, P0, giant));
    // Or until another player gains control of it; getting it back doesn't restore it.
    run(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    t.settle();
    assert_eq!(ring_bearer(&t.g, P0), None);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj(bears).controller, P0);
    assert_eq!(ring_bearer(&t.g, P0), None);
}

#[test]
fn ring_bearer_isnt_a_copiable_value() {
    cr!("701.54b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    tempt(&mut t, P0);
    assert_eq!(ring_bearer(&t.g, P0), Some(bears));
    let copy = run(
        &mut t,
        P0,
        None,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &[Entity::Object(bears)],
    )
    .var_objects(vars::CREATED)[0];
    assert!(!is_ring_bearer(&t.g, P0, copy));
    // The Ring-bearer is legendary; the copy isn't.
    assert!(t.obj(bears).chars.is_legendary());
    assert!(!t.obj(copy).chars.is_legendary());
}

#[test]
fn the_ring_emblem_gains_abilities_as_the_ring_tempts_you() {
    cr!("701.54c");
    ruling!(
        "Birthday Escape",
        "As the Ring tempts you, you get an emblem named The Ring if you don't have one. Then your emblem gains its next ability"
    );
    let mut t = TestGame::new(2);
    let bearer = t.battlefield(P0, "Grizzly Bears");
    let big = t.battlefield(P1, "Hill Giant");
    let small = t.battlefield(P1, "Llanowar Elves");
    assert!(the_ring(&t.g, P0).is_none());
    tempt(&mut t, P0);
    let ring = the_ring(&t.g, P0).expect("an emblem named The Ring");
    assert_eq!(t.obj(ring).chars.name.as_str(), "The Ring");
    assert_eq!(t.obj(ring).zone, Zone::Command);
    // Once: legendary, and can't be blocked by creatures with greater power.
    assert!(t.obj(bearer).chars.is_legendary());
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bearer, Entity::Player(P1))], &[(big, bearer)]);
    assert_eq!(t.life(P1), 18);
    // Twice: "Whenever your Ring-bearer attacks, draw a card, then discard a card."
    tempt(&mut t, P0);
    assert_eq!(the_ring(&t.g, P0), Some(ring));
    let mut t2 = TestGame::new(2);
    let bearer2 = t2.battlefield(P0, "Grizzly Bears");
    tempt(&mut t2, P0);
    let library = t2.library_size(P0);
    t2.set_step(P0, Step::BeginningOfCombat);
    t2.attack(&[(bearer2, Entity::Player(P1))], &[]);
    assert_eq!(t2.library_size(P0), library, "no loot at level 1");
    tempt(&mut t2, P0);
    t2.advance_to(P0, Step::PrecombatMain);
    let library = t2.library_size(P0);
    let graveyard = t2.graveyard_size(P0);
    t2.set_step(P0, Step::BeginningOfCombat);
    t2.attack(&[(bearer2, Entity::Player(P1))], &[]);
    assert_eq!(t2.library_size(P0), library - 1);
    assert_eq!(t2.graveyard_size(P0), graveyard + 1);
    // Three times: the blocking creature's controller sacrifices it at end of combat.
    let mut t3 = TestGame::new(2);
    let bearer3 = t3.battlefield(P0, "Hill Giant");
    // A 0/5 blocker survives combat damage.
    let blocker = t3.custom(P1, vanilla("Stone Wall", "{0}", 0, 5), Zone::Battlefield);
    for _ in 0..3 {
        tempt(&mut t3, P0);
    }
    t3.set_step(P0, Step::BeginningOfCombat);
    t3.attack(&[(bearer3, Entity::Player(P1))], &[(blocker, bearer3)]);
    t3.resolve_all();
    t3.advance_to(P0, Step::PostcombatMain);
    assert!(t3.in_graveyard(P1, "Stone Wall"));
    assert!(t3.on_battlefield(bearer3));
    let _ = small;
    // Four times: combat damage to a player makes each opponent lose 3 life.
    let mut t4 = TestGame::new(2);
    let bearer4 = t4.battlefield(P0, "Grizzly Bears");
    for _ in 0..4 {
        tempt(&mut t4, P0);
    }
    t4.set_step(P0, Step::BeginningOfCombat);
    t4.attack(&[(bearer4, Entity::Player(P1))], &[]);
    t4.resolve_all();
    assert_eq!(t4.life(P1), 20 - 2 - 3);
}

#[test]
fn the_ring_tempts_you_even_without_a_creature() {
    cr!("701.54d");
    ruling!(
        "Birthday Escape",
        "The Ring can tempt you even if you don't control a creature. In this case, abilities that trigger \"whenever the Ring tempts you\" will still trigger."
    );
    supported("Faramir, Field Commander");
    // "Whenever the Ring tempts you, if you chose a creature other than Faramir as your
    // Ring-bearer, create a 1/1 white Human Soldier creature token."
    let mut t = TestGame::new(2);
    let watcher = text_card(
        "Ring Watcher",
        "Enchantment",
        "{0}",
        None,
        "Whenever the Ring tempts you, you gain 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    tempt(&mut t, P0);
    t.resolve_all();
    assert_eq!(custom_events(&t, RING_TEMPTS), vec![(Some(P0), None, 1)]);
    assert_eq!(t.life(P0), 21);
    assert!(the_ring(&t.g, P0).is_some());
    // Faramir: only if a creature other than it was chosen.
    let mut t = TestGame::new(2);
    let faramir = t.battlefield(P0, "Faramir, Field Commander");
    let bears = t.battlefield(P0, "Grizzly Bears");
    choose(&mut t, P0, &[faramir]);
    tempt(&mut t, P0);
    t.resolve_all();
    assert!(t.named_on_battlefield("Human Soldier Token").is_empty());
    choose(&mut t, P0, &[bears]);
    tempt(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Human Soldier Token").len(), 1);
}

#[test]
fn is_your_ring_bearer_only_on_the_battlefield_under_your_control() {
    cr!("701.54e");
    let mut t = TestGame::new(2);
    let checker = text_card(
        "Ring Checker",
        "Creature — Halfling",
        "{0}",
        Some((1, 1)),
        "At the beginning of your end step, if this creature is your Ring-bearer, draw a card.",
    );
    let c = t.custom(P0, checker, Zone::Battlefield);
    tempt(&mut t, P0);
    assert!(is_ring_bearer(&t.g, P0, c));
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Under another player's control, it isn't your Ring-bearer (and theirs neither).
    run(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(c)],
    );
    assert!(!is_ring_bearer(&t.g, P0, c));
    assert!(!is_ring_bearer(&t.g, P1, c));
    assert!(!t.obj(c).chars.is_legendary());
}
