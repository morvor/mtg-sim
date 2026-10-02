//! Rulings batch P076 — split cards with three halves (CR 709), manifested permanents
//! (CR 701.40, 708), Aang's delayed trigger, token copies of lands and tokens (CR 707),
//! planes (CR 311, 901), and Higure's search during the first-strike damage step.

use crate::r_p076_common::*;
use crate::r_s01_common::supported;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const THERE: &str = "There // They're // Their";

/// All three halves of There // They're // Their compile.
fn there_halves_supported() {
    supported(THERE);
}

#[test]
fn there_theyre_their_is_one_card_in_the_graveyard() {
    cr!("709.2", "709.4c");
    ruling!(
        "There // They're // Their",
        "Each split card is a single card. For example, if you discard a split card, you’ve discarded one card. If an effect counts the number of instant and sorcery cards in your graveyard, There // They’re // Their counts once, not three times."
    );
    there_halves_supported();
    let mut t = TestGame::new(2);
    // Ghitu Lavarunner: +1/+0 and haste with two or more instant and/or sorcery cards in
    // your graveyard.
    let runner = t.battlefield(P0, "Ghitu Lavarunner");
    t.graveyard(P0, THERE);
    assert_eq!(t.pt(runner), (1, 2));
    t.graveyard(P0, "Giant Growth");
    assert_eq!(t.pt(runner), (2, 2));
}

#[test]
fn there_theyre_their_has_mana_value_6_outside_the_stack() {
    cr!("709.4", "202.3");
    ruling!(
        "There // They're // Their",
        "This split card’s characteristics are a combination of its three thirds while it’s not on the stack. For example, There // They’re // Their has a mana value of 6 while it’s in your library. If an effect allows you to search your library for a card with mana value 4 or less, you can’t find There // They’re // Their."
    );
    there_halves_supported();
    let mut t = TestGame::new(2);
    let c = t.library_top(P0, THERE);
    assert_eq!(crate::r_s26_common::mv(&mut t, c), 6);
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    let f = mtg_engine::ability::Filter::ManaValue(
        mtg_engine::ability::Cmp::Le,
        Box::new(mtg_engine::ability::Value::c(4)),
    );
    assert!(!t.g.matches(c, &f, &ctx));
}

#[test]
fn there_theyre_their_cast_third_only_on_the_stack() {
    cr!("709.3a", "709.3b");
    ruling!(
        "There // They're // Their",
        "To cast this split card, choose one of its thirds to cast. The characteristics of the thirds you didn’t cast are ignored while the spell is on the stack."
    );
    there_halves_supported();
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::U, 2);
    let card = t.hand(P0, THERE);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    let c = &t.obj(spell).chars;
    assert_eq!(c.name, "They're");
    assert_eq!(crate::r_s26_common::mv(&mut t, spell), 2);
}

#[test]
fn there_theyre_their_naming_one_name() {
    cr!("709.4a", "201.4b");
    ruling!(
        "There // They're // Their",
        "This split card has three names. If an effect instructs you to choose a card name, you may choose one of those names, but not all of them (though you might want to write it down and not just say it out loud)."
    );
    there_halves_supported();
    assert!(mtg_engine::choices::valid_card_name("They're", None));
    assert!(mtg_engine::choices::valid_card_name("Their", None));
    assert!(!mtg_engine::choices::valid_card_name(THERE, None));
    // Meddling Mage naming "They're": They're can't be cast; There can.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(P1, DecisionKind::Name, Answer::Text("They're".into()));
    t.enter(P1, "Meddling Mage");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::U, 2);
    let card = t.hand(P0, THERE);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert!(t.cast(P0, card).method(CastMethod::Half(1)).try_go().is_err());
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert!(t.cast(P0, card).method(CastMethod::Half(0)).try_go().is_ok());
}

/// P0's Grizzly Bears enchanted with Soul-Strike Technique dies: the top card of P0's
/// library (`top`) is manifested. Returns the face-down permanent.
fn soul_strike_manifest(t: &mut TestGame, top: &str) -> ObjectId {
    supported("Soul-Strike Technique");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Soul-Strike Technique");
    t.g.attach(aura, Entity::Object(bears));
    t.g.recompute();
    t.library_top(P0, top);
    let face_down = |t: &TestGame| -> Vec<ObjectId> {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.face_down)
            .map(|o| o.id)
            .collect()
    };
    let before = face_down(t);
    crate::r_s02_common::destroy(t, bears);
    t.resolve_all();
    let fd: Vec<ObjectId> = face_down(t)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect();
    assert_eq!(fd.len(), 1);
    fd[0]
}

