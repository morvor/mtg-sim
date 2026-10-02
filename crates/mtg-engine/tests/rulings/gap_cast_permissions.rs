//! Permissions to play cards from other zones (gap-cast-permissions): "you may cast that
//! card" doesn't let a land be played (CR 305.9), a permission that requires an
//! alternative cost ("pay life equal to its mana value rather than pay its mana cost",
//! CR 118.9a–b), the player choosing which permission they use as they play a card
//! (CR 601.2, 601.3, 305.1) — including Muldrotha's one-of-each-permanent-type
//! permissions (CR 110.4, 601.3e) — and foretold modal double-faced cards cast as either
//! face (CR 702.143d, 712.11b).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::casting::PERMISSION_COST;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FORETELL: CastMethod = CastMethod::Keyword(KeywordKind::Foretell);
const PAY_LIFE: CastMethod = CastMethod::Alternative(PERMISSION_COST);

/// Whether `p` may play `card` as a land now.
fn can_play_land(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let card = t.g.current(card);
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p).contains(&Action::PlayLand { card })
}

// ---------------------------------------------------------------------------
// Foretold modal double-faced cards (Ethereal Valkyrie)
// ---------------------------------------------------------------------------

/// Ethereal Valkyrie enters for P0, who exiles `card` from their hand face down: it
/// becomes foretold with a foretell cost equal to its mana cost reduced by {2}. Returns
/// the exiled card.
fn valkyrie_foretells(t: &mut TestGame, card: ObjectId) -> ObjectId {
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.enter(P0, "Ethereal Valkyrie");
    t.resolve_all();
    let exiled = t.g.current(card);
    assert_eq!(t.obj(exiled).zone, Zone::Exile);
    assert!(t.obj(exiled).face_down);
    exiled
}

#[test]
fn ethereal_valkyrie_foretells_an_mdfc_cast_as_either_face_for_that_faces_cost() {
    cr!("702.143d", "712.11b", "601.2f");
    ruling!(
        "Ethereal Valkyrie",
        "If you foretell a modal double-faced card, the foretell cost will be based on the mana cost of the face you cast from exile. For example, if you foretell Kolvori, God of Kinship/The Ringhart Crest, you could cast Kolvori by paying {G}{G} or The Ringhart Crest by paying {G} on a future turn."
    );
    supported("Ethereal Valkyrie");
    for crest in [false, true] {
        let mut t = TestGame::new(2);
        let kolvori = t.hand(P0, "Kolvori, God of Kinship // The Ringhart Crest");
        let card = valkyrie_foretells(&mut t, kolvori);
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        // Kolvori ({2}{G}{G}) for {G}{G}; The Ringhart Crest ({1}{G}) for {G}.
        add_mana(&mut t, P0, ManaType::G, if crest { 1 } else { 2 });
        if crest {
            // One green mana: only the Crest's foretell cost can be paid.
            t.answer(P0, DecisionKind::Option, Answer::Index(1));
        }
        assert_eq!(legal_cast_methods(&mut t, P0, card), vec![FORETELL]);
        let spell = t.cast(P0, card).method(FORETELL).go();
        let name = if crest {
            "The Ringhart Crest"
        } else {
            "Kolvori, God of Kinship"
        };
        assert_eq!(t.obj(spell).chars.name, name);
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "paid all of it");
        t.resolve_all();
        assert_eq!(t.named_on_battlefield(name).len(), 1);
    }
}

#[test]
fn ethereal_valkyrie_a_foretold_mdfc_with_a_land_back_cant_be_played_as_a_land() {
    cr!("702.143d", "305.9", "601.3");
    ruling!(
        "Ethereal Valkyrie",
        "If you foretell a modal double-faced card whose front face is a nonland face but whose back face is a land face, you can't play that card as a land."
    );
    supported("Ethereal Valkyrie");
    let mut t = TestGame::new(2);
    let mammoth = t.hand(P0, "Kazandu Mammoth // Kazandu Valley");
    let card = valkyrie_foretells(&mut t, mammoth);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, card));
    assert!(t.play_land(P0, card).is_err());
    // Kazandu Mammoth ({1}{G}{G}) for {G}{G}: the front face only.
    add_mana(&mut t, P0, ManaType::G, 2);
    assert_eq!(legal_cast_methods(&mut t, P0, card), vec![FORETELL]);
    let spell = t.cast(P0, card).method(FORETELL).go();
    assert_eq!(t.obj(spell).chars.name, "Kazandu Mammoth");
}

#[test]
fn ethereal_valkyrie_a_card_with_foretell_has_both_foretell_costs() {
    cr!("702.143a", "702.143d", "118.9a");
    ruling!(
        "Ethereal Valkyrie",
        "If the card you exile has foretell, it will have two foretell costs, the one printed on it and the one given to it by Ethereal Valkyrie. You may cast that card for either of its foretell costs."
    );
    supported("Ethereal Valkyrie");
    supported("Battle Mammoth");
    // Battle Mammoth ({3}{G}{G}, foretell {2}{G}{G}): its printed foretell cost and
    // {1}{G}{G} ({3}{G}{G} reduced by {2}).
    let foretold = |t: &mut TestGame| {
        let mammoth = t.hand(P0, "Battle Mammoth");
        let card = valkyrie_foretells(t, mammoth);
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        card
    };
    // {1}{G}{G}: only the cost Ethereal Valkyrie gave it can be paid.
    let mut t = TestGame::new(2);
    let card = foretold(&mut t);
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    assert_eq!(legal_cast_methods(&mut t, P0, card), vec![FORETELL]);
    t.cast(P0, card).method(FORETELL).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Battle Mammoth").len(), 1);
    // With {2}{G}{G}, either: the player chooses one of the two (the printed one here).
    let mut t = TestGame::new(2);
    let card = foretold(&mut t);
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 2);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, card).method(FORETELL).go();
    let offered = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseCastingMethod { options, .. } => Some(options.len()),
        _ => None,
    });
    assert_eq!(offered, Some(2), "two foretell costs");
    assert_eq!(t.g.player(P0).mana_pool.total(), 0, "paid {{2}}{{G}}{{G}}");
    // The other one leaves {C}.
    let mut t = TestGame::new(2);
    let card = foretold(&mut t);
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, card).method(FORETELL).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1, "paid {{1}}{{G}}{{G}}");
}

