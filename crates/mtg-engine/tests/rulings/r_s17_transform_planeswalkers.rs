//! Rulings batch S17 — transform (CR 701.27, 712): creatures that transform into
//! planeswalkers (Magic Origins' Nissa and Liliana, Modern Horizons 3's Ajani): the
//! characteristics of each face, mana value and color indicators, loyalty, casting only
//! the front face, face-down double-faced cards, and single-faced copies.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s08_common::{legal_cast_methods, mana_value};
use crate::r_s11_common::{can_turn_face_up, is_plain_face_down_2_2, manifest_card, turn_face_up};
use crate::r_s17_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Nissa, Vastwood Seer ({2}{G} 2/2 legendary creature) // Nissa, Sage Animist (green
/// color indicator, loyalty 3).
const NISSA: &str = "Nissa, Vastwood Seer // Nissa, Sage Animist";
/// Ajani, Nacatl Pariah ({1}{W} 1/2 Cat Warrior) // Ajani, Nacatl Avenger (red and white
/// color indicator, loyalty 3).
const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";
/// Liliana, Heretical Healer ({1}{B}{B} 2/3) // Liliana, Defiant Necromancer (loyalty 3).
const LILIANA: &str = "Liliana, Heretical Healer // Liliana, Defiant Necromancer";

/// `p` controls seven lands and Nissa; a land entering transforms her ("exile Nissa, then
/// return her to the battlefield transformed"). Returns Nissa, Sage Animist.
fn nissa_transformed_by_a_land(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let nissa = t.battlefield(p, NISSA);
    t.lands(p, "Forest", 6);
    let land = t.graveyard(p, "Forest");
    put_onto_battlefield(t, p, land, false).unwrap();
    t.resolve_all();
    let animist = t.named_on_battlefield("Nissa, Sage Animist");
    assert_eq!(animist.len(), 1, "Nissa didn't transform");
    assert!(!t.g.is_live(nissa));
    animist[0]
}

#[test]
fn each_face_has_its_own_power_toughness_and_loyalty() {
    cr!("712.8", "712.8a", "712.8d", "712.8e", "306.5b");
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "Each face of a double-faced card has its own set of characteristics: name, types, subtypes, power and toughness, loyalty, abilities, and so on."
    );
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "The mana value of a double-faced card not on the battlefield is the mana value of its front face."
    );
    supported(NISSA);
    let mut t = TestGame::new(2);
    // Off the battlefield: only the front face — a 2/2 creature with no loyalty, mana
    // value 3.
    for id in [t.hand(P0, NISSA), t.graveyard(P0, NISSA), t.exile(P0, NISSA)] {
        let o = t.obj(id);
        assert_eq!(o.chars.name, "Nissa, Vastwood Seer");
        assert!(o.is_creature() && !o.is(CardType::Planeswalker));
        assert_eq!((o.chars.power, o.chars.toughness), (Some(2), Some(2)));
        assert_eq!(o.chars.loyalty, None);
        assert_eq!(t.g.mana_value_of(id), 3);
    }
    // On the battlefield back face up: a planeswalker with loyalty 3, no power or
    // toughness, and only its own abilities.
    let animist = nissa_transformed_by_a_land(&mut t, P0);
    let o = t.obj(animist);
    assert!(o.is(CardType::Planeswalker) && !o.is_creature());
    assert_eq!((o.chars.power, o.chars.toughness), (None, None));
    assert_eq!(o.chars.loyalty, Some(3));
    assert_eq!(t.counters(animist, counters::LOYALTY), 3);
    assert!(o.chars.has_subtype("Nissa") && !o.chars.has_subtype("Elf"));
    assert!(!o
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("search your library")));
}

