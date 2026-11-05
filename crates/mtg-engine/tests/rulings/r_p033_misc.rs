//! Rulings batch P033 — the other abilities of the cost reducers' cards and the choices
//! their reductions depend on: names (CR 201.4, 709), colors, card types and companion
//! conditions, triggered abilities (Gavi, Ranar, Archmage of Runes), searches (Eye of
//! Ugin), loyalty abilities (Urza, Planeswalker), counters (Loading Zone, Ferrafor, charge
//! counters) and modes with targets (Umezawa's Jitte).

use crate::r_p033_common::*;
use crate::r_s01_common::{supported, tapped_lands, tokens, with_subtype};
use crate::r_s02_common::create_token;
use crate::r_s04_common::cycle;
use crate::r_s06_common::activate_containing;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

/// `p` casts `name` from hand (from a full pool) and returns what it cost.
fn cast(t: &mut TestGame, p: PlayerId, name: &str) -> Paid {
    let card = t.hand(p, name);
    cast_normal(t, p, card)
}

/// P0's Council of the Absolute, naming `name` as it enters.
fn council_naming(t: &mut TestGame, name: &str) -> ObjectId {
    t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
    let c = t.enter(P0, "Council of the Absolute");
    t.resolve_all();
    c
}

#[test]
fn council_of_the_absolute_reduces_generic_mana_only() {
    cr!("601.2f", "118.7a", "201.4");
    ruling!(
        "Council of the Absolute",
        "If a spell with the chosen name includes one generic mana in its cost to cast, that cost will be reduced by {1}."
    );
    ruling!(
        "Council of the Absolute",
        "The last ability can’t reduce the colored mana requirement of a spell."
    );
    supported("Council of the Absolute");
    // "Spells with the chosen name you cast cost {2} less to cast." Think Twice ({1}{U}):
    // {U}; Brainstorm ({U}): {U}.
    let mut t = main_phase(2);
    council_naming(&mut t, "Think Twice");
    let paid = cast(&mut t, P0, "Think Twice");
    assert_spent(&paid.spent, "{U}", "Think Twice");
    t.resolve_all();
    // Another name isn't reduced.
    let paid = cast(&mut t, P0, "Divination");
    assert_spent(&paid.spent, "{2}{U}", "Divination");
    let mut t = main_phase(2);
    council_naming(&mut t, "Brainstorm");
    let paid = cast(&mut t, P0, "Brainstorm");
    assert_spent(&paid.spent, "{U}", "Brainstorm");
}

#[test]
fn council_of_the_absolute_names_one_half_of_a_split_card() {
    cr!("201.4", "709.3", "702.102c", "601.2f");
    ruling!(
        "Council of the Absolute",
        "If you name half of a split card with fuse, and then you cast both halves, the cost reduction will apply to the total cost of the spell. This is true even if the half with the chosen name requires only colored mana to cast."
    );
    ruling!(
        "Council of the Absolute",
        "You can name either half of a split card, but not both. If you name half of a split card, your opponents can cast the other half. If that split card has fuse, your opponents can’t cast that card as a fused split card."
    );
    const WT: &str = "Wear // Tear";
    supported(WT);
    let fused = CastMethod::Keyword(KeywordKind::Fuse);
    // Wear ({1}{R}: "Destroy target artifact.") // Tear ({W}: "Destroy target
    // enchantment."), fuse. Named Tear, the fused spell's total cost {1}{R}{W} is {R}{W}.
    let mut t = main_phase(2);
    council_naming(&mut t, "Tear");
    let stone = t.battlefield(P1, "Mind Stone");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_targets(P0, &[obj(stone)]);
    t.answer_targets(P0, &[obj(anthem)]);
    let wt = t.hand(P0, WT);
    let paid = cast_paying(&mut t, P0, wt, fused.clone()).expect("fused");
    assert_spent(&paid.spent, "{R}{W}", "fused Wear // Tear");
    t.resolve_all();
    assert!(!t.on_battlefield(stone) && !t.on_battlefield(anthem));
    // Tear alone costs {W}.
    let mut t = main_phase(2);
    council_naming(&mut t, "Tear");
    let a = t.battlefield(P1, "Glorious Anthem");
    t.answer_targets(P0, &[obj(a)]);
    let wt = t.hand(P0, WT);
    let paid = cast_paying(&mut t, P0, wt, CastMethod::Half(1)).expect("Tear");
    assert_spent(&paid.spent, "{W}", "Tear");
    // P1 can cast Wear, but not Tear or the fused spell.
    let mut t = main_phase(2);
    council_naming(&mut t, "Tear");
    let stone = t.battlefield(P0, "Mind Stone");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    t.set_step(P1, Step::PrecombatMain);
    for (method, ok) in [(CastMethod::Half(1), false), (fused.clone(), false), (CastMethod::Half(0), true)] {
        let wt = t.hand(P1, WT);
        t.clear_answers();
        t.answer_targets(P1, &[obj(stone)]);
        t.answer_targets(P1, &[obj(anthem)]);
        let r = cast_paying(&mut t, P1, wt, method.clone());
        assert_eq!(r.is_some(), ok, "{method:?}");
    }
    // Both names at once ("Wear // Tear") isn't a card name that can be chosen: nothing is
    // named, and P1 can cast Tear.
    let mut t = main_phase(2);
    let c = council_naming(&mut t, WT);
    assert_eq!(t.obj(c).choices.card_name.as_deref(), Some(""));
    let anthem = t.battlefield(P0, "Glorious Anthem");
    t.set_step(P1, Step::PrecombatMain);
    let wt = t.hand(P1, WT);
    t.answer_targets(P1, &[obj(anthem)]);
    assert!(cast_paying(&mut t, P1, wt, CastMethod::Half(1)).is_some());
}

