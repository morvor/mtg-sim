//! Rulings batch P162 — surveil cards and their neighbors: what stays on top of the
//! library (CR 701.25), illegal targets (CR 608.2b), values counted on resolution, mana
//! value with {X} and cost reductions (CR 202.3), "cast it this turn" permissions, and
//! the abilities granted by Glowcap Lantern.

use crate::r_p076_common::{is_blocked, mana};
use crate::r_p162_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported, triggers_on_stack};
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s05_common::{enter, move_to};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn surveil_asked(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Surveil { .. }))
        .count()
}

#[test]
fn eat_to_extinction_illegal_target_means_no_surveil_and_kept_card_stays_on_top() {
    cr!("608.2b", "701.25a");
    ruling!(
        "Eat to Extinction",
        "If the target creature or planeswalker is an illegal target by the time Eat to Extinction tries to resolve, the spell won't resolve. You won't look at or move the top card of your library."
    );
    ruling!(
        "Eat to Extinction",
        "If you don't put the top card of your library into your graveyard, it remains on top of your library."
    );
    supported("Eat to Extinction");
    // Target gone: no surveil.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s01_common::give_mana_for(&mut t, P0, "Eat to Extinction");
    let c = t.hand(P0, "Eat to Extinction");
    t.cast(P0, c).target(bears).go();
    destroy(&mut t, bears);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(surveil_asked(&t, from), 0);
    assert!(t.in_graveyard(P0, "Eat to Extinction"));
    // Kept on top.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let top = t.library_top(P0, "Hill Giant");
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![top], vec![]));
    cast_resolve(&mut t, P0, "Eat to Extinction", &[Entity::Object(bears)]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.g.library_top(P0), Some(top));
}

#[test]
fn celestus_sanctifier_the_other_card_stays_on_top() {
    cr!("701.25a", "731.1");
    ruling!(
        "Celestus Sanctifier",
        "The card you don't put into your graveyard stays on top of your library."
    );
    supported("Celestus Sanctifier");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestus Sanctifier");
    t.g.set_day(true);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Grizzly Bears"]);
    t.answer_choose(P0, &[Entity::Object(cards[1])]);
    t.g.set_day(false);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "graveyard"), 1);
    t.resolve_all();
    assert_eq!(t.zone(cards[1]), Zone::Graveyard(P0));
    assert_eq!(t.g.library_top(P0), Some(cards[0]));
}

#[test]
fn gutless_plunderer_with_fewer_than_three_cards() {
    cr!("609.3");
    ruling!(
        "Gutless Plunderer",
        "If there are fewer than three cards remaining in your library when Gutless Plunderer's last ability resolves, look at all of the cards in your library and follow the instructions in order."
    );
    supported("Gutless Plunderer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    crate::r_s11_common::empty_library(&mut t, P0);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Craw Wurm"]);
    t.answer_choose(P0, &[Entity::Object(cards[1])]);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Gutless Plunderer", &[]);
    assert!(chosen_from(&t, from).iter().any(|c| c.len() == 2));
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(t.g.library_top(P0), Some(cards[1]));
    assert_eq!(t.zone(cards[0]), Zone::Graveyard(P0));
}