#[test]
fn ethereal_valkyrie_the_card_becomes_foretold_but_wasnt_foretold() {
    cr!("702.143c", "702.143d");
    ruling!(
        "Ethereal Valkyrie",
        "Although the card you exile becomes foretold, you did not foretell it. An ability that triggers whenever you foretell a card will not trigger."
    );
    ruling!(
        "Ethereal Valkyrie",
        "Because the card in exile becomes foretold, you can look at it in exile. (Most effects that have you exile a card face down don't allow you to look at the card in exile.)"
    );
    supported("Ethereal Valkyrie");
    // Dream Devourer's "Whenever you foretell a card, this creature gets +2/+0 until end
    // of turn."
    let def = custom_card(
        "Foretelling Devourer",
        "Creature — Demon Cleric",
        "{1}{B}",
        Some((0, 3)),
        "Whenever you foretell a card, ~ gets +2/+0 until end of turn.",
    );
    let mut t = TestGame::new(2);
    let devourer = t.custom(P0, def, Zone::Battlefield);
    let giant = t.hand(P0, "Hill Giant");
    let card = valkyrie_foretells(&mut t, giant);
    assert_eq!(t.pt(devourer), (0, 3), "no foretell trigger");
    // Its owner may look at it; other players may not.
    assert!(mtg_engine::zones::may_look(&t.g, P0, card));
    assert!(!mtg_engine::zones::may_look(&t.g, P1, card));
    // Foretelling a card does trigger it.
    let bolt = t.hand(P0, "Demon Bolt");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        Action::Special(mtg_engine::decision::SpecialAction::Foretell { card: bolt }),
    )
    .expect("foretell");
    t.resolve_all();
    assert_eq!(t.pt(devourer), (2, 3));
}

// ---------------------------------------------------------------------------
// "You may cast that card": not a permission to play a land (Ragavan)
// ---------------------------------------------------------------------------

/// Ragavan deals combat damage to P1, whose library's top card is `top`: it's exiled.
/// Returns the exiled card (combat is over: the end of combat step).
fn ragavan_hits(t: &mut TestGame, top: &str) -> ObjectId {
    let ragavan = t.battlefield(P0, "Ragavan, Nimble Pilferer");
    let card = t.library_top(P1, top);
    t.attack(&[(ragavan, Entity::Player(P1))], &[]);
    let exiled = t.g.current(card);
    assert_eq!(t.obj(exiled).zone, Zone::Exile, "{top} exiled");
    exiled
}

#[test]
fn ragavan_the_exiled_card_is_cast_normally_and_a_land_cant_be_played() {
    cr!("601.3", "305.9", "307.1", "601.2f");
    ruling!(
        "Ragavan, Nimble Pilferer",
        "You must still follow all timing restrictions and pay all costs when casting the exiled card. If you exile a land card, you can't play that card."
    );
    supported("Ragavan, Nimble Pilferer");
    // A land card: "you may cast that card" doesn't let P0 play it.
    let mut t = TestGame::new(2);
    let forest = ragavan_hits(&mut t, "Forest");
    t.advance_to_step(Step::PostcombatMain);
    assert!(!can_play_land(&mut t, P0, forest));
    assert!(t.play_land(P0, forest).is_err());
    // A modal double-faced card: its creature face may be cast, its land face not played.
    let mut t = TestGame::new(2);
    let mammoth = ragavan_hits(&mut t, "Kazandu Mammoth // Kazandu Valley");
    t.lands(P0, "Forest", 3);
    // A creature spell: not during combat (CR 307.1), and its mana cost is paid.
    assert!(legal_cast_methods(&mut t, P0, mammoth).is_empty());
    t.advance_to_step(Step::PostcombatMain);
    assert!(!can_play_land(&mut t, P0, mammoth));
    assert!(t.play_land(P0, mammoth).is_err());
    assert_eq!(
        legal_cast_methods(&mut t, P0, mammoth),
        vec![CastMethod::Normal]
    );
    let spell = t.cast(P0, mammoth).go();
    assert_eq!(t.obj(spell).chars.name, "Kazandu Mammoth");
    assert_eq!(tapped_lands(&t, P0), 3);
    // Until end of turn: an exiled card still there next turn can't be cast.
    let mut t = TestGame::new(2);
    let bolt = ragavan_hits(&mut t, "Lightning Bolt");
    t.advance_to(P1, Step::Upkeep);
    add_mana(&mut t, P0, ManaType::R, 1);
    t.g.turn.priority = Some(P0);
    assert!(legal_cast_methods(&mut t, P0, bolt).is_empty());
}

// ---------------------------------------------------------------------------
// A permission that requires an alternative cost (Inside Information, Xander's Pact,
// Bolas's Citadel)
// ---------------------------------------------------------------------------

