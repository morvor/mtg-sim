//! CR 400.7–400.10, 403.4, 406.7: an object that changes zones becomes a new object, and
//! the exceptions that let effects follow it.

use crate::r600_common::*;
use crate::r703_common::{archenemy_game, add_scheme_deck, oracle_card, run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::stickers::{self, SheetFormat, StickerDef, StickerKind, StickerSheet};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The given objects, which are in a zone of this kind.
fn in_zone(zone: ZoneKind, ids: Vec<ObjectId>) -> Sel {
    Sel::All(Filter::and(vec![Filter::InZone(zone), Filter::Objects(ids)]))
}

// ---------------------------------------------------------------------------
// 400.7a–400.7c: from a permanent spell to the permanent
// ---------------------------------------------------------------------------

#[test]
fn changes_to_a_permanent_spells_characteristics_and_controller_carry_over() {
    cr!("400.7", "400.7a");
    supported("Chaoslace");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    // "Target spell or permanent becomes red." and "Gain control of target spell."
    t.lands(P0, "Mountain", 1);
    let lace = t.hand(P0, "Chaoslace");
    t.cast(P0, lace).target(spell).go();
    t.resolve();
    let thief = t.custom(
        P0,
        compile_def("Spell Thief", "Instant", "{0}", "Gain control of target spell."),
        Zone::Hand(P0),
    );
    t.cast(P0, thief).target(spell).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    let perm = t.named_on_battlefield("Grizzly Bears")[0];
    assert_ne!(perm, spell);
    assert_eq!(t.obj(perm).chars.colors, ColorSet::single(Color::Red));
    assert_eq!(t.obj(perm).controller, P0);
    // The effects don't follow it to yet another zone: returned to its owner's hand,
    // it's green again.
    t.g.move_object(perm, Zone::Hand(P1), MoveCause::Effect, None);
    t.g.recompute();
    assert_eq!(t.obj_now(perm).chars.colors, ColorSet::single(Color::Green));
}

/// "You may cast creature spells from your graveyard. Creature spells you cast from your
/// graveyard have haste." — with no duration: the granted ability lasts for the game.
fn graveyard_paragon() -> CardDef {
    CB::new("Graveyard Paragon")
        .creature(3, 3)
        .ability(stat(StaticEffect::PlayPermission(PlayPermission {
            who: PlayerRel::You,
            zone: ZoneKind::Graveyard,
            top_only: false,
            what: Filter::creature(),
            lands: false,
            spells: true,
            cost: None,
        })))
        .ability(stat(StaticEffect::CastGrant {
            zone: ZoneKind::Graveyard,
            what: Filter::creature(),
            mods: vec![Modification::AddKeyword(keywords::Keyword::new(
                KeywordKind::Haste,
            ))],
        }))
        .build()
}

#[test]
fn a_static_ability_granting_an_ability_to_a_permanent_spell_carries_over() {
    cr!("400.7b");
    let mut t = TestGame::new(2);
    let paragon = t.custom(P0, graveyard_paragon(), Zone::Battlefield);
    t.lands(P0, "Forest", 2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    t.resolve();
    let perm = t.named_on_battlefield("Grizzly Bears")[0];
    assert_ne!(perm, spell);
    assert!(t.obj(perm).has_keyword(KeywordKind::Haste));
    // Even once the Paragon is gone.
    t.g.move_object(paragon, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.g.recompute();
    assert!(t.obj(perm).has_keyword(KeywordKind::Haste));
}

#[test]
fn prevention_of_damage_from_a_permanent_spell_carries_over() {
    cr!("400.7c");
    supported("Circle of Protection: Red");
    supported("Keldon Marauders");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let circle = t.battlefield(P0, "Circle of Protection: Red");
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Mountain", 2);
    let marauders = t.hand(P1, "Keldon Marauders");
    let spell = t.cast(P1, marauders).go();
    // P0 chooses the Marauders spell as the red source.
    t.answer_choose(P0, &[Entity::Object(spell)]);
    t.activate(P0, circle, 0, &[]).unwrap();
    t.resolve();
    // The spell resolves; its enters trigger targets P0.
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve();
    let perm = t.named_on_battlefield("Keldon Marauders")[0];
    assert_ne!(perm, spell);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "the damage from the permanent was prevented");
    // Without the shield, it would have dealt 1 damage.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    let marauders = t.hand(P1, "Keldon Marauders");
    t.cast(P1, marauders).go();
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

// ---------------------------------------------------------------------------
// 400.7f: Auras that left with the enchanted permanent
// ---------------------------------------------------------------------------

#[test]
fn an_ability_can_find_an_aura_that_went_to_the_graveyard_with_its_permanent() {
    cr!("400.7f");
    supported("Angelic Destiny");
    ruling!(
        "Angelic Destiny",
        "If Angelic Destiny is no longer in a graveyard when its triggered ability resolves, it won't be returned"
    );
    // The Aura is put into its owner's graveyard as a state-based action.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let destiny = t.hand(P0, "Angelic Destiny");
    t.cast(P0, destiny).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (6, 6));
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Angelic Destiny"));
    // Both are put into the graveyard at the same time.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let destiny = t.hand(P0, "Angelic Destiny");
    t.cast(P0, destiny).target(bears).go();
    t.resolve();
    let aura = t.named_on_battlefield("Angelic Destiny")[0];
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Destroy {
            what: Sel::All(Filter::Objects(vec![bears, aura])),
            no_regen: false,
        },
        &[],
    );
    t.resolve_all();
    assert!(t.in_hand(P0, "Angelic Destiny"));
    // If it has left the graveyard by the time the ability resolves, it stays where it is.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let destiny = t.hand(P0, "Angelic Destiny");
    t.cast(P0, destiny).target(bears).go();
    t.resolve();
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let in_gy = t.g.find_in_zone(Zone::Graveyard(P0), "Angelic Destiny")[0];
    t.g.move_object(in_gy, Zone::Exile, MoveCause::Effect, None);
    t.resolve_all();
    assert!(!t.in_hand(P0, "Angelic Destiny"));
    assert!(t.in_exile("Angelic Destiny"));
}

// ---------------------------------------------------------------------------
// 400.7j: other parts of an effect find what it moved to a public zone
// ---------------------------------------------------------------------------

#[test]
fn an_effect_finds_the_object_it_moved_to_a_public_zone() {
    cr!("400.7j", "403.4");
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 2, None);
    t.g.deal_damage_batch(vec![(bears, Entity::Object(bears), 1)], false);
    t.lands(P0, "Plains", 1);
    let shift = t.hand(P0, "Cloudshift");
    t.cast(P0, shift).target(bears).go();
    t.resolve();
    // "Exile target creature you control, then return that card to the battlefield": the
    // return found the card in exile. It's a new permanent (CR 403.4): no counters, no
    // damage.
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], bears);
    assert_eq!(t.obj(back[0]).damage, 0);
    assert_eq!(t.counters(back[0], counters::PLUS1), 0);
    assert_eq!(t.pt(back[0]), (2, 2));
}

