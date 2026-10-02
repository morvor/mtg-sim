//! Rulings batch P058 — flicker effects: tokens that leave the battlefield cease to exist
//! (CR 111.7, 111.8), double-faced and melded cards return front face up (CR 712.14,
//! 712.21), "at the beginning of the next end step" when it's already the end step (CR
//! 603.7, 513.1), sources that leave before their abilities resolve (CR 610.3c,
//! 113.7a), and exile effects that last until a permanent leaves (CR 610.3).

use crate::r_p058_common::*;
use crate::r_s01_common::{custom_card, supported};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s06_common::{activate_containing, attached_to};
use crate::r_s12_common::morph;
use crate::r_s17_common::{enter_transformed, face, name_of, token_copy};
use mtg_engine::decision::Decision;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Whether the token is gone: neither on the battlefield nor in exile (CR 111.8).
fn gone(t: &TestGame, token: ObjectId) -> bool {
    let z = t.zone(token);
    z != Zone::Battlefield && !t.g.exile.contains(&t.g.current(token))
}

#[test]
fn tokens_exiled_by_flicker_spells_dont_return() {
    cr!("111.7", "111.8", "400.7");
    ruling!(
        "Displace",
        "If a creature token is exiled this way, it will cease to exist and will not return to the battlefield."
    );
    ruling!(
        "Illusionist's Stratagem",
        "If a creature token is exiled this way, it will cease to exist and will not return to the battlefield."
    );
    ruling!(
        "Acrobatic Maneuver",
        "If a creature token is exiled this way, it will cease to exist and won't return to the battlefield."
    );
    ruling!(
        "Essence Flux",
        "If a creature token is exiled, it ceases to exist. It won’t be returned to the battlefield."
    );
    ruling!(
        "Eerie Interlude",
        "If a creature token is exiled, it ceases to exist. It won’t be returned to the battlefield."
    );
    ruling!(
        "Long Road Home",
        "If a creature token is exiled this way, it will cease to exist and won’t return to the battlefield."
    );
    ruling!(
        "Settle Beyond Reality",
        "If a token you control is exiled this way, it will cease to exist and won't return to the battlefield."
    );
    for name in [
        "Displace",
        "Illusionist's Stratagem",
        "Acrobatic Maneuver",
        "Essence Flux",
        "Eerie Interlude",
        "Long Road Home",
        "Settle Beyond Reality",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        mana(&mut t, P0, 3);
        let soldier = create_token(&mut t, P0, "Soldier");
        let spell = t.hand(P0, name);
        let c = t.cast(P0, spell);
        let c = if name == "Settle Beyond Reality" {
            c.modes(&[1])
        } else {
            c
        };
        c.target(soldier).go();
        t.resolve_all();
        through_end_step(&mut t, P0);
        assert!(gone(&t, soldier), "{name}");
        assert!(t.g.permanents().all(|o| !o.is_token()), "{name}");
    }
}

#[test]
fn tokens_exiled_by_flicker_abilities_dont_return() {
    cr!("111.7", "111.8", "610.3");
    ruling!(
        "Eldrazi Displacer",
        "If a creature token is exiled this way, it will cease to exist and will not return to the battlefield."
    );
    // "{2}{C}: Exile another target creature, then return it to the battlefield tapped
    // under its owner's control."
    supported("Eldrazi Displacer");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let d = t.battlefield(P0, "Eldrazi Displacer");
    let soldier = create_token(&mut t, P1, "Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    activate_containing(&mut t, P0, d, "Exile another target").unwrap();
    t.resolve_all();
    assert!(gone(&t, soldier));
    // A card returns, tapped.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, d, "Exile another target").unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
    assert_ne!(t.g.current(bears), bears);

    ruling!(
        "Fairgrounds Warden",
        "If a creature token is exiled this way, it will cease to exist and won't return to the battlefield."
    );
    ruling!(
        "Aligned Hedron Network",
        "If a creature token is exiled, it ceases to exist. It won’t be returned to the battlefield."
    );
    // Fairgrounds Warden: "When this creature enters, exile target creature an opponent
    // controls until this creature leaves the battlefield."
    supported("Fairgrounds Warden");
    let mut t = TestGame::new(2);
    let soldier = create_token(&mut t, P1, "Soldier");
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    let warden = t.enter(P0, "Fairgrounds Warden");
    t.resolve_all();
    assert!(gone(&t, soldier));
    destroy(&mut t, warden);
    t.resolve_all();
    assert!(t.g.permanents().all(|o| !o.is_token()));
    // Aligned Hedron Network: "When this artifact enters, exile all creatures with power 5
    // or greater until this artifact leaves the battlefield."
    supported("Aligned Hedron Network");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let copy = token_copy(&mut t, P1, wurm)[0];
    let network = t.enter(P0, "Aligned Hedron Network");
    t.resolve_all();
    assert!(gone(&t, copy));
    assert!(t.in_exile("Craw Wurm"));
    destroy(&mut t, network);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1, "only the card");
    assert!(t.g.permanents().all(|o| !o.is_token()));
}