#[test]
fn cheering_fanatic_may_name_any_card() {
    cr!("201.4", "601.2f");
    ruling!(
        "Cheering Fanatic",
        "You don’t need to choose the name of a card anyone intends to cast this turn. You also don’t need to choose the name of a card that can have its cost reduced. For example, some Cheering Fanatics are enthusiastic about Shock or Mountain."
    );
    supported("Cheering Fanatic");
    // "Whenever this creature attacks, choose a card name. Spells with the chosen name
    // cost {1} less to cast this turn."
    let attack_naming = |name: &str| {
        let mut t = main_phase(2);
        let f = t.battlefield(P0, "Cheering Fanatic");
        t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
        t.attack(&[(f, Entity::Player(P1))], &[]);
        t.resolve_all();
        (t, f)
    };
    for name in ["Mountain", "Shock", "Divination"] {
        let (t, f) = attack_naming(name);
        let asked = t
            .asked()
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::NameCard { .. }))
            .count();
        assert_eq!(asked, 1, "{name}");
        assert_eq!(t.obj(f).choices.card_name.as_deref(), Some(name));
    }
    // Shock ({R}) still costs {R}; Divination named costs {1}{U} (the reduction works).
    let (mut t, _) = attack_naming("Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let paid = cast(&mut t, P0, "Shock");
    assert_spent(&paid.spent, "{R}", "Shock");
    let (mut t, _) = attack_naming("Divination");
    t.set_step(P0, Step::PostcombatMain);
    let paid = cast(&mut t, P0, "Divination");
    assert_spent(&paid.spent, "{1}{U}", "Divination");
}

#[test]
fn seal_of_the_guildpact_two_different_colors_reduce_generic_mana_only() {
    cr!("105.1", "601.2f", "118.7a", "202.3");
    ruling!("Seal of the Guildpact", "You must choose two different colors.");
    ruling!(
        "Seal of the Guildpact",
        "Seal of the Guildpact's last ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic component of that cost."
    );
    ruling!(
        "Seal of the Guildpact",
        "Seal of the Guildpact's last ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    supported("Seal of the Guildpact");
    // "As this artifact enters, choose two colors. Each spell you cast costs {1} less to
    // cast for each of the chosen colors it is." Every option is two different colors.
    let mut t = main_phase(2);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(9));
    let seal = t.enter(P0, "Seal of the Guildpact");
    let options = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .expect("asked for the colors");
    assert_eq!(options.len(), 10, "{options:?}");
    for o in &options {
        let words: Vec<&str> = o.split(" and ").collect();
        assert!(words.len() == 2 && words[0] != words[1], "{o}");
    }
    let colors = t.obj(seal).choices.colors.unwrap();
    assert!(colors.contains(Color::Red) && colors.contains(Color::Green));
    assert_eq!(colors.iter().count(), 2);
    // Bloodbraid Elf ({2}{R}{G}, red and green): {R}{G}, mana value 4. Kird Ape ({R}):
    // {R}.
    let paid = cast(&mut t, P0, "Bloodbraid Elf");
    assert_spent(&paid.spent, "{R}{G}", "Bloodbraid Elf");
    assert_eq!(paid.mv, 4);
    t.resolve_all();
    let paid = cast(&mut t, P0, "Kird Ape");
    assert_spent(&paid.spent, "{R}", "Kird Ape");
    assert_eq!(paid.mv, 1);
}

#[test]
fn gravebreaker_lamia_doesnt_let_you_cast_cards_from_your_graveyard() {
    cr!("601.2f", "601.3", "702.138a", "702.34a");
    ruling!(
        "Gravebreaker Lamia",
        "Gravebreaker Lamia’s last ability doesn’t give you permission to cast cards from your graveyard. You need something else allowing you to do so, such as the escape keyword ability."
    );
    supported("Gravebreaker Lamia");
    supported("Satyr's Cunning");
    // "Spells you cast from your graveyard cost {1} less to cast."
    let mut t = main_phase(2);
    t.battlefield(P0, "Gravebreaker Lamia");
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(cast_paying(&mut t, P0, bears, CastMethod::Normal).is_none());
    // Satyr's Cunning's "Escape—{2}{R}, Exile two other cards from your graveyard.":
    // {1}{R}.
    let sc = t.graveyard(P0, "Satyr's Cunning");
    t.graveyard(P0, "Kird Ape");
    let paid = cast_paying(&mut t, P0, sc, CastMethod::Keyword(KeywordKind::Escape)).expect("escape");
    assert_spent(&paid.spent, "{1}{R}", "escaped Satyr's Cunning");
    t.resolve_all();
    // Think Twice's flashback {2}{U}: {1}{U}.
    let tt = t.graveyard(P0, "Think Twice");
    let paid =
        cast_paying(&mut t, P0, tt, CastMethod::Keyword(KeywordKind::Flashback)).expect("flashback");
    assert_spent(&paid.spent, "{1}{U}", "Think Twice flashback");
}