/// P0 casts Inside Information for X = `top_first.len()` targeting P1, whose library's
/// top cards are `top_first`: they're exiled.
fn inside_information(t: &mut TestGame, top_first: &[&str]) -> Vec<ObjectId> {
    let cards = stack_library(t, P1, top_first);
    let ii = t.hand(P0, "Inside Information");
    add_mana(t, P0, ManaType::B, 2 + top_first.len() as u32);
    t.cast(P0, ii)
        .x(top_first.len() as i64)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    cards
        .into_iter()
        .map(|c| {
            let c = t.g.current(c);
            assert_eq!(t.obj(c).zone, Zone::Exile);
            c
        })
        .collect()
}

#[test]
fn inside_information_spells_are_cast_for_life_with_additional_costs_but_no_other_alternative_cost()
{
    cr!("118.9a", "118.9b", "601.2b", "601.2f");
    ruling!(
        "Inside Information",
        "If you cast a spell for another cost \"rather than pay its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If a spell has any mandatory additional costs, such as that of Stir Up Trouble, those must be paid for it."
    );
    supported("Inside Information");
    let mut t = TestGame::new(2);
    let exiled = inside_information(&mut t, &["Mulldrifter", "Burst Lightning"]);
    let (mulldrifter, burst) = (exiled[0], exiled[1]);
    t.lands(P0, "Island", 5);
    // Mulldrifter ({4}{U}, evoke {2}{U}): only for 5 life — not its mana cost, nor evoke.
    assert_eq!(legal_cast_methods(&mut t, P0, mulldrifter), vec![PAY_LIFE]);
    assert!(t
        .cast(P0, mulldrifter)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .try_go()
        .is_err());
    t.cast(P0, mulldrifter).method(PAY_LIFE).go();
    assert_eq!(t.life(P0), 15);
    assert_eq!(tapped_lands(&t, P0), 0, "no mana paid");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
    // Burst Lightning ({R}, kicker {4}): 1 life, and the kicker may be paid too.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, burst)
        .method(PAY_LIFE)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.life(P0), 14);
    assert_eq!(tapped_lands(&t, P0), 4, "the kicker cost {{4}}");
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "kicked: 4 damage");
    // Stir Up Trouble ({B}; as an additional cost, sacrifice an artifact or creature or
    // pay {4}): its additional cost must be paid as well.
    let mut t2 = TestGame::new(2);
    let exiled = inside_information(&mut t2, &["Stir Up Trouble"]);
    let bears = t2.battlefield(P1, "Grizzly Bears");
    t2.g.turn.priority = Some(P0);
    // Nothing to sacrifice and no {4}: it can't be cast (the attempt is undone, CR 733).
    assert!(t2
        .cast(P0, exiled[0])
        .method(PAY_LIFE)
        .target(bears)
        .try_go()
        .is_err());
    assert_eq!(t2.life(P0), 20);
    assert_eq!(t2.obj(exiled[0]).zone, Zone::Exile);
    t2.lands(P0, "Swamp", 4);
    assert_eq!(legal_cast_methods(&mut t2, P0, exiled[0]), vec![PAY_LIFE]);
    t2.cast(P0, exiled[0]).method(PAY_LIFE).target(bears).go();
    assert_eq!(t2.life(P0), 19);
    assert_eq!(tapped_lands(&t2, P0), 4);
}

#[test]
fn inside_information_lands_follow_the_land_rules() {
    cr!("305.1", "305.2", "305.2b", "601.3");
    ruling!(
        "Inside Information",
        "You must follow all normal timing rules when playing a land or casting a spell for cards exiled with Inside Information. For example, if an exiled card is a land card, you may play it only during your main phase while the stack is empty and only if you have an available land play remaining."
    );
    supported("Inside Information");
    let mut t = TestGame::new(2);
    let exiled = inside_information(&mut t, &["Forest", "Mountain", "Lightning Bolt"]);
    let (forest, mountain, bolt) = (exiled[0], exiled[1], exiled[2]);
    // Not while the stack isn't empty.
    let shock = t.hand(P0, "Shock");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    // In the main phase with the stack empty and a land play left: one of them.
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).expect("play the Forest");
    assert!(!can_play_land(&mut t, P0, mountain), "no land play left");
    // Not during combat (P0 has a land play the next turn, but not then).
    // Lightning Bolt, an instant, may be cast any time P0 could cast an instant, for 1 life.
    t.advance_to_step(Step::BeginningOfCombat);
    t.g.turn.priority = Some(P0);
    assert_eq!(legal_cast_methods(&mut t, P0, bolt), vec![PAY_LIFE]);
    t.cast(P0, bolt)
        .method(PAY_LIFE)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn xanders_pact_spells_cast_for_life_no_blitz_but_casualty() {
    cr!("118.9a", "118.9b", "702.152a", "702.153a");
    ruling!(
        "Xander's Pact",
        "If you cast a spell for another cost “rather than pay its mana cost,” you can't choose to cast it for any alternative costs such as a blitz cost. You can, however, pay additional costs, such as a casualty cost. If the spell has any mandatory additional costs, those must be paid to cast it."
    );
    supported("Xander's Pact");
    supported("Riveteers Requisitioner");
    supported("Join the Maestros");
    let mut t = TestGame::new(3);
    t.library_top(P1, "Riveteers Requisitioner");
    t.library_top(P2, "Join the Maestros");
    let pact = t.hand(P0, "Xander's Pact");
    add_mana(&mut t, P0, ManaType::B, 6);
    t.cast(P0, pact).go();
    t.resolve_all();
    // Each opponent exiled their top card.
    let req =
        t.g.exile
            .iter()
            .copied()
            .find(|c| t.obj(*c).chars.name == "Riveteers Requisitioner");
    let join =
        t.g.exile
            .iter()
            .copied()
            .find(|c| t.obj(*c).chars.name == "Join the Maestros");
    let (req, join) = (req.expect("exiled"), join.expect("exiled"));
    t.lands(P0, "Mountain", 3);
    // Riveteers Requisitioner ({1}{R}, blitz {2}{R}): only for 2 life, not blitzed.
    assert_eq!(legal_cast_methods(&mut t, P0, req), vec![PAY_LIFE]);
    t.cast(P0, req).method(PAY_LIFE).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(tapped_lands(&t, P0), 0);
    t.resolve_all();
    // Join the Maestros ({4}{B}, casualty 2): 5 life, and its casualty cost may be paid
    // (sacrificing Hill Giant, power 3): it's copied.
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, join).method(PAY_LIFE).go();
    assert_eq!(t.life(P0), 13);
    assert!(t.in_graveyard(P0, "Hill Giant"), "sacrificed for casualty");
    t.resolve_all();
    assert_eq!(
        creatures(&t, P0).len(),
        3,
        "the Requisitioner and two Rogue tokens"
    );
}