#[test]
fn mission_briefing_permission_and_exile() {
    cr!("400.7", "601.3", "307.1");
    ruling!(
        "Mission Briefing",
        "If you cast a spell this way and that card is exiled, it's considered a new object in the zone it's put into. It won't be exiled if it's put into your graveyard later in the turn."
    );
    ruling!(
        "Mission Briefing",
        "Mission Briefing doesn't change when you can cast the chosen card. For example, if you choose a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    ruling!(
        "Mission Briefing",
        "Mission Briefing is still on the stack while you choose an instant or sorcery card in your graveyard. Your Mission Briefing can't be to give yourself that same Mission Briefing."
    );
    supported("Mission Briefing");
    // A sorcery chosen: only at sorcery speed; cast, it's exiled; put back into the
    // graveyard later, it stays there.
    let mut t = TestGame::new(2);
    let ritual = t.graveyard(P0, "Divination");
    t.answer_choose(P0, &[Entity::Object(ritual)]);
    t.set_step(P0, Step::BeginningOfCombat);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Mission Briefing", &[]);
    // The Mission Briefing being cast wasn't offered.
    for c in chosen_from(&t, from) {
        assert!(c.contains(&Entity::Object(ritual)));
        assert_eq!(c.len(), 1);
    }
    let mb = t.g.find_in_zone(Zone::Graveyard(P0), "Mission Briefing")[0];
    assert!(!can_cast(&mut t, P0, mb, CastMethod::Normal));
    mana(&mut t, P0, ManaType::U, 3);
    assert!(!can_cast(&mut t, P0, ritual, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, opt).go();
    assert!(!can_cast(&mut t, P0, ritual, CastMethod::Normal));
    t.resolve_all();
    mana(&mut t, P0, ManaType::U, 3);
    assert!(can_cast(&mut t, P0, ritual, CastMethod::Normal));
    t.cast(P0, ritual).go();
    t.resolve_all();
    assert_eq!(t.zone(ritual), Zone::Exile);
    move_to(&mut t, ritual, Zone::Graveyard(P0));
    assert_eq!(t.zone(ritual), Zone::Graveyard(P0));
}

#[test]
fn consuming_ashes_x_in_the_cost_is_zero_on_the_battlefield() {
    cr!("202.3e", "107.3f");
    ruling!(
        "Consuming Ashes",
        "If the target creature has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Consuming Ashes");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Hangarback Walker");
    t.g.objects[walker.0 as usize]
        .counters
        .insert("+1/+1".into(), 5);
    t.g.recompute();
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Consuming Ashes", &[Entity::Object(walker)]);
    assert_eq!(t.zone(walker), Zone::Exile);
    assert_eq!(surveil_asked(&t, from), 1);
}

#[test]
fn price_of_fame_mana_value_ignores_the_reduction() {
    cr!("202.3", "601.2f");
    ruling!(
        "Price of Fame",
        "Price of Fame's mana value is 4, even if it costs {2} less to cast."
    );
    supported("Price of Fame");
    let mut t = TestGame::new(2);
    let legend = t.battlefield(P1, "Isamaru, Hound of Konda");
    mana(&mut t, P0, ManaType::B, 2);
    let c = t.hand(P0, "Price of Fame");
    let spell = t.cast(P0, c).target(legend).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(crate::r_s08_common::mana_value(&t, spell), 4);
}

#[test]
fn mephitic_vapors_affects_only_creatures_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Mephitic Vapors",
        "Mephitic Vapors affects only creatures on the battlefield at the time it resolves. Creatures that enter the battlefield later in the turn won't get -1/-1."
    );
    supported("Mephitic Vapors");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Mephitic Vapors", &[]);
    assert_eq!(t.pt(giant), (2, 2));
    let later = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn cephalid_inkmage_stays_blocked() {
    cr!("509.1h", "611.3a");
    ruling!(
        "Cephalid Inkmage",
        "Once Cephalid Inkmage has been blocked, reaching a total of seven or more cards in your graveyard won't cause it to become unblocked."
    );
    supported("Cephalid Inkmage");
    let mut t = TestGame::new(2);
    let ink = t.battlefield(P0, "Cephalid Inkmage");
    let wall = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(ink, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, ink)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(is_blocked(&t, ink));
    for _ in 0..7 {
        t.graveyard(P0, "Forest");
    }
    t.g.recompute();
    assert!(is_blocked(&t, ink));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn lodestone_needle_can_target_a_tapped_permanent() {
    cr!("701.26a", "609.3");
    ruling!(
        "Lodestone Needle // Guidestone Compass",
        "Lodestone Needle's enters-the-battlefield ability can target an artifact or creature that's already tapped. If so, you'll just put two stun counters on that artifact or creature."
    );
    supported("Lodestone Needle // Guidestone Compass");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    t.answer_targets(P0, &[Entity::Object(bears)]);
    enter(&mut t, P0, "Lodestone Needle // Guidestone Compass");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, "stun"), 2);
}

#[test]
fn owlin_historian_triggers_once_for_cards_leaving_together() {
    cr!("603.2c");
    ruling!(
        "Owlin Historian",
        "If multiple cards leave your graveyard at the same time, Owlin Historian's last ability will trigger only once."
    );
    supported("Owlin Historian");
    supported("Bojuka Bog");
    let mut t = TestGame::new(2);
    let owl = t.battlefield(P0, "Owlin Historian");
    for _ in 0..3 {
        t.graveyard(P0, "Forest");
    }
    t.answer_targets(P0, &[Entity::Player(P0)]);
    enter(&mut t, P0, "Bojuka Bog");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.pt(owl), (3, 4));
}

#[test]
fn cerebral_download_counts_artifacts_on_resolution() {
    cr!("608.2h", "701.25a");
    ruling!(
        "Cerebral Download",
        "The value of X is calculated only once, as Cerebral Download resolves."
    );
    supported("Cerebral Download");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    crate::r_s01_common::give_mana_for(&mut t, P0, "Cerebral Download");
    let c = t.hand(P0, "Cerebral Download");
    t.cast(P0, c).go();
    // An artifact arrives before it resolves: surveil 2.
    t.battlefield(P0, "Ornithopter");
    let from = t.asked().len();
    t.resolve_all();
    let seen: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Surveil { cards } => Some(cards.len()),
            _ => None,
        })
        .collect();
    assert_eq!(seen, vec![2]);
}

#[test]
fn sarinth_steelseeker_a_land_may_go_to_the_graveyard_unrevealed() {
    cr!("701.20a");
    ruling!(
        "Sarinth Steelseeker",
        "You don't have to reveal the card if it's a land card. You may choose not to reveal it and instead put it into your graveyard."
    );
    supported("Sarinth Steelseeker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sarinth Steelseeker");
    let top = t.library_top(P0, "Forest");
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Graveyard(P0));
}