#[test]
fn fortune_tellers_talent_reduces_generic_mana_of_spells_cast_from_elsewhere() {
    cr!("601.2f", "118.7a", "716.2a");
    ruling!(
        "Fortune Teller's Talent",
        "The cost reduction applies only to generic mana in the total cost of spells you cast from anywhere other than your hand."
    );
    supported("Fortune Teller's Talent");
    // Level 3: "Spells you cast from anywhere other than your hand cost {2} less to cast."
    // (Level 2: "As long as you've cast a spell this turn, you may play cards from the top
    // of your library.")
    let mut t = main_phase(2);
    let talent = t.battlefield(P0, "Fortune Teller's Talent");
    mtg_engine::classes::set_level(&mut t.g, talent, 3);
    t.g.recompute();
    // From hand: Divination costs {2}{U}.
    let paid = cast(&mut t, P0, "Divination");
    assert_spent(&paid.spent, "{2}{U}", "Divination from hand");
    t.resolve_all();
    // From the graveyard: Think Twice's flashback {2}{U}: {U} (its {U} stays).
    let tt = t.graveyard(P0, "Think Twice");
    let paid =
        cast_paying(&mut t, P0, tt, CastMethod::Keyword(KeywordKind::Flashback)).expect("flashback");
    assert_spent(&paid.spent, "{U}", "Think Twice flashback");
    t.resolve_all();
    // From the top of the library: Hieroglyphic Illumination ({3}{U}): {1}{U}.
    let hi = t.library_top(P0, "Hieroglyphic Illumination");
    let paid = cast_paying(&mut t, P0, hi, CastMethod::Normal).expect("from the library");
    assert_spent(&paid.spent, "{1}{U}", "from the library");
}

#[test]
fn gavi_free_cycling_makes_x_zero_and_its_draw_trigger_is_once_a_turn() {
    cr!("107.3b", "118.9", "702.29a", "603.2");
    ruling!(
        "Gavi, Nest Warden",
        "If a cycling cost includes {X} and you pay an alternative activation cost of {0} instead, X is 0."
    );
    ruling!(
        "Gavi, Nest Warden",
        "The triggered ability can trigger only once each turn. It doesn’t matter Gavi was on the battlefield when the first card was drawn. If it’s not on the battlefield when the second card is drawn, the ability can’t trigger at all that turn. It won’t trigger when the third or fourth card is drawn."
    );
    supported("Gavi, Nest Warden");
    supported("Shark Typhoon");
    // Shark Typhoon: "Cycling {X}{1}{U}", "When you cycle this card, create an X/X blue
    // Shark creature token with flying." Cycled for {0} (Gavi's option, index 1), X is 0:
    // the Shark is 0/0 (an X of 3 queued isn't used).
    let mut t = main_phase(2);
    t.battlefield(P0, "Gavi, Nest Warden");
    let shark = t.hand(P0, "Shark Typhoon");
    crate::r_p046_common::fill_pool(&mut t, P0, 5);
    let before = crate::r_p046_common::pool(&t, P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    cycle(&mut t, P0, shark, 0).expect("cycle for {0}");
    assert_eq!(crate::r_p046_common::pool(&t, P0), before, "mana was paid");
    t.resolve_all();
    let created: Vec<ObjectId> = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::TokenCreated { obj, .. } => Some(*obj),
            _ => None,
        })
        .collect();
    assert_eq!(created.len(), 1, "a Shark was created");
    assert!(with_subtype(&t, P0, "Shark").is_empty(), "a 0/0 Shark dies");
    // "Whenever you draw your second card each turn, create a 2/2 red and white Dinosaur
    // Cat creature token."
    let draw = |t: &mut TestGame, n: u32| {
        for _ in 0..n {
            t.g.draw_cards(P0, 1);
            t.g.flush_events();
            t.resolve_all();
        }
    };
    let cats = |t: &TestGame| tokens(t, P0).len();
    // On the battlefield from the start: one token on the second draw, none for the third
    // and fourth.
    let mut t = main_phase(2);
    t.battlefield(P0, "Gavi, Nest Warden");
    draw(&mut t, 1);
    assert_eq!(cats(&t), 0);
    draw(&mut t, 1);
    assert_eq!(cats(&t), 1);
    draw(&mut t, 2);
    assert_eq!(cats(&t), 1);
    // Gavi enters after the first draw: the second still triggers it.
    let mut t = main_phase(2);
    draw(&mut t, 1);
    t.battlefield(P0, "Gavi, Nest Warden");
    draw(&mut t, 1);
    assert_eq!(cats(&t), 1);
    // Gavi enters after the second draw: nothing that turn.
    let mut t = main_phase(2);
    draw(&mut t, 2);
    t.battlefield(P0, "Gavi, Nest Warden");
    draw(&mut t, 2);
    assert_eq!(cats(&t), 0);
}

