//! Rulings batch P026 — copying a permanent that's a creature only because of a non-copy
//! effect (an animated land, a crewed Vehicle, an animated planeswalker) copies only its
//! printed values, so the copy isn't a creature (CR 707.2, 613.1a); a copy of an Equipment
//! or Vehicle doesn't copy what it's attached to or that it was crewed (CR 707.2, 301.5,
//! 702.122); and a creature that becomes a copy of something it couldn't have attacked as
//! keeps attacking, unless it stops being a creature (CR 506.4).

use crate::r_p026_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's crewed Smuggler's Copter (an artifact Vehicle that's a 3/3 creature until end of
/// turn), crewed by a Grizzly Bears.
fn crewed_copter(t: &mut TestGame) -> ObjectId {
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(is_creature(t, copter));
    copter
}

/// A token copy of a Vehicle that isn't crewed and isn't a creature.
fn uncrewed_copter(t: &TestGame, id: ObjectId) {
    assert_eq!(name_of(t, id), "Smuggler's Copter");
    assert!(t.obj_now(id).is(CardType::Artifact) && has_subtype(t, id, "Vehicle"));
    assert!(!is_creature(t, id), "the copy is crewed");
}

/// P0's Bonesplitter attached to P0's Grizzly Bears.
fn attached_bonesplitter(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let b = attach_new(t, P0, "Bonesplitter", bears);
    assert!(!t.g.attachments_of(obj(bears)).is_empty());
    b
}

fn unattached(t: &TestGame, id: ObjectId) {
    assert_eq!(name_of(t, id), "Bonesplitter");
    assert_eq!(t.obj_now(id).attached_to, None);
}

#[test]
fn copies_of_a_crewed_vehicle_arent_crewed() {
    cr!("707.2", "702.122a", "613.1a");
    ruling!(
        "Echocasting Symposium",
        "If it is a Vehicle, it is not crewed."
    );
    ruling!(
        "Sandstorm Crasher",
        "It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on. If it is a Vehicle, it is not crewed."
    );
    // Echocasting Symposium: "Target player creates a token that's a copy of target
    // creature you control."
    let mut t = TestGame::new(2);
    let copter = crewed_copter(&mut t);
    let tok = one(cast_for_tokens(
        &mut t,
        P0,
        "Echocasting Symposium",
        "{4}{U}{U}",
        &[Entity::Player(P0), obj(copter)],
        None,
        None,
    ));
    uncrewed_copter(&t, tok);
    // Sandstorm Crasher: "create a tapped and attacking token that's a copy of target
    // creature you control." The copy of the crewed Copter isn't a creature, so it isn't
    // attacking.
    supported("Sandstorm Crasher");
    let mut t = TestGame::new(2);
    let crasher = t.battlefield(P0, "Sandstorm Crasher");
    t.set_step(P0, Step::BeginningOfCombat);
    let copter = crewed_copter(&mut t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(copter)]);
    let before = t.g.battlefield.clone();
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(crasher, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    uncrewed_copter(&t, tok);
    assert!(!t.g.is_attacking(tok));
}

#[test]
fn copies_of_vehicles_and_equipment_arent_crewed_or_attached() {
    cr!("707.2", "702.122a");
    ruling!(
        "Season of Weaving",
        "If it is a Vehicle, it is not crewed. If it is an Equipment, it is not attached to any creature."
    );
    ruling!(
        "Devastating Onslaught",
        "If the permanent is a Vehicle, the copies are not crewed. If it is an Equipment, the copies are not attached to any creature."
    );
    // Season of Weaving: "{P}{P} — Choose an artifact or creature you control. Create a
    // token that's a copy of it." Chosen twice: the Copter and the Bonesplitter.
    let mut t = TestGame::new(2);
    let copter = crewed_copter(&mut t);
    let b = attached_bonesplitter(&mut t);
    t.answer_choose(P0, &[obj(copter)]);
    t.answer_choose(P0, &[obj(b)]);
    let toks = cast_for_tokens(
        &mut t,
        P0,
        "Season of Weaving",
        "{4}{U}{U}",
        &[],
        Some(&[1, 1]),
        None,
    );
    assert_eq!(toks.len(), 2);
    uncrewed_copter(&t, one(tokens_named(&t, "Smuggler's Copter")));
    unattached(&t, one(tokens_named(&t, "Bonesplitter")));
    // Devastating Onslaught (X = 1) on each.
    for which in ["Smuggler's Copter", "Bonesplitter"] {
        let mut t = TestGame::new(2);
        let copter = crewed_copter(&mut t);
        let b = attached_bonesplitter(&mut t);
        let target = if which == "Bonesplitter" { b } else { copter };
        let tok = one(cast_for_tokens(
            &mut t,
            P0,
            "Devastating Onslaught",
            "{2}{R}",
            &[obj(target)],
            None,
            Some(1),
        ));
        if which == "Bonesplitter" {
            unattached(&t, tok);
        } else {
            uncrewed_copter(&t, tok);
        }
    }
}

#[test]
fn copies_of_animated_noncreatures_by_jaxis_and_reflection_arent_creatures() {
    cr!("707.2", "613.1a", "613.1d");
    ruling!(
        "Jaxis, the Troublemaker",
        "Most notably, if the target creature isn't normally a creature, the copy won't be a creature."
    );
    ruling!(
        "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki",
        "It doesn't copy whether the creature is tapped or untapped, whether it has any counters on it or Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on. Most notably, if the target creature isn't normally a creature, the copy won't be a creature."
    );
    // Jaxis: "{R}, {T}, Discard a card: Create a token that's a copy of another target
    // creature you control." The target is an animated Forest.
    supported("Jaxis, the Troublemaker");
    let mut t = TestGame::new(2);
    let jaxis = t.battlefield(P0, "Jaxis, the Troublemaker");
    let land = t.battlefield(P0, "Forest");
    animate_land(&mut t, land);
    t.hand(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{R}");
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        jaxis,
        "Create a token",
        &[obj(land)],
    ));
    assert_eq!(name_of(&t, tok), "Forest");
    assert!(t.obj_now(tok).is(CardType::Land) && !is_creature(&t, tok));
    // Reflection of Kiki-Jiki: "{1}, {T}: Create a token that's a copy of another target
    // nonlegendary creature you control, except it has haste."
    let mut t = TestGame::new(2);
    let r = reflection(&mut t);
    let land = t.battlefield(P0, "Forest");
    animate_land(&mut t, land);
    lands_for(&mut t, P0, "{1}");
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        r,
        "Create a token",
        &[obj(land)],
    ));
    assert_eq!(name_of(&t, tok), "Forest");
    assert!(t.obj_now(tok).is(CardType::Land) && !is_creature(&t, tok));
}

