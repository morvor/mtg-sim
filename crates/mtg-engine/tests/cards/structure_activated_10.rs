//! Structure-coverage tests (round 10): activated-ability structures shared by several
//! cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks
//! that a real card's ability does what its text says.

use super::structure_activated_2::{act, next_turn, shuffled, target_choices, untapped, with_p1p1};
use super::structure_activated_6::entity_choices;
use super::structure_seed_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::*;

/// The sources and amounts of the damage dealt to `target` this turn.
fn damage_sources(t: &TestGame, target: Entity) -> Vec<(ObjectId, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target: tg,
                amount,
                ..
            } if *tg == target => Some((*source, *amount)),
            _ => None,
        })
        .collect()
}

/// The ids of `p`'s graveyard cards named `name`.
fn in_yard(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.find_in_zone(Zone::Graveyard(p), name)
}

#[test]
fn gavony_township_puts_a_counter_on_each_creature_you_control() {
    cr!("602.2", "122.1", "118.3");
    // "{2}{G}{W}, {T}: Put a +1/+1 counter on each creature you control."
    supported("Gavony Township");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gavony Township");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let art = t.battlefield(P0, "Ornithopter");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 1);
    assert!(act(&mut t, P0, g, "+1/+1 counter", &[]).is_err(), "needs {{2}}{{G}}{{W}}");
    t.clear_answers();
    t.lands(P0, "Plains", 1);
    act(&mut t, P0, g, "+1/+1 counter", &[]).unwrap();
    assert!(t.obj_now(g).tapped);
    assert_eq!(untapped(&t, P0, "Forest") + untapped(&t, P0, "Plains"), 0);
    t.resolve_all();
    for c in [a, b, art] {
        assert_eq!(t.counters(c, "+1/+1"), 1, "each creature you control");
    }
    assert_eq!(t.counters(theirs, "+1/+1"), 0, "not the opponent's");
    assert_eq!(t.counters(anthem, "+1/+1"), 0, "not a noncreature");
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn stampeding_scurryfoot_exhausts_for_a_counter_and_an_elephant() {
    cr!("602.2", "702.177a", "111.1", "122.1");
    // "Exhaust — {3}{G}: Put a +1/+1 counter on this creature. Create a 3/3 green Elephant
    // creature token. (Activate each exhaust ability only once.)"
    supported("Stampeding Scurryfoot");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Stampeding Scurryfoot");
    t.lands(P0, "Forest", 8);
    act(&mut t, P0, s, "elephant", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Forest"), 4);
    t.resolve_all();
    assert_eq!(t.counters(s, "+1/+1"), 1);
    assert_eq!(t.pt(s), (2, 2));
    let e = subtype_ids(&t, P0, "Elephant");
    assert_eq!(e.len(), 1);
    assert_eq!(count_subtype(&t, P1, "Elephant"), 0);
    let o = t.obj_now(e[0]);
    assert!(o.is_token() && o.chars.is(CardType::Creature));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert_eq!(t.pt(e[0]), (3, 3));
    // Exhaust: only once, even in a later turn.
    assert!(act(&mut t, P0, s, "elephant", &[]).is_err());
    next_turn(&mut t);
    t.set_step(P0, Step::PrecombatMain);
    assert!(act(&mut t, P0, s, "elephant", &[]).is_err(), "only once ever");
    assert_eq!(count_subtype(&t, P0, "Elephant"), 1);
}

#[test]
fn deathless_ancient_returns_by_tapping_three_vampires() {
    cr!("602.2", "118.3", "602.2a");
    // "Tap three untapped Vampires you control: Return this card from your graveyard to
    // your hand."
    supported("Deathless Ancient");
    let mut t = TestGame::new(2);
    let d = t.graveyard(P0, "Deathless Ancient");
    let v1 = t.battlefield(P0, "Bloodthrone Vampire");
    let v2 = t.battlefield(P0, "Bloodthrone Vampire");
    let theirs = t.battlefield(P1, "Bloodthrone Vampire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(
        act(&mut t, P0, d, "return", &[]).is_err(),
        "two Vampires (the opponent's doesn't count)"
    );
    t.clear_answers();
    let v3 = t.battlefield(P0, "Vampire Nighthawk");
    let tapped = t.battlefield(P0, "Bloodthrone Vampire");
    t.g.tap(tapped);
    t.answer_choose(
        P0,
        &[Entity::Object(v1), Entity::Object(v2), Entity::Object(v3)],
    );
    act(&mut t, P0, d, "return", &[]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(v1)) && c.contains(&Entity::Object(v3)));
    assert!(!c.contains(&Entity::Object(theirs)) && !c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(tapped)), "only untapped Vampires");
    for v in [v1, v2, v3] {
        assert!(t.obj_now(v).tapped, "tapped to pay the cost");
    }
    assert!(!t.obj_now(theirs).tapped && !t.obj_now(bears).tapped);
    t.resolve_all();
    assert!(t.in_hand(P0, "Deathless Ancient"));
    assert!(in_yard(&t, P0, "Deathless Ancient").is_empty());
    // On the battlefield the ability can't be activated (it works only from the graveyard).
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Deathless Ancient");
    for _ in 0..3 {
        t.battlefield(P0, "Bloodthrone Vampire");
    }
    assert!(act(&mut t, P0, d, "return", &[]).is_err());
}