/// Spirit tokens P0 has.
fn spirits(t: &TestGame) -> usize {
    // (Ranar itself is a Spirit.)
    with_subtype(t, P0, "Spirit")
        .into_iter()
        .filter(|id| t.obj(*id).is_token())
        .count()
}

#[test]
fn ranar_triggers_once_for_each_exile_from_your_hand_or_by_your_spells() {
    cr!("603.2c", "603.2", "406.3", "702.143a");
    ruling!(
        "Ranar the Ever-Watchful",
        "Ranar has recieved an update to its Oracle text. Specifically, its triggered ability doesn't care which player is exiling cards and permanents. Cards put into exile from your hand for any reason, such as the foretell ability, cause the ability to trigger, and permanents exiled from the battlefield cause it to trigger if you controlled the spell or ability that exiled them."
    );
    ruling!(
        "Ranar the Ever-Watchful",
        "Ranar's triggered ability triggers only once for each time you exile cards from your hand or permanents from the battlefield, no matter how many objects were exiled at the same time."
    );
    ruling!(
        "Ranar the Ever-Watchful",
        "Ranar's triggered ability will trigger when a spell or ability you control exiles any permanent, including token permanents and permanents you don't control."
    );
    supported("Ranar the Ever-Watchful");
    // "Whenever one or more cards are put into exile from your hand or a spell or ability
    // you control exiles one or more permanents from the battlefield, create a 1/1 white
    // Spirit creature token with flying."
    // Foretelling Demon Bolt (free with Ranar's first ability): one Spirit.
    let mut t = main_phase(2);
    t.battlefield(P0, "Ranar the Ever-Watchful");
    let bolt = t.hand(P0, "Demon Bolt");
    t.g.turn.priority = Some(P0);
    t.g.take_action(P0, Action::Special(SpecialAction::Foretell { card: bolt }));
    t.g.flush_events();
    t.resolve_all();
    assert!(t.obj_now(bolt).face_down && t.zone(bolt) == Zone::Exile);
    assert_eq!(spirits(&t), 1);
    // P1's Thought-Knot Seer exiles a card from P0's hand: another.
    let shock = t.hand(P0, "Shock");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.answer_choose(P1, &[obj(shock)]);
    t.enter(P1, "Thought-Knot Seer");
    t.resolve_all();
    assert!(t.in_exile("Shock"));
    assert_eq!(spirits(&t), 2);
    // Allosaurus Rider cast by exiling two green cards from hand: one Spirit for both.
    let mut t = main_phase(2);
    t.battlefield(P0, "Ranar the Ever-Watchful");
    let rider = t.hand(P0, "Allosaurus Rider");
    let g1 = t.hand(P0, "Grizzly Bears");
    let g2 = t.hand(P0, "Craw Wurm");
    tap_all(&mut t);
    let alt = crate::r_s08_common::legal_cast_methods(&mut t, P0, rider)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("the exile-two-cards alternative cost");
    t.answer_choose(P0, &[obj(g1), obj(g2)]);
    t.cast(P0, rider).method(alt).go();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Craw Wurm"));
    t.resolve_all();
    assert_eq!(spirits(&t), 1);
    // Final Judgment ("Exile all creatures.") exiling P1's creature and token and P0's
    // own: one Spirit.
    let mut t = main_phase(2);
    t.battlefield(P0, "Ranar the Ever-Watchful");
    t.battlefield(P1, "Grizzly Bears");
    create_token(&mut t, P1, "Spirit");
    let fj = t.hand(P0, "Final Judgment");
    cast_normal(&mut t, P0, fj);
    t.resolve_all();
    assert!(t.in_exile("Ranar the Ever-Watchful"));
    // (Ranar left with the others; its leaves-the-battlefield look-back sees it.)
    assert_eq!(spirits(&t), 1);
    // Swords to Plowshares on P1's token: a Spirit; P1's Swords on P0's Bears: none.
    let mut t = main_phase(2);
    t.battlefield(P0, "Ranar the Ever-Watchful");
    let tok = create_token(&mut t, P1, "Spirit");
    t.answer_targets(P0, &[obj(tok)]);
    cast(&mut t, P0, "Swords to Plowshares");
    t.resolve_all();
    assert!(!t.g.is_live(tok) || !t.on_battlefield(tok));
    assert_eq!(spirits(&t), 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P1, &[obj(bears)]);
    cast(&mut t, P1, "Swords to Plowshares");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(spirits(&t), 1);
}

/// Taps every land P0 controls and empties nothing else (Allosaurus Rider's alternative
/// cost needs no mana).
fn tap_all(t: &mut TestGame) {
    crate::r_p046_common::tap_lands(t, P0);
}

