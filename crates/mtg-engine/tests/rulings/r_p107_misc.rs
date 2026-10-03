//! Rulings batch P107 — assorted interactions: cumulative replacement effects and block
//! permissions, token copies of tokens, entering as a copy of nothing, "choose a color /
//! creature type", playing exiled lands, Saga chapters with divided counters, casting
//! during resolution, and what counts as "modified" (CR 700.9).

use crate::r_p107_common::*;
use crate::r_p125_common::options_asked;
use crate::r_s19_common::add_lore;
use crate::r_s21_common::legal_blocks;
use crate::r_s29_common::divide;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn gain(t: &mut TestGame, p: PlayerId, n: i32) {
    use mtg_engine::ability::{Effect, PlayerRef, Value};
    t.g.recompute();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::GainLife {
            who: PlayerRef::You,
            n: Value::Const(n),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

#[test]
fn two_phials_of_galadriel_quadruple_life_gain() {
    cr!("616.1", "614.5");
    ruling!(
        "Phial of Galadriel",
        "The second abilities of multiple Phials of Galadriel are cumulative. If you control two, life gained while you have 5 or less life will be multiplied by 4."
    );
    // Mirror Gallery lets P0 keep two of the legendary Phials.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Phial of Galadriel");
    t.battlefield(P0, "Phial of Galadriel");
    t.g.players[0].life = 5;
    gain(&mut t, P0, 1);
    assert_eq!(t.life(P0), 9);
    // At 9 life, no doubling.
    gain(&mut t, P0, 1);
    assert_eq!(t.life(P0), 10);
    // Three: times 8.
    t.battlefield(P0, "Phial of Galadriel");
    t.g.players[0].life = 3;
    gain(&mut t, P0, 1);
    assert_eq!(t.life(P0), 11);
}

#[test]
fn foriysian_totem_block_permissions_are_cumulative() {
    cr!("509.1b", "509.1c");
    ruling!(
        "Foriysian Totem",
        "The third ability normally lets Foriysian Totem block two creatures. If another effect also lets it block an additional creature, the effects are cumulative."
    );
    let mut t = TestGame::new(2);
    let totem = t.battlefield(P0, "Foriysian Totem");
    t.battlefield(P0, "Brave the Sands");
    let atk: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    t.advance_to(P1, Step::Upkeep);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 4);
    act(&mut t, P0, totem, "4/4", &[]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, totem));
    crate::r_s01_common::attack_with(
        &mut t,
        &atk.iter().map(|a| (*a, Entity::Player(P0))).collect::<Vec<_>>(),
    );
    let three: Vec<(ObjectId, ObjectId)> = atk[..3].iter().map(|a| (totem, *a)).collect();
    let four: Vec<(ObjectId, ObjectId)> = atk.iter().map(|a| (totem, *a)).collect();
    assert!(legal_blocks(&mut t, P0, &three));
    assert!(!legal_blocks(&mut t, P0, &four));
}

#[test]
fn city_of_death_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.4");
    ruling!(
        "City of Death",
        "The token you create copies the original characteristics of the token as stated by the effect that created that token"
    );
    // Chapter I makes a Treasure; a 1/1 Soldier token gets a counter, is tapped and pumped.
    let mut t = TestGame::new(2);
    let city = t.enter(P0, "City of Death");
    t.resolve_all();
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    put_counters(&mut t, soldier, counters::PLUS1, 2);
    t.g.tap(soldier);
    crate::r_p206_common::pump(&mut t, soldier, 3, 3);
    let before = tokens(&t, P0);
    t.answer_targets(P0, &[obj(soldier)]);
    add_lore(&mut t, city, 1);
    t.resolve_all();
    let new: Vec<ObjectId> = tokens(&t, P0)
        .into_iter()
        .filter(|id| !before.contains(id))
        .collect();
    assert_eq!(new.len(), 1);
    let copy = t.obj_now(new[0]);
    assert!(copy.chars.has_subtype("Soldier"));
    assert!(!copy.tapped);
    assert_eq!(t.counters(new[0], counters::PLUS1), 0);
    assert_eq!(t.pt(new[0]), (1, 1));
}