#[test]
fn dragon_blood_puts_a_counter_on_any_target_creature() {
    cr!("602.2", "115.1", "122.1");
    // "{3}, {T}: Put a +1/+1 counter on target creature."
    supported("Dragon Blood");
    let mut t = TestGame::new(2);
    let db = t.battlefield(P0, "Dragon Blood");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Wastes", 2);
    assert!(act(&mut t, P0, db, "counter", &[Entity::Object(theirs)]).is_err());
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, db, "counter", &[Entity::Object(theirs)]).unwrap();
    assert!(t.obj_now(db).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(mine)) && c.contains(&Entity::Object(theirs)));
    assert!(!c.contains(&Entity::Object(land)) && !c.contains(&Entity::Object(db)));
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
    assert_eq!(t.counters(mine, "+1/+1"), 0);
    assert_eq!(t.pt(theirs), (4, 4));
}

#[test]
fn auratog_sacrifices_an_enchantment_to_get_bigger() {
    cr!("602.2", "701.21a", "611.2a", "514.2");
    // "Sacrifice an enchantment: This creature gets +2/+2 until end of turn."
    supported("Auratog");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Auratog");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Glorious Anthem");
    assert!(act(&mut t, P0, a, "+2/+2", &[]).is_err(), "no enchantment to sacrifice");
    t.clear_answers();
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let pac = t.battlefield(P0, "Pacifism");
    t.g.attach(pac, Entity::Object(bears));
    t.answer_choose(P0, &[Entity::Object(anthem)]);
    act(&mut t, P0, a, "+2/+2", &[]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(anthem)) && c.contains(&Entity::Object(pac)));
    assert!(!c.contains(&Entity::Object(theirs)), "not yours");
    assert!(!c.contains(&Entity::Object(bears)), "not an enchantment");
    assert!(!t.on_battlefield(anthem) && t.in_graveyard(P0, "Glorious Anthem"));
    assert!(t.on_battlefield(pac));
    t.resolve_all();
    assert_eq!(t.pt(a), (3, 4), "+2/+2 (and the opponent's anthem doesn't apply)");
    next_turn(&mut t);
    assert_eq!(t.pt(a), (1, 2), "until end of turn");
}

#[test]
fn tablet_of_compleation_taps_for_an_oil_counter() {
    cr!("602.2", "122.1", "602.5", "605.1a");
    // "{T}: Put an oil counter on this artifact." / "{T}: Add {C}. Activate only if this
    // artifact has two or more oil counters on it."
    supported("Tablet of Compleation");
    let mut t = TestGame::new(2);
    let tab = t.battlefield(P0, "Tablet of Compleation");
    let other = t.battlefield(P0, "Tablet of Compleation");
    act(&mut t, P0, tab, "oil counter on", &[]).unwrap();
    assert!(t.obj_now(tab).tapped);
    t.resolve_all();
    assert_eq!(t.counters(tab, "oil"), 1);
    assert_eq!(t.counters(other, "oil"), 0);
    assert!(act(&mut t, P0, tab, "oil counter on", &[]).is_err(), "already tapped");
    t.g.untap(tab);
    assert!(act(&mut t, P0, tab, "add {c}", &[]).is_err(), "one oil counter");
    t.clear_answers();
    act(&mut t, P0, tab, "oil counter on", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(tab, "oil"), 2);
    t.g.untap(tab);
    act(&mut t, P0, tab, "add {c}", &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 1, "two oil counters");
}

#[test]
fn whirlermaker_makes_a_flying_thopter() {
    cr!("602.2", "111.1", "118.3");
    // "{4}, {T}: Create a 1/1 colorless Thopter artifact creature token with flying."
    supported("Whirlermaker");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Whirlermaker");
    t.lands(P0, "Wastes", 3);
    assert!(act(&mut t, P0, w, "thopter", &[]).is_err());
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, w, "thopter", &[]).unwrap();
    assert!(t.obj_now(w).tapped);
    t.resolve_all();
    let th = subtype_ids(&t, P0, "Thopter");
    assert_eq!(th.len(), 1);
    assert_eq!(count_subtype(&t, P1, "Thopter"), 0);
    let o = t.obj_now(th[0]);
    assert!(o.is_token());
    assert!(o.chars.is(CardType::Artifact) && o.chars.is(CardType::Creature));
    assert!(o.chars.colors.is_colorless());
    assert!(o.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(th[0]), (1, 1));
}

