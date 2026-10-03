//! Rulings batch P201 — adamant, adapt, affinity, afterlife, airbend (experience
//! counters), annihilator, ascend, assist, and "whenever you cast ..." triggers that
//! resolve before the spell (Lattice Library, Murmuring Mystic, Rite of the Dragoncaller).

use crate::r_s01_common::{give_mana_for, supported, tokens};
use crate::r_s02_common::create_token as create_token_named;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn lands_tapped(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.tapped && o.is(mtg_engine::types::CardType::Land))
        .count()
}

// --- Adamant ---------------------------------------------------------------------------

#[test]
fn searing_barrage_adamant_damage_is_additional() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Searing Barrage",
        "Searing Barrage's adamant ability causes it to deal additional damage. It doesn't replace the damage dealt to the target creature."
    );
    supported("Searing Barrage");
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 5);
    let c = t.hand(P0, "Searing Barrage");
    t.cast(P0, c).target(dreadmaw).go();
    t.resolve_all();
    assert_eq!(t.obj_now(dreadmaw).damage, 5);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn cauldrons_gift_can_return_a_card_it_just_milled() {
    cr!("207.2c", "608.2c", "115.1");
    ruling!(
        "Cauldron's Gift",
        "The creature card you return with Cauldron's Gift may be one that you just put into your graveyard with its adamant ability."
    );
    supported("Cauldron's Gift");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    let c = t.hand(P0, "Cauldron's Gift");
    t.cast(P0, c).go();
    let from = t.asked().len();
    t.resolve_all();
    let giants = t.named_on_battlefield("Hill Giant");
    assert_eq!(giants.len(), 1, "{:?}", &t.asked()[from..]);
    assert_eq!(t.counters(giants[0], "+1/+1"), 1);
    assert_eq!(t.graveyard_size(P0), 4);
}

// --- Adapt -----------------------------------------------------------------------------

#[test]
fn two_cursed_wombats_grant_two_triggers() {
    cr!("701.46a", "603.2", "113.2c");
    ruling!(
        "Cursed Wombat",
        "If you control multiple Cursed Wombats, each permanent you control will have that many instances of the granted ability."
    );
    supported("Cursed Wombat");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Cursed Wombat");
    t.battlefield(P0, "Cursed Wombat");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve_all();
    // Two from adapt, then one from each of the two granted triggers; they trigger only
    // once each turn.
    assert_eq!(t.counters(w, "+1/+1"), 4);
    assert_eq!(t.stack_len(), 0);
}

// --- Affinity --------------------------------------------------------------------------

/// Whether P0 can cast `name` (with the given targets) with `n` copies of the enabler
/// `enabler` (a card, or a creature token of that type if `token`) and the given lands.
fn affinity_cast(
    name: &str,
    enabler: &str,
    token: bool,
    n: usize,
    lands: &[&str],
    targets: &[Entity],
) -> (bool, usize) {
    supported(name);
    let mut t = TestGame::new(2);
    for _ in 0..n {
        if token {
            create_token_named(&mut t, P0, enabler);
        } else {
            t.battlefield(P0, enabler);
        }
    }
    for l in lands {
        t.lands(P0, l, 1);
    }
    let c = t.hand(P0, name);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    if name == "Allies at Last" {
        // Up to two target creatures you control (none), and a creature an opponent
        // controls.
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[]);
        t.answer_targets(P0, &[Entity::Object(bears)]);
    }
    let ok = t.cast(P0, c).try_go().is_ok();
    (ok, lands_tapped(&t, P0))
}

