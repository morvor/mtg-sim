//! CR 109: objects — what an object is, how descriptions of objects are read,
//! characteristics, who controls an object, and what "you" means.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// "Copy target spell." (no new targets).
fn copier() -> mtg_engine::card::CardDef {
    card_with(
        "Copy Test",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec {
                what: TargetKind::Spell(Filter::Any),
                ..TargetSpec::object(Filter::Any, "target spell")
            }],
            Effect::CopySpell {
                what: Sel::Target(0),
                count: Value::c(1),
                new_targets: false,
            },
        )],
    )
}

#[test]
fn the_kinds_of_objects() {
    cr!("109.1");
    let mut t = TestGame::new(2);
    // A card, a permanent, a token.
    let card_in_hand = t.hand(P0, "Grizzly Bears");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.obj(card_in_hand).kind, ObjKind::Card);
    assert_eq!(t.obj(bears).zone, Zone::Battlefield);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.lands(P0, "Plains", 2);
    t.cast(P0, alarm).go();
    t.resolve();
    assert_eq!(t.obj(tokens_of(&t, P0)[0]).kind, ObjKind::Token);
    // An activated ability on the stack is an object: Stifle can target and counter it.
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    let ability = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(ability).kind, ObjKind::StackAbility);
    let stifle = t.hand(P1, "Stifle");
    t.lands(P1, "Island", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, stifle).target(ability).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
    // A spell and a copy of a spell.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.g.turn.priority = Some(P0);
    let spell = t.cast(P0, bolt).target(P1).go();
    assert_eq!(t.obj(spell).zone, Zone::Stack);
    let c = put_in_hand(&mut t, P0, copier());
    t.cast(P0, c).target(spell).go();
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert_eq!(t.obj(copy).kind, ObjKind::SpellCopy);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // An emblem.
    let emblem_maker = card_with(
        "Emblem Maker",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::CreateEmblem {
                abilities: vec![static_ab(StaticEffect::Continuous {
                    affected: Filter::creature().you_control(),
                    mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
                })],
            },
        )],
    );
    let e = put_in_hand(&mut t, P0, emblem_maker);
    t.cast(P0, e).go();
    t.resolve();
    let emblem = *t.g.command.last().unwrap();
    assert_eq!(t.obj(emblem).kind, ObjKind::Emblem);
}

#[test]
fn a_description_with_a_card_type_means_a_permanent_on_the_battlefield() {
    cr!("109.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let in_gy = t.graveyard(P1, "Grizzly Bears");
    let in_hand = t.hand(P1, "Grizzly Bears");
    let giant = t.hand(P1, "Hill Giant");
    t.lands(P1, "Mountain", 4);
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, giant).go();
    // "Destroy target creature": only creatures on the battlefield.
    let murder = t.hand(P0, "Murder");
    let cands = spell_target_candidates(&t, P0, murder, 0);
    assert!(cands.contains(&Entity::Object(bears)));
    for other in [in_gy, in_hand, spell] {
        assert!(!cands.contains(&Entity::Object(other)));
    }
}

#[test]
fn a_description_with_card_and_a_zone_means_a_card_in_that_zone() {
    cr!("109.2a");
    let mut t = TestGame::new(2);
    let mine = t.graveyard(P0, "Grizzly Bears");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    let on_bf = t.battlefield(P0, "Hill Giant");
    let in_hand = t.hand(P0, "Hill Giant");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    // Raise Dead: "Return target creature card from your graveyard to your hand."
    let raise = t.hand(P0, "Raise Dead");
    let cands = spell_target_candidates(&t, P0, raise, 0);
    assert_eq!(cands, vec![Entity::Object(mine)]);
    for other in [theirs, on_bf, in_hand, bolt] {
        assert!(!cands.contains(&Entity::Object(other)));
    }
}

