//! Rulings batch P058 — a flickered permanent returns as a new object (CR 400.7): the
//! counters on it cease to exist, Auras attached to it are put into their owners'
//! graveyards (CR 704.5m) and Equipment becomes unattached (CR 704.5n), whether it
//! returns immediately, at the beginning of the next end step (CR 603.7), or when the
//! exiling permanent leaves the battlefield (CR 610.3).

use crate::r_p058_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls a decorated Grizzly Bears and plenty of lands; `act` flickers it.
fn flicker_bears(card_name: &str, act: impl FnOnce(&mut TestGame, ObjectId)) {
    supported(card_name);
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let d = decorate(&mut t, P0, bears);
    act(&mut t, bears);
    assert_new_object(&t, &d);
}

/// Casts the instant or sorcery `name` from P0's hand targeting `target`, and resolves.
fn cast_at(t: &mut TestGame, name: &str, target: ObjectId) {
    let spell = t.hand(P0, name);
    t.cast(P0, spell).target(target).go();
    t.resolve_all();
}

#[test]
fn spells_that_flicker_your_creature() {
    cr!("400.7", "704.5m", "704.5n", "122.2");
    ruling!(
        "Displace",
        "After each creature returns to the battlefield, it will be a new object with no connection to the creature that was exiled."
    );
    ruling!(
        "Acrobatic Maneuver",
        "After the creature returns to the battlefield, it will be a new object with no connection to the creature that was exiled."
    );
    ruling!(
        "Essence Flux",
        "Auras attached to the exiled creature will be put into their owners’ graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    ruling!(
        "Illusionist's Stratagem",
        "Each card that returns to the battlefield will be a new object with no connection to the creature that was exiled."
    );
    ruling!(
        "Ephemerate",
        "Once the exiled creature returns, it's considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Slip On the Ring",
        "Auras attached to the exiled creatures will be put into their owners' graveyards. Equipment attached to the exiled creatures will become unattached"
    );
    ruling!(
        "Justiciar's Portal",
        "Once the exiled creature returns, it’s considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Cloudshift",
        "Once the exiled permanent returns, it's considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Momentary Blink",
        "When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Blur",
        "When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    for name in [
        "Displace",
        "Acrobatic Maneuver",
        "Essence Flux",
        "Illusionist's Stratagem",
        "Ephemerate",
        "Slip On the Ring",
        "Justiciar's Portal",
        "Cloudshift",
        "Momentary Blink",
        "Blur",
        "Go Ninja Go",
    ] {
        flicker_bears(name, |t, bears| {
            if name == "Go Ninja Go" {
                // "Choose one or both — • Exile target creature you control, then return
                // it to the battlefield under its owner's control. • ..."
                let spell = t.hand(P0, name);
                t.cast(P0, spell).modes(&[0]).target(bears).go();
                t.resolve_all();
            } else {
                cast_at(t, name, bears);
            }
        });
    }
    ruling!(
        "Go Ninja Go",
        "Once the exiled creature returns, it's considered a new object with no relation to the object that it was."
    );
}

