//! Rulings batch P148 — mana with spending restrictions (CR 106.6): what a spell's "total
//! cost" includes (additional and alternative costs, CR 601.2f), which abilities count
//! ("abilities of [quality] sources" includes cards in hands and graveyards, "abilities of
//! [quality]s" only permanents), and that the mana may be split between costs.

use crate::r_p148_common::*;
use crate::r_s04_common::cycle;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

const EVOKE: CastMethod = CastMethod::Keyword(KeywordKind::Evoke);

/// Queues `p`'s choice of a color of mana ("W", "U", "B", "R" or "G") for the next mana
/// ability that asks.
fn color(t: &mut TestGame, p: PlayerId, c: &str) {
    let i = ["W", "U", "B", "R", "G"]
        .iter()
        .position(|x| *x == c)
        .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// `p` activates `source`'s `idx`th activated ability (a mana ability) and the mana is
/// in `p`'s pool.
fn produce(t: &mut TestGame, p: PlayerId, source: ObjectId, idx: usize) {
    t.activate(p, source, idx, &[]).expect("mana ability");
    assert!(pool_total(t, p) > 0, "no mana was added");
}

// --- Creature-spell mana: the whole total cost -------------------------------------------

/// With `source` (P0's) as the source of the mana a creature spell's kicker or evoke cost
/// needs, the spell can be cast.
fn pays_kicker_and_evoke(source: &str, kicker_lands: usize, evoke_lands: &[&str]) {
    supported(source);
    let mut t = TestGame::new(2);
    t.battlefield(P0, source);
    t.lands(P0, "Forest", kicker_lands);
    // Kavu Titan: {1}{G}, kicker {2}{G}.
    let kavu = t.hand(P0, "Kavu Titan");
    t.cast(P0, kavu).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.counters(kavu, counters::PLUS1), 3, "{source}: not kicked");
    // Mulldrifter's evoke cost {2}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, source);
    for l in evoke_lands {
        t.lands(P0, l, 1);
    }
    let drifter = t.hand(P0, "Mulldrifter");
    t.cast(P0, drifter).method(EVOKE).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mulldrifter"), "{source}: not evoked");
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn ancient_ziggurat_pays_additional_and_alternative_costs() {
    cr!("106.6", "601.2f", "118.9");
    ruling!(
        "Ancient Ziggurat",
        "Mana produced by Ancient Ziggurat can be spent on any part of a creature spell's total cost. This includes additional costs (such as kicker) and alternative costs"
    );
    pays_kicker_and_evoke("Ancient Ziggurat", 4, &["Island", "Wastes"]);
}

#[test]
fn somberwald_sage_pays_additional_and_alternative_costs() {
    cr!("106.6", "601.2f", "118.9");
    ruling!(
        "Somberwald Sage",
        "Mana produced by Somberwald Sage can be spent on any part of a creature spell's total cost. This includes additional costs (such as kicker) and alternative costs (such as evoke costs)."
    );
    pays_kicker_and_evoke("Somberwald Sage", 2, &[]);
}

#[test]
fn humble_naturalist_pays_kicker_but_not_creature_abilities() {
    cr!("106.6", "601.2f");
    ruling!(
        "Humble Naturalist",
        "Mana produced by Humble Naturalist can be spent on any part of a creature spell’s total cost, including additional costs (such as kicker costs) and alternative costs (such as mutate costs). It can’t be spent to pay the costs of abilities of creatures you control."
    );
    supported("Humble Naturalist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Humble Naturalist");
    t.lands(P0, "Forest", 4);
    let kavu = t.hand(P0, "Kavu Titan");
    t.cast(P0, kavu).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.counters(kavu, counters::PLUS1), 3);
    // Carrion Ants' "{1}: +1/+1" can't be paid with it.
    let mut t = TestGame::new(2);
    let nat = t.battlefield(P0, "Humble Naturalist");
    let ants = t.battlefield(P0, "Carrion Ants");
    produce(&mut t, P0, nat, 0);
    assert!(t.activate(P0, ants, 0, &[]).is_err());
    assert_eq!(pool_total(&t, P0), 1);
}