#[test]
fn a_description_with_spell_means_a_spell_on_the_stack() {
    cr!("109.2b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.hand(P1, "Hill Giant");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 5);
    t.set_step(P1, Step::PrecombatMain);
    let creature_spell = t.cast(P1, giant).go();
    t.g.turn.priority = Some(P1);
    let instant_spell = t.cast(P1, bolt).target(P0).go();
    // Essence Scatter: "Counter target creature spell."
    let scatter = t.hand(P0, "Essence Scatter");
    let cands = spell_target_candidates(&t, P0, scatter, 0);
    assert_eq!(cands, vec![Entity::Object(creature_spell)]);
    assert!(!cands.contains(&Entity::Object(bears)));
    assert!(!cands.contains(&Entity::Object(instant_spell)));
}

#[test]
fn a_description_with_source_includes_sources_in_any_zone() {
    cr!("109.2c");
    let mut t = TestGame::new(2);
    // "Whenever a source an opponent controls deals damage to you, you gain 1 life": an
    // instant spell on the stack is such a source, as is a creature on the battlefield.
    let watcher = card_from_text(
        "Source Watcher",
        "{0}",
        "Enchantment",
        None,
        "Whenever a source an opponent controls deals damage to you, you gain 1 life.",
    );
    put(&mut t, P0, watcher);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

/// A scheme card with the given text, face up in `p`'s command zone.
fn scheme(t: &mut TestGame, p: PlayerId, text: &str) -> ObjectId {
    let def = card_from_text("Test Scheme", "", "Ongoing Scheme", None, text);
    t.custom(p, def, Zone::Command)
}

#[test]
fn this_scheme_means_the_scheme_card_in_the_command_zone() {
    cr!("109.2d", "109.4f");
    let mut t = TestGame::new(2);
    let s = scheme(
        &mut t,
        P1,
        "At the beginning of your upkeep, put a doom counter on this scheme.",
    );
    // Its owner controls it: it triggers at the beginning of P1's upkeep, not P0's.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(s, "doom"), 1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(s, "doom"), 1);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(s, "doom"), 2);
    assert_eq!(t.zone(s), Zone::Command);
}

#[test]
fn copying_an_object_copies_only_its_characteristics() {
    cr!("109.3");
    let mut t = TestGame::new(2);
    // P1's Grizzly Bears: tapped, with a +1/+1 counter, enchanted by an Aura, pumped until
    // end of turn.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    t.g.objects[bears.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 1);
    let growth = t.hand(P1, "Giant Growth");
    t.lands(P1, "Forest", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, growth).target(bears).go();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.pt(bears), (6, 6));
    // P0's Clone copies it: only the copiable characteristics (name, mana cost, color,
    // types, abilities, printed P/T) — not the tapped status, counters, controller, or
    // other effects.
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let clone = t.hand(P0, "Clone");
    t.lands(P0, "Island", 4);
    t.g.turn.priority = Some(P0);
    t.cast(P0, clone).go();
    t.resolve();
    let copy = *t
        .named_on_battlefield("Grizzly Bears")
        .iter()
        .find(|b| **b != bears)
        .unwrap();
    let c = &t.obj(copy).chars;
    assert_eq!(c.name.as_str(), "Grizzly Bears");
    assert_eq!(c.mana_cost.as_ref().unwrap().to_string(), "{1}{G}");
    assert_eq!(c.colors, cs("G"));
    assert!(c.has_subtype("Bear"));
    assert_eq!(t.pt(copy), (2, 2));
    let o = t.obj(copy);
    assert!(!o.tapped);
    assert_eq!(o.counters.get(counters::PLUS1).copied().unwrap_or(0), 0);
    assert_eq!(o.controller, P0);
    assert_eq!(o.owner, P0);
}

#[test]
fn a_mana_abilitys_controller_is_determined_as_though_it_were_on_the_stack() {
    cr!("109.4a");
    let mut t = TestGame::new(2);
    // An ability any player may activate: its controller is the player who activated it,
    // so the mana goes to that player.
    let mut ab = mtg_engine::ability::ActivatedAbility::new(
        Cost::tap(),
        Body::effect(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::Fixed(vec![mtg_engine::mana::ManaType::C]),
            restriction: None,
        }),
    );
    ab.is_mana_ability = true;
    ab.any_player = true;
    let well = card_with(
        "Shared Well",
        "{0}",
        "Artifact",
        None,
        vec![AbilityDef::new(AbilityKind::Activated(ab), "{T}: Add {C}.")],
    );
    let w = put(&mut t, P0, well);
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, w, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P1), 1);
    assert_eq!(pool_total(&t, P0), 0);
    // A triggered mana ability is controlled by its source's controller: P0's Aura on
    // P1's land adds mana to P0's pool when P1 taps the land.
    let aura = card_from_text(
        "Borrowed Growth",
        "{G}",
        "Enchantment — Aura",
        None,
        "Enchant land\nWhenever enchanted land is tapped for mana, add an additional {G}.",
    );
    let forest = t.battlefield(P1, "Forest");
    let a = put(&mut t, P0, aura);
    assert!(t.g.attach(a, Entity::Object(forest)));
    t.activate(P1, forest, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P1, mtg_engine::mana::ManaType::G), 1);
    assert_eq!(pool_count(&t, P0, mtg_engine::mana::ManaType::G), 1);
}

