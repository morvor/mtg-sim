//! CR 613.1–613.2 (starting values, layer 1 and its sublayers, copiable values) and
//! CR 613.10–613.11 (effects on players and on game rules).

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn clone_def() -> CardDef {
    creature_with(
        "Clone",
        0,
        0,
        &[Color::Blue],
        vec![replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::EnterAsCopy {
                filter: Filter::creature(),
                optional: true,
            },
        )],
    )
}

fn clone_of(t: &mut TestGame, what: ObjectId) -> ObjectId {
    t.answer_choose(P0, &[Entity::Object(what)]);
    let c = t.custom(P0, clone_def(), Zone::Hand(P0));
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: c,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(P0),
            ..Default::default()
        },
        source: None,
    })
    .unwrap()
}

#[test]
fn starting_values_come_from_the_card_or_the_creating_effect() {
    // CR 613.1: a card starts with its printed values; a token with the values defined by
    // the effect that created it; then effects apply in layers.
    cr!("613.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let bear = t.battlefield(P0, "Grizzly Bears");
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::CreateToken {
            spec: TokenSpec {
                name: "Soldier".into(),
                colors: colors(&[Color::White]),
                supertypes: vec![],
                card_types: vec![CardType::Creature],
                subtypes: vec!["Soldier".into()],
                power: Some(1),
                toughness: Some(1),
                abilities: vec![],
                scryfall_name: None,
            },
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
    );
    let tok = t.named_on_battlefield("Soldier")[0];
    assert_eq!(t.obj_now(tok).base.power, Some(1));
    assert_eq!(t.pt(tok), (2, 2));
    assert_eq!(t.obj_now(bear).base.power, Some(2));
    assert_eq!(t.pt(bear), (3, 3));
}

#[test]
fn copy_effects_apply_in_layer_1_and_copy_only_copiable_values() {
    // CR 613.1a, 613.2, 613.2a: a copy effect applies in layer 1a; counters and other
    // effects on the original aren't copied, and effects on the copy apply after.
    // CR 613.2c: after layer 1, its characteristics are its copiable values.
    cr!("613.1a", "613.2", "613.2a", "613.2c");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bear, counters::PLUS1, 1);
    modify_target(
        &mut t,
        P1,
        bear,
        vec![pt(2, 2), Modification::SetColors(colors(&[Color::Red]))],
    );
    assert_eq!(t.pt(bear), (5, 5));
    let c = clone_of(&mut t, bear);
    assert_eq!(t.obj_now(c).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(c), (2, 2));
    assert_eq!(color_of(&t, c), colors(&[Color::Green]));
    assert_eq!(t.obj_now(c).copiable.name, "Grizzly Bears");
    modify_target(&mut t, P0, c, vec![pt(1, 1)]);
    assert_eq!(t.pt(c), (3, 3));
    // A copy of the copy copies the copiable values: Grizzly Bears, 2/2.
    let c2 = clone_of(&mut t, c);
    assert_eq!(t.obj_now(c2).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(c2), (2, 2));
}

#[test]
fn copy_effects_apply_in_timestamp_order() {
    // CR 613.2a: within layer 1a, copy effects apply in timestamp order.
    cr!("613.2a");
    let mut t = TestGame::new(2);
    let shifter = t.custom(
        P0,
        creature("Shifter", 0, 1, &[Color::Blue]),
        Zone::Battlefield,
    );
    let bear = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    for of in [giant, bear] {
        resolve_effect(
            &mut t,
            P0,
            vec![target_creature()],
            &[Entity::Object(of)],
            Effect::BecomeCopy {
                what: Sel::All(Filter::Objects(vec![shifter])),
                of: Sel::Target(0),
                duration: Duration::EndOfTurn,
            },
        );
    }
    assert_eq!(t.obj_now(shifter).chars.name, "Grizzly Bears");
}

#[test]
fn as_enters_power_and_toughness_choices_are_copiable() {
    // CR 613.2a: "As this creature enters, it becomes your choice of a 3/3 artifact
    // creature or a 1/6 Wall artifact creature with defender" generates a copiable effect,
    // so a copy of it is the chosen form.
    cr!("613.2a");
    let mut t = TestGame::new(2);
    let wall = Effect::Modify {
        what: Sel::This,
        mods: vec![
            Modification::AddTypes(vec![CardType::Artifact]),
            Modification::AddSubtypes(vec!["Wall".into()]),
            Modification::AddKeyword(Keyword::new(KeywordKind::Defender)),
            set_pt(1, 6),
        ],
        duration: Duration::Permanent,
    };
    let beast = Effect::Modify {
        what: Sel::This,
        mods: vec![
            Modification::AddTypes(vec![CardType::Artifact]),
            set_pt(3, 3),
        ],
        duration: Duration::Permanent,
    };
    let clay = creature_with(
        "Primal Clay",
        0,
        0,
        &[],
        vec![replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::AsEnters(Box::new(Effect::ChooseOne {
                who: PlayerRef::You,
                options: vec![("3/3".into(), beast), ("1/6 Wall".into(), wall)],
            })),
        )],
    );
    let c = t.custom(P0, clay, Zone::Hand(P0));
    t.cast_with(P0, c, &[]).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve();
    let clay = t.named_on_battlefield("Primal Clay")[0];
    assert_eq!(t.pt(clay), (1, 6));
    assert_eq!(t.obj_now(clay).copiable.power, Some(1));
    // Something that becomes a copy of it after entering (so no "as enters" choice of its
    // own) is the chosen form.
    let other = t.custom(
        P0,
        creature("Mimic", 2, 2, &[Color::Blue]),
        Zone::Battlefield,
    );
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(clay)],
        Effect::BecomeCopy {
            what: Sel::All(Filter::Objects(vec![other])),
            of: Sel::Target(0),
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.pt(other), (1, 6));
    assert!(t.obj_now(other).has_keyword(KeywordKind::Defender));
    assert!(t.obj_now(other).chars.has_subtype("Wall"));
}