#[test]
fn a_token_copy_of_flickering_spirit_doesnt_return() {
    cr!("111.7", "111.8", "707.2");
    ruling!(
        "Flickering Spirit",
        "If a token copy of Flickering Spirit uses its activated ability, the token will not return to the battlefield"
    );
    supported("Flickering Spirit");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let spirit = t.graveyard(P0, "Flickering Spirit");
    let token = token_copy(&mut t, P0, spirit)[0];
    activate_containing(&mut t, P0, token, "then return it").unwrap();
    t.resolve_all();
    assert!(gone(&t, token));
    assert!(t.named_on_battlefield("Flickering Spirit").is_empty());
}

#[test]
fn double_faced_and_melded_cards_return_front_face_up() {
    cr!("712.14", "712.21", "400.7");
    ruling!(
        "Eerie Interlude",
        "If a double-faced card is exiled, it will return with its front face up."
    );
    ruling!(
        "Essence Flux",
        "If a double-faced card is exiled, it will return with its front face up."
    );
    for name in ["Eerie Interlude", "Essence Flux"] {
        let mut t = TestGame::new(2);
        mana(&mut t, P0, 2);
        let wolf = enter_transformed(&mut t, P0, "Kessig Prowler // Sinuous Predator");
        assert_eq!(name_of(&t, wolf), "Sinuous Predator");
        let spell = t.hand(P0, name);
        t.cast(P0, spell).target(wolf).go();
        t.resolve_all();
        through_end_step(&mut t, P0);
        assert!(t.on_battlefield(wolf), "{name}");
        assert_eq!(face(&t, wolf), FaceState::Front);
        assert_eq!(name_of(&t, wolf), "Kessig Prowler");
    }
    // Long Road Home: "Exile target creature. At the beginning of the next end step,
    // return that card to the battlefield under its owner's control with a +1/+1 counter
    // on it." Ormendahl, Profane Prince returns as Westvale Abbey (a land), with the
    // counter anyway.
    ruling!(
        "Long Road Home",
        "If a double-faced card is exiled this way, it’ll return with its front face up. It gets a +1/+1 counter even if it isn’t a creature."
    );
    supported("Long Road Home");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let abbey = enter_transformed(&mut t, P0, "Westvale Abbey // Ormendahl, Profane Prince");
    assert_eq!(name_of(&t, abbey), "Ormendahl, Profane Prince");
    let spell = t.hand(P0, "Long Road Home");
    t.cast(P0, spell).target(abbey).go();
    t.resolve_all();
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(abbey));
    assert_eq!(name_of(&t, abbey), "Westvale Abbey");
    assert!(!t.obj_now(abbey).is(mtg_engine::types::CardType::Creature));
    assert_eq!(t.counters(abbey, counters::PLUS1), 1);
    // A melded permanent: both cards return front face up, each with a counter.
    ruling!(
        "Long Road Home",
        "If a melded permanent is exiled this way, the two cards will both return with their front faces up. Each gets a +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fang, Fearless l'Cie");
    t.battlefield(P0, "Vanille, Cheerful l'Cie");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let r = t.named_on_battlefield("Ragnarok, Divine Deliverance");
    assert_eq!(r.len(), 1, "{}", t.dump_log());
    mana(&mut t, P0, 2);
    let spell = t.hand(P0, "Long Road Home");
    t.cast(P0, spell).target(r[0]).go();
    t.resolve_all();
    through_end_step(&mut t, P0);
    for n in ["Fang, Fearless l'Cie", "Vanille, Cheerful l'Cie"] {
        let on = t.named_on_battlefield(n);
        assert_eq!(on.len(), 1, "{n}: {}", t.dump_log());
        assert_eq!(t.counters(on[0], counters::PLUS1), 1, "{n}");
    }
}

/// P0 casts `name` targeting `target` during P0's end step; the card is still in exile
/// after that turn, and returns at the next turn's end step.
fn exiled_in_end_step(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::End);
    let spell = t.hand(P0, name);
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{name}");
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_exile("Grizzly Bears"), "{name}: not this turn");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{name}: the next turn's end step");
}

