//! Rulings batch P098 — "return target creature to its owner's hand" enters triggers
//! (Aethersnipe, Aether Adept, Mist Raven, Voidwielder, Separatist Voidmage, Lumengrid
//! Drake, Reflector Mage), Crush of Tentacles, Waterspout Elemental, Stolen by the Fae,
//! Ghitu Amplifier and Malevolent Whispers.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, destroy, target_candidates};
use crate::r_s06_common::attach_new;
use crate::r_s12_common::{morph, MORPH};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The only legal target of a mandatory "return target [creature / nonland permanent]"
/// enters trigger is the creature itself: it is offered just that, and it's returned.
#[test]
fn a_mandatory_bounce_trigger_with_only_itself_to_target_returns_itself() {
    cr!("603.3d");
    ruling!(
        "Aethersnipe",
        "Aethersnipe can be the target of its own ability. If it's the only nonland permanent on the battlefield, it must be the target of its ability."
    );
    ruling!(
        "Aether Adept",
        "If there are no other creatures on the battlefield when Aether Adept enters, it must target itself."
    );
    ruling!(
        "Mist Raven",
        "Mist Raven’s ability is mandatory. If Mist Raven is the only creature on the battlefield when its ability triggers, you must choose it as the target."
    );
    for name in ["Aethersnipe", "Aether Adept", "Mist Raven"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 2);
        t.lands(P1, "Forest", 2);
        let from = t.asked().len();
        let me = t.enter(P0, name);
        t.resolve_all();
        let offered = target_candidates(&t, P0, from);
        assert!(
            offered.iter().all(|c| c == &vec![obj(me)]),
            "{name}: {offered:?}"
        );
        assert!(t.in_hand(P0, name), "{name}");
    }
}

#[test]
fn optional_bounce_may_target_itself_and_need_not_return_it() {
    cr!("603.3d", "603.5");
    ruling!(
        "Voidwielder",
        "If Voidwielder is the only creature on the battlefield, you’ll have to choose it as the target of its ability, although you may choose to not return it to its owner’s hand."
    );
    ruling!(
        "Separatist Voidmage",
        "Separatist Voidmage’s ability can target itself."
    );
    for name in ["Voidwielder", "Separatist Voidmage"] {
        supported(name);
        // With another creature around, it's among the legal targets.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let from = t.asked().len();
        let me = t.enter(P0, name);
        t.settle();
        let offered = target_candidates(&t, P0, from);
        assert!(
            offered
                .iter()
                .any(|c| c.contains(&obj(me)) && c.contains(&obj(bears))),
            "{name}"
        );
        // Alone: it's the target, and returning it is optional.
        for yes in [false, true] {
            let mut t = TestGame::new(2);
            let from = t.asked().len();
            t.answer_yes(P0, yes);
            let me = t.enter(P0, name);
            t.resolve_all();
            let offered = target_candidates(&t, P0, from);
            assert!(offered.iter().all(|c| c == &vec![obj(me)]), "{name}");
            assert_eq!(t.in_hand(P0, name), yes, "{name} {yes}");
            assert_eq!(t.on_battlefield(me), !yes, "{name} {yes}");
        }
    }
}

