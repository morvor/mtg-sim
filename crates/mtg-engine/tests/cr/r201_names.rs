//! CR 201: names — English names, the same and different names, interchangeable names,
//! choosing card names, and text referring to its object by name.

use crate::r105_util::matches;
use crate::r200_common::*;
use crate::r609_common::{continuous, permanent};
use crate::r703_common::run_effect;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::choices::valid_card_name;
use mtg_engine::deck::{check_constructed_with, DeckProblem, NameEquivalence};
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

/// Puts `name` from `p`'s hand onto the battlefield face down (it has no name, CR 708.2).
fn face_down(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let c = t.hand(p, name);
    run_effect(
        t,
        p,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                face_down: true,
                ..Destination::battlefield()
            },
        },
        &[Entity::Object(c)],
    );
    let id = t.g.current(c);
    assert!(t.obj_now(id).face_down && t.obj_now(id).chars.name.is_empty());
    id
}

/// Advances to the beginning of P0's next upkeep, with its triggers on the stack.
fn next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

// ---------------------------------------------------------------------------
// 201.2: English names; the same name, different names
// ---------------------------------------------------------------------------

#[test]
fn a_cards_name_is_its_english_name() {
    cr!("201.2");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert_eq!(t.obj_now(bolt).chars.name, "Lightning Bolt");
    // "Blitzschlag" is printed on German Lightning Bolts, but the card's name is always
    // its English name: that isn't a card name at all, so nothing is named.
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Blitzschlag");
    assert_eq!(chosen_name(&t, m), "");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    assert!(can_cast_face(&mut t, P1, bolt, FaceState::Front));
    // Naming the English name stops it.
    enter_naming(&mut t, P0, "Meddling Mage", "Lightning Bolt");
    assert!(!can_cast_face(&mut t, P1, bolt, FaceState::Front));
}

#[test]
fn a_group_has_different_names_only_if_each_has_a_name() {
    // CR 201.2b and its example: three Demons with different names and a face-down
    // creature made a Demon aren't four Demons with different names.
    cr!("201.2b");
    ruling!(
        "Liliana's Contract",
        "If you don’t control four Demons with different names as your upkeep begins"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana's Contract");
    // Xenograft naming Demon: each creature you control is a Demon.
    let demon = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Demon")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(demon));
    t.enter(P0, "Xenograft");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Llanowar Elves");
    let fd = face_down(&mut t, P0, "Grinning Demon");
    t.g.recompute();
    assert!(t.obj_now(fd).chars.has_subtype("Demon"));
    next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 0, "the ability doesn't trigger");
    assert!(!t.has_lost(P1));
    // A second Grizzly Bears doesn't have a different name either.
    t.battlefield(P0, "Grizzly Bears");
    next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 0, "the ability doesn't trigger");
    assert!(!t.has_lost(P1));
    // A fourth name makes four Demons with different names.
    t.battlefield(P0, "Craw Wurm");
    next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn differently_named_objects_are_counted_once_per_name() {
    cr!("201.2b");
    ruling!(
        "Awakened Amalgam",
        "count each land you control once, but only if its English name isn’t exactly the same"
    );
    ruling!(
        "Field of the Dead",
        "If you control multiple lands with the same name, only one of those lands will count"
    );
    let mut t = TestGame::new(2);
    let amalgam = t.battlefield(P0, "Awakened Amalgam");
    t.lands(P0, "Plains", 4);
    t.lands(P0, "Island", 2);
    t.battlefield(P0, "Drowned Catacomb");
    // Opponents' lands aren't counted.
    t.battlefield(P1, "Forest");
    t.g.recompute();
    // Plains, Island, Drowned Catacomb.
    assert_eq!(t.pt(amalgam), (3, 3));
    let v = Value::DistinctNames(Filter::Type(CardType::Land).you_control());
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert_eq!(t.g.eval_value(&v, &ctx), 3);
    // An object with two names (a split card) has a name in common with an object with
    // either name: Fire // Ice and Ice aren't differently named; Fire // Ice and Shock
    // are.
    let mut t = TestGame::new(2);
    let fi = t.graveyard(P0, "Fire // Ice");
    let ice = t.custom(
        P0,
        plain_card("Ice", "Instant", "{1}{U}", None),
        Zone::Graveyard(P0),
    );
    let shock = t.graveyard(P0, "Shock");
    let chars = |ids: &[ObjectId]| -> Vec<mtg_engine::object::Characteristics> {
        ids.iter().map(|i| t.obj_now(*i).chars.clone()).collect()
    };
    assert_eq!(
        mtg_engine::names::distinct_name_count(&chars(&[fi, ice])),
        1
    );
    assert_eq!(
        mtg_engine::names::distinct_name_count(&chars(&[fi, shock])),
        2
    );
}

