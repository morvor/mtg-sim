//! Rulings batch P181 — searching for basic lands: values fixed as an ability resolves
//! (Celebrate the Harvest, Edge of Autumn, Kyoshi Island Plaza, Boundless Realms,
//! Prismatic Undercurrents, Dance of the Tumbleweeds, The Earth King), last known
//! information (Rampant Rejuvenator), trigger ordering (Yavimaya Elder, Horizon
//! Spellbomb, Vastwood Surge, Mycosynth Wellspring), land plays, Lander tokens, and
//! "up to" searches.

use crate::r_p181_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts `n` basic lands named `name` on top of `p`'s library (replacing it). Returns them
/// as answers to a card choice.
fn basics(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<Entity> {
    set_library(t, p, &vec![name; n])
        .into_iter()
        .map(obj)
        .collect()
}

/// The maximum of the last card choice `p` was asked since `from`.
fn last_max(t: &TestGame, p: PlayerId, from: usize) -> u32 {
    t.asked()[from..]
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { max, .. } if *q == p => Some(*max),
            _ => None,
        })
        .expect("no card choice")
}

/// The index of the first event matching `f` this turn.
fn first(t: &TestGame, f: impl Fn(&Event) -> bool) -> usize {
    t.g.turn_events.iter().position(f).expect("no such event")
}

#[test]
fn values_are_counted_as_the_spell_or_ability_resolves() {
    cr!("608.2h", "107.3c");
    ruling!(
        "Celebrate the Harvest",
        "Celebrate the Harvest looks at the creatures you control as it resolves to determine how many basic lands cards you can search for."
    );
    ruling!(
        "Boundless Realms",
        "The number of basic land cards you put onto the battlefield is equal to the number of lands you control when Boundless Realms begins resolving."
    );
    ruling!(
        "Prismatic Undercurrents",
        "The value of X is calculated only once, as Prismatic Undercurrents's first ability resolves."
    );
    for c in [
        "Celebrate the Harvest",
        "Boundless Realms",
        "Prismatic Undercurrents",
    ] {
        supported(c);
    }
    // Celebrate the Harvest: a 1/1 and a 2/2 are two powers; the 1/1 pumped to 2/1 in
    // response leaves one.
    for pumped in [false, true] {
        let mut t = TestGame::new(2);
        let forests = basics(&mut t, P0, "Forest", 3);
        let elves = t.battlefield(P0, "Llanowar Elves");
        t.battlefield(P0, "Grizzly Bears");
        lots(&mut t, P0);
        t.answer_choose(P0, &forests);
        let c = t.hand(P0, "Celebrate the Harvest");
        t.cast(P0, c).go();
        if pumped {
            pump(&mut t, elves, 1, 0);
        }
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(last_max(&t, P0, from), if pumped { 1 } else { 2 });
    }
    // Boundless Realms: three lands when it's cast, a fourth (a Wastes) before it resolves:
    // four lands as it begins resolving; the ones it puts onto the battlefield don't
    // raise X.
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 8);
    t.lands(P0, "Forest", 3);
    lots(&mut t, P0);
    t.answer_choose(P0, &forests[..4]);
    let realms = t.hand(P0, "Boundless Realms");
    t.cast(P0, realms).go();
    t.battlefield(P0, "Wastes");
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(last_max(&t, P0, from), 4);
    assert_eq!(controlled(&t, P0, "Forest").len(), 3 + 4);
    // Prismatic Undercurrents: green (itself) and red (Goblin Guide) as it triggers, and
    // blue (Delver of Secrets) by the time it resolves: up to three cards.
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 4);
    t.battlefield(P0, "Goblin Guide");
    t.answer_choose(P0, &forests[..3]);
    let from = t.asked().len();
    t.enter(P0, "Prismatic Undercurrents");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Delver of Secrets // Insectile Aberration");
    t.resolve_all();
    assert_eq!(last_max(&t, P0, from), 3);
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn edge_of_autumn_checks_lands_as_it_resolves() {
    cr!("608.2c");
    ruling!(
        "Edge of Autumn",
        "Edge of Autumn checks how many lands you control as it resolves. If you control five or more lands, it simply does nothing as it resolves."
    );
    supported("Edge of Autumn");
    for fifth in [false, true] {
        let mut t = TestGame::new(2);
        let forests = basics(&mut t, P0, "Forest", 2);
        t.lands(P0, "Wastes", 4);
        lots(&mut t, P0);
        t.answer_choose(P0, &forests);
        let edge = t.hand(P0, "Edge of Autumn");
        t.cast(P0, edge).go();
        if fifth {
            t.battlefield(P0, "Wastes");
        }
        t.resolve_all();
        assert_eq!(controlled(&t, P0, "Forest").len(), !fifth as usize);
        assert_eq!(searches(&t, P0), !fifth as usize);
    }
}