#[test]
fn lumengrid_drake_metalcraft_bounce_is_mandatory_and_rechecked() {
    cr!("603.4");
    ruling!(
        "Lumengrid Drake",
        "The triggered ability is mandatory. If you control three or more artifacts as Lumengrid Drake enters, you must target a creature. If there are no other creatures on the battlefield, you must target Lumengrid Drake itself. As the ability resolves, if you control three or more artifacts, you must return the targeted creature to its owner's hand."
    );
    supported("Lumengrid Drake");
    let setup = |t: &mut TestGame| -> Vec<ObjectId> {
        (0..3).map(|_| t.battlefield(P0, "Sol Ring")).collect()
    };
    // No other creature: it targets itself and returns.
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.enter(P0, "Lumengrid Drake");
    t.resolve_all();
    assert!(t.in_hand(P0, "Lumengrid Drake"));
    // Another creature, and an artifact leaves before resolution: nothing returns.
    let mut t = TestGame::new(2);
    let rings = setup(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let drake = t.enter(P0, "Lumengrid Drake");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, rings[0]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(drake));
    // Two artifacts as it enters: it doesn't trigger.
    let mut t = TestGame::new(2);
    let rings = setup(&mut t);
    destroy(&mut t, rings[0]);
    t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Lumengrid Drake");
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn crush_of_tentacles_creates_the_octopus_after_everything_is_returned() {
    cr!("608.2c", "702.117a");
    ruling!(
        "Crush of Tentacles",
        "All other nonland permanents will already be in their owners' hands by the time the Octopus token is created. Any triggered abilities or abilities that would have otherwise affected that Octopus won't be around to matter."
    );
    supported("Crush of Tentacles");
    let mut t = TestGame::new(2);
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    t.battlefield(P1, "Soul Warden");
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Grizzly Bears");
    // A spell cast first this turn enables surge.
    let orni = t.hand(P0, "Ornithopter");
    t.cast(P0, orni).go();
    t.resolve_all();
    let crush = t.hand(P0, "Crush of Tentacles");
    t.lands(P0, "Island", 5);
    t.cast(P0, crush)
        .method(CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Surge,
        ))
        .go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Soul Warden") && t.in_hand(P1, "Soul Warden"));
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Ornithopter"));
    let octopus: Vec<_> = tokens(&t, P0);
    assert_eq!(octopus.len(), 1);
    assert_eq!(t.pt(octopus[0]), (8, 8));
    // Ornithopter entering gained each 1 life; the Octopus gains nobody anything.
    assert_eq!((t.life(P0), t.life(P1)), (21, 21));
}

#[test]
fn waterspout_elemental_returns_your_creatures_too() {
    cr!("702.33d");
    ruling!(
        "Waterspout Elemental",
        "It affects your creatures as well as your opponent's."
    );
    supported("Waterspout Elemental");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Waterspout Elemental");
    t.lands(P0, "Island", 6);
    let spell = t.cast(P0, card).kicked(true).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(t.on_battlefield(t.g.current(spell)));
}

#[test]
fn stolen_by_the_fae_illegal_target_makes_no_faeries() {
    cr!("608.2b");
    ruling!(
        "Stolen by the Fae",
        "If the target creature is an illegal target by the time Stolen by the Fae tries to resolve, the spell won't resolve. You won't create any Faerie tokens."
    );
    supported("Stolen by the Fae");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let card = t.hand(P0, "Stolen by the Fae");
        t.lands(P0, "Island", 4);
        t.cast(P0, card).x(2).target(bears).go();
        if leaves {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), if leaves { 0 } else { 2 }, "{leaves}");
        assert_eq!(t.in_hand(P1, "Grizzly Bears"), !leaves);
    }
}

#[test]
fn ghitu_amplifier_pump_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3");
    ruling!(
        "Ghitu Amplifier",
        "Ghitu Amplifier's last triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    supported("Ghitu Amplifier");
    let mut t = TestGame::new(2);
    let amp = t.battlefield(P0, "Ghitu Amplifier");
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let counter = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.pt(amp), (3, 2));
}

/// P0's Reflector Mage enters returning `target`; it's then P1's main phase, before P0's
/// next turn.
fn reflect(t: &mut TestGame, target: ObjectId) {
    supported("Reflector Mage");
    t.answer_targets(P0, &[obj(target)]);
    t.enter(P0, "Reflector Mage");
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
}