/// `source`'s restricted mana (from its `idx`th ability) can't pay Dregscape Zombie's
/// unearth cost ({B}).
fn cant_unearth_with(source: &str, idx: usize) {
    supported(source);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, source);
    let zombie = t.graveyard(P0, "Dregscape Zombie");
    color(&mut t, P0, "B");
    produce(&mut t, P0, s, idx);
    assert!(
        activate_containing(&mut t, P0, zombie, "Unearth").is_err(),
        "{source}'s mana paid for unearth"
    );
    assert!(t.in_graveyard(P0, "Dregscape Zombie"));
}

#[test]
fn ancient_ziggurat_cant_pay_unearth() {
    cr!("106.6", "702.84a");
    ruling!(
        "Ancient Ziggurat",
        "Mana produced by Ancient Ziggurat can't be spent on activated abilities that put a creature card directly onto the battlefield, such as unearth or ninjutsu."
    );
    cant_unearth_with("Ancient Ziggurat", 0);
}

#[test]
fn somberwald_sage_cant_pay_abilities_or_token_making_spells() {
    cr!("106.6", "702.84a");
    ruling!(
        "Somberwald Sage",
        "Mana produced by Somberwald Sage can't be spent on activated abilities, even ones that put a creature card directly onto the battlefield, such as unearth or ninjutsu."
    );
    ruling!(
        "Somberwald Sage",
        "Mana produced by Somberwald Sage can't be spent on noncreature spells that would put creature tokens onto the battlefield."
    );
    cant_unearth_with("Somberwald Sage", 0);
    let mut t = TestGame::new(2);
    let sage = t.battlefield(P0, "Somberwald Sage");
    color(&mut t, P0, "W");
    produce(&mut t, P0, sage, 0);
    let alarm = t.hand(P0, "Raise the Alarm");
    assert!(t.cast(P0, alarm).try_go().is_err());
    assert_eq!(pool_total(&t, P0), 3);
}

#[test]
fn primal_beyond_and_smokebraider_pay_an_elementals_evoke_cost() {
    cr!("106.6", "601.2f", "118.9");
    ruling!(
        "Primal Beyond",
        "You can use the mana produced by Primal Beyond's last ability to pay an alternative cost (such as evoke) or additional cost incurred while casting an Elemental spell."
    );
    ruling!(
        "Smokebraider",
        "You can use this mana to pay an alternative cost (such as evoke) or additional cost incurred while casting an Elemental spell."
    );
    // Mulldrifter (an Elemental) evoked for {2}{U}.
    for (source, idx, other) in [("Primal Beyond", 1, 2), ("Smokebraider", 0, 1)] {
        supported(source);
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, source);
        t.lands(P0, "Wastes", other);
        color(&mut t, P0, "U");
        color(&mut t, P0, "U");
        produce(&mut t, P0, s, idx);
        let drifter = t.hand(P0, "Mulldrifter");
        t.cast(P0, drifter).method(EVOKE).go();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Mulldrifter"), "{source}");
        assert_eq!(t.hand_size(P0), 2, "{source}");
    }
}

// --- "[Quality] sources" (any zone) and "[quality]s" (permanents) ------------------------