// ---------------------------------------------------------------------------
// 400.7m: stickers
// ---------------------------------------------------------------------------

#[test]
fn stickers_stay_on_an_object_moving_between_public_zones() {
    cr!("400.7m");
    let mut t = TestGame::new(2);
    stickers::choose_sheets(
        &mut t.g,
        P0,
        vec![StickerSheet {
            name: "Big".into(),
            stickers: vec![StickerDef {
                kind: StickerKind::PowerToughness(5, 5),
                ticket_cost: 0,
            }],
        }],
        SheetFormat::Limited,
    )
    .unwrap();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert!(stickers::put_from_sheets(
        &mut t.g, P0, bears, None, None, false
    ));
    t.clear_answers();
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    // Battlefield → graveyard → exile: public zones. The sticker and its effect follow.
    let gy = t
        .g
        .move_object(bears, Zone::Graveyard(P0), MoveCause::Effect, Some(P0))
        .unwrap();
    let ex = t
        .g
        .move_object(gy, Zone::Exile, MoveCause::Effect, Some(P0))
        .unwrap();
    t.g.recompute();
    assert!(stickers::is_stickered(&t.g, ex));
    assert_eq!(t.obj(ex).chars.power, Some(5));
    // Into a hand (hidden), it's gone.
    let hand = t
        .g
        .move_object(ex, Zone::Hand(P0), MoveCause::Effect, Some(P0))
        .unwrap();
    t.g.recompute();
    assert!(!stickers::is_stickered(&t.g, hand));
    assert_eq!(t.obj(hand).chars.power, Some(2));
}

// ---------------------------------------------------------------------------
// 400.8, 406.7: exiling an exiled object
// ---------------------------------------------------------------------------

#[test]
fn an_exiled_object_that_is_exiled_again_becomes_a_new_object() {
    cr!("400.8", "406.7");
    supported("Banisher Priest");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let priest = t.hand(P0, "Banisher Priest");
    t.cast(P0, priest).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    let exiled = t.g.current(bears);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // Another effect exiles the exiled card: it stays in exile, as a new object that has
    // just been exiled.
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Exile {
            what: in_zone(ZoneKind::Exile, vec![exiled]),
            face_down: false,
            link: false,
        },
        &[],
    );
    assert!(!t.is_live(exiled));
    let again = t.g.current(exiled);
    assert_ne!(again, exiled);
    assert_eq!(t.zone(again), Zone::Exile);
    // So it's no longer the card Banisher Priest exiled: it doesn't return.
    let priest = t.named_on_battlefield("Banisher Priest")[0];
    t.g.destroy(priest, None);
    t.resolve_all();
    assert_eq!(t.zone(again), Zone::Exile);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

// ---------------------------------------------------------------------------
// 400.9, 400.10: the command zone
// ---------------------------------------------------------------------------

#[test]
fn a_face_up_object_in_the_command_zone_turned_face_down_becomes_a_new_object() {
    cr!("400.9");
    let mut t = archenemy_game();
    let ongoing = oracle_card(
        "Endless Plot",
        "Ongoing Scheme",
        "",
        None,
        "At the beginning of your end step, abandon this scheme.",
    );
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![ongoing, oracle_card("Idle Scheme", "Scheme", "", None, "")],
    );
    crate::r703_common::keyword_action(&mut t, P0, KeywordAction::SetInMotion, 1);
    t.settle();
    let face_up = deck[0];
    assert!(t.is_live(face_up) && !t.obj(face_up).face_down);
    // Abandoned, it's turned face down: a new object.
    crate::r703_common::to_step_start(&mut t, P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(!t.is_live(face_up));
    let now = t.g.current(face_up);
    assert_ne!(now, face_up);
    assert!(t.obj(now).face_down);
    assert_eq!(t.zone(now), Zone::Command);
}

#[test]
fn an_object_put_into_the_command_zone_from_it_becomes_a_new_object() {
    cr!("400.10");
    let mut t = TestGame::new(2);
    let emblem_source = t.command(P0, "Grizzly Bears");
    let moved = t
        .g
        .move_object(emblem_source, Zone::Command, MoveCause::Effect, Some(P0))
        .unwrap();
    assert_ne!(moved, emblem_source);
    assert!(!t.is_live(emblem_source));
    assert_eq!(t.zone(moved), Zone::Command);
    assert_eq!(t.g.command.iter().filter(|x| **x == moved).count(), 1);
    assert!(!t.g.command.contains(&emblem_source));
}