#[test]
fn counters_for_each_differently_named_land_go_on_the_created_token() {
    // Emil, Vastlands Roamer: "{4}{G}, {T}: Create a 0/0 green and blue Fractal creature
    // token. Put X +1/+1 counters on it, where X is the number of differently named lands
    // you control." Three Forests, an Island and a Plains are three differently named
    // lands; the counters go on the token ("it"), not on Emil.
    cr!("201.2b");
    crate::r703_common::supported("Emil, Vastlands Roamer");
    let mut t = TestGame::new(2);
    let emil = t.battlefield(P0, "Emil, Vastlands Roamer");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Plains", 1);
    t.battlefield(P1, "Mountain");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, emil, 0, &[]).unwrap();
    t.resolve_all();
    let fractals = t.named_on_battlefield("Fractal Token");
    assert_eq!(fractals.len(), 1);
    assert_eq!(t.counters(fractals[0], "+1/+1"), 3);
    assert_eq!(t.pt(fractals[0]), (3, 3));
    assert_eq!(t.counters(emil, "+1/+1"), 0);
}

#[test]
fn a_different_name_than_others_requires_a_name() {
    // CR 201.2c: the first object has a different name than the others if it has at
    // least one name and no name in common with any of them, even if some of them have
    // no name; an object with no name doesn't have a different name than anything.
    cr!("201.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bears2 = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let fd = face_down(&mut t, P1, "Grinning Demon");
    let diff = |sel: Sel| {
        Filter::and(vec![
            Filter::creature(),
            Filter::DifferentNameFrom(Box::new(sel)),
        ])
    };
    let targets = vec![Entity::Object(bears)];
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![targets];
    let f = diff(Sel::Target(0));
    // Compared with Grizzly Bears: Hill Giant differs; the other Bears doesn't; the
    // face-down creature (no name) doesn't have a different name.
    assert!(t.g.matches(giant, &f, &ctx));
    assert!(!t.g.matches(bears2, &f, &ctx));
    assert!(!t.g.matches(fd, &f, &ctx));
    // Compared with the face-down creature: every named creature has a different name.
    ctx.targets = vec![vec![Entity::Object(fd)]];
    assert!(t.g.matches(bears, &f, &ctx));
    assert!(t.g.matches(giant, &f, &ctx));
    assert!(!t.g.matches(fd, &f, &ctx));
    // Destroying "each creature with a different name than target creature" (the Bears).
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Destroy {
            what: Sel::All(diff(Sel::Target(0))),
            no_regen: false,
        },
        &[Entity::Object(bears)],
    );
    t.settle();
    assert!(!t.g.is_live(giant));
    assert!(t.g.is_live(bears) && t.g.is_live(bears2) && t.g.is_live(fd));
}

// ---------------------------------------------------------------------------
// 201.3: interchangeable names
// ---------------------------------------------------------------------------

/// Custom versions of two real cards whose names are interchangeable (CR 201.3).
fn interchangeable_pair() -> (CardDef, CardDef) {
    (
        with_interchangeable_names("Grizzly Bears", &["Runeclaw Bear"]),
        with_interchangeable_names("Runeclaw Bear", &["Grizzly Bears"]),
    )
}

use mtg_engine::card::CardDef;