#[test]
fn elementals_mana_and_elemental_cards_in_hand() {
    cr!("106.6", "702.29a", "702.77a");
    ruling!(
        "Primal Beyond",
        "The mana can't be spent to activate activated abilities of Elemental sources that aren't on the battlefield (such as the reinforce ability of an Elemental card in your hand)."
    );
    ruling!(
        "Smokebraider",
        "The mana can't be spent to activate activated abilities of Elemental sources that aren't on the battlefield."
    );
    ruling!(
        "Flamebraider",
        "\"Elemental sources\" include any objects with the creature type Elemental."
    );
    supported("Brighthearth Banneret");
    supported("Granitic Titan");
    // Primal Beyond: Brighthearth Banneret's reinforce {1}{R} from hand.
    let mut t = TestGame::new(2);
    let beyond = t.battlefield(P0, "Primal Beyond");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    let banneret = t.hand(P0, "Brighthearth Banneret");
    color(&mut t, P0, "R");
    produce(&mut t, P0, beyond, 1);
    assert!(activate_containing(&mut t, P0, banneret, "Reinforce")
        .map(|_| ())
        .is_err());
    assert!(t.in_hand(P0, "Brighthearth Banneret"));
    let _ = bears;
    // Smokebraider: Granitic Titan's cycling {2} from hand.
    let mut t = TestGame::new(2);
    let smoke = t.battlefield(P0, "Smokebraider");
    let titan = t.hand(P0, "Granitic Titan");
    color(&mut t, P0, "R");
    color(&mut t, P0, "R");
    produce(&mut t, P0, smoke, 0);
    assert!(cycle(&mut t, P0, titan, 0).is_err());
    assert_eq!(pool_total(&t, P0), 2);
    // Flamebraider's "Elemental sources": it can, from hand or graveyard.
    let mut t = TestGame::new(2);
    let flame = t.battlefield(P0, "Flamebraider");
    let titan = t.hand(P0, "Granitic Titan");
    color(&mut t, P0, "R");
    color(&mut t, P0, "R");
    produce(&mut t, P0, flame, 0);
    cycle(&mut t, P0, titan, 0).unwrap();
    assert_eq!(pool_total(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Granitic Titan"));
    let mut t = TestGame::new(2);
    let flame = t.battlefield(P0, "Flamebraider");
    t.battlefield(P0, "Grizzly Bears");
    let hellspark = t.graveyard(P0, "Hellspark Elemental");
    color(&mut t, P0, "R");
    color(&mut t, P0, "R");
    produce(&mut t, P0, flame, 0);
    activate_containing(&mut t, P0, hellspark, "Unearth").unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hellspark Elemental").len(), 1);
}

#[test]
fn artifact_mana_and_artifact_cards_in_hand() {
    cr!("106.6", "702.29a");
    ruling!(
        "Renowned Weaponsmith",
        "The mana generated by Renowned Weaponsmith’s first ability can’t be spent to activate abilities of artifact sources that aren’t on the battlefield."
    );
    ruling!(
        "Dalakos, Crafter of Wonders",
        "You can’t spend mana generated by Dalakos’s first ability to activate abilities of artifact cards that aren’t on the battlefield"
    );
    ruling!(
        "Steelswarm Operator",
        "An “artifact source” is any object with the card type artifact. This means you could spend the mana from Steelswarm Operator’s last ability to activate an ability of an artifact you control or an artifact card in your hand or graveyard, for example."
    );
    supported("Indatha Crystal");
    // Indatha Crystal's cycling {2} from hand.
    for (source, idx, ok) in [
        ("Renowned Weaponsmith", 0, false),
        ("Dalakos, Crafter of Wonders", 0, false),
        ("Steelswarm Operator", 1, true),
    ] {
        supported(source);
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, source);
        let crystal = t.hand(P0, "Indatha Crystal");
        produce(&mut t, P0, s, idx);
        assert_eq!(pool_total(&t, P0), 2);
        assert_eq!(cycle(&mut t, P0, crystal, 0).is_ok(), ok, "{source}");
        assert_eq!(t.in_hand(P0, "Indatha Crystal"), !ok, "{source}");
    }
}

#[test]
fn renowned_weaponsmith_pays_equip() {
    cr!("106.6", "702.6a");
    ruling!(
        "Renowned Weaponsmith",
        "Some keyword abilities, such as equip, are activated abilities and will have colons in their reminder text."
    );
    let mut t = TestGame::new(2);
    let smith = t.battlefield(P0, "Renowned Weaponsmith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    produce(&mut t, P0, smith, 0);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, splitter, "Equip").unwrap();
    t.resolve_all();
    assert_eq!(t.g.obj(splitter).attached_to, Some(obj(bears)));
    assert_eq!(pool_total(&t, P0), 1);
    // Not to cast a nonartifact spell.
    let shock = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, shock).try_go().is_err());
}

#[test]
fn eldrazi_temple_and_eldrazi_cards_in_hand() {
    cr!("106.6", "702.29a");
    ruling!(
        "Eldrazi Temple",
        "The mana generated by the last ability can’t be spent to activate abilities of Eldrazi sources that aren’t on the battlefield."
    );
    supported("Drownyard Lurker");
    // Drownyard Lurker's cycling {2}{U} from hand: the Temple's {C}{C} and an Island.
    let mut t = TestGame::new(2);
    let temple = t.battlefield(P0, "Eldrazi Temple");
    t.lands(P0, "Island", 1);
    let lurker = t.hand(P0, "Drownyard Lurker");
    produce(&mut t, P0, temple, 1);
    assert_eq!(pool_total(&t, P0), 2);
    assert!(cycle(&mut t, P0, lurker, 0).is_err());
    // It can cast that Eldrazi spell.
    t.lands(P0, "Wastes", 5);
    t.cast(P0, lurker).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Drownyard Lurker").len(), 1);
}

