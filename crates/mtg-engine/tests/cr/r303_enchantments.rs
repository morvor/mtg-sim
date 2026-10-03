//! CR 303: enchantments — casting and resolving, enchantment types, Auras (what they can
//! enchant and how they enter the battlefield attached), Sagas, Classes, and Roles.

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::replacement::TokenCreate;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn enchantment_spells_are_cast_at_sorcery_speed_and_use_the_stack() {
    cr!("303.1");
    check_sorcery_timing("Glorious Anthem", "{1}{W}{W}");
    check_sorcery_timing("Pacifism", "{1}{W}");
}

#[test]
fn an_enchantment_spell_resolves_onto_the_battlefield_under_its_controllers_control() {
    cr!("303.2");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let anthem = cast_others_card(&mut t, P0, P1, "Glorious Anthem", "{1}{W}{W}", &[]);
    assert!(t.on_battlefield(anthem));
    assert_eq!((t.obj(anthem).controller, t.obj(anthem).owner), (P0, P1));
    // "Creatures you control" are its controller's.
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn enchantment_subtypes_are_single_words_and_an_enchantment_may_have_several() {
    cr!("303.3");
    assert_eq!(subtypes_of("Curse of the Pierced Heart"), vec!["Aura", "Curse"]);
    for s in ["Aura", "Curse", "Shrine", "Saga", "Class", "Role", "Background"] {
        assert_eq!(subtype_kind(s), Some(SubtypeKind::Enchantment), "{s}");
    }
    let tl = TypeLine::parse("Enchantment — Aura Curse");
    assert_eq!(tl.subtypes.len(), 2);
}

#[test]
fn an_aura_cant_enchant_itself() {
    cr!("303.4d");
    let mut t = TestGame::new(2);
    let aura = t.custom(
        P0,
        oracle_card(
            "Self Regard",
            "Enchantment — Aura",
            "{1}",
            None,
            "Enchant enchantment",
        ),
        Zone::Battlefield,
    );
    let anthem = t.battlefield(P0, "Glorious Anthem");
    // It can be attached to another enchantment, but not to itself.
    assert!(mtg_engine::attach::can_attach(&t.g, aura, Entity::Object(anthem)));
    assert!(!mtg_engine::attach::can_attach(&t.g, aura, Entity::Object(aura)));
    // If it somehow enchants itself, it's put into its owner's graveyard.
    t.g.objects[aura.0 as usize].attached_to = Some(Entity::Object(aura));
    t.settle();
    assert!(t.in_graveyard(P0, "Self Regard"));
}

#[test]
fn an_aura_that_becomes_a_creature_becomes_unattached_then_goes_to_the_graveyard() {
    cr!("303.4d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let c = t.hand(P0, "Pacifism");
    t.cast(P0, c).target(bears).go();
    t.resolve();
    let pacifism = t.named_on_battlefield("Pacifism")[0];
    assert_eq!(t.obj(pacifism).attached_to, Some(Entity::Object(bears)));
    let before = t.turn_events.len();
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::AddTypes(vec![CardType::Creature]),
                Modification::SetPT(Some(Value::c(3)), Some(Value::c(3))),
            ],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(pacifism)],
    );
    t.settle();
    assert!(t.in_graveyard(P0, "Pacifism"));
    let events = &t.turn_events[before..];
    let unattached = events
        .iter()
        .position(|e| matches!(e, Event::Unattached { obj, .. } if *obj == pacifism));
    let to_gy = events.iter().position(|e| {
        matches!(e, Event::ZoneChange { old, to: Zone::Graveyard(_), .. } if *old == pacifism)
    });
    assert!(unattached.is_some() && to_gy.is_some());
    assert!(unattached < to_gy, "unattached first, then put into the graveyard");
}

fn replenish(t: &mut TestGame, p: PlayerId) {
    mana_for(t, p, "{3}{W}");
    let c = t.hand(p, "Replenish");
    t.cast(p, c).go();
    t.resolve();
}

