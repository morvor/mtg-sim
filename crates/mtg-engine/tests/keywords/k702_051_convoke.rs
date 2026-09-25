//! CR 702.51 Convoke.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_038_051::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Queues the creatures to tap for convoke.
fn convoke_with(t: &mut TestGame, p: PlayerId, creatures: &[ObjectId]) {
    let v: Vec<Entity> = creatures.iter().map(|c| Entity::Object(*c)).collect();
    t.answer(p, DecisionKind::Entities, Answer::Entities(v));
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

fn convoke_asked(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("convoke"))
        })
        .count()
}

#[test]
fn creatures_pay_colored_mana_of_their_color_or_generic_mana() {
    cr!("702.51", "702.51a");
    assert_supported("Siege Wurm");
    let mut t = TestGame::new(2);
    // Siege Wurm {5}{G}{G}: two green creatures pay {G}{G}, three colorless ones pay
    // {3}, and two Forests pay the rest.
    let g1 = t.battlefield(P0, "Grizzly Bears");
    let g2 = t.battlefield(P0, "Grizzly Bears");
    let m: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Memnite")).collect();
    t.lands(P0, "Forest", 2);
    let wurm = t.hand(P0, "Siege Wurm");
    convoke_with(&mut t, P0, &[g1, g2, m[0], m[1], m[2]]);
    t.cast(P0, wurm).go();
    for c in [g1, g2, m[0], m[1], m[2]] {
        assert!(tapped(&t, c));
    }
    assert!(t.g.permanents().all(|o| !o.chars.is_land() || o.tapped));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Siege Wurm").len(), 1);
}