#[test]
fn the_back_face_has_no_mana_cost_and_a_color_indicator() {
    cr!("202.3b", "204.1", "712.8e");
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "The back face of a double-faced card doesn't have a mana cost. A double-faced permanent with its back face up has a mana value equal to the mana value of its front face. Each back face has a color indicator that defines its color."
    );
    let mut t = TestGame::new(2);
    let animist = nissa_transformed_by_a_land(&mut t, P0);
    let o = t.obj(animist);
    assert!(o
        .chars
        .mana_cost
        .as_ref()
        .is_none_or(|m| m.to_string().is_empty()));
    assert_eq!(mana_value(&t, animist), 3);
    assert_eq!(o.chars.colors, color_set(&[Color::Green]));
}

#[test]
fn a_creature_that_transforms_into_a_planeswalker_without_entering_has_no_loyalty() {
    cr!("701.27a", "704.5i", "712.18");
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "the resulting planeswalker won't have any loyalty counters on it and will subsequently be put into its owner's graveyard"
    );
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, NISSA);
    // An effect transforms her on the battlefield: she doesn't enter, so she gets no
    // loyalty counters, and she's put into her owner's graveyard.
    transform(&mut t, nissa);
    assert!(!t.g.is_live(nissa));
    assert!(t.in_graveyard(P0, "Nissa, Vastwood Seer"));
    assert!(t.named_on_battlefield("Nissa, Sage Animist").is_empty());
    // Exiled and returned transformed, she enters with her loyalty.
    let mut t = TestGame::new(2);
    let animist = nissa_transformed_by_a_land(&mut t, P0);
    assert_eq!(t.counters(animist, counters::LOYALTY), 3);
}

#[test]
fn a_manifested_double_faced_card_is_a_face_down_2_2_that_cant_transform() {
    cr!("701.40a", "712.15", "712.15a", "712.16", "701.40b");
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "If a double-faced card is manifested, it will be put onto the battlefield face down"
    );
    let mut t = TestGame::new(2);
    let m = manifest_card(&mut t, P0, NISSA);
    assert!(is_plain_face_down_2_2(&t, m));
    // While face down, it can't transform.
    transform(&mut t, m);
    assert!(is_plain_face_down_2_2(&t, m));
    // Its front face is a creature card: it can be turned face up for its mana cost, front
    // face up.
    t.lands(P0, "Forest", 3);
    assert!(can_turn_face_up(&mut t, P0, m));
    assert!(turn_face_up(&mut t, P0, m));
    t.settle();
    let o = t.obj_now(m);
    assert!(!o.face_down);
    assert_eq!(o.face, FaceState::Front);
    assert_eq!(o.chars.name, "Nissa, Vastwood Seer");
    // A double-faced card on the battlefield can't be turned face down.
    let nissa = t.g.current(m);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(nissa)]];
    t.g.exec(
        &Effect::TurnFaceDown {
            what: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(!t.obj(nissa).face_down);
    assert_eq!(t.obj(nissa).chars.name, "Nissa, Vastwood Seer");
}

#[test]
fn the_back_face_cant_be_cast() {
    cr!("712.11", "712.11c");
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "The back face of a double-faced card (in the case of Magic Origins, the planeswalker face) can't be cast."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let nissa = t.hand(P0, NISSA);
    // Only the front face may be cast: there's no way to cast the planeswalker face.
    let methods = legal_cast_methods(&mut t, P0, nissa);
    assert!(methods.contains(&CastMethod::Normal));
    assert!(!methods.iter().any(|m| matches!(m, CastMethod::Half(_))));
    assert!(t
        .cast(P0, nissa)
        .method(CastMethod::Half(1))
        .try_go()
        .is_err());
    let spell = t.cast(P0, nissa).go();
    assert_eq!(t.obj(spell).chars.name, "Nissa, Vastwood Seer");
    assert!(t.obj(spell).is_creature());
    // Compare a modal double-faced card, whose back face may be cast.
    let mut t = TestGame::new(2);
    let eddie = t.hand(P0, "Eddie Brock // Venom, Lethal Protector");
    t.lands(P0, "Swamp", 4);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let methods = legal_cast_methods(&mut t, P0, eddie);
    assert!(methods.contains(&CastMethod::Half(1)));
}

#[test]
fn loyalty_abilities_may_be_activated_the_turn_it_enters_in_a_main_phase_with_an_empty_stack() {
    cr!("606.3", "606.4");
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "You can activate one of the planeswalker's loyalty abilities the turn it enters the battlefield. However, you may do so only during one of your main phases when the stack is empty."
    );
    // She transforms during combat: no loyalty ability until a main phase.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let animist = nissa_transformed_by_a_land(&mut t, P0);
    assert!(t.activate(P0, animist, 1, &[]).is_err());
    // In the postcombat main phase of the same turn, with an empty stack: "−2: Create
    // Ashaya, the Awoken World."
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.on_battlefield(animist));
    assert!(t.activate(P0, animist, 1, &[]).is_ok());
    t.resolve_all();
    assert_eq!(t.counters(animist, counters::LOYALTY), 1);
    assert_eq!(t.named_on_battlefield("Ashaya, the Awoken World").len(), 1);
    // Not while a spell is on the stack.
    let mut t = TestGame::new(2);
    let animist = nissa_transformed_by_a_land(&mut t, P0);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(t.activate(P0, animist, 1, &[]).is_err());
    t.resolve_all();
    assert!(t.activate(P0, animist, 1, &[]).is_ok());
}