#[test]
fn interchangeable_names_are_the_same_name_for_rules_and_effects() {
    cr!("201.3", "201.3a");
    let mut t = TestGame::new(2);
    let (a, b) = interchangeable_pair();
    let bears = t.custom(P0, a, Zone::Battlefield);
    let rune = t.custom(P1, b, Zone::Battlefield);
    let plain = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    // Effects referring to names: "named Runeclaw Bear" matches the Grizzly Bears.
    assert!(matches(
        &t,
        bears,
        &Filter::Named("Runeclaw Bear".into()),
        P0
    ));
    assert!(matches(
        &t,
        rune,
        &Filter::Named("Grizzly Bears".into()),
        P0
    ));
    assert!(t
        .obj_now(bears)
        .chars
        .shares_name_with(&t.obj_now(rune).chars));
    // Their own names remain: "named Grizzly Bears" still matches it too.
    assert!(matches(
        &t,
        bears,
        &Filter::Named("Grizzly Bears".into()),
        P0
    ));
    assert!(t
        .obj_now(plain)
        .chars
        .shares_name_with(&t.obj_now(rune).chars));
    // Rules referring to names: two legendary permanents with interchangeable names are
    // subject to the legend rule (CR 704.5j).
    let mut ta = TestGame::new(2);
    let isamaru =
        with_interchangeable_names("Isamaru, Hound of Konda", &["Ragavan, Nimble Pilferer"]);
    let x = ta.custom(P0, isamaru, Zone::Battlefield);
    let y = ta.battlefield(P0, "Ragavan, Nimble Pilferer");
    ta.settle();
    let alive = [x, y].iter().filter(|i| ta.g.is_live(**i)).count();
    assert_eq!(alive, 1, "the legend rule applies to the pair");
    // Without the interchangeable names, both stay.
    let mut tb = TestGame::new(2);
    let x = tb.battlefield(P0, "Isamaru, Hound of Konda");
    let y = tb.battlefield(P0, "Ragavan, Nimble Pilferer");
    tb.settle();
    assert!(tb.g.is_live(x) && tb.g.is_live(y));
}

#[test]
fn interchangeable_names_are_the_same_name_for_deck_construction() {
    cr!("201.3b");
    let (a, b) = interchangeable_pair();
    let (a, b) = (Arc::new(a), Arc::new(b));
    let filler = || card("Forest");
    // Three Grizzly Bears and two Runeclaw Bears are five cards with the same name.
    let mut deck: Vec<_> = (0..55).map(|_| filler()).collect();
    deck.extend((0..3).map(|_| a.clone()));
    deck.extend((0..2).map(|_| b.clone()));
    let problems = check_constructed_with(&deck, &[], &NameEquivalence::default());
    assert!(
        problems.iter().any(|p| matches!(
            p,
            DeckProblem::TooManyCopies {
                have: 5,
                max: 4,
                ..
            }
        )),
        "{problems:?}"
    );
    // Four in all is fine.
    let mut deck: Vec<_> = (0..56).map(|_| filler()).collect();
    deck.extend((0..2).map(|_| a.clone()));
    deck.extend((0..2).map(|_| b.clone()));
    assert!(check_constructed_with(&deck, &[], &NameEquivalence::default()).is_empty());
    // The same holds for real cards declared interchangeable by the format.
    let names = NameEquivalence(vec![("Grizzly Bears".into(), "Runeclaw Bear".into())]);
    let mut deck: Vec<_> = (0..55).map(|_| filler()).collect();
    deck.extend((0..3).map(|_| card("Grizzly Bears")));
    deck.extend((0..2).map(|_| card("Runeclaw Bear")));
    assert!(!check_constructed_with(&deck, &[], &names).is_empty());
    assert!(check_constructed_with(&deck, &[], &NameEquivalence::default()).is_empty());
}

#[test]
fn choosing_an_interchangeable_name_chooses_each_of_them() {
    cr!("201.4g");
    let mut t = TestGame::new(2);
    let (a, _) = interchangeable_pair();
    let bears = t.custom(P1, a, Zone::Hand(P1));
    let rune = t.hand(P1, "Runeclaw Bear");
    enter_naming(&mut t, P0, "Meddling Mage", "Runeclaw Bear");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    // The Grizzly Bears whose name is interchangeable with Runeclaw Bear has the chosen
    // name too.
    assert!(!can_cast_face(&mut t, P1, rune, FaceState::Front));
    assert!(!can_cast_face(&mut t, P1, bears, FaceState::Front));
    // An ordinary Grizzly Bears doesn't.
    let plain = t.hand(P1, "Grizzly Bears");
    assert!(can_cast_face(&mut t, P1, plain, FaceState::Front));
}

// ---------------------------------------------------------------------------
// 201.4: choosing card names
// ---------------------------------------------------------------------------