#[test]
fn affinity_reduces_only_generic_mana() {
    cr!("702.41a", "601.2f", "118.7");
    ruling!(
        "Ethersworn Sphinx",
        "Affinity can reduce only the generic mana in a spell's total cost."
    );
    ruling!(
        "Allies at Last",
        "Affinity for Allies reduces only the generic mana in Allies at Last's cost."
    );
    ruling!(
        "Cantankerous Keepers",
        "Affinity for Elves reduces only the generic mana in the cost to cast Cantankerous Keepers."
    );
    // (spell, enabler, token?, enablers, targets): with more enablers than its generic
    // mana, the colored mana must still be paid: castable with exactly its colored lands,
    // not with colorless ones.
    let cases: Vec<(&str, &str, bool, usize, Vec<&str>, Vec<&str>)> = vec![
        (
            "Ethersworn Sphinx",
            "Ornithopter",
            false,
            9,
            vec!["Plains", "Island"],
            vec!["Wastes", "Wastes"],
        ),
        (
            "Allies at Last",
            "Ally",
            true,
            3,
            vec!["Forest"],
            vec!["Wastes"],
        ),
        (
            "Cantankerous Keepers",
            "Elf",
            true,
            7,
            vec!["Forest"],
            vec!["Wastes"],
        ),
    ];
    for (name, enabler, token, n, colored, colorless) in cases {
        let targets: Vec<Entity> = vec![];
        let (ok, tapped) = affinity_cast(name, enabler, token, n, &colored, &targets);
        assert!(ok, "{name}");
        assert_eq!(tapped, colored.len(), "{name}");
        let (ok, _) = affinity_cast(name, enabler, token, n, &colorless, &targets);
        assert!(!ok, "{name} with colorless mana");
    }
}

#[test]
fn affinity_for_creatures_and_multiple_instances() {
    cr!("702.41a", "702.41b", "601.2f");
    ruling!(
        "Witherbloom, the Balancer",
        "Affinity for creatures means \"This spell costs {1} less to cast for each creature you control.\""
    );
    ruling!(
        "Witherbloom, the Balancer",
        "If a spell has multiple instances of affinity, each one applies."
    );
    supported("Witherbloom, the Balancer");
    // Witherbloom ({6}{B}{G}) with five creatures: {1}{B}{G}.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.battlefield(P0, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 1);
    let w = t.hand(P0, "Witherbloom, the Balancer");
    t.cast(P0, w).go();
    assert_eq!(lands_tapped(&t, P0), 3);
    // Two Witherblooms (the legend rule not yet applied) and no other creatures: an
    // instant or sorcery spell costs {4} less. Overflowing Insight {4}{U}{U}{U}.
    supported("Overflowing Insight");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Witherbloom, the Balancer");
    t.battlefield(P0, "Witherbloom, the Balancer");
    t.lands(P0, "Island", 3);
    let oi = t.hand(P0, "Overflowing Insight");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.cast(P0, oi).go();
    assert_eq!(lands_tapped(&t, P0), 3);
}

// --- Afterlife -------------------------------------------------------------------------

const BESTOW: CastMethod = CastMethod::Keyword(KeywordKind::Bestow);

fn spirits(t: &TestGame) -> usize {
    tokens(t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Spirit"))
        .count()
}

#[test]
fn a_bestowed_indebted_spirit_has_afterlife_as_an_aura() {
    cr!("702.135a", "702.103e", "603.10a");
    ruling!(
        "Indebted Spirit",
        "If Indebted Spirit is put into a graveyard from the battlefield while it's an Aura, its afterlife ability will still trigger."
    );
    ruling!(
        "Indebted Spirit",
        "If a bestowed Indebted Spirit and the creature it's attached to are put into a graveyard from the battlefield at the same time, each of their afterlife abilities will trigger."
    );
    supported("Indebted Spirit");
    // The Aura is destroyed (Disenchant).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let c = t.hand(P0, "Indebted Spirit");
    let spirit = t.cast(P0, c).method(BESTOW).target(bears).go();
    t.resolve_all();
    let aura = t.g.current(spirit);
    assert!(t.on_battlefield(aura));
    assert_eq!(t.pt(bears), (3, 3));
    t.lands(P0, "Plains", 2);
    let dis = t.hand(P0, "Disenchant");
    t.cast(P0, dis).target(aura).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Indebted Spirit"));
    assert_eq!(spirits(&t), 1);
    // The Aura and the creature are destroyed together (Planar Cleansing): two Spirits.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let c = t.hand(P0, "Indebted Spirit");
    t.cast(P0, c).method(BESTOW).target(bears).go();
    t.resolve_all();
    t.lands(P1, "Plains", 6);
    t.set_step(P1, Step::PrecombatMain);
    let pc = t.hand(P1, "Planar Cleansing");
    t.cast(P1, pc).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Indebted Spirit"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(spirits(&t), 2);
}

// --- Airbend / experience counters -----------------------------------------------------

#[test]
fn aang_counts_all_experience_counters_which_stay_with_the_player() {
    cr!("122.1", "122.8", "503.1a");
    ruling!(
        "Aang, Airbending Master",
        "All experience counters are identical, no matter how you got them."
    );
    ruling!(
        "Aang, Airbending Master",
        "The experience counter goes on you, the player, not on Aang."
    );
    supported("Aang, Airbending Master");
    let mut t = TestGame::new(2);
    let aang = t.battlefield(P0, "Aang, Airbending Master");
    // One experience counter from elsewhere.
    t.g.players[0]
        .counters
        .insert(counters::EXPERIENCE.into(), 1);
    // A creature P0 controls leaves without dying (Unsummon).
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let un = t.hand(P0, "Unsummon");
    t.cast(P0, un).target(bears).go();
    t.resolve_all();
    assert_eq!(t.player(P0).counter(counters::EXPERIENCE), 2);
    assert_eq!(t.counters(aang, counters::EXPERIENCE), 0);
    // Aang dies: the counters stay with P0.
    t.g.destroy(aang, None);
    t.resolve_all();
    assert_eq!(t.player(P0).counter(counters::EXPERIENCE), 2);
    // Another Aang counts both at P0's upkeep.
    t.battlefield(P0, "Aang, Airbending Master");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let allies = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Ally"))
        .count();
    assert_eq!(allies, 2);
}

// --- Annihilator -----------------------------------------------------------------------

/// P0 attacks with `attacker`; returns once the declare attackers step has begun (with
/// the triggers on the stack).
fn attack(t: &mut TestGame, attacker: ObjectId, defender: Entity) {
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, defender)]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
}

