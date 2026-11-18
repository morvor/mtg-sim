//! Rulings batch P181 — the other abilities of land-searching cards: split cards (Road //
//! Ruin), Awakened Skyclave cast from a defeated Siege, Monument to Perfection's
//! ability-losing animation, Bioengineered Future, beholding with Elven Passage,
//! Borderland Explorer's simultaneous discards, Roar of the Fifth People, Beanstalk
//! Giant's characteristic-defining ability, Alpine Houndmaster, Nylea's Intervention and
//! Lessons.

use crate::r_p181_common::*;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const ROAD: &str = "Road // Ruin";
const AFTERMATH: CastMethod = CastMethod::Keyword(KeywordKind::Aftermath);

#[test]
fn road_ruin_halves_on_the_stack_and_the_whole_card_elsewhere() {
    cr!("709.3a", "709.4", "702.127a");
    ruling!(
        "Road // Ruin",
        "All split cards have two card faces on a single card, and you put a split card onto the stack with only the half you're casting. The characteristics of the half you didn't cast are ignored while the spell is on the stack. For example, if an effect prevents you from casting red spells, you can cast Road, but not Ruin."
    );
    ruling!(
        "Road // Ruin",
        "While not on the stack, the characteristics of a split card are the combination of its two halves. For example, Road /// Ruin is a green and red card, it is both an instant card and a sorcery card, and its mana value is 6."
    );
    supported(ROAD);
    let mut t = TestGame::new(2);
    // Iona, red chosen: P0 can't cast red spells.
    let red = Color::ALL.iter().position(|c| *c == Color::Red).unwrap();
    t.answer(P1, DecisionKind::Option, Answer::Index(red));
    t.enter(P1, "Iona, Shield of Emeria");
    lots(&mut t, P0);
    // A creature for Ruin to target.
    t.battlefield(P1, "Grizzly Bears");
    let in_hand = t.hand(P0, ROAD);
    let in_gy = t.graveyard(P0, ROAD);
    // The whole card: green and red, an instant and a sorcery, mana value 6.
    for id in [in_hand, in_gy] {
        let o = t.obj_now(id);
        assert!(o.chars.colors.contains(Color::Green) && o.chars.colors.contains(Color::Red));
        assert!(o.is(CardType::Instant) && o.is(CardType::Sorcery));
        assert_eq!(t.g.mana_value_of(id), 6);
    }
    assert!(crate::r_s02_common::can_cast(
        &mut t,
        P0,
        in_hand,
        CastMethod::Half(0)
    ));
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, in_gy, AFTERMATH));
    // Road (green) on the stack is only green.
    t.answer_choose(P0, &[]);
    let spell = t.cast(P0, in_hand).method(CastMethod::Half(0)).go();
    let o = t.obj_now(spell);
    assert!(!o.chars.colors.contains(Color::Red));
    assert!(!o.is(CardType::Sorcery));
    assert_eq!(t.g.mana_value_of(spell), 3);
    t.resolve_all();
    // Without Iona, Ruin can be cast from the graveyard.
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    t.battlefield(P1, "Grizzly Bears");
    let in_gy = t.graveyard(P0, ROAD);
    assert!(crate::r_s02_common::can_cast(&mut t, P0, in_gy, AFTERMATH));
}

#[test]
fn road_ruin_is_one_card() {
    cr!("709.1");
    ruling!(
        "Road // Ruin",
        "Each split card is a single card. For example, if you discard one, you've discarded one card, not two. If an effect counts the number of instant and sorcery cards in your graveyard, Road /// Ruin counts once, not twice."
    );
    let mut t = TestGame::new(2);
    // Crackling Drake: power = instant and sorcery cards you own in exile and graveyard.
    let drake = t.battlefield(P0, "Crackling Drake");
    let road = t.hand(P0, ROAD);
    let hand = t.hand_size(P0);
    t.g.discard(P0, road, None);
    t.settle();
    assert_eq!(t.hand_size(P0), hand - 1);
    let discards =
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Discarded { .. }))
            .count();
    assert_eq!(discards, 1);
    assert_eq!(t.pt(drake).0, 1);
}

/// P0's Invasion of Zendikar (P1 protecting it) is defeated; P0 casts Awakened Skyclave.
/// Returns the spell, on the stack.
fn skyclave_cast(t: &mut TestGame) -> ObjectId {
    supported("Invasion of Zendikar // Awakened Skyclave");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[]);
    let siege = t.enter(P0, "Invasion of Zendikar // Awakened Skyclave");
    t.resolve_all();
    let source = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    crate::r_s06_common::damage(t, source, 3, siege);
    t.settle();
    // The defeat trigger resolves: Awakened Skyclave is cast.
    t.resolve();
    let spell = *t.g.stack.last().expect("Awakened Skyclave cast");
    assert_eq!(t.obj_now(spell).chars.name, "Awakened Skyclave");
    spell
}

