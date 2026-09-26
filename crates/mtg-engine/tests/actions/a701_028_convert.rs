//! CR 701.28: convert.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{Effect, Filter, KeywordAction, Sel};
use mtg_engine::events::Event;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const GOLDBUG: &str = "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout";
const JETFIRE: &str = "Jetfire, Ingenious Scientist // Jetfire, Air Guardian";
const CYCLONUS: &str = "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter";

/// Converts `id` with an effect of no source.
fn convert(t: &mut TestGame, id: ObjectId) {
    run(t, P0, None, ka(KeywordAction::Convert, Sel::Target(0), 1), &[Entity::Object(id)]);
    t.resolve_all();
}

fn transformed_events(t: &TestGame) -> usize {
    t.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, Event::Transformed { .. }))
        .count()
}

fn wolves(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Wolf"))
        .count()
}

#[test]
fn converting_turns_a_permanent_so_its_other_face_is_up() {
    cr!("701.28a");
    ruling!(
        "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout",
        "Converting a permanent doesn't affect any Auras or Equipment that are attached to it. Similarly, any counters on the permanent will remain on that permanent after it converts."
    );
    ruling!(
        "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout",
        "The mana value of a converted permanent on the battlefield is equal to the mana value of the card's front face, no matter which face is up."
    );
    // Goldbug, Humanity's Ally: "Whenever you cast your second spell each turn, convert
    // Goldbug."
    let mut t = TestGame::new(2);
    let goldbug = t.battlefield(P0, GOLDBUG);
    t.g.add_counters(Entity::Object(goldbug), counters::PLUS1, 1, None);
    // Ensoul Artifact: "Enchant artifact. Enchanted artifact is a creature with base power
    // and toughness 5/5 in addition to its other types."
    let aura = t.battlefield(P0, "Ensoul Artifact");
    t.g.attach(aura, Entity::Object(goldbug));
    for _ in 0..2 {
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    assert_eq!(t.obj_now(goldbug).face, FaceState::Back);
    assert_eq!(t.obj_now(goldbug).chars.name, "Goldbug, Scrappy Scout");
    // The same object, with its counter and Aura.
    assert!(t.g.is_live(goldbug));
    assert_eq!(t.counters(goldbug, counters::PLUS1), 1);
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(goldbug)));
    // The Aura's effect still applies: a 5/5 creature, plus its +1/+1 counter.
    assert!(t.obj_now(goldbug).is_creature());
    assert_eq!(t.pt(goldbug), (6, 6));
    // Its mana value is its front face's ({1}{W}{U}).
    assert_eq!(t.g.mana_value_of(goldbug), 3);
    assert_eq!(transformed_events(&t), 1);
}

#[test]
fn abilities_that_trigger_when_a_permanent_transforms_trigger_when_it_converts() {
    cr!("701.28a");
    ruling!(
        "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout",
        "Any triggered ability of another card that triggers whenever a permanent transforms will also trigger whenever a permanent converts."
    );
    ruling!(
        "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout",
        "an ability of another card that instructs you to \"transform\" one of these cards will cause you to convert it"
    );
    supported("Cult of the Waxing Moon");
    // "Whenever a permanent you control transforms into a non-Human creature, create a 2/2
    // green Wolf creature token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cult of the Waxing Moon");
    let goldbug = t.battlefield(P0, GOLDBUG);
    // Into a Vehicle (not a creature on the opponent's turn): no Wolf.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    convert(&mut t, goldbug);
    assert_eq!(t.obj_now(goldbug).chars.name, "Goldbug, Scrappy Scout");
    assert_eq!(wolves(&t), 0);
    // Back into a Robot creature, by an instruction to transform it: a Wolf.
    run(
        &mut t,
        P0,
        None,
        Effect::Transform {
            what: Sel::All(Filter::Objects(vec![goldbug])),
        },
        &[],
    );
    t.resolve_all();
    assert_eq!(t.obj_now(goldbug).chars.name, "Goldbug, Humanity's Ally");
    assert_eq!(wolves(&t), 1);
}

