//! Rulings batch P090 — "look at / reveal the top N cards, put a creature card (and/or
//! another kind of card) into your hand or onto the battlefield, the rest elsewhere"
//! (CR 701.20, 401.4), with Brass Herald's chosen creature type (CR 205.3m, 614.12) and
//! Aethermage's Touch's granted end-step trigger (CR 400.7, 707.2).

use crate::r_s01_common::*;
use crate::r_s04_common::{graveyard_names, hand_names};
use crate::r_s06_common::damage;
use crate::r_s15_common::{library_bottom_n, library_top_n};
use crate::r_s17_common::token_copy;
use crate::r_s24_common::choose_creature_type;
use crate::r_s35_common::creature_type_options;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|i| Entity::Object(*i)).collect()
}

fn names_of(t: &TestGame, ids: &[ObjectId]) -> Vec<String> {
    let mut v: Vec<String> = ids
        .iter()
        .map(|i| t.g.obj(*i).chars.name.to_string())
        .collect();
    v.sort();
    v
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

/// Casts the sorcery or instant `name` from P0's hand with lands for its cost, queueing
/// `choice` (the cards chosen from among the looked-at cards) first, and resolves it.
fn cast_choosing(t: &mut TestGame, name: &str, choice: &[ObjectId]) {
    give_mana_for(t, P0, name);
    t.answer_choose(P0, &objs(choice));
    let spell = t.hand(P0, name);
    t.cast(P0, spell).go();
    t.resolve();
}

/// Brass Herald enters under P0's control with `ty` chosen, its reveal trigger resolved.
fn herald(t: &mut TestGame, ty: &str) -> ObjectId {
    choose_creature_type(t, P0, ty);
    let h = t.enter(P0, "Brass Herald");
    t.settle();
    t.resolve_all();
    h
}

#[test]
fn brass_herald_leaving_can_make_marked_damage_lethal() {
    cr!("613.4c", "704.5g", "120.6");
    ruling!(
        "Brass Herald",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to creatures of the chosen type may become lethal if Brass Herald leaves the battlefield during that turn."
    );
    supported("Brass Herald");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let h = herald(&mut t, "Bear");
    assert_eq!(t.pt(bears), (3, 3));
    let bolt = t.battlefield(P1, "Prodigal Pyromancer");
    damage(&mut t, bolt, 2, bears);
    t.settle();
    assert!(t.on_battlefield(bears), "3/3 with 2 damage survives");
    crate::r_s02_common::destroy(&mut t, h);
    t.settle();
    assert!(!t.on_battlefield(bears), "2 damage is lethal for a 2/2");
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn brass_herald_chooses_as_it_enters_and_the_bonus_applies_at_once() {
    cr!("614.12", "614.1c", "205.3m");
    ruling!(
        "Brass Herald",
        "The choice of creature type is made as Brass Herald enters the battlefield. Players can't take any actions between the time the choice is made and the time the appropriate creatures begin to get +1/+1."
    );
    ruling!(
        "Brass Herald",
        "You must choose an existing creature type, such as Human or Warrior. You could even choose Coward, if you dared. Card types such as artifact and supertypes such as legendary can't be chosen."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // The choice is made before Brass Herald is on the battlefield.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseOption { options, .. } if options.iter().any(|o| o == "Human")),
        |g| g.permanents().any(|o| o.chars.name == "Brass Herald"),
    );
    let from = t.asked().len();
    choose_creature_type(&mut t, P0, "Coward");
    let h = t.enter(P0, "Brass Herald");
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    // No decision at all (no priority) between the choice and the entry.
    let asked = asked_since(&t, from);
    assert_eq!(asked.len(), 1, "{asked:?}");
    let opts = &creature_type_options(&t, P0, from)[0];
    for ok in ["Human", "Warrior", "Coward"] {
        assert!(opts.iter().any(|o| o == ok), "{ok}");
    }
    for no in ["Artifact", "Legendary", "Creature", "Basic"] {
        assert!(!opts.iter().any(|o| o == no), "{no}");
    }
    // A Coward gets +1/+1 as soon as Brass Herald is on the battlefield; the Bears
    // don't.
    let hulk = t.battlefield(P0, "Craven Hulk");
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(hulk), (5, 5));
    assert!(t.on_battlefield(h));

    // Choosing Bear: the Bears get +1/+1 with no priority in between.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Brass Herald");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn guild_feud_targets_the_opponent_who_goes_first() {
    cr!("115.1", "608.2c", "701.14a");
    ruling!(
        "Guild Feud",
        "Guild Feud has one target: the opponent. You'll see what creature that player puts onto the battlefield before revealing the top three cards of your library and deciding whether to put a creature onto the battlefield yourself."
    );
    supported("Guild Feud");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guild Feud");
    t.advance_to(P1, Step::End);
    let theirs = stack_library(&mut t, P1, &["Hill Giant", "Island", "Shock"]);
    t.answer_choose(P1, &[Entity::Object(theirs[0])]);
    let mine = stack_library(&mut t, P0, &["Grizzly Bears", "Forest", "Plains"]);
    // When P0 chooses, P1's creature is already on the battlefield.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }),
        |g| g.permanents().any(|o| o.chars.name == "Hill Giant"),
    );
    t.answer_choose(P0, &[Entity::Object(mine[0])]);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    // One target: the opponent.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(
        mtg_engine::copy::chosen_targets(&t.g, t.g.stack[0]),
        vec![Entity::Player(P1)]
    );
    t.resolve_all();
    assert_eq!(
        *seen.lock().unwrap(),
        vec![true],
        "{:?}\n{}",
        asked_since(&t, from),
        t.dump_log()
    );
    // They fought: Hill Giant (3/3) kills the Bears (2/2) and survives with 2 damage.
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(
        crate::r_s07_common::damage_on(&t, t.named_on_battlefield("Hill Giant")[0]),
        2
    );
    assert!(t.in_graveyard(P1, "Island") && t.in_graveyard(P1, "Shock"));
}

