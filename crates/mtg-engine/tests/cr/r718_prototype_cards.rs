//! CR 718: prototype cards.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::prototype;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Blitz Automaton ({7} 6/4 Artifact Creature — Construct, "Prototype {2}{R} — 3/2",
/// haste).
const BLITZ: &str = "Blitz Automaton";

fn prototyped() -> CastMethod {
    CastMethod::Keyword(KeywordKind::Prototype)
}

fn red() -> ColorSet {
    ColorSet::single(Color::Red)
}

/// Casts Blitz Automaton as a prototyped spell with {2}{R} from Mountains.
fn cast_prototyped(t: &mut TestGame) -> ObjectId {
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    let b = t.hand(P0, BLITZ);
    t.cast(P0, b).method(prototyped()).go()
}

#[test]
fn a_prototype_card_has_a_second_set_of_mana_cost_power_and_toughness() {
    cr!("718.1", "718.2");
    supported(BLITZ);
    let def = card(BLITZ);
    assert_eq!(def.layout, Layout::Prototype);
    let kw = def
        .front()
        .chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Prototype)
        .cloned()
        .expect("prototype keyword");
    let (mana, p, t) = prototype::prototype_values(&kw).unwrap();
    assert_eq!(mana.to_string(), "{2}{R}");
    assert_eq!((p, t), (3, 2));
    // The card's normal characteristics appear as usual.
    let c = &def.front().chars;
    assert_eq!(c.mana_cost.as_ref().unwrap().to_string(), "{7}");
    assert_eq!((c.power, c.toughness), (Some(6), Some(4)));
}

#[test]
fn a_player_chooses_to_cast_it_normally_or_prototyped() {
    cr!("718.3", "718.3a");
    ruling!(
        "Goring Warplow",
        "When casting a prototyped spell, use only its prototype characteristics to determine whether it's legal to cast it"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let b = t.hand(P0, BLITZ);
    let methods: Vec<CastMethod> =
        t.g.cast_options(P0, b)
            .into_iter()
            .map(|o| o.method)
            .collect();
    assert!(methods.contains(&CastMethod::Normal) && methods.contains(&prototyped()));
    // With {2}{R}: only as a prototyped spell.
    t.lands(P0, "Mountain", 3);
    assert!(t.cast(P0, b).try_go().is_err());
    let b = t.g.current(b);
    assert!(t.cast(P0, b).method(prototyped()).try_go().is_ok());
    // "Creature spells with power 4 or greater can't be cast": the prototyped spell's
    // power is 3, so it can be; the normal one's is 6.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.custom(
        P1,
        CB::new("Low Ceiling")
            .enchantment()
            .ability(stat(StaticEffect::Restriction(Restriction::CantCast {
                who: PlayerFilter::Any,
                what: Filter::Power(Cmp::Ge, Box::new(Value::c(4))),
            })))
            .build(),
        Zone::Battlefield,
    );
    t.lands(P0, "Mountain", 7);
    let b = t.hand(P0, BLITZ);
    assert!(t.cast(P0, b).try_go().is_err());
    let b = t.g.current(b);
    assert!(t.cast(P0, b).method(prototyped()).try_go().is_ok());
}

#[test]
fn a_prototyped_spell_and_its_permanent_have_the_prototype_characteristics() {
    cr!("718.3b", "718.5");
    ruling!(
        "Goring Warplow",
        "Its color and mana value are determined by that mana cost"
    );
    let mut t = TestGame::new(2);
    let spell = cast_prototyped(&mut t);
    let check = |t: &mut TestGame, id: ObjectId| {
        t.g.recompute();
        let c = t.obj(id).chars.clone();
        assert_eq!(c.mana_cost.as_ref().unwrap().to_string(), "{2}{R}");
        assert_eq!(c.colors, red());
        assert_eq!(mv(t, id), 3);
        // Everything else is the card's.
        assert_eq!(c.name, BLITZ);
        assert!(c.is(CardType::Artifact) && c.is_creature() && c.has_subtype("Construct"));
        assert!(c.has_keyword(KeywordKind::Haste));
    };
    check(&mut t, spell);
    assert_eq!(
        (t.obj(spell).chars.power, t.obj(spell).chars.toughness),
        (Some(3), Some(2))
    );
    t.resolve_all();
    let perm = t.g.current(spell);
    assert_eq!(t.obj(perm).zone, Zone::Battlefield);
    check(&mut t, perm);
    assert_eq!(t.pt(perm), (3, 2));
}