#[test]
fn crucible_of_the_spirit_dragon_and_dragons() {
    cr!("106.6", "602.1", "702.29a");
    ruling!(
        "Crucible of the Spirit Dragon",
        "The mana generated by the last ability can't be spent to activate abilities of Dragon sources that aren't on the battlefield."
    );
    ruling!(
        "Crucible of the Spirit Dragon",
        "An activated ability appears in the form “Cost: Effect.”"
    );
    supported("Timeless Dragon");
    let mut t = TestGame::new(2);
    let crucible = t.battlefield(P0, "Crucible of the Spirit Dragon");
    t.g.add_counters(Entity::Object(crucible), "storage", 2, None);
    let shivan = t.battlefield(P0, "Shivan Dragon");
    let timeless = t.hand(P0, "Timeless Dragon");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    color(&mut t, P0, "R");
    color(&mut t, P0, "R");
    produce(&mut t, P0, crucible, 2);
    assert_eq!(pool_total(&t, P0), 2);
    // Plainscycling a Dragon card in hand: no.
    assert!(cycle(&mut t, P0, timeless, 0).is_err());
    // A Dragon's activated ability ("{R}: +1/+0"): yes.
    t.activate(P0, shivan, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(shivan).0, 6);
}

#[test]
fn crucible_of_the_spirit_dragon_pays_a_dash_cost() {
    cr!("106.6", "118.9", "702.109a");
    ruling!(
        "Crucible of the Spirit Dragon",
        "You can use mana generated by the last ability to pay an alternative cost (such as a dash cost) or an additional cost to cast a Dragon spell."
    );
    supported("Kolaghan, the Storm's Fury");
    let mut t = TestGame::new(2);
    let crucible = t.battlefield(P0, "Crucible of the Spirit Dragon");
    t.g.add_counters(Entity::Object(crucible), "storage", 5, None);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    for c in ["B", "R", "R", "R", "R"] {
        color(&mut t, P0, c);
    }
    produce(&mut t, P0, crucible, 2);
    assert_eq!(pool_total(&t, P0), 5);
    let kolaghan = t.hand(P0, "Kolaghan, the Storm's Fury");
    t.cast(P0, kolaghan)
        .method(CastMethod::Keyword(KeywordKind::Dash))
        .go();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(
        t.named_on_battlefield("Kolaghan, the Storm's Fury").len(),
        1
    );
}

#[test]
fn automated_artificer_pays_a_channel_ability() {
    cr!("106.6", "702.29a");
    ruling!(
        "Automated Artificer",
        "Mana from Automated Artificer's ability can be spent to activate any ability, including those activated from off the battlefield, such as ninjutsu or channel abilities."
    );
    supported("Automated Artificer");
    supported("Boseiju, Who Endures");
    let mut t = TestGame::new(2);
    let art = t.battlefield(P0, "Automated Artificer");
    t.lands(P0, "Forest", 1);
    let boseiju = t.hand(P0, "Boseiju, Who Endures");
    let target = t.battlefield(P1, "Glorious Anthem");
    produce(&mut t, P0, art, 0);
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(&mut t, P0, boseiju, "Discard ~").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn any_activated_ability_not_just_colorless_or_artifact_sources() {
    cr!("106.6");
    ruling!(
        "Sage of the Unknowable",
        "The mana generated by Sage of the Unknowable's activated ability can be used to activate any ability, not just abilities of colorless sources."
    );
    ruling!(
        "Guidelight Optimizer",
        "You may use mana from this creature to pay the cost of any activated ability, not just abilities of artifacts."
    );
    for source in ["Sage of the Unknowable", "Guidelight Optimizer"] {
        supported(source);
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, source);
        // Carrion Ants (black): "{1}: +1/+1".
        let ants = t.battlefield(P0, "Carrion Ants");
        produce(&mut t, P0, s, 0);
        t.activate(P0, ants, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.pt(ants), (1, 2), "{source}");
    }
}

// --- Cryptic Trilobite, James ----------------------------------------------------------------