#[test]
fn an_aura_put_onto_the_battlefield_without_a_specified_object_enchants_what_its_controller_chooses(
) {
    cr!("303.4f");
    ruling!(
        "Replenish",
        "You must return Auras if possible, even if this means enchanting an opponent's permanent"
    );
    ruling!(
        "Zur the Enchanter",
        "An Aura put onto the battlefield this way doesn't target anything"
    );
    // P0 controls no creature; the only creatures are P1's, one with hexproof.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.graveyard(P0, "Holy Strength");
    t.answer_choose(P0, &[Entity::Object(scout)]);
    replenish(&mut t, P0);
    let hs = t.named_on_battlefield("Holy Strength");
    assert_eq!(hs.len(), 1, "it must be returned, enchanting an opponent's creature");
    assert_eq!(t.obj(hs[0]).attached_to, Some(Entity::Object(scout)));
    assert_eq!(t.obj(hs[0]).controller, P0);
    assert_eq!(t.pt(scout), (2, 3));
    // The choice was among the legal objects to enchant: creatures, not lands.
    let offered = t
        .asked()
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, prompt, .. }
                if *p == P0 && prompt.contains("enchant") =>
            {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap();
    assert!(offered.contains(&Entity::Object(bears)));
    assert!(offered
        .iter()
        .all(|e| e.object().is_some_and(|o| t.obj(o).is_creature())));
}

#[test]
fn an_aura_with_nothing_to_enchant_stays_where_it_is() {
    cr!("303.4g");
    ruling!(
        "Replenish",
        "Auras can only be placed on permanents that were on the battlefield before this effect started to resolve"
    );
    // No creatures: Pacifism stays in the graveyard. The Aura that can enchant only an
    // enchantment can't enchant the Glorious Anthem returned at the same time.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Pacifism");
    t.graveyard(P0, "Glorious Anthem");
    t.custom(
        P0,
        oracle_card(
            "Enchanted Enchantment",
            "Enchantment — Aura",
            "{1}",
            None,
            "Enchant enchantment",
        ),
        Zone::Graveyard(P0),
    );
    replenish(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 1);
    assert!(t.in_graveyard(P0, "Pacifism"));
    assert!(t.in_graveyard(P0, "Enchanted Enchantment"));
    assert!(t.named_on_battlefield("Pacifism").is_empty());
    // An Aura token with nothing to enchant isn't created.
    let mut t = TestGame::new(2);
    let spec = mtg_engine::tokens::predefined("Monster").expect("Monster Role");
    let tc = TokenCreate {
        chars: mtg_engine::tokens::token_characteristics(&spec),
        card: mtg_engine::tokens::predefined_card(&spec),
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    let made = t.g.create_tokens(P0, tc, 1, None);
    assert!(made.is_empty());
    assert!(t.named_on_battlefield("Monster").is_empty());
    assert!(t.g.player(P0).life == 20 && t.g.history.tokens_created.is_empty());
}

#[test]
fn a_non_aura_put_onto_the_battlefield_attached_enters_unattached() {
    cr!("303.4h");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    run_effect_slots(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                attached_to: Some(Sel::Target(1)),
                ..Destination::battlefield()
            },
        },
        vec![vec![Entity::Object(bears)], vec![Entity::Object(giant)]],
    );
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj(bears).attached_to, None);
}

fn put_attached(t: &mut TestGame, card: ObjectId, to: Vec<Entity>) {
    run_effect_slots(
        t,
        P0,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                attached_to: Some(Sel::Target(1)),
                ..Destination::battlefield()
            },
        },
        vec![vec![Entity::Object(card)], to],
    );
}