#[test]
fn a_split_card_name_is_one_of_its_halves() {
    // CR 201.4b: the name of one half, not both; with characteristics, that half's.
    cr!("201.4b");
    let mut t = TestGame::new(2);
    let fire_ice = t.hand(P1, "Fire // Ice");
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Fire // Ice");
    assert_eq!(chosen_name(&t, m), "", "both names at once isn't a name");
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Fire");
    assert_eq!(chosen_name(&t, m), "Fire");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Volcanic Island", 3);
    // Fire can't be cast, Ice can.
    assert!(!can_cast_face(&mut t, P1, fire_ice, FaceState::Half(0)));
    assert!(can_cast_face(&mut t, P1, fire_ice, FaceState::Half(1)));
    // The card has the chosen name in every zone other than the stack (CR 709.4a).
    let gy = t.graveyard(P1, "Fire // Ice");
    assert!(matches(&t, gy, &Filter::Named("Ice".into()), P0));
    // Only that half's characteristics count: Fire and Ice are instants, Fire is red and
    // Ice is blue — the combined card is neither a creature nor a land.
    assert!(valid_card_name("Ice", Some("nonland")));
    assert!(!valid_card_name("Ice", Some("creature")));
    // Wear // Tear: each half is an instant; "Tear" is a legal instant card name.
    assert!(valid_card_name("Tear", Some("instant")));
    assert!(!valid_card_name("Wear // Tear", Some("instant")));
}

#[test]
fn a_flip_cards_alternative_name_may_be_chosen() {
    // CR 201.4c: Erayo, Soratami Ascendant flips into Erayo's Essence, a legendary
    // enchantment. Its alternative name may be chosen; with characteristics, those of
    // the card as modified by its alternative characteristics are used.
    cr!("201.4c");
    let mut t = TestGame::new(2);
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Erayo's Essence");
    assert_eq!(chosen_name(&t, m), "Erayo's Essence");
    assert!(valid_card_name("Erayo's Essence", Some("enchantment")));
    assert!(!valid_card_name("Erayo's Essence", Some("creature")));
    assert!(valid_card_name(
        "Erayo, Soratami Ascendant",
        Some("creature")
    ));
    assert!(!valid_card_name(
        "Erayo, Soratami Ascendant",
        Some("enchantment")
    ));
}

#[test]
fn the_back_face_name_of_a_double_faced_card_may_be_chosen() {
    // CR 201.4d: naming Lord of Lineage (Bloodline Keeper's back face) with Pithing
    // Needle stops the transformed permanent's activated abilities, not the front face's.
    cr!("201.4d");
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P1, "Bloodline Keeper");
    let needle = enter_naming(&mut t, P0, "Pithing Needle", "Lord of Lineage");
    assert_eq!(chosen_name(&t, needle), "Lord of Lineage");
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    assert!(
        t.activate(P1, keeper, 0, &[]).is_ok(),
        "the front face isn't named"
    );
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    assert!(mtg_engine::dfc::transform(&mut t.g, keeper));
    t.g.recompute();
    t.g.objects[keeper.0 as usize].tapped = false;
    assert_eq!(t.obj_now(keeper).chars.name, "Lord of Lineage");
    assert!(t.activate(P1, keeper, 0, &[]).is_err());
    // With characteristics, only the back face's count: Kazandu Mammoth's back face,
    // Kazandu Valley, is a land, so it isn't a nonland card name.
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Kazandu Valley");
    assert_eq!(chosen_name(&t, m), "");
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Kazandu Mammoth");
    assert_eq!(chosen_name(&t, m), "Kazandu Mammoth");
}

#[test]
fn the_combined_back_face_name_of_a_meld_pair_may_be_chosen() {
    // CR 201.4e: Hanweir, the Writhing Township is the combined back face of Hanweir
    // Battlements (a land) and Hanweir Garrison. With characteristics, only the combined
    // back face's are used: it's a nonland creature card name, not a land card name.
    cr!("201.4e");
    let mut t = TestGame::new(2);
    let m = enter_naming(
        &mut t,
        P0,
        "Meddling Mage",
        "Hanweir, the Writhing Township",
    );
    assert_eq!(chosen_name(&t, m), "Hanweir, the Writhing Township");
    assert!(valid_card_name(
        "Hanweir, the Writhing Township",
        Some("creature")
    ));
    assert!(!valid_card_name(
        "Hanweir, the Writhing Township",
        Some("land")
    ));
    // The front face Hanweir Battlements is a land.
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Hanweir Battlements");
    assert_eq!(chosen_name(&t, m), "");
}