#[test]
fn bonebind_orator_exiles_itself_to_return_another_creature_card() {
    cr!("602.2", "602.2a", "115.1", "118.3");
    // "{3}{B}, Exile this card from your graveyard: Return another target creature card
    // from your graveyard to your hand."
    supported("Bonebind Orator");
    let mut t = TestGame::new(2);
    let o = t.graveyard(P0, "Bonebind Orator");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let theirs = t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    act(&mut t, P0, o, "return another", &[Entity::Object(bears)]).unwrap();
    assert_eq!(t.zone(o), Zone::Exile, "exiled as a cost");
    assert_eq!(untapped(&t, P0, "Swamp"), 0);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(o)), "another");
    assert!(!c.contains(&Entity::Object(bolt)), "not a creature card");
    assert!(!c.contains(&Entity::Object(theirs)), "not your graveyard");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P0), 1);
    // It can't be activated from the battlefield.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Bonebind Orator");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    assert!(act(&mut t, P0, o, "return another", &[]).is_err());
}

#[test]
fn etherium_astrolabe_sacrifices_an_artifact_to_draw() {
    cr!("602.2", "701.21a", "121.1", "118.3");
    // "{B}, {T}, Sacrifice an artifact: Draw a card."
    supported("Etherium Astrolabe");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Etherium Astrolabe");
    let orni = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Memnite");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[Entity::Object(orni)]);
    act(&mut t, P0, a, "draw", &[]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(orni)) && c.contains(&Entity::Object(a)));
    assert!(!c.contains(&Entity::Object(bears)) && !c.contains(&Entity::Object(theirs)));
    assert!(!t.on_battlefield(orni), "sacrificed as a cost");
    assert!(t.obj_now(a).tapped);
    assert_eq!(untapped(&t, P0, "Swamp"), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.on_battlefield(theirs));
}

#[test]
fn travelers_amulet_fetches_a_basic_land_to_hand() {
    cr!("602.2", "701.23a", "701.24a", "118.3");
    // "{1}, Sacrifice this artifact: Search your library for a basic land card, reveal it,
    // put it into your hand, then shuffle."
    supported("Traveler's Amulet");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Traveler's Amulet");
    let forest = t.library_top(P0, "Forest");
    let bayou = t.library_top(P0, "Bayou");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    act(&mut t, P0, a, "search", &[]).unwrap();
    assert!(!t.on_battlefield(a), "sacrificed as a cost");
    t.resolve_all();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(forest)));
    assert!(!c.contains(&Entity::Object(bayou)), "not basic");
    assert!(!c.contains(&Entity::Object(bears)), "not a land");
    assert_eq!(t.zone(forest), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.zone(bayou), Zone::Library(P0));
    assert!(shuffled(&t, P0));
    assert!(!shuffled(&t, P1));
}

#[test]
fn viral_drake_proliferates() {
    cr!("602.2", "701.34a");
    // "{3}{U}: Proliferate."
    supported("Viral Drake");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Viral Drake");
    let mine = with_p1p1(&mut t, P0, "Grizzly Bears", 1);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(theirs), "-1/-1", 1, None);
    let skipped = with_p1p1(&mut t, P0, "Hill Giant", 2);
    t.g.add_counters(Entity::Player(P1), "poison", 2, None);
    t.lands(P0, "Island", 4);
    t.answer_choose(
        P0,
        &[
            Entity::Object(mine),
            Entity::Object(theirs),
            Entity::Player(P1),
        ],
    );
    act(&mut t, P0, d, "proliferate", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Island"), 0);
    assert!(!t.obj_now(d).tapped, "no {{T}} in the cost");
    t.resolve_all();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(skipped)));
    assert!(!c.contains(&Entity::Object(d)), "no counters on it");
    assert!(!c.contains(&Entity::Player(P0)), "no counters");
    assert_eq!(t.counters(mine, "+1/+1"), 2);
    assert_eq!(t.counters(theirs, "-1/-1"), 2);
    assert_eq!(t.counters(skipped, "+1/+1"), 2, "not chosen");
    assert_eq!(t.g.player(P1).counters.get("poison").copied().unwrap_or(0), 3);
}

#[test]
fn relentless_hunter_pumps_and_gains_trample_until_end_of_turn() {
    cr!("602.2", "611.2a", "514.2", "702.19b");
    // "{1}{R}{G}: This creature gets +1/+1 and gains trample until end of turn."
    supported("Relentless Hunter");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Relentless Hunter");
    let other = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 2);
    act(&mut t, P0, h, "trample", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(h), (4, 4));
    assert!(t.obj_now(h).has_keyword(KeywordKind::Trample));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(other), (2, 2));
    // Blocked by a 2/2: 2 lethal to the blocker, 2 tramples over.
    t.attack(&[(h, Entity::Player(P1))], &[(wall, h)]);
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(wall));
    next_turn(&mut t);
    assert_eq!(t.pt(h), (3, 3));
    assert!(!t.obj_now(h).has_keyword(KeywordKind::Trample));
}