#[test]
fn kyoshi_island_plaza_with_no_shrines_still_searches() {
    cr!("608.2h", "701.23a");
    ruling!(
        "Kyoshi Island Plaza",
        "For Kyoshi Island Plaza's first ability, the number of Shrines you control is checked when the ability resolves. You will still search and shuffle your library if you control no Shrines when the ability resolves, but you won't get any lands."
    );
    supported("Kyoshi Island Plaza");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 2);
    t.answer_choose(P0, &forests);
    let plaza = t.enter(P0, "Kyoshi Island Plaza");
    t.settle();
    destroy(&mut t, plaza);
    t.resolve_all();
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    assert!(controlled(&t, P0, "Forest").is_empty());
}

#[test]
fn dance_of_the_tumbleweeds_x_is_fixed_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Dance of the Tumbleweeds",
        "The value of X is determined only once, as Dance of the Tumbleweeds resolves. The token’s power and toughness won’t change if the number of lands you control later changes."
    );
    supported("Dance of the Tumbleweeds");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    lots(&mut t, P0);
    let dance = t.hand(P0, "Dance of the Tumbleweeds");
    t.cast(P0, dance).modes(&[1]).go();
    t.resolve_all();
    let token = crate::r_s01_common::with_subtype(&t, P0, "Elemental")[0];
    assert_eq!(t.pt(token), (3, 3));
    t.lands(P0, "Forest", 2);
    assert_eq!(t.pt(token), (3, 3));
}

#[test]
fn the_earth_king_counts_the_attackers_as_it_triggers() {
    cr!("603.2", "608.2h");
    ruling!(
        "The Earth King",
        "The Earth King's second ability counts the number of creatures with power 4 or greater you attack with when the ability triggers."
    );
    supported("The Earth King");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 3);
    t.battlefield(P0, "The Earth King");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Hill Giant");
    pump(&mut t, a, 1, 0);
    pump(&mut t, b, 1, 0);
    t.answer_choose(P0, &forests);
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    // One leaves combat and the other's power drops before it resolves: still two.
    destroy(&mut t, a);
    pump(&mut t, b, -2, 0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(last_max(&t, P0, from), 2);
    assert_eq!(controlled(&t, P0, "Forest").len(), 2);
}

#[test]
fn rampant_rejuvenator_uses_its_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Rampant Rejuvenator",
        "Use Rampant Rejuvenator's power as it last existed on the battlefield to determine the value of X for its last ability."
    );
    supported("Rampant Rejuvenator");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 5);
    let r = t.enter(P0, "Rampant Rejuvenator");
    t.resolve_all();
    pump(&mut t, r, 2, 0);
    assert_eq!(t.pt(r), (4, 2));
    t.answer_choose(P0, &forests);
    let from = t.asked().len();
    destroy(&mut t, r);
    t.resolve_all();
    assert_eq!(last_max(&t, P0, from), 4);
    assert_eq!(controlled(&t, P0, "Forest").len(), 4);
}