#[test]
fn it_that_betrays_annihilator_resolves_before_blocks_and_takes_the_cards() {
    cr!("702.86a", "508.1m", "509.1a");
    ruling!(
        "It That Betrays",
        "Annihilator abilities trigger and resolve during the declare attackers step."
    );
    ruling!(
        "It That Betrays",
        "The second ability triggers whenever an opponent sacrifices a nontoken permanent for any reason, not just due to the annihilator ability."
    );
    supported("It That Betrays");
    let mut t = TestGame::new(2);
    let itb = t.battlefield(P0, "It That Betrays");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Forest");
    attack(&mut t, itb, Entity::Player(P1));
    t.answer_choose(P1, &[Entity::Object(bears), Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    // The sacrificed creatures can't block; P0 now controls them.
    let mine: Vec<_> =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.owner == P1)
            .map(|o| o.chars.name.to_string())
            .collect();
    assert_eq!(mine.len(), 2);
    assert!(mine.contains(&"Grizzly Bears".to_string()));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 9);
    // Any other sacrifice by an opponent: Diabolic Edict.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "It That Betrays");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let edict = t.hand(P0, "Diabolic Edict");
    t.cast(P0, edict).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t
        .g
        .permanents()
        .any(|o| o.controller == P0 && o.chars.name == "Grizzly Bears"));
    // Not tokens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "It That Betrays");
    let tok = create_token_named(&mut t, P1, "Soldier");
    t.g.sacrifice(tok, P1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn an_attacker_whose_planeswalker_was_sacrificed_keeps_attacking() {
    cr!("702.86a", "506.4c", "510.1c");
    ruling!(
        "It That Betrays",
        "If a creature with annihilator is attacking a planeswalker, and the defending player chooses to sacrifice that planeswalker, the attacking creature continues to attack."
    );
    ruling!(
        "Flayer of Loyalties",
        "If a creature with annihilator    is attacking a planeswalker, and the defending player chooses to sacrifice that planeswalker, the attacking creature continues to attack."
    );
    for name in ["It That Betrays", "Flayer of Loyalties"] {
        supported(name);
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, name);
        let jace = t.battlefield(P1, "Jace Beleren");
        let land = t.battlefield(P1, "Island");
        attack(&mut t, attacker, Entity::Object(jace));
        t.answer_choose(P1, &[Entity::Object(jace), Entity::Object(land)]);
        t.resolve_all();
        // (It That Betrays then puts the sacrificed Jace onto the battlefield under P0's
        // control: a new object.)
        if name == "It That Betrays" {
            let now = t.g.current(jace);
            assert!(now != jace && t.obj(now).controller == P0, "{name}");
        } else {
            assert!(t.in_graveyard(P1, "Jace Beleren"));
        }
        assert!(t.g.is_attacking(attacker), "{name}");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{name}");
    }
}

// --- Ascend ----------------------------------------------------------------------------

