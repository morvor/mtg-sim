//! CR 702.162 More Than Meets the Eye.

use crate::common_k702_153_167::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState};
use mtg_engine::testing::*;
use mtg_engine::*;

const MTMTE: CastMethod = CastMethod::Keyword(KeywordKind::MoreThanMeetsTheEye);

#[test]
fn more_than_meets_the_eye_casts_the_card_converted_for_its_cost() {
    cr!("702.162", "702.162a");
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "When you cast a spell using its More Than Meets the Eye ability, the card is put onto the stack with its back face up. The resulting spell has all characteristics of that face."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let card = t.hand(P0, "Starscream, Power Hungry");
    // Three lands: not enough for its {3}{B} mana cost, enough for {2}{B}.
    assert!(t.cast(P0, card).try_go().is_err());
    let spell = t.cast(P0, card).method(MTMTE).go();
    let s = t.g.obj(spell);
    // The spell has its back face up and only its back face's characteristics
    // (CR 712.8c), but the mana value of its front face.
    assert_eq!(s.face, FaceState::Back);
    assert_eq!(s.chars.name, "Starscream, Seeker Leader");
    assert!(s.chars.has_subtype("Vehicle"));
    assert_eq!(t.g.mana_value_of(spell), 4);
    assert!(t.g.player(P0).mana_pool.is_empty());
    t.resolve();
    // It enters converted: the Vehicle face is up.
    let v = named(&t, P0, "Starscream, Seeker Leader");
    assert_eq!(v.len(), 1);
    assert_eq!(t.g.obj(v[0]).face, FaceState::Back);
    assert!(has_kw(&t, v[0], KeywordKind::Menace));
}

#[test]
fn more_than_meets_the_eye_is_an_alternative_cost_from_zones_it_can_be_cast_from() {
    cr!("702.162a");
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "It functions in any zone from which the spell can be cast."
    );
    // Not from the graveyard without a permission to cast it from there.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let card = t.graveyard(P0, "Starscream, Power Hungry");
    assert!(t.cast(P0, card).method(MTMTE).try_go().is_err());
    // Cast normally, it's front face up.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let card = t.hand(P0, "Starscream, Power Hungry");
    let spell = t.cast(P0, card).go();
    assert_eq!(t.g.obj(spell).face, FaceState::Front);
    t.resolve();
    assert_eq!(named(&t, P0, "Starscream, Power Hungry").len(), 1);
}

#[test]
fn cost_changes_apply_to_the_more_than_meets_the_eye_cost() {
    cr!("702.162a");
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a More Than Meets the Eye cost) you're paying, add any cost increases, then apply any cost reductions."
    );
    // Etherium Sculptor: "Artifact spells you cast cost {1} less to cast." {2}{B} - {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Etherium Sculptor");
    t.lands(P0, "Swamp", 2);
    let card = t.hand(P0, "Starscream, Power Hungry");
    let spell = t.cast(P0, card).method(MTMTE).go();
    // Its mana value is still that of its front face.
    assert_eq!(t.g.mana_value_of(spell), 4);
}

#[test]
fn a_copy_of_a_converted_spell_has_the_back_face_characteristics() {
    cr!("702.162a");
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "If you copy a permanent spell cast this way, the copy has the characteristics of the card's back face, even though it isn't itself a double-faced card."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let card = t.hand(P0, "Starscream, Power Hungry");
    let spell = t.cast(P0, card).method(MTMTE).go();
    run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::CopySpell {
            what: mtg_engine::ability::Sel::Target(0),
            count: mtg_engine::ability::Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    let copy =
        t.g.stack
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).kind == mtg_engine::object::ObjKind::SpellCopy)
            .expect("a copy");
    assert_eq!(t.g.obj(copy).chars.name, "Starscream, Seeker Leader");
    t.resolve();
    let token = named(&t, P0, "Starscream, Seeker Leader");
    assert_eq!(token.len(), 1);
    assert!(t.obj(token[0]).is_token());
    assert!(has_kw(&t, token[0], KeywordKind::LivingMetal));
}