#[test]
fn copy_land_copying_nothing_is_just_an_enchantment() {
    cr!("707.5", "614.1c");
    ruling!(
        "Copy Land",
        "You can choose not to copy anything. In that case, Copy Land simply enters the battlefield as an enchantment"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Forest");
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    let cl = t.enter(P0, "Copy Land");
    t.settle();
    let o = t.obj_now(cl);
    assert_eq!(o.chars.name, "Copy Land");
    assert!(o.is(CardType::Enchantment));
    assert!(!o.is(CardType::Land));
}

#[test]
fn patchwork_banner_offers_only_creature_types() {
    cr!("205.3m");
    ruling!(
        "Patchwork Banner",
        "You must choose an existing creature type, such as Bird or Wizard. Card types such as artifact can't be chosen."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    crate::r_p125_common::choose_creature_type(&mut t, P0, "Bear");
    let from = t.asked().len();
    t.enter(P0, "Patchwork Banner");
    t.settle();
    let offered = options_asked(&t, P0, from);
    assert_eq!(offered.len(), 1);
    let opts = &offered[0].1;
    assert!(opts.iter().any(|o| o == "Wizard"));
    assert!(!opts.iter().any(|o| o == "Artifact" || o == "Treasure"));
    assert_eq!(t.pt(bear), (3, 3));
}

#[test]
fn heraldic_banner_offers_the_five_colors() {
    cr!("105.1", "105.2c");
    ruling!(
        "Heraldic Banner",
        "You must choose white, blue, black, red, or green for Heraldic Banner's ability."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    t.enter(P0, "Heraldic Banner");
    t.settle();
    let offered = options_asked(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].1.len(), 5);
}

const BIRGI: &str = "Birgi, God of Storytelling // Harnfel, Horn of Bounty";

/// P0's Harnfel, Horn of Bounty (cast as its back face), having exiled two lands with its
/// ability (discarding a Plains). Returns (Harnfel, the two exiled lands).
fn harnfel_exiles_two_lands(t: &mut TestGame) -> (ObjectId, Vec<ObjectId>) {
    mana(t, P0, ManaType::R, 1);
    mana(t, P0, ManaType::C, 4);
    let b = t.hand(P0, BIRGI);
    let spell = t.cast(P0, b).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let harnfel = t.g.current(spell);
    assert_eq!(t.obj(harnfel).chars.name, "Harnfel, Horn of Bounty");
    let lands = stack_library(t, P0, &["Forest", "Island"]);
    t.hand(P0, "Plains");
    act(t, P0, harnfel, "Exile the top two", &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Forest") && t.in_exile("Island"));
    (harnfel, lands)
}

#[test]
fn harnfel_lands_need_land_plays_and_survive_harnfel_leaving() {
    cr!("305.2", "305.3", "610.3");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "Unless an effect allows you to play additional lands that turn, you can play land cards exiled with Harnfel only if you haven’t played a land yet that turn."
    );
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "You may play those cards that turn even if Harnfel leaves the battlefield or another player gains control of it."
    );
    let mut t = TestGame::new(2);
    let (harnfel, lands) = harnfel_exiles_two_lands(&mut t);
    let forest = t.g.find_in_zone(Zone::Exile, "Forest")[0];
    let island = t.g.find_in_zone(Zone::Exile, "Island")[0];
    let _ = lands;
    destroy(&mut t, harnfel);
    assert!(!t.on_battlefield(harnfel));
    assert!(t.play_land(P0, forest).is_ok());
    assert!(t.play_land(P0, island).is_err());
    assert!(t.in_exile("Island"));
}

#[test]
fn the_grand_evolution_divides_counters_with_at_least_one_per_target() {
    cr!("601.2d", "115.1", "714.3b");
    ruling!(
        "Vorinclex // The Grand Evolution",
        "You choose how many targets the chapter II ability of The Grand Evolution has and how the counters are distributed as you put the ability onto the stack. Each target must receive at least one counter."
    );
    let mut t = TestGame::new(2);
    let saga = crate::r_s17_common::enter_transformed(&mut t, P0, "Vorinclex // The Grand Evolution");
    t.resolve_all();
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[6, 1]);
    add_lore(&mut t, saga, 1);
    // The division is made as the ability goes on the stack: before it resolves, the
    // amounts are already fixed.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 6);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn perception_bobblehead_casts_during_resolution_ignoring_timing() {
    cr!("608.2g", "601.2");
    ruling!(
        "Perception Bobblehead",
        "You choose whether to cast a spell from among the cards you’re looking at as Perception Bobblehead’s last ability resolves."
    );
    // Activated in the opponent's turn, it casts the sorcery Lava Spike.
    let mut t = TestGame::new(2);
    let bob = t.battlefield(P0, "Perception Bobblehead");
    let spike = t.library_top(P0, "Lava Spike");
    t.advance_to(P1, Step::Upkeep);
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_choose(P0, &[obj(spike)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    act(&mut t, P0, bob, "Look at the top X", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Declined: the card goes to the bottom and can't be cast later.
    let mut t = TestGame::new(2);
    let bob = t.battlefield(P0, "Perception Bobblehead");
    t.library_top(P0, "Lava Spike");
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_choose(P0, &[]);
    t.answer_yes(P0, false);
    act(&mut t, P0, bob, "Look at the top X", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.g.find_in_zone(Zone::Library(P0), "Lava Spike").len(), 1);
    assert_ne!(
        t.g.player(P0).library.last().copied(),
        Some(t.g.find_in_zone(Zone::Library(P0), "Lava Spike")[0])
    );
}

#[test]
fn crystal_skull_lets_its_controller_look_at_the_top_card_any_time() {
    cr!("401.5");
    ruling!(
        "Crystal Skull, Isu Spyglass",
        "You can look at the top card of your library whenever you want (with one restriction; see below), even if you don’t have priority."
    );
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, top));
    t.battlefield(P0, "Crystal Skull, Isu Spyglass");
    t.g.recompute();
    t.set_step(P1, Step::DeclareAttackers);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, top));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
}

