//! CR 613: interaction of continuous effects — the layer system, sublayers, and the
//! examples given in the rules.

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn honor_of_the_pure_follows_color_changes() {
    // CR 613.5 example: Honor of the Pure applies (layer 7c) once an effect turns the
    // creature white (layer 5), and stops applying when it's later turned red.
    cr!("613.5", "613.1e", "613.1g", "611.3a", "611.3b", "613.9");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Honor of the Pure");
    let c = t.custom(
        P0,
        creature("Black Bear", 2, 2, &[Color::Black]),
        Zone::Battlefield,
    );
    assert_eq!(t.pt(c), (2, 2));
    modify_target(
        &mut t,
        P0,
        c,
        vec![Modification::SetColors(colors(&[Color::White]))],
    );
    assert_eq!(color_of(&t, c), colors(&[Color::White]));
    assert_eq!(t.pt(c), (3, 3));
    modify_target(
        &mut t,
        P0,
        c,
        vec![Modification::SetColors(colors(&[Color::Red]))],
    );
    assert_eq!(color_of(&t, c), colors(&[Color::Red]));
    assert_eq!(t.pt(c), (2, 2));
}

#[test]
fn gray_ogre_layer_7_example() {
    // CR 613.5 example: counter (7c), +4/+4 (7c), +0/+2 anthem (7c), becomes 0/1 (7b).
    cr!("613.5", "613.4b", "613.4c", "613.4");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Gray Ogre");
    assert_eq!(t.pt(ogre), (2, 2));
    put_counters(&mut t, ogre, counters::PLUS1, 1);
    assert_eq!(t.pt(ogre), (3, 3));
    t.lands(P0, "Forest", 3);
    let growth = t.hand(P0, "Titanic Growth");
    t.cast(P0, growth).target(ogre).go();
    t.resolve();
    assert_eq!(t.pt(ogre), (7, 7));
    t.custom(
        P0,
        permanent(
            "Toughness Anthem",
            &[CardType::Enchantment],
            vec![continuous(Filter::creature().you_control(), vec![pt(0, 2)])],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    assert_eq!(t.pt(ogre), (7, 9));
    modify_target(&mut t, P0, ogre, vec![set_pt(0, 1)]);
    assert_eq!(t.pt(ogre), (5, 8));
}

#[test]
fn switching_power_and_toughness_examples() {
    // CR 613.4d, first example: 1/3 gets +0/+1, is switched (4/1), then gets +5/+0:
    // "unswitched" it would be 6/4, so it's 4/6.
    cr!("613.4d");
    let mut t = TestGame::new(2);
    let c = t.custom(P0, creature("One Three", 1, 3, &[]), Zone::Battlefield);
    modify_target(&mut t, P0, c, vec![pt(0, 1)]);
    modify_target(&mut t, P0, c, vec![Modification::SwitchPT]);
    assert_eq!(t.pt(c), (4, 1));
    modify_target(&mut t, P0, c, vec![pt(5, 0)]);
    assert_eq!(t.pt(c), (4, 6));

    // Third example: two switches cancel each other: 1/4.
    let mut t = TestGame::new(2);
    let c = t.custom(P0, creature("One Three", 1, 3, &[]), Zone::Battlefield);
    modify_target(&mut t, P0, c, vec![pt(0, 1)]);
    modify_target(&mut t, P0, c, vec![Modification::SwitchPT]);
    modify_target(&mut t, P0, c, vec![Modification::SwitchPT]);
    assert_eq!(t.pt(c), (1, 4));
}

#[test]
fn switch_outlasting_the_modifier() {
    // CR 613.4d, second example: if the +0/+1 effect ends before the switch effect ends,
    // the creature becomes 3/1.
    cr!("613.4d", "611.2a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let c = t.custom(P0, creature("One Three", 1, 3, &[]), Zone::Battlefield);
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(c)],
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![pt(0, 1)],
            duration: Duration::EndOfCombat,
        },
    );
    modify_target(&mut t, P0, c, vec![Modification::SwitchPT]);
    assert_eq!(t.pt(c), (4, 1));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.pt(c), (3, 1));
    // The switch lasts until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(c), (1, 3));
}

#[test]
fn effect_parts_apply_in_their_own_layers() {
    // CR 613.6, first example: "gets +1/+1 and becomes the color of your choice" applies
    // its color part in layer 5 and its P/T part in layer 7c. A white-creature anthem
    // (layer 7c) sees the new color.
    cr!("613.6", "613.1e", "613.4c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Honor of the Pure");
    let c = t.custom(
        P0,
        creature("Red Bear", 2, 2, &[Color::Red]),
        Zone::Battlefield,
    );
    modify_target(
        &mut t,
        P0,
        c,
        vec![pt(1, 1), Modification::SetColors(colors(&[Color::White]))],
    );
    assert_eq!(t.pt(c), (4, 4));
}