#[test]
fn awakened_skyclave_is_a_creature_spell_and_a_land_once_on_the_battlefield() {
    cr!("310.12b", "712.11a");
    ruling!(
        "Invasion of Zendikar // Awakened Skyclave",
        "Awakened Skyclave is cast as a creature spell. It can be countered and otherwise responded to. It won't be a land in addition to its other types until it enters the battlefield."
    );
    ruling!(
        "Invasion of Zendikar // Awakened Skyclave",
        "If Awakened Skyclave enters the battlefield, any ability that triggers whenever a land enters the battlefield will trigger."
    );
    // Countered.
    let mut t = TestGame::new(2);
    let spell = skyclave_cast(&mut t);
    let o = t.obj_now(spell);
    assert!(o.is(CardType::Creature));
    assert!(!o.is(CardType::Land));
    lots(&mut t, P1);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Invasion of Zendikar"));
    // It resolves: a land creature, and landfall triggers.
    let mut t = TestGame::new(2);
    let geopede = t.battlefield(P0, "Plated Geopede");
    skyclave_cast(&mut t);
    t.resolve_all();
    let sky = t.named_on_battlefield("Awakened Skyclave")[0];
    assert!(t.obj_now(sky).is(CardType::Land) && t.obj_now(sky).is_creature());
    assert_eq!(triggered_from(&t, geopede), 1);
}

#[test]
fn monument_to_perfection_has_one_toxic_and_stays_a_creature() {
    cr!("613.1f", "613.7", "611.2a");
    ruling!(
        "Monument to Perfection",
        "Because the last ability causes Monument to Perfection to lose all the abilities, activating it multiple times before the first activation resolves does not result in it having multiple instances of toxic 9."
    );
    ruling!(
        "Monument to Perfection",
        "Monument to Perfection's last ability lasts indefinitely."
    );
    supported("Monument to Perfection");
    let mut t = TestGame::new(2);
    for land in [
        "Plains",
        "Island",
        "Swamp",
        "Mountain",
        "Forest",
        "Wastes",
        "Snow-Covered Plains",
        "Snow-Covered Island",
        "Snow-Covered Swamp",
    ] {
        t.battlefield(P0, land);
    }
    let m = t.battlefield(P0, "Monument to Perfection");
    lots(&mut t, P0);
    activate_containing(&mut t, P0, m, "9/9").unwrap();
    activate_containing(&mut t, P0, m, "9/9").unwrap();
    t.resolve_all();
    let o = t.obj_now(m);
    assert_eq!(t.pt(m), (9, 9));
    assert_eq!(o.chars.keyword_count(KeywordKind::Toxic), 1);
    assert!(o.chars.has_keyword(KeywordKind::Indestructible));
    // Turns later, still a 9/9 creature.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.obj_now(m).is_creature());
    assert_eq!(t.pt(m), (9, 9));
}

/// Puts the real cards onto the battlefield under `p`'s control at the same time.
fn enter_together(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    use mtg_engine::replacement::{EtbInfo, MoveEv};
    let evs = names
        .iter()
        .map(|n| MoveEv {
            obj: t.g.create_card_object(card(n), p, Zone::Nowhere),
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let ids = t.g.move_objects(evs).into_iter().flatten().collect();
    t.g.flush_events();
    ids
}

#[test]
fn bioengineered_future_counts_earlier_lands_but_not_creatures_entering_with_it() {
    cr!("614.1c", "614.12");
    ruling!(
        "Bioengineered Future",
        "Bioengineered Future’s last ability will consider lands that entered the battlefield under your control before it was on the battlefield."
    );
    ruling!(
        "Bioengineered Future",
        "In the unlikely event that Bioengineered Future and one or more creatures you control enter at the same time, Bioengineered Future’s last ability won’t cause those creatures to enter with additional +1/+1 counters on them."
    );
    supported("Bioengineered Future");
    let mut t = TestGame::new(2);
    t.enter(P0, "Forest");
    t.enter(P0, "Forest");
    t.enter(P0, "Bioengineered Future");
    t.resolve_all();
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // Entering at the same time.
    let mut t = TestGame::new(2);
    t.enter(P0, "Forest");
    let ids = enter_together(&mut t, P0, &["Bioengineered Future", "Grizzly Bears"]);
    let bears = ids
        .into_iter()
        .find(|i| t.g.obj(*i).chars.name == "Grizzly Bears")
        .unwrap();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    let later = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(later, counters::PLUS1), 1);
}

/// P0 sacrifices Elven Passage (with P0's hand and battlefield set up by `setup`) and
/// answers the behold choice with `behold`. Returns the found Forest and the candidates
/// offered to behold.
fn elven_passage(
    setup: impl FnOnce(&mut TestGame) -> Vec<Entity>,
) -> (TestGame, ObjectId, Vec<Entity>) {
    supported("Elven Passage");
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Forest", "Forest"]);
    let passage = t.battlefield(P0, "Elven Passage");
    let behold = setup(&mut t);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &behold);
    let from = t.asked().len();
    activate_containing(&mut t, P0, passage, "Search").unwrap();
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt == "Behold" => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    let f = t.g.current(lib[0]);
    (t, f, offered)
}

