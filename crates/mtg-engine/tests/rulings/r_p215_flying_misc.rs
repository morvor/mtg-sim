//! Rulings batch P215 — flying (CR 702.9): cost changes for spells with flying, copies
//! "except it has flying", flying counters on merged creatures, and Food tokens
//! (CR 111.10b).

use crate::r_p215_flash::lose;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::untapped_lands;
use crate::r_s05_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::*;
use crate::r_s21_common::castable;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType, Color};
use mtg_engine::*;

const FLYING: KeywordKind = KeywordKind::Flying;

fn flies(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(FLYING)
}

/// How many of `p`'s lands casting `card` with `targets` tapped.
fn lands_used(t: &mut TestGame, p: PlayerId, card: &str, n: usize, targets: &[Entity]) -> usize {
    let land = match card {
        "Shock" => "Mountain",
        "Seeds of Strength" => "Savannah",
        _ => "Island",
    };
    t.lands(p, land, n);
    let c = t.hand(p, card);
    let before = untapped_lands(t, p);
    let mut b = t.cast(p, c);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    before - untapped_lands(t, p)
}

#[test]
fn jubilant_skybonder_tax_per_flying_creature_targeted() {
    cr!("601.2f", "115.3");
    ruling!(
        "Jubilant Skybonder",
        "An opponent’s spell that targets a single creature with flying you control more than once costs only {2} more to cast. An opponent’s spell that targets multiple creatures with flying you control costs {2} more for each of those creatures."
    );
    ruling!(
        "Jubilant Skybonder",
        "Jubilant Skybonder’s last ability applies to itself as long as it still has flying."
    );
    supported("Jubilant Skybonder");
    // Seeds of Strength ({G}{W}): three "target creature gets +1/+1" instructions.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jubilant Skybonder");
    let crow = t.battlefield(P0, "Storm Crow");
    t.set_step(P1, Step::PrecombatMain);
    let crow_e = Entity::Object(crow);
    assert_eq!(
        lands_used(
            &mut t,
            P1,
            "Seeds of Strength",
            8,
            &[crow_e, crow_e, crow_e]
        ),
        4
    );
    t.resolve_all();
    let mut t = TestGame::new(2);
    let sky2 = t.battlefield(P0, "Jubilant Skybonder");
    let crow = t.battlefield(P0, "Storm Crow");
    t.set_step(P1, Step::PrecombatMain);
    let (a, b) = (Entity::Object(crow), Entity::Object(sky2));
    assert_eq!(
        lands_used(&mut t, P1, "Seeds of Strength", 8, &[a, a, b]),
        6
    );
    // Skybonder itself: {2} more while it has flying, not after it lost flying.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Jubilant Skybonder");
    assert_eq!(lands_used(&mut t, P1, "Shock", 3, &[Entity::Object(s)]), 3);
    t.resolve_all();
    lose(&mut t, s, FLYING);
    assert_eq!(lands_used(&mut t, P1, "Shock", 3, &[Entity::Object(s)]), 1);
}

#[test]
fn flying_cost_reductions_reduce_only_generic_mana() {
    cr!("601.2f", "118.7");
    ruling!(
        "Watcher of the Spheres",
        "The cost reduction applies only to generic mana in the cost of creature spells with flying you cast. For example, if you cast a second Watcher of the Spheres, its cost won't be reduced below {W}{U}."
    );
    ruling!(
        "Warden of Evos Isle",
        "The last ability of Warden of Evos Isle can't reduce the colored mana requirement of a creature spell with flying."
    );
    for lord in ["Watcher of the Spheres", "Warden of Evos Isle"] {
        supported(lord);
        let mut t = TestGame::new(2);
        t.battlefield(P0, lord);
        // A {1}{U} Storm Crow costs {U}.
        assert_eq!(lands_used(&mut t, P0, "Storm Crow", 2, &[]), 1, "{lord}");
        // Another Watcher of the Spheres ({W}{U}) still costs {W}{U}.
        let mut t = TestGame::new(2);
        t.battlefield(P0, lord);
        t.lands(P0, "Plains", 1);
        let w = t.hand(P0, "Watcher of the Spheres");
        assert!(!castable(&mut t, P0, w), "{lord}");
        t.lands(P0, "Island", 1);
        assert!(castable(&mut t, P0, w), "{lord}");
    }
}