#[test]
fn yavimaya_elder_searches_before_drawing() {
    cr!("603.3", "608.2");
    ruling!(
        "Yavimaya Elder",
        "If you activate Yavimaya Elder's last ability, its triggered ability will trigger and go on the stack. It will resolve first, meaning you'll do the search and shuffle before drawing a card."
    );
    supported("Yavimaya Elder");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 3);
    let elder = t.battlefield(P0, "Yavimaya Elder");
    lots(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &forests[..2]);
    activate_containing(&mut t, P0, elder, "Draw").unwrap();
    t.resolve_all();
    let searched = first(&t, |e| matches!(e, Event::Searched { .. }));
    let drew = first(&t, |e| matches!(e, Event::Drew { .. }));
    assert!(searched < drew);
    assert_eq!(t.hand_size(P0), 3, "two Forests and the drawn card");
}

#[test]
fn horizon_spellbomb_draws_before_searching() {
    cr!("603.3", "405.5");
    ruling!(
        "Horizon Spellbomb",
        "When you sacrifice Horizon Spellbomb, it’s search ability goes on the stack, and then the triggered draw ability goes on the stack on top of it. This means you first choose whether to pay {G} and draw a card, then you search; you do not search first."
    );
    supported("Horizon Spellbomb");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 3);
    let bomb = t.battlefield(P0, "Horizon Spellbomb");
    lots(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &forests[..1]);
    activate_containing(&mut t, P0, bomb, "Search").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let searched = first(&t, |e| matches!(e, Event::Searched { .. }));
    let drew = first(&t, |e| matches!(e, Event::Drew { .. }));
    assert!(drew < searched);
}

#[test]
fn vastwood_surge_puts_counters_before_landfall_triggers_resolve() {
    cr!("603.3", "608.2");
    ruling!(
        "Vastwood Surge",
        "If any abilities trigger when the lands are put onto the battlefield and Vastwood Surge was kicked, those abilities won't resolve until after you've put two +1/+1 counters on creatures you control."
    );
    supported("Vastwood Surge");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 2);
    // Scute Swarm: "Landfall — ... create a 1/1 green Insect creature token."
    let swarm = t.battlefield(P0, "Scute Swarm");
    lots(&mut t, P0);
    t.answer_choose(P0, &forests);
    let surge = t.hand(P0, "Vastwood Surge");
    t.cast(P0, surge).kicked(true).go();
    t.resolve();
    // The counters are on; the two landfall triggers wait.
    assert_eq!(plus1(&t, swarm), 2);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let insects = crate::r_s01_common::with_subtype(&t, P0, "Insect");
    assert_eq!(insects.len(), 3, "Scute Swarm and two tokens");
    assert!(
        insects
            .iter()
            .filter(|i| **i != swarm)
            .all(|i| plus1(&t, *i) == 0),
        "the tokens came after the counters"
    );
}

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(t.g.current(id), counters::PLUS1)
}

#[test]
fn mycosynth_wellspring_triggers_on_leaving_before_its_enters_trigger_resolves() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Mycosynth Wellspring",
        "The ability will trigger when Mycosynth Wellspring is put into a graveyard from the battlefield, even if the ability that triggered when it entered hasn't resolved yet."
    );
    supported("Mycosynth Wellspring");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 3);
    let w = t.enter(P0, "Mycosynth Wellspring");
    t.settle();
    destroy(&mut t, w);
    assert_eq!(triggered_from(&t, w), 2);
    for f in &forests[..2] {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[*f]);
    }
    t.resolve_all();
    assert_eq!(searches(&t, P0), 2);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn heaped_harvest_triggers_when_sacrificed_for_any_reason() {
    cr!("701.21a", "603.6c");
    ruling!(
        "Heaped Harvest",
        "Heaped Harvest's first ability will trigger if it's sacrificed for any reason, not only if it's sacrificed to its last ability."
    );
    supported("Heaped Harvest");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 2);
    let harvest = t.battlefield(P0, "Heaped Harvest");
    // Krark-Clan Ironworks: "Sacrifice an artifact: Add {C}{C}."
    let kci = t.battlefield(P0, "Krark-Clan Ironworks");
    t.answer_choose(P0, &[obj(harvest)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &forests[..1]);
    activate_containing(&mut t, P0, kci, "Sacrifice").unwrap();
    t.resolve_all();
    assert_eq!(triggered_from(&t, harvest), 1);
    assert_eq!(controlled(&t, P0, "Forest").len(), 1);
}