#[test]
fn starnheim_aspirant_reduces_only_angel_creature_spells() {
    cr!("205.3m", "601.2f");
    ruling!(
        "Starnheim Aspirant",
        "An Angel spell is a creature spell with the creature type Angel. Instant, sorcery, and enchantment cards that may create Angel tokens are not Angel spells."
    );
    supported("Starnheim Aspirant");
    // "Angel spells you cast cost {2} less to cast."
    let mut t = main_phase(2);
    t.battlefield(P0, "Starnheim Aspirant");
    let paid = cast(&mut t, P0, "Serra Angel");
    assert_spent(&paid.spent, "{1}{W}{W}", "Serra Angel");
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let paid = cast(&mut t, P0, "Angelic Ascension");
    assert_spent(&paid.spent, "{1}{W}", "Angelic Ascension");
    t.resolve_all();
    let paid = cast(&mut t, P0, "Empyrial Storm");
    assert_spent(&paid.spent, "{4}{W}{W}", "Empyrial Storm");
    t.resolve_all();
    let paid = cast(&mut t, P0, "Rampage of the Valkyries");
    assert_spent(&paid.spent, "{3}{W}{B}", "Rampage of the Valkyries");
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Angel").len(), 3);
}

#[test]
fn marauding_raptor_and_jhoiras_familiar_dont_reduce_themselves() {
    cr!("601.2f", "601.2a", "700.6");
    ruling!(
        "Marauding Raptor",
        "Marauding Raptor's cost reduction doesn't affect itself."
    );
    ruling!(
        "Jhoira's Familiar",
        "The last ability of Jhoira's Familiar doesn't reduce its own cost while you're casting it."
    );
    // (card, its cost alone, with another on the battlefield)
    for (name, alone, with) in [
        ("Marauding Raptor", "{1}{R}", "{R}"),
        ("Jhoira's Familiar", "{4}", "{3}"),
    ] {
        supported(name);
        let mut t = main_phase(2);
        let paid = cast(&mut t, P0, name);
        assert_spent(&paid.spent, alone, name);
        let mut t = main_phase(2);
        t.battlefield(P0, name);
        let paid = cast(&mut t, P0, name);
        assert_spent(&paid.spent, with, name);
    }
}

#[test]
fn a_spell_is_historic_or_it_isnt() {
    cr!("700.6", "601.2f", "603.2");
    ruling!(
        "Jhoira's Familiar",
        "A spell is historic if it has the legendary supertype, the artifact card type, or the Saga enchantment subtype. Having two of those qualities doesn't make a spell more historic than another or provide an additional bonus—a spell either is historic or it isn't."
    );
    ruling!(
        "Jhoira, Weatherlight Captain",
        "A spell is historic if it has the legendary supertype, the artifact card type, or the Saga enchantment subtype. Having two of those qualities doesn't make a spell more historic than another or provide an additional bonus—a spell either is historic or it isn't."
    );
    supported("Jhoira, Weatherlight Captain");
    // Jhoira's Familiar: "Historic spells you cast cost {1} less to cast." Jhoira:
    // "Whenever you cast a historic spell, draw a card." Lithoform Engine ({4}) is a
    // legendary artifact: {3}, one card. Mind Stone ({2}, artifact): {1}, one card.
    // Divination: neither.
    let mut t = main_phase(2);
    t.battlefield(P0, "Jhoira's Familiar");
    t.battlefield(P0, "Jhoira, Weatherlight Captain");
    for (name, cost, drawn) in [
        ("Lithoform Engine", "{3}", 1),
        ("Mind Stone", "{1}", 1),
        ("Divination", "{2}{U}", 2),
    ] {
        let hand = t.hand_size(P0);
        let paid = cast(&mut t, P0, name);
        assert_spent(&paid.spent, cost, name);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + drawn, "{name}");
    }
    // A legendary creature is historic too: Thalia ({1}{W}) costs {W}, one card. A Saga
    // as well: The Mirari Conjecture ({4}{U}) costs {3}{U}, one card.
    // (The Saga first: Thalia makes noncreature spells cost {1} more.)
    for (name, cost) in [("The Mirari Conjecture", "{3}{U}"), ("Thalia, Guardian of Thraben", "{W}")] {
        let hand = t.hand_size(P0);
        let paid = cast(&mut t, P0, name);
        assert_spent(&paid.spent, cost, name);
        t.settle();
        while t.stack_len() > 0 {
            t.g.resolve_top();
            t.settle();
        }
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    }
}