/// A double-faced card: "Kicker Robot" ({3} 2/2 artifact creature, "More Than Meets the
/// Eye {1}") whose back face "Kicker Coupe" (3/3) has "Kicker {2}".
fn kicker_robot() -> mtg_engine::card::CardDef {
    use crate::common_k702_038_051::with_cost;
    let mut def = with_cost(
        custom_card(
            "Kicker Robot",
            "Artifact Creature — Robot",
            Some((2, 2)),
            "More Than Meets the Eye {1}",
        ),
        "{3}",
    );
    let back = custom_card(
        "Kicker Coupe",
        "Artifact Creature — Robot",
        Some((3, 3)),
        "Kicker {2}",
    );
    def.layout = mtg_engine::card::Layout::Transform;
    def.faces.push(mtg_engine::card::FaceDef {
        chars: back.faces[0].chars.clone(),
        unsupported: vec![],
        star_power: false,
        star_toughness: false,
    });
    def
}

#[test]
fn more_than_meets_the_eye_is_an_alternative_cost() {
    cr!("702.162a", "118.9a");
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "The cost is an alternative cost, so it can't be combined with any other alternative costs. It can be combined with any applicable additional costs."
    );
    // An effect lets P0 cast the exiled Starscream without paying its mana cost: that's
    // an alternative cost, which can't be combined with More Than Meets the Eye.
    let grant = |t: &mut TestGame, card: ObjectId| {
        run_effect(
            t,
            None,
            P0,
            mtg_engine::ability::Effect::GrantPlayPermission {
                who: mtg_engine::ability::PlayerRef::You,
                what: mtg_engine::ability::Sel::Target(0),
                duration: mtg_engine::ability::Duration::EndOfTurn,
                free: true,
            },
            &[Entity::Object(card)],
        );
    };
    let mut t = TestGame::new(2);
    let card = t.exile(P0, "Starscream, Power Hungry");
    grant(&mut t, card);
    assert!(t.cast(P0, card).method(MTMTE).try_go().is_err());
    let card = t.g.current(card);
    let spell = t.cast(P0, card).method(CastMethod::Free).go();
    assert_eq!(t.g.obj(spell).face, FaceState::Front);
    // Even with {2}{B}, it can't be cast converted instead: the permission is to cast
    // it without paying its mana cost, which is itself an alternative cost (see the
    // Nicol Bolas, God-Pharaoh ruling in tests/rulings/r_s22_exile_until.rs).
    let mut t = TestGame::new(2);
    let card = t.exile(P0, "Starscream, Power Hungry");
    grant(&mut t, card);
    t.lands(P0, "Swamp", 3);
    assert!(t.cast(P0, card).method(MTMTE).try_go().is_err());
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).tapped)
            .count(),
        0
    );
    // From the hand, it can be cast converted for {2}{B}.
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Starscream, Power Hungry");
    t.lands(P0, "Swamp", 3);
    let spell = t.cast(P0, card).method(MTMTE).go();
    assert_eq!(t.g.obj(spell).face, FaceState::Back);
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).tapped)
            .count(),
        3
    );
    // An additional cost can be paid along with it: the converted spell's kicker.
    let mut t = TestGame::new(2);
    let card = t.custom(P0, kicker_robot(), mtg_engine::object::Zone::Hand(P0));
    t.lands(P0, "Plains", 3);
    let spell = t.cast(P0, card).method(MTMTE).kicked(true).go();
    let s = t.g.obj(spell);
    assert_eq!(s.chars.name, "Kicker Coupe");
    let paid = &s.stack.as_ref().expect("a spell").cast.paid;
    assert!(paid.iter().any(|p| p == "kicker"), "{paid:?}");
    assert!(
        paid.iter().any(|p| p == "more than meets the eye"),
        "{paid:?}"
    );
    assert!(t
        .g
        .battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.is_land())
        .all(|id| t.g.obj(*id).tapped));
}

#[test]
fn converting_a_permanent_turns_it_over() {
    cr!("702.162a", "701.28a");
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "The convert keyword action functions the same way as the transform keyword action found on some other cards; to convert a permanent on the battlefield, turn it over so that its other face is up."
    );
    // Starscream, Power Hungry: "Whenever one or more creatures deal combat damage to you,
    // convert Starscream."
    let mut t = TestGame::new(2);
    let star = t.battlefield(P0, "Starscream, Power Hungry");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    t.resolve_all();
    // The same permanent, with its other face up.
    assert!(t.g.is_live(star));
    let o = t.g.obj(star);
    assert_eq!(o.face, FaceState::Back);
    assert_eq!(o.chars.name, "Starscream, Seeker Leader");
    assert!(o.chars.has_keyword(KeywordKind::LivingMetal));
}