fn blessed(t: &TestGame, p: PlayerId) -> bool {
    t.g.player(p).has_citys_blessing
}

#[test]
fn detective_of_the_month_gives_no_blessing_until_it_resolves() {
    cr!("702.131b", "608.3");
    ruling!(
        "Detective of the Month",
        "If you cast a spell with ascend, you don't get the city’s blessing until it resolves."
    );
    supported("Detective of the Month");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Island", 8);
        let det = t.hand(P0, "Detective of the Month");
        t.cast(P0, det).go();
        t.settle();
        assert!(!blessed(&t, P0));
        if respond {
            t.lands(P1, "Mountain", 1);
            let bolt = t.hand(P1, "Lightning Bolt");
            t.cast(P1, bolt).target(bears).go();
        }
        t.resolve_all();
        assert_eq!(blessed(&t, P0), !respond);
    }
}

#[test]
fn ten_permanents_without_ascend_give_no_blessing() {
    cr!("702.131b", "702.131c");
    ruling!(
        "Andúril, Narsil Reforged",
        "If you control ten permanents but don't control a permanent or resolving spell with ascend, you don't get the city's blessing."
    );
    ruling!(
        "Illustrious Wanderglyph",
        "If you control ten permanents but don't control a permanent or resolving spell with ascend, you don't get the city's blessing."
    );
    ruling!(
        "Ocelot Pride",
        "If you control ten permanents but don't control a permanent or resolving spell with ascend, you don't get the city's blessing."
    );
    ruling!(
        "Detective of the Month",
        "If you control ten permanents but don’t control a permanent or resolving spell with ascend, you don’t get the city’s blessing."
    );
    for name in [
        "Andúril, Narsil Reforged",
        "Illustrious Wanderglyph",
        "Ocelot Pride",
        "Detective of the Month",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        give_mana_for(&mut t, P0, name);
        let k = t.g.permanents().filter(|o| o.controller == P0).count();
        let wastes = t.lands(P0, "Plains", 10 - k);
        t.settle();
        assert_eq!(t.g.permanents().filter(|o| o.controller == P0).count(), 10);
        assert!(!blessed(&t, P0));
        // Lose control of two (they leave), then cast the card with ascend.
        for l in &wastes[..2] {
            t.g.move_object(*l, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
        }
        t.settle();
        let c = t.hand(P0, name);
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(
            t.g.permanents().filter(|o| o.controller == P0).count(),
            9,
            "{name}"
        );
        assert!(!blessed(&t, P0), "{name}");
    }
}

// --- Assist ----------------------------------------------------------------------------

/// The maximum amounts P1 was offered to pay for assist since decision `from`.
fn assist_max(t: &TestGame, from: usize) -> Vec<i64> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseNumber { max, .. } if *p == P1 => Some(*max),
            _ => None,
        })
        .collect()
}

#[test]
fn assist_pays_any_amount_of_generic_mana_only() {
    cr!("702.132a", "601.2f");
    ruling!(
        "Fan Favorite",
        "Assist allows another player to pay for any amount of generic mana."
    );
    ruling!(
        "Fan Favorite",
        "Only the generic mana portion of a spell's cost can be paid with assist."
    );
    supported("Fan Favorite");
    // Sphere of Resistance: "Spells cost {1} more to cast." Fan Favorite costs {4}{B};
    // P1 may pay up to four.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    t.lands(P0, "Swamp", 1);
    let theirs = t.lands(P1, "Swamp", 5);
    let ff = t.hand(P0, "Fan Favorite");
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Number, Answer::Number(4));
    t.cast(P0, ff).go();
    assert_eq!(assist_max(&t, from), vec![4]);
    assert_eq!(theirs.iter().filter(|l| t.obj_now(**l).tapped).count(), 4);
    assert_eq!(lands_tapped(&t, P0), 1);
    // P0 can't have P1 pay the {B}: with only colorless mana P0 can't cast it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    t.lands(P1, "Swamp", 5);
    let ff = t.hand(P0, "Fan Favorite");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Number, Answer::Number(3));
    assert!(t.cast(P0, ff).try_go().is_err());
}

// --- "Whenever you cast" triggers resolve first ----------------------------------------