#[test]
fn becoming_a_copy_of_an_animated_noncreature_stops_being_a_creature() {
    cr!("707.2", "613.1a", "613.1d");
    ruling!(
        "Mirrorweave",
        "This effect can cause each other creature to stop being a creature. For example, if you target an animated Mutavault (a land with an activated ability that turns it into a creature), only the printed wording will be copied — the “becomes a creature” effect won’t. Each other creature will become an unanimated Mutavault."
    );
    ruling!(
        "Nanogene Conversion",
        "This effect can cause each other creature to stop being a creature. For example, if you target an animated Vehicle, only the printed wording will be copied—the crew effect that is making it a creature until end of turn won't be copied. Each other creature will become an unanimated copy of that Vehicle."
    );
    ruling!(
        "Cytoshape",
        "This effect can cause the target to stop being a creature. For example, if it becomes a copy of an animated Blinkmoth Nexus, the printed wording will be copied and it will become an unanimated Blinkmoth Nexus."
    );
    ruling!(
        "Polymorphous Rush",
        "This effect can cause the targets to stop being creatures. For example, if they each become a copy of a land that has become a creature, the effect that turned that land into a creature won't be copied. The targets will become noncreature lands."
    );
    // Mirrorweave: "Each other creature becomes a copy of target nonlegendary creature
    // until end of turn." Mutavault: "{1}: Until end of turn, this land becomes a 2/2
    // creature with all creature types. It's still a land."
    supported("Mirrorweave");
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Mutavault");
    lands_for(&mut t, P0, "{1}");
    activate_containing(&mut t, P0, vault, "becomes a 2/2").expect("animate");
    t.resolve_all();
    assert!(is_creature(&t, vault));
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for(&mut t, P0, "{2}{W}{U}");
    let mw = t.hand(P0, "Mirrorweave");
    t.cast(P0, mw).target(obj(vault)).go();
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Mutavault");
    assert!(t.obj_now(bears).is(CardType::Land) && !is_creature(&t, bears));
    assert!(is_creature(&t, vault));
    // Nanogene Conversion: "Choose target creature you control. Each other creature becomes
    // a copy of that creature until end of turn, except it isn't legendary."
    supported("Nanogene Conversion");
    let mut t = TestGame::new(2);
    let copter = crewed_copter(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{3}{U}");
    let nc = t.hand(P0, "Nanogene Conversion");
    t.cast(P0, nc).target(obj(copter)).go();
    t.resolve_all();
    assert_eq!(name_of(&t, giant), "Smuggler's Copter");
    assert!(t.obj_now(giant).is(CardType::Artifact) && !is_creature(&t, giant));
    // Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
    // becomes a copy of that creature until end of turn." Blinkmoth Nexus: "{1}: This land
    // becomes a 1/1 Faerie artifact creature with flying until end of turn. It's still a
    // land."
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P1, "Blinkmoth Nexus");
    lands_for(&mut t, P1, "{1}");
    activate_containing(&mut t, P1, nexus, "becomes a 1/1").expect("animate");
    t.resolve_all();
    assert!(is_creature(&t, nexus));
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{1}{G}{U}");
    let cs = t.hand(P0, "Cytoshape");
    t.answer_choose(P0, &[obj(nexus)]);
    t.cast(P0, cs).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Blinkmoth Nexus");
    assert!(t.obj_now(bears).is(CardType::Land) && !is_creature(&t, bears));
    // Polymorphous Rush: "Choose a creature on the battlefield. Any number of target
    // creatures you control each become a copy of that creature until end of turn."
    supported("Polymorphous Rush");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    animate_land(&mut t, land);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{2}{U}");
    let pr = t.hand(P0, "Polymorphous Rush");
    t.answer_choose(P0, &[obj(land)]);
    t.cast(P0, pr).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Forest");
    assert!(t.obj_now(bears).is(CardType::Land) && !is_creature(&t, bears));
}