#[test]
fn converting_isnt_turning_face_up_or_face_down() {
    cr!("701.28b");
    supported("Aven Farseer");
    // "Whenever a permanent is turned face up, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let farseer = t.battlefield(P0, "Aven Farseer");
    let goldbug = t.battlefield(P0, GOLDBUG);
    convert(&mut t, goldbug);
    convert(&mut t, goldbug);
    assert_eq!(t.obj_now(goldbug).face, FaceState::Front);
    assert!(!t.obj_now(goldbug).face_down);
    assert_eq!(transformed_events(&t), 2);
    assert_eq!(t.counters(farseer, counters::PLUS1), 0);
    assert!(!t.turn_events.iter().any(|e| matches!(
        e,
        Event::TurnedFaceUp { .. } | Event::TurnedFaceDown { .. }
    )));
}

#[test]
fn a_permanent_that_isnt_double_faced_doesnt_convert() {
    cr!("701.28c");
    ruling!(
        "Ugin's Mastery",
        "While face down, a transforming double-faced card can't transform or convert."
    );
    // A single-faced permanent with "{T}: Convert this creature."
    let mut t = TestGame::new(2);
    let robot = t.custom(
        P0,
        text_card(
            "Toy Robot",
            "Artifact Creature — Robot",
            "{2}",
            Some((2, 2)),
            "{T}: Convert this creature.",
        ),
        Zone::Battlefield,
    );
    t.activate(P0, robot, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.obj_now(robot).chars.name, "Toy Robot");
    assert_eq!(t.obj_now(robot).face, FaceState::Front);
    assert_eq!(transformed_events(&t), 0);
    // A face-down double-faced permanent doesn't either.
    let mut t = TestGame::new(2);
    let card = t.library_top(P0, GOLDBUG);
    run(&mut t, P0, None, ka(KeywordAction::Manifest, Sel::None, 1), &[]);
    let manifested = t.g.current(card);
    assert!(t.obj_now(manifested).face_down);
    convert(&mut t, manifested);
    assert!(t.obj_now(manifested).face_down);
    assert_eq!(t.obj_now(manifested).face, FaceState::Front);
    assert_eq!(t.pt(manifested), (2, 2));
    assert_eq!(transformed_events(&t), 0);
}

#[test]
fn a_permanent_doesnt_convert_into_an_instant_or_sorcery_face() {
    cr!("701.28d");
    let mut t = TestGame::new(2);
    // Invasion of Alara's back face is Awaken the Maelstrom, a sorcery.
    let siege = t.battlefield(P0, "Invasion of Alara");
    convert(&mut t, siege);
    assert_eq!(t.obj_now(siege).face, FaceState::Front);
    assert_eq!(t.obj_now(siege).chars.name, "Invasion of Alara");
    // Soporific Springs, played as a land, would convert into Sink into Stupor, an instant.
    let springs = t.hand(P0, "Sink into Stupor // Soporific Springs");
    t.play_land(P0, springs).expect("play land");
    let springs = t.g.current(springs);
    convert(&mut t, springs);
    assert_eq!(t.obj_now(springs).chars.name, "Soporific Springs");
    assert_eq!(transformed_events(&t), 0);
}

#[test]
fn an_ability_of_a_permanent_doesnt_convert_it_if_it_converted_since_it_was_put_on_the_stack() {
    cr!("701.28e");
    // Jetfire, Air Guardian: "{U}{U}{U}: Convert Jetfire, then adapt 3."
    let mut t = TestGame::new(2);
    let jetfire = t.battlefield(P0, JETFIRE);
    convert(&mut t, jetfire);
    assert_eq!(t.obj_now(jetfire).chars.name, "Jetfire, Air Guardian");
    t.lands(P0, "Island", 6);
    t.activate(P0, jetfire, 0, &[]).expect("activate");
    t.activate(P0, jetfire, 0, &[]).expect("activate again");
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.obj_now(jetfire).chars.name, "Jetfire, Ingenious Scientist");
    assert_eq!(t.counters(jetfire, counters::PLUS1), 3);
    // The other activation doesn't convert it back (and adapt does nothing).
    t.resolve();
    assert_eq!(t.obj_now(jetfire).chars.name, "Jetfire, Ingenious Scientist");
    assert_eq!(t.counters(jetfire, counters::PLUS1), 3);
    assert_eq!(transformed_events(&t), 2);
    // Transforming counts too: an activation put on the stack before it transformed
    // doesn't convert it.
    let mut t = TestGame::new(2);
    let jetfire = t.battlefield(P0, JETFIRE);
    convert(&mut t, jetfire);
    t.lands(P0, "Island", 3);
    t.activate(P0, jetfire, 0, &[]).expect("activate");
    run(
        &mut t,
        P0,
        None,
        Effect::Transform {
            what: Sel::All(Filter::Objects(vec![jetfire])),
        },
        &[],
    );
    assert_eq!(t.obj_now(jetfire).chars.name, "Jetfire, Ingenious Scientist");
    t.resolve();
    assert_eq!(t.obj_now(jetfire).chars.name, "Jetfire, Ingenious Scientist");
    assert_eq!(t.counters(jetfire, counters::PLUS1), 3);
}