#[test]
fn aphetto_grifter_taps_two_wizards_to_tap_a_permanent() {
    cr!("602.2", "118.3", "302.6", "701.26a");
    // "Tap two untapped Wizards you control: Tap target permanent."
    supported("Aphetto Grifter");
    let mut t = TestGame::new(2);
    let g = t.battlefield_sick(P0, "Aphetto Grifter");
    let land = t.battlefield(P1, "Forest");
    assert!(
        act(&mut t, P0, g, "tap target", &[Entity::Object(land)]).is_err(),
        "only one Wizard"
    );
    t.clear_answers();
    let w = t.battlefield_sick(P0, "Prodigal Sorcerer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(g), Entity::Object(w)]);
    // No {T} symbol: summoning-sick Wizards can pay the cost (CR 302.6).
    act(&mut t, P0, g, "tap target", &[Entity::Object(land)]).unwrap();
    assert!(t.obj_now(g).tapped && t.obj_now(w).tapped);
    assert!(!t.obj_now(bears).tapped, "not a Wizard");
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(land)) && c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(t.obj_now(land).tapped);
}

#[test]
fn adanto_vanguard_pays_four_life_for_indestructible() {
    cr!("602.2", "119.4", "702.12b", "611.2a", "514.2");
    // "Pay 4 life: This creature gains indestructible until end of turn."
    supported("Adanto Vanguard");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Adanto Vanguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, v, "indestructible", &[]).unwrap();
    assert_eq!(t.life(P0), 16, "4 life paid");
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert!(t.obj_now(v).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(v).go();
    t.resolve_all();
    assert!(t.on_battlefield(v), "lethal damage doesn't destroy it");
    next_turn(&mut t);
    assert!(!t.obj_now(v).has_keyword(KeywordKind::Indestructible));
    // With only 3 life, 4 life can't be paid (CR 119.4).
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Adanto Vanguard");
    t.g.player_mut(P0).life = 3;
    assert!(act(&mut t, P0, v, "indestructible", &[]).is_err());
    assert_eq!(t.life(P0), 3);
}

#[test]
fn aether_spellbomb_bounces_a_creature_to_its_owners_hand() {
    cr!("602.2", "701.21a", "115.1", "118.3");
    // "{U}, Sacrifice this artifact: Return target creature to its owner's hand."
    supported("Aether Spellbomb");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Aether Spellbomb");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    act(&mut t, P0, b, "return target", &[Entity::Object(theirs)]).unwrap();
    assert!(!t.on_battlefield(b) && t.in_graveyard(P0, "Aether Spellbomb"));
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(theirs)) && c.contains(&Entity::Object(mine)));
    assert!(!c.contains(&Entity::Object(land)));
    t.resolve_all();
    assert_eq!(t.zone(theirs), Zone::Hand(P1));
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.on_battlefield(mine));
    // A creature you gained control of goes to its owner's hand, not yours.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Aether Spellbomb");
    let stolen = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let ti = t.hand(P0, "Traitorous Instinct");
    t.cast(P0, ti).target(stolen).go();
    t.resolve_all();
    assert_eq!(t.obj_now(stolen).controller, P0);
    t.lands(P0, "Island", 1);
    act(&mut t, P0, b, "return target", &[Entity::Object(stolen)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(stolen), Zone::Hand(P1));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn ahriman_sacrifices_another_creature_or_artifact_to_draw() {
    cr!("602.2", "701.21a", "121.1", "118.3");
    // "{3}, Sacrifice another creature or artifact: Draw a card."
    supported("Ahriman");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ahriman");
    t.lands(P0, "Wastes", 3);
    assert!(act(&mut t, P0, a, "draw", &[]).is_err(), "nothing else to sacrifice");
    t.clear_answers();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let art = t.battlefield(P0, "Dragon Blood");
    let ench = t.battlefield(P0, "Glorious Anthem");
    let theirs = t.battlefield(P1, "Memnite");
    t.answer_choose(P0, &[Entity::Object(art)]);
    act(&mut t, P0, a, "draw", &[]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Object(art)));
    assert!(!c.contains(&Entity::Object(a)), "another");
    assert!(!c.contains(&Entity::Object(ench)) && !c.contains(&Entity::Object(theirs)));
    assert!(!t.on_battlefield(art));
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn gideon_ally_of_zendikar_emblem_pumps_your_creatures() {
    cr!("606.3", "114.1", "114.4", "613.4c", "704.5i");
    // "−4: You get an emblem with "Creatures you control get +1/+1.""
    supported("Gideon, Ally of Zendikar");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gideon, Ally of Zendikar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, g, "emblem", &[]).unwrap();
    assert_eq!(t.counters(g, "loyalty"), 0, "−4 paid");
    t.resolve_all();
    assert!(!t.on_battlefield(g), "0 loyalty");
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(theirs), (2, 2), "not the opponent's");
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (4, 4), "creatures that arrive later too");
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (3, 3), "the emblem is permanent");
}

