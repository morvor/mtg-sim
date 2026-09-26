//! CR 202: mana cost and color (and CR 204, the color indicator): paying mana costs, the
//! colors mana symbols and color indicators give, and mana value.

use crate::r105_util::{colors, cs};
use crate::r200_common::*;
use crate::r300_common::can_cast;
use crate::r703_common::run_effect;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::{ManaCost, ManaSymbol};
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `obj` becomes a copy of `of` (CR 707).
fn become_copy(t: &mut TestGame, obj: ObjectId, of: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(obj)], vec![Entity::Object(of)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
}

// ---------------------------------------------------------------------------
// 202.1: mana cost
// ---------------------------------------------------------------------------

#[test]
fn a_mana_cost_is_the_mana_symbols_printed_on_the_card() {
    cr!("202.1");
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    let seer = t.hand(P0, "Thought-Knot Seer");
    let c = t.obj_now(bears).chars.mana_cost.clone().unwrap();
    assert_eq!(
        c.symbols.to_vec(),
        vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Green)]
    );
    assert_eq!(c.to_string(), "{1}{G}");
    let c = t.obj_now(seer).chars.mana_cost.clone().unwrap();
    assert_eq!(
        c.symbols.to_vec(),
        vec![ManaSymbol::Generic(3), ManaSymbol::Colorless]
    );
}

