//! Rulings batch P166 — putting Auras and Equipment onto the battlefield attached
//! (CR 303.4f, 301.5c), casting them from other zones (Danitha, Armory Paladin), and
//! abilities that move Equipment whose targets become illegal.

use crate::r_p125_common::at_p1;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use crate::r_s21_common::castable;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Danitha, Benalia's Hope
// ---------------------------------------------------------------------------------------

/// Danitha enters with `card` in P0's hand, and leaves before her trigger resolves.
fn danitha_gone(card: &str) -> (TestGame, ObjectId) {
    supported("Danitha, Benalia's Hope");
    let mut t = TestGame::new(2);
    let c = t.hand(P0, card);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(c)]);
    let danitha = t.enter(P0, "Danitha, Benalia's Hope");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, danitha, Zone::Exile);
    t.resolve_all();
    (t, c)
}

#[test]
fn danitha_gone_equipment_enters_unattached() {
    cr!("301.5c", "608.2b");
    ruling!(
        "Danitha, Benalia's Hope",
        "Conversely, if you attempt to put a non-Aura Equipment onto the battlefield this way and Danitha is no longer on the battlefield or that Equipment can't legally be attached to Danitha, that Equipment will enter the battlefield unattached."
    );
    let (t, c) = danitha_gone("Bonesplitter");
    let c = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert_eq!(attached_to(&t, c), None);
}

#[test]
fn danitha_gone_aura_stays_where_it_was() {
    cr!("303.4g", "608.2b");
    ruling!(
        "Danitha, Benalia's Hope",
        "If you attempt to put an Aura onto the battlefield this way and Danitha is no longer on the battlefield or that Aura can't legally be attached to Danitha, that Aura does not enter the battlefield at all. Instead, it will stay in its previous zone."
    );
    let (t, _) = danitha_gone("Holy Strength");
    assert!(t.in_hand(P0, "Holy Strength"));
    assert!(t.named_on_battlefield("Holy Strength").is_empty());
}

// ---------------------------------------------------------------------------------------
// Danitha, New Benalia's Light
// ---------------------------------------------------------------------------------------

#[test]
fn danitha_new_card_in_graveyard_can_be_cast_first() {
    cr!("117.1a", "601.3");
    ruling!(
        "Danitha, New Benalia's Light",
        "If an Aura or Equipment card is put into your graveyard during your main phase and the stack is empty, you have a chance to cast it before any player may attempt to remove that card from your graveyard."
    );
    supported("Danitha, New Benalia's Light");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Danitha, New Benalia's Light");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = attach_new(&mut t, P0, "Short Sword", bears);
    t.set_step(P0, Step::PrecombatMain);
    destroy(&mut t, sword);
    assert!(t.in_graveyard(P0, "Short Sword"));
    // Stack empty in P0's main phase: P0 gets priority and may cast it.
    assert_eq!(t.stack_len(), 0);
    t.lands(P0, "Wastes", 1);
    let card = t.g.find_in_zone(Zone::Graveyard(P0), "Short Sword")[0];
    assert!(castable(&mut t, P0, card));
}

#[test]
fn danitha_new_other_permissions_dont_use_hers() {
    cr!("601.2", "702.138a");
    ruling!(
        "Danitha, New Benalia's Light",
        "If you cast a spell from your graveyard using another permission, Danitha's effect doesn't apply. You can cast another Aura or Equipment spell from your graveyard."
    );
    supported("Mogis's Favor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Danitha, New Benalia's Light");
    let giant = t.battlefield(P0, "Hill Giant");
    let favor = t.graveyard(P0, "Mogis's Favor");
    t.graveyard(P0, "Opt");
    t.graveyard(P0, "Opt");
    let strength = t.graveyard(P0, "Holy Strength");
    let pacifism = t.graveyard(P0, "Pacifism");
    // Mogis's Favor with escape ({2}{B}, exile two other cards: the Opts).
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    let opts = t.g.find_in_zone(Zone::Graveyard(P0), "Opt");
    t.answer_choose(P0, &[Entity::Object(opts[0]), Entity::Object(opts[1])]);
    t.cast(P0, favor)
        .method(CastMethod::Keyword(KeywordKind::Escape))
        .target(Entity::Object(giant))
        .go();
    t.resolve_all();
    assert!(!t.named_on_battlefield("Mogis's Favor").is_empty());
    // Danitha's permission is still available: Holy Strength.
    t.lands(P0, "Plains", 1);
    assert!(castable(&mut t, P0, strength));
    t.cast(P0, strength).target(Entity::Object(giant)).go();
    t.resolve_all();
    assert!(!t.named_on_battlefield("Holy Strength").is_empty());
    // And now it's used.
    t.lands(P0, "Plains", 2);
    assert!(!castable(&mut t, P0, pacifism));
}