#[test]
fn a_permanent_that_cant_transform_cant_convert() {
    cr!("701.28f");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        text_card(
            "Toy Box",
            "Artifact",
            "{2}",
            None,
            "Artifact creatures you control can't transform.",
        ),
        Zone::Battlefield,
    );
    let goldbug = t.battlefield(P0, GOLDBUG);
    convert(&mut t, goldbug);
    assert_eq!(t.obj_now(goldbug).face, FaceState::Front);
    assert_eq!(transformed_events(&t), 0);
    // It applies to transforming as well: Immerwolf ("Non-Human Werewolves you control
    // can't transform.").
    supported("Immerwolf");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Immerwolf");
    let messenger = t.battlefield(P0, "Village Messenger // Moonrise Intruder");
    let intruder = t.battlefield(P0, "Village Messenger // Moonrise Intruder");
    convert(&mut t, intruder);
    // A Human Werewolf can transform; once a non-Human Werewolf, it can't transform back.
    assert_eq!(t.obj_now(intruder).chars.name, "Moonrise Intruder");
    run(
        &mut t,
        P0,
        None,
        Effect::Transform {
            what: Sel::All(Filter::Objects(vec![intruder])),
        },
        &[],
    );
    assert_eq!(t.obj_now(intruder).chars.name, "Moonrise Intruder");
    assert_eq!(t.obj_now(messenger).chars.name, "Village Messenger");
}

#[test]
fn if_you_do_after_convert_is_whether_it_converted() {
    cr!("701.28a", "701.28f");
    supported(CYCLONUS);
    // Cyclonus, Cybertronian Fighter: "Whenever Cyclonus deals combat damage to a player,
    // convert it. If you do, there is an additional beginning phase after this phase."
    let fighter = |cant_transform: bool| -> (String, bool) {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, CYCLONUS);
        convert(&mut t, c);
        assert_eq!(t.obj_now(c).chars.name, "Cyclonus, Cybertronian Fighter");
        // A creature (living metal isn't needed for this).
        run(
            &mut t,
            P0,
            None,
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![mtg_engine::ability::Modification::AddTypes(vec![
                    CardType::Creature,
                ])],
                duration: mtg_engine::ability::Duration::Permanent,
            },
            &[Entity::Object(c)],
        );
        if cant_transform {
            t.custom(
                P1,
                text_card(
                    "Rust Box",
                    "Artifact",
                    "{2}",
                    None,
                    "Artifact creatures can't transform.",
                ),
                Zone::Battlefield,
            );
        }
        t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
        t.attack(&[(c, Entity::Player(P1))], &[]);
        assert_eq!(t.life(P1), 15);
        let extra = t
            .g
            .turn
            .schedule
            .contains(&mtg_engine::turn::Step::Untap);
        (t.obj_now(c).chars.name.to_string(), extra)
    };
    assert_eq!(
        fighter(false),
        ("Cyclonus, the Saboteur".to_string(), true)
    );
    // It can't convert: no additional beginning phase.
    assert_eq!(
        fighter(true),
        ("Cyclonus, Cybertronian Fighter".to_string(), false)
    );
}