#[test]
fn an_adventurers_alternative_name_may_be_chosen() {
    cr!("201.4f");
    ruling!(
        "Brazen Borrower",
        "If an effect instructs you to choose a card name, you may choose the alternative Adventure name"
    );
    let mut t = TestGame::new(2);
    let borrower = t.hand(P1, "Brazen Borrower");
    t.battlefield(P0, "Grizzly Bears");
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Petty Theft");
    assert_eq!(chosen_name(&t, m), "Petty Theft");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 3);
    // Petty Theft (the Adventure, half 1) can't be cast; Brazen Borrower can.
    assert!(!can_cast_face(&mut t, P1, borrower, FaceState::Half(1)));
    assert!(can_cast_face(&mut t, P1, borrower, FaceState::Front));
    // Only the alternative characteristics count: Petty Theft is an instant.
    assert!(valid_card_name("Petty Theft", Some("instant")));
    assert!(!valid_card_name("Petty Theft", Some("creature")));
}

// ---------------------------------------------------------------------------
// 201.5: text referring to its object by name
// ---------------------------------------------------------------------------

#[test]
fn text_naming_its_object_means_only_that_object() {
    // CR 201.5: Skithiryx's "{B}{B}: Regenerate Skithiryx" regenerates only that
    // Skithiryx, not another object named Skithiryx — even after its name changes.
    cr!("201.5");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Skithiryx, the Blight Dragon");
    let theirs = t.battlefield(P1, "Skithiryx, the Blight Dragon");
    // Its name becomes "Nameless Dragon".
    let renamer = permanent(
        "Renamer",
        &[CardType::Enchantment],
        vec![continuous(
            Filter::creature().you_control(),
            vec![Modification::SetName("Nameless Dragon".into())],
        )],
    );
    t.custom(P0, renamer, Zone::Battlefield);
    t.g.recompute();
    assert_eq!(t.obj_now(mine).chars.name, "Nameless Dragon");
    t.lands(P0, "Swamp", 2);
    let regen = t
        .obj_now(mine)
        .chars
        .abilities
        .iter()
        .position(|a| a.text.contains("Regenerate"))
        .expect("regenerate ability");
    let n = t.obj_now(mine).chars.abilities[..regen]
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count();
    t.activate(P0, mine, n, &[]).unwrap();
    t.resolve_all();
    // Both are destroyed: only the renamed one regenerates.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Destroy {
            what: Sel::All(Filter::creature()),
            no_regen: false,
        },
        &[],
    );
    t.settle();
    assert!(t.g.is_live(mine), "it regenerated");
    assert!(t.obj_now(mine).tapped);
    assert!(!t.g.is_live(theirs));
}

#[test]
fn a_gained_ability_naming_its_first_source_refers_to_the_new_object() {
    // CR 201.5b and its example: Glacial Ray ("Glacial Ray deals 2 damage to any
    // target") spliced onto another Arcane spell: that spell deals the damage.
    cr!("201.5b");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let spike = t.hand(P0, "Lava Spike");
    let ray = t.hand(P0, "Glacial Ray");
    let spell = t.cast(P0, spike).kicked(true).target(P1).target(bear).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let dealt: Vec<(ObjectId, ObjectId)> = t.g.history.damage_by_source.iter().copied().collect();
    assert!(
        dealt.iter().any(|(s, d)| *s == spell && *d == bear),
        "{dealt:?}"
    );
    assert!(!dealt
        .iter()
        .any(|(s, _)| *s == ray || t.g.current(*s) == t.g.current(ray)));
}

// ---------------------------------------------------------------------------
// 201.6: alternate names
// ---------------------------------------------------------------------------

#[test]
fn an_alternate_name_is_not_the_cards_name() {
    // CR 201.6: the Oracle name (in the secondary title bar) is the card's only name;
    // the alternate name in the upper left corner has no effect on game play.
    cr!("201.6");
    let def = card("Kibo, Uktabi Prince");
    assert_eq!(
        def.alternate_name.as_deref(),
        Some("Monkey, Awakened to Emptiness")
    );
    let mut t = TestGame::new(2);
    let kibo = t.hand(P1, "Kibo, Uktabi Prince");
    assert_eq!(t.obj_now(kibo).chars.name, "Kibo, Uktabi Prince");
    assert!(!matches(
        &t,
        kibo,
        &Filter::Named("Monkey, Awakened to Emptiness".into()),
        P0
    ));
    // The alternate name can't be chosen as a card name; the Oracle name can.
    let m = enter_naming(&mut t, P0, "Meddling Mage", "Monkey, Awakened to Emptiness");
    assert_eq!(chosen_name(&t, m), "");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 3);
    assert!(can_cast_face(&mut t, P1, kibo, FaceState::Front));
    enter_naming(&mut t, P0, "Meddling Mage", "Kibo, Uktabi Prince");
    assert!(!can_cast_face(&mut t, P1, kibo, FaceState::Front));
}
