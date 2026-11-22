//! Rulings batch P088 — hatebears that stop activated abilities, casting, land plays and
//! searching (Collector Ouphe, Grand Abolisher, Harsh Mentor, Tomik, Meddling Mage,
//! Leonin Arbiter; CR 602.5, 605.1a, 101.2, 116.2d, 701.23).

use crate::r_p088_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn collector_ouphe_stops_only_artifacts_on_the_battlefield() {
    cr!("602.5", "702.29a", "113.6");
    ruling!(
        "Collector Ouphe",
        "Collector Ouphe's ability affects only artifacts on the battlefield. Activated abilities that work in other zones (such as cycling) can still be activated."
    );
    supported("Collector Ouphe");
    supported("Ketria Crystal");
    let mut t = TestGame::new(2);
    // Its mana ability ("{T}: Add {G}, {U}, or {R}."): yes without Collector Ouphe, no
    // with it.
    let a = t.battlefield(P0, "Ketria Crystal");
    let b = t.battlefield(P0, "Ketria Crystal");
    assert!(t.activate(P0, a, 0, &[]).is_ok());
    assert_eq!(t.player(P0).mana_pool.total(), 1);
    t.battlefield(P1, "Collector Ouphe");
    assert!(t.activate(P0, b, 0, &[]).is_err());
    assert_eq!(t.player(P0).mana_pool.total(), 1);
    t.g.players[0].mana_pool = Default::default();
    let in_hand = t.hand(P0, "Ketria Crystal");
    colorless(&mut t, P0, 2);
    assert!(crate::r_s04_common::can_cycle(&mut t, P0, in_hand));
    crate::r_s04_common::cycle(&mut t, P0, in_hand, 0).expect("cycle");
    assert_eq!(t.zone(in_hand), Zone::Graveyard(P0));
}

#[test]
fn collector_ouphe_cant_be_sacrificed_to_its_controllers_artifact() {
    cr!("602.5", "602.2");
    ruling!(
        "Collector Ouphe",
        "If you control Collector Ouphe, you can't activate any abilities of your artifacts, even if you'd sacrifice Collector Ouphe as a cost to activate one of those abilities."
    );
    supported("Ashnod's Altar");
    // Ashnod's Altar: "Sacrifice a creature: Add {C}{C}." Without Collector Ouphe, P0 can
    // sacrifice a creature to it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let altar = t.battlefield(P0, "Ashnod's Altar");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, altar, 0, &[]).expect("sacrifice the Bears");
    assert_eq!(t.player(P0).mana_pool.total(), 2);
    let mut t = TestGame::new(2);
    let ouphe = t.battlefield(P0, "Collector Ouphe");
    let altar = t.battlefield(P0, "Ashnod's Altar");
    t.answer_choose(P0, &[obj(ouphe)]);
    assert!(t.activate(P0, altar, 0, &[]).is_err());
    assert!(t.on_battlefield(ouphe));
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}

#[test]
fn grand_abolisher_doesnt_stop_abilities_of_cards_in_other_zones() {
    cr!("602.5", "702.29a");
    ruling!(
        "Grand Abolisher",
        "Grand Abolisher doesn't stop your opponents from activating abilities of artifact, creature, or entorchantment cards in zones other than the battlefield (like cycling abilities, for example)."
    );
    supported("Grand Abolisher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grand Abolisher");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let crystal = t.hand(P1, "Ketria Crystal");
    colorless(&mut t, P1, 2);
    // During P0's turn: P1's creature's ability can't be activated, the artifact card's
    // cycling ability can.
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_activate(&mut t, P1, sorcerer));
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P1, sorcerer));
    assert!(crate::r_s04_common::can_cycle(&mut t, P1, crystal));
    crate::r_s04_common::cycle(&mut t, P1, crystal, 0).expect("cycle");
    assert_eq!(t.zone(crystal), Zone::Graveyard(P1));
}

/// P0 controls Harsh Mentor; it's P1's main phase.
fn mentor() -> TestGame {
    supported("Harsh Mentor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Harsh Mentor");
    t.set_step(P1, Step::PrecombatMain);
    t
}