/// P0's Identity Thief attacks and exiles `what` (controlled by P1), becoming a copy of
/// it. Stops in the declare attackers step after the trigger resolved.
fn thief_attacks_copying(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported("Identity Thief");
    let thief = t.battlefield(P0, "Identity Thief");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(what)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(thief, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert!(!t.on_battlefield(what));
    thief
}

#[test]
fn identity_thief_copying_a_defender_remains_attacking() {
    cr!("506.4a", "707.2");
    ruling!(
        "Identity Thief",
        "This effect can cause Identity Thief to become a copy of a creature that can't normally attack, such as Lupine Prototype or a creature with defender. In these cases, it remains attacking."
    );
    // Wall of Stone: a 0/8 Wall with defender.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    let thief = thief_attacks_copying(&mut t, wall);
    assert_eq!(name_of(&t, thief), "Wall of Stone");
    assert!(t.g.is_attacking(thief));
}

#[test]
fn identity_thief_copying_an_animated_planeswalker_leaves_combat_and_dies() {
    cr!("707.2", "704.5i", "306.5b");
    ruling!(
        "Identity Thief",
        "This effect can cause Identity Thief to stop being a creature. For example, if you target an animated Gideon Jura (a planeswalker with a loyalty ability that turns him into a creature), only the printed wording will be copied; the “becomes a creature” effect won't. Identity Thief will become a noncreature Gideon with no loyalty, be removed from combat, and then be put into its owner's graveyard."
    );
    // Gideon Jura's 0: "Until end of turn, Gideon Jura becomes a 6/6 Human Soldier
    // creature that's still a planeswalker. Prevent all damage that would be dealt to him
    // this turn." (applied here as the same non-copy effect).
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P1, "Gideon Jura");
    modify(
        &mut t,
        gideon,
        vec![
            mtg_engine::ability::Modification::AddTypes(vec![CardType::Creature]),
            mtg_engine::ability::Modification::SetPT(
                Some(mtg_engine::ability::Value::c(6)),
                Some(mtg_engine::ability::Value::c(6)),
            ),
        ],
    );
    assert!(is_creature(&t, gideon));
    let thief = thief_attacks_copying(&mut t, gideon);
    assert!(!t.g.is_attacking(thief));
    assert!(!t.on_battlefield(thief));
    assert!(t.in_graveyard(P0, "Identity Thief"));
}