#[test]
fn settle_beyond_reality_and_rescue_the_foal() {
    cr!("400.7", "704.5m", "704.5n", "715.3");
    ruling!(
        "Settle Beyond Reality",
        "Once the exiled creature you control returns, it's considered a new object with no relation to the object that it was."
    );
    // "Choose one or both — • Exile target creature you don't control. • Exile target
    // creature you control, then return it to the battlefield under its owner's control."
    flicker_bears("Settle Beyond Reality", |t, bears| {
        let spell = t.hand(P0, "Settle Beyond Reality");
        t.cast(P0, spell).modes(&[1]).target(bears).go();
        t.resolve_all();
    });
    ruling!(
        "Pegasus Guardian // Rescue the Foal",
        "When the card exiled by Rescue the Foal returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    // The Adventure: "Exile target creature you control, then return that card to the
    // battlefield under its owner's control."
    flicker_bears("Pegasus Guardian // Rescue the Foal", |t, bears| {
        let card = t.hand(P0, "Pegasus Guardian // Rescue the Foal");
        t.cast(P0, card)
            .method(CastMethod::Half(1))
            .target(bears)
            .go();
        t.resolve_all();
        assert!(t.in_exile("Pegasus Guardian // Rescue the Foal") || t.in_exile("Pegasus Guardian"));
    });
}

#[test]
fn abilities_that_flicker_a_creature_immediately() {
    cr!("400.7", "704.5m", "704.5n", "603.6a");
    ruling!(
        "Distinguished Conjurer",
        "When the card exiled by the second ability returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    // "{4}{W}, {T}: Exile another target creature you control, then return it to the
    // battlefield under its owner's control."
    flicker_bears("Distinguished Conjurer", |t, bears| {
        let c = t.battlefield(P0, "Distinguished Conjurer");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        activate_containing(t, P0, c, "Exile another target").unwrap();
        t.resolve_all();
    });
    ruling!(
        "Felidar Guardian",
        "After the permanent returns to the battlefield, it will be a new object with no connection to the permanent that was exiled."
    );
    // "When this creature enters, you may exile another target permanent you control,
    // then return that card to the battlefield under its owner's control."
    flicker_bears("Felidar Guardian", |t, bears| {
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.answer_yes(P0, true);
        t.enter(P0, "Felidar Guardian");
        t.resolve_all();
    });
    ruling!(
        "Restoration Angel",
        "Once the exiled creature returns, it's considered a new object with no relation to the object that it was."
    );
    flicker_bears("Restoration Angel", |t, bears| {
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.answer_yes(P0, true);
        t.enter(P0, "Restoration Angel");
        t.resolve_all();
    });
    // "At the beginning of your end step, you may exile target creature you control,
    // then return that card to the battlefield under your control."
    ruling!(
        "Conjurer's Closet",
        "Once the exiled creature returns, it's considered a new object with no relation to the object that it was."
    );
    flicker_bears("Conjurer's Closet", |t, bears| {
        t.battlefield(P0, "Conjurer's Closet");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.answer_yes(P0, true);
        through_end_step(t, P0);
    });
    ruling!(
        "Teleportation Circle",
        "Once a permanent exiled this way returns, it's considered a new object with no relation to the object that it was."
    );
    flicker_bears("Teleportation Circle", |t, bears| {
        t.battlefield(P0, "Teleportation Circle");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        through_end_step(t, P0);
    });
}

#[test]
fn creatures_returned_at_the_next_end_step_are_new_objects() {
    cr!("400.7", "704.5m", "704.5n", "603.7");
    ruling!(
        "Angel of Condemnation",
        "Auras attached to the exiled creature will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    // "{2}{W}, {T}: Exile another target creature. Return that card to the battlefield
    // under its owner's control at the beginning of the next end step."
    flicker_bears("Angel of Condemnation", |t, bears| {
        let angel = t.battlefield(P0, "Angel of Condemnation");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        activate_containing(t, P0, angel, "beginning of the next end step").unwrap();
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        through_end_step(t, P0);
    });
    ruling!(
        "Koya, Death from Above",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. If the card returns to the battlefield, it will be a new object"
    );
    // "When Koya enters, exile up to one other target creature. At the beginning of the
    // next end step, you may pay {3}{B}. If you don't, return that card ..."
    flicker_bears("Koya, Death from Above", |t, bears| {
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.enter(P0, "Koya, Death from Above");
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        t.answer_yes(P0, false);
        through_end_step(t, P0);
    });
    ruling!(
        "Charming Prince",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. Once the exiled creature returns"
    );
    // "• Exile another target creature you own. Return it to the battlefield under your
    // control at the beginning of the next end step."
    flicker_bears("Charming Prince", |t, bears| {
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.enter(P0, "Charming Prince");
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        through_end_step(t, P0);
    });
    ruling!(
        "Conciliator's Duelist",
        "Once the exiled permanent returns, it's considered a new object with no relation to the object that it was."
    );
    // Repartee — "Whenever you cast an instant or sorcery spell that targets a creature,
    // exile up to one target creature. Return that card to the battlefield under its
    // owner's control at the beginning of the next end step."
    flicker_bears("Conciliator's Duelist", |t, bears| {
        t.battlefield(P0, "Conciliator's Duelist");
        let spell = t.hand(P0, "Giant Growth");
        t.cast(P0, spell).target(bears).go();
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        through_end_step(t, P0);
    });
    // "Exile target creature. At the beginning of the next end step, return that card
    // to the battlefield under its owner's control with a +1/+1 counter on it."
    ruling!(
        "Long Road Home",
        "At the time the creature is exiled, Auras attached to it will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on it at that time will cease to exist."
    );
    supported("Long Road Home");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let d = decorate(&mut t, P0, bears);
    cast_at(&mut t, "Long Road Home", bears);
    let aura_owner = t.obj_now(d.aura).owner;
    assert_eq!(
        t.zone(d.aura),
        mtg_engine::object::Zone::Graveyard(aura_owner)
    );
    assert!(t.on_battlefield(d.equipment));
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(bears));
    // Only the new counter.
    assert_eq!(t.counters(bears, mtg_engine::types::counters::PLUS1), 1);
}

#[test]
fn a_creature_exiled_until_a_permanent_leaves_returns_as_a_new_object() {
    cr!("400.7", "610.3", "704.5m", "704.5n");
    ruling!(
        "Fiend Hunter",
        "Once the exiled creature returns, it's considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Suspension Field",
        "Auras attached to the exiled creature will be put into their owners’ graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    for name in ["Fiend Hunter", "Suspension Field"] {
        supported(name);
        let mut t = TestGame::new(2);
        let ogre = t.battlefield(P1, "Gray Ogre");
        let d = decorate(&mut t, P1, ogre);
        t.answer_targets(P0, &[Entity::Object(ogre)]);
        t.answer_yes(P0, true);
        let hunter = t.enter(P0, name);
        t.resolve_all();
        assert!(t.in_exile("Gray Ogre"), "{name}");
        crate::r_s02_common::destroy(&mut t, hunter);
        t.resolve_all();
        assert_new_object(&t, &d);
        assert_eq!(t.obj_now(ogre).controller, P1);
    }
}

#[test]
fn flickering_spirit_returns_as_a_new_creature() {
    cr!("400.7", "608.2b", "704.5m");
    ruling!(
        "Flickering Spirit",
        "When the ability resolves, Flickering Spirit is exiled, then immediately returned to the battlefield. It returns as a \"new\" creature. Any counters, Auras, and so on are removed. Any spells or abilities targeting Flickering Spirit no longer target it."
    );
    supported("Flickering Spirit");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 3);
    mana(&mut t, P1, 1);
    let spirit = t.battlefield(P0, "Flickering Spirit");
    let d = decorate(&mut t, P0, spirit);
    // P1 casts Shock at the Spirit; in response, P0 flickers it.
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(spirit).go();
    activate_containing(&mut t, P0, spirit, "then return it").unwrap();
    t.resolve_all();
    assert_new_object(&t, &d);
    // Shock had no legal target: no damage on the new Spirit.
    assert_eq!(t.obj_now(spirit).damage, 0);
    assert!(t.on_battlefield(spirit));
}