#[test]
fn selective_adaptation_a_card_with_two_listed_keywords_stands_for_either() {
    cr!("608.2c");
    ruling!(
        "Selective Adaptation",
        "If a card has more than one of the listed keywords, you pick which keyword you’re choosing it for."
    );
    supported("Selective Adaptation");
    // Serra Angel has flying and vigilance; Wind Drake has flying; Knight of the White
    // Orchid has first strike; Elite Vanguard has none.
    for (angel_for_flying, other) in [(true, "Ornithopter"), (false, "Wind Drake")] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 6);
        let ids = stack_library(
            &mut t,
            P0,
            &[
                "Serra Angel",
                other,
                "Elite Vanguard",
                "Forest",
                "Forest",
                "Forest",
                "Forest",
            ],
        );
        // Ornithopter has flying too: with the Angel standing for flying, a second flier
        // can't be chosen, but with the Angel for vigilance, Wind Drake stands for flying.
        let chosen = if angel_for_flying {
            vec![ids[0]]
        } else {
            vec![ids[0], ids[1]]
        };
        t.answer_choose(P0, &objs(&chosen));
        t.answer_choose(P0, &[Entity::Object(ids[0])]);
        let spell = t.hand(P0, "Selective Adaptation");
        t.cast(P0, spell).go();
        t.resolve();
        assert_eq!(
            t.named_on_battlefield("Serra Angel").len(),
            1,
            "{}",
            t.dump_log()
        );
        if !angel_for_flying {
            assert!(t.in_hand(P0, "Wind Drake"), "{}", t.dump_log());
        }
    }
    // Serra Angel and Wind Drake both chosen: legal only because the Angel stands for
    // vigilance. Two fliers without another listed keyword can't both be chosen.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let ids = stack_library(
        &mut t,
        P0,
        &[
            "Wind Drake",
            "Ornithopter",
            "Forest",
            "Forest",
            "Forest",
            "Forest",
            "Forest",
        ],
    );
    t.answer_choose(P0, &objs(&[ids[0], ids[1]]));
    t.answer_choose(P0, &[Entity::Object(ids[0])]);
    let spell = t.hand(P0, "Selective Adaptation");
    t.cast(P0, spell).go();
    t.resolve();
    let both = t.named_on_battlefield("Wind Drake").len()
        + t.named_on_battlefield("Ornithopter").len()
        + hand_names(&t, P0)
            .iter()
            .filter(|n| *n == "Wind Drake" || *n == "Ornithopter")
            .count();
    assert_eq!(both, 1, "only one flier chosen: {}", t.dump_log());
}