#[test]
fn ajani_strength_of_the_pride_makes_a_named_pridemate() {
    cr!("606.3", "111.1", "111.4");
    // "−2: Create a 2/2 white Cat Soldier creature token named Ajani's Pridemate with
    // "Whenever you gain life, put a +1/+1 counter on this token.""
    supported("Ajani, Strength of the Pride");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ajani, Strength of the Pride");
    act(&mut t, P0, a, "pridemate", &[]).unwrap();
    assert_eq!(t.counters(a, "loyalty"), 3);
    t.resolve_all();
    let p = t.named_on_battlefield("Ajani's Pridemate");
    assert_eq!(p.len(), 1);
    let o = t.obj_now(p[0]);
    assert!(o.is_token() && o.controller == P0);
    assert!(o.chars.has_subtype("Cat") && o.chars.has_subtype("Soldier"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert_eq!(t.pt(p[0]), (2, 2));
    // Its own ability: gaining life puts a counter on it.
    t.lands(P0, "Plains", 2);
    let r = t.hand(P0, "Revitalize");
    t.cast(P0, r).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.counters(p[0], "+1/+1"), 1);
    assert_eq!(t.pt(p[0]), (3, 3));
}

#[test]
fn elspeth_tirel_gains_life_for_each_creature_you_control() {
    cr!("606.3", "119.3");
    // "+2: You gain 1 life for each creature you control."
    supported("Elspeth Tirel");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Elspeth Tirel");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Memnite");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Forest");
    act(&mut t, P0, e, "gain 1 life", &[]).unwrap();
    assert_eq!(t.counters(e, "loyalty"), 6);
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "two creatures you control");
    assert_eq!(t.life(P1), 20);
    // Loyalty abilities: once per turn.
    assert!(act(&mut t, P0, e, "gain 1 life", &[]).is_err());
}

#[test]
fn ambassador_laquatus_mills_a_target_player() {
    cr!("602.2", "701.17a", "115.1");
    // "{3}: Target player mills three cards."
    supported("Ambassador Laquatus");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ambassador Laquatus");
    t.lands(P0, "Wastes", 3);
    act(&mut t, P0, a, "mills", &[Entity::Player(P1)]).unwrap();
    assert!(!t.obj_now(a).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P0)) && c.contains(&Entity::Player(P1)));
    t.resolve_all();
    assert_eq!(t.library_size(P1), 27);
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn seal_of_removal_sacrifices_to_bounce_a_creature() {
    cr!("602.2", "701.21a", "115.1");
    // "Sacrifice this enchantment: Return target creature to its owner's hand."
    supported("Seal of Removal");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seal of Removal");
    let theirs = t.battlefield(P1, "Hill Giant");
    let art = t.battlefield(P1, "Dragon Blood");
    act(&mut t, P0, s, "return target", &[Entity::Object(theirs)]).unwrap();
    assert!(t.in_graveyard(P0, "Seal of Removal"));
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(theirs)) && !c.contains(&Entity::Object(art)));
    t.resolve_all();
    assert_eq!(t.zone(theirs), Zone::Hand(P1));
    assert!(t.on_battlefield(art));
}

