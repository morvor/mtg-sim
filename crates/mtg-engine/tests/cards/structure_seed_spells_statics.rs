//! Structure-coverage seed tests: spell, static-ability and keyword structures shared by
//! many cards (see `docs/STRUCTURE_COVERAGE.md`). Each test checks a real card's ability
//! does what its text says.

use super::structure_seed_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the activated ability of `src` whose text contains `text` (case-insensitive).
fn activate_text(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    text: &str,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let acts: Vec<_> = t
        .obj_now(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.to_lowercase())
        .collect();
    let i = acts
        .iter()
        .position(|a| a.contains(&text.to_lowercase()))
        .unwrap_or_else(|| panic!("no activated ability with {text:?} among {acts:?}"));
    t.activate(p, src, i, &[])
}

// ---------------------------------------------------------------------------
// Spells
// ---------------------------------------------------------------------------

#[test]
fn tread_upon_pumps_and_grants_trample() {
    cr!("608.2", "611.2a");
    // "Target creature gets +2/+2 and gains trample until end of turn."
    supported("Tread Upon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Tread Upon");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Trample));
}

#[test]
fn impeccable_timing_damages_an_attacking_creature() {
    cr!("608.2", "506.4", "120.3e");
    // "Impeccable Timing deals 3 damage to target attacking or blocking creature."
    supported("Impeccable Timing");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P1, "Hill Giant");
    let idle = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Impeccable Timing");
    t.set_step(P1, Step::BeginningOfCombat);
    declare(&mut t, &[(attacker, Entity::Player(P0))]);
    go_to(&mut t, Step::DeclareAttackers);
    // A creature that isn't attacking or blocking isn't a legal target: the requested
    // target is replaced by the only legal one.
    t.cast(P0, s).target(idle).go();
    t.resolve_all();
    assert!(!t.on_battlefield(attacker));
    assert!(t.on_battlefield(idle));
}

#[test]
fn unfortunate_accident_spree_destroys_a_creature() {
    cr!("702.172a", "700.2h");
    // "Spree / + {2}{B} — Destroy target creature. / + {1} — Create a 1/1 red Mercenary
    // creature token ..."
    supported("Unfortunate Accident");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Unfortunate Accident");
    // {B} plus the chosen mode's {2}{B}.
    t.cast(P0, s).modes(&[0]).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(subtype_ids(&t, P0, "Mercenary").len(), 0);
    let untapped = t
        .g
        .battlefield
        .iter()
        .filter(|id| t.obj_now(**id).chars.name == "Swamp" && !t.obj_now(**id).tapped)
        .count();
    assert_eq!(untapped, 0, "the spree cost was paid along with the mana cost");
}

// ---------------------------------------------------------------------------
// Static abilities and enchant
// ---------------------------------------------------------------------------

#[test]
fn bladed_ambassador_enters_with_an_oil_counter() {
    cr!("614.1c", "122.6");
    // "This creature enters with an oil counter on it."
    supported("Bladed Ambassador");
    let mut t = TestGame::new(2);
    let b = t.enter(P0, "Bladed Ambassador");
    assert_eq!(t.counters(b, "oil"), 1);
}

#[test]
fn urban_utopia_enchants_a_land_and_grants_a_mana_ability() {
    cr!("702.5a", "303.4a", "613.1f");
    // "Enchant land / When this Aura enters, draw a card. / Enchanted land has "{T}: Add
    // one mana of any color.""
    supported("Urban Utopia");
    let mut t = TestGame::new(2);
    let forests = t.lands(P0, "Forest", 2);
    let aura = t.hand(P0, "Urban Utopia");
    let target = t.lands(P0, "Wastes", 1)[0];
    t.cast(P0, aura).target(target).go();
    t.resolve_all();
    let aura = t.named_on_battlefield("Urban Utopia")[0];
    assert_eq!(
        t.obj_now(aura).attached_to,
        Some(Entity::Object(target)),
        "attached to the land"
    );
    assert_eq!(t.hand_size(P0), 1, "drew a card");
    let _ = forests;
    // The Aura may have been paid for with the target land: untap it.
    t.g.objects[target.0 as usize].tapped = false;
    activate_text(&mut t, P0, target, "any color").unwrap();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.mana.len(), 1);
    assert_eq!(pool.count(ManaType::C), 0, "colored mana");
    assert!(t.obj_now(target).tapped);
}

#[test]
fn boon_of_emrakul_gives_plus_three_minus_three() {
    cr!("702.5a", "613.4c");
    // "Enchant creature / Enchanted creature gets +3/-3."
    supported("Boon of Emrakul");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let aura = t.hand(P0, "Boon of Emrakul");
    t.cast(P0, aura).target(wurm).go();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (9, 1));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn swashbuckling_gives_plus_two_and_haste() {
    cr!("702.5a", "613.4c", "613.1f");
    // "Enchant creature / Enchanted creature gets +2/+2 and has haste."
    supported("Swashbuckling");
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let aura = t.hand(P0, "Swashbuckling");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
}