#[test]
fn momo_reduces_only_generic_mana_of_the_first_flying_creature_spell() {
    cr!("601.2f", "118.7");
    ruling!(
        "Momo, Friendly Flier",
        "Momo, Friendly Flier's second ability only reduces the generic mana cost of the spell. If first non-Lemur creature spell with flying you cast on your turn doesn't have generic mana in its cost, the cost will not be reduced and the cost reduction will not apply to the next spell you cast that turn."
    );
    supported("Momo, Friendly Flier");
    // First, a {1}{U} Storm Crow: {U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Momo, Friendly Flier");
    assert_eq!(lands_used(&mut t, P0, "Storm Crow", 2, &[]), 1);
    // First, a {U} Faerie Miscreant: still {U}; then the Storm Crow costs {1}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Momo, Friendly Flier");
    assert_eq!(lands_used(&mut t, P0, "Faerie Miscreant", 1, &[]), 1);
    t.resolve_all();
    assert_eq!(lands_used(&mut t, P0, "Storm Crow", 2, &[]), 2);
}

#[test]
fn the_lord_of_the_eagles_cost_is_locked_before_mana_abilities() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "The Lord of the Eagles",
        "The total cost to cast The Lord of the Eagles is locked in before you pay that cost. For example, if you control three flying creatures, each with power 2, including one you can sacrifice to add {C}, the total cost of The Lord of the Eagles is {1}{U}{U}. Then you can sacrifice the creature as you activate mana abilities just before paying the cost."
    );
    supported("The Lord of the Eagles");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wind Drake");
    t.battlefield(P0, "Wind Drake");
    let def = custom_card(
        "Fuel Bird",
        "Creature — Bird",
        "{1}{U}",
        Some((2, 2)),
        "Flying\nSacrifice this creature: Add {C}.",
    );
    let bird = t.custom(P0, def, Zone::Battlefield);
    t.lands(P0, "Island", 2);
    let lord = t.hand(P0, "The Lord of the Eagles");
    // {7}{U}{U} less 6: {1}{U}{U}, paid with the Bird's {C}.
    t.answer_choose(P0, &[Entity::Object(bird)]);
    let spell = t.cast(P0, lord).go();
    assert!(!t.on_battlefield(bird));
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(spell)));
}

#[test]
fn dion_has_flying_on_your_turn_even_if_not_a_knight() {
    cr!("611.3a", "613.1f");
    ruling!(
        "Dion, Bahamut's Dominant // Bahamut, Warden of Light",
        "Dion's first ability will still give it flying during your turn even if Dion somehow isn't a Knight."
    );
    supported("Dion, Bahamut's Dominant // Bahamut, Warden of Light");
    let mut t = TestGame::new(2);
    let dion = t.battlefield(P0, "Dion, Bahamut's Dominant // Bahamut, Warden of Light");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllCreatureTypes],
            duration: Duration::Permanent,
        },
        &[Entity::Object(dion)],
    );
    assert!(!t.obj_now(dion).chars.has_subtype("Knight"));
    assert!(flies(&mut t, dion));
    t.set_step(P1, Step::Upkeep);
    assert!(!flies(&mut t, dion));
}

#[test]
fn sprite_noble_bonus_affects_only_flyers_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Sprite Noble",
        "The set of creatures affected by Sprite Noble's last ability is determined as the ability resolves. Creatures with flying you begin to control later in the turn and creatures that gain flying later in the turn won't get +1/+0."
    );
    supported("Sprite Noble");
    let mut t = TestGame::new(2);
    let noble = t.battlefield(P0, "Sprite Noble");
    let crow = t.battlefield(P0, "Storm Crow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, noble, "+1/+0").expect("activate");
    t.resolve_all();
    // Storm Crow 1/2, +0/+1 static, +1/+0 until end of turn.
    assert_eq!(t.pt(crow), (2, 3));
    crate::r_p215_fight::grant(
        &mut t,
        P0,
        bears,
        mtg_engine::keywords::Keyword::new(FLYING),
    );
    assert_eq!(t.pt(bears), (2, 3));
    let later = t.battlefield(P0, "Storm Crow");
    assert_eq!(t.pt(later), (1, 3));
}

#[test]
fn verix_bladewing_creates_karox() {
    cr!("111.4");
    ruling!(
        "Verix Bladewing",
        "Verix Bladewing features a new template for creating legendary tokens. It's functionally identical to the text \"create a legendary 4/4 red Dragon creature token with flying named Karox Bladewing.\""
    );
    supported("Verix Bladewing");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let v = t.hand(P0, "Verix Bladewing");
    t.cast(P0, v).kicked(true).go();
    t.resolve_all();
    let karox = t.named_on_battlefield("Karox Bladewing");
    assert_eq!(karox.len(), 1);
    let k = t.obj_now(karox[0]);
    assert!(k.is_token());
    assert!(k
        .chars
        .supertypes
        .contains(mtg_engine::types::Supertype::Legendary));
    assert!(k.chars.has_subtype("Dragon"));
    assert_eq!(
        k.chars.colors,
        mtg_engine::types::ColorSet::single(Color::Red)
    );
    assert_eq!(t.pt(karox[0]), (4, 4));
    assert!(flies(&mut t, karox[0]));
}

