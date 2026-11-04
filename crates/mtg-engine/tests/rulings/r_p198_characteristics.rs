//! Rulings batch P198 — characteristic-changing effects that last indefinitely: copies
//! (CR 707.2), type changes (205.1), text changes (612), exchanges of life totals and
//! power or toughness (701.12g), values locked in on resolution (608.2h), and Sagas whose
//! chapters grant abilities.

use crate::r_p198_common::*;
use crate::r_s01_common::stack_library;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::{CardType, Color, Supertype};
use mtg_engine::*;

#[test]
fn mirrorform_copies_only_the_copiable_values() {
    cr!("707.2", "707.3");
    ruling!(
        "Mirrorform",
        "The permanents copy exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else; see below). They don't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let ring = t.battlefield(P0, "Sol Ring");
    let land = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(obj(bears), "+1/+1", 2, None);
    t.g.tap(bears);
    cast_at(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    cast_at(&mut t, P0, "Holy Strength", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 9));
    cast_at(&mut t, P0, "Mirrorform", &[obj(bears)]);
    t.resolve_all();
    for id in [giant, ring] {
        let o = t.obj_now(id);
        assert_eq!(o.chars.name, "Grizzly Bears");
        assert!(!o.tapped);
        assert_eq!(o.counter("+1/+1"), 0);
        assert_eq!(t.pt(id), (2, 2));
    }
    assert_eq!(t.obj_now(land).chars.name, "Forest");
}

#[test]
fn opal_acrolith_stops_being_a_creature_when_it_becomes_an_enchantment() {
    cr!("205.1a", "613.1d");
    ruling!(
        "Opal Acrolith",
        "When it turns into an enchantment, it is no longer a creature."
    );
    supported("Opal Acrolith");
    let mut t = TestGame::new(2);
    let opal = t.battlefield(P0, "Opal Acrolith");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    cast_at(&mut t, P1, "Grizzly Bears", &[]);
    t.resolve_all();
    assert!(t.obj_now(opal).is_creature());
    assert_eq!(t.pt(opal), (2, 4));
    act(&mut t, P0, opal, 0, &[]);
    let o = t.obj_now(opal);
    assert!(!o.is_creature());
    assert!(o.is(CardType::Enchantment));
    assert!(!o.chars.has_subtype("Soldier"));
}

#[test]
fn evras_exchange_is_a_life_gain_or_loss() {
    cr!("701.12g", "119.7", "119.8", "701.12a");
    ruling!(
        "Evra, Halcyon Witness",
        "When its activated ability resolves, Evra's power will become your former life total and you will gain or lose an amount of life such that your life total equals Evra's former power. Other effects that interact with life gain or life loss will interact with this effect accordingly."
    );
    supported("Ajani's Pridemate");
    supported("Platinum Emperion");
    // At 2 life: you gain 2 (Ajani's Pridemate triggers) and Evra becomes 2/4.
    let mut t = TestGame::new(2);
    let evra = t.battlefield(P0, "Evra, Halcyon Witness");
    let pm = t.battlefield(P0, "Ajani's Pridemate");
    t.g.players[0].life = 2;
    act(&mut t, P0, evra, 0, &[]);
    assert_eq!(t.life(P0), 4);
    assert_eq!(t.pt(evra), (2, 4));
    assert_eq!(t.counters(pm, "+1/+1"), 1);
    // Your life total can't change: no part of the exchange happens.
    let mut t = TestGame::new(2);
    let evra = t.battlefield(P0, "Evra, Halcyon Witness");
    t.battlefield(P0, "Platinum Emperion");
    act(&mut t, P0, evra, 0, &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.pt(evra), (4, 4));
}

#[test]
fn tree_of_perditions_exchange_is_a_life_gain_or_loss() {
    cr!("701.12g", "614.1a");
    ruling!(
        "Tree of Perdition",
        "When the ability resolves, Tree of Perdition's toughness becomes the targeted opponent's former life total and that player gains or loses an amount of life necessary so that their life total equals Tree of Perdition's former toughness. Other effects that interact with life gain or life loss will interact with this effect accordingly."
    );
    supported("Alhammarret's Archive");
    // The opponent at 5 gains 8, doubled by their Alhammarret's Archive.
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Tree of Perdition");
    t.battlefield(P1, "Alhammarret's Archive");
    t.g.players[1].life = 5;
    act(&mut t, P0, tree, 0, &[pl(P1)]);
    assert_eq!(t.life(P1), 21);
    assert_eq!(t.pt(tree), (0, 5));
    // From 20: they lose 7; it's 0/20.
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Tree of Perdition");
    act(&mut t, P0, tree, 0, &[pl(P1)]);
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.pt(tree), (0, 20));
}