#[test]
fn scout_the_wilderness_kicked_makes_soldiers_without_a_land() {
    cr!("702.33d", "608.2c");
    ruling!(
        "Scout the Wilderness",
        "If Scout the Wilderness was kicked, you will put two 1/1 white Soldier tokens onto the battlefield whether you found a land card while searching your library or not."
    );
    supported("Scout the Wilderness");
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Grizzly Bears"]);
    lots(&mut t, P0);
    let scout = t.hand(P0, "Scout the Wilderness");
    t.cast(P0, scout).kicked(true).go();
    t.resolve_all();
    assert_eq!(
        crate::r_s01_common::with_subtype(&t, P0, "Soldier").len(),
        2
    );
}

#[test]
fn kav_landseeker_sacrifices_every_lander_its_ability_made() {
    cr!("614.1a", "603.7a");
    ruling!(
        "Kav Landseeker",
        "If a replacement effect (such as that of Exalted Sunborn’s second ability) causes additional tokens to be created during the resolution of Kav Landseeker’s last ability, you’ll sacrifice those tokens at the beginning of the end step on your next turn as well."
    );
    supported("Kav Landseeker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exalted Sunborn");
    t.enter(P0, "Kav Landseeker");
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Lander").len(), 2);
    // Not this turn's end step.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Lander").len(), 2);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Lander").len(), 0);
}

#[test]
fn lander_tokens_are_predefined() {
    cr!("111.10u");
    ruling!(
        "Beamsaw Prospector",
        "Lander tokens are a kind of predefined token. Each one is a colorless artifact with the artifact subtype Lander and the ability “{2}, {T}, Sacrifice this token: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.”"
    );
    supported("Beamsaw Prospector");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 2);
    let p = t.battlefield(P0, "Beamsaw Prospector");
    destroy(&mut t, p);
    t.resolve_all();
    let landers = crate::r_s01_common::with_subtype(&t, P0, "Lander");
    assert_eq!(landers.len(), 1);
    let l = t.obj_now(landers[0]);
    assert_eq!(l.chars.colors, mtg_engine::types::ColorSet::NONE);
    assert!(l.is(mtg_engine::types::CardType::Artifact));
    assert!(!l.is_creature() && l.chars.has_subtype("Lander"));
    lots(&mut t, P0);
    t.answer_choose(P0, &forests[..1]);
    activate_containing(&mut t, P0, landers[0], "Search").unwrap();
    t.resolve_all();
    let found = controlled(&t, P0, "Forest");
    assert_eq!(found.len(), 1);
    assert!(t.obj_now(found[0]).tapped);
    assert!(!t.on_battlefield(landers[0]));
}

#[test]
fn fabled_passage_enters_tapped_then_untaps() {
    cr!("608.2c", "701.26b");
    ruling!(
        "Fabled Passage",
        "If you control four or more lands, the basic land doesn't enter untapped; it enters tapped and then you untap it."
    );
    supported("Fabled Passage");
    for others in [2usize, 3, 4] {
        let mut t = TestGame::new(2);
        let forests = basics(&mut t, P0, "Forest", 2);
        t.lands(P0, "Wastes", others);
        let passage = t.battlefield(P0, "Fabled Passage");
        t.answer_choose(P0, &forests[..1]);
        activate_containing(&mut t, P0, passage, "Search").unwrap();
        t.resolve_all();
        let f = controlled(&t, P0, "Forest")[0];
        // It entered tapped. With four or more lands (counting it, not the sacrificed
        // Passage) it was then untapped, which only a tapped permanent can be (CR 701.26b).
        let untapped =
            t.g.turn_events
                .iter()
                .any(|e| matches!(e, Event::Untapped { obj } if *obj == f));
        assert_eq!(untapped, others >= 3, "{others} other lands");
        assert_eq!(t.obj_now(f).tapped, others < 3);
    }
}

