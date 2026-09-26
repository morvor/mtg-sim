//! CR 701.27: transform.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn transform(t: &mut TestGame, id: ObjectId) {
    run(
        t,
        P0,
        None,
        Effect::Transform {
            what: Sel::All(Filter::Objects(vec![id])),
        },
    );
    t.resolve_all();
}

fn transformed_events(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Transformed { .. }))
        .count()
}

fn wolves(t: &TestGame) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| t.obj(**o).chars.has_subtype("Wolf"))
        .count()
}

#[test]
fn transforming_turns_a_double_faced_permanent_over() {
    cr!("701.27", "701.27a");
    supported("Kessig Prowler");
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, "Kessig Prowler");
    t.lands(P0, "Forest", 5);
    // "{4}{G}: Transform this creature."
    t.activate(P0, prowler, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj(prowler).face, FaceState::Back);
    assert_eq!(t.obj(prowler).chars.name, "Sinuous Predator");
    // It's the same object.
    assert!(t.g.is_live(prowler));
}

#[test]
fn transforming_isnt_turning_face_up() {
    cr!("701.27b");
    supported("Aven Farseer");
    supported("Cult of the Waxing Moon");
    let mut t = TestGame::new(2);
    // "Whenever a permanent is turned face up, put a +1/+1 counter on this creature."
    let farseer = t.battlefield(P0, "Aven Farseer");
    // "Whenever a permanent you control transforms into a non-Human creature, create a 2/2
    // green Wolf creature token."
    t.battlefield(P0, "Cult of the Waxing Moon");
    let prowler = t.battlefield(P0, "Kessig Prowler");
    transform(&mut t, prowler);
    assert_eq!(t.obj(prowler).face, FaceState::Back);
    assert_eq!(t.counters(farseer, "+1/+1"), 0);
    assert_eq!(wolves(&t), 1);
    // Turning a face-down creature face up isn't transforming it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    mtg_engine::facedown::turn_face_down(&mut t.g, bears);
    t.g.recompute();
    mtg_engine::facedown::turn_face_up(&mut t.g, bears, false);
    t.g.recompute();
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(farseer, "+1/+1"), 1);
    assert_eq!(wolves(&t), 1);
}