/// Bolas's Citadel on P0's battlefield and `top` on top of their library.
fn citadel_with(t: &mut TestGame, top: &str) -> ObjectId {
    t.battlefield(P0, "Bolas's Citadel");
    t.library_top(P0, top)
}

#[test]
fn bolas_citadel_follows_the_normal_timing() {
    cr!("307.1", "601.3");
    ruling!(
        "Bolas's Citadel",
        "You must follow the normal timing permissions and restrictions of the cards you play from your library."
    );
    supported("Bolas's Citadel");
    // A sorcery from the top, paying life: only at sorcery speed.
    let mut t = TestGame::new(2);
    let harvest = citadel_with(&mut t, "Spark Harvest");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.advance_to_step(Step::BeginningOfCombat);
    t.g.turn.priority = Some(P0);
    assert!(legal_cast_methods(&mut t, P0, harvest).is_empty());
    t.advance_to_step(Step::PostcombatMain);
    assert_eq!(legal_cast_methods(&mut t, P0, harvest), vec![PAY_LIFE]);
}

#[test]
fn elsha_of_the_infinite_casts_noncreature_spells_from_the_top_with_flash_and_their_costs() {
    cr!("702.8a", "601.3", "118.9a");
    ruling!(
        "Elsha of the Infinite",
        "You'll still pay all costs for a spell you cast from your library, including additional costs. You may also pay alternative costs."
    );
    supported("Elsha of the Infinite");
    // Divination, a sorcery, may be cast from the top of the library at instant speed,
    // paying its mana cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elsha of the Infinite");
    let div = t.library_top(P0, "Divination");
    t.lands(P0, "Island", 3);
    t.advance_to_step(Step::BeginningOfCombat);
    t.g.turn.priority = Some(P0);
    assert_eq!(
        legal_cast_methods(&mut t, P0, div),
        vec![CastMethod::Normal]
    );
    t.cast(P0, div).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    // A creature card isn't allowed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elsha of the Infinite");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
    // Force of Will may be cast for its alternative cost (1 life and a blue card exiled).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elsha of the Infinite");
    let force = t.library_top(P0, "Force of Will");
    t.hand(P0, "Brainstorm");
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.g.turn.priority = Some(P0);
    let methods = legal_cast_methods(&mut t, P0, force);
    assert_eq!(methods.len(), 1, "{methods:?}");
    assert!(matches!(methods[0], CastMethod::Alternative(_)));
    t.cast(P0, force)
        .method(methods[0].clone())
        .target(spell)
        .go();
    assert_eq!(t.life(P0), 19);
}

// ---------------------------------------------------------------------------
// Choosing the permission (Muldrotha, Lurrus, Karador)
// ---------------------------------------------------------------------------

/// Answers which of Muldrotha's permissions P0 uses: `i` among those allowing the card,
/// in the order land, artifact, battle, creature, enchantment, planeswalker.
fn use_slot(t: &mut TestGame, i: usize) {
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn muldrotha_an_artifact_creature_as_the_artifact_spell_and_another_as_the_creature_spell() {
    cr!("110.4", "601.2", "601.3");
    ruling!(
        "Muldrotha, the Gravetide",
        "For example, you may cast an artifact creature spell as your artifact spell and cast another artifact creature spell as your creature spell."
    );
    supported("Muldrotha, the Gravetide");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muldrotha, the Gravetide");
    let a = t.graveyard(P0, "Ornithopter");
    let b = t.graveyard(P0, "Ornithopter");
    let c = t.graveyard(P0, "Ornithopter");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    use_slot(&mut t, 0);
    t.cast(P0, a).go();
    // P0 was asked which permission: as the artifact or the creature spell.
    let asked = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseOption { options, .. } => Some(options.clone()),
        _ => None,
    });
    assert_eq!(asked.map(|o| o.len()), Some(2));
    t.resolve_all();
    // The second one as the creature spell.
    assert!(!legal_cast_methods(&mut t, P0, b).is_empty());
    t.cast(P0, b).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 2);
    // Both used: no third artifact creature, and no creature at all.
    assert!(legal_cast_methods(&mut t, P0, c).is_empty());
    t.lands(P0, "Forest", 2);
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
}