/// P1's Pacifism enchants `id` (an Aura controlled by another player).
fn p1_aura_on(t: &mut TestGame, id: ObjectId) {
    attach_new(t, P1, "Pacifism", id);
}

#[test]
fn an_opponents_aura_doesnt_make_a_creature_modified() {
    cr!("700.9");
    ruling!(
        "Envoy of the Ancestors",
        "An Aura controlled by another player does not cause a creature you control to be modified."
    );
    ruling!(
        "Temperamental Oozewagg",
        "An Aura controlled by another player does not cause a creature you control to be modified."
    );
    ruling!(
        "Red XIII, Proud Warrior",
        "An Aura controlled by another player does not cause a creature you control to be modified."
    );
    ruling!(
        "Obstinate Gargoyle",
        "An Aura controlled by another player does not cause a creature you control to be modified."
    );
    // "Modified creatures you control have <keyword>" (Envoy, Oozewagg), "Other modified
    // creatures you control have ..." (Red XIII), "~ has flying as long as it's modified"
    // (Obstinate Gargoyle).
    for (lord, kw) in [
        ("Envoy of the Ancestors", KeywordKind::Lifelink),
        ("Temperamental Oozewagg", KeywordKind::Trample),
        ("Red XIII, Proud Warrior", KeywordKind::Vigilance),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, lord);
        let bear = t.battlefield(P0, "Grizzly Bears");
        p1_aura_on(&mut t, bear);
        assert!(!has_kw(&t, bear, kw), "{lord}");
        // Its controller's own Aura (or an Equipment anyone controls) does.
        attach_new(&mut t, P1, "Bonesplitter", bear);
        assert!(has_kw(&t, bear, kw), "{lord}");
        let other = t.battlefield(P0, "Hill Giant");
        attach_new(&mut t, P0, "Holy Strength", other);
        assert!(has_kw(&t, other, kw), "{lord}");
    }
    let mut t = TestGame::new(2);
    let gargoyle = t.battlefield(P0, "Obstinate Gargoyle");
    p1_aura_on(&mut t, gargoyle);
    assert!(!has_kw(&t, gargoyle, KeywordKind::Flying));
    attach_new(&mut t, P0, "Holy Strength", gargoyle);
    assert!(has_kw(&t, gargoyle, KeywordKind::Flying));
}

#[test]
fn an_opponents_aura_doesnt_make_a_creature_a_modified_target_or_death() {
    cr!("700.9", "115.1");
    ruling!(
        "Lion Umbra",
        "An Aura controlled by another player does not cause a creature you control to be modified."
    );
    ruling!(
        "Guardian of the Forgotten",
        "An Aura controlled by another player does not cause a creature you control to be modified."
    );
    // Lion Umbra ("Enchant modified creature") can't target a creature that has only an
    // opponent's Aura.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    p1_aura_on(&mut t, bear);
    mana(&mut t, P0, ManaType::G, 2);
    let umbra = t.hand(P0, "Lion Umbra");
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, umbra, CastMethod::Normal));
    put_counters(&mut t, bear, counters::PLUS1, 1);
    assert!(crate::r_s02_common::can_cast(&mut t, P0, umbra, CastMethod::Normal));
    // Guardian of the Forgotten: "Whenever a modified creature you control dies,
    // manifest the top card of your library." Not for a creature with only P1's Aura.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian of the Forgotten");
    let bear = t.battlefield(P0, "Grizzly Bears");
    p1_aura_on(&mut t, bear);
    destroy(&mut t, bear);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert!(creatures(&t, P0).len() == 1);
    let bear = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", bear);
    destroy(&mut t, bear);
    t.resolve_all();
    assert_eq!(creatures(&t, P0).len(), 2);
}