#[test]
fn lands_put_onto_the_battlefield_dont_use_the_land_play() {
    cr!("305.4", "305.2");
    ruling!(
        "Quirion Trailblazer",
        "The land does not count toward your normal limit of playing one land per turn."
    );
    ruling!(
        "Thawing Glaciers",
        "The land put onto the battlefield does not count toward your one per turn limit because it was put onto the battlefield by an effect."
    );
    ruling!(
        "Harrow",
        "The two lands do not count toward your normal one land you can play each turn."
    );
    for c in ["Quirion Trailblazer", "Thawing Glaciers", "Harrow"] {
        supported(c);
        let mut t = TestGame::new(2);
        let forests = basics(&mut t, P0, "Forest", 3);
        lots(&mut t, P0);
        let sac = t.battlefield(P0, "Wastes");
        match c {
            "Quirion Trailblazer" => {
                t.answer_yes(P0, true);
                t.answer_choose(P0, &forests[..1]);
                t.enter(P0, c);
            }
            "Thawing Glaciers" => {
                let g = t.battlefield(P0, c);
                t.answer_choose(P0, &forests[..1]);
                activate_containing(&mut t, P0, g, "Search").unwrap();
            }
            _ => {
                t.answer_choose(P0, &[obj(sac)]);
                t.answer_choose(P0, &forests[..2]);
                cast_pool(&mut t, P0, c, &[]);
            }
        }
        t.resolve_all();
        assert!(!controlled(&t, P0, "Forest").is_empty(), "{c}");
        let land = t.hand(P0, "Plains");
        assert!(crate::r_s02_common::can_play_land(&mut t, P0, land), "{c}");
    }
}

#[test]
fn harrow_sacrifices_one_land_once() {
    cr!("601.2b", "601.2h");
    ruling!(
        "Harrow",
        "The sacrifice of a land is part of the cost of casting Harrow. You can't pay this cost more than once to get a multiple effect."
    );
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 4);
    let a = t.battlefield(P0, "Wastes");
    let b = t.battlefield(P0, "Wastes");
    lots(&mut t, P0);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_choose(P0, &forests);
    let from = t.asked().len();
    cast_pool(&mut t, P0, "Harrow", &[]);
    assert_eq!(last_max(&t, P0, from), 2);
    assert_eq!(controlled(&t, P0, "Forest").len(), 2);
    assert_eq!(controlled(&t, P0, "Wastes").len(), 1, "one land sacrificed");
}

#[test]
fn springbloom_druid_sacrifices_only_one_land() {
    cr!("608.2c");
    ruling!(
        "Springbloom Druid",
        "While resolving Springbloom Druid's triggered ability, you can't sacrifice multiple lands to search for more than two basic land cards."
    );
    supported("Springbloom Druid");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 4);
    let a = t.battlefield(P0, "Wastes");
    let b = t.battlefield(P0, "Wastes");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_choose(P0, &forests);
    let from = t.asked().len();
    t.enter(P0, "Springbloom Druid");
    t.resolve_all();
    assert_eq!(last_max(&t, P0, from), 2);
    assert_eq!(controlled(&t, P0, "Forest").len(), 2);
    assert_eq!(controlled(&t, P0, "Wastes").len(), 1);
}