#[test]
fn astral_dragon_tokens_copy_only_the_printed_permanent() {
    cr!("707.2", "707.9b");
    ruling!(
        "Astral Dragon",
        "Except for power, toughness, types, and the flying ability, the token copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, and so on."
    );
    supported("Astral Dragon");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    t.g.tap(ring);
    t.g.add_counters(Entity::Object(ring), counters::CHARGE, 2, None);
    t.answer_targets(P0, &[Entity::Object(ring)]);
    enter(&mut t, P0, "Astral Dragon");
    t.resolve_all();
    let copies: Vec<_> = t
        .named_on_battlefield("Sol Ring")
        .into_iter()
        .filter(|id| *id != ring)
        .collect();
    assert_eq!(copies.len(), 2);
    for c in copies {
        let o = t.obj_now(c);
        assert!(!o.tapped);
        assert!(o.counters.is_empty());
        assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
        assert!(o.chars.has_subtype("Dragon"));
        assert_eq!(t.pt(c), (3, 3));
        assert!(flies(&mut t, c));
    }
}

#[test]
fn mockingbird_copies_the_printed_creature_plus_bird_and_flying() {
    cr!("707.2", "707.9b", "707.9d");
    ruling!(
        "Mockingbird",
        "Mockingbird copies exactly what was printed on the original creature, except it’s a Bird in addition to its other types and it has flying, and nothing else (unless that creature is copying something else or is a token; see below). It doesn’t copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    supported("Mockingbird");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
    t.lands(P0, "Island", 3);
    let m = t.hand(P0, "Mockingbird");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let spell = t.cast(P0, m).x(2).go();
    t.resolve_all();
    let m = t.g.current(spell);
    let o = t.obj_now(m);
    assert_eq!(o.chars.name.as_str(), "Grizzly Bears");
    assert!(o.chars.has_subtype("Bear") && o.chars.has_subtype("Bird"));
    assert!(!o.tapped);
    assert_eq!(t.counters(m, counters::PLUS1), 0);
    assert_eq!(t.pt(m), (2, 2));
    assert!(flies(&mut t, m));
}

#[test]
fn clones_that_dont_copy_are_their_printed_selves() {
    cr!("707.5", "614.1c");
    ruling!(
        "Mockingbird",
        "You can choose not to have Mockingbird enter as a copy of another creature. If you do, it will just be a 1/1 Bird Bard creature with flying, an ability that didn’t do anything, and the opportunity for a good self-deprecating joke or two."
    );
    ruling!(
        "Deceptive Frostkite",
        "You can choose not to have Deceptive Frostkite enter as a copy of another creature. If you do, it will just be a 1/1 Dragon with flying."
    );
    for (card, sub) in [("Mockingbird", "Bard"), ("Deceptive Frostkite", "Dragon")] {
        supported(card);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Craw Wurm");
        t.lands(P0, "Island", 2);
        let c = t.hand(P0, card);
        t.answer_yes(P0, false);
        let spell = if card == "Mockingbird" {
            t.cast(P0, c).x(1).go()
        } else {
            t.cast(P0, c).go()
        };
        t.resolve_all();
        let id = t.g.current(spell);
        assert_eq!(t.obj_now(id).chars.name.as_str(), card);
        assert!(t.obj_now(id).chars.has_subtype(sub));
        assert_eq!(t.pt(id), (1, 1));
        assert!(flies(&mut t, id));
    }
}

#[test]
fn vesuvan_drifter_copies_the_revealed_card_plus_flying() {
    cr!("707.2", "707.9b");
    ruling!(
        "Vesuvan Drifter",
        "Vesuvan Drifter copies exactly what was printed on the original creature card and nothing else, except it has flying."
    );
    supported("Vesuvan Drifter");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Vesuvan Drifter");
    stack_library(&mut t, P0, &["Hill Giant"]);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(d);
    assert_eq!(o.chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(d), (3, 3));
    assert!(flies(&mut t, d));
}

#[test]
fn broodmoth_returns_each_card_of_a_merged_creature_without_flying() {
    cr!("730.2", "730.3", "111.7");
    ruling!(
        "Luminous Broodmoth",
        "If a merged creature dies and that creature didn't have flying, each card that was part of that merged creature is returned to the battlefield. Each one has a flying counter on it."
    );
    supported("Luminous Broodmoth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Luminous Broodmoth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gem = t.hand(P0, "Gemrazer");
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::G, 2);
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::C, 1);
    t.g.turn.priority = Some(P0);
    t.cast(P0, gem)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    let merged = t.g.current(bears);
    assert!(!flies(&mut t, merged));
    destroy(&mut t, merged);
    t.resolve_all();
    for name in ["Grizzly Bears", "Gemrazer"] {
        let back = t.named_on_battlefield(name);
        assert_eq!(back.len(), 1, "{name}");
        assert_eq!(t.counters(back[0], "flying"), 1, "{name}");
    }
}