#[test]
fn turning_a_manifested_permanent_face_up_keeps_it_the_same() {
    cr!("708.8", "701.40b");
    ruling!(
        "Soul-Strike Technique",
        "A permanent that turns face up changes characteristics but is otherwise the same permanent. Turning a permanent face up doesn’t change whether that permanent is tapped or untapped. Spells and abilities that were targeting that permanent still target it, and Auras and Equipment that were attached to the permanent are still attached to it. If anything now targets or is attached to the creature illegally, the game rules will clean this up as appropriate."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let m = soul_strike_manifest(&mut t, "Hill Giant");
    t.g.tap(m);
    let aura = t.battlefield(P0, "Soul-Strike Technique");
    t.g.attach(aura, Entity::Object(m));
    t.g.recompute();
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 3);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, m));
    t.settle();
    assert!(t.g.is_live(m), "the same object");
    assert_eq!(t.obj(m).chars.name, "Hill Giant");
    assert!(t.obj(m).tapped);
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(m)));
    assert_eq!(t.pt(m), (4, 4));
}

#[test]
fn you_can_look_at_your_face_down_permanents() {
    cr!("708.5");
    ruling!(
        "Soul-Strike Technique",
        "At any time, you can look at a face-down permanent you control. You can’t look at face-down permanents you don’t control unless an effect instructs you to do so."
    );
    let mut t = TestGame::new(2);
    let m = soul_strike_manifest(&mut t, "Hill Giant");
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, m));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, m));
}

#[test]
fn face_down_creatures_have_no_name_or_creature_types() {
    cr!("708.2a", "201.4");
    ruling!(
        "Soul-Strike Technique",
        "Because face-down creatures don’t have a name, they can’t have the same name as any other creature or share any creature types with any other creature, even another face-down creature."
    );
    let mut t = TestGame::new(2);
    let a = soul_strike_manifest(&mut t, "Grizzly Bears");
    let b = soul_strike_manifest(&mut t, "Grizzly Bears");
    for x in [a, b] {
        let c = &t.obj_now(x).chars;
        assert!(c.name.is_empty());
        assert!(c.subtypes.is_empty());
    }
}

#[test]
fn a_manifested_card_returned_to_the_battlefield_is_face_up() {
    cr!("400.7", "400.4a");
    ruling!(
        "Soul-Strike Technique",
        "If an effect tries to return a face-down creature to the battlefield after it leaves (such as Astral Drift’s delayed triggered ability), that effect returns the card face up. If it tries to put an instant or sorcery card onto the battlefield this way, that card remains in its current zone instead."
    );
    // A creature card: Cloudshift returns it face up.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let m = soul_strike_manifest(&mut t, "Hill Giant");
    cast_new(&mut t, P0, "Cloudshift", &[Entity::Object(m)]);
    t.resolve_all();
    let giants = t.named_on_battlefield("Hill Giant");
    assert_eq!(giants.len(), 1);
    assert!(!t.obj(giants[0]).face_down);
    // An instant card: it stays in exile.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let m = soul_strike_manifest(&mut t, "Giant Growth");
    cast_new(&mut t, P0, "Cloudshift", &[Entity::Object(m)]);
    t.resolve_all();
    assert!(t.in_exile("Giant Growth"));
    assert!(t.named_on_battlefield("Giant Growth").is_empty());
}

#[test]
fn aangs_delayed_trigger_at_the_next_upkeep_of_any_player() {
    cr!("603.7", "503.1");
    ruling!(
        "Aang, at the Crossroads // Aang, Destined Savior",
        "Aang, at the Crossroads's delayed triggered ability triggers at the beginning of the next upkeep regardless of whose turn it is."
    );
    supported("Aang, at the Crossroads // Aang, Destined Savior");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let aang = t.battlefield(P0, "Aang, at the Crossroads // Aang, Destined Savior");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.obj_now(aang).chars.name, "Aang, at the Crossroads");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.obj_now(aang).chars.name, "Aang, Destined Savior");
}

/// Casts Rebuild the City targeting `land`; returns the new tokens.
fn rebuild(t: &mut TestGame, land: ObjectId) -> Vec<ObjectId> {
    supported("Rebuild the City");
    t.set_step(P0, Step::PrecombatMain);
    let before = crate::r_s01_common::tokens(t, P0);
    for (ty, n) in [
        (ManaType::B, 1),
        (ManaType::R, 1),
        (ManaType::G, 1),
        (ManaType::C, 3),
    ] {
        mana(t, P0, ty, n);
    }
    let card = t.hand(P0, "Rebuild the City");
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.cast(P0, card).go();
    t.resolve_all();
    crate::r_s01_common::tokens(t, P0)
        .into_iter()
        .filter(|x| !before.contains(x) && t.obj(*x).chars.card_types.contains(CardType::Land))
        .collect()
}