#[test]
fn trench_gorger_is_a_6_6_while_its_ability_is_on_the_stack() {
    cr!("603.3", "611.2a");
    ruling!(
        "Trench Gorger",
        "While Trench Gorger’s ability is on the stack, it’s on the battlefield as a 6/6."
    );
    supported("Trench Gorger");
    let mut t = TestGame::new(2);
    let lands = stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    let g = t.enter(P0, "Trench Gorger");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.pt(g), (6, 6));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &lands.iter().map(|l| obj(*l)).collect::<Vec<_>>());
    t.resolve_all();
    assert_eq!(t.pt(g), (3, 3));
    pass_turn(&mut t);
    assert_eq!(t.pt(g), (3, 3));
}

#[test]
fn princess_yue_as_moon_doesnt_share_a_name_with_princess_yue() {
    cr!("704.5j", "201.2");
    ruling!(
        "Princess Yue",
        "You cannot have multiple legendary permanents with the same name on the battlefield at the same time. However, Princess Yue's name becomes Moon when she returns to the battlefield as a land."
    );
    supported("Princess Yue");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Princess Yue");
    kill(&mut t, a);
    t.resolve_all();
    let moon: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.chars.name == "Moon")
            .map(|o| o.id)
            .collect();
    assert_eq!(moon.len(), 1);
    let m = t.obj_now(moon[0]);
    assert!(m.is(CardType::Land) && !m.is_creature());
    assert!(m.chars.has_supertype(Supertype::Legendary));
    assert!(m.tapped);
    // Another Princess Yue joins it: both stay.
    let b = t.battlefield(P0, "Princess Yue");
    t.settle();
    assert!(t.on_battlefield(b));
    assert!(t.on_battlefield(moon[0]));
    pass_turn(&mut t);
    assert_eq!(t.obj_now(moon[0]).chars.name, "Moon");
    assert!(has_ability(&t, moon[0], "Add {C}"));
}

#[test]
fn urzas_saga_keeps_the_abilities_its_chapters_grant() {
    cr!("611.2a", "714.2b");
    ruling!(
        "Urza's Saga",
        "Urza's Saga gains an ability from its first and second chapters. It keeps those abilities for as long as it's on the battlefield."
    );
    supported("Urza's Saga");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Urza's Saga");
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    pass_turn(&mut t);
    assert!(has_ability(&t, s, "Add {C}"));
    assert!(has_ability(&t, s, "Construct"));
    // The Construct ability works: {2}, {T} (paid with its own mana and a land).
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, s, 1, &[]);
    assert_eq!(with_subtype(&t, P0, "Construct").len(), 1);
}

#[test]
fn urzas_saga_is_played_as_a_land_not_cast() {
    cr!("305.1", "305.9");
    ruling!(
        "Urza's Saga",
        "Urza's Saga is a land, so it can only be played as a land. It cannot be cast as a spell."
    );
    let mut t = TestGame::new(2);
    let s = t.hand(P0, "Urza's Saga");
    lots(&mut t, P0);
    assert!(t.cast(P0, s).try_go().is_err());
    assert!(t.play_land(P0, s).is_ok());
    assert_eq!(t.named_on_battlefield("Urza's Saga").len(), 1);
}