#[test]
fn up_to_searches_can_find_fewer() {
    cr!("701.23b");
    ruling!(
        "Rites of Spring",
        "You can choose to find fewer basic land cards if you want to."
    );
    ruling!(
        "Peregrination",
        "You can choose to search for only one basic land card with Peregrination. If you do, you’ll put that card onto the battlefield tapped."
    );
    ruling!(
        "Viewpoint Synchronization",
        "If you find two or fewer basic land cards, you’ll put them onto the battlefield tapped. You won’t be able to put any of them into your hand, even if you want to."
    );
    for c in [
        "Rites of Spring",
        "Peregrination",
        "Viewpoint Synchronization",
    ] {
        supported(c);
    }
    // Rites of Spring: two cards discarded, one land found.
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 3);
    let d1 = t.hand(P0, "Grizzly Bears");
    let d2 = t.hand(P0, "Hill Giant");
    lots(&mut t, P0);
    // "Discard any number of cards": how many, then which.
    t.answer_choose(P0, &[obj(d1), obj(d2)]);
    t.answer_choose(P0, &[obj(d1), obj(d2)]);
    t.answer_choose(P0, &forests[..1]);
    cast_pool(&mut t, P0, "Rites of Spring", &[]);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Forest"));
    // Peregrination: one card goes onto the battlefield tapped.
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 3);
    lots(&mut t, P0);
    t.answer_choose(P0, &forests[..1]);
    cast_pool(&mut t, P0, "Peregrination", &[]);
    let f = controlled(&t, P0, "Forest");
    assert_eq!(f.len(), 1);
    assert!(t.obj_now(f[0]).tapped);
    assert!(!t.in_hand(P0, "Forest"));
    // Viewpoint Synchronization: one or two cards all go onto the battlefield tapped.
    for n in [1usize, 2] {
        let mut t = TestGame::new(2);
        let forests = basics(&mut t, P0, "Forest", 4);
        lots(&mut t, P0);
        t.answer_choose(P0, &forests[..n]);
        // Trying to put one into the hand.
        t.answer_choose(P0, &forests[..1]);
        cast_pool(&mut t, P0, "Viewpoint Synchronization", &[]);
        let f = controlled(&t, P0, "Forest");
        assert_eq!(f.len(), n);
        assert!(f.iter().all(|x| t.obj_now(*x).tapped));
        assert!(!t.in_hand(P0, "Forest"));
    }
}

#[test]
fn snow_covered_basics_are_basic_land_cards() {
    cr!("205.4a", "205.4c");
    ruling!("Untamed Wilds", "Can fetch a snow land.");
    supported("Untamed Wilds");
    let mut t = TestGame::new(2);
    let snow = basics(&mut t, P0, "Snow-Covered Forest", 1);
    lots(&mut t, P0);
    t.answer_choose(P0, &snow);
    cast_pool(&mut t, P0, "Untamed Wilds", &[]);
    assert_eq!(controlled(&t, P0, "Snow-Covered Forest").len(), 1);
}

#[test]
fn land_tax_has_an_intervening_if_clause() {
    cr!("603.4");
    ruling!(
        "Land Tax",
        "This ability has an \"intervening 'if' clause.\" That means (1) the ability won't trigger at all unless any one of your opponents controls more lands than you, and (2) the ability will do nothing if you control at least as many lands as each of your opponents by the time it resolves."
    );
    ruling!(
        "Land Tax",
        "You can fetch any of the \"Snow-Covered\" lands, since they all also have the supertype basic."
    );
    supported("Land Tax");
    // Equal lands: no trigger.
    let mut t = TestGame::new(2);
    let tax = t.battlefield(P0, "Land Tax");
    t.lands(P1, "Plains", 1);
    t.lands(P0, "Wastes", 1);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggered_from(&t, tax), 0);
    // More: it triggers; finding snow-covered basics.
    let mut t = TestGame::new(2);
    let tax = t.battlefield(P0, "Land Tax");
    t.lands(P1, "Plains", 2);
    let snow = basics(&mut t, P0, "Snow-Covered Forest", 3);
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &snow);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(triggered_from(&t, tax), 1);
    assert_eq!(t.hand_size(P0), 3);
    // It triggered, but P0 has as many lands as it resolves: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Land Tax");
    t.lands(P1, "Plains", 1);
    let snow = basics(&mut t, P0, "Snow-Covered Forest", 3);
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &snow);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Wastes", 1);
    t.resolve_all();
    assert_eq!(searches(&t, P0), 0);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn elfhame_sanctuary_skips_the_draw_even_if_nothing_is_found() {
    cr!("614.10");
    ruling!(
        "Elfhame Sanctuary",
        "The “if you do” means “if you search your library”. You get to skip your draw step and shuffle your deck even if you don’t find (or choose not to find) a basic land."
    );
    supported("Elfhame Sanctuary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elfhame Sanctuary");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(draws(&t, P0), 0, "the draw step was skipped");
}