#[test]
fn rebuild_the_city_copies_have_the_lands_entry_abilities() {
    cr!("707.2", "603.6a", "614.12");
    ruling!(
        "Rebuild the City",
        "Any enters-the-battlefield abilities of the copied land will trigger when the token enters the battlefield. Any \"as [this land] enters the battlefield\" or \"[this land] enters the battlefield with\" abilities of the copied land will also work."
    );
    let mut t = TestGame::new(2);
    // Khalni Garden: "This land enters tapped. When this land enters, create a 0/1 green
    // Plant creature token."
    let garden = t.battlefield(P0, "Khalni Garden");
    let toks = rebuild(&mut t, garden);
    assert_eq!(toks.len(), 3);
    assert!(toks.iter().all(|x| t.obj(*x).tapped));
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Plant").len(), 3);
}

#[test]
fn rebuild_the_city_copies_only_the_printed_land() {
    cr!("707.2", "707.9b");
    ruling!(
        "Rebuild the City",
        "Each of the tokens copy exactly what was printed on the original land and nothing else (unless that land is copying something else or is a token; see below). It doesn't copy whether that land is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its types, color, and so on. However, if the original land has an ability that says it enters the battlefield tapped, the copies will also enter tapped."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.g.tap(forest);
    crate::r_s29_common::put_counters(&mut t, forest, "+1/+1", 2);
    crate::r_s26_common::modify_until_eot(
        &mut t,
        forest,
        vec![Modification::AddTypes(vec![CardType::Artifact])],
    );
    let toks = rebuild(&mut t, forest);
    assert_eq!(toks.len(), 3);
    for x in toks {
        let o = t.obj(x);
        assert!(!o.tapped);
        assert_eq!(t.counters(x, "+1/+1"), 0);
        assert!(!o.chars.card_types.contains(CardType::Artifact));
        assert!(o.chars.has_subtype("Forest"));
        assert!(o.chars.card_types.contains(CardType::Creature));
        assert_eq!(t.pt(x), (3, 3));
        assert!(crate::r_s06_common::has_kw(&t, x, KeywordKind::Vigilance));
        assert!(crate::r_s06_common::has_kw(&t, x, KeywordKind::Menace));
    }
}

#[test]
fn rebuild_the_city_copying_a_token_uses_its_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Rebuild the City",
        "If the copied land is a token, the token that's created copies the original characteristics of that token as stated by the effect that created that token."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let first = rebuild(&mut t, forest);
    // A non-copy effect on the token isn't copied.
    crate::r_s26_common::modify_until_eot(
        &mut t,
        first[0],
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flying))],
    );
    let second = rebuild(&mut t, first[0]);
    assert_eq!(second.len(), 3);
    for x in second {
        let o = t.obj(x);
        assert!(o.chars.has_subtype("Forest"));
        assert_eq!(t.pt(x), (3, 3));
        assert!(!crate::r_s06_common::has_kw(&t, x, KeywordKind::Flying));
    }
}

#[test]
fn rebuild_the_city_copying_a_land_that_copies_something() {
    cr!("707.3");
    ruling!(
        "Rebuild the City",
        "If the copied land is copying something else, then the token enters the battlefield as whatever that land copied."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Azorius Guildgate");
    let stage = t.battlefield(P0, "Thespian's Stage");
    let gate = t.named_on_battlefield("Azorius Guildgate")[0];
    // "{2}, {T}: This land becomes a copy of target land, except it has this ability."
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, stage, 1, &[Entity::Object(gate)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(stage).chars.name, "Azorius Guildgate");
    let toks = rebuild(&mut t, stage);
    assert_eq!(toks.len(), 3);
    for x in toks {
        assert_eq!(t.obj(x).chars.name, "Azorius Guildgate");
    }
}

#[test]
fn three_blind_mice_copying_a_token_that_copies_something() {
    cr!("707.3");
    ruling!(
        "Three Blind Mice",
        "If the copied token is copying something else, then the token enters the battlefield as whatever that token copied."
    );
    supported("Three Blind Mice");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    crate::r_s17_common::become_copy(&mut t, tok, giant);
    assert_eq!(t.obj_now(tok).chars.name, "Hill Giant");
    let mice = t.battlefield(P0, "Three Blind Mice");
    crate::r_s29_common::put_counters(&mut t, mice, "lore", 1);
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(tok)]);
    crate::r_s29_common::put_counters(&mut t, mice, "lore", 1);
    t.resolve_all();
    let giants: Vec<ObjectId> = t
        .named_on_battlefield("Hill Giant")
        .into_iter()
        .filter(|x| t.obj(*x).controller == P0)
        .collect();
    assert_eq!(giants.len(), 2, "the original token and its copy");
}