#[test]
fn a_triggered_ability_is_controlled_by_its_sources_controller_when_it_triggered() {
    cr!("109.4b", "109.5");
    let mut t = TestGame::new(2);
    // P0 gains control of P1's Blood Artist, then it dies: its ability triggered while P0
    // controlled it, so P0 controls the ability and "you gain 1 life" means P0.
    let artist = t.battlefield(P1, "Blood Artist");
    let treason = t.hand(P0, "Act of Treason");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, treason).target(artist).go();
    t.resolve();
    assert_eq!(t.obj_now(artist).controller, P0);
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, murder).target(artist).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Blood Artist"));
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    assert_eq!(t.obj(trig).controller, P0);
    t.resolve();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn an_emblem_is_controlled_by_the_player_who_puts_it_into_the_command_zone() {
    cr!("109.4c");
    let mut t = TestGame::new(2);
    let emblem_maker = card_with(
        "Emblem Maker",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::CreateEmblem {
                abilities: vec![static_ab(StaticEffect::Continuous {
                    affected: Filter::creature().you_control(),
                    mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
                })],
            },
        )],
    );
    let mine = t.battlefield(P1, "Grizzly Bears");
    let theirs = t.battlefield(P0, "Grizzly Bears");
    let e = put_in_hand(&mut t, P1, emblem_maker);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, e).go();
    t.resolve();
    let emblem = *t.g.command.last().unwrap();
    assert_eq!(t.obj(emblem).controller, P1);
    t.g.recompute();
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn a_vanguard_card_is_controlled_by_its_owner() {
    cr!("109.4e");
    let mut t = TestGame::new(2);
    // Titania: "You may play an additional land on each of your turns."
    let titania = t.command(P1, "Titania");
    assert_eq!(t.obj(titania).controller, P1);
    // P0 can't play a second land.
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Forest");
    t.play_land(P0, f1).unwrap();
    assert!(t.play_land(P0, f2).is_err());
    // P1, its owner, can.
    t.set_step(P1, Step::PrecombatMain);
    t.g.players[1].lands_played_this_turn = 0;
    let i1 = t.hand(P1, "Island");
    let i2 = t.hand(P1, "Island");
    t.play_land(P1, i1).unwrap();
    t.play_land(P1, i2).unwrap();
}

#[test]
fn a_conspiracy_card_is_controlled_by_its_owner() {
    cr!("109.4g");
    let mut t = TestGame::new(2);
    let def = card_from_text(
        "Test Conspiracy",
        "",
        "Conspiracy",
        None,
        "Creatures you control get +1/+1.",
    );
    let c = t.custom(P1, def, Zone::Command);
    assert_eq!(t.obj(c).controller, P1);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(theirs), (3, 3));
    assert_eq!(t.pt(mine), (2, 2));
}

#[test]
fn you_means_the_controller_of_a_static_or_activated_ability() {
    cr!("109.5");
    let mut t = TestGame::new(2);
    // Static: Benalish Marshal ("Other creatures you control get +1/+1") pumps its
    // current controller's creatures.
    let marshal = t.battlefield(P1, "Benalish Marshal");
    let p0_bears = t.battlefield(P0, "Grizzly Bears");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(p1_bears), (3, 3));
    assert_eq!(t.pt(p0_bears), (2, 2));
    let treason = t.hand(P0, "Act of Treason");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, treason).target(marshal).go();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.pt(p0_bears), (3, 3));
    assert_eq!(t.pt(p1_bears), (2, 2));
    // Activated: the player who activates Jade Mage's ability creates the token.
    let mage = t.battlefield(P1, "Jade Mage");
    let treason = t.hand(P0, "Act of Treason");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, treason).target(mage).go();
    t.resolve();
    t.lands(P0, "Forest", 3);
    t.activate(P0, mage, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert!(tokens_of(&t, P1).is_empty());
}

#[test]
fn you_means_the_owner_of_an_object_without_a_controller() {
    cr!("109.5");
    let mut t = TestGame::new(2);
    // Bloodghast in P0's graveyard: "Whenever a land you control enters, you may return
    // this card from your graveyard to the battlefield." "You" is its owner.
    let ghast = t.graveyard(P0, "Bloodghast");
    t.set_step(P1, Step::PrecombatMain);
    let island = t.hand(P1, "Island");
    t.play_land(P1, island).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(ghast), Zone::Graveyard(P0));
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    let swamp = t.hand(P0, "Swamp");
    t.play_land(P0, swamp).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(ghast));
    assert_eq!(t.obj_now(ghast).controller, P0);
}

#[test]
fn you_means_the_would_be_controller_of_a_spell_being_cast() {
    cr!("109.5");
    let mut t = TestGame::new(2);
    // Frogmite: affinity for artifacts — "This spell costs {1} less to cast for each
    // artifact you control": the artifacts of the player casting it.
    for _ in 0..4 {
        t.battlefield(P1, "Ornithopter");
    }
    let frog = t.hand(P0, "Frogmite");
    assert!(t.cast(P0, frog).try_go().is_err());
    let frog2 = t.hand(P1, "Frogmite");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, frog2).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Frogmite").len(), 1);
    let _ = card("Frogmite");
}