/// P0 casts Lightning Bolt at P1 with `src` on the battlefield; P1 counters it with
/// Cancel. Returns the game after everything resolved, and whether the trigger was on
/// top of the Bolt.
fn bolt_countered(src: &str) -> (TestGame, bool) {
    supported(src);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, src);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.settle();
    let top = *t.g.stack.last().unwrap();
    let on_top = top != bolt
        && matches!(&t.obj(top).stack.as_ref().unwrap().kind,
            mtg_engine::object::StackKind::Triggered { source, .. } if *source == s);
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(bolt).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    (t, on_top)
}

#[test]
fn cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Murmuring Mystic",
        "Murmuring Mystic's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Rite of the Dragoncaller",
        "Rite of the Dragoncaller's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Lattice Library",
        "Lattice Library's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    for (src, sub) in [
        ("Murmuring Mystic", "Bird"),
        ("Rite of the Dragoncaller", "Dragon"),
    ] {
        let (t, on_top) = bolt_countered(src);
        assert!(on_top, "{src}");
        assert_eq!(
            tokens(&t, P0)
                .into_iter()
                .filter(|id| t.obj(*id).chars.has_subtype(sub))
                .count(),
            1,
            "{src}"
        );
    }
    // Lattice Library with two study counters; P0 casts Blaze (X=1), P1 counters it.
    supported("Lattice Library");
    let mut t = TestGame::new(2);
    let lib = t.battlefield(P0, "Lattice Library");
    t.g.objects[lib.0 as usize]
        .counters
        .insert("study".into(), 2);
    t.g.dirty = true;
    t.lands(P0, "Mountain", 2);
    let blaze = t.hand(P0, "Blaze");
    let blaze = t.cast(P0, blaze).x(1).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), blaze);
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(blaze).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    let fractals: Vec<_> = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Fractal"))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(t.pt(fractals[0]), (2, 2));
}

#[test]
fn lattice_library_cast_as_the_first_x_spell_doesnt_trigger_for_the_second() {
    cr!("603.2", "603.4");
    ruling!(
        "Lattice Library",
        "If you cast your first spell with {X} in its mana cost during a turn before Lattice Library is on the battlefield (including Lattice Library itself), casting another spell with {X} in its mana cost later in the turn won't cause its last ability to trigger."
    );
    let fractals = |t: &TestGame| {
        tokens(t, P0)
            .into_iter()
            .filter(|id| t.obj(*id).chars.has_subtype("Fractal"))
            .count()
    };
    // Lattice Library (X=1) is the first X spell this turn: one Fractal from its enters
    // trigger, none from Blaze afterwards.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let lib = t.hand(P0, "Lattice Library");
    t.cast(P0, lib).x(1).go();
    t.resolve_all();
    assert_eq!(fractals(&t), 1);
    t.lands(P0, "Mountain", 2);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(1).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(fractals(&t), 1);
    // With the Library already there, the first X spell triggers it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lattice Library");
    t.lands(P0, "Mountain", 2);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(1).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

// --- Cards from outside the game -------------------------------------------------------

#[test]
fn legion_angel_outside_the_game_casual_and_constructed() {
    cr!("400.11b", "100.2a", "100.4a");
    ruling!(
        "Legion Angel",
        "In a casual game, you may find a Legion Angel outside the game even if your deck contains already four copies of Legion Angel."
    );
    supported("Legion Angel");
    // Four copies in the game already, a fifth outside it: the enters ability finds it.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Legion Angel");
    }
    let outside = t.custom(
        P0,
        (*mtg_engine::card::card("Legion Angel")).clone(),
        Zone::Outside(P0),
    );
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(outside)]);
    t.enter(P0, "Legion Angel");
    t.resolve_all();
    assert!(t.in_hand(P0, "Legion Angel"));
    assert!(t.player(P0).sideboard.is_empty());
    // In a constructed deck the four-copy limit counts the deck and sideboard together.
    let angel = mtg_engine::card::card("Legion Angel");
    let plains = mtg_engine::card::card("Plains");
    let mut deck = vec![angel.clone(); 4];
    deck.extend(std::iter::repeat_n(plains.clone(), 56));
    let names = mtg_engine::deck::NameEquivalence::default();
    assert!(mtg_engine::deck::check_constructed_with(&deck, &[plains], &names).is_empty());
    let problems = mtg_engine::deck::check_constructed_with(&deck, &[angel], &names);
    assert!(problems.iter().any(|p| matches!(
        p,
        mtg_engine::deck::DeckProblem::TooManyCopies {
            have: 5,
            max: 4,
            ..
        }
    )));
}