#[test]
fn danitha_new_a_new_danitha_gives_another_cast() {
    cr!("400.7", "601.2");
    ruling!(
        "Danitha, New Benalia's Light",
        "If you cast one Aura or Equipment spell from your graveyard and then have a new Danitha come under your control in the same turn, you may cast another Aura or Equipment spell from your graveyard that turn."
    );
    let mut t = TestGame::new(2);
    let danitha = t.battlefield(P0, "Danitha, New Benalia's Light");
    let giant = t.battlefield(P0, "Hill Giant");
    let strength = t.graveyard(P0, "Holy Strength");
    let sword = t.graveyard(P0, "Short Sword");
    t.lands(P0, "Plains", 1);
    t.cast(P0, strength).target(Entity::Object(giant)).go();
    t.resolve_all();
    t.lands(P0, "Wastes", 1);
    assert!(!castable(&mut t, P0, sword));
    // A new Danitha.
    move_to(&mut t, danitha, Zone::Exile);
    t.battlefield(P0, "Danitha, New Benalia's Light");
    t.settle();
    assert!(castable(&mut t, P0, sword));
}

// ---------------------------------------------------------------------------------------
// Moving Equipment
// ---------------------------------------------------------------------------------------

#[test]
fn vulshok_battlemaster_leaves_equipment_that_cant_equip_it() {
    cr!("301.5c", "702.16c");
    ruling!(
        "Vulshok Battlemaster",
        "If an Equipment can't equip Vulshok Battlemaster, it isn't attached to the Battlemaster, and it doesn't become unattached (if it's attached to a creature)."
    );
    supported("Vulshok Battlemaster");
    supported("Apostle's Blessing");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    let bm = t.enter(P0, "Vulshok Battlemaster");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response: protection from artifacts.
    let blessing = in_hand_with_mana(&mut t, P0, "Apostle's Blessing");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, blessing).target(Entity::Object(bm)).go();
    t.resolve();
    assert!(t.obj_now(bm).chars.has_keyword(KeywordKind::Protection));
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
}

#[test]
fn brass_squire_does_nothing_if_a_target_is_illegal() {
    cr!("608.2b", "115.1");
    ruling!(
        "Brass Squire",
        "If one of the targets is illegal when the activated ability resolves, nothing will happen and the Equipment won't move. Notably, if an opponent gains control of either target in response, the Equipment won't move."
    );
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, "Brass Squire");
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", thopter);
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, squire, "Attach").unwrap();
    give_control(&mut t, bears, P1);
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(thopter)));
}

#[test]
fn yuffie_illegal_target_means_no_attaching() {
    cr!("608.2b");
    ruling!(
        "Yuffie, Materia Hunter",
        "If the target of Yuffie's last ability is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't attach any Equipment to Yuffie."
    );
    supported("Yuffie, Materia Hunter");
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Short Sword");
    let splitter = t.battlefield(P1, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(sword)]);
    t.enter(P0, "Yuffie, Materia Hunter");
    t.settle();
    move_to(&mut t, splitter, Zone::Exile);
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), None);
}