#[test]
fn muldrotha_the_type_as_its_cast_counts() {
    cr!("601.3e", "702.103a", "110.4");
    ruling!(
        "Muldrotha, the Gravetide",
        "Use the type of the card as it's played or cast to determine which permanent type to count it as. For example, if you cast a creature spell from your graveyard, you can cast a card with bestow as an enchantment spell."
    );
    supported("Muldrotha, the Gravetide");
    supported("Boon Satyr");
    // A creature spell first: Boon Satyr (an enchantment creature card) may then be cast
    // bestowed, as an Aura spell (an enchantment spell).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muldrotha, the Gravetide");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let satyr = t.graveyard(P0, "Boon Satyr");
    t.lands(P0, "Forest", 7);
    t.cast(P0, bears).go();
    t.resolve_all();
    let methods = legal_cast_methods(&mut t, P0, satyr);
    assert!(methods.contains(&CastMethod::Keyword(KeywordKind::Bestow)));
    let target = t.named_on_battlefield("Grizzly Bears")[0];
    t.cast(P0, satyr)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(target)
        .go();
    t.resolve_all();
    // An enchantment spell first: Boon Satyr may be cast as an (enchantment) creature
    // spell, using the creature permission, but not bestowed (an Aura spell, not a
    // creature spell).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muldrotha, the Gravetide");
    let anthem = t.graveyard(P0, "Glorious Anthem");
    let satyr = t.graveyard(P0, "Boon Satyr");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Forest", 5);
    t.cast(P0, anthem).go();
    t.resolve_all();
    assert_eq!(
        legal_cast_methods(&mut t, P0, satyr),
        vec![CastMethod::Normal]
    );
}

#[test]
fn muldrotha_a_new_muldrotha_gives_new_permissions() {
    cr!("400.7", "601.3");
    ruling!(
        "Muldrotha, the Gravetide",
        "If you play a card from your graveyard and then have a new Muldrotha come under your control in the same turn, you may play another land or spell of that type from your graveyard that turn."
    );
    supported("Muldrotha, the Gravetide");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muldrotha, the Gravetide");
    let forest = t.graveyard(P0, "Forest");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 3);
    t.play_land(P0, forest).expect("land from the graveyard");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(legal_cast_methods(&mut t, P0, elves).is_empty());
    t.battlefield(P0, "Muldrotha, the Gravetide");
    assert!(!legal_cast_methods(&mut t, P0, elves).is_empty());
    // The same Muldrotha leaving and returning is a new object (CR 400.7), with
    // permissions of its own.
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 3);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(legal_cast_methods(&mut t, P0, elves).is_empty());
    let cloudshift = t.hand(P0, "Cloudshift");
    add_mana(&mut t, P0, ManaType::W, 1);
    t.cast(P0, cloudshift).target(muldrotha).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Muldrotha, the Gravetide").len(), 1);
    assert!(!t.is_live(muldrotha), "a new object");
    assert!(!legal_cast_methods(&mut t, P0, elves).is_empty());
    t.cast(P0, elves).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn muldrotha_normal_timing_land_plays_and_costs() {
    cr!("305.2b", "307.1", "601.3", "118.9", "601.2h");
    ruling!(
        "Muldrotha, the Gravetide",
        "You must follow the normal timing permissions and restrictions of the cards you play from your graveyard. For example, you can't use Muldrotha to play a land if you don't have an available land play or to cast a planeswalker spell during your end step."
    );
    ruling!(
        "Muldrotha, the Gravetide",
        "You must pay the costs to cast a spell this way. If it has an alternative cost, you may cast it for that cost instead."
    );
    ruling!(
        "Muldrotha, the Gravetide",
        "Once you begin to cast a spell, losing control of Muldrotha won't affect the spell."
    );
    supported("Muldrotha, the Gravetide");
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    let forest = t.graveyard(P0, "Forest");
    let liliana = t.graveyard(P0, "Liliana of the Veil");
    let md = t.graveyard(P0, "Mulldrifter");
    t.lands(P0, "Swamp", 3);
    // No land play left: the land can't be played.
    let plains = t.hand(P0, "Plains");
    t.play_land(P0, plains).expect("land from hand");
    assert!(!can_play_land(&mut t, P0, forest));
    // No planeswalker during the end step.
    t.advance_to_step(Step::End);
    t.g.turn.priority = Some(P0);
    assert!(legal_cast_methods(&mut t, P0, liliana).is_empty());
    // On the next turn: Mulldrifter for its evoke cost ({2}{U}, not {4}{U}).
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 1);
    let methods = legal_cast_methods(&mut t, P0, md);
    assert!(
        methods.contains(&CastMethod::Keyword(KeywordKind::Evoke)),
        "{methods:?}"
    );
    let hand = t.hand_size(P0);
    t.cast(P0, md)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert!(
        t.hand_size(P0) >= hand + 2,
        "Mulldrifter resolved and drew two"
    );
    assert!(t.in_graveyard(P0, "Mulldrifter"), "evoked");
    let _ = muldrotha;
    // Losing Muldrotha while casting: Wretched Gryff ({7}, emerge {5}{U}) cast from the
    // graveyard by sacrificing Muldrotha (mana value 6) as its emerge cost, for {U}.
    // Muldrotha leaves as the costs are paid (CR 601.2h); the spell is still cast.
    supported("Wretched Gryff");
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    let gryff = t.graveyard(P0, "Wretched Gryff");
    t.lands(P0, "Island", 1);
    t.answer_choose(P0, &[Entity::Object(muldrotha)]);
    let spell = t
        .cast(P0, gryff)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .go();
    assert!(
        t.in_graveyard(P0, "Muldrotha, the Gravetide"),
        "sacrificed to emerge"
    );
    assert_eq!(tapped_lands(&t, P0), 1, "{{U}}");
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn lurrus_another_permission_leaves_lurrus_unused() {
    cr!("601.2", "601.3");
    ruling!(
        "Lurrus of the Dream-Den",
        "If you cast a spell from your graveyard using another permission, Lurrus's effect doesn't apply. You can cast another permanent spell from your graveyard."
    );
    ruling!(
        "Lurrus of the Dream-Den",
        "If you cast one permanent spell from your graveyard and then have a new Lurrus come under your control in the same turn, you may cast another permanent spell from your graveyard that turn."
    );
    supported("Lurrus of the Dream-Den");
    supported("Emry, Lurker of the Loch");
    // Emry's permission for Ornithopter: Lurrus's remains for Grizzly Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lurrus of the Dream-Den");
    let emry = t.battlefield(P0, "Emry, Lurker of the Loch");
    let thopter = t.graveyard(P0, "Ornithopter");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.activate(P0, emry, 0, &[Entity::Object(thopter)])
        .expect("activate Emry");
    t.resolve_all();
    t.answer_choose(P0, &[Entity::Object(emry)]);
    t.cast(P0, thopter).go();
    t.resolve_all();
    t.lands(P0, "Forest", 3);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Lurrus's used: no other permanent spell — until a new Lurrus arrives.
    assert!(legal_cast_methods(&mut t, P0, elves).is_empty());
    t.battlefield(P0, "Lurrus of the Dream-Den");
    assert!(!legal_cast_methods(&mut t, P0, elves).is_empty());
}