#[test]
fn exiled_during_an_end_step_returns_at_the_next_turns_end_step() {
    cr!("603.7", "513.1");
    ruling!(
        "Otherworldly Journey",
        "If Otherworldly Journey is cast during an end step, the creature won't return until the beginning of the next end step."
    );
    ruling!(
        "Long Road Home",
        "If that creature is exiled during an end step, it won’t come back until the next turn’s end step."
    );
    exiled_in_end_step("Otherworldly Journey");
    exiled_in_end_step("Long Road Home");
    // Planar Guide: "{3}{W}, Exile this creature: Exile all creatures. At the beginning
    // of the next end step, return those cards to the battlefield under their owners'
    // control."
    ruling!(
        "Planar Guide",
        "If the ability is activated during the End step, then the creatures do not return until the End step of the following turn."
    );
    supported("Planar Guide");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let guide = t.battlefield(P0, "Planar Guide");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.set_step(P0, Step::End);
    activate_containing(&mut t, P0, guide, "Exile all creatures").unwrap();
    t.resolve_all();
    assert!(t.in_exile("Gray Ogre"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_exile("Gray Ogre"));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(ogre));
    assert_eq!(t.obj_now(ogre).controller, P1);
    assert!(t.in_exile("Planar Guide"), "exiled as a cost, not returned");
}

#[test]
fn otherworldly_journey_counters_auras_and_equipment() {
    cr!("400.7", "704.5m", "704.5n", "614.1c");
    ruling!(
        "Otherworldly Journey",
        "Any Aura attached to the creature is put into its owner's graveyard as a state-based action. Equipment attached to the creature become unattached but remain on the battlefield."
    );
    ruling!(
        "Otherworldly Journey",
        "The creature comes back onto the battlefield as a new creature, with one +1/+1 counter on it (plus any other counters that it would normally enter with)."
    );
    supported("Otherworldly Journey");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let def = custom_card(
        "Counter Beast",
        "Creature — Beast",
        "{2}{G}",
        Some((1, 1)),
        "This creature enters with two +1/+1 counters on it.",
    );
    let beast = t.custom(P0, def, Zone::Battlefield);
    let d = decorate(&mut t, P0, beast);
    let spell = t.hand(P0, "Otherworldly Journey");
    t.cast(P0, spell).target(beast).go();
    t.resolve_all();
    assert_eq!(t.zone(d.aura), Zone::Graveyard(P0));
    assert!(t.on_battlefield(d.equipment));
    assert_eq!(attached_to(&t, d.equipment), None);
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(beast));
    assert_eq!(t.counters(beast, counters::PLUS1), 3);
    assert_eq!(t.pt(beast), (4, 4));
}

#[test]
fn planar_guide_exiles_and_returns_all_creatures() {
    cr!("400.7", "603.6a", "708.10", "111.8", "704.5m");
    ruling!("Planar Guide", "All \"enters\" abilities trigger as normal.");
    ruling!(
        "Planar Guide",
        "Creatures that were face down return to the battlefield face up."
    );
    ruling!(
        "Planar Guide",
        "When the creatures leave the battlefield, all Auras on them go to their owners' graveyards, any counters on them are removed, and effects on them end. Token creatures do not return to the battlefield."
    );
    supported("Planar Guide");
    let mut t = TestGame::new(2);
    let haruspex = morph(&mut t, P0, "Grim Haruspex");
    mana(&mut t, P0, 2);
    let guide = t.battlefield(P0, "Planar Guide");
    let missionary = t.battlefield(P0, "Lone Missionary");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let d = decorate(&mut t, P1, ogre);
    let soldier = create_token(&mut t, P1, "Soldier");
    pump_eot(&mut t, missionary);
    assert_eq!(t.pt(missionary), (4, 1));
    activate_containing(&mut t, P0, guide, "Exile all creatures").unwrap();
    t.resolve_all();
    assert!(t.in_exile("Gray Ogre") && t.in_exile("Lone Missionary"));
    assert!(gone(&t, soldier));
    assert_eq!(t.zone(d.aura), Zone::Graveyard(P1));
    through_end_step(&mut t, P0);
    // "When this creature enters, you gain 4 life."
    assert_eq!(t.life(P0), 24);
    assert!(t.on_battlefield(missionary));
    assert_eq!(t.pt(missionary), (2, 1), "the pump ended");
    assert!(t.on_battlefield(haruspex));
    assert!(!t.obj_now(haruspex).face_down);
    assert_eq!(name_of(&t, haruspex), "Grim Haruspex");
    assert_new_object(&t, &d);
    assert!(t.g.permanents().all(|o| !o.is_token()));
}

