//! Rulings batch S05 — disguise (CR 702.168) and face-down spells and permanents
//! (CR 708).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::facedown;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts `name` face down with its disguise ability, paying {3} with three Wastes.
/// Returns the spell.
fn cast_disguised(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, name);
    t.cast(p, card)
        .method(CastMethod::FaceDown(KeywordKind::Disguise))
        .go()
}

/// `name` cast face down with disguise and resolved: the face-down permanent.
fn disguised(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let s = cast_disguised(t, p, name);
    t.resolve_all();
    let id = t.g.current(s);
    assert!(t.obj(id).face_down && t.on_battlefield(id));
    id
}

/// Whether turning `obj` face up is among `p`'s legal actions now.
fn can_turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .contains(&Action::Special(SpecialAction::TurnFaceUp { obj }))
}

/// `p` turns `obj` face up (the special action). Whether it was legal.
fn turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let r =
        t.g.perform_action(p, Action::Special(SpecialAction::TurnFaceUp { obj }));
    t.g.flush_events();
    r.is_ok()
}

fn assert_face_down_2_2(t: &TestGame, id: ObjectId) {
    let o = t.obj(id);
    assert!(o.face_down);
    assert!(o.chars.name.is_empty());
    assert_eq!((o.chars.power, o.chars.toughness), (Some(2), Some(2)));
    assert!(o.chars.is(CardType::Creature));
    assert!(o.chars.subtypes.is_empty());
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert_eq!(t.g.mana_value_of(id), 0);
    assert!(o.chars.has_keyword(KeywordKind::Ward));
}

#[test]
fn a_face_down_spell_costs_3_as_an_alternative_cost_and_has_mana_value_0() {
    cr!("702.168a", "702.168b", "708.4", "118.9");
    ruling!(
        "Alley Assailant",
        "The face-down spell has no mana cost and a mana value of 0. When you cast a face-down spell, put it on the stack face down so no other player knows what it is, and pay {3} to cast it. This is an alternative cost."
    );
    supported("Alley Assailant");
    // Alley Assailant ({2}{B}) cast face down with three colorless mana: {3} is paid
    // rather than its mana cost.
    let mut t = TestGame::new(2);
    let spell = cast_disguised(&mut t, P0, "Alley Assailant");
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_face_down_2_2(&t, spell);
    assert!(!facedown::can_look_at(&t.g, P1, spell));
    assert!(facedown::can_look_at(&t.g, P0, spell));
    // Three colorless mana pay for it face down but not for its mana cost; two lands
    // don't pay for it face down.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Alley Assailant");
    let face_down = CastMethod::FaceDown(KeywordKind::Disguise);
    assert!(can_cast(&mut t, P0, card, face_down.clone()));
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let card = t.hand(P0, "Alley Assailant");
    assert!(!can_cast(&mut t, P0, card, face_down));
}

#[test]
fn a_face_down_creature_that_lost_its_abilities_cant_be_turned_up_with_disguise() {
    cr!("702.168d", "708.7", "613.1f");
    ruling!(
        "Alley Assailant",
        "If a face-down creature loses its abilities, it can't be turned face up with a disguise ability because it will no longer have a disguise ability (or a disguise cost) once face up."
    );
    ruling!(
        "Arno Dorian",
        "If a face-down creature loses its abilities, it can’t be turned face up with a disguise ability because it will no longer have a disguise ability (or a disguise cost) once face up."
    );
    supported("Humility");
    supported("Arno Dorian");
    let mut t = TestGame::new(2);
    let arno = disguised(&mut t, P0, "Arno Dorian");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    assert!(can_turn_up(&mut t, P0, arno));
    // Humility: "All creatures lose all abilities and have base power and toughness 1/1."
    let humility = t.battlefield(P1, "Humility");
    t.g.recompute();
    assert!(t.obj(arno).chars.abilities.is_empty());
    assert!(!can_turn_up(&mut t, P0, arno));
    assert!(!turn_up(&mut t, P0, arno));
    assert!(t.obj(arno).face_down);
    // Without Humility, it can be.
    destroy(&mut t, humility);
    assert!(can_turn_up(&mut t, P0, arno));
    assert!(turn_up(&mut t, P0, arno));
    assert_eq!(t.obj(arno).chars.name.as_str(), "Arno Dorian");
}