#[test]
fn cryptic_trilobite_x_x_costs_twice_x() {
    cr!("107.3", "601.2f");
    ruling!(
        "Cryptic Trilobite",
        "A mana cost of {X}{X} means that you pay twice X. If you want X to be 3, you pay {6} to cast Cryptic Trilobite."
    );
    supported("Cryptic Trilobite");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 6);
    let tri = t.hand(P0, "Cryptic Trilobite");
    assert!(t.cast(P0, tri).x(4).try_go().is_err());
    let tri = t.g.current(tri);
    t.cast(P0, tri).x(3).go();
    t.resolve_all();
    assert_eq!(t.counters(tri, counters::PLUS1), 3);
    assert_eq!(crate::r_p148_common::untapped_lands(&t, P0), 0);
}

#[test]
fn cryptic_trilobite_mana_pays_activated_abilities_including_two_different_ones() {
    cr!("106.6", "602.1");
    ruling!(
        "Cryptic Trilobite",
        "Activated abilities contain a colon, such as Cryptic Trilobite’s activated mana ability and its last activated ability."
    );
    ruling!(
        "Cryptic Trilobite",
        "You can spend the two colorless mana produced by Cryptic Trilobite on two different activated abilities."
    );
    let mut t = TestGame::new(2);
    let tri = t.battlefield(P0, "Cryptic Trilobite");
    t.g.add_counters(Entity::Object(tri), counters::PLUS1, 3, None);
    let icy1 = t.battlefield(P0, "Icy Manipulator");
    let icy2 = t.battlefield(P0, "Icy Manipulator");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    t.activate(P0, tri, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 2);
    // Not a spell.
    let walker = t.hand(P0, "Bonesplitter");
    assert!(t.cast(P0, walker).try_go().is_err());
    t.activate(P0, icy1, 0, &[obj(b1)]).unwrap();
    t.activate(P0, icy2, 0, &[obj(b2)]).unwrap();
    t.resolve_all();
    assert!(t.g.obj(b1).tapped && t.g.obj(b2).tapped);
    assert_eq!(pool_total(&t, P0), 0);
    // Its own last ability ("{1}, {T}: Put a +1/+1 counter").
    t.activate(P0, tri, 0, &[]).unwrap();
    t.activate(P0, tri, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(tri, counters::PLUS1), 2);
}

#[test]
fn james_mana_pays_activated_abilities_only() {
    cr!("106.6", "602.1");
    ruling!(
        "James, Wandering Dad // Follow Him",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\""
    );
    supported("James, Wandering Dad // Follow Him");
    let mut t = TestGame::new(2);
    let james = t.battlefield(P0, "James, Wandering Dad // Follow Him");
    let icy = t.battlefield(P0, "Icy Manipulator");
    let bears = t.battlefield(P1, "Grizzly Bears");
    produce(&mut t, P0, james, 0);
    let splitter = t.hand(P0, "Bonesplitter");
    assert!(t.cast(P0, splitter).try_go().is_err());
    t.activate(P0, icy, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.g.obj(bears).tapped);
}

// --- Sunken Citadel, Intrepid Stablemaster: splitting the mana ------------------------------