#[test]
fn a_permanent_that_isnt_a_double_faced_card_doesnt_transform() {
    cr!("701.27c", "712.4c", "712.9", "712.15a");
    supported("Clone");
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P1, "Kessig Prowler");
    // A Clone copying Kessig Prowler has "{4}{G}: Transform this creature.", but it's not
    // a double-faced card: nothing happens.
    t.answer_choose(P0, &[Entity::Object(prowler)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Kessig Prowler");
    let clone = t.g.current(clone);
    t.lands(P0, "Forest", 5);
    t.activate(P0, clone, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj(clone).face, FaceState::Front);
    assert_eq!(t.obj(clone).chars.name, "Kessig Prowler");
    // Nor does a single-faced permanent, ...
    let bears = t.battlefield(P0, "Grizzly Bears");
    transform(&mut t, bears);
    assert_eq!(t.obj(bears).chars.name, "Grizzly Bears");
    // ... a meld card (CR 712.4c), ...
    let bruna = t.battlefield(P0, "Bruna, the Fading Light");
    transform(&mut t, bruna);
    assert_eq!(t.obj(bruna).face, FaceState::Front);
    // ... or a face-down double-faced permanent (CR 712.15, 712.15a): a Kessig Prowler that
    // entered the battlefield face down.
    let card = t
        .g
        .create_card_object(mtg_engine::card::card("Kessig Prowler"), P0, Zone::Nowhere);
    let down = t
        .g
        .move_object_ev(MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(P0),
            etb: EtbInfo {
                controller: Some(P0),
                face_down: Some(KeywordKind::Morph),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    t.g.recompute();
    assert!(t.obj(down).face_down);
    assert_eq!(
        t.obj(down).card.as_ref().unwrap().name,
        "Kessig Prowler // Sinuous Predator"
    );
    transform(&mut t, down);
    assert!(t.obj(down).face_down);
    assert_eq!(t.obj(down).face, FaceState::Front);
    assert_eq!(transformed_events(&t), 0);
    // A double-faced card does.
    transform(&mut t, prowler);
    assert_eq!(t.obj(prowler).face, FaceState::Back);
}

#[test]
fn a_modal_double_faced_permanent_can_transform() {
    cr!("701.27a", "712.9");
    supported("Monica Rambeau // Photon, Living Light");
    let mut t = TestGame::new(2);
    // "{2}{R}{W}{W}: Transform Monica Rambeau. Activate only as a sorcery."
    let monica = t.battlefield(P0, "Monica Rambeau // Photon, Living Light");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 4);
    t.set_step(P0, Step::PrecombatMain);
    let i = t
        .obj(monica)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains("Transform"))
        .expect("no transform ability");
    t.activate(P0, monica, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj(monica).face, FaceState::Back);
    assert_eq!(t.obj(monica).chars.name, "Photon, Living Light");
    assert!(t.g.is_live(monica));
    assert_eq!(transformed_events(&t), 1);
}

#[test]
fn a_permanent_doesnt_transform_into_an_instant_or_sorcery_face() {
    cr!("701.27d");
    let mut t = TestGame::new(2);
    // Invasion of Alara's back face is Awaken the Maelstrom, a sorcery.
    let siege = t.battlefield(P0, "Invasion of Alara");
    transform(&mut t, siege);
    assert_eq!(t.obj(siege).face, FaceState::Front);
    assert_eq!(t.zone(siege), Zone::Battlefield);
    assert_eq!(t.obj(siege).chars.name, "Invasion of Alara");
    // Soporific Springs, played as a land, would turn into Sink into Stupor, an instant.
    let springs = t.hand(P0, "Sink into Stupor // Soporific Springs");
    t.set_step(P0, Step::PrecombatMain);
    t.play_land(P0, springs).unwrap();
    let springs = t.g.current(springs);
    assert_eq!(t.obj(springs).chars.name, "Soporific Springs");
    transform(&mut t, springs);
    assert_eq!(t.obj(springs).face, FaceState::Back);
    assert_eq!(t.obj(springs).chars.name, "Soporific Springs");
    assert_eq!(transformed_events(&t), 0);
    // A creature face it can transform into.
    let monica = t.battlefield(P0, "Monica Rambeau // Photon, Living Light");
    transform(&mut t, monica);
    assert_eq!(t.obj(monica).chars.name, "Photon, Living Light");
    assert_eq!(transformed_events(&t), 1);
}

#[test]
fn transforms_into_triggers_on_the_quality_after_transforming() {
    cr!("701.27e");
    ruling!("Cult of the Waxing Moon", "Westvale Abbey and Thraben Gargoyle will both cause");
    ruling!(
        "Cult of the Waxing Moon",
        "put onto the battlefield transformed, such as Skin Shedder, will not cause"
    );
    supported("Cult of the Waxing Moon");
    supported("Thraben Gargoyle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cult of the Waxing Moon");
    // A non-Human creature before and after (Thraben Gargoyle, then Stonewing
    // Antagonizer).
    let gargoyle = t.battlefield(P0, "Thraben Gargoyle");
    transform(&mut t, gargoyle);
    assert_eq!(t.obj(gargoyle).chars.name, "Stonewing Antagonizer");
    assert_eq!(wolves(&t), 1);
    // A Human Werewolf that transforms into a (non-Human) Werewolf triggers it, but not
    // when it transforms back into a Human.
    let messenger = t.battlefield(P0, "Village Messenger");
    transform(&mut t, messenger);
    assert_eq!(t.obj(messenger).chars.name, "Moonrise Intruder");
    assert_eq!(wolves(&t), 2);
    transform(&mut t, messenger);
    assert_eq!(t.obj(messenger).chars.name, "Village Messenger");
    assert_eq!(wolves(&t), 2);
    // Entering the battlefield transformed isn't transforming.
    let card = t.hand(P0, "Kessig Prowler");
    let mut to = Destination::battlefield();
    to.transformed = true;
    run(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::All(Filter::Objects(vec![card]).in_zone(ZoneKind::Hand)),
            to,
        },
    );
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sinuous Predator").len(), 1);
    assert_eq!(wolves(&t), 2);
}

#[test]
fn an_ability_of_a_permanent_doesnt_transform_it_if_it_transformed_since_it_was_put_on_the_stack(
) {
    cr!("701.27f");
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, "Kessig Prowler");
    t.lands(P0, "Forest", 10);
    t.activate(P0, prowler, 0, &[]).unwrap();
    t.activate(P0, prowler, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.obj(prowler).face, FaceState::Back);
    // The first ability to be put onto the stack is ignored: it doesn't transform it back.
    t.resolve();
    assert_eq!(t.obj(prowler).face, FaceState::Back);
    assert_eq!(transformed_events(&t), 1);
    // A new activation of it (had it one) would; an effect of another source does.
    transform(&mut t, prowler);
    assert_eq!(t.obj(prowler).face, FaceState::Front);
}