#[test]
fn whip_spine_drake_turned_face_up_stays_blocked() {
    cr!("509.1h", "708.8", "702.37e");
    ruling!(
        "Whip-Spine Drake",
        "If Whip-Spine Drake is face down and blocked by a creature without flying, turning it face up won't cause it to become unblocked or stop the blocking creature from dealing combat damage to it."
    );
    supported("Whip-Spine Drake");
    let mut t = TestGame::new(2);
    let drake = crate::r_s12_common::morph(&mut t, P0, "Whip-Spine Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(drake, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, drake)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.lands(P0, "Plains", 3);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, drake));
    assert!(flies(&mut t, drake));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_on(&t, drake), 2);
}

#[test]
fn steamcore_scholar_discard_choices() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Steamcore Scholar",
        "You can discard an instant card, a sorcery card, a creature card with flying, or any two cards, even if one or both of those cards have the listed characteristics."
    );
    supported("Steamcore Scholar");
    // One creature card with flying is enough.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Island", "Island"]);
    let crow = t.hand(P0, "Storm Crow");
    t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(crow)]);
    enter(&mut t, P0, "Steamcore Scholar");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert!(t.in_graveyard(P0, "Storm Crow"));
    // Or two cards, one of them an instant.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Island", "Island"]);
    let shock = t.hand(P0, "Shock");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[Entity::Object(shock), Entity::Object(bears)]);
    enter(&mut t, P0, "Steamcore Scholar");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_graveyard(P0, "Shock") && t.in_graveyard(P0, "Grizzly Bears"));
}

fn assert_food(t: &mut TestGame, food: ObjectId) {
    let o = t.obj_now(food);
    assert!(o.is_token());
    assert!(o.is(CardType::Artifact) && o.chars.has_subtype("Food"));
    assert!(o.chars.colors.is_colorless());
    let life = t.life(P0);
    t.lands(P0, "Wastes", 2);
    activate_containing(t, P0, food, "gain 3 life").expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P0), life + 3);
    assert!(!t.on_battlefield(food));
}

#[test]
fn food_tokens_are_colorless_artifacts_that_gain_3_life() {
    cr!("111.10b", "111.4");
    ruling!(
        "Campsite Cuisine",
        "A Food token is a colorless Food artifact token with \"{2}, {T}, Sacrifice this token: You gain 3 life.\""
    );
    ruling!(
        "Ignis Scientia",
        "A Food token is an artifact token with \"{2}, {T}, Sacrifice this token: You gain 3 life.\""
    );
    supported("Campsite Cuisine");
    supported("Ignis Scientia");
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Campsite Cuisine");
    t.resolve_all();
    let food = tokens_with_subtype(&t, P0, "Food");
    assert_eq!(food.len(), 1);
    assert_food(&mut t, food[0]);
    // Ignis Scientia exiles a creature card from a graveyard: a Food.
    let mut t = TestGame::new(2);
    let ignis = t.battlefield(P0, "Ignis Scientia");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, ignis, "Exile target card").expect("activate");
    t.resolve_all();
    let food = tokens_with_subtype(&t, P0, "Food");
    assert_eq!(food.len(), 1);
    assert_food(&mut t, food[0]);
}

#[test]
fn killer_service_may_sacrifice_any_token() {
    cr!("701.21a", "603.5");
    ruling!(
        "Killer Service",
        "As Killer Service's second ability resolves, you may sacrifice any token, not just a Food token."
    );
    supported("Killer Service");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Killer Service");
    let soldier = create_token(&mut t, P0, "Soldier");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(soldier)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(soldier));
    assert_eq!(tokens_with_subtype(&t, P0, "Rhino").len(), 1);
}