#[test]
fn sunken_citadel_land_sources_and_two_abilities() {
    cr!("106.6", "702.29a");
    ruling!(
        "Sunken Citadel",
        "\"Land sources\" include any objects with the card type land. This means you could spend the mana to activate an ability of a land you control or a land card in your hand or graveyard"
    );
    ruling!(
        "Sunken Citadel",
        "You may spend the two mana added by the last ability on the same ability or on two different abilities."
    );
    supported("Sunken Citadel");
    let mut t = TestGame::new(2);
    color(&mut t, P0, "U");
    let citadel = t.enter(P0, "Sunken Citadel");
    t.g.untap(citadel);
    produce(&mut t, P0, citadel, 1);
    assert_eq!(pool_total(&t, P0), 2);
    // Not a spell.
    let walker = t.hand(P0, "Bonesplitter");
    assert!(t.cast(P0, walker).try_go().is_err());
    let v1 = t.battlefield(P0, "Mutavault");
    let v2 = t.battlefield(P0, "Mutavault");
    // Two different abilities of lands: each Mutavault's "{1}: becomes a 2/2 creature".
    activate_containing(&mut t, P0, v1, "becomes").unwrap();
    activate_containing(&mut t, P0, v2, "becomes").unwrap();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
    assert!(t.g.obj(v1).is(CardType::Creature) && t.g.obj(v2).is(CardType::Creature));
    // A land card in hand: Lonely Sandbar's cycling {U}.
    let mut t = TestGame::new(2);
    color(&mut t, P0, "U");
    let citadel = t.enter(P0, "Sunken Citadel");
    t.g.untap(citadel);
    let sandbar = t.hand(P0, "Lonely Sandbar");
    produce(&mut t, P0, citadel, 1);
    cycle(&mut t, P0, sandbar, 0).unwrap();
    assert!(t.in_graveyard(P0, "Lonely Sandbar"));
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn intrepid_stablemaster_two_different_vehicle_spells() {
    cr!("106.6");
    ruling!(
        "Intrepid Stablemaster",
        "You may spend the two mana added by the last ability on the same Mount or Vehicle spell or on two different Mount or Vehicle spells."
    );
    supported("Intrepid Stablemaster");
    supported("Consulate Dreadnought");
    let mut t = TestGame::new(2);
    let stable = t.battlefield(P0, "Intrepid Stablemaster");
    color(&mut t, P0, "G");
    produce(&mut t, P0, stable, 1);
    assert_eq!(pool_total(&t, P0), 2);
    for _ in 0..2 {
        let d = t.hand(P0, "Consulate Dreadnought");
        t.cast(P0, d).go();
        t.resolve_all();
    }
    assert_eq!(t.named_on_battlefield("Consulate Dreadnought").len(), 2);
    assert_eq!(pool_total(&t, P0), 0);
}

// --- Spell types ----------------------------------------------------------------------------

#[test]
fn tournament_grounds_knight_spells_only_and_no_equip() {
    cr!("106.6", "205.3", "702.6a");
    ruling!(
        "Tournament Grounds",
        "If an effect refers to a “[subtype] spell,” it refers only to a spell that has that subtype. For example, Knights' Charge is a card with “Knight” in its name and benefits Knights, but it isn't a Knight card."
    );
    ruling!(
        "Worthy Knight",
        "If an effect refers to a “[subtype] spell,” it refers only to a spell that has that subtype."
    );
    ruling!(
        "Tournament Grounds",
        "Mana produced by the second ability of Tournament Grounds can't be spent to activate abilities, including equip abilities."
    );
    supported("Tournament Grounds");
    supported("Worthy Knight");
    supported("Knights' Charge");
    // Knights' Charge ({1}{W}{B}) isn't a Knight spell.
    let mut t = TestGame::new(2);
    let grounds = t.battlefield(P0, "Tournament Grounds");
    t.battlefield(P0, "Worthy Knight");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    produce(&mut t, P0, grounds, 1);
    let charge = t.hand(P0, "Knights' Charge");
    assert!(t.cast(P0, charge).try_go().is_err());
    // Cast with other mana, Worthy Knight doesn't trigger.
    t.lands(P0, "Wastes", 1);
    empty_pool(&mut t, P0);
    let charge = t.g.current(charge);
    t.cast(P0, charge).go();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.named_on_battlefield("Human").is_empty());
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 0);
    // A Knight spell: yes, and Worthy Knight triggers.
    let mut t = TestGame::new(2);
    let grounds = t.battlefield(P0, "Tournament Grounds");
    t.battlefield(P0, "Worthy Knight");
    t.lands(P0, "Plains", 1);
    produce(&mut t, P0, grounds, 1);
    let knight = t.hand(P0, "Worthy Knight");
    t.cast(P0, knight).go();
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    // Equip: no.
    let mut t = TestGame::new(2);
    let grounds = t.battlefield(P0, "Tournament Grounds");
    t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    produce(&mut t, P0, grounds, 1);
    assert!(activate_containing(&mut t, P0, splitter, "Equip").is_err());
}

#[test]
fn corrupted_crossroads_colorless_spell_needs_devoid() {
    cr!("106.6", "702.114a");
    ruling!(
        "Corrupted Crossroads",
        "The mana produced by the last ability can’t be spent on a colorless spell unless that spell specifically has the devoid ability."
    );
    supported("Corrupted Crossroads");
    let mut t = TestGame::new(2);
    let cross = t.battlefield(P0, "Corrupted Crossroads");
    color(&mut t, P0, "U");
    produce(&mut t, P0, cross, 1);
    assert_eq!(t.life(P0), 19);
    let splitter = t.hand(P0, "Bonesplitter");
    assert!(t.cast(P0, splitter).try_go().is_err());
    t.lands(P0, "Island", 1);
    let intruder = t.hand(P0, "Mist Intruder");
    t.cast(P0, intruder).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mist Intruder").len(), 1);
}