#[test]
fn reflector_mage_forbids_only_the_name_it_had_on_the_battlefield() {
    cr!("201.2a", "708.2", "707.2");
    ruling!(
        "Reflector Mage",
        "If the returned creature had no name while it was on the battlefield (likely because the creature was face down), its owner can still cast face-down creature spells with morph or megamorph, or cast the same card face up, before your next turn."
    );
    ruling!(
        "Reflector Mage",
        "In several cases, the name of the creature that was returned won’t match the name of that card once it’s back in its owner’s hand. For example, if the card was copying another creature, it will probably have a different name in its owner’s hand and can be cast again before your next turn. The same is true if a double-faced card with its back face up is returned to its owner’s hand this way."
    );
    ruling!(
        "Reflector Mage",
        "Reflector Mage’s ability doesn’t stop any player from playing lands (in the case that the returned creature was also a land)."
    );
    // Grizzly Bears returned: Grizzly Bears can't be cast until P0's next turn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    reflect(&mut t, bears);
    let card = t.g.players[P1.idx()].hand[0];
    let other = t.hand(P1, "Grizzly Bears");
    let elves = t.hand(P1, "Llanowar Elves");
    t.lands(P1, "Forest", 2);
    assert!(!can_cast(&mut t, P1, card, CastMethod::Normal));
    assert!(!can_cast(&mut t, P1, other, CastMethod::Normal));
    assert!(can_cast(&mut t, P1, elves, CastMethod::Normal));
    t.advance_to(P0, Step::Upkeep);
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_cast(&mut t, P1, card, CastMethod::Normal));
    // A face-down Exalted Angel.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let angel = morph(&mut t, P1, "Exalted Angel");
    t.set_step(P0, Step::PrecombatMain);
    reflect(&mut t, angel);
    let card = t.g.players[P1.idx()].hand[0];
    assert_eq!(t.obj(card).chars.name, "Exalted Angel");
    t.lands(P1, "Plains", 6);
    assert!(can_cast(&mut t, P1, card, CastMethod::Normal));
    assert!(can_cast(&mut t, P1, card, MORPH));
    // A Clone copying Grizzly Bears: Clone can be cast, Grizzly Bears can't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let bears = t.g.permanents().next().unwrap().id;
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(bears)]);
    let clone = t.enter(P1, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    reflect(&mut t, clone);
    let clone_card = t.g.players[P1.idx()].hand[0];
    let bears_card = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Island", 4);
    t.lands(P1, "Forest", 2);
    assert!(can_cast(&mut t, P1, clone_card, CastMethod::Normal));
    assert!(!can_cast(&mut t, P1, bears_card, CastMethod::Normal));
    // Insectile Aberration (Delver of Secrets transformed): Delver can be cast.
    let mut t = TestGame::new(2);
    let delver = t.battlefield(P1, "Delver of Secrets // Insectile Aberration");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.recompute();
    assert_eq!(t.obj_now(delver).chars.name, "Insectile Aberration");
    reflect(&mut t, delver);
    let card = t.g.players[P1.idx()].hand[0];
    t.lands(P1, "Island", 1);
    assert!(can_cast(&mut t, P1, card, CastMethod::Normal));
    // Dryad Arbor (a land creature) can still be played as a land.
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    reflect(&mut t, arbor);
    let card = t.g.players[P1.idx()].hand[0];
    assert!(crate::r_s02_common::can_play_land(&mut t, P1, card));
    t.play_land(P1, card).expect("play Dryad Arbor");
}

#[test]
fn malevolent_whispers_any_creature_and_not_its_attachments() {
    cr!("303.4e", "301.5d");
    ruling!(
        "Malevolent Whispers",
        "Malevolent Whispers can target any creature, even one that's untapped or one you already control."
    );
    ruling!(
        "Malevolent Whispers",
        "Gaining control of a creature doesn't cause you to gain control of any Auras or Equipment attached to it, even though those remain attached to it."
    );
    supported("Malevolent Whispers");
    // P0's own untapped creature.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Malevolent Whispers");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, card).target(mine).go();
    t.resolve_all();
    assert_eq!(t.pt(mine), (4, 2));
    // An opponent's creature with an Aura and an Equipment.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(&mut t, P1, "Holy Strength", theirs);
    let eq = attach_new(&mut t, P1, "Bonesplitter", theirs);
    let card = t.hand(P0, "Malevolent Whispers");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, card).target(theirs).go();
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert_eq!(t.obj_now(aura).controller, P1);
    assert_eq!(t.obj_now(eq).controller, P1);
    assert_eq!(crate::r_s06_common::attached_to(&t, eq), Some(obj(theirs)));
    assert_eq!(t.pt(theirs), (7, 4));
}