#[test]
fn a_copy_of_a_prototyped_spell_is_prototyped() {
    cr!("718.3c");
    ruling!(
        "Goring Warplow",
        "If an effect copies a prototyped spell, that copy (as well as the token it becomes on the battlefield) will have the same characteristics"
    );
    let mut t = TestGame::new(2);
    let spell = cast_prototyped(&mut t);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(spell)],
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
    );
    let copy = t.g.stack[1];
    assert!(prototype::is_prototyped(t.obj(copy)));
    assert_eq!(t.obj(copy).chars.colors, red());
    assert_eq!(
        (t.obj(copy).chars.power, t.obj(copy).chars.toughness),
        (Some(3), Some(2))
    );
    t.resolve();
    let token =
        t.g.battlefield
            .iter()
            .copied()
            .find(|id| t.obj(*id).kind == ObjKind::Token)
            .expect("token");
    assert_eq!(t.pt(token), (3, 2));
    assert_eq!(mv(&mut t, token), 3);
}

#[test]
fn a_copy_of_a_prototyped_permanent_has_the_prototype_characteristics() {
    cr!("718.2a", "718.3d");
    ruling!(
        "Goring Warplow",
        "if an effect creates a token that's a copy of a prototyped permanent or causes another permanent to become a copy of it, the copy would have the same characteristics"
    );
    let mut t = TestGame::new(2);
    cast_prototyped(&mut t);
    t.resolve_all();
    let blitz = t.named_on_battlefield(BLITZ)[0];
    t.answer_choose(P1, &[Entity::Object(blitz)]);
    t.answer_yes(P1, true);
    let clone = t.enter(P1, "Clone");
    let clone = t.g.current(clone);
    assert_eq!(t.obj(clone).chars.name, BLITZ);
    assert_eq!(t.pt(clone), (3, 2));
    assert_eq!(t.obj(clone).chars.colors, red());
    assert_eq!(mv(&mut t, clone), 3);
}

#[test]
fn otherwise_a_prototype_card_has_its_normal_characteristics() {
    cr!("718.4");
    ruling!(
        "Goring Warplow",
        "A prototype card is a colorless card in every zone except the stack or the battlefield"
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, BLITZ);
    t.g.recompute();
    assert_eq!(t.obj(gy).chars.colors, ColorSet::NONE);
    assert_eq!(mv(&mut t, gy), 7);
    // Cast normally.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 7);
    let b = t.hand(P0, BLITZ);
    let spell = t.cast(P0, b).go();
    assert_eq!(mv(&mut t, spell), 7);
    t.resolve_all();
    let perm = t.g.current(spell);
    assert_eq!(t.pt(perm), (6, 4));
    // A prototyped permanent that leaves the battlefield resumes its normal
    // characteristics.
    let mut t = TestGame::new(2);
    cast_prototyped(&mut t);
    t.resolve_all();
    let blitz = t.named_on_battlefield(BLITZ)[0];
    let hand =
        t.g.move_object(blitz, Zone::Hand(P0), MoveCause::Effect, None)
            .unwrap();
    t.g.recompute();
    assert_eq!(t.obj(hand).chars.colors, ColorSet::NONE);
    assert_eq!(mv(&mut t, hand), 7);
    assert_eq!(t.obj(hand).chars.power, Some(6));
}