#[test]
fn archmage_of_runes_draw_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Archmage of Runes",
        "Archmage of Runes's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    supported("Archmage of Runes");
    // "Whenever you cast an instant or sorcery spell, draw a card."
    let mut t = main_phase(2);
    t.battlefield(P0, "Archmage of Runes");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bolt = cast(&mut t, P0, "Lightning Bolt");
    t.settle();
    // The trigger resolves first: P0 has drawn and the Bolt is still on the stack.
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(bolt.spell), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Countered in response to the trigger: the trigger still draws.
    let mut t = main_phase(2);
    t.battlefield(P0, "Archmage of Runes");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bolt = cast(&mut t, P0, "Lightning Bolt");
    t.settle();
    let cs = t.hand(P1, "Counterspell");
    add(&mut t, P1, ManaType::U, 2);
    t.cast(P1, cs).target(obj(bolt.spell)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn urza_planeswalker_activates_loyalty_abilities_twice_each_turn() {
    cr!("606.3", "712.4", "701.42a");
    ruling!(
        "Urza, Lord Protector",
        "During each of your turns, you may activate Urza, Planeswalker's loyalty abilities twice, whether that's activating the same ability two times, or activating two different abilities once each."
    );
    supported("Urza, Lord Protector");
    // Urza, Lord Protector's "{7}: If you both own and control Urza, Lord Protector and an
    // artifact named The Mightstone and Weakstone, exile them, then meld them into Urza,
    // Planeswalker. Activate only as a sorcery."
    let planeswalker = || {
        let mut t = main_phase(2);
        let urza = t.battlefield(P0, "Urza, Lord Protector");
        t.battlefield(P0, "The Mightstone and Weakstone");
        t.lands(P0, "Wastes", 7);
        t.activate(P0, urza, 0, &[]).unwrap();
        t.resolve_all();
        let pw = t.named_on_battlefield("Urza, Planeswalker");
        assert_eq!(pw.len(), 1);
        (t, pw[0])
    };
    // The same ability ("+1: Draw two cards, then discard a card.") twice, not a third
    // time.
    let (mut t, pw) = planeswalker();
    let loyalty = t.counters(pw, counters::LOYALTY);
    for _ in 0..2 {
        activate_containing(&mut t, P0, pw, "Draw two cards").expect("+1");
        t.resolve_all();
    }
    assert_eq!(t.counters(pw, counters::LOYALTY), loyalty + 2);
    assert!(activate_containing(&mut t, P0, pw, "Draw two cards").is_err());
    assert!(activate_containing(&mut t, P0, pw, "Soldier").is_err());
    // Two different ones ("0: Create two 1/1 colorless Soldier artifact creature tokens."
    // and the +1), then no more.
    let (mut t, pw) = planeswalker();
    activate_containing(&mut t, P0, pw, "Soldier").expect("0");
    t.resolve_all();
    activate_containing(&mut t, P0, pw, "Draw two cards").expect("+1");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 2);
    assert!(activate_containing(&mut t, P0, pw, "You gain 2 life").is_err());
}

#[test]
fn eye_of_ugin_has_no_mana_ability_and_finds_colorless_creature_cards() {
    cr!("605.1a", "105.2c", "701.23a", "601.2f");
    ruling!("Eye of Ugin", "Eye of Ugin doesn’t have a mana ability.");
    ruling!(
        "Eye of Ugin",
        "Eye of Ugin’s second ability lets you find any colorless creature card in your deck, such as an artifact creature card that has no colored mana symbols in its mana cost, or a creature card that’s become colorless due to Mycosynth Lattice."
    );
    ruling!(
        "Eye of Ugin",
        "Mistform Ultimus and creatures with changeling are Eldrazi. If they become colorless, possibly due to Mycosynth Lattice, Eye of Ugin would reduce the cost to cast them by {2}."
    );
    supported("Eye of Ugin");
    supported("Mycosynth Lattice");
    // "Colorless Eldrazi spells you cast cost {2} less to cast." and "{7}, {T}: Search
    // your library for a colorless creature card, reveal it, put it into your hand, then
    // shuffle."
    let mut t = main_phase(2);
    let eye = t.battlefield(P0, "Eye of Ugin");
    assert!(t
        .obj(eye)
        .chars
        .abilities
        .iter()
        .all(|a| !matches!(&a.kind, mtg_engine::ability::AbilityKind::Activated(act) if act.is_mana_ability)));
    // The search offers Ornithopter (an artifact creature) but not Grizzly Bears; with
    // Mycosynth Lattice ("All cards that aren't on the battlefield, spells, and
    // permanents are colorless.") the Bears too.
    for lattice in [false, true] {
        let mut t = main_phase(2);
        let eye = t.battlefield(P0, "Eye of Ugin");
        if lattice {
            t.battlefield(P0, "Mycosynth Lattice");
        }
        let thopter = t.library_top(P0, "Ornithopter");
        let bears = t.library_top(P0, "Grizzly Bears");
        t.lands(P0, "Wastes", 7);
        let from = t.asked().len();
        t.answer_choose(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, eye, "Search").expect("search");
        t.resolve_all();
        let offered: Vec<Entity> = t.asked()[from..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(offered.contains(&obj(thopter)), "Lattice {lattice}: {offered:?}");
        assert_eq!(offered.contains(&obj(bears)), lattice, "Lattice {lattice}");
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), lattice);
    }
    // With Mycosynth Lattice, Mistform Ultimus ({3}{U}) and Chameleon Colossus ({2}{G}{G},
    // changeling) are colorless Eldrazi spells: {2} less (the Lattice lets any mana pay
    // the colored symbols). Without it, the Ultimus is blue: {3}{U}.
    for (name, total) in [("Mistform Ultimus", 2), ("Chameleon Colossus", 2)] {
        let mut t = main_phase(2);
        t.battlefield(P0, "Eye of Ugin");
        t.battlefield(P0, "Mycosynth Lattice");
        let paid = cast(&mut t, P0, name);
        assert_eq!(paid.spent.len(), total, "{name}: {:?}", paid.spent);
    }
    let mut t = main_phase(2);
    t.battlefield(P0, "Eye of Ugin");
    let paid = cast(&mut t, P0, "Mistform Ultimus");
    assert_spent(&paid.spent, "{3}{U}", "Mistform Ultimus, no Lattice");
}