#[test]
fn act_of_treason_control_then_haste() {
    // CR 613.6, second example: Act of Treason's control change applies in layer 2 and
    // "it gains haste" in layer 6.
    cr!("613.6", "613.1b", "613.1f");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let act = t.hand(P0, "Act of Treason");
    t.cast(P0, act).target(bear).go();
    t.resolve();
    assert_eq!(t.obj_now(bear).controller, P0);
    assert!(has_kw(&t, bear, KeywordKind::Haste));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bear).controller, P1);
    assert!(!has_kw(&t, bear, KeywordKind::Haste));
}

#[test]
fn noncreature_artifacts_become_creatures_same_set_in_7b() {
    // CR 613.6, third example: "All noncreature artifacts become 2/2 artifact creatures"
    // applies in layer 4 and then in layer 7b to the same permanents, even though they're
    // no longer noncreature artifacts by then.
    cr!("613.6", "613.1d", "613.4b", "611.2c");
    let mut t = TestGame::new(2);
    let rock = t.custom(
        P0,
        permanent("Rock", &[CardType::Artifact], vec![]),
        Zone::Battlefield,
    );
    let golem = t.custom(
        P0,
        {
            let mut d = creature("Golem", 3, 3, &[]);
            d.faces[0].chars.card_types.insert(CardType::Artifact);
            d
        },
        Zone::Battlefield,
    );
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::Modify {
            what: Sel::All(Filter::and(vec![
                Filter::Type(CardType::Artifact),
                Filter::not(Filter::creature()),
            ])),
            mods: vec![
                Modification::AddTypes(vec![CardType::Artifact, CardType::Creature]),
                set_pt(2, 2),
            ],
            duration: Duration::EndOfTurn,
        },
    );
    assert!(t.obj_now(rock).is_creature());
    assert_eq!(t.pt(rock), (2, 2));
    // The artifact creature wasn't a noncreature artifact, so it isn't affected.
    assert_eq!(t.pt(golem), (3, 3));
    // An artifact entering later isn't affected either (the set was locked in).
    let rock2 = t.custom(
        P0,
        permanent("Rock", &[CardType::Artifact], vec![]),
        Zone::Battlefield,
    );
    t.recompute();
    assert!(!t.obj_now(rock2).is_creature());
}

#[test]
fn svogthos_example_timestamps_in_7b() {
    // CR 613.6, fourth example (Svogthos, the Restless Tomb): a land becomes a 3/3 (4, 7b),
    // gets +1/+1 (7c), then becomes a creature with "P/T each equal to the number of
    // creature cards in your graveyard" (4, 5, 7b) — an 11/11 with ten creature cards.
    // Reapplying the first effect makes it 4/4 again. The granted ability's effect has the
    // timestamp of the effect that granted it (CR 613.7a).
    cr!("613.6", "613.7a", "613.7b", "613.4b", "611.3a");
    let mut t = TestGame::new(2);
    let land = t.custom(
        P0,
        permanent("Tomb", &[CardType::Land], vec![]),
        Zone::Battlefield,
    );
    for _ in 0..10 {
        t.custom(P0, creature("Dead", 1, 1, &[]), Zone::Graveyard(P0));
    }
    let becomes_3_3 = vec![
        Modification::AddTypes(vec![CardType::Creature]),
        set_pt(3, 3),
    ];
    resolve_effect(
        &mut t,
        P0,
        vec![TargetSpec::object(
            Filter::Type(CardType::Land),
            "target land",
        )],
        &[Entity::Object(land)],
        Effect::Modify {
            what: Sel::Target(0),
            mods: becomes_3_3.clone(),
            duration: Duration::EndOfTurn,
        },
    );
    modify_target(&mut t, P0, land, vec![pt(1, 1)]);
    assert_eq!(t.pt(land), (4, 4));
    let count = Value::CardsInGraveyard(PlayerRef::You, Filter::creature());
    let pt_ability = continuous(
        Filter::Source,
        vec![Modification::SetPT(Some(count.clone()), Some(count))],
    );
    // Svogthos's own ability: "this land becomes a black and green Plant Zombie creature
    // with '...'".
    let svogthos = Effect::Modify {
        what: Sel::This,
        mods: vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::AddSubtypes(vec!["Plant".into(), "Zombie".into()]),
            Modification::SetColors(colors(&[Color::Black, Color::Green])),
            Modification::AddAbility(pt_ability),
        ],
        duration: Duration::EndOfTurn,
    };
    let mut ctx = mtg_engine::eval::Ctx::new(Some(land), P0);
    t.g.exec(&svogthos, &mut ctx);
    t.recompute();
    assert_eq!(t.pt(land), (11, 11));
    assert!(t.obj_now(land).chars.has_subtype("Zombie"));
    // A creature card entering the graveyard changes it.
    t.custom(P0, creature("Dead", 1, 1, &[]), Zone::Graveyard(P0));
    t.recompute();
    assert_eq!(t.pt(land), (12, 12));
    // The first effect again, with a new timestamp.
    resolve_effect(
        &mut t,
        P0,
        vec![TargetSpec::object(
            Filter::Type(CardType::Land),
            "target land",
        )],
        &[Entity::Object(land)],
        Effect::Modify {
            what: Sel::Target(0),
            mods: becomes_3_3,
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.pt(land), (4, 4));
}