#[test]
fn a_delayed_ability_doesnt_transform_it_if_it_transformed_since_it_was_created() {
    cr!("701.27f");
    ruling!(
        "Archangel Avacyn",
        "won't cause it to transform back into Archangel Avacyn if it has already transformed"
    );
    supported("Archangel Avacyn");
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P0, "Archangel Avacyn");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    // "When a non-Angel creature you control dies, transform Archangel Avacyn at the
    // beginning of the next upkeep." Two creatures die: two delayed triggered abilities.
    t.g.destroy(a, None);
    t.g.destroy(b, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.delayed_triggers.len(), 2);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    // It transformed once: "When this creature transforms into Avacyn, the Purifier, it
    // deals 3 damage to each other creature and each opponent."
    assert_eq!(t.obj(avacyn).chars.name, "Avacyn, the Purifier");
    assert_eq!(transformed_events(&t), 1);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_delayed_ability_counts_transformations_since_it_was_created_not_since_it_triggered() {
    cr!("701.27f");
    supported("Archangel Avacyn");
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P0, "Archangel Avacyn");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A non-Angel creature dies: the delayed triggered ability is created now.
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.delayed_triggers.len(), 1);
    // Another effect transforms Avacyn before the next upkeep.
    transform(&mut t, avacyn);
    assert_eq!(t.obj(avacyn).chars.name, "Avacyn, the Purifier");
    assert_eq!(transformed_events(&t), 1);
    // The delayed ability triggers and is put on the stack after that, but Avacyn has
    // transformed since it was created: it doesn't transform her back.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert!(t.g.delayed_triggers.is_empty());
    assert_eq!(t.obj(avacyn).chars.name, "Avacyn, the Purifier");
    // (No transformation this turn.)
    assert_eq!(transformed_events(&t), 0);
}

#[test]
fn a_transformed_permanent_is_a_double_faced_permanent_with_its_back_face_up() {
    cr!("701.27g");
    ruling!(
        "Mutagen Connoisseur",
        "melded permanents are never transformed permanents"
    );
    supported("Mutagen Connoisseur");
    let mut t = TestGame::new(2);
    // "This creature gets +1/+0 for each transformed permanent you control."
    let mutagen = t.battlefield(P0, "Mutagen Connoisseur");
    let base = t.pt(mutagen).0;
    let prowler = t.battlefield(P0, "Kessig Prowler");
    // An opponent's transformed permanent doesn't count.
    let theirs = t.battlefield(P1, "Thraben Gargoyle");
    transform(&mut t, theirs);
    assert_eq!(t.pt(mutagen).0, base);
    transform(&mut t, prowler);
    assert_eq!(t.pt(mutagen).0, base + 1);
    // Front face up again: not a transformed permanent, though it was one.
    transform(&mut t, prowler);
    assert_eq!(t.pt(mutagen).0, base);
    // A modal double-faced permanent that transformed is one too (CR 712.9).
    let monica = t.battlefield(P0, "Monica Rambeau // Photon, Living Light");
    assert_eq!(t.pt(mutagen).0, base);
    transform(&mut t, monica);
    assert_eq!(t.pt(mutagen).0, base + 1);
    transform(&mut t, monica);
    assert_eq!(t.pt(mutagen).0, base);
    // A melded permanent (back faces up) isn't one.
    t.battlefield(P0, "Graf Rats");
    t.battlefield(P0, "Midnight Scavengers");
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    let host = t.named_on_battlefield("Chittering Host");
    assert_eq!(host.len(), 1);
    assert!(!mtg_engine::transform_rules::is_transformed(&t.g, host[0]));
    // Only Chittering Host's "other creatures you control get +1/+0" applies.
    assert_eq!(t.pt(mutagen).0, base + 1);
}

#[test]
fn a_dies_ability_granted_while_you_control_a_transformed_permanent_looks_back() {
    cr!("701.27g", "603.10a");
    ruling!(
        "Oculus Whelp",
        "If Oculus Whelp dies at the same time as your transformed permanents"
    );
    supported("Oculus Whelp");
    let mut t = TestGame::new(2);
    // "As long as you control a transformed permanent, this creature has "When this
    // creature dies, draw a card.""
    t.battlefield(P0, "Oculus Whelp");
    let prowler = t.battlefield(P0, "Kessig Prowler");
    transform(&mut t, prowler);
    let hand = t.hand_size(P0);
    run(
        &mut t,
        P1,
        None,
        Effect::Destroy {
            what: Sel::All(Filter::Type(CardType::Creature)),
            no_regen: false,
        },
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