#[test]
fn power_artifact_cant_reduce_snow_mana() {
    cr!("107.4h", "118.7a", "602.2b");
    ruling!("Power Artifact", "Can't reduce Snow mana costs.");
    // Power Artifact: "Enchanted artifact's activated abilities cost {2} less to activate.
    // This effect can't reduce the mana in that cost to less than one mana." Phyrexian
    // Ironfoot: "{1}{S}: Untap this creature." It costs {S}: one mana from a snow source
    // pays it, an Island doesn't.
    for (land, ok) in [("Snow-Covered Island", true), ("Island", false)] {
        let mut t = main_phase(2);
        let foot = t.battlefield(P0, "Phyrexian Ironfoot");
        t.g.objects[foot.0 as usize].tapped = true;
        crate::r_s06_common::attach_new(&mut t, P0, "Power Artifact", obj(foot));
        t.lands(P0, land, 1);
        let r = activate_containing(&mut t, P0, foot, "Untap");
        assert_eq!(r.is_ok(), ok, "{land}");
        if ok {
            assert_eq!(tapped_lands(&t, P0), 1);
            t.resolve_all();
            assert!(!t.obj(foot).tapped);
        }
    }
}

/// A 60-card deck of `n` Savannah Lions and the given other cards, Plains for the rest.
fn deck(n: usize, others: &[&str]) -> Vec<Arc<CardDef>> {
    let mut d: Vec<Arc<CardDef>> = (0..n).map(|_| card("Savannah Lions")).collect();
    d.extend(others.iter().map(|c| card(c)));
    while d.len() < 60 {
        d.push(card("Plains"));
    }
    d
}

#[test]
fn umori_ignores_the_types_of_land_cards() {
    cr!("702.139a", "103.2b");
    ruling!(
        "Umori, the Collector",
        "Land cards in your deck may have any number of additional types that may be shared with other cards or not."
    );
    supported("Umori, the Collector");
    // "Companion — Each nonland card in your starting deck shares a card type." Creatures,
    // with Dryad Arbor (a land creature) and Seat of the Synod (an artifact land): Umori
    // can be the companion. With a Mind Stone (a nonland artifact) it can't.
    for (others, ok) in [
        (&["Dryad Arbor", "Seat of the Synod", "Seat of the Synod"][..], true),
        (&["Dryad Arbor", "Mind Stone"][..], false),
    ] {
        let mut t = crate::r_s12_common::pregame(
            mtg_engine::game::GameConfig {
                starting_player: Some(P0),
                ..Default::default()
            },
            vec![deck(20, others), deck(20, &[])],
        );
        let side = t.g.add_to_sideboard(P0, vec![card("Umori, the Collector")]);
        t.answer_choose(P0, &[obj(side[0])]);
        t.g.start();
        assert_eq!(t.g.companion_of(P0) == Some(side[0]), ok, "{others:?}");
    }
}

#[test]
fn loading_zone_doubles_counters_a_creature_spacecraft_or_planet_enters_with() {
    cr!("614.1c", "614.5", "616.1", "122.6");
    ruling!(
        "Loading Zone",
        "If a creature, Spacecraft, or Planet you control would enter the battlefield with a number of counters on it, it enters with twice that many instead."
    );
    ruling!(
        "Loading Zone",
        "If you control two Loading Zones, the number of counters put on a creature, Spacecraft, or Planet you control is four times the original number. Three Loading Zones multiplies the original number by eight, and so on."
    );
    supported("Loading Zone");
    // Walking Ballista with X = 1: 1, 2, 4, 8 counters with 0–3 Loading Zones.
    for (zones, want) in [(0, 1), (1, 2), (2, 4), (3, 8)] {
        let mut t = main_phase(2);
        for _ in 0..zones {
            t.battlefield(P0, "Loading Zone");
        }
        let wb = t.hand(P0, "Walking Ballista");
        t.answer(P0, DecisionKind::X, Answer::Number(1));
        let paid = cast_normal(&mut t, P0, wb);
        t.resolve_all();
        let wb = t.g.current(paid.spell);
        assert_eq!(t.counters(wb, "+1/+1"), want, "{zones} Loading Zones");
    }
    // A Spacecraft and a Planet entering with two charge counters: four, eight with two
    // Zones; a plain artifact (Tumble Magnet, three) isn't doubled.
    let spacecraft = crate::r_s01_common::custom_card(
        "Test Spacecraft",
        "Artifact — Spacecraft",
        "{2}",
        None,
        "This artifact enters with two charge counters on it.",
    );
    let planet = crate::r_s01_common::custom_card(
        "Test Planet",
        "Land — Planet",
        "",
        None,
        "This land enters with two charge counters on it.",
    );
    for (zones, want) in [(1, 4), (2, 8)] {
        let mut t = main_phase(2);
        for _ in 0..zones {
            t.battlefield(P0, "Loading Zone");
        }
        for def in [spacecraft.clone(), planet.clone()] {
            let name = def.front().chars.name.clone();
            let c = t.custom(P0, def, Zone::Hand(P0));
            let paid = cast_paying(&mut t, P0, c, CastMethod::Normal);
            let id = match paid {
                Some(p) => {
                    t.resolve_all();
                    t.g.current(p.spell)
                }
                None => {
                    // The Planet is a land: played.
                    t.play_land(P0, c).expect("land play");
                    t.g.flush_events();
                    t.settle();
                    t.g.current(c)
                }
            };
            assert!(t.on_battlefield(id), "{name}");
            assert_eq!(t.counters(id, "charge"), want, "{name}, {zones} Zones");
        }
        let magnet = t.hand(P0, "Tumble Magnet");
        let paid = cast_normal(&mut t, P0, magnet);
        t.resolve_all();
        assert_eq!(t.counters(t.g.current(paid.spell), "charge"), 3);
    }
}