#[test]
fn paying_a_mana_cost_requires_matching_colored_and_colorless_symbols() {
    cr!("202.1a");
    // {1}{G}: two Mountains can't pay it; a Forest and a Mountain can.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    t.lands(P0, "Forest", 1);
    assert!(t.cast(P0, bears).try_go().is_ok());
    // {3}{C}: the {C} must be paid with colorless mana.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let seer = t.hand(P0, "Thought-Knot Seer");
    assert!(t.cast(P0, seer).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    assert!(t.cast(P0, seer).try_go().is_ok());
    // Phyrexian mana symbols are the exception: {B/P} may be paid with 2 life instead.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let d = t.hand(P0, "Dismember"); // {1}{B/P}{B/P}
    t.cast(P0, d).target(bear).go();
    assert_eq!(t.life(P0), 16);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn objects_with_no_mana_cost() {
    // CR 202.1b: lands, cards with no mana symbols where the mana cost would be, and
    // tokens have no mana cost; no mana cost is an unpayable cost (CR 118.6), and lands
    // are played without paying any costs.
    cr!("202.1b");
    ruling!(
        "Ancestral Vision",
        "A card with no mana cost can't be cast normally"
    );
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let vision = t.hand(P0, "Ancestral Vision");
    assert!(t.obj_now(forest).chars.mana_cost.is_none());
    assert!(t.obj_now(vision).chars.mana_cost.is_none());
    t.lands(P0, "Island", 6);
    // Ancestral Vision can't be cast for its (unpayable) mana cost.
    assert!(!can_cast(&mut t, P0, vision));
    assert!(t.cast(P0, vision).try_go().is_err());
    // A land is played without paying any cost.
    let untapped = |t: &TestGame| t.g.battlefield.iter().filter(|i| !t.obj_now(**i).tapped).count();
    let before = untapped(&t);
    t.play_land(P0, forest).unwrap();
    assert_eq!(untapped(&t), before + 1);
    // A token has no mana cost unless the effect creating it says otherwise.
    t.lands(P0, "Plains", 3);
    let sp = t.hand(P0, "Spectral Procession");
    t.cast(P0, sp).go();
    t.resolve();
    let spirits = t.named_on_battlefield("Spirit Token");
    assert_eq!(spirits.len(), 3);
    for s in spirits {
        assert!(t.obj_now(s).chars.mana_cost.is_none());
        assert_eq!(mv(&mut t, s), 0);
    }
}

// ---------------------------------------------------------------------------
// 202.2: color
// ---------------------------------------------------------------------------

#[test]
fn an_object_is_the_colors_of_its_mana_symbols_regardless_of_frame() {
    cr!("202.2", "202.2a");
    // The five colors and their symbols.
    for (letter, color) in [
        ('W', Color::White),
        ('U', Color::Blue),
        ('B', Color::Black),
        ('R', Color::Red),
        ('G', Color::Green),
    ] {
        let sym = ManaSymbol::parse(&letter.to_string()).unwrap();
        assert_eq!(sym, ManaSymbol::Colored(color));
        assert_eq!(sym.colors(), ColorSet::single(color));
        assert_eq!(color.letter(), letter);
    }
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let plains = t.battlefield(P0, "Plains");
    assert_eq!(colors(&t, bolt), cs("R"));
    assert_eq!(colors(&t, bears), cs("G"));
    // A land (no mana cost) is colorless, whatever its frame looks like.
    assert_eq!(colors(&t, plains), ColorSet::NONE);
}

#[test]
fn objects_without_colored_mana_symbols_are_colorless() {
    cr!("202.2b");
    let mut t = TestGame::new(2);
    let memnite = t.battlefield(P0, "Memnite"); // {0}
    let seer = t.battlefield(P0, "Thought-Knot Seer"); // {3}{C}
    let thopter = t.hand(P0, "Ornithopter"); // {0}
    for id in [memnite, seer, thopter] {
        assert_eq!(colors(&t, id), ColorSet::NONE);
        assert!(matches!(t.obj_now(id).chars.colors, c if c.is_colorless()));
    }
}

#[test]
fn two_different_colored_symbols_make_a_multicolored_object() {
    cr!("202.2c");
    let mut t = TestGame::new(2);
    let helix = t.hand(P0, "Lightning Helix"); // {R}{W}
    let charm = t.hand(P0, "Esper Charm"); // {W}{U}{B}
    assert_eq!(colors(&t, helix), cs("RW"));
    assert!(colors(&t, helix).is_multicolored());
    assert_eq!(colors(&t, charm), cs("WUB"));
    // Two symbols of the same color make just one color.
    let counterspell = t.hand(P0, "Counterspell"); // {U}{U}
    assert_eq!(colors(&t, counterspell), cs("U"));
    assert!(colors(&t, counterspell).is_monocolored());
}

#[test]
fn hybrid_and_phyrexian_symbols_give_all_their_colors() {
    cr!("202.2d");
    ruling!(
        "Spectral Procession",
        "Thus, Spectral Procession is white even if you spend six blue mana to cast it."
    );
    let mut t = TestGame::new(2);
    let recruit = t.hand(P0, "Boros Recruit"); // {R/W}
    let finks = t.hand(P0, "Kitchen Finks"); // {1}{G/W}{G/W}
    let procession = t.hand(P0, "Spectral Procession"); // {2/W}{2/W}{2/W}
    let dismember = t.hand(P0, "Dismember"); // {1}{B/P}{B/P}
    let guildmage = t.hand(P0, "Azorius Guildmage"); // {W/U}{W/U}
    assert_eq!(colors(&t, recruit), cs("RW"));
    assert_eq!(colors(&t, finks), cs("GW"));
    assert_eq!(colors(&t, procession), cs("W"));
    assert_eq!(colors(&t, dismember), cs("B"));
    assert_eq!(colors(&t, guildmage), cs("WU"));
    // "In addition to any other colors": {W/U/P} with {R} is white, blue and red.
    assert_eq!(ManaCost::parse("{R}{W/U/P}").unwrap().colors(), cs("RWU"));
}

#[test]
fn a_color_indicator_gives_its_colors() {
    cr!("202.2e", "204.1", "204.2");
    ruling!(
        "Dryad Arbor",
        "Due to its color indicator (appearing to the left of its type line), Dryad Arbor is green. Color indicators apply in all zones"
    );
    ruling!(
        "Nicol Bolas, the Ravager",
        "This color indicator means that it's a blue, black, and red permanent."
    );
    let mut t = TestGame::new(2);
    // Nonland cards without a mana cost usually have a color indicator.
    let vision = t.hand(P0, "Ancestral Vision");
    assert_eq!(t.obj_now(vision).chars.color_indicator, Some(cs("U")));
    assert_eq!(colors(&t, vision), cs("U"));
    // Dryad Arbor: a land with a green color indicator is green, in every zone.
    let arbor = t.battlefield(P0, "Dryad Arbor");
    assert_eq!(colors(&t, arbor), cs("G"));
    let arbor_gy = t.graveyard(P0, "Dryad Arbor");
    assert_eq!(colors(&t, arbor_gy), cs("G"));
    // An indicator with several colors: Nicol Bolas, the Arisen (the back face of Nicol
    // Bolas, the Ravager) is blue, black, and red.
    let bolas = t.battlefield(P0, "Nicol Bolas, the Ravager");
    assert!(mtg_engine::dfc::transform(&mut t.g, bolas));
    t.g.recompute();
    let o = t.obj_now(bolas);
    assert_eq!(o.chars.name, "Nicol Bolas, the Arisen");
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(o.chars.color_indicator, Some(cs("UBR")));
    assert_eq!(colors(&t, bolas), cs("UBR"));
    // Insectile Aberration (a blue color indicator) is blue.
    let delver = t.battlefield(P0, "Delver of Secrets");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    assert_eq!(colors(&t, delver), cs("U"));
}

#[test]
fn effects_can_change_an_objects_color() {
    cr!("202.2f");
    let mut t = TestGame::new(2);
    // Cerulean Wisps: "Target creature becomes blue until end of turn."
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let w1 = t.hand(P0, "Cerulean Wisps");
    t.cast(P0, w1).target(thopter).go();
    t.resolve_all();
    assert_eq!(colors(&t, thopter), cs("U"), "a colorless object gets a color");
    let w2 = t.hand(P0, "Cerulean Wisps");
    t.cast(P0, w2).target(bears).go();
    t.resolve_all();
    assert_eq!(colors(&t, bears), cs("U"), "the new color replaces its color");
    // An effect can make a colored object colorless.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetColors(ColorSet::NONE)],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert_eq!(colors(&t, bears), ColorSet::NONE);
}

// ---------------------------------------------------------------------------
// 202.3: mana value
// ---------------------------------------------------------------------------

#[test]
fn objects_with_no_mana_cost_have_mana_value_zero() {
    cr!("202.3", "202.3a");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let vision = t.hand(P0, "Ancestral Vision");
    let seer = t.hand(P0, "Thought-Knot Seer"); // {3}{C}
    let bears = t.hand(P0, "Grizzly Bears");
    assert_eq!(mv(&mut t, forest), 0);
    assert_eq!(mv(&mut t, vision), 0);
    assert_eq!(mv(&mut t, seer), 4);
    assert_eq!(mv(&mut t, bears), 2);
    // A mana cost of {3}{U}{U} has mana value 5, regardless of color.
    assert_eq!(ManaCost::parse("{3}{U}{U}").unwrap().mana_value(), 5);
    // The mana value filters agree ("with mana value 0").
    let f = Filter::ManaValue(Cmp::Eq, Box::new(Value::c(0)));
    assert!(crate::r105_util::matches(&t, vision, &f, P0));
    assert!(!crate::r105_util::matches(&t, bears, &f, P0));
}

#[test]
fn a_transformed_permanent_uses_its_front_faces_mana_cost() {
    // CR 202.3b and its examples.
    cr!("202.3b");
    ruling!(
        "Nicol Bolas, the Ravager",
        "For example, the mana value of Nicol Bolas, the Arisen is 4."
    );
    ruling!(
        "Lunarch Veteran // Luminous Phantom",
        "The mana value of a spell cast using disturb is determined by the mana cost on the front face of the card"
    );
    let mut t = TestGame::new(2);
    let bolas = t.battlefield(P0, "Nicol Bolas, the Ravager"); // {1}{U}{B}{R}
    assert!(mtg_engine::dfc::transform(&mut t.g, bolas));
    t.g.recompute();
    assert_eq!(t.obj_now(bolas).chars.name, "Nicol Bolas, the Arisen");
    assert_eq!(mv(&mut t, bolas), 4);
    let hunt = t.battlefield(P0, "Huntmaster of the Fells"); // {2}{R}{G}
    assert_eq!(mv(&mut t, hunt), 4);
    assert!(mtg_engine::dfc::transform(&mut t.g, hunt));
    t.g.recompute();
    assert_eq!(t.obj_now(hunt).chars.name, "Ravager of the Fells");
    assert!(t.obj_now(hunt).chars.mana_cost.is_none());
    assert_eq!(mv(&mut t, hunt), 4, "its mana value remains 4");
    let f = Filter::ManaValue(Cmp::Eq, Box::new(Value::c(4)));
    assert!(crate::r105_util::matches(&t, hunt, &f, P0));
    // A permanent that becomes a copy of Ravager of the Fells has mana value 0.
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(&mut t, bears, hunt);
    assert_eq!(t.obj_now(bears).chars.name, "Ravager of the Fells");
    assert_eq!(mv(&mut t, bears), 0);
    // Insectile Aberration (the back face of Delver of Secrets, {U}) has mana value 1;
    // once it's a copy of Ravager of the Fells, 0.
    let delver = t.battlefield(P0, "Delver of Secrets");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    assert_eq!(t.obj_now(delver).chars.name, "Insectile Aberration");
    assert_eq!(mv(&mut t, delver), 1);
    become_copy(&mut t, delver, hunt);
    assert_eq!(t.obj_now(delver).chars.name, "Ravager of the Fells");
    assert_eq!(mv(&mut t, delver), 0);
    // A nonmodal double-faced spell cast transformed (disturb) also uses its front face's
    // mana cost: Luminous Phantom, back face of Lunarch Veteran ({W}).
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let vet = t.graveyard(P0, "Lunarch Veteran // Luminous Phantom");
    let spell = t
        .cast(P0, vet)
        .method(CastMethod::Keyword(KeywordKind::Disturb))
        .go();
    assert_eq!(t.obj_now(spell).face, FaceState::Back);
    assert!(t.obj_now(spell).chars.mana_cost.is_none());
    assert_eq!(mv(&mut t, spell), 1);
    t.resolve_all();
    let phantom = t.named_on_battlefield("Luminous Phantom")[0];
    assert_eq!(mv(&mut t, phantom), 1);
    // A modal double-faced card's back face has its own mana cost: Kazandu Valley (a
    // land) has none, so its mana value is 0.
    let mut t = TestGame::new(2);
    let mammoth = t.hand(P0, "Kazandu Mammoth");
    assert_eq!(mv(&mut t, mammoth), 3);
    t.play_land(P0, mammoth).unwrap();
    let valley = t.named_on_battlefield("Kazandu Valley")[0];
    assert_eq!(mv(&mut t, valley), 0);
}

#[test]
fn a_melded_permanent_uses_its_front_faces_combined_mana_cost() {
    cr!("202.3c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Graf Rats"); // {1}{B}
    t.battlefield(P0, "Midnight Scavengers"); // {4}{B}
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    let host = t.named_on_battlefield("Chittering Host")[0];
    assert!(t.obj_now(host).chars.mana_cost.is_none());
    assert_eq!(mv(&mut t, host), 7);
    // A copy of a melded permanent has mana value 0.
    let bears = t.battlefield(P1, "Grizzly Bears");
    become_copy(&mut t, bears, host);
    assert_eq!(t.obj_now(bears).chars.name, "Chittering Host");
    assert_eq!(mv(&mut t, bears), 0);
}

#[test]
fn split_card_mana_value_on_and_off_the_stack() {
    cr!("202.3d");
    ruling!(
        "Fire // Ice",
        "For example, Assault // Battery has a mana value of 5 while it is in your library."
    );
    let mut t = TestGame::new(2);
    let fi = t.hand(P0, "Fire // Ice"); // {1}{R} // {1}{U}
    assert_eq!(mv(&mut t, fi), 4);
    let gy = t.graveyard(P0, "Fire // Ice");
    assert_eq!(mv(&mut t, gy), 4);
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t
        .cast(P0, fi)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    assert_eq!(t.obj_now(spell).chars.name, "Ice");
    assert_eq!(mv(&mut t, spell), 2);
    t.resolve_all();
    // A fused split spell: Turn // Burn ({2}{U} // {1}{R}) has mana value 5 on the stack.
    let mut t = TestGame::new(2);
    let tb = t.hand(P0, "Turn // Burn");
    let fused = mtg_engine::card::card("Turn // Burn").characteristics(FaceState::Fused);
    assert_eq!(fused.mana_cost.unwrap().mana_value(), 5);
    assert_eq!(mv(&mut t, tb), 5);
}

#[test]
fn x_is_zero_except_on_the_stack() {
    cr!("202.3e");
    let mut t = TestGame::new(2);
    let blaze = t.hand(P0, "Blaze"); // {X}{R}
    assert_eq!(mv(&mut t, blaze), 1);
    t.lands(P0, "Mountain", 4);
    let spell = t.cast(P0, blaze).x(3).target(P1).go();
    assert_eq!(mv(&mut t, spell), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let gy = t.g.find_in_zone(Zone::Graveyard(P0), "Blaze")[0];
    assert_eq!(mv(&mut t, gy), 1);
}

#[test]
fn hybrid_symbols_count_their_largest_component() {
    cr!("202.3f");
    ruling!(
        "Spectral Procession",
        "Thus, Spectral Procession has a mana value of 6, even if you spend {W}{W}{W} to cast"
    );
    // The examples: {1}{W/U}{W/U} is 3, {2/B}{2/B}{2/B} is 6.
    assert_eq!(ManaCost::parse("{1}{W/U}{W/U}").unwrap().mana_value(), 3);
    assert_eq!(ManaCost::parse("{2/B}{2/B}{2/B}").unwrap().mana_value(), 6);
    let mut t = TestGame::new(2);
    let finks = t.hand(P0, "Kitchen Finks"); // {1}{G/W}{G/W}
    let procession = t.hand(P0, "Spectral Procession"); // {2/W}{2/W}{2/W}
    assert_eq!(mv(&mut t, finks), 3);
    assert_eq!(mv(&mut t, procession), 6);
    // ... even though Spectral Procession can be cast with three white mana.
    t.lands(P0, "Plains", 3);
    let spell = t.cast(P0, procession).go();
    assert_eq!(mv(&mut t, spell), 6);
}

#[test]
fn phyrexian_symbols_contribute_one() {
    cr!("202.3g");
    ruling!(
        "Dismember",
        "A Phyrexian mana symbol contributes 1 toward the mana value of a card, even if life is paid for it."
    );
    assert_eq!(ManaCost::parse("{1}{W/P}{W/P}").unwrap().mana_value(), 3);
    let mut t = TestGame::new(2);
    let legionnaire = t.hand(P0, "Porcelain Legionnaire"); // {2}{W/P}
    let dismember = t.hand(P0, "Dismember"); // {1}{B/P}{B/P}
    assert_eq!(mv(&mut t, legionnaire), 3);
    assert_eq!(mv(&mut t, dismember), 3);
    // Paying life for them doesn't change the mana value.
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    let spell = t.cast(P0, dismember).target(bear).go();
    assert_eq!(t.life(P0), 16);
    assert_eq!(mv(&mut t, spell), 3);
}

#[test]
fn additional_costs_arent_part_of_the_mana_cost() {
    cr!("202.4");
    // Burst Lightning ({R}, kicker {4}): kicked, it's still a one-mana-value red spell;
    // the kicker cost is paid along with the mana cost.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let burst = t.hand(P0, "Burst Lightning");
    let spell = t.cast(P0, burst).kicked(true).target(P1).go();
    let o = t.obj_now(spell);
    assert_eq!(o.chars.mana_cost.as_ref().unwrap().to_string(), "{R}");
    assert_eq!(o.chars.colors, cs("R"));
    assert_eq!(mv(&mut t, spell), 1);
    assert!(t.g.battlefield.iter().all(|l| t.obj_now(*l).tapped));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Bone Splinters ({B}; as an additional cost, sacrifice a creature): mana value 1.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.battlefield(P0, "Grizzly Bears");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let splinters = t.hand(P0, "Bone Splinters");
    let spell = t.cast(P0, splinters).target(bear).go();
    assert_eq!(mv(&mut t, spell), 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "the sacrifice was paid while casting");
}