#[test]
fn you_can_control_one_front_face_up_and_one_back_face_up() {
    cr!("704.5j", "712.8d", "712.8e");
    ruling!(
        "Liliana, Heretical Healer // Liliana, Defiant Necromancer",
        "You can control two of this permanent, one front-face up and the other back-face up, at the same time."
    );
    supported(LILIANA);
    let mut t = TestGame::new(2);
    let healer = t.battlefield(P0, LILIANA);
    let necromancer = enter_transformed(&mut t, P0, LILIANA);
    t.settle();
    // Their names differ: the "legend rule" doesn't apply.
    assert!(t.on_battlefield(healer) && t.on_battlefield(necromancer));
    assert_eq!(name_of(&t, healer), "Liliana, Heretical Healer");
    assert_eq!(name_of(&t, necromancer), "Liliana, Defiant Necromancer");
    // Two with the same face up do: one is put into the graveyard.
    let another = t.battlefield(P0, LILIANA);
    t.answer_choose(P0, &[Entity::Object(healer)]);
    t.settle();
    assert!(t.on_battlefield(healer));
    assert!(!t.on_battlefield(another));
}

// ---------------------------------------------------------------------------
// Ajani, Nacatl Pariah
// ---------------------------------------------------------------------------

/// P0's Ajani on the battlefield with its Cat Warrior token; the token dies and Ajani
/// transforms (P0 chooses to). Returns (the original Ajani, Ajani, Nacatl Avenger).
fn ajani_transformed(t: &mut TestGame) -> (ObjectId, ObjectId) {
    t.lands(P0, "Plains", 2);
    let card = t.hand(P0, AJANI);
    let spell = t.cast(P0, card).go();
    t.resolve_all();
    let cat = tokens(t, P0);
    assert_eq!(cat.len(), 1);
    t.answer_yes(P0, true);
    destroy(t, cat[0]);
    t.resolve_all();
    let avenger = t.named_on_battlefield("Ajani, Nacatl Avenger");
    assert_eq!(avenger.len(), 1);
    (spell, avenger[0])
}