#[test]
fn a_disguised_permanent_is_turned_face_up_any_time_its_controller_has_priority() {
    cr!("702.168a", "702.168d", "116.2b");
    ruling!(
        "Alley Assailant",
        "A disguise ability lets you cast a card face down by paying {3} and announcing that you are using a disguise ability. Any time you have priority, you can turn a face-down permanent with disguise face up by paying its disguise cost."
    );
    ruling!(
        "Arno Dorian",
        "A disguise ability lets you cast a card face down by paying {3} and announcing that you are using a disguise ability. Any time you have priority, you can turn a face-down permanent you control face up by paying its disguise cost."
    );
    // Arno Dorian: disguise {B}{R}. During the opponent's turn, with a spell on the stack.
    let mut t = TestGame::new(2);
    let arno = disguised(&mut t, P0, "Arno Dorian");
    t.advance_to(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    // Not by the opponent, and not without the mana.
    assert!(!can_turn_up(&mut t, P1, arno));
    assert!(!can_turn_up(&mut t, P0, arno));
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    assert!(turn_up(&mut t, P0, arno));
    assert_eq!(t.obj(arno).chars.name.as_str(), "Arno Dorian");
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn turning_face_up_is_a_special_action_that_doesnt_use_the_stack() {
    cr!("702.168d", "116.2b", "708.7");
    ruling!(
        "Arno Dorian",
        "Any time you have priority, you may turn the face-down creature face up by revealing what its disguise cost is and paying that cost. This is a special action. It doesn’t use the stack and can’t be responded to. Only a face-down permanent can be turned face up this way; a face-down spell cannot."
    );
    let mut t = TestGame::new(2);
    let spell = cast_disguised(&mut t, P0, "Arno Dorian");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    // A face-down spell can't be turned face up.
    assert!(!can_turn_up(&mut t, P0, spell));
    assert!(!turn_up(&mut t, P0, spell));
    t.resolve_all();
    let arno = t.g.current(spell);
    // Turning the permanent face up puts nothing on the stack and gives no one priority
    // before it's done.
    let from = t.asked().len();
    assert!(turn_up(&mut t, P0, arno));
    assert_eq!(t.stack_len(), 0);
    assert!(!t.obj(arno).face_down);
    assert_eq!(
        asked_of_since(&t, P1, from, |d| matches!(
            d,
            mtg_engine::decision::Decision::Priority { .. }
        )),
        0
    );
}

#[test]
fn face_down_spells_and_creatures_are_nameless_colorless_2_2s_with_ward_2() {
    cr!("702.168a", "708.2", "708.4");
    ruling!(
        "Arno Dorian",
        "Both the spell and the resulting creature are colorless and have a mana value of 0. Other effects that apply to the spell or creature can still grant it any characteristics it doesn’t have or change the characteristics it does have."
    );
    supported("Cerulean Wisps");
    let mut t = TestGame::new(2);
    let spell = cast_disguised(&mut t, P0, "Arno Dorian");
    assert_face_down_2_2(&t, spell);
    t.resolve_all();
    let arno = t.g.current(spell);
    assert_face_down_2_2(&t, arno);
    // Other effects still apply: it becomes blue, and gets +3/+3.
    t.lands(P0, "Island", 1);
    t.lands(P0, "Forest", 1);
    let wisps = t.hand(P0, "Cerulean Wisps");
    t.cast(P0, wisps).target(arno).go();
    t.resolve_all();
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(arno).go();
    t.resolve_all();
    assert_eq!(colors(&t, arno), ColorSet::single(Color::Blue));
    assert_eq!(t.pt(arno), (5, 5));
    assert!(t.obj(arno).face_down);
}

#[test]
fn turning_face_up_keeps_targets_and_attachments() {
    cr!("708.8", "702.168d");
    ruling!(
        "Arno Dorian",
        "A permanent that turns face up or face down changes characteristics but is otherwise the same permanent. Spells and abilities that were targeting that permanent and Auras and Equipment that were attached to that permanent aren’t affected unless the new characteristics of the object change the legality of those targets or attachments."
    );
    supported("Rancor");
    let mut t = TestGame::new(2);
    let arno = disguised(&mut t, P0, "Arno Dorian");
    // Rancor on it ("Enchanted creature gets +2/+0 and has trample"), and a Giant Growth
    // targeting it.
    t.lands(P0, "Forest", 2);
    let rancor = t.hand(P0, "Rancor");
    t.cast(P0, rancor).target(arno).go();
    t.resolve_all();
    let rancor = t.named_on_battlefield("Rancor")[0];
    assert_eq!(t.obj(rancor).attached_to, Some(Entity::Object(arno)));
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(arno).go();
    // Turned face up in response: the same permanent.
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    assert!(turn_up(&mut t, P0, arno));
    assert!(t.g.is_live(arno));
    t.resolve_all();
    assert_eq!(t.obj(arno).chars.name.as_str(), "Arno Dorian");
    // Arno Dorian is 3/3: +2/+0 from Rancor, +3/+3 from Giant Growth.
    assert_eq!(t.pt(arno), (8, 6));
    assert_eq!(t.obj(rancor).attached_to, Some(Entity::Object(arno)));
    assert!(t.obj(arno).chars.has_keyword(KeywordKind::Trample));
}

#[test]
fn turning_face_up_or_down_doesnt_change_whether_it_is_tapped() {
    cr!("708.8", "708.2a");
    ruling!(
        "Arno Dorian",
        "Turning a permanent face up or face down doesn’t change whether that permanent is tapped or untapped."
    );
    let mut t = TestGame::new(2);
    let arno = disguised(&mut t, P0, "Arno Dorian");
    t.g.tap(arno);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    assert!(turn_up(&mut t, P0, arno));
    assert!(t.obj(arno).tapped);
    // Turned face down again (a 2/2 with no text): still tapped; untapped stays untapped.
    run_from(
        &mut t,
        P0,
        None,
        Effect::TurnFaceDown {
            what: Sel::Target(0),
        },
        &[Entity::Object(arno)],
    );
    assert!(t.obj(arno).face_down && t.obj(arno).tapped);
    let other = disguised(&mut t, P0, "Aveline de Grandpré");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    assert!(turn_up(&mut t, P0, other));
    assert!(!t.obj(other).tapped);
}

#[test]
fn face_down_creatures_dont_share_a_name_with_anything() {
    cr!("201.2a", "708.2");
    ruling!(
        "Arno Dorian",
        "Because face-down creatures don’t have a name, they can’t have the same name as any other creature, even another face-down creature."
    );
    supported("Bifurcate");
    supported("Bolrac-Clan Basher");
    let mut t = TestGame::new(2);
    let a = disguised(&mut t, P0, "Bolrac-Clan Basher");
    let b = disguised(&mut t, P0, "Bolrac-Clan Basher");
    // Two face-down Bolrac-Clan Bashers don't have the same name.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(a)]];
    let same = Filter::SameNameAs(Box::new(Sel::Target(0)));
    assert!(!t.g.matches(b, &same, &ctx));
    assert!(!t.g.matches(a, &same, &ctx));
    // Bifurcate: "Search your library for a permanent card with the same name as target
    // nontoken creature, put that card onto the battlefield" finds no Basher.
    let lib = t.library_top(P0, "Bolrac-Clan Basher");
    give_mana_for(&mut t, P0, "Bifurcate");
    let bif = t.hand(P0, "Bifurcate");
    t.cast(P0, bif).target(a).go();
    t.resolve_all();
    assert_eq!(t.zone(lib), Zone::Library(P0));
    // A face-up one has the same name as the library card.
    let face_up = t.battlefield(P0, "Bolrac-Clan Basher");
    give_mana_for(&mut t, P0, "Bifurcate");
    let bif = t.hand(P0, "Bifurcate");
    t.cast(P0, bif).target(face_up).go();
    t.resolve_all();
    assert_eq!(t.zone(lib), Zone::Battlefield);
}

#[test]
fn you_can_look_only_at_face_down_spells_and_permanents_you_control() {
    cr!("708.5");
    ruling!(
        "Arno Dorian",
        "At any time, you can look at a face-down spell or permanent you control. You can’t look at face-down permanents or spells you don’t control unless an effect instructs or allows you to do so."
    );
    let mut t = TestGame::new(2);
    let spell = cast_disguised(&mut t, P0, "Arno Dorian");
    assert!(facedown::can_look_at(&t.g, P0, spell));
    assert!(!facedown::can_look_at(&t.g, P1, spell));
    t.resolve_all();
    let arno = t.g.current(spell);
    assert!(facedown::can_look_at(&t.g, P0, arno));
    assert!(!facedown::can_look_at(&t.g, P1, arno));
    // P1's own face-down creature: P1 may look at it, P0 may not.
    t.advance_to(P1, Step::PrecombatMain);
    let theirs = disguised(&mut t, P1, "Bayek of Siwa");
    assert!(facedown::can_look_at(&t.g, P1, theirs));
    assert!(!facedown::can_look_at(&t.g, P0, theirs));
}