#[test]
fn emergent_sequence_land_isnt_a_creature_as_it_enters_and_counts_itself() {
    cr!("603.6a", "613.1d");
    ruling!(
        "Emergent Sequence",
        "The land won’t be a creature as it enters the battlefield unless some other effect is making it one. Abilities that trigger whenever a creature enters the battlefield won’t trigger."
    );
    ruling!(
        "Emergent Sequence",
        "The last sentence of Emergent Sequence’s ability counts the land it put onto the battlefield."
    );
    supported("Emergent Sequence");
    let mut t = TestGame::new(2);
    let forests = basics(&mut t, P0, "Forest", 2);
    // "Whenever another creature enters, you gain 1 life."
    let warden = t.battlefield(P0, "Soul Warden");
    // A land played earlier this turn.
    let played = t.hand(P0, "Wastes");
    t.play_land(P0, played).unwrap();
    lots(&mut t, P0);
    t.answer_choose(P0, &forests[..1]);
    cast_pool(&mut t, P0, "Emergent Sequence", &[]);
    assert_eq!(triggered_from(&t, warden), 0);
    let f = controlled(&t, P0, "Forest")[0];
    assert!(t.obj_now(f).is_creature());
    assert_eq!(plus1(&t, f), 2, "the played Wastes and itself");
    assert_eq!(t.pt(f), (2, 2));
}

#[test]
fn ancient_excavation_counts_the_hand_on_resolution_and_discards_per_card_drawn() {
    cr!("608.2h", "614.11", "702.52a");
    ruling!(
        "Ancient Excavation",
        "The number of cards in your hand is counted only as Ancient Excavation resolves. Ancient Excavation is on the stack at this time, so it’s not counted."
    );
    ruling!(
        "Ancient Excavation",
        "The number of cards drawn this way might not be equal to the number of cards in your hand (for example, if you apply Necroplasm’s dredge ability instead of drawing one of those cards). The number of cards that you actually drew is what determines how many cards you discard."
    );
    supported("Ancient Excavation");
    for dredge in [false, true] {
        let mut t = TestGame::new(2);
        t.hand(P0, "Grizzly Bears");
        t.hand(P0, "Hill Giant");
        let necro = t.graveyard(P0, "Necroplasm");
        lots(&mut t, P0);
        // The first draw is replaced by dredge 2 (or not); the others aren't.
        t.answer_yes(P0, dredge);
        t.answer_yes(P0, false);
        t.answer_yes(P0, false);
        let exc = t.hand(P0, "Ancient Excavation");
        t.cast(P0, exc).go();
        // A third card joins the hand while Ancient Excavation is on the stack.
        t.hand(P0, "Forest");
        t.resolve_all();
        let drawn = draws(&t, P0);
        assert_eq!(drawn, if dredge { 2 } else { 3 });
        let dredged = t.g.turn_events.iter().any(|e| {
            matches!(e, Event::ZoneChange { old, to: Zone::Hand(_), .. } if *old == necro)
        });
        assert_eq!(dredged, dredge);
        let discarded =
            t.g.turn_events
                .iter()
                .filter(|e| matches!(e, Event::Discarded { .. }))
                .count();
        assert_eq!(discarded, drawn, "one discard per card drawn");
        // Three in hand (not counting the Excavation) + three cards gained - discards.
        assert_eq!(t.hand_size(P0), 3 + 3 - discarded);
    }
}