/// Gives `id` +2/+0 until end of turn.
fn pump_eot(t: &mut TestGame, id: ObjectId) {
    use mtg_engine::ability::*;
    crate::r_s05_common::run_from(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![id])),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
}

#[test]
fn liberate_returns_after_end_step_abilities_trigger() {
    cr!("603.7", "513.1", "603.6a");
    ruling!(
        "Liberate",
        "The creature returns after “at the beginning of the end step” abilities trigger, so any such abilities on the creature will not trigger this turn."
    );
    supported("Liberate");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let def = custom_card(
        "Dusk Keeper",
        "Creature — Cleric",
        "{1}{W}",
        Some((1, 1)),
        "At the beginning of your end step, you gain 1 life.",
    );
    let keeper = t.custom(P0, def, Zone::Battlefield);
    let spell = t.hand(P0, "Liberate");
    t.cast(P0, spell).target(keeper).go();
    t.resolve_all();
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(keeper));
    assert_eq!(t.life(P0), 20, "its end step ability didn't trigger");
    // It does on P0's next turn.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn galepowder_mage_targets_and_returns() {
    cr!("603.3d", "115.1", "603.7");
    ruling!(
        "Galepowder Mage",
        "Galepowder Mage's ability can target a creature controlled by any player. It's not optional."
    );
    ruling!(
        "Galepowder Mage",
        "The exiled card will be returned to the battlefield at the beginning of the end step even if Galepowder Mage is no longer on the battlefield at that time."
    );
    supported("Galepowder Mage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Galepowder Mage");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    crate::r_s01_common::attack_with(&mut t, &[(mage, Entity::Player(P1))]);
    let asked = t.asked()[from..].to_vec();
    let spec = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets {
                min, candidates, ..
            } => Some((*min, candidates.clone())),
            _ => None,
        })
        .expect("a target choice");
    assert_eq!(spec.0, 1, "not optional");
    assert!(spec.1.contains(&Entity::Object(ogre)));
    t.resolve_all();
    assert!(t.in_exile("Gray Ogre"));
    destroy(&mut t, mage);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(ogre));
    assert_eq!(t.obj_now(ogre).controller, P1);

    ruling!(
        "Galepowder Mage",
        "If Galepowder Mage is the only creature on the battlefield when it attacks, its ability has no effect."
    );
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Galepowder Mage");
    crate::r_s01_common::attack_with(&mut t, &[(mage, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(mage));
    assert_eq!(t.g.current(mage), mage, "it isn't exiled");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.g.current(mage), mage);
}

#[test]
fn exile_until_leaves_when_the_source_left_first() {
    cr!("610.3c", "603.3");
    ruling!(
        "All-Fates Stalker",
        "If All-Fates Stalker leaves the battlefield before its first ability resolves, the target creature won’t be exiled."
    );
    ruling!(
        "Touch the Spirit Realm",
        "If Touch the Spirit Realm leaves the battlefield before its first ability resolves, the target permanent won't be exiled."
    );
    for name in ["All-Fates Stalker", "Touch the Spirit Realm"] {
        supported(name);
        let mut t = TestGame::new(2);
        let ogre = t.battlefield(P1, "Gray Ogre");
        t.answer_targets(P0, &[Entity::Object(ogre)]);
        let src = t.enter(P0, name);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}");
        destroy(&mut t, src);
        t.resolve_all();
        assert_eq!(t.g.current(ogre), ogre, "{name}: not exiled");
        assert!(t.on_battlefield(ogre));
    }
}