#[test]
fn inferno_fist_enchants_only_a_creature_you_control_and_can_be_sacrificed() {
    cr!("702.5a", "303.4a", "602.2");
    // "Enchant creature you control / Enchanted creature gets +2/+0. / {R}, Sacrifice this
    // Aura: This Aura deals 2 damage to any target."
    supported("Inferno Fist");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let aura = t.hand(P0, "Inferno Fist");
    // The opponent's creature isn't a legal target: the requested target is replaced by
    // the only legal one.
    t.cast(P0, aura).target(theirs).go();
    t.resolve_all();
    assert_eq!(t.pt(mine), (4, 2));
    assert_eq!(t.pt(theirs), (2, 2));
    let fist = t.named_on_battlefield("Inferno Fist")[0];
    t.activate(P0, fist, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(mine), (2, 2));
}

#[test]
fn encrust_stops_activated_abilities_and_untapping() {
    cr!("702.5a", "502.3");
    // "Enchant artifact or creature / Enchanted permanent doesn't untap during its
    // controller's untap step and its activated abilities can't be activated."
    supported("Encrust");
    let mut t = TestGame::new(2);
    let officer = t.battlefield(P1, "Checkpoint Officer");
    t.lands(P0, "Island", 3);
    t.lands(P1, "Plains", 2);
    let aura = t.hand(P0, "Encrust");
    t.cast(P0, aura).target(officer).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Encrust").len() == 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.activate(P1, officer, 0, &[Entity::Object(bears)]).is_err());
    t.g.tap(officer);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(officer).tapped, "doesn't untap");
}

#[test]
fn ensoul_artifact_makes_an_artifact_a_5_5_creature() {
    cr!("702.5a", "613.4b", "613.1d");
    // "Enchant artifact / Enchanted artifact is a creature with base power and toughness
    // 5/5 in addition to its other types."
    supported("Ensoul Artifact");
    let mut t = TestGame::new(2);
    let ob = t.battlefield(P0, "Obelisk of Naya");
    t.lands(P0, "Island", 2);
    let aura = t.hand(P0, "Ensoul Artifact");
    t.cast(P0, aura).target(ob).go();
    t.resolve_all();
    assert!(t.obj_now(ob).is_creature());
    assert_eq!(t.pt(ob), (5, 5));
}

#[test]
fn take_possession_gains_control_of_enchanted_permanent() {
    cr!("702.5a", "613.1b");
    // "Split second / Enchant permanent / You control enchanted permanent."
    supported("Take Possession");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Island", 7);
    let aura = t.hand(P0, "Take Possession");
    t.cast(P0, aura).target(land).go();
    t.resolve_all();
    assert_eq!(t.obj_now(land).controller, P0);
}

#[test]
fn curse_of_bloodletting_enchants_a_player_and_doubles_damage() {
    cr!("702.5d", "303.4a", "614.1a");
    // "Enchant player / If a source would deal damage to enchanted player, it deals double
    // that damage to that player instead."
    supported("Curse of Bloodletting");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let curse = t.hand(P0, "Curse of Bloodletting");
    t.cast(P0, curse).target(P1).go();
    t.resolve_all();
    let curse = t.named_on_battlefield("Curse of Bloodletting")[0];
    assert_eq!(t.obj_now(curse).attached_to, Some(Entity::Player(P1)));
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[test]
fn frogmite_costs_one_less_for_each_artifact() {
    cr!("702.41a");
    // "Affinity for artifacts"
    supported("Frogmite");
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.battlefield(P0, "Ornithopter");
    }
    let f = t.hand(P0, "Frogmite");
    t.cast(P0, f).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Frogmite").len(), 1);
}

#[test]
fn fen_hauler_is_paid_for_by_tapping_artifacts() {
    cr!("702.126a");
    // "Improvise"
    supported("Fen Hauler");
    let mut t = TestGame::new(2);
    let thopters: Vec<ObjectId> = (0..6).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    t.lands(P0, "Swamp", 1);
    let f = t.hand(P0, "Fen Hauler");
    t.cast(P0, f).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Fen Hauler").len(), 1);
    assert!(thopters.iter().all(|o| t.obj_now(*o).tapped));
}

#[test]
fn baneslayer_angel_cant_be_blocked_by_dragons() {
    cr!("702.16f", "702.16g");
    // "protection from Demons and from Dragons"
    supported("Baneslayer Angel");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Baneslayer Angel");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(angel, Entity::Player(P1))], &[(dragon, angel)]);
    assert_eq!(t.life(P1), 15, "unblocked");
    assert_eq!(t.life(P0), 25, "lifelink");
}

#[test]
fn mountain_goat_cant_be_blocked_while_defender_controls_a_mountain() {
    cr!("702.14c");
    // "Mountainwalk"
    supported("Mountain Goat");
    let mut t = TestGame::new(2);
    let goat = t.battlefield(P0, "Mountain Goat");
    let wall = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Mountain");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(goat, Entity::Player(P1))], &[(wall, goat)]);
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(goat));
}

#[test]
fn typecycling_searches_for_the_basic_land_type() {
    cr!("702.29a", "702.29e");
    // "Plainscycling {2}", "Forestcycling {2}"
    for (card, land) in [("Eternal Dragon", "Plains"), ("Elvish Aberration", "Forest")] {
        supported(card);
        let mut t = TestGame::new(2);
        let c = t.hand(P0, card);
        let wanted = t.library_top(P0, land);
        t.library_top(P0, "Island");
        t.lands(P0, "Wastes", 2);
        t.answer_choose(P0, &[Entity::Object(wanted)]);
        activate_text(&mut t, P0, c, "cycling").unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, card), "{card} discarded");
        assert!(t.in_hand(P0, land), "{card} found a {land}");
        assert!(!t.in_hand(P0, "Island"));
    }
}