#[test]
fn ferrafor_counts_counters_as_its_ability_resolves() {
    cr!("608.2h", "603.3");
    ruling!(
        "Ferrafor, Young Yew",
        "Use the number of counters among creatures the target player controls when Ferrafor's first ability resolves to determine how many Saprolings are created."
    );
    supported("Ferrafor, Young Yew");
    // "When Ferrafor enters, create a number of 1/1 green Saproling creature tokens equal
    // to the number of counters among creatures target player controls." One counter as
    // it triggers, three as it resolves: three Saprolings.
    let mut t = main_phase(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].counters.insert("+1/+1".into(), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Ferrafor, Young Yew");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.objects[wurm.0 as usize].counters.insert("-1/-1".into(), 1);
    t.g.objects[bears.0 as usize].counters.insert("+1/+1".into(), 2);
    t.g.recompute();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Saproling").len(), 3);
}

#[test]
fn umezawas_jitte_announces_the_target_of_its_minus_mode() {
    cr!("700.2a", "601.2c", "602.2b");
    ruling!(
        "Umezawa's Jitte",
        "If the \"target creature gets -1/-1 until end of turn\" mode is chosen, the target creature must also be announced."
    );
    supported("Umezawa's Jitte");
    let mut t = main_phase(2);
    let jitte = t.battlefield(P0, "Umezawa's Jitte");
    t.g.objects[jitte.0 as usize].counters.insert("charge".into(), 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(bears)]);
    let ab = activate_containing(&mut t, P0, jitte, "Choose one")
        .expect("activated")
        .expect("on the stack");
    // On the stack with its target, before anyone has priority.
    let si = t.obj(ab).stack.as_ref().unwrap();
    assert_eq!(si.chosen.len(), 1);
    assert_eq!(si.chosen[0].mode, Some(1));
    assert_eq!(si.chosen[0].targets, vec![vec![obj(bears)]]);
    assert_eq!(t.counters(jitte, "charge"), 0);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
    // With no creature to target, that mode can't be chosen: the queued mode isn't used.
    let mut t = main_phase(2);
    let jitte = t.battlefield(P0, "Umezawa's Jitte");
    t.g.objects[jitte.0 as usize].counters.insert("charge".into(), 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    if let Ok(Some(ab)) = activate_containing(&mut t, P0, jitte, "Choose one") {
        let si = t.obj(ab).stack.as_ref().unwrap();
        assert!(si.chosen.iter().all(|c| c.mode != Some(1)));
    }
}

#[test]
fn charge_counter_artifacts_stay_once_out_of_counters() {
    cr!("122.1", "704.5");
    ruling!(
        "Surge Node",
        "Once Surge Node has run out of charge counters, it remains on the battlefield."
    );
    ruling!(
        "Tumble Magnet",
        "Once Tumble Magnet has run out of charge counters, it remains on the battlefield."
    );
    supported("Surge Node");
    supported("Tumble Magnet");
    // Surge Node: "{1}, {T}, Remove a charge counter from this artifact: Put a charge
    // counter on target artifact." Tumble Magnet: "{T}, Remove a charge counter from this
    // artifact: Tap target artifact or creature."
    for name in ["Surge Node", "Tumble Magnet"] {
        let mut t = main_phase(2);
        let a = t.battlefield(P0, name);
        t.g.objects[a.0 as usize].counters.insert("charge".into(), 1);
        let stone = t.battlefield(P0, "Mind Stone");
        t.lands(P0, "Wastes", 1);
        t.answer_targets(P0, &[obj(stone)]);
        activate_containing(&mut t, P0, a, "Remove a charge counter").expect("activate");
        t.resolve_all();
        assert_eq!(t.counters(a, "charge"), 0, "{name}");
        assert!(t.on_battlefield(a), "{name}");
    }
}