#[test]
fn copies_of_face_down_permanents_are_face_down_values() {
    // CR 613.2b, 613.2c: face-down characteristics apply in layer 1b, after copy effects,
    // and are part of the copiable values — a copy of a face-down creature is a nameless
    // 2/2.
    cr!("613.2b", "613.2c");
    let mut t = TestGame::new(2);
    let hidden = t.battlefield(P1, "Hill Giant");
    t.g.objects[hidden.0 as usize].face_down = true;
    t.recompute();
    assert_eq!(t.pt(hidden), (2, 2));
    let c = clone_of(&mut t, hidden);
    assert_eq!(t.obj_now(c).chars.name, "");
    assert_eq!(t.pt(c), (2, 2));
    assert_eq!(color_of(&t, c), ColorSet::NONE);
}

#[test]
fn player_effects_apply_after_characteristics_in_timestamp_order() {
    // CR 613.10: effects on players are applied after objects' characteristics are
    // determined: "As long as you control a white creature, you have hexproof" sees a
    // creature made white by a layer 5 effect.
    cr!("613.10");
    let mut t = TestGame::new(2);
    let mut s = StaticAbility::new(StaticEffect::PlayerEffect {
        affected: PlayerFilter::You,
        effect: PlayerModification::Hexproof,
    });
    s.condition = Some(Condition::Exists(Filter::and(vec![
        Filter::creature().you_control(),
        Filter::Color(Color::White),
    ])));
    t.custom(
        P0,
        permanent(
            "Ward Sigil",
            &[CardType::Enchantment],
            vec![AbilityDef::new(AbilityKind::Static(s), "static")],
        ),
        Zone::Battlefield,
    );
    let bear = t.battlefield(P0, "Grizzly Bears");
    let hexproof = |t: &TestGame| {
        t.player(P0)
            .has_mod(|m| matches!(m, PlayerModification::Hexproof))
    };
    assert!(!hexproof(&t));
    modify_target(
        &mut t,
        P0,
        bear,
        vec![Modification::SetColors(colors(&[Color::White]))],
    );
    assert!(hexproof(&t));
}

#[test]
fn rule_effects_apply_last_and_in_timestamp_order() {
    // CR 613.10, 613.11: effects on players and on the rules (maximum hand size) apply after
    // all other effects, in timestamp order: "no maximum hand size" then "maximum hand size
    // reduced by 2" leaves no maximum; the other order leaves 5.
    cr!("613.11", "613.10");
    let no_max = || {
        permanent(
            "Spellbook",
            &[CardType::Artifact],
            vec![static_ab(StaticEffect::PlayerEffect {
                affected: PlayerFilter::You,
                effect: PlayerModification::MaxHandSize(None),
            })],
        )
    };
    let minus_two = || {
        permanent(
            "Shrink",
            &[CardType::Enchantment],
            vec![static_ab(StaticEffect::PlayerEffect {
                affected: PlayerFilter::You,
                effect: PlayerModification::HandSizeDelta(-2),
            })],
        )
    };
    let set_five = || {
        permanent(
            "Five",
            &[CardType::Enchantment],
            vec![static_ab(StaticEffect::PlayerEffect {
                affected: PlayerFilter::You,
                effect: PlayerModification::MaxHandSize(Some(Value::c(7))),
            })],
        )
    };
    let mut t = TestGame::new(2);
    t.custom(P0, no_max(), Zone::Battlefield);
    t.custom(P0, minus_two(), Zone::Battlefield);
    t.recompute();
    assert_eq!(t.player(P0).max_hand_size, None);
    let mut t = TestGame::new(2);
    t.custom(P0, minus_two(), Zone::Battlefield);
    t.custom(P0, set_five(), Zone::Battlefield);
    t.recompute();
    assert_eq!(t.player(P0).max_hand_size, Some(7));
    let mut t = TestGame::new(2);
    t.custom(P0, set_five(), Zone::Battlefield);
    t.custom(P0, minus_two(), Zone::Battlefield);
    t.recompute();
    assert_eq!(t.player(P0).max_hand_size, Some(5));
    // A rule effect sees characteristics from all layers: red creatures can't block.
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        permanent(
            "Red Rule",
            &[CardType::Enchantment],
            vec![restriction(Restriction::CantBlock(Filter::and(vec![
                Filter::creature(),
                Filter::Color(Color::Red),
            ])))],
        ),
        Zone::Battlefield,
    );
    let attacker = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    modify_target(
        &mut t,
        P0,
        blocker,
        vec![Modification::SetColors(colors(&[Color::Red]))],
    );
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    assert_eq!(t.life(P1), 17);
}