#[test]
fn lurrus_no_lands_costs_or_alternative_costs_and_losing_lurrus() {
    cr!("601.3", "118.9", "305.1", "601.2h");
    ruling!(
        "Lurrus of the Dream-Den",
        "Lurrus doesn't let you play lands from your graveyard."
    );
    ruling!(
        "Lurrus of the Dream-Den",
        "You must pay the costs to cast that spell. If it has an alternative cost, such as a mutate cost, you may cast it for that cost instead."
    );
    ruling!(
        "Lurrus of the Dream-Den",
        "Once you begin to cast the spell, losing control of Lurrus won't affect the spell. You can finish casting it as normal."
    );
    supported("Lurrus of the Dream-Den");
    supported("Mardu Scout");
    let mut t = TestGame::new(2);
    let lurrus = t.battlefield(P0, "Lurrus of the Dream-Den");
    let forest = t.graveyard(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, forest));
    // Mardu Scout ({R}{R}, dash {1}{R}) for its dash cost.
    let scout = t.graveyard(P0, "Mardu Scout");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    assert_eq!(
        legal_cast_methods(&mut t, P0, scout),
        vec![CastMethod::Keyword(KeywordKind::Dash)]
    );
    let spell = t
        .cast(P0, scout)
        .method(CastMethod::Keyword(KeywordKind::Dash))
        .go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    let _ = lurrus;
    // Losing Lurrus while casting: Dusk Rose Reliquary ({W}; as an additional cost,
    // sacrifice an artifact or creature) cast from the graveyard with Lurrus's permission,
    // sacrificing Lurrus as the costs are paid (CR 601.2h).
    supported("Dusk Rose Reliquary");
    let mut t = TestGame::new(2);
    let lurrus = t.battlefield(P0, "Lurrus of the Dream-Den");
    let reliquary = t.graveyard(P0, "Dusk Rose Reliquary");
    t.lands(P0, "Plains", 1);
    t.answer_choose(P0, &[Entity::Object(lurrus)]);
    let spell = t.cast(P0, reliquary).go();
    assert!(
        t.in_graveyard(P0, "Lurrus of the Dream-Den"),
        "sacrificed as a cost"
    );
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn karador_losing_karador_doesnt_affect_the_spell() {
    cr!("601.3", "601.2h", "702.119a");
    ruling!(
        "Karador, Ghost Chieftain",
        "Once you begin to cast the spell, losing control of Karador won't affect the spell. You can finish casting it as normal."
    );
    supported("Karador, Ghost Chieftain");
    supported("Wretched Gryff");
    // Wretched Gryff cast from the graveyard with Karador's permission, sacrificing Karador
    // (mana value 8) as its emerge cost: Karador leaves as the costs are paid (CR 601.2h),
    // and the spell is cast and resolves.
    let mut t = TestGame::new(2);
    let karador = t.battlefield(P0, "Karador, Ghost Chieftain");
    let gryff = t.graveyard(P0, "Wretched Gryff");
    t.lands(P0, "Island", 1);
    t.answer_choose(P0, &[Entity::Object(karador)]);
    let spell = t
        .cast(P0, gryff)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .go();
    assert!(
        t.in_graveyard(P0, "Karador, Ghost Chieftain"),
        "sacrificed to emerge"
    );
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn lurrus_the_mana_value_counts_the_value_chosen_for_x() {
    cr!("601.2e", "601.3e", "202.3e");
    ruling!(
        "Lurrus of the Dream-Den",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell's mana value. For example, if a permanent spell costs {X}{W}, you could cast it with X as 1 but not as 2."
    );
    supported("Lurrus of the Dream-Den");
    supported("Hangarback Walker");
    // Hangarback Walker ({X}{X}): with X = 1 its mana value is 2; with X = 2, 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lurrus of the Dream-Den");
    let walker = t.graveyard(P0, "Hangarback Walker");
    t.lands(P0, "Wastes", 4);
    assert_eq!(
        legal_cast_methods(&mut t, P0, walker),
        vec![CastMethod::Normal]
    );
    // X = 2: the proposed spell isn't one Lurrus allows; the casting is undone (CR 733).
    assert!(t.cast(P0, walker).x(2).try_go().is_err());
    assert_eq!(t.obj(walker).zone, Zone::Graveyard(P0));
    assert_eq!(tapped_lands(&t, P0), 0);
    // X = 1 is fine.
    let spell = t.cast(P0, walker).x(1).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

/// P0 casts Hurl Through Hell on `target` and it resolves: the card is exiled.
fn hurl(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let hurl = t.hand(P0, "Hurl Through Hell");
    add_mana(t, P0, ManaType::B, 1);
    add_mana(t, P0, ManaType::R, 1);
    add_mana(t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, hurl).target(target).go();
    t.resolve_all();
    let exiled = t.g.current(target);
    assert_eq!(t.obj(exiled).zone, Zone::Exile);
    exiled
}

#[test]
fn hurl_through_hell_an_animated_land_cant_be_played() {
    cr!("305.9", "601.3");
    ruling!(
        "Hurl Through Hell",
        "If the creature you exile is actually a land card that was animated, you won't be able to play the land card from exile."
    );
    supported("Hurl Through Hell");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let mutavault = t.battlefield(P1, "Mutavault");
    t.lands(P1, "Wastes", 1);
    // P1 animates Mutavault ("{1}: ... becomes a 2/2 creature ... It's still a land.").
    t.g.turn.priority = Some(P1);
    t.activate(P1, mutavault, 1, &[])
        .expect("animate Mutavault");
    t.resolve_all();
    assert!(t
        .obj_now(mutavault)
        .chars
        .is(mtg_engine::types::CardType::Creature));
    let card = hurl(&mut t, mutavault);
    // P0 has a land play, in their main phase with an empty stack: still not this card.
    assert!(!can_play_land(&mut t, P0, card));
    assert!(t.play_land(P0, card).is_err());
    assert!(legal_cast_methods(&mut t, P0, card).is_empty());
}

#[test]
fn hurl_through_hell_the_card_is_cast_with_normal_timing_and_mana_of_any_color() {
    cr!("307.1", "601.3", "609.4b", "118.14");
    ruling!(
        "Hurl Through Hell",
        "You must still follow all normal timing rules for casting the spell."
    );
    supported("Hurl Through Hell");
    supported("Thought-Knot Seer");
    // Grizzly Bears ({1}{G}) with red mana: mana may be spent as though it were mana of
    // any color — but not during combat (a creature spell's timing).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = hurl(&mut t, bears);
    t.lands(P0, "Mountain", 2);
    t.advance_to_step(Step::BeginningOfCombat);
    t.g.turn.priority = Some(P0);
    assert!(legal_cast_methods(&mut t, P0, card).is_empty());
    // Until the end of P0's next turn: in their next main phase.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(
        legal_cast_methods(&mut t, P0, card),
        vec![CastMethod::Normal]
    );
    t.cast(P0, card).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Thought-Knot Seer ({3}{C}): {C} still needs colorless mana (mana of any color, not
    // of any type).
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P1, "Thought-Knot Seer");
    let card = hurl(&mut t, seer);
    t.lands(P0, "Mountain", 4);
    assert!(t
        .cast(P0, card)
        .target(Entity::Player(P1))
        .try_go()
        .is_err());
    t.lands(P0, "Wastes", 1);
    t.cast(P0, card).target(Entity::Player(P1)).go();
}

#[test]
fn gaeas_will_lands_and_spells_from_the_graveyard_follow_the_usual_rules() {
    cr!("305.2", "307.1", "601.3", "601.2f");
    ruling!(
        "Gaea's Will",
        "The lands you play and spells you cast from your graveyard must follow the usual timing restrictions, and you must pay any costs for spells you cast."
    );
    supported("Gaea's Will");
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Forest");
    let island = t.graveyard(P0, "Island");
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(!can_play_land(&mut t, P0, forest));
    // Gaea's Will has no mana cost: cast it as suspend would, without paying it.
    let will = t.hand(P0, "Gaea's Will");
    mtg_engine::casting::cast_during_resolution(&mut t.g, P0, will, CastMethod::Free)
        .expect("cast Gaea's Will");
    t.resolve_all();
    // One land play.
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).expect("play the Forest");
    assert!(!can_play_land(&mut t, P0, island), "no land play left");
    // Grizzly Bears: its mana cost is paid, at sorcery speed only.
    assert!(
        legal_cast_methods(&mut t, P0, bears).is_empty(),
        "one land: {{1}}{{G}} unpaid"
    );
    t.lands(P0, "Forest", 1);
    t.advance_to_step(Step::BeginningOfCombat);
    t.g.turn.priority = Some(P0);
    assert!(
        legal_cast_methods(&mut t, P0, bears).is_empty(),
        "not during combat"
    );
    t.advance_to_step(Step::PostcombatMain);
    assert_eq!(
        legal_cast_methods(&mut t, P0, bears),
        vec![CastMethod::Normal]
    );
    t.cast(P0, bears).go();
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn karador_with_yawgmoths_will_uses_the_permission_chosen() {
    cr!("601.2", "601.3");
    ruling!(
        "Muldrotha, the Gravetide",
        "If multiple effects allow you to play a card from your graveyard, you must announce which permission you're using as you begin to play the card."
    );
    supported("Karador, Ghost Chieftain");
    supported("Yawgmoth's Will");
    for use_karador in [false, true] {
        let mut t = TestGame::new(2);
        let karador = t.battlefield(P0, "Karador, Ghost Chieftain");
        let bears = t.graveyard(P0, "Grizzly Bears");
        let will = t.hand(P0, "Yawgmoth's Will");
        add_mana(&mut t, P0, ManaType::B, 3);
        let will_spell = t.cast(P0, will).go();
        t.resolve_all();
        t.lands(P0, "Forest", 2);
        // By default, the permission that can be used any number of times.
        if use_karador {
            t.answer_choose(P0, &[Entity::Object(karador)]);
        }
        let from = t.asked().len();
        t.cast(P0, bears).go();
        let candidates = t.asked()[from..].iter().find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        });
        assert_eq!(
            candidates.map(|c| c.len()),
            Some(2),
            "Karador's or Yawgmoth's Will's ({will_spell:?})"
        );
        let used =
            t.g.history
                .once_permissions_used
                .iter()
                .any(|(o, _)| *o == karador);
        assert_eq!(used, use_karador);
    }
}