#[test]
fn creature_and_or_other_kind_choices() {
    cr!("608.2c", "205.2a");
    ruling!(
        "Gift of the Gargantuan",
        "If one of the cards is Dryad Arbor (a creature land card), you may reveal Dryad Arbor and another land card or Dryad Arbor and another creature card."
    );
    ruling!(
        "Gift of the Gargantuan",
        "You may reveal a creature card, or you may reveal a land card, or you may reveal both a creature card and a land card."
    );
    ruling!(
        "Benefaction of Rhonas",
        "While resolving Benefaction of Rhonas, you could put no cards, a creature card, an enchantment card, or a creature card and an enchantment card into your hand."
    );
    ruling!(
        "Relentless Pursuit",
        "While resolving Relentless Pursuit, you could put no cards, a creature card, a land card, or a creature card and a land card into your hand."
    );
    // (spell, the five top cards: a creature, a card of the other kind, a second
    // creature, a second card of the other kind, a filler; where the rest go)
    let cases: [(&str, [&str; 4], Zone); 3] = [
        (
            "Gift of the Gargantuan",
            ["Grizzly Bears", "Forest", "Hill Giant", "Island"],
            Zone::Library(P0),
        ),
        (
            "Benefaction of Rhonas",
            ["Grizzly Bears", "Pacifism", "Hill Giant", "Glorious Anthem"],
            Zone::Graveyard(P0),
        ),
        (
            "Relentless Pursuit",
            ["Grizzly Bears", "Forest", "Hill Giant", "Island"],
            Zone::Graveyard(P0),
        ),
    ];
    for (spell, cards, rest) in cases {
        supported(spell);
        let lib = [cards[0], cards[1], cards[2], cards[3], "Shock"];
        // Legal choices: none, a creature, the other kind, one of each.
        for pick in [vec![], vec![0], vec![1], vec![0, 1]] {
            let mut t = TestGame::new(2);
            t.library_top(P0, "Plains");
            let ids = stack_library(&mut t, P0, &lib);
            let chosen: Vec<ObjectId> = pick.iter().map(|i| ids[*i]).collect();
            cast_choosing(&mut t, spell, &chosen);
            let expect: Vec<String> = sorted(pick.iter().map(|i| cards[*i].to_string()).collect());
            assert_eq!(sorted(hand_names(&t, P0)), expect, "{spell} {pick:?}");
            for i in 0..4 {
                if !pick.contains(&i) {
                    assert_eq!(t.zone(ids[i]), rest, "{spell} {pick:?} {}", cards[i]);
                }
            }
        }
        // Two creatures (or two of the other kind) aren't a legal choice.
        for pick in [vec![0, 2], vec![1, 3]] {
            let mut t = TestGame::new(2);
            t.library_top(P0, "Plains");
            let ids = stack_library(&mut t, P0, &lib);
            let chosen: Vec<ObjectId> = pick.iter().map(|i| ids[*i]).collect();
            cast_choosing(&mut t, spell, &chosen);
            assert!(
                t.hand_size(P0) <= 1,
                "{spell} {pick:?}: {:?}",
                hand_names(&t, P0)
            );
        }
    }
    // Dryad Arbor stands for the creature or the land.
    for other in ["Forest", "Grizzly Bears"] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Plains");
        let ids = stack_library(&mut t, P0, &["Dryad Arbor", other, "Shock", "Shock"]);
        cast_choosing(&mut t, "Gift of the Gargantuan", &[ids[0], ids[1]]);
        assert_eq!(
            sorted(hand_names(&t, P0)),
            sorted(vec!["Dryad Arbor".to_string(), other.to_string()]),
            "{other}"
        );
    }
}