#[test]
fn monoskelion_removes_a_counter_to_ping_any_target() {
    cr!("602.2", "118.3", "122.1", "120.3a");
    // "{1}, Remove a +1/+1 counter from this creature: It deals 1 damage to any target."
    supported("Monoskelion");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Monoskelion");
    t.lands(P0, "Wastes", 2);
    assert!(
        act(&mut t, P0, m, "damage", &[Entity::Player(P1)]).is_err(),
        "no +1/+1 counter to remove"
    );
    t.clear_answers();
    t.g.add_counters(Entity::Object(m), "+1/+1", 1, None);
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, m, "damage", &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.counters(m, "+1/+1"), 0, "removed as a cost");
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Player(P0)));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(damage_sources(&t, Entity::Player(P1)), vec![(m, 1)]);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn archangel_elspeth_makes_a_lifelink_soldier() {
    cr!("606.3", "111.1", "702.15b");
    // "+1: Create a 1/1 white Soldier creature token with lifelink."
    supported("Archangel Elspeth");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Archangel Elspeth");
    act(&mut t, P0, e, "lifelink", &[]).unwrap();
    assert_eq!(t.counters(e, "loyalty"), 5);
    t.resolve_all();
    let s = subtype_ids(&t, P0, "Soldier");
    assert_eq!(s.len(), 1);
    let o = t.obj_now(s[0]);
    assert!(o.is_token() && o.has_keyword(KeywordKind::Lifelink));
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert_eq!(t.pt(s[0]), (1, 1));
    assert!(act(&mut t, P0, e, "lifelink", &[]).is_err(), "once per turn");
    // Its lifelink works.
    t.g.objects[s[0].0 as usize].summoning_sick = false;
    t.attack(&[(s[0], Entity::Player(P1))], &[]);
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn teferis_care_counters_only_an_enchantment_spell() {
    cr!("602.2", "701.6a", "115.1");
    // "{3}{U}{U}: Counter target enchantment spell."
    supported("Teferi's Care");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Teferi's Care");
    t.lands(P0, "Island", 5);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    assert!(
        act(&mut t, P0, c, "counter target", &[]).is_err(),
        "a creature spell isn't a legal target"
    );
    t.clear_answers();
    t.resolve_all();
    t.lands(P1, "Plains", 3);
    let anthem = t.hand(P1, "Glorious Anthem");
    let spell = t.cast(P1, anthem).go();
    act(&mut t, P0, c, "counter target", &[Entity::Object(spell)]).unwrap();
    assert_eq!(untapped(&t, P0, "Island"), 0);
    t.resolve_all();
    assert!(t.named_on_battlefield("Glorious Anthem").is_empty());
    assert!(t.in_graveyard(P1, "Glorious Anthem"), "countered");
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn arensons_aura_sacrifices_an_enchantment_to_destroy_one() {
    cr!("602.2", "701.21a", "701.8a", "115.1");
    // "{W}, Sacrifice an enchantment: Destroy target enchantment."
    supported("Arenson's Aura");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Arenson's Aura");
    let fodder = t.battlefield(P0, "Glorious Anthem");
    let target = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    act(&mut t, P0, a, "destroy target", &[Entity::Object(target)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(target)) && !c.contains(&Entity::Object(bears)));
    let s = entity_choices(&t, P0).pop().unwrap();
    assert!(s.contains(&Entity::Object(fodder)) && s.contains(&Entity::Object(a)));
    assert!(!s.contains(&Entity::Object(target)), "not yours");
    assert!(!t.on_battlefield(fodder) && t.on_battlefield(a));
    t.resolve_all();
    assert!(!t.on_battlefield(target) && t.in_graveyard(P1, "Glorious Anthem"));
}

#[test]
fn ark_of_blight_destroys_a_target_land() {
    cr!("602.2", "701.21a", "701.8a", "115.1");
    // "{3}, {T}, Sacrifice this artifact: Destroy target land."
    supported("Ark of Blight");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ark of Blight");
    let land = t.battlefield(P1, "Bayou");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    act(&mut t, P0, a, "destroy", &[Entity::Object(land)]).unwrap();
    assert!(t.in_graveyard(P0, "Ark of Blight"));
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(land)) && !c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Bayou"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn aryel_makes_a_vigilant_knight() {
    cr!("602.2", "111.1");
    // "{2}{W}, {T}: Create a 2/2 white Knight creature token with vigilance."
    supported("Aryel, Knight of Windgrace");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Aryel, Knight of Windgrace");
    t.lands(P0, "Plains", 3);
    act(&mut t, P0, a, "create", &[]).unwrap();
    assert!(t.obj_now(a).tapped);
    t.resolve_all();
    let k: Vec<_> = subtype_ids(&t, P0, "Knight")
        .into_iter()
        .filter(|k| t.obj_now(*k).is_token())
        .collect();
    assert_eq!(k.len(), 1);
    let o = t.obj_now(k[0]);
    assert!(o.has_keyword(KeywordKind::Vigilance));
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert_eq!(t.pt(k[0]), (2, 2));
}

#[test]
fn atomwheel_acrobats_rolls_a_six_sided_die() {
    cr!("602.2", "706.1", "706.2");
    // "{2}{G}: Roll a six-sided die." (and "Whenever you roll a 1 or 2, put that many
    // +1/+1 counters on this creature.")
    supported("Atomwheel Acrobats");
    for _ in 0..4 {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Atomwheel Acrobats");
        t.lands(P0, "Forest", 3);
        act(&mut t, P0, a, "roll", &[]).unwrap();
        t.resolve_all();
        let rolls: Vec<_> = t
            .g
            .turn_events
            .iter()
            .filter_map(|e| match e {
                Event::DieRolled {
                    player,
                    sides,
                    result,
                    ..
                } => Some((*player, *sides, *result)),
                _ => None,
            })
            .collect();
        assert_eq!(rolls.len(), 1);
        let (p, sides, r) = rolls[0];
        assert_eq!((p, sides), (P0, 6));
        assert!((1..=6).contains(&r));
        let expect = if r <= 2 { r } else { 0 };
        assert_eq!(t.counters(a, "+1/+1"), expect);
    }
}

#[test]
fn auriok_transfixer_taps_a_target_artifact() {
    cr!("602.2", "701.26a", "115.1");
    // "{W}, {T}: Tap target artifact."
    supported("Auriok Transfixer");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Auriok Transfixer");
    let art = t.battlefield(P1, "Memnite");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    act(&mut t, P0, a, "tap target", &[Entity::Object(art)]).unwrap();
    assert!(t.obj_now(a).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(art)) && !c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(t.obj_now(art).tapped);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn visara_destroys_a_creature_that_cant_be_regenerated() {
    cr!("602.2", "701.8a", "701.19c");
    // "{T}: Destroy target creature. It can't be regenerated."
    supported("Visara the Dreadful");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Visara the Dreadful");
    let sk = t.battlefield(P1, "Drudge Skeletons");
    let land = t.battlefield(P1, "Forest");
    t.lands(P1, "Swamp", 1);
    act(&mut t, P1, sk, "regenerate", &[]).unwrap();
    t.resolve_all();
    act(&mut t, P0, v, "destroy", &[Entity::Object(sk)]).unwrap();
    assert!(t.obj_now(v).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(sk)) && !c.contains(&Entity::Object(land)));
    t.resolve_all();
    assert!(
        t.in_graveyard(P1, "Drudge Skeletons"),
        "the regeneration shield doesn't apply"
    );
}

#[test]
fn resurrected_cultist_returns_with_delirium_as_a_sorcery() {
    cr!("602.2", "602.5d", "602.5", "122.1");
    // "Delirium — {2}{B}{B}: Return this card from your graveyard to the battlefield with a
    // finality counter on it. Activate only if there are four or more card types among
    // cards in your graveyard and only as a sorcery."
    supported("Resurrected Cultist");
    let mut t = TestGame::new(2);
    let c = t.graveyard(P0, "Resurrected Cultist");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    t.graveyard(P1, "Ornithopter");
    t.lands(P0, "Swamp", 4);
    assert!(
        act(&mut t, P0, c, "finality", &[]).is_err(),
        "three card types in your graveyard (the opponent's artifact doesn't count)"
    );
    t.clear_answers();
    t.graveyard(P0, "Ornithopter");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(act(&mut t, P0, c, "finality", &[]).is_err(), "only as a sorcery");
    t.clear_answers();
    t.set_step(P0, Step::PostcombatMain);
    act(&mut t, P0, c, "finality", &[]).unwrap();
    t.resolve_all();
    let now = t.g.current(c);
    assert!(t.on_battlefield(now));
    assert_eq!(t.obj_now(now).controller, P0);
    assert_eq!(t.counters(now, "finality"), 1);
}

#[test]
fn icatian_infantry_gains_banding_until_end_of_turn() {
    cr!("602.2", "611.2a", "514.2");
    // "{1}: This creature gains banding until end of turn."
    supported("Icatian Infantry");
    let mut t = TestGame::new(2);
    let i = t.battlefield(P0, "Icatian Infantry");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, i, "banding", &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(i).has_keyword(KeywordKind::Banding));
    assert!(!t.obj_now(i).has_keyword(KeywordKind::FirstStrike), "the other ability");
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Banding));
    next_turn(&mut t);
    assert!(!t.obj_now(i).has_keyword(KeywordKind::Banding));
}