// ---------------------------------------------------------------------------
// Other cards whose permissions now compile
// ---------------------------------------------------------------------------

#[test]
fn korvold_and_the_noble_thief_plays_an_opponents_exiled_cards_this_turn() {
    cr!("714.2b", "305.1", "601.3", "601.2f");
    ruling!(
        "Korvold and the Noble Thief",
        "You pay all costs and follow all normal timing rules for a card played this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Korvold and the Noble Thief");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Korvold and the Noble Thief");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 2);
    let cards = stack_library(
        &mut t,
        P1,
        &["Forest", "Grizzly Bears", "Hill Giant", "Shock"],
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s29_common::put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    // Chapter III: the top three cards of P1's library are exiled.
    let [forest, bears, giant, shock] = [0, 1, 2, 3].map(|i| t.g.current(cards[i]));
    for c in [forest, bears, giant] {
        assert_eq!(t.obj(c).zone, Zone::Exile);
    }
    assert_eq!(t.obj(shock).zone, Zone::Library(P1));
    // P0 may play the land with their land play, and cast the spells paying their costs.
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).expect("play P1's Forest");
    t.lands(P0, "Forest", 1);
    assert_eq!(
        legal_cast_methods(&mut t, P0, bears),
        vec![CastMethod::Normal]
    );
    assert!(
        legal_cast_methods(&mut t, P0, giant).is_empty(),
        "{{3}}{{R}} unpaid"
    );
    t.cast(P0, bears).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(creatures(&t, P0).len(), 1);
    // Only this turn.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    assert!(legal_cast_methods(&mut t, P0, giant).is_empty());
}