#[test]
fn prismatic_undercurrents_additional_land_plays_are_cumulative() {
    cr!("305.2a");
    ruling!(
        "Prismatic Undercurrents",
        "The effect of Prismatic Undercurrents's last ability is cumulative with similar effects."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prismatic Undercurrents");
    t.battlefield(P0, "Prismatic Undercurrents");
    for i in 0..3 {
        let land = t.hand(P0, "Wastes");
        assert!(
            crate::r_s02_common::can_play_land(&mut t, P0, land),
            "land {i}"
        );
        t.play_land(P0, land).unwrap();
    }
    let land = t.hand(P0, "Wastes");
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, land));
}

#[test]
fn nissa_worldwaker_animated_lands_are_sick_and_stay_creatures() {
    cr!("302.6", "611.2a", "613.1d");
    ruling!(
        "Nissa, Worldwaker",
        "If the land targeted by the first ability entered the battlefield this turn, it won’t be able to attack and its activated abilities with {T} in the cost (including its mana ability) can’t be activated (unless something gives it haste)."
    );
    ruling!(
        "Nissa, Worldwaker",
        "The type-changing effects of the first and third abilities don’t have a duration. They last indefinitely."
    );
    supported("Nissa, Worldwaker");
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa, Worldwaker");
    let new = t.enter(P0, "Forest");
    t.answer_targets(P0, &[obj(new)]);
    activate_containing(&mut t, P0, nissa, "Elemental").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(new), (4, 4));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, new));
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, new));
    // Next turn: still a 4/4 creature, and no longer sick.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.obj_now(new).is_creature());
    assert_eq!(t.pt(new), (4, 4));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, new));
    // The third ability's lands: sick the turn they enter, and creatures for good.
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P0,
        &["Forest", "Forest", "Grizzly Bears", "Grizzly Bears"],
    );
    let forests = [obj(lib[0]), obj(lib[1])];
    let nissa = t.battlefield(P0, "Nissa, Worldwaker");
    t.g.objects[nissa.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    t.answer_choose(P0, &forests);
    activate_containing(&mut t, P0, nissa, "Search").unwrap();
    t.resolve_all();
    let found = controlled(&t, P0, "Forest");
    assert_eq!(found.len(), 2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(found
        .iter()
        .all(|f| !crate::r_s02_common::can_attack(&mut t, *f)));
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(found.iter().all(|f| t.pt(*f) == (4, 4)));
}

#[test]
fn sundering_eruption_cant_block_applies_to_creatures_as_blockers_are_declared() {
    cr!("611.2c", "509.1a");
    ruling!(
        "Sundering Eruption // Volcanic Fissure",
        "The effect of Sundering Eruption prevents all creatures without flying from blocking that turn, including creatures that lose flying after Sundering Eruption resolves and creatures without flying that enter the battlefield later in the turn. Creatures without flying that gain flying later in the turn will be able to block if they have flying when blockers are being declared."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Wastes");
    let birds = t.battlefield(P1, "Birds of Paradise");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    t.answer_yes(P1, false);
    cast_pool(
        &mut t,
        P0,
        "Sundering Eruption // Volcanic Fissure",
        &[obj(land)],
    );
    let can = |t: &mut TestGame, id: ObjectId| {
        t.g.recompute();
        t.g.can_block_at_all(t.g.current(id))
    };
    assert!(can(&mut t, birds));
    assert!(!can(&mut t, bears));
    // A creature entering later.
    let later = t.battlefield(P1, "Hill Giant");
    assert!(!can(&mut t, later));
    // Birds lose flying; Bears gain it.
    crate::r_s26_common::modify_until_eot(
        &mut t,
        birds,
        vec![Modification::RemoveKeyword(KeywordKind::Flying)],
    );
    crate::r_s26_common::modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flying))],
    );
    assert!(!can(&mut t, birds));
    assert!(can(&mut t, bears));
}