#[test]
fn a_convoke_spell_can_be_cast_with_creatures_alone() {
    cr!("702.51a");
    let mut t = TestGame::new(2);
    // No lands: four red creatures pay all of Stoke the Flames's {2}{R}{R}.
    for _ in 0..4 {
        t.battlefield(P0, "Hill Giant");
    }
    let stoke = t.hand(P0, "Stoke the Flames");
    let actions = t.g.legal_actions(P0);
    assert!(actions
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == stoke)));
    // By default, the creatures needed are tapped.
    t.cast(P0, stoke).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn a_creature_cant_pay_colored_mana_of_another_color() {
    cr!("702.51a");
    ruling!(
        "Venerated Loxodon",
        "You can't tap more creatures to convoke Venerated Loxodon than it takes to pay for its total cost."
    );
    assert_supported("Stoke the Flames");
    let mut t = TestGame::new(2);
    // Stoke the Flames {2}{R}{R}: green creatures can pay only the {2}.
    let greens: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Mountain", 1);
    let stoke = t.hand(P0, "Stoke the Flames");
    convoke_with(&mut t, P0, &greens);
    assert!(t.cast(P0, stoke).target(P1).try_go().is_err());
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    convoke_with(&mut t, P0, &greens);
    t.cast(P0, stoke).target(P1).go();
    // Only two of them paid something, so only two were tapped.
    assert_eq!(greens.iter().filter(|g| tapped(&t, **g)).count(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn a_multicolored_creature_pays_any_of_its_colors() {
    cr!("702.51a");
    ruling!(
        "Stoke the Flames",
        "Tapping a multicolored creature using convoke will pay for {1} or one mana of your choice of any of that creature's colors."
    );
    let mut t = TestGame::new(2);
    // Boros Reckoner is red and white.
    let r1 = t.battlefield(P0, "Boros Reckoner");
    let r2 = t.battlefield(P0, "Boros Reckoner");
    let r3 = t.battlefield(P0, "Boros Reckoner");
    t.lands(P0, "Plains", 1);
    let stoke = t.hand(P0, "Stoke the Flames");
    convoke_with(&mut t, P0, &[r1, r2, r3]);
    t.cast(P0, stoke).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn summoning_sick_and_attacking_creatures_can_convoke() {
    cr!("702.51a");
    ruling!(
        "Stoke the Flames",
        "You can tap any untapped creature you control to convoke a spell, even one you haven't controlled continuously since the beginning of your most recent turn."
    );
    ruling!(
        "Siege Wurm",
        "Tapping an untapped creature that's attacking or blocking to convoke a spell won't cause that creature to stop attacking or blocking."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let sick: Vec<ObjectId> = (0..2)
        .map(|_| t.battlefield_sick(P0, "Hill Giant"))
        .collect();
    t.lands(P0, "Mountain", 1);
    crate::common_k702_011_017::attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    // Serra Angel has vigilance: it's attacking and untapped.
    assert!(!tapped(&t, angel));
    let stoke = t.hand(P0, "Stoke the Flames");
    convoke_with(&mut t, P0, &[angel, sick[0], sick[1]]);
    t.cast(P0, stoke).target(P1).go();
    assert!(tapped(&t, angel) && tapped(&t, sick[0]) && tapped(&t, sick[1]));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.g.is_attacking(angel));
    t.advance_to(P0, turn::Step::EndOfCombat);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn a_creature_tapped_for_mana_cant_also_convoke() {
    cr!("702.51a");
    ruling!(
        "Siege Wurm",
        "If a creature you control has a mana ability with {T} in the cost, activating that ability while casting a spell with convoke will result in the creature being tapped before you pay the spell's costs. You won't be able to tap it again for convoke."
    );
    let mut t = TestGame::new(2);
    let mystic = t.battlefield(P0, "Elvish Mystic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Tap the Mystic for {G} first.
    t.activate(P0, mystic, 0, &[]).unwrap();
    assert!(tapped(&t, mystic));
    t.lands(P0, "Mountain", 2);
    let stoke = t.hand(P0, "Stoke the Flames");
    t.cast(P0, stoke).target(P1).go();
    let cands = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("convoke") => Some(candidates),
            _ => None,
        })
        .expect("convoke offered");
    assert_eq!(cands, vec![Entity::Object(bears)]);
    // By default the Bears pay for what the mana can't.
    assert!(tapped(&t, bears));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn convoke_applies_after_the_total_cost_is_determined() {
    cr!("702.51b");
    ruling!(
        "Siege Wurm",
        "Convoke applies after the total cost is calculated. Convoke doesn't change a spell's mana cost or mana value."
    );
    assert_supported("Heartless Summoning");
    let mut t = TestGame::new(2);
    // CR 702.51b's example: with Heartless Summoning, Siege Wurm's total cost is
    // {3}{G}{G}: two green creatures and three other creatures pay it all.
    t.battlefield(P0, "Heartless Summoning");
    let g1 = t.battlefield(P0, "Grizzly Bears");
    let g2 = t.battlefield(P0, "Grizzly Bears");
    let others: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Hill Giant")).collect();
    let wurm = t.hand(P0, "Siege Wurm");
    convoke_with(&mut t, P0, &[g1, g2, others[0], others[1], others[2]]);
    let spell = t.cast(P0, wurm).go();
    assert_eq!(t.g.mana_value_of(spell), 7);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Siege Wurm").len(), 1);
}

#[test]
fn x_is_chosen_before_creatures_are_tapped() {
    cr!("702.51b");
    ruling!(
        "Chord of Calling",
        "If you tap two green creatures and two red creatures, you'll have to pay {1}{G}."
    );
    assert_supported("Chord of Calling");
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Grizzly Bears");
    let g2 = t.battlefield(P0, "Grizzly Bears");
    let r1 = t.battlefield(P0, "Hill Giant");
    let r2 = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 2);
    t.library_top(P0, "Serra Angel");
    t.library_top(P0, "Elvish Mystic");
    t.g.search_finds_by_default = true;
    let chord = t.hand(P0, "Chord of Calling");
    convoke_with(&mut t, P0, &[g1, g2, r1, r2]);
    // X = 3: {3}{G}{G}{G}; the creatures pay {2}{G}{G}, the Forests {1}{G}.
    t.cast(P0, chord).x(3).go();
    assert!(t.g.permanents().all(|o| !o.chars.is_land() || o.tapped));
    t.resolve_all();
    // A creature card with mana value 3 or less was found (not Serra Angel).
    assert_eq!(t.named_on_battlefield("Elvish Mystic").len(), 1);
    assert!(t.named_on_battlefield("Serra Angel").is_empty());
}

#[test]
fn convoking_creatures_arent_mana_spent() {
    cr!("702.51b", "702.44b");
    // A sunburst creature convoked by green creatures and paid for with {R}: only red
    // mana was spent to cast it.
    let def = with_cost(
        custom_card(
            "Convoked Myr",
            "Artifact Creature — Myr",
            Some((0, 0)),
            "Convoke\nSunburst",
        ),
        "{3}",
    );
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Grizzly Bears");
    let g2 = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let myr = t.custom(P0, def, Zone::Hand(P0));
    convoke_with(&mut t, P0, &[g1, g2]);
    t.cast(P0, myr).go();
    t.resolve_all();
    let myr = t.named_on_battlefield("Convoked Myr")[0];
    assert_eq!(t.counters(myr, counters::PLUS1), 1);
}

#[test]
fn creatures_tapped_for_convoke_convoked_the_spell() {
    cr!("702.51c");
    assert_supported("Venerated Loxodon");
    let mut t = TestGame::new(2);
    // Venerated Loxodon {4}{W}: "put a +1/+1 counter on each creature that convoked it."
    let c: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let lox = t.hand(P0, "Venerated Loxodon");
    convoke_with(&mut t, P0, &c);
    t.cast(P0, lox).go();
    t.resolve_all();
    for x in &c {
        assert_eq!(t.counters(*x, counters::PLUS1), 1);
    }
    assert_eq!(t.counters(other, counters::PLUS1), 0);
    let lox = t.named_on_battlefield("Venerated Loxodon")[0];
    assert_eq!(t.counters(lox, counters::PLUS1), 0);
}

#[test]
fn the_number_of_creatures_that_convoked_it() {
    cr!("702.51c");
    ruling!(
        "Ancient Imperiosaur",
        "You can't tap more creatures to convoke Ancient Imperiosaur than is necessary to pay for the spell."
    );
    assert_supported("Ancient Imperiosaur");
    assert_supported("Zephyr Singer");
    let mut t = TestGame::new(2);
    let c: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Forest", 3);
    let dino = t.hand(P0, "Ancient Imperiosaur");
    convoke_with(&mut t, P0, &c);
    t.cast(P0, dino).go();
    t.resolve_all();
    let dino = t.named_on_battlefield("Ancient Imperiosaur")[0];
    assert_eq!(t.counters(dino, counters::PLUS1), 8);
    // Zephyr Singer: a flying counter on each creature that convoked it.
    let blue = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let red = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 2);
    let singer = t.hand(P0, "Zephyr Singer");
    convoke_with(&mut t, P0, &[blue, red]);
    t.cast(P0, singer).go();
    t.resolve_all();
    assert_eq!(t.counters(blue, "flying"), 1);
    assert!(t.obj_now(red).has_keyword(keywords::KeywordKind::Flying));
}

#[test]
fn spells_can_be_given_convoke() {
    cr!("702.51a");
    assert_supported("Chief Engineer");
    let mut t = TestGame::new(2);
    // Chief Engineer: "Artifact spells you cast have convoke." Juggernaut {4} with four
    // creatures and no lands.
    let engineer = t.battlefield(P0, "Chief Engineer");
    let others: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let jugg = t.hand(P0, "Juggernaut");
    let bears = t.hand(P0, "Grizzly Bears");
    let actions = t.g.legal_actions(P0);
    let castable = |c: ObjectId| {
        actions
            .iter()
            .any(|a| matches!(a, Action::Cast { card, .. } if *card == c))
    };
    assert!(castable(jugg));
    // A nonartifact spell doesn't have convoke.
    assert!(!castable(bears));
    convoke_with(&mut t, P0, &[engineer, others[0], others[1], others[2]]);
    t.cast(P0, jugg).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Juggernaut").len(), 1);
}

#[test]
fn multiple_instances_of_convoke_are_redundant() {
    cr!("702.51d");
    let def = with_cost(
        custom_card(
            "Doubly Convoked Wurm",
            "Creature — Wurm",
            Some((4, 4)),
            "Convoke\nConvoke",
        ),
        "{2}{G}",
    );
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let w = t.custom(P0, def, Zone::Hand(P0));
    // One Bears can pay only one mana; the rest can't be paid.
    convoke_with(&mut t, P0, &[g1]);
    convoke_with(&mut t, P0, &[g1]);
    assert!(t.cast(P0, w).try_go().is_err());
    assert_eq!(convoke_asked(&t), 1);
}

#[test]
fn a_spell_has_convoke_whether_or_not_creatures_are_tapped() {
    cr!("702.51a");
    ruling!(
        "Kasla, the Broken Halo",
        "Kasla's last ability will trigger whether you tapped any creatures to pay for the spell or not, as long as it has convoke."
    );
    assert_supported("Kasla, the Broken Halo");
    let mut t = TestGame::new(2);
    // Kasla: "Whenever you cast another spell that has convoke, scry 2, then draw a
    // card."
    t.battlefield(P0, "Kasla, the Broken Halo");
    t.lands(P0, "Mountain", 4);
    let stoke = t.hand(P0, "Stoke the Flames");
    convoke_with(&mut t, P0, &[]);
    t.cast(P0, stoke).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), 1);
    // A spell without convoke doesn't trigger it.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn the_next_spell_can_be_given_convoke() {
    cr!("702.51a", "702.51d");
    ruling!(
        "Wand of the Worldsoul",
        "If the next spell you cast after Wand of the Worldsoul's ability resolves already has convoke, giving it convoke again doesn't have any real benefit."
    );
    assert_supported("Wand of the Worldsoul");
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Wand of the Worldsoul");
    let c: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let giant = t.hand(P0, "Hill Giant");
    // "{T}: The next spell you cast this turn has convoke."
    t.activate(P0, wand, 1, &[]).unwrap();
    t.resolve_all();
    // Hill Giant {3}{R} is castable: two creatures and... not enough.
    let castable = |t: &mut TestGame, x: ObjectId| {
        t.g.legal_actions(P0)
            .iter()
            .any(|a| matches!(a, Action::Cast { card, .. } if *card == x))
    };
    assert!(!castable(&mut t, giant));
    t.lands(P0, "Mountain", 2);
    assert!(castable(&mut t, giant));
    convoke_with(&mut t, P0, &c);
    t.cast(P0, giant).go();
    assert!(c.iter().all(|x| tapped(&t, *x)));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Only the next spell had it; Stoke the Flames has its own convoke (once).
    t.clear_answers();
    let stoke = t.hand(P0, "Stoke the Flames");
    let more: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Hill Giant")).collect();
    convoke_with(&mut t, P0, &more);
    t.cast(P0, stoke).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn convoke_from_two_sources_is_redundant() {
    cr!("702.51d");
    ruling!(
        "Chief Engineer",
        "Multiple instances of convoke on a single spell are redundant."
    );
    let mut t = TestGame::new(2);
    let e1 = t.battlefield(P0, "Chief Engineer");
    let e2 = t.battlefield(P0, "Chief Engineer");
    t.lands(P0, "Wastes", 2);
    let jugg = t.hand(P0, "Juggernaut");
    convoke_with(&mut t, P0, &[e1, e2]);
    t.cast(P0, jugg).go();
    assert_eq!(convoke_asked(&t), 1);
    assert!(tapped(&t, e1) && tapped(&t, e2));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Juggernaut").len(), 1);
}