#[test]
fn an_aura_put_onto_the_battlefield_attached_to_something_it_cant_enchant_stays_where_it_is() {
    cr!("303.4i");
    ruling!("Monstrous Rage", "In such cases, the Role token isn");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Attached to a land: Pacifism stays in the graveyard.
    let a = t.graveyard(P0, "Pacifism");
    put_attached(&mut t, a, vec![Entity::Object(forest)]);
    assert_eq!(t.zone(a), Zone::Graveyard(P0));
    // Attached to an undefined object: it stays too.
    put_attached(&mut t, a, vec![]);
    assert_eq!(t.zone(a), Zone::Graveyard(P0));
    // Attached to a creature: it enters enchanting it.
    put_attached(&mut t, a, vec![Entity::Object(bears)]);
    let a = t.g.current(a);
    assert!(t.on_battlefield(a));
    assert_eq!(t.obj(a).attached_to, Some(Entity::Object(bears)));
    // From the stack: an Aura spell is put into its owner's graveyard instead.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let c = t.hand(P0, "Pacifism");
    let spell = t.cast(P0, c).target(bears).go();
    put_attached(&mut t, spell, vec![Entity::Object(forest)]);
    assert!(t.in_graveyard(P0, "Pacifism"));
    assert_eq!(t.stack_len(), 0);
    // A Role token that can't be attached to it isn't created.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::CreateTokenAttached {
            spec: mtg_engine::tokens::predefined("Monster").unwrap(),
            count: Value::c(1),
            controller: PlayerRef::You,
            to: Sel::Target(0),
        },
        &[Entity::Object(forest)],
    );
    assert!(t.named_on_battlefield("Monster").is_empty());
    assert!(t.g.history.tokens_created.is_empty());
}

#[test]
fn an_aura_on_the_battlefield_cant_be_attached_to_something_it_cant_enchant() {
    cr!("303.4j");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 2);
    let c = t.hand(P0, "Pacifism");
    t.cast(P0, c).target(bears).go();
    t.resolve();
    let pacifism = t.named_on_battlefield("Pacifism")[0];
    let attach_to = |t: &mut TestGame, e: ObjectId| {
        run_effect(
            t,
            P0,
            Some(pacifism),
            Effect::Attach {
                what: Sel::This,
                to: Sel::Target(0),
            },
            &[Entity::Object(e)],
        );
    };
    attach_to(&mut t, forest);
    assert_eq!(t.obj(pacifism).attached_to, Some(Entity::Object(bears)));
    attach_to(&mut t, giant);
    assert_eq!(t.obj(pacifism).attached_to, Some(Entity::Object(giant)));
}

#[test]
fn an_aura_turned_face_up_is_attached_according_to_its_face_up_characteristics() {
    cr!("303.4k");
    supported("Gift of Doom");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Swamp", 3);
    let c = t.hand(P0, "Gift of Doom");
    t.cast(P0, c)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let gift = t.g.current(c);
    assert!(t.obj(gift).face_down && t.obj(gift).is_creature());
    // Morph—Sacrifice another creature. As it's turned face up, attach it to a creature:
    // one it could enchant as the face-up Aura.
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        Action::Special(SpecialAction::TurnFaceUp { obj: gift }),
    )
    .unwrap();
    t.settle();
    let gift = t.g.current(gift);
    assert!(t.on_battlefield(gift));
    assert!(!t.obj(gift).face_down);
    assert_eq!(t.obj(gift).attached_to, Some(Entity::Object(bears)));
    assert!(t.obj(bears).has_keyword(KeywordKind::Indestructible));
    // The choice was among creatures it could enchant (not a land, nor itself).
    let offered = t
        .asked()
        .iter()
        .rev()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert!(offered.contains(&Entity::Object(bears)));
    assert!(offered.contains(&Entity::Object(theirs)));
    assert!(!offered.contains(&Entity::Object(forest)));
    assert!(!offered.contains(&Entity::Object(gift)));
}

#[test]
fn an_aura_turned_face_up_without_being_attached_goes_to_the_graveyard() {
    cr!("303.4k");
    ruling!(
        "Gift of Doom",
        "or if you choose not to attach it to a creature, it"
    );
    ruling!("Gift of Doom", "may be chosen this way");
    // Declining to attach it: it's an Aura attached to nothing.
    let mut t = TestGame::new(2);
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Gladecover Scout");
    t.lands(P0, "Swamp", 3);
    let c = t.hand(P0, "Gift of Doom");
    t.cast(P0, c)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let gift = t.g.current(c);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.answer_yes(P0, false);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        Action::Special(SpecialAction::TurnFaceUp { obj: gift }),
    )
    .unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Gift of Doom"));
    // Attaching it doesn't target: an opponent's creature with hexproof can be chosen.
    let mut t = TestGame::new(2);
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.lands(P0, "Swamp", 3);
    let c = t.hand(P0, "Gift of Doom");
    t.cast(P0, c)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let gift = t.g.current(c);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(scout)]);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        Action::Special(SpecialAction::TurnFaceUp { obj: gift }),
    )
    .unwrap();
    t.settle();
    let gift = t.g.current(gift);
    assert_eq!(t.obj(gift).attached_to, Some(Entity::Object(scout)));
    let _ = theirs;
}