#[test]
fn urzas_saga_finds_cards_by_mana_cost_not_mana_value() {
    cr!("202.1", "701.23a");
    ruling!(
        "Urza's Saga",
        "While resolving the chapter III ability, you can find only a card with actual mana cost {0} or {1}, not mana value 0 or 1. For example, you couldn't find a card with mana cost {U} or one with mana cost {X}."
    );
    let mut t = TestGame::new(2);
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Giant's Amulet",
            "Sigil of Distinction",
            "Ornithopter",
            "Sol Ring",
        ],
    );
    let s = t.battlefield(P0, "Urza's Saga");
    t.g.add_counters(obj(s), "lore", 2, None);
    let from = t.asked().len();
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&obj(lib[2])));
    assert!(offered.contains(&obj(lib[3])));
    assert!(!offered.contains(&obj(lib[0])));
    assert!(!offered.contains(&obj(lib[1])));
}

#[test]
fn the_legend_of_kyoshi_reads_the_greatest_power_on_resolution() {
    cr!("608.2h");
    ruling!(
        "The Legend of Kyoshi // Avatar Kyoshi",
        "The greatest power among creatures you control is determined as The Legend of Kyoshi's first chapter ability resolves. If the greatest power among creatures you control is 0 or less at that time, you will not draw any cards."
    );
    // A Giant Growth in response raises it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let s = t.battlefield(P0, "The Legend of Kyoshi // Avatar Kyoshi");
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_at(&mut t, P0, "Giant Growth", &[obj(bears)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 5);
    // Power 0 or less: no cards.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    let s = t.battlefield(P0, "The Legend of Kyoshi // Avatar Kyoshi");
    let hand = t.hand_size(P0);
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn ascent_of_the_worthy_chooses_its_creature_on_resolution() {
    cr!("608.2c", "714.2b");
    ruling!(
        "Ascent of the Worthy",
        "You choose a creature for the chapter I/II ability as that ability resolves."
    );
    supported("Ascent of the Worthy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let s = t.battlefield(P0, "Ascent of the Worthy");
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A creature that arrives with the ability on the stack can be chosen.
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    damage(&mut t, giant, obj(bears), 2);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn liliana_waker_of_the_deads_x_is_locked_in_on_resolution() {
    cr!("608.2h", "611.2b");
    ruling!(
        "Liliana, Waker of the Dead",
        "The value of X is determined only as Liliana's second ability resolves. Once that happens, the value of X won't change later in the turn even if the number of cards in your graveyard changes."
    );
    supported("Liliana, Waker of the Dead");
    let mut t = TestGame::new(2);
    let lil = t.battlefield(P0, "Liliana, Waker of the Dead");
    let giant = t.battlefield(P1, "Hill Giant");
    t.graveyard(P0, "Island");
    act_only(&mut t, P0, lil, 1, &[obj(giant)]);
    // One more card before it resolves counts.
    t.graveyard(P0, "Island");
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    // Later cards don't.
    t.graveyard(P0, "Island");
    t.graveyard(P0, "Island");
    t.g.recompute();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn dream_coat_can_make_it_any_combination_of_colors() {
    cr!("105.1", "105.2");
    ruling!(
        "Dream Coat",
        "You can choose any single color or combination of colors. The valid colors are Black, Blue, Green, Red, and White."
    );
    supported("Dream Coat");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let coat = cast_at(&mut t, P0, "Dream Coat", &[obj(bears)]);
    t.resolve_all();
    let coat = t.g.current(coat);
    let seen = choose_words(&mut t, P0, &["blue and red"]);
    act(&mut t, P0, coat, 0, &[]);
    // Every single color and combination (31), never colorless.
    let lists = seen.lock().unwrap().clone();
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0].len(), 31);
    assert!(lists[0].iter().any(|w| w == "green"));
    pass_turn(&mut t);
    let colors = t.obj_now(bears).chars.colors;
    assert!(colors.contains(Color::Blue) && colors.contains(Color::Red));
    assert!(!colors.contains(Color::Green));
}

#[test]
fn glamerdye_chooses_two_different_color_words_on_resolution() {
    cr!("612.2", "608.2c");
    ruling!(
        "Glamerdye",
        "You choose the two color words when Glamerdye resolves. You can't change a word to the same word; you must choose a different word."
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let seen = choose_words(&mut t, P0, &["black", "red"]);
    cast_at(&mut t, P0, "Glamerdye", &[obj(knight)]);
    assert!(seen.lock().unwrap().is_empty());
    t.resolve_all();
    let lists = seen.lock().unwrap().clone();
    assert_eq!(lists.len(), 2);
    assert!(lists[0].iter().any(|w| w == "black"));
    assert!(!lists[1].iter().any(|w| w == "black"));
    let prot: Vec<String> = t
        .obj_now(knight)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .map(|k| format!("{:?}", k.filter))
        .collect();
    assert_eq!(prot, vec!["Some(Color(Red))".to_string()]);
}

#[test]
fn artificial_evolution_doesnt_change_names() {
    cr!("612.2", "201.2");
    ruling!(
        "Artificial Evolution",
        "You can't change proper nouns (i.e. card names) such as “Island Fish Jasconius”."
    );
    supported("Goblin King");
    // Goblin King ("Other Goblins get +1/+1 and have mountainwalk."): Goblin -> Elf.
    let mut t = TestGame::new(2);
    let king = t.battlefield(P0, "Goblin King");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let gob = t.battlefield(P0, "Raging Goblin");
    assert_eq!(t.pt(gob), (2, 2));
    choose_words(&mut t, P0, &["Goblin", "Elf"]);
    cast_at(&mut t, P0, "Artificial Evolution", &[obj(king)]);
    t.resolve_all();
    assert_eq!(t.obj_now(king).chars.name, "Goblin King");
    assert!(t.obj_now(king).chars.has_subtype("Elf"));
    assert_eq!(t.pt(elf), (2, 2));
    assert_eq!(t.pt(gob), (1, 1));
}

#[test]
fn new_blood_changes_part_of_a_word_and_any_creature_type() {
    cr!("612.2", "612.1");
    ruling!(
        "New Blood",
        "The type-changing effect can change part of a word (such as \"non-Human\") if the part of the word is being used to refer to a creature type."
    );
    ruling!(
        "New Blood",
        "You choose what word you're changing as New Blood resolves. You don't have to choose a word that appears in the creature's text."
    );
    ruling!(
        "New Blood",
        "To pay the additional cost of New Blood, you may tap any untapped Vampire you control, including one you haven't controlled continuously since the beginning of your most recent turn."
    );
    supported("Diligent Zookeeper");
    supported("Elite Vanguard");
    supported("Vampire Interloper");
    // Diligent Zookeeper: "Each non-Human creature you control gets +1/+1 for each of its
    // creature types, to a maximum of 10." Human -> Vampire.
    let mut t = TestGame::new(2);
    let zoo = t.battlefield(P1, "Diligent Zookeeper");
    let vamp = t.battlefield_sick(P0, "Vampire Interloper");
    let human = t.battlefield(P0, "Elite Vanguard");
    let seen = choose_words(&mut t, P0, &["Human", "Vampire"]);
    cast_at(&mut t, P0, "New Blood", &[obj(zoo)]);
    // The summoning-sick Vampire was tapped for the cost.
    assert!(t.obj_now(vamp).tapped);
    assert!(seen.lock().unwrap().is_empty());
    t.resolve_all();
    assert_eq!(ctl(&t, zoo), P0);
    // "non-Vampire" now: the Human gets +2/+2, the Vampire nothing.
    assert_eq!(t.pt(human), (4, 3));
    assert_eq!(t.pt(vamp), (2, 1));
    // A word that isn't on the creature: it's still taken, nothing else changes.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Vampire Interloper");
    choose_words(&mut t, P0, &["Elf", "Vampire"]);
    cast_at(&mut t, P0, "New Blood", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    assert!(t.obj_now(bears).chars.has_subtype("Bear"));
    assert!(!t.obj_now(bears).chars.has_subtype("Vampire"));
}