/// A Planechase game with `plane` as the starting plane (P0 is the planar controller).
fn on_plane(plane: &str) -> TestGame {
    if plane == "Esper" {
        // Esper's chaos ability doesn't compile yet; only its first (static) ability is
        // used here.
        assert_eq!(mtg_engine::card::card(plane).unsupported_text().len(), 1);
    } else {
        supported(plane);
    }
    let mut t = crate::r_s19_common::planechase_game(2);
    crate::r_s19_common::start_planar_deck(&mut t, P0, &[plane]);
    t.set_step(P0, Step::PrecombatMain);
    t
}

#[test]
fn esper_reduces_the_cost_not_the_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Esper",
        "Esper’s first ability doesn’t change the mana cost or mana value of any artifact spell. It changes only the total cost players pay."
    );
    let mut t = on_plane("Esper");
    mana(&mut t, P0, ManaType::C, 1);
    let stone = t.hand(P0, "Mind Stone");
    let spell = t.cast(P0, stone).go();
    assert_eq!(crate::r_s26_common::mv(&mut t, spell), 2);
    assert_eq!(format!("{}", t.obj(spell).chars.mana_cost.clone().unwrap()), "{2}");
}

#[test]
fn esper_x_is_chosen_before_the_reduction() {
    cr!("601.2b", "601.2f");
    ruling!(
        "Esper",
        "If a spell you cast has {X} in its mana cost, you choose the value of X before calculating the spell’s total cost. For example, if an artifact spell’s mana cost is {X}{X}, you could choose 2 as the value of X and pay {3} to cast the spell."
    );
    let mut t = on_plane("Esper");
    mana(&mut t, P0, ManaType::C, 3);
    let chalice = t.hand(P0, "Chalice of the Void");
    t.cast(P0, chalice).x(2).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    let c = t.named_on_battlefield("Chalice of the Void")[0];
    assert_eq!(t.counters(c, "charge"), 2);
}

#[test]
fn esper_increases_apply_before_the_reduction() {
    cr!("601.2f");
    ruling!(
        "Esper",
        "If there are additional costs to cast an artifact spell, or if the cost to cast a spell is increased by an effect (such as the one created by Thalia, Guardian of Thraben’s ability), apply those increases before applying cost reductions."
    );
    let mut t = on_plane("Esper");
    // Thalia: "Noncreature spells cost {1} more to cast." Accorder's Shield costs {0}:
    // {0} + {1} - {1} = {0}.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let shield = t.hand(P0, "Accorder's Shield");
    assert!(t.cast(P0, shield).try_go().is_ok());
}

#[test]
fn gavony_indestructible_stays_after_a_control_change() {
    cr!("611.2c", "311.7");
    ruling!(
        "Gavony",
        "If another player gains control of one of your creatures after the chaos ability resolves, that creature will continue to have indestructible until end of turn."
    );
    let mut t = on_plane("Gavony");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s19_common::chaos(&mut t, P0);
    t.resolve_all();
    assert!(crate::r_s06_common::has_kw(&t, bears, KeywordKind::Indestructible));
    crate::r_s06_common::give_control(&mut t, bears, P1);
    assert!(crate::r_s06_common::has_kw(&t, bears, KeywordKind::Indestructible));
}

#[test]
fn higure_first_strike_search_then_ninjutsu() {
    cr!("702.49a", "510.4", "702.7b");
    ruling!(
        "Higure, the Still Wind",
        "If Higure deals first strike damage (or the first part of double strike damage), the triggered ability would resolve before regular combat damage, and the searched-for Ninja would be able to get swapped with an unblocked creature and still deal its regular damage."
    );
    supported("Higure, the Still Wind");
    let mut t = TestGame::new(2);
    let higure = t.battlefield(P0, "Higure, the Still Wind");
    crate::r_s26_common::modify_until_eot(
        &mut t,
        higure,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::FirstStrike))],
    );
    let giant = t.battlefield(P0, "Hill Giant");
    t.library_top(P0, "Ninja of the Deep Hours");
    crate::r_s01_common::attack_with(
        &mut t,
        &[(higure, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.answer_yes(P0, true);
    crate::r_s22_common::choose_named_when_offered(&mut t, P0, "Ninja of the Deep Hours");
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P0, "Ninja of the Deep Hours"));
    let ninja = t.g.player(P0).hand.iter().copied().find(|c| t.obj(*c).chars.name == "Ninja of the Deep Hours").unwrap();
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, ninja, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    t.advance_to(P0, Step::EndOfCombat);
    // Higure's first-strike 3, then the Ninja's regular 2.
    assert_eq!(t.life(P1), 15);
}