#[test]
fn birthing_boughs_makes_a_changeling_shapeshifter() {
    cr!("602.2", "111.1", "702.73a");
    // "{4}, {T}: Create a 2/2 colorless Shapeshifter creature token with changeling."
    supported("Birthing Boughs");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Birthing Boughs");
    t.lands(P0, "Wastes", 4);
    act(&mut t, P0, b, "shapeshifter", &[]).unwrap();
    assert!(t.obj_now(b).tapped);
    t.resolve_all();
    let s = subtype_ids(&t, P0, "Shapeshifter");
    assert_eq!(s.len(), 1);
    let o = t.obj_now(s[0]);
    assert!(o.is_token() && o.chars.colors.is_colorless());
    assert!(o.chars.has_subtype("Elf") && o.chars.has_subtype("Goblin"), "changeling");
    assert!(!o.chars.has_subtype("Forest"), "not a land type");
    assert_eq!(t.pt(s[0]), (2, 2));
    // A Goblin lord pumps it: it's a Goblin.
    t.battlefield(P0, "Goblin King");
    assert_eq!(t.pt(s[0]), (3, 3));
}

#[test]
fn hurler_cyclops_sacrifices_another_creature_to_ping() {
    cr!("602.2", "701.21a", "120.3a", "118.3");
    // "{1}, Sacrifice another creature: This creature deals 1 damage to any target."
    supported("Hurler Cyclops");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hurler Cyclops");
    t.lands(P0, "Wastes", 2);
    assert!(
        act(&mut t, P0, h, "damage", &[Entity::Player(P1)]).is_err(),
        "no other creature"
    );
    t.clear_answers();
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let art = t.battlefield(P0, "Dragon Blood");
    let theirs = t.battlefield(P1, "Savannah Lions");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    act(&mut t, P0, h, "damage", &[Entity::Object(theirs)]).unwrap();
    let s = entity_choices(&t, P0).pop().unwrap();
    assert!(s.contains(&Entity::Object(fodder)));
    assert!(!s.contains(&Entity::Object(h)) && !s.contains(&Entity::Object(art)));
    assert!(!s.contains(&Entity::Object(theirs)));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Savannah Lions"));
    assert_eq!(damage_sources(&t, Entity::Object(theirs)), vec![(h, 1)]);
}