#[test]
fn commune_and_aethermages_touch_put_everything_on_the_bottom_without_a_creature() {
    cr!("401.4", "608.2c");
    ruling!(
        "Commune with Nature",
        "If you don't reveal a creature card, put all the cards on the bottom of your library in any order."
    );
    ruling!(
        "Aethermage's Touch",
        "If you don't reveal any creature cards, or if you choose not to put a revealed creature card onto the battlefield, all the revealed cards go on the bottom of your library."
    );
    supported("Commune with Nature");
    supported("Aethermage's Touch");
    for (spell, n, top) in [
        (
            "Commune with Nature",
            5,
            ["Forest", "Shock", "Island", "Plains", "Swamp"],
        ),
        (
            "Commune with Nature",
            5,
            ["Forest", "Grizzly Bears", "Island", "Plains", "Swamp"],
        ),
        (
            "Aethermage's Touch",
            4,
            ["Forest", "Shock", "Island", "Plains", "Swamp"],
        ),
        (
            "Aethermage's Touch",
            4,
            ["Forest", "Grizzly Bears", "Island", "Plains", "Swamp"],
        ),
    ] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Mountain");
        let ids = stack_library(&mut t, P0, &top[..n]);
        // Nothing chosen (a creature, when there is one, declined).
        cast_choosing(&mut t, spell, &[]);
        assert_eq!(t.hand_size(P0), 0, "{spell}");
        assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
        let bottom = library_bottom_n(&t, P0, n);
        assert_eq!(names_of(&t, &bottom), names_of(&t, &ids), "{spell}");
        assert_eq!(
            names_of(&t, &library_top_n(&t, P0, 1)),
            ["Mountain"],
            "{spell}"
        );
    }
}

#[test]
fn aethermages_touch_creature_leaving_first_isnt_returned() {
    cr!("400.7");
    ruling!(
        "Aethermage's Touch",
        "If the creature that you put onto the battlefield leaves the battlefield before your next end step, it won't be returned to your hand at the beginning of your next end step."
    );
    let mut t = TestGame::new(2);
    let ids = stack_library(&mut t, P0, &["Serra Angel", "Forest", "Forest", "Forest"]);
    cast_choosing(&mut t, "Aethermage's Touch", &[ids[0]]);
    let angel = t.named_on_battlefield("Serra Angel")[0];
    crate::r_s02_common::destroy(&mut t, angel);
    // It comes back from the graveyard as a new object, without the granted ability.
    crate::r_s05_common::move_to(&mut t, angel, Zone::Battlefield);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 1);
    assert!(!t.in_hand(P0, "Serra Angel"));
}

#[test]
fn aethermages_touch_during_your_end_step_waits_for_the_next_one() {
    cr!("603.2b", "513.2");
    ruling!(
        "Aethermage's Touch",
        "If you put a creature onto the battlefield this way during your end step, it won't be returned to your hand until your next end step."
    );
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::End);
    t.settle();
    let ids = stack_library(&mut t, P0, &["Serra Angel", "Forest", "Forest", "Forest"]);
    cast_choosing(&mut t, "Aethermage's Touch", &[ids[0]]);
    t.resolve_all();
    let angel = t.named_on_battlefield("Serra Angel");
    assert_eq!(angel.len(), 1);
    // Through P1's turn, then P0's next end step.
    t.advance_to(P1, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(angel[0]));
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.in_hand(P0, "Serra Angel"), "{}", t.dump_log());
}

#[test]
fn aethermages_touch_granted_trigger_isnt_copied() {
    cr!("707.2", "613.1a");
    ruling!(
        "Aethermage's Touch",
        "The triggered ability that the creature gains won't be copied if an effect creates a token that's a copy of that creature or causes another object to become a copy of that creature."
    );
    let mut t = TestGame::new(2);
    let ids = stack_library(&mut t, P0, &["Serra Angel", "Forest", "Forest", "Forest"]);
    cast_choosing(&mut t, "Aethermage's Touch", &[ids[0]]);
    let angel = t.named_on_battlefield("Serra Angel")[0];
    let copies = token_copy(&mut t, P0, angel);
    assert_eq!(copies.len(), 1);
    // A Clone copying it too.
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s17_common::become_copy(&mut t, bears, angel);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.in_hand(P0, "Serra Angel"));
    assert!(t.on_battlefield(copies[0]), "the token copy stays");
    assert!(
        t.on_battlefield(bears),
        "the creature that became a copy stays"
    );
}