#[test]
fn ajani_is_cast_front_face_up_and_only_the_face_up_counts() {
    cr!("712.8a", "712.8d", "712.8e", "712.11");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "Each transforming double-faced card in this set is cast with its front face up. In every zone other than the battlefield, consider only the characteristics of its front face."
    );
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "A transforming double-faced card enters the battlefield with its front face up by default, unless a spell or ability instructs you to put it onto the battlefield transformed or allows you to cast it transformed"
    );
    supported(AJANI);
    let mut t = TestGame::new(2);
    let in_yard = t.graveyard(P1, AJANI);
    let o = t.obj(in_yard);
    assert_eq!(o.chars.name, "Ajani, Nacatl Pariah");
    assert!(o.is_creature() && o.chars.has_subtype("Cat"));
    assert_eq!(o.chars.colors, color_set(&[Color::White]));
    // Cast front face up; it enters front face up.
    t.lands(P0, "Plains", 2);
    let card = t.hand(P0, AJANI);
    let spell = t.cast(P0, card).go();
    assert_eq!(t.obj(spell).face, FaceState::Front);
    assert_eq!(t.obj(spell).chars.name, "Ajani, Nacatl Pariah");
    t.resolve_all();
    assert_eq!(face(&t, spell), FaceState::Front);
    assert!(t.obj_now(spell).is_creature());
    // Its ability returns it transformed: a planeswalker, not a Cat creature.
    t.answer_yes(P0, true);
    let cat = tokens(&t, P0)[0];
    destroy(&mut t, cat);
    t.resolve_all();
    let avenger = t.named_on_battlefield("Ajani, Nacatl Avenger")[0];
    let o = t.obj(avenger);
    assert_eq!(o.face, FaceState::Back);
    assert!(o.is(CardType::Planeswalker) && !o.is_creature());
    assert!(!o.chars.has_subtype("Cat"));
    // Put onto the battlefield by an effect (not transformed): front face up.
    let perm = put_onto_battlefield(&mut t, P1, in_yard, false).unwrap();
    assert_eq!(face(&t, perm), FaceState::Front);
    assert_eq!(name_of(&t, perm), "Ajani, Nacatl Pariah");
}

#[test]
fn a_back_face_is_the_color_of_its_indicator_but_a_colorless_land_face_has_none() {
    cr!("204.1", "105.2", "105.2c", "712.8e");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "The back face of a transforming double-faced card usually has a color indicator that defines its color. Colorless back faces, such as lands, do not."
    );
    let mut t = TestGame::new(2);
    let (_, avenger) = ajani_transformed(&mut t);
    assert_eq!(
        t.obj(avenger).chars.colors,
        color_set(&[Color::Red, Color::White])
    );
    // Search for Azcanta ({1}{U}) transforms into Azcanta, the Sunken Ruin, a colorless
    // land.
    let search = t.battlefield(P0, "Search for Azcanta // Azcanta, the Sunken Ruin");
    assert_eq!(t.obj(search).chars.colors, color_set(&[Color::Blue]));
    transform(&mut t, search);
    assert_eq!(name_of(&t, search), "Azcanta, the Sunken Ruin");
    assert!(t.obj_now(search).chars.colors.is_colorless());
}

#[test]
fn a_single_faced_copy_of_ajani_is_exiled_and_stays_in_exile() {
    cr!("712.14a", "701.27c");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "If you are instructed to put a card that isn't a double-faced card onto the battlefield transformed, it will not enter the battlefield at all. In that case, it stays in the zone it was previously in."
    );
    let mut t = TestGame::new(2);
    // P0's Grizzly Bears becomes a copy of P1's Ajani, Nacatl Pariah.
    let original = t.battlefield(P1, AJANI);
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(&mut t, bears, original);
    assert_eq!(name_of(&t, bears), "Ajani, Nacatl Pariah");
    // Told to transform, it doesn't: it isn't represented by a double-faced card.
    transform(&mut t, bears);
    assert_eq!(name_of(&t, bears), "Ajani, Nacatl Pariah");
    assert_eq!(face(&t, bears), FaceState::Front);
    // Another Cat P0 controls dies: P0 exiles the copy, which can't return transformed.
    let lions = t.battlefield(P0, "Savannah Lions");
    t.answer_yes(P0, true);
    destroy(&mut t, lions);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.named_on_battlefield("Ajani, Nacatl Avenger").is_empty());
    assert_eq!(t.zone(bears), Zone::Exile);
}