#[test]
fn enchanted_refers_to_what_a_non_aura_permanent_is_attached_to() {
    cr!("303.4m");
    // An Equipment whose ability refers to the "enchanted creature".
    let odd = oracle_card(
        "Odd Charm",
        "Artifact — Equipment",
        "{1}",
        None,
        "Enchanted creature gets +2/+2.\nEquip {1}",
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let charm = t.custom(P0, odd, Zone::Battlefield);
    t.lands(P0, "Wastes", 1);
    t.activate(P0, charm, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.obj(charm).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn a_saga_is_an_enchantment_with_lore_counters_and_chapter_abilities() {
    cr!("303.5");
    ruling!("History of Benalia", "As a Saga enters the battlefield, its controller puts a lore counter on it");
    let mut t = TestGame::new(2);
    mana_for(&mut t, P0, "{1}{W}{W}");
    let c = t.hand(P0, "History of Benalia");
    t.cast(P0, c).go();
    t.resolve();
    let saga = t.named_on_battlefield("History of Benalia")[0];
    assert!(t.obj(saga).is(CardType::Enchantment) && t.obj(saga).chars.has_subtype("Saga"));
    assert_eq!(t.counters(saga, counters::LORE), 1);
    // Chapter I triggered: a Knight token.
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Knight Token").len(), 1);
}

#[test]
fn a_class_is_an_enchantment_that_gains_levels() {
    cr!("303.6");
    ruling!("Ranger Class", "Gaining a level is a normal activated ability");
    let mut t = TestGame::new(2);
    mana_for(&mut t, P0, "{1}{G}");
    let c = t.hand(P0, "Ranger Class");
    t.cast(P0, c).go();
    t.resolve_all();
    let class = t.named_on_battlefield("Ranger Class")[0];
    assert!(t.obj(class).chars.has_subtype("Class"));
    let wolf = t.named_on_battlefield("Wolf Token")[0];
    // {1}{G}: Level 2 — "Whenever you attack, put a +1/+1 counter on target attacking
    // creature."
    mana_for(&mut t, P0, "{1}{G}");
    t.activate(P0, class, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.obj(class).class_level, 2);
    // (As if the Wolf had been under P0's control since the turn began.)
    t.g.objects[wolf.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(wolf)]);
    t.attack(&[(wolf, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(wolf, counters::PLUS1), 1);
}

#[test]
fn only_the_newest_role_a_player_controls_on_a_permanent_stays() {
    cr!("303.7", "303.7a");
    ruling!("Monstrous Rage", "each of those Roles except the one with the most recent timestamp");
    ruling!("Monstrous Rage", "A permanent can have multiple Roles attached to it if each one is controlled by a different player");
    ruling!("Monstrous Rage", "Each one has the Aura and Role subtypes");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Monstrous Rage: a Monster Role attached to the target.
    t.lands(P0, "Mountain", 2);
    let rage = t.hand(P0, "Monstrous Rage");
    t.cast(P0, rage).target(bears).go();
    t.resolve();
    let monster = t.named_on_battlefield("Monster")[0];
    let o = t.obj(monster);
    assert!(o.chars.has_subtype("Aura") && o.chars.has_subtype("Role"));
    assert_eq!(o.attached_to, Some(Entity::Object(bears)));
    // Embereth Veteran: a Young Hero Role on the same creature, controlled by the same
    // player. The older Role is put into its owner's graveyard.
    let vet = t.battlefield(P0, "Embereth Veteran");
    t.activate(P0, vet, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    let hero = t.named_on_battlefield("Young Hero");
    assert_eq!(hero.len(), 1);
    assert_eq!(t.obj(hero[0]).attached_to, Some(Entity::Object(bears)));
    assert!(t.named_on_battlefield("Monster").is_empty());
    // A Role another player controls on it stays.
    let rage = t.hand(P1, "Monstrous Rage");
    t.lands(P1, "Mountain", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, rage).target(bears).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Monster").len(), 1);
    assert_eq!(t.named_on_battlefield("Young Hero").len(), 1);
}