#[test]
fn blessing_pumps_the_enchanted_creature() {
    cr!("602.2", "303.4b", "611.2a", "514.2");
    // "Enchant creature" / "{W}: Enchanted creature gets +1/+1 until end of turn."
    supported("Blessing");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Blessing");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(b, Entity::Object(theirs));
    t.lands(P0, "Plains", 2);
    act(&mut t, P0, b, "+1/+1", &[]).unwrap();
    act(&mut t, P0, b, "+1/+1", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(theirs), (4, 4), "the enchanted creature, whoever controls it");
    assert_eq!(t.pt(mine), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn hansk_deals_two_damage_to_a_target_creature() {
    cr!("602.2", "115.1", "120.3e");
    // "{T}: Hansk, Slayer Zealot deals 2 damage to target creature."
    supported("Hansk, Slayer Zealot");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hansk, Slayer Zealot");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    act(&mut t, P0, h, "2 damage", &[Entity::Object(bears)]).unwrap();
    assert!(t.obj_now(h).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Object(h)));
    assert!(!c.contains(&Entity::Player(P1)) && !c.contains(&Entity::Object(garruk)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_sources(&t, Entity::Object(bears)), vec![(h, 2)]);
}

#[test]
fn laurine_sacrifices_an_artifact_or_creature_to_goad() {
    cr!("602.2", "701.15a", "701.21a");
    // "{2}, Sacrifice an artifact or creature: Goad target creature."
    supported("Laurine, the Diversion");
    let mut t = TestGame::new(3);
    let l = t.battlefield(P0, "Laurine, the Diversion");
    let art = t.battlefield(P0, "Dragon Blood");
    let ench = t.battlefield(P0, "Glorious Anthem");
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[Entity::Object(art)]);
    act(&mut t, P0, l, "goad", &[Entity::Object(target)]).unwrap();
    let s = entity_choices(&t, P0).pop().unwrap();
    assert!(s.contains(&Entity::Object(art)) && s.contains(&Entity::Object(l)));
    assert!(!s.contains(&Entity::Object(ench)));
    assert!(!t.on_battlefield(art));
    t.resolve_all();
    assert_eq!(t.obj_now(target).goaded_by, vec![P0]);
    assert!(t.obj_now(other).goaded_by.is_empty());
    // Until your next turn.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(target).goaded_by.is_empty());
}

#[test]
fn dark_depths_removes_an_ice_counter() {
    cr!("602.2", "122.1", "614.1c");
    // "Dark Depths enters with ten ice counters on it." / "{3}: Remove an ice counter from
    // Dark Depths."
    supported("Dark Depths");
    let mut t = TestGame::new(2);
    let d = t.enter(P0, "Dark Depths");
    assert_eq!(t.counters(d, "ice"), 10);
    t.lands(P0, "Wastes", 6);
    act(&mut t, P0, d, "remove", &[]).unwrap();
    act(&mut t, P0, d, "remove", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    assert!(!t.obj_now(d).tapped);
    t.resolve_all();
    assert_eq!(t.counters(d, "ice"), 8);
    assert!(t.named_on_battlefield("Marit Lage").is_empty());
}

#[test]
fn bloodpyre_elemental_sacrifices_for_four_damage_as_a_sorcery() {
    cr!("602.2", "602.5d", "701.21a", "120.3e");
    // "Sacrifice this creature: It deals 4 damage to target creature. Activate only as a
    // sorcery."
    supported("Bloodpyre Elemental");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Bloodpyre Elemental");
    let target = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(act(&mut t, P0, e, "4 damage", &[Entity::Object(target)]).is_err());
    t.clear_answers();
    assert!(t.on_battlefield(e));
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, e, "4 damage", &[Entity::Object(target)]).unwrap();
    assert!(t.in_graveyard(P0, "Bloodpyre Elemental"));
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(!c.contains(&Entity::Player(P1)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(damage_sources(&t, Entity::Object(target)), vec![(e, 4)]);
}

#[test]
fn frostwalla_snow_pump_once_each_turn() {
    cr!("602.2", "107.4h", "602.5b", "611.2a", "514.2");
    // "{S}: This creature gets +2/+2 until end of turn. Activate only once each turn."
    supported("Frostwalla");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Frostwalla");
    t.lands(P0, "Forest", 2);
    assert!(act(&mut t, P0, f, "+2/+2", &[]).is_err(), "{{S}} needs snow mana");
    t.clear_answers();
    t.lands(P0, "Snow-Covered Forest", 2);
    act(&mut t, P0, f, "+2/+2", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Snow-Covered Forest"), 1);
    t.resolve_all();
    assert_eq!(t.pt(f), (4, 4));
    assert!(act(&mut t, P0, f, "+2/+2", &[]).is_err(), "once each turn");
    t.clear_answers();
    next_turn(&mut t);
    assert_eq!(t.pt(f), (2, 2));
    act(&mut t, P0, f, "+2/+2", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(f), (4, 4), "again in the next turn");
}
