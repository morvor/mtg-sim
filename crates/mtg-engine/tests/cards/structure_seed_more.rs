//! Structure-coverage seed tests, continued: more structures shared by many cards (see
//! `docs/STRUCTURE_COVERAGE.md`). Each test checks a real card's ability does what its text
//! says.

use super::structure_seed_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The index (among activated abilities) of the ability of `src` whose text contains
/// `text`.
fn ability_index(t: &mut TestGame, src: ObjectId, text: &str) -> usize {
    t.g.recompute();
    let acts: Vec<String> = t
        .obj_now(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.to_lowercase())
        .collect();
    acts.iter()
        .position(|a| a.contains(&text.to_lowercase()))
        .unwrap_or_else(|| panic!("no activated ability with {text:?} among {acts:?}"))
}

#[test]
fn evermind_is_spliced_onto_an_arcane_spell() {
    cr!("702.47a", "702.47b");
    // "Splice onto Arcane {1}{U}" (Evermind: "Draw a card.")
    supported("Evermind");
    supported("Desperate Ritual");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    let evermind = t.hand(P0, "Evermind");
    let ritual = t.hand(P0, "Desperate Ritual");
    let _ = evermind;
    t.answer(P0, DecisionKind::OptionalCost, decision::Answer::Bool(true));
    t.cast(P0, ritual).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Evermind"), "a spliced card stays in hand");
    assert_eq!(t.hand_size(P0), 2, "Evermind and the card drawn");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 3);
}

#[test]
fn seraph_of_new_capenna_transforms_paying_phyrexian_mana_with_life() {
    cr!("701.27a", "107.4f");
    // "{4}{B/P}: Transform this creature. Activate only as a sorcery."
    supported("Seraph of New Capenna // Seraph of New Phyrexia");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seraph of New Capenna // Seraph of New Phyrexia");
    t.lands(P0, "Plains", 4);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "{{B/P}} paid with 2 life");
    assert_eq!(t.obj_now(s).chars.name, "Seraph of New Phyrexia");
}

#[test]
fn faithful_squire_gets_a_ki_counter_for_an_arcane_spell() {
    cr!("603.2", "122.6");
    // "Whenever you cast a Spirit or Arcane spell, you may put a ki counter on this
    // creature."
    supported("Faithful Squire // Kaiso, Memory of Loyalty");
    let mut t = TestGame::new(2);
    let sq = t.battlefield(P0, "Faithful Squire // Kaiso, Memory of Loyalty");
    t.lands(P0, "Mountain", 2);
    let ritual = t.hand(P0, "Desperate Ritual");
    t.answer_yes(P0, true);
    t.cast(P0, ritual).go();
    t.resolve_all();
    assert_eq!(t.counters(sq, "ki"), 1);
}

#[test]
fn fetid_heath_filters_hybrid_mana_into_two() {
    cr!("605.1a", "106.1a");
    // "{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}."
    supported("Fetid Heath");
    let mut t = TestGame::new(2);
    let heath = t.battlefield(P0, "Fetid Heath");
    t.lands(P0, "Plains", 1);
    let i = ability_index(&mut t, heath, "{w/b}");
    // Choose the third combination, {B}{B}.
    t.answer(P0, DecisionKind::Option, decision::Answer::Index(2));
    t.activate(P0, heath, i, &[]).unwrap();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.count(ManaType::B), 2);
    assert_eq!(pool.mana.len(), 2);
}

#[test]
fn cadaverous_bloom_adds_one_of_two_two_mana_combinations() {
    cr!("605.1a", "106.1a");
    // "Exile a card from your hand: Add {B}{B} or {G}{G}."
    supported("Cadaverous Bloom");
    let mut t = TestGame::new(2);
    let bloom = t.battlefield(P0, "Cadaverous Bloom");
    t.hand(P0, "Grizzly Bears");
    // Choose the second combination, {G}{G}.
    t.answer(P0, DecisionKind::Option, decision::Answer::Index(1));
    t.activate(P0, bloom, 0, &[]).unwrap();
    assert_eq!(t.hand_size(P0), 0, "the card was exiled as the cost");
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.count(ManaType::G), 2);
    assert_eq!(pool.mana.len(), 2);
}