#[test]
fn harsh_mentor_what_counts_as_an_activated_non_mana_ability() {
    cr!("602.1", "605.1a", "702.6a", "701.43a");
    ruling!(
        "Harsh Mentor",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder text. An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate. Notably, exerting a creature isn't an activated ability."
    );
    supported("Bonesplitter");
    supported("Frozen Shade");
    supported("Nef-Crop Entangler");
    // Equip (an activated ability): 2 damage.
    let mut t = mentor();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bs = t.battlefield(P1, "Bonesplitter");
    colorless(&mut t, P1, 1);
    t.answer_targets(P1, &[obj(bears)]);
    activate_containing(&mut t, P1, bs, "Equip").expect("equip");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals 2 damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // A mana ability (Llanowar Elves): nothing.
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.activate(P1, elves, 0, &[]).expect("tap for mana");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // An ability that costs mana but doesn't produce it (Frozen Shade's "{B}: This
    // creature gets +1/+1 until end of turn."): 2 damage.
    let shade = t.battlefield(P1, "Frozen Shade");
    add_mana(&mut t, P1, ManaType::B, 1);
    t.activate(P1, shade, 0, &[]).expect("pump");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Exerting Nef-Crop Entangler as it attacks ("You may exert this creature as it
    // attacks. When you do, it gets +1/+2 until end of turn."): not an activated ability.
    let e = t.battlefield(P1, "Nef-Crop Entangler");
    t.answer_yes(P1, true);
    crate::r_s01_common::attack_with(&mut t, &[(e, Entity::Player(P0))]);
    assert!(t.obj_now(e).exerted);
    assert_eq!(triggers_on_stack(&t, "deals 2 damage"), 0);
    t.resolve_all();
    assert_eq!(t.pt(e), (3, 3));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn harsh_mentor_ignores_abilities_of_cards_in_hand_or_graveyard() {
    cr!("602.1", "702.29a", "702.128a");
    ruling!(
        "Harsh Mentor",
        "Harsh Mentor's ability doesn't trigger when an opponent activates an ability of a card in hand (such as cycling) or a card in a graveyard (such as embalm), even if that causes a card to be put onto the battlefield."
    );
    supported("Sacred Cat");
    let mut t = mentor();
    let crystal = t.hand(P1, "Ketria Crystal");
    colorless(&mut t, P1, 2);
    crate::r_s04_common::cycle(&mut t, P1, crystal, 0).expect("cycle");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals 2 damage"), 0);
    t.resolve_all();
    // Embalm {W}: a token copy of Sacred Cat is created.
    let cat = t.graveyard(P1, "Sacred Cat");
    add_mana(&mut t, P1, ManaType::W, 1);
    activate_containing(&mut t, P1, cat, "Embalm").expect("embalm");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals 2 damage"), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sacred Cat").len(), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn harsh_mentor_resolves_before_the_ability() {
    cr!("603.3", "405.2", "602.2a");
    ruling!(
        "Harsh Mentor",
        "Harsh Mentor's ability resolves before the ability that caused it to trigger."
    );
    let mut t = mentor();
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)])
        .expect("ping");
    t.settle();
    let items = stack_items(&t);
    assert_eq!(items.len(), 2);
    assert!(items[1].contains("deals 2 damage"), "{items:?}");
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    t.resolve();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn tomik_overrides_permission_to_play_lands_from_graveyards() {
    cr!("101.2", "305.1");
    ruling!(
        "Tomik, Distinguished Advokist",
        "Players can’t normally play a land card from a graveyard. If a player finds a way to do so, Tomik’s last ability takes precedence over that permission."
    );
    supported("Tomik, Distinguished Advokist");
    supported("Crucible of Worlds");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P1, "Crucible of Worlds");
    let forest = t.graveyard(P1, "Forest");
    assert!(can_play_land(&mut t, P1, forest));
    // A land card in P1's own graveyard without Crucible: no.
    let mut u = TestGame::new(2);
    u.set_step(P1, Step::PrecombatMain);
    let f2 = u.graveyard(P1, "Forest");
    assert!(!can_play_land(&mut u, P1, f2));
    // With P0's Tomik: no, despite Crucible of Worlds.
    t.battlefield(P0, "Tomik, Distinguished Advokist");
    assert!(!can_play_land(&mut t, P1, forest));
    assert!(t.play_land(P1, forest).is_err());
    assert_eq!(t.zone(forest), Zone::Graveyard(P1));
}

#[test]
fn tomiks_lands_dont_have_hexproof() {
    cr!("702.11b", "115.1");
    ruling!(
        "Tomik, Distinguished Advokist",
        "Tomik’s middle ability is similar to hexproof, but effects that interact with hexproof don’t count these lands or land cards as having hexproof."
    );
    supported("Shadowspear");
    supported("Stone Rain");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tomik, Distinguished Advokist");
    let land = t.battlefield(P0, "Forest");
    assert!(!t.obj_now(land).has_keyword(KeywordKind::Hexproof));
    // Shadowspear's "{1}: Permanents your opponents control lose hexproof and
    // indestructible until end of turn." doesn't let P1 target the land.
    let spear = t.battlefield(P1, "Shadowspear");
    t.set_step(P1, Step::PrecombatMain);
    colorless(&mut t, P1, 1);
    activate_containing(&mut t, P1, spear, "lose hexproof").expect("activate");
    t.resolve_all();
    let from = t.asked().len();
    t.lands(P1, "Mountain", 3);
    let rain = t.hand(P1, "Stone Rain");
    let r = t.cast(P1, rain).target(land).try_go();
    let offered = crate::r_s02_common::target_candidates(&t, P1, from)
        .iter()
        .any(|c| c.contains(&obj(land)));
    assert!(r.is_err() || !offered);
    t.clear_answers();
    t.resolve_all();
    assert!(t.on_battlefield(land));
}