#[test]
fn yuffie_gone_means_no_control() {
    cr!("611.2b", "608.2h");
    ruling!(
        "Yuffie, Materia Hunter",
        "If you lose control of Yuffie or it leaves the battlefield before its last ability resolves, you won't gain control of the target noncreature artifact."
    );
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P1, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    let yuffie = t.enter(P0, "Yuffie, Materia Hunter");
    t.settle();
    move_to(&mut t, yuffie, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.obj_now(splitter).controller, P1);
    // Otherwise P0 gains it.
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P1, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    t.enter(P0, "Yuffie, Materia Hunter");
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(splitter).controller, P0);
}

// ---------------------------------------------------------------------------------------
// Armory Paladin
// ---------------------------------------------------------------------------------------

fn paladin_game() -> TestGame {
    supported("Armory Paladin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Armory Paladin");
    stack_library(&mut t, P0, &["Short Sword", "Opt", "Hill Giant"]);
    let splitter = in_hand_with_mana(&mut t, P0, "Bonesplitter");
    t.cast(P0, splitter).go();
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Short Sword"));
    t
}

#[test]
fn armory_paladin_casting_the_exiled_equipment_triggers_again() {
    cr!("603.2", "601.2i");
    ruling!(
        "Armory Paladin",
        "If the card exiled with Armory Paladin’s last ability is an Aura or Equipment card, casting it will cause Armory Paladin’s last ability to trigger again."
    );
    let mut t = paladin_game();
    let sword = t.g.find_in_zone(Zone::Exile, "Short Sword")[0];
    t.lands(P0, "Wastes", 1);
    t.cast(P0, sword).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "exile the top card"), 1);
    t.resolve_all();
    assert!(t.in_exile("Opt"));
}

#[test]
fn armory_paladin_leaving_doesnt_end_the_permission() {
    cr!("610.3", "611.2a");
    ruling!(
        "Armory Paladin",
        "Once Armory Paladin’s last ability resolves, it doesn’t matter what happens to Armory Paladin after that. You’ll still be able to play the exiled card until the end of your next turn."
    );
    let mut t = paladin_game();
    let paladin = t.named_on_battlefield("Armory Paladin")[0];
    move_to(&mut t, paladin, Zone::Exile);
    let sword = t.g.find_in_zone(Zone::Exile, "Short Sword")[0];
    t.lands(P0, "Wastes", 1);
    assert!(castable(&mut t, P0, sword));
}

// ---------------------------------------------------------------------------------------
// Armored Skyhunter
// ---------------------------------------------------------------------------------------

#[test]
fn armored_skyhunter_auras_dont_target_but_need_something_to_enchant() {
    cr!("303.4f", "303.4a");
    ruling!(
        "Armored Skyhunter",
        "An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, you can't choose to put it onto the battlefield at all."
    );
    supported("Armored Skyhunter");
    supported("Animate Dead");
    // Pacifism onto P1's hexproof Gladecover Scout.
    let mut t = TestGame::new(2);
    let sky = t.battlefield(P0, "Armored Skyhunter");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let lib = stack_library(
        &mut t,
        P0,
        &["Pacifism", "Forest", "Forest", "Forest", "Forest", "Forest"],
    );
    t.answer_choose(P0, &[Entity::Object(lib[0])]);
    t.answer_choose(P0, &[Entity::Object(scout)]);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[sky]));
    t.resolve_all();
    let p = t.named_on_battlefield("Pacifism");
    assert_eq!(p.len(), 1);
    assert_eq!(attached_to(&t, p[0]), Some(Entity::Object(scout)));
    // Animate Dead ("Enchant creature card in a graveyard") with no creature card in any
    // graveyard can't be chosen.
    let mut t = TestGame::new(2);
    let sky = t.battlefield(P0, "Armored Skyhunter");
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Animate Dead",
            "Forest",
            "Forest",
            "Forest",
            "Forest",
            "Forest",
        ],
    );
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(lib[0])]);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[sky]));
    t.resolve_all();
    assert!(t.named_on_battlefield("Animate Dead").is_empty());
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!offered.contains(&Entity::Object(lib[0])));
}