#[test]
fn dogged_detective_only_needs_to_be_in_the_graveyard_for_the_second_draw() {
    cr!("603.2", "113.6");
    ruling!(
        "Dogged Detective",
        "Dogged Detective doesn't need to be in the graveyard when the first card is drawn for its last ability to trigger. As long as it is in the graveyard when an opponent draws their second card in a turn, it will trigger."
    );
    supported("Dogged Detective");
    let mut t = TestGame::new(2);
    t.g.draw_cards(P1, 1);
    t.g.flush_events();
    t.settle();
    let dd = t.graveyard(P0, "Dogged Detective");
    t.answer_yes(P0, true);
    t.g.draw_cards(P1, 1);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(dd), Zone::Hand(P0));
}

#[test]
fn glowcap_lantern_needs_to_be_on_a_creature_you_control() {
    cr!("401.5", "604.1");
    ruling!(
        "Glowcap Lantern",
        "If Glowcap Lantern isn't attached to a creature you control, you can't look at the top card of your library. The lantern isn't that bright."
    );
    ruling!(
        "Glowcap Lantern",
        "The ability granted by Glowcap Lantern lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack."
    );
    supported("Glowcap Lantern");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    let lantern = t.battlefield(P0, "Glowcap Lantern");
    t.g.recompute();
    assert!(!can_look_at(&t.g, P0, top));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(lantern, Entity::Object(bears));
    t.g.recompute();
    // On P1's turn, with P1 holding priority, and nothing on the stack.
    t.set_step(P1, Step::DeclareAttackers);
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn esper_maduins_mana_chapter_uses_the_stack() {
    cr!("714.2b", "605.5a");
    ruling!(
        "Esper Origins // Summon: Esper Maduin",
        "Summon: Esper Maduin's second chapter ability isn't a mana ability. It uses the stack and can be responded to."
    );
    supported("Esper Origins // Summon: Esper Maduin");
    let mut t = TestGame::new(2);
    let saga = crate::r_s17_common::enter_transformed(&mut t, P0, "Esper Origins // Summon: Esper Maduin");
    t.resolve_all();
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    assert_eq!(crate::r_s19_common::chapters_on_stack(&t, saga), vec![2]);
    t.resolve();
    assert_eq!(crate::r_s24_common::pool(&t, P0, ManaType::G), 2);
}

#[test]
fn gf_cerberus_copies_the_next_spell_even_without_targets() {
    cr!("707.10", "603.7c");
    ruling!(
        "Summon: G.F. Cerberus",
        "After Summon: G.F. Cerberus's second or third chapter ability resolves, the next instant or sorcery spell you cast that turn will be copied whether or not it has targets."
    );
    supported("Summon: G.F. Cerberus");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Summon: G.F. Cerberus");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    let hand = t.hand_size(P0);
    mana(&mut t, P0, ManaType::U, 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    // Opt and its copy each drew a card.
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn ponder_and_natural_selection() {
    cr!("701.24a", "115.1a");
    ruling!(
        "Ponder",
        "If you choose to shuffle your library, that includes the three cards you just looked at and put back on top of it."
    );
    ruling!("Natural Selection", "This is now targeted.");
    supported("Ponder");
    supported("Natural Selection");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Grizzly Bears", "Craw Wurm"]);
    let size = t.library_size(P0);
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Ponder", &[]);
    assert_eq!(t.library_size(P0), size - 1);
    let in_lib = cards
        .iter()
        .filter(|c| t.zone(**c) == Zone::Library(P0))
        .count();
    let in_hand = cards
        .iter()
        .filter(|c| t.zone(**c) == Zone::Hand(P0))
        .count();
    assert_eq!(in_lib + in_hand, 3);
    // The library was shuffled, with those cards in it.
    let shuffles = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, mtg_engine::events::Event::Shuffled { player } if *player == P0))
            .count()
    };
    assert_eq!(shuffles(&t), 1);
    // Not shuffling: the top card (as put back) is drawn.
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Grizzly Bears", "Craw Wurm"]);
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Ponder", &[]);
    assert_eq!(shuffles(&t), 0);
    assert_eq!(cards.iter().filter(|c| t.zone(**c) == Zone::Hand(P0)).count(), 1);
    // Natural Selection targets a player: it can't target one with hexproof.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Sanctity");
    t.g.recompute();
    crate::r_s01_common::give_mana_for(&mut t, P0, "Natural Selection");
    let c = t.hand(P0, "Natural Selection");
    let from = t.asked().len();
    t.cast(P0, c).target(Entity::Player(P0)).go();
    for cands in crate::r_s02_common::target_candidates(&t, P0, from) {
        assert!(!cands.contains(&Entity::Player(P1)));
        assert!(cands.contains(&Entity::Player(P0)));
    }
}