#[test]
fn rundvelt_hordemaster_triggers_for_each_goblin_dying_at_once() {
    cr!("603.2c", "603.10a", "601.3");
    ruling!(
        "Rundvelt Hordemaster",
        "If Rundvelt Hordemaster and one or more other Goblins you control die at the same time, its ability will trigger once for each of them."
    );
    supported("Rundvelt Hordemaster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rundvelt Hordemaster");
    t.battlefield(P0, "Raging Goblin");
    let cards = stack_library(&mut t, P0, &["Goblin Guide", "Grizzly Bears"]);
    let doj = t.hand(P0, "Day of Judgment");
    add_mana(&mut t, P0, ManaType::W, 2);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, doj).go();
    t.resolve_all();
    // Two triggers: each exiles the top card of P0's library.
    let (guide, bears) = (t.g.current(cards[0]), t.g.current(cards[1]));
    assert_eq!(t.obj(guide).zone, Zone::Exile);
    assert_eq!(t.obj(bears).zone, Zone::Exile);
    // A Goblin creature card may be cast (until the end of P0's next turn); another card
    // may not.
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 2);
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(
        legal_cast_methods(&mut t, P0, guide),
        vec![CastMethod::Normal]
    );
    let spell = t.cast(P0, guide).go();
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn yawgmoths_will_exiles_itself_and_cards_put_into_the_graveyard_as_costs() {
    cr!("614.1a", "601.3", "608.2n");
    ruling!(
        "Yawgmoth's Will",
        "It will exile itself since it goes to the graveyard after its effect starts."
    );
    ruling!(
        "Yawgmoth's Will",
        "The second ability creates a replacement effect. It applies to both costs and effects."
    );
    supported("Yawgmoth's Will");
    supported("Tormenting Voice");
    let mut t = TestGame::new(2);
    let will = t.hand(P0, "Yawgmoth's Will");
    let voice = t.graveyard(P0, "Tormenting Voice");
    let giant = t.hand(P0, "Hill Giant");
    add_mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, will).go();
    t.resolve_all();
    assert!(t.in_exile("Yawgmoth's Will"));
    assert!(!t.in_graveyard(P0, "Yawgmoth's Will"));
    // Tormenting Voice from the graveyard, discarding Hill Giant as its additional cost:
    // both are exiled instead of put into the graveyard.
    t.lands(P0, "Mountain", 2);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, voice).go();
    assert!(t.in_exile("Hill Giant"), "discarded as a cost: exiled");
    t.resolve_all();
    assert!(t.in_exile("Tormenting Voice"));
    assert_eq!(t.graveyard_size(P0), 0);
}