#[test]
fn adventure_awaits_draws_the_card_after_the_five() {
    cr!("121.1", "401.4");
    ruling!(
        "Adventure Awaits",
        "If you draw a card this way, that card is the next one in your library after putting the five cards from the top of your library on the bottom."
    );
    supported("Adventure Awaits");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Swamp");
    t.library_top(P0, "Mountain");
    stack_library(
        &mut t,
        P0,
        &["Forest", "Shock", "Island", "Plains", "Grizzly Bears"],
    );
    cast_choosing(&mut t, "Adventure Awaits", &[]);
    assert_eq!(hand_names(&t, P0), ["Mountain"]);
}

#[test]
fn cantankerous_keepers_cost_reduction_doesnt_change_mana_value() {
    cr!("601.2f", "202.3", "702.41a");
    ruling!(
        "Cantankerous Keepers",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Cantankerous Keepers's first ability). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Cantankerous Keepers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Elvish Mystic");
    // {5}{G} less {2} for the two Elves.
    t.lands(P0, "Forest", 4);
    let k = t.hand(P0, "Cantankerous Keepers");
    // Tap only the lands (the Elves stay untapped).
    t.cast(P0, k).go();
    assert_eq!(t.zone(k), Zone::Stack, "castable for {{3}}{{G}}");
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(crate::r_s08_common::mana_value(&t, k), 6);
    // The cost increase first, then the reduction: with Sphere of Resistance ("Spells
    // cost {1} more to cast.") and six Elves, {5}{G} plus {1} less {6} is {G} (reducing
    // first would leave {1}{G}).
    supported("Sphere of Resistance");
    for elves in [5, 6] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Sphere of Resistance");
        for _ in 0..elves {
            t.battlefield(P0, "Elvish Warrior");
        }
        t.lands(P0, "Forest", 1);
        let k = t.hand(P0, "Cantankerous Keepers");
        let normal = mtg_engine::object::CastMethod::Normal;
        let castable = crate::r_s02_common::can_cast(&mut t, P0, k, normal);
        // With five Elves, {1}{G}: one Forest isn't enough.
        assert_eq!(castable, elves == 6, "{elves} Elves");
        if castable {
            t.cast(P0, k).go();
            assert_eq!(t.zone(k), Zone::Stack, "castable for {{G}}");
            assert_eq!(tapped_lands(&t, P0), 1);
            assert_eq!(crate::r_s08_common::mana_value(&t, k), 6);
        }
    }
}

#[test]
fn trackers_instincts_must_take_a_creature() {
    cr!("608.2c");
    ruling!(
        "Tracker's Instincts",
        "You must put a creature card from among the revealed cards into your hand if one is there, even if you'd rather put all the revealed cards into your graveyard."
    );
    supported("Tracker's Instincts");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Shock", "Island"]);
    // Choosing nothing isn't allowed: the creature goes to hand anyway.
    cast_choosing(&mut t, "Tracker's Instincts", &[]);
    assert_eq!(hand_names(&t, P0), ["Grizzly Bears"]);
    assert_eq!(
        sorted(graveyard_names(&t, P0)),
        ["Forest", "Island", "Shock", "Tracker's Instincts"]
    );
}

#[test]
fn telling_time_with_fewer_than_three_cards() {
    cr!("608.2c");
    ruling!(
        "Telling Time",
        "If there are fewer than three cards in your library, follow the instructions in the order given for any cards that are there."
    );
    supported("Telling Time");
    // Two cards: one into hand, the other on top.
    let mut t = TestGame::new(2);
    crate::r_s11_common::empty_library(&mut t, P0);
    let ids = stack_library(&mut t, P0, &["Forest", "Shock"]);
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.answer_choose(P0, &[Entity::Object(ids[0])]);
    give_mana_for(&mut t, P0, "Telling Time");
    let spell = t.hand(P0, "Telling Time");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(hand_names(&t, P0), ["Shock"]);
    assert_eq!(t.library_size(P0), 1);
    // One card: it goes into hand.
    let mut t = TestGame::new(2);
    crate::r_s11_common::empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Forest"]);
    give_mana_for(&mut t, P0, "Telling Time");
    let spell = t.hand(P0, "Telling Time");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(hand_names(&t, P0), ["Forest"]);
    assert_eq!(t.library_size(P0), 0);
}