#[test]
fn elven_passage_beholds_an_elf_card_or_permanent() {
    cr!("701.4a", "205.3m");
    ruling!(
        "Elven Passage",
        "Effects that say \"If a [quality] was beheld\" only care if a card of that quality was revealed or a permanent you control of that quality was chosen."
    );
    ruling!(
        "Elven Passage",
        "If an effect refers to a \"[subtype] card,\" it refers only to a card that has that subtype. For example, Elven Chorus is not an Elf, despite referencing elves in its name and featuring elves in its art."
    );
    // An Elf card revealed from the hand.
    let (t, f, _) = elven_passage(|t| vec![obj(t.hand(P0, "Llanowar Elves"))]);
    assert!(!t.obj_now(f).tapped);
    assert!(!revealed(&t).is_empty());
    // An Elf permanent chosen.
    let (t, f, _) = elven_passage(|t| vec![obj(t.battlefield(P0, "Llanowar Elves"))]);
    assert!(!t.obj_now(f).tapped);
    // Elven Chorus isn't an Elf: nothing to behold, the land stays tapped.
    let (t, f, offered) = elven_passage(|t| vec![obj(t.hand(P0, "Elven Chorus"))]);
    assert!(offered.is_empty());
    assert!(t.obj_now(f).tapped);
}

#[test]
fn borderland_explorer_discards_in_turn_order_then_all_at_once() {
    cr!("101.4", "701.9a");
    ruling!(
        "Borderland Explorer",
        "Players decide whether or not they’re discarding a card in turn order. Each player will know if previous players are discarding, but not what they are discarding. All cards to be discarded are set aside, then revealed and discarded simultaneously. Similarly, all basic land cards searched for are set aside, then revealed simultaneously."
    );
    supported("Borderland Explorer");
    let mut t = TestGame::new(2);
    let l0 = set_library(&mut t, P0, &["Forest", "Forest"]);
    let l1 = set_library(&mut t, P1, &["Plains", "Plains"]);
    let d0 = t.hand(P0, "Grizzly Bears");
    let d1 = t.hand(P1, "Hill Giant");
    for (p, d, l) in [(P0, d0, l0[0]), (P1, d1, l1[0])] {
        t.answer_yes(p, true);
        t.answer_choose(p, &[obj(d)]);
        t.answer_yes(p, true);
        t.answer_choose(p, &[obj(l)]);
    }
    // At each of P1's decisions, P0's discard hasn't happened yet.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. } | Decision::YesNo { .. }),
        |g| g.player(PlayerId(0)).graveyard.len(),
    );
    let from = t.asked().len();
    t.enter(P0, "Borderland Explorer");
    t.resolve_all();
    // P0 decided first.
    let first_p1 = t.asked()[from..]
        .iter()
        .position(|(p, _)| *p == P1)
        .unwrap();
    let first_p0 = t.asked()[from..]
        .iter()
        .position(|(p, _)| *p == P0)
        .unwrap();
    assert!(first_p0 < first_p1);
    let gy = seen.lock().unwrap().clone();
    assert_eq!(gy[0], 0, "P0's card isn't discarded when P1 decides");
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P1, "Hill Giant"));
    assert!(is_in(&t, l0[0], Zone::Hand(P0)) && is_in(&t, l1[0], Zone::Hand(P1)));
    // The found cards were revealed.
    let shown = revealed(&t);
    assert!(shown.contains(&l0[0]) || shown.contains(&t.g.current(l0[0])));
    assert!(shown.contains(&l1[0]) || shown.contains(&t.g.current(l1[0])));
}

/// Whether the permanent has a mana ability adding {R}.
fn taps_for_red(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Add {R}"))
}