#[test]
fn angel_of_condemnation_leaving_before_its_abilities_resolve() {
    cr!("610.3c", "603.7", "113.7a", "506.4");
    ruling!(
        "Angel of Condemnation",
        "If Angel of Condemnation leaves the battlefield before its first activated ability resolves, the target creature is still exiled. That card returns to the battlefield even if Angel of Condemnation has left the battlefield before the next end step."
    );
    supported("Angel of Condemnation");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let angel = t.battlefield(P0, "Angel of Condemnation");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    activate_containing(&mut t, P0, angel, "beginning of the next end step").unwrap();
    destroy(&mut t, angel);
    t.resolve_all();
    assert!(t.in_exile("Gray Ogre"));
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(ogre));

    ruling!(
        "Angel of Condemnation",
        "If Angel of Condemnation leaves the battlefield before its last ability resolves, the target creature won’t be exiled."
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let angel = t.battlefield(P0, "Angel of Condemnation");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    activate_containing(&mut t, P0, angel, "Exert").unwrap();
    destroy(&mut t, angel);
    t.resolve_all();
    assert_eq!(t.g.current(ogre), ogre);
    assert!(t.on_battlefield(ogre));

    ruling!(
        "Angel of Condemnation",
        "The card exiled with Angel of Condemnation’s last ability returns to the battlefield immediately after Angel of Condemnation leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let angel = t.battlefield(P0, "Angel of Condemnation");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    activate_containing(&mut t, P0, angel, "Exert").unwrap();
    t.resolve_all();
    assert!(t.in_exile("Gray Ogre"));
    destroy(&mut t, angel);
    // No ability to resolve: it's back already.
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(ogre));

    ruling!(
        "Angel of Condemnation",
        "Tapping Angel of Condemnation to activate either of its abilities while it’s attacking doesn’t remove it from combat."
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let angel = t.battlefield(P0, "Angel of Condemnation");
    let ogre = t.battlefield(P1, "Gray Ogre");
    crate::r_s01_common::attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    assert!(!t.obj_now(angel).tapped, "vigilance");
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    activate_containing(&mut t, P0, angel, "beginning of the next end step").unwrap();
    t.resolve_all();
    assert!(t.obj_now(angel).tapped);
    assert!(crate::r_s10_common::attacking(&t, angel));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn koya_leaving_doesnt_stop_its_delayed_ability() {
    cr!("603.7", "603.7c", "118.12");
    ruling!(
        "Koya, Death from Above",
        "If Koya leaves the battlefield before the delayed triggered ability goes on the stack at the beginning of the next end step, the delayed triggered ability will still resolve as normal."
    );
    supported("Koya, Death from Above");
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        mana(&mut t, P0, 2);
        let ogre = t.battlefield(P1, "Gray Ogre");
        t.answer_targets(P0, &[Entity::Object(ogre)]);
        let koya = t.enter(P0, "Koya, Death from Above");
        t.resolve_all();
        assert!(t.in_exile("Gray Ogre"));
        destroy(&mut t, koya);
        t.answer_yes(P0, pay);
        through_end_step(&mut t, P0);
        if pay {
            assert!(t.in_exile("Gray Ogre"), "paid: it stays exiled");
        } else {
            assert!(t.on_battlefield(ogre), "{}", t.dump_log());
            assert_eq!(t.obj_now(ogre).controller, P1);
        }
    }
}

#[test]
fn roons_target_commander_may_go_to_the_command_zone() {
    cr!("903.9a", "400.7");
    ruling!(
        "Roon of the Hidden Realm",
        "If Roon of the Hidden Realm's ability targets a commander, that card's owner may choose to have it go to the command zone after it is exiled. If they do, it will not be returned to the battlefield at the beginning of the next end step."
    );
    supported("Roon of the Hidden Realm");
    let mut t = crate::r_s13_common::commander_game();
    let cmdr = crate::r_s13_common::commander(&mut t, P1, "Gray Ogre");
    let cmdr = crate::r_s05_common::move_to(&mut t, cmdr, Zone::Battlefield).unwrap();
    mana(&mut t, P0, 2);
    let roon = t.battlefield(P0, "Roon of the Hidden Realm");
    t.answer_targets(P0, &[Entity::Object(cmdr)]);
    t.answer_yes(P1, true);
    activate_containing(&mut t, P0, roon, "Exile another target").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(cmdr), Zone::Command);
    through_end_step(&mut t, P0);
    assert_eq!(t.zone(cmdr), Zone::Command);
    assert!(t.named_on_battlefield("Gray Ogre").is_empty());
}

#[test]
fn the_princess_takes_flight_returns_every_exiled_card() {
    cr!("714.2b", "707.10", "607.2a");
    ruling!(
        "The Princess Takes Flight",
        "If The Princess Takes Flight's first chapter ability exiled more than one creature (usually because the ability was copied), its third chapter ability will return all of the exiled cards"
    );
    supported("The Princess Takes Flight");
    supported("Strionic Resonator");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let resonator = t.battlefield(P0, "Strionic Resonator");
    // Chapter I on the stack targeting the Ogre; Strionic Resonator copies it ("Copy
    // target triggered ability you control. You may choose new targets for the copy."),
    // and the copy exiles the Bears.
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    let saga = t.enter(P0, "The Princess Takes Flight");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let chapter = *t.g.stack.last().unwrap();
    t.answer_targets(P0, &[Entity::Object(chapter)]);
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(Entity::Object(bears))]);
    activate_containing(&mut t, P0, resonator, "Copy target triggered").unwrap();
    t.resolve_all();
    assert!(t.in_exile("Gray Ogre") && t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    // Chapters II and III.
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.answer_targets(P0, &[]);
    t.resolve_all();
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert!(t.on_battlefield(ogre), "{}", t.dump_log());
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}