#[test]
fn meddling_mage_names_as_it_enters_with_no_window() {
    cr!("614.12", "614.1c", "101.2");
    ruling!(
        "Meddling Mage",
        "No one can cast spells or activate abilities between the time a card is named and the time that Meddling Mage's ability begins to apply."
    );
    supported("Meddling Mage");
    let mut t = TestGame::new(2);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::NameCard { .. }),
        |g| g.battlefield.len(),
    );
    let before = t.g.battlefield.len();
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    let from = t.asked().len();
    let mage = cast_new(&mut t, P0, "Meddling Mage", &[]);
    let lands = t.g.battlefield.len() - before;
    t.resolve();
    // The name was chosen during the resolution, before the Mage was on the battlefield,
    // and nobody got priority between the choice and the Mage entering.
    assert_eq!(seen.lock().unwrap().as_slice(), &[before + lands]);
    let asked = t.asked()[from..].to_vec();
    let named = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::NameCard { .. }))
        .expect("named");
    assert!(!asked[named..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(t.on_battlefield(mage));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(!can_cast(&mut t, P1, bolt, CastMethod::Normal));
}

#[test]
fn meddling_mage_doesnt_affect_spells_already_on_the_stack() {
    cr!("601.2", "608.2");
    ruling!(
        "Meddling Mage",
        "Spells with the chosen name that somehow happen to already be on the stack when Meddling Mage enters the battlefield aren't affected by Meddling Mage's ability."
    );
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    t.enter(P0, "Meddling Mage");
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

/// `p` casts Sylvan Scrying ("Search your library for a land card, reveal it, put it into
/// your hand, then shuffle.") with a Forest on top of their library; whether they found it.
fn scrying(t: &mut TestGame, p: PlayerId) -> bool {
    let forest = t.library_top(p, "Forest");
    t.answer_choose(p, &[obj(forest)]);
    add_mana(t, p, ManaType::G, 1);
    add_mana(t, p, ManaType::C, 1);
    let s = t.hand(p, "Sylvan Scrying");
    t.cast(p, s).go();
    t.resolve_all();
    t.clear_answers();
    t.zone(forest) == Zone::Hand(p)
}

#[test]
fn leonin_arbiter_multiple_arbiters_and_paying_once() {
    cr!("116.2d", "701.23a");
    ruling!(
        "Leonin Arbiter",
        "If there are multiple Leonin Arbiters on the battlefield, a player must pay {2} for each one before being able to search libraries that turn."
    );
    ruling!(
        "Leonin Arbiter",
        "Once a player has paid {2}, they may search libraries that turn as many times as they are instructed to do so."
    );
    supported("Leonin Arbiter");
    supported("Sylvan Scrying");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Leonin Arbiter");
    let b = t.battlefield(P1, "Leonin Arbiter");
    colorless(&mut t, P0, 2);
    let sa = static_action(&mut t, P0, a).expect("pay for the first");
    take_special(&mut t, P0, sa);
    assert!(!scrying(&mut t, P0));
    colorless(&mut t, P0, 2);
    let sa = static_action(&mut t, P0, b).expect("pay for the second");
    take_special(&mut t, P0, sa);
    assert!(scrying(&mut t, P0));
    assert!(scrying(&mut t, P0));
}

#[test]
fn leonin_arbiter_paying_doesnt_search() {
    cr!("116.2d", "701.23a");
    ruling!(
        "Leonin Arbiter",
        "Paying {2} doesn't let a player pick up a library and search it — it just allows them to ignore Leonin Arbiter's effect that turn. That player can search a library only if another spell or ability instructs them to do so."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Leonin Arbiter");
    let library: Vec<ObjectId> = t.g.players[0].library.clone();
    let hand = t.hand_size(P0);
    colorless(&mut t, P0, 2);
    let decisions = t.asked().len();
    let sa = static_action(&mut t, P0, a).expect("pay");
    take_special(&mut t, P0, sa);
    // No search happens: no choice is offered and the library is untouched.
    assert!(!t.asked()[decisions..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(t.g.players[0].library, library);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn leonin_arbiter_the_rest_of_the_effect_still_works() {
    cr!("701.23a", "609.3");
    ruling!(
        "Leonin Arbiter",
        "Since players who haven't paid {2} can't search, they won't be able to find any cards in a library. The effect applies to all players and all libraries. If a spell or ability's effect has other parts that don't depend on searching for or finding cards, they will still work normally."
    );
    supported("Path to Exile");
    // P0's own Arbiter stops P0's search too; Path to Exile still exiles P1's creature, and
    // P1 can't search for a land.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leonin Arbiter");
    assert!(!scrying(&mut t, P0));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.library_top(P1, "Forest");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(forest)]);
    cast_new(&mut t, P0, "Path to Exile", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(forest), Zone::Library(P1));
    assert!(t
        .g
        .permanents()
        .all(|o| !(o.controller == P1 && o.is(CardType::Land))));
}