#[test]
fn white_lotus_hideout_lesson_spells_not_shrine_abilities() {
    cr!("106.6", "205.3");
    ruling!(
        "White Lotus Hideout",
        "A \"Lesson or Shrine spell\" is any spell with the subtype Lesson or Shrine. Mana produced by the second ability can't be spent to pay the costs of abilities of Shrines you control."
    );
    supported("White Lotus Hideout");
    supported("Sanctum of Shattered Heights");
    supported("Aang's Journey");
    let mut t = TestGame::new(2);
    let hideout = t.battlefield(P0, "White Lotus Hideout");
    t.battlefield(P0, "Sanctum of Shattered Heights");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Forest");
    color(&mut t, P0, "R");
    produce(&mut t, P0, hideout, 1);
    let sanctum = t.named_on_battlefield("Sanctum of Shattered Heights")[0];
    t.answer_targets(P0, &[obj(bears)]);
    assert!(t.activate(P0, sanctum, 0, &[obj(bears)]).is_err());
    assert_eq!(pool_total(&t, P0), 1);
    // A Lesson spell ({2}).
    t.lands(P0, "Wastes", 1);
    let journey = t.hand(P0, "Aang's Journey");
    t.cast(P0, journey).go();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn brotherhood_headquarters_freerunning_spells_and_changelings() {
    cr!("106.6", "702.73a");
    ruling!(
        "Brotherhood Headquarters",
        "You can use the mana produced by Brotherhood Headquarters’s last ability to cast a spell with freerunning even if you’re not paying that spell’s freerunning cost."
    );
    ruling!(
        "Brotherhood Headquarters",
        "This means you could spend the mana to activate an ability of a permanent with changeling"
    );
    supported("Brotherhood Headquarters");
    supported("Eagle Vision");
    let mut t = TestGame::new(2);
    let hq = t.battlefield(P0, "Brotherhood Headquarters");
    t.lands(P0, "Wastes", 4);
    color(&mut t, P0, "U");
    produce(&mut t, P0, hq, 1);
    let vision = t.hand(P0, "Eagle Vision");
    t.cast(P0, vision).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    // Mirror Entity (changeling): "{X}: ...".
    let mut t = TestGame::new(2);
    let hq = t.battlefield(P0, "Brotherhood Headquarters");
    let entity = t.battlefield(P0, "Mirror Entity");
    produce(&mut t, P0, hq, 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, entity, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(entity), (1, 1));
}

#[test]
fn rivaz_of_the_claw_mana_pays_a_dragons_kicker() {
    cr!("106.6", "601.2f");
    ruling!(
        "Rivaz of the Claw",
        "You can use mana generated by the activated ability to pay an alternative cost (such as an escape cost) or an additional cost to cast a Dragon spell."
    );
    supported("Rivaz of the Claw");
    supported("Verix Bladewing");
    let mut t = TestGame::new(2);
    let rivaz = t.battlefield(P0, "Rivaz of the Claw");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 3);
    color(&mut t, P0, "R");
    color(&mut t, P0, "R");
    produce(&mut t, P0, rivaz, 0);
    let verix = t.hand(P0, "Verix Bladewing");
    t.cast(P0, verix).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Karox Bladewing").len(), 1);
}

#[test]
fn smokebraider_two_mana_of_any_colors_never_colorless() {
    cr!("106.1a", "106.1b");
    ruling!(
        "Smokebraider",
        "The mana can be two mana of the same color, or one mana of each of two different colors. The mana can't be colorless."
    );
    let mut t = TestGame::new(2);
    let smoke = t.battlefield(P0, "Smokebraider");
    let from = t.asked().len();
    color(&mut t, P0, "R");
    color(&mut t, P0, "G");
    produce(&mut t, P0, smoke, 0);
    let pool = &t.g.player(P0).mana_pool.mana;
    assert_eq!(pool.len(), 2);
    use mtg_engine::mana::ManaType;
    assert_eq!(pool[0].ty, ManaType::R);
    assert_eq!(pool[1].ty, ManaType::G);
    for (_, d) in &t.asked()[from..] {
        if let Decision::ChooseOption { options, .. } = d {
            assert!(options.iter().all(|o| !o.contains('C')), "{options:?}");
        }
    }
}