#[test]
fn roar_of_the_fifth_people_chapter_two_has_no_duration() {
    cr!("714.2b", "611.2a");
    ruling!(
        "Huatli, Poet of Unity // Roar of the Fifth People",
        "Roar of the Fifth People's second chapter ability doesn't have a duration. It will continue to apply as long as Roar of the Fifth People remains on the battlefield."
    );
    supported("Huatli, Poet of Unity // Roar of the Fifth People");
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Poet of Unity // Roar of the Fifth People");
    lots(&mut t, P0);
    activate_containing(&mut t, P0, huatli, "transformed").unwrap();
    t.resolve_all();
    let roar = t.named_on_battlefield("Roar of the Fifth People")[0];
    let dinos = crate::r_s01_common::with_subtype(&t, P0, "Dinosaur");
    assert_eq!(dinos.len(), 2);
    // Chapter II.
    t.g.add_counters(Entity::Object(roar), counters::LORE, 1, None);
    t.resolve_all();
    assert!(taps_for_red(&t, dinos[0]));
    // Next turn (chapter III), and a creature that arrives later.
    t.answer_choose(P0, &[]);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(roar, counters::LORE), 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(taps_for_red(&t, dinos[0]));
    assert!(taps_for_red(&t, bears));
}

#[test]
fn beanstalk_giants_power_counts_lands_in_every_zone() {
    cr!("604.3", "208.3");
    ruling!(
        "Beanstalk Giant // Fertile Footsteps",
        "The ability that defines Beanstalk Giant's power and toughness works in all zones, not just the battlefield."
    );
    supported("Beanstalk Giant // Fertile Footsteps");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let hand = t.hand(P0, "Beanstalk Giant // Fertile Footsteps");
    let lib = t.library_top(P0, "Beanstalk Giant // Fertile Footsteps");
    let gy = t.graveyard(P0, "Beanstalk Giant // Fertile Footsteps");
    for id in [hand, lib, gy] {
        let c = &t.obj_now(id).chars;
        assert_eq!((c.power, c.toughness), (Some(3), Some(3)));
    }
}

#[test]
fn alpine_houndmasters_bonus_is_fixed_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Alpine Houndmaster",
        "The value of X is determined only as Alpine Houndmaster's last ability resolves. Once that happens, the value of X won't change later in the turn even if the number of attacking creatures changes."
    );
    supported("Alpine Houndmaster");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Alpine Houndmaster");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[
            (h, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert_eq!(t.pt(h).0, 4);
    destroy(&mut t, a);
    assert_eq!(t.pt(h).0, 4);
}

#[test]
fn nyleas_intervention_deals_twice_x_damage_once() {
    cr!("120.2", "700.1");
    ruling!(
        "Nylea's Intervention",
        "The second mode of Nylea’s Intervention has it deal damage to each creature with flying equal to twice X. It doesn’t deal X damage to them then deal X damage again."
    );
    supported("Nylea's Intervention");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Shivan Dragon");
    lots(&mut t, P0);
    let ni = t.hand(P0, "Nylea's Intervention");
    t.cast(P0, ni).modes(&[1]).x(2).go();
    t.resolve_all();
    let hits: Vec<u32> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { target, amount, .. } if *target == obj(dragon) => Some(*amount),
                _ => None,
            })
            .collect();
    assert_eq!(hits, vec![4]);
}

#[test]
fn lessons_are_ordinary_sorceries_in_the_main_deck() {
    cr!("100.2a", "205.3k");
    ruling!(
        "Environmental Sciences",
        "Although you may want to include Lessons in your sideboard if you're playing with cards that instruct you to learn, Lesson cards can be included in your main deck like other instant or sorcery cards."
    );
    ruling!(
        "Environmental Sciences",
        "Lesson is a spell subtype found on some instant and sorcery cards in the Strixhaven set. The Lesson subtype has no special rules associated with it."
    );
    supported("Environmental Sciences");
    let mut deck = vec![card("Environmental Sciences"); 4];
    deck.extend(vec![card("Forest"); 56]);
    assert!(mtg_engine::deck::check_constructed(&deck).is_empty());
    // Cast like any sorcery: only at sorcery speed.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Forest"]);
    lots(&mut t, P0);
    let es = t.hand(P0, "Environmental Sciences");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!crate::r_s02_common::can_cast(
        &mut t,
        P0,
        es,
        CastMethod::Normal
    ));
    t.set_step(P0, Step::PrecombatMain);
    lots(&mut t, P0);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.cast(P0, es).go();
    t.resolve_all();
    assert!(is_in(&t, lib[0], Zone::Hand(P0)));
    assert_eq!(t.life(P0), 22);
}