#[test]
fn frenzied_rage_gives_plus_two_plus_one_and_menace() {
    cr!("702.5a", "613.4c", "702.111a");
    // "Enchant creature / Enchanted creature gets +2/+1 and has menace."
    supported("Frenzied Rage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let aura = t.hand(P0, "Frenzied Rage");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 3));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Menace));
    // A single blocker can't block it.
    let wall = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(wall, bears)]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn lonely_arroyo_deals_1_damage_to_target_opponent() {
    cr!("603.6a", "115.1");
    // "This land enters tapped. When this land enters, it deals 1 damage to target
    // opponent."
    supported("Lonely Arroyo");
    let mut t = TestGame::new(2);
    let land = t.enter(P0, "Lonely Arroyo");
    t.resolve_all();
    assert!(t.obj_now(land).tapped);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn rockfall_vale_enters_untapped_with_two_other_lands() {
    cr!("614.1d");
    // "This land enters tapped unless you control two or more other lands."
    supported("Rockfall Vale");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let first = t.enter(P0, "Rockfall Vale");
    assert!(t.obj_now(first).tapped, "only one other land");
    let second = t.enter(P0, "Rockfall Vale");
    assert!(!t.obj_now(second).tapped, "two other lands");
}

#[test]
fn spire_garden_enters_untapped_with_two_or_more_opponents() {
    cr!("614.1d", "102.2");
    // "This land enters tapped unless you have two or more opponents."
    supported("Spire Garden");
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Spire Garden");
    assert!(t.obj_now(a).tapped);
    let mut t = TestGame::new(3);
    let b = t.enter(P0, "Spire Garden");
    assert!(!t.obj_now(b).tapped);
}

#[test]
fn theorix_annex_enters_untapped_with_a_planeswalker() {
    cr!("614.1d");
    // "This land enters tapped unless you control a planeswalker."
    supported("Theorix Annex");
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Theorix Annex");
    assert!(t.obj_now(a).tapped);
    t.battlefield(P1, "Jace Beleren");
    let b = t.enter(P0, "Theorix Annex");
    assert!(t.obj_now(b).tapped, "an opponent's planeswalker doesn't count");
    t.battlefield(P0, "Jace Beleren");
    let c = t.enter(P0, "Theorix Annex");
    assert!(!t.obj_now(c).tapped);
}

#[test]
fn sanguine_indulgence_returns_up_to_two_creature_cards() {
    cr!("115.1", "601.2c");
    // "Return up to two target creature cards from your graveyard to your hand."
    supported("Sanguine Indulgence");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let c = t.graveyard(P0, "Craw Wurm");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Sanguine Indulgence");
    t.cast(P0, s)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    let _ = c;
}

#[test]
fn thrumming_hivepool_costs_less_for_each_sliver() {
    cr!("702.41a");
    // "Affinity for Slivers"
    supported("Thrumming Hivepool");
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.battlefield(P0, "Muscle Sliver");
    }
    t.lands(P0, "Wastes", 1);
    let h = t.hand(P0, "Thrumming Hivepool");
    t.cast(P0, h).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Thrumming Hivepool").len(), 1);
}

#[test]
fn wizards_staff_equips_a_wizard_for_1() {
    cr!("702.6a", "702.6c", "702.6d");
    // "Equip Wizard {1}"
    supported("Wizard's Staff");
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Wizard's Staff");
    let wizard = t.battlefield(P0, "Soothsayer Adept");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.set_step(P0, Step::PrecombatMain);
    // Its two equip abilities: only "Equip Wizard {1}" can be paid with one land.
    let activated = (0..2).any(|i| {
        t.clear_answers();
        t.activate(P0, staff, i, &[Entity::Object(wizard)]).is_ok()
    });
    assert!(activated);
    t.resolve_all();
    assert_eq!(t.obj_now(staff).attached_to, Some(Entity::Object(wizard)));
    assert!(t.obj_now(wizard).has_keyword(KeywordKind::Prowess));
}

#[test]
fn arwens_gift_scries_2_then_draws_2() {
    cr!("701.22a", "121.1");
    // "Scry 2, then draw two cards."
    supported("Arwen's Gift");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let s = t.hand(P0, "Arwen's Gift");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(scries(&t), vec![2]);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn horned_kavu_returns_a_red_or_green_creature_you_control() {
    cr!("603.6a", "608.2d");
    // "When this creature enters, return a red or green creature you control to its
    // owner's hand."
    supported("Horned Kavu");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let white = t.battlefield(P0, "Savannah Lions");
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Horned Kavu");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(white));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Horned Kavu").len(), 1);
}

#[test]
fn igneous_inspiration_deals_damage_and_learns() {
    cr!("701.48a");
    // "Igneous Inspiration deals 3 damage to any target. / Learn."
    supported("Igneous Inspiration");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.hand(P0, "Grizzly Bears");
    let s = t.hand(P0, "Igneous Inspiration");
    // Learn: discard a card to draw a card (no Lessons outside the game).
    t.answer(P0, DecisionKind::Option, decision::Answer::Index(1));
    t.answer_choose(P0, &[]);
    t.cast(P0, s).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "discarded to learn");
    assert_eq!(t.hand_size(P0), 1, "and drew a card");
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}
