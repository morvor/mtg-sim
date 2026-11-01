//! Rulings batch P013 — what the "when you cast" triggers and related abilities do as they
//! resolve: Conduit of Ruin's cost reduction (CR 601.2f), Dash Hopes' life payment
//! (CR 119.4), Distended Mindbender's choices, Eldrazi Obligator, Herigast, Twisted
//! Riddlekeeper, reveals from the library (Hatchery Spider, Ajani Unyielding, Genesis
//! Hydra, Genesis Wave), Oblivion Sower, Ulamog's attack trigger, Rory Williams, Leering
//! Emblem and Doomskar Titan.

use crate::r_s01_common::{stack_library, supported};
use crate::r_s04_common::top_of_stack;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land) && !o.tapped)
        .count()
}

/// Whether `p` can cast `card` with `method` now (casting it if so).
fn try_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.cast(p, card).method(method).try_go().is_ok()
}

// ---------------------------------------------------------------------------------------
// Conduit of Ruin: "The first creature spell you cast each turn costs {2} less to cast."
// ---------------------------------------------------------------------------------------

#[test]
fn conduit_of_ruin_reduces_only_generic_mana_and_not_the_mana_value() {
    cr!("601.2f", "202.3", "118.7");
    ruling!(
        "Conduit of Ruin",
        "Conduit of Ruin's last ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic component of that mana cost."
    );
    ruling!(
        "Conduit of Ruin",
        "Conduit of Ruin's last ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you actually pay."
    );
    supported("Conduit of Ruin");
    // Llanowar Elves ({G}) still costs {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conduit of Ruin");
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(!try_cast(&mut t, P0, elves, CastMethod::Normal));
    // Grizzly Bears ({1}{G}) costs {G}, but its mana value is still 2.
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(try_cast(&mut t, P0, bears, CastMethod::Normal));
    let spell = t.g.current(bears);
    assert_eq!(t.obj_now(spell).chars.mana_value(), 2);
    assert_eq!(
        format!("{}", t.obj_now(spell).chars.mana_cost.clone().unwrap()),
        "{1}{G}"
    );
}

#[test]
fn conduit_of_ruin_looks_at_the_whole_turn() {
    cr!("601.2f", "601.2");
    ruling!(
        "Conduit of Ruin",
        "Conduit of Ruin's last ability will look at the entire turn, even if Conduit of Ruin wasn't on the battlefield for some of it. Notably, if you cast Conduit of Ruin in a turn, then no other creature spell you cast that turn can be your first."
    );
    // Casting Conduit of Ruin itself was the first creature spell.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 6);
    let conduit = t.hand(P0, "Conduit of Ruin");
    t.cast(P0, conduit).go();
    t.resolve_all();
    assert!(t.on_battlefield(conduit));
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!try_cast(&mut t, P0, bears, CastMethod::Normal));
    // A creature spell cast before it entered was the first.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.resolve_all();
    t.battlefield(P0, "Conduit of Ruin");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!try_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn two_conduits_reduce_the_same_first_spell_by_four() {
    cr!("601.2f", "118.7");
    ruling!(
        "Conduit of Ruin",
        "If you control more than one Conduit of Ruin, the last ability of each of them applies only to the first creature spell you cast each turn, not to different spells; the first creature spell you cast each turn will cost {4} less."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conduit of Ruin");
    t.battlefield(P0, "Conduit of Ruin");
    // Craw Wurm ({4}{G}{G}) for {G}{G}.
    t.lands(P0, "Forest", 2);
    let wurm = t.hand(P0, "Craw Wurm");
    assert!(try_cast(&mut t, P0, wurm, CastMethod::Normal));
    t.resolve_all();
    // The second creature spell pays in full.
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!try_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn conduit_of_ruin_reduces_alternative_costs_and_converge_cant_ignore_it() {
    cr!("601.2f", "118.9d", "702.109a", "702.111a");
    ruling!(
        "Conduit of Ruin",
        "The cost reduction can apply to alternative costs (such as dash costs)."
    );
    ruling!(
        "Conduit of Ruin",
        "If the first creature spell you cast in a turn has converge, you can't ignore the cost reduction of Conduit of Ruin's last ability in order to spend more colors of mana."
    );
    supported("Mardu Scout");
    supported("Skyrider Elf");
    // Mardu Scout's dash cost {1}{R} becomes {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conduit of Ruin");
    t.lands(P0, "Mountain", 1);
    let scout = t.hand(P0, "Mardu Scout");
    assert!(try_cast(
        &mut t,
        P0,
        scout,
        CastMethod::Keyword(KeywordKind::Dash)
    ));
    // Skyrider Elf ({X}{G}{U}, converge) with X = 2 costs {G}{U}: two colors spent, two
    // counters, although four lands of four colors were available.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conduit of Ruin");
    for land in ["Forest", "Island", "Plains", "Mountain"] {
        t.lands(P0, land, 1);
    }
    let elf = t.hand(P0, "Skyrider Elf");
    t.cast(P0, elf).x(2).go();
    assert_eq!(untapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 2);
}

// ---------------------------------------------------------------------------------------
// Dash Hopes, Distended Mindbender, Eldrazi Obligator, Herigast, Twisted Riddlekeeper
// ---------------------------------------------------------------------------------------

#[test]
fn dash_hopes_needs_five_life_to_pay() {
    cr!("119.4", "118.3");
    ruling!(
        "Dash Hopes",
        "A player can’t choose to pay 5 life unless they actually have 5 or more life to pay."
    );
    // P1 at 4 life tries to pay: Dash Hopes still counters the Bolt.
    let mut t = TestGame::new(2);
    t.g.players[1].life = 4;
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.lands(P0, "Swamp", 2);
    let hopes = t.hand(P0, "Dash Hopes");
    t.cast(P0, hopes).target(obj(bolt)).go();
    t.answer_yes(P0, false);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.resolve_all();
    assert_eq!(t.life(P1), 4);
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

/// P0 casts Distended Mindbender ({8}) targeting P1.
fn mindbender(t: &mut TestGame) {
    t.lands(P0, "Wastes", 8);
    let m = t.hand(P0, "Distended Mindbender");
    t.cast(P0, m).target(P1).go();
}

#[test]
fn distended_mindbender_takes_two_cards_if_able_or_whichever_it_can() {
    cr!("609.3", "701.9b");
    ruling!(
        "Distended Mindbender",
        "You must choose two appropriate cards if able."
    );
    ruling!(
        "Distended Mindbender",
        "If there aren’t two appropriate cards to choose, choose whichever you can and that player discards that card."
    );
    supported("Distended Mindbender");
    // Both kinds of card: two are discarded, even if P0 tries to choose only one.
    let mut t = TestGame::new(2);
    let shock = t.hand(P1, "Shock");
    t.hand(P1, "Hill Giant");
    t.hand(P1, "Forest");
    t.answer_choose(P0, &[obj(shock)]);
    mindbender(&mut t);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_hand(P1, "Forest"));
    // Only a cheap nonland card (and a land): that one card is discarded.
    let mut t = TestGame::new(2);
    t.hand(P1, "Shock");
    t.hand(P1, "Forest");
    mindbender(&mut t);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert!(t.in_hand(P1, "Forest"));
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn eldrazi_obligator_may_target_any_creature() {
    cr!("115.1", "601.2c", "701.26");
    ruling!(
        "Eldrazi Obligator",
        "You may choose any creature as the target of Eldrazi Obligator’s triggered ability, including one that’s untapped or one you already control."
    );
    supported("Eldrazi Obligator");
    // (target's controller, tapped)
    for (owner, was_tapped) in [(P0, false), (P1, false), (P1, true)] {
        let mut t = TestGame::new(2);
        let target = t.battlefield_sick(owner, "Grizzly Bears");
        t.g.objects[target.0 as usize].tapped = was_tapped;
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 4);
        let ob = t.hand(P0, "Eldrazi Obligator");
        t.answer_targets(P0, &[obj(target)]);
        t.answer_yes(P0, true);
        t.cast(P0, ob).go();
        t.resolve_all();
        let o = t.obj_now(target);
        assert_eq!(o.controller, P0);
        assert!(!o.tapped);
        assert!(o.has_keyword(KeywordKind::Haste));
    }
}

#[test]
fn herigast_with_an_empty_hand_still_draws_three() {
    cr!("603.5", "118.12");
    ruling!(
        "Herigast, Erupting Nullkite",
        "You may choose to exile your hand even if you have no cards in hand. If you do, you'll still draw three cards."
    );
    supported("Herigast, Erupting Nullkite");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 9);
    let h = t.hand(P0, "Herigast, Erupting Nullkite");
    t.cast(P0, h).go();
    assert_eq!(t.hand_size(P0), 0);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn twisted_riddlekeeper_stuns_already_tapped_permanents() {
    cr!("122.1d", "701.26a");
    ruling!(
        "Twisted Riddlekeeper",
        "The triggered ability may target permanents that are already tapped. It will still put a stun counter on them."
    );
    supported("Twisted Riddlekeeper");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].tapped = true;
    t.lands(P0, "Wastes", 8);
    let r = t.hand(P0, "Twisted Riddlekeeper");
    t.cast(P0, r).targets(&[obj(giant)]).go();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.counters(giant, "stun"), 1);
}

// ---------------------------------------------------------------------------------------
// Revealing cards from the library
// ---------------------------------------------------------------------------------------

#[test]
fn a_forest_is_not_a_green_permanent_card() {
    cr!("105.2", "202.2", "305.7");
    ruling!(
        "Hatchery Spider",
        "A land card that produces green mana, even a Forest, normally has no color."
    );
    // X = 1 (one creature card in the graveyard); the Forest (mana value 0) is revealed.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let forest = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    t.lands(P0, "Forest", 7);
    let s = t.hand(P0, "Hatchery Spider");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Library(P0));
    assert_eq!(t.named_on_battlefield("Forest").len(), 7);
}

#[test]
fn ajani_unyielding_takes_artifact_creature_enchantment_and_planeswalker_cards() {
    cr!("110.4", "306.1");
    ruling!(
        "Ajani Unyielding",
        "A nonland permanent card is an artifact, creature, enchantment, or planeswalker card."
    );
    supported("Ajani Unyielding");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Unyielding");
    t.g.objects[ajani.0 as usize]
        .counters
        .insert("loyalty".into(), 4);
    stack_library(&mut t, P0, &["Ornithopter", "Pacifism", "Forest"]);
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Ornithopter") && t.in_hand(P0, "Pacifism"));
    assert!(!t.in_hand(P0, "Forest"));
    t.set_step(P0, Step::PrecombatMain);
    let ajani = t.g.current(ajani);
    t.g.objects[ajani.0 as usize].activations_this_turn.clear();
    stack_library(&mut t, P0, &["Grizzly Bears", "Shock", "Chandra, Torch of Defiance"]);
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Chandra, Torch of Defiance"));
    assert!(!t.in_hand(P0, "Shock"));
}

/// The number of times `p`'s library was shuffled so far.
fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, mtg_engine::events::Event::Shuffled { player } if *player == p))
        .count()
}

/// P0 casts Genesis Hydra with X.
fn genesis_hydra(t: &mut TestGame, x: i64) -> ObjectId {
    t.lands(P0, "Forest", 2 + x as usize);
    let h = t.hand(P0, "Genesis Hydra");
    t.cast(P0, h).x(x).go()
}

#[test]
fn genesis_hydra_shuffles_even_with_no_rest() {
    cr!("701.24a", "701.24d");
    ruling!(
        "Genesis Hydra",
        "If \"the rest\" is zero cards, either because X was 0 or because X was 1 and that card was put onto the battlefield, the library is still shuffled."
    );
    supported("Genesis Hydra");
    for (x, take) in [(0, false), (1, true)] {
        let mut t = TestGame::new(2);
        let elves = t.library_top(P0, "Llanowar Elves");
        if take {
            t.answer_choose(P0, &[obj(elves)]);
        }
        genesis_hydra(&mut t, x);
        let before = shuffles(&t, P0);
        t.resolve();
        assert_eq!(t.on_battlefield(elves), take);
        assert!(shuffles(&t, P0) > before, "X = {x}");
    }
}

#[test]
fn revealing_more_cards_than_the_library_has_reveals_all() {
    cr!("701.20a", "121.4");
    ruling!(
        "Genesis Hydra",
        "If you have fewer than X cards in your library, you reveal all of them."
    );
    supported("Genesis Wave");
    // Libraries of two permanent cards; X = 5.
    let setup = |t: &mut TestGame| {
        while let Some(c) = t.g.players[0].library.pop() {
            t.g.objects[c.0 as usize].zone = Zone::Outside(P0);
        }
        stack_library(t, P0, &["Grizzly Bears", "Llanowar Elves"]);
    };
    let mut t = TestGame::new(2);
    setup(&mut t);
    let bears = t.g.players[0].library[1];
    t.answer_choose(P0, &[obj(bears)]);
    genesis_hydra(&mut t, 5);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(!t.has_lost(P0));
    // Genesis Wave puts both.
    let mut t = TestGame::new(2);
    setup(&mut t);
    let lib = t.g.players[0].library.clone();
    t.answer_choose(P0, &lib.iter().map(|o| obj(*o)).collect::<Vec<_>>());
    t.lands(P0, "Forest", 8);
    let w = t.hand(P0, "Genesis Wave");
    t.cast(P0, w).x(5).go();
    t.resolve_all();
    assert!(lib.iter().all(|o| t.on_battlefield(*o)));
}

#[test]
fn an_aura_put_onto_the_battlefield_needs_a_legal_object_not_a_target() {
    cr!("303.4f", "303.4a", "702.16b");
    ruling!(
        "Genesis Hydra",
        "If you put an Aura onto the battlefield this way, you choose what it enchants as it enters the battlefield. This doesn't target any permanent or player, but it must be able to enchant that permanent or player."
    );
    // Pacifism (mana value 2, X = 2) onto a hexproof creature P1 controls.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let pac = t.library_top(P0, "Pacifism");
    t.answer_choose(P0, &[obj(pac)]);
    t.answer_choose(P0, &[obj(scout)]);
    genesis_hydra(&mut t, 2);
    t.resolve();
    assert!(t.on_battlefield(pac));
    assert_eq!(t.obj_now(pac).attached_to, Some(obj(scout)));
    // Not onto a creature with protection from white (Black Knight), the only creature.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "Black Knight");
    let pac = t.library_top(P0, "Pacifism");
    t.answer_choose(P0, &[obj(pac)]);
    t.answer_choose(P0, &[obj(knight)]);
    genesis_hydra(&mut t, 2);
    t.resolve();
    assert!(!t.on_battlefield(pac));
    assert!(t.obj_now(knight).attached_to.is_none());
}

// ---------------------------------------------------------------------------------------
// Oblivion Sower, Ulamog
// ---------------------------------------------------------------------------------------

#[test]
fn oblivion_sower_takes_any_face_up_exiled_land_card_the_player_owns() {
    cr!("406.3", "708.2");
    ruling!(
        "Oblivion Sower",
        "Oblivion Sower's ability allows you to put any land cards the player owns from exile onto the battlefield, regardless of how those cards were exiled."
    );
    ruling!(
        "Oblivion Sower",
        "Cards that are face down in exile have no characteristics. Such cards can't be put onto the battlefield with Oblivion Sower's ability."
    );
    supported("Oblivion Sower");
    let mut t = TestGame::new(2);
    let earlier = t.exile(P1, "Forest");
    let hidden = t.exile(P1, "Mountain");
    t.g.objects[hidden.0 as usize].face_down = true;
    t.g.recompute();
    let mine = t.exile(P0, "Plains");
    let top = t.library_top(P1, "Island");
    t.answer_choose_moved(P0, &[obj(earlier), obj(top)]);
    t.lands(P0, "Wastes", 6);
    let s = t.hand(P0, "Oblivion Sower");
    t.cast(P0, s).target(P1).go();
    let from = t.asked().len();
    t.resolve();
    // The face-down card and P0's own land card weren't offered.
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&obj(earlier)));
    assert!(!offered[0].contains(&obj(hidden)) && !offered[0].contains(&obj(mine)));
    assert!(t.on_battlefield(earlier));
    assert!(t.on_battlefield(top));
    assert_eq!(t.obj_now(earlier).controller, P0);
    assert_eq!(t.zone(hidden), Zone::Exile);
    assert_eq!(t.zone(mine), Zone::Exile);
}

#[test]
fn ulamog_exiles_face_up_and_a_short_library_entirely() {
    cr!("406.3", "704.5b", "121.4");
    ruling!(
        "Ulamog, the Ceaseless Hunger",
        "Cards exiled by Ulamog's first and third abilities are exiled face up."
    );
    ruling!(
        "Ulamog, the Ceaseless Hunger",
        "If the player has less than twenty cards in their library, exile all of them. That player won't lose the game until they have to draw a card from an empty library."
    );
    let mut t = TestGame::new(2);
    let ulamog = t.battlefield(P0, "Ulamog, the Ceaseless Hunger");
    // P1's library: 10 cards.
    for _ in 0..20 {
        let c = t.g.players[1].library.pop().unwrap();
        t.g.objects[c.0 as usize].zone = Zone::Outside(P1);
    }
    let lib = t.g.players[1].library.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(ulamog, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    for c in lib {
        assert_eq!(t.zone(c), Zone::Exile);
        assert!(!t.obj_now(c).face_down);
    }
    assert!(!t.has_lost(P1));
    assert!(t.g.run_until(10_000, |g| g.player(PlayerId(1)).has_lost));
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.g.turn.step, Step::Draw);
}

// ---------------------------------------------------------------------------------------
// Rory Williams: "When you cast this spell from anywhere other than exile, exile it with
// three time counters on it. It gains suspend. Then investigate."
// ---------------------------------------------------------------------------------------

#[test]
fn rory_williams_is_exiled_from_hand_and_resolves_when_cast_from_exile() {
    cr!("702.62a", "603.4", "601.2i");
    ruling!(
        "Rory Williams",
        "For example, if you cast Rory Williams from your hand, the triggered ability will exile it and it will gain suspend. When you cast it from exile, it resolves normally."
    );
    ruling!(
        "Rory Williams",
        "Rory will resolve normally if cast from exile for any reason, not only if it was cast with its suspend ability."
    );
    supported("Rory Williams");
    // From hand: exiled with three time counters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let rory = t.hand(P0, "Rory Williams");
    t.cast(P0, rory).go();
    t.resolve_all();
    let exiled = t.g.current(rory);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, "time"), 3);
    // The last time counter comes off: cast from exile, it resolves.
    t.g.objects[exiled.0 as usize]
        .counters
        .insert("time".into(), 1);
    t.advance_to(P0, Step::Upkeep);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(rory), "{}", t.dump_log());
    // Cast from exile another way (Light Up the Stage): it resolves too.
    let mut t = TestGame::new(2);
    let rory = t.library_top(P0, "Rory Williams");
    t.lands(P0, "Mountain", 3);
    let lus = t.hand(P0, "Light Up the Stage");
    t.cast(P0, lus).go();
    t.resolve_all();
    assert_eq!(t.zone(rory), Zone::Exile);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let rory = t.g.current(rory);
    t.cast(P0, rory).go();
    t.resolve_all();
    assert!(t.on_battlefield(rory));
}

#[test]
fn rory_williams_resolves_if_its_trigger_is_countered() {
    cr!("701.6b", "603.3");
    ruling!(
        "Rory Williams",
        "If you somehow counter the ability that triggers when you cast Rory Williams, it will resolve normally and will not be exiled."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let rory = t.hand(P0, "Rory Williams");
    t.cast(P0, rory).go();
    t.settle();
    let trigger = top_of_stack(&t);
    assert!(t.g.counter(trigger, None));
    t.resolve_all();
    assert!(t.on_battlefield(rory));
}

// ---------------------------------------------------------------------------------------
// Leering Emblem, Doomskar Titan
// ---------------------------------------------------------------------------------------

#[test]
fn leering_emblem_pumps_the_creature_equipped_as_it_resolves() {
    cr!("608.2h", "301.5");
    ruling!(
        "Leering Emblem",
        "The bonus is given to the creature Leering Emblem is attached to as the ability resolves."
    );
    supported("Leering Emblem");
    let mut t = TestGame::new(2);
    let emblem = t.battlefield(P0, "Leering Emblem");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.g.attach(emblem, obj(a));
    lands_for_cost(&mut t, P0, "Shock");
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.settle();
    // Moved to the Giant with the trigger on the stack.
    t.g.attach(emblem, obj(b));
    t.resolve_all();
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(t.pt(b), (5, 5));
}

#[test]
fn doomskar_titan_affects_creatures_controlled_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Doomskar Titan",
        "The triggered ability affects only creatures you control at the time the ability resolves, probably including Doomskar Titan itself. Creatures you begin to control later in the turn won’t gain the bonuses."
    );
    supported("Doomskar Titan");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let titan = t.enter(P0, "Doomskar Titan");
    t.resolve_all();
    assert_eq!(t.pt(titan), (5, 4));
    assert_eq!(t.pt(bears), (3, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    let later = t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(later), (3, 3));
    assert!(!t.obj_now(later).has_keyword(KeywordKind::Haste));
}

#[test]
fn hope_ender_coatl_asks_the_targeted_spells_controller_to_pay() {
    cr!("118.12a", "603.3");
    // "When you cast this spell, counter target spell an opponent controls unless they
    // pay {1}": "they" is that spell's controller, not the player who cast the Coatl.
    for p1_pays in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 2);
        let bolt = t.hand(P1, "Lightning Bolt");
        let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
        t.lands(P0, "Island", 4);
        let coatl = t.hand(P0, "Hope-Ender Coatl");
        t.answer_targets(P0, &[obj(bolt)]);
        t.cast(P0, coatl).go();
        let from = t.asked().len();
        t.answer(P1, DecisionKind::YesNo, Answer::Bool(p1_pays));
        t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
        t.resolve();
        let payers: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(payers, vec![P1]);
        assert_eq!(t.zone(bolt) == Zone::Stack, p1_pays);
        t.resolve_all();
        assert_eq!(t.life(P0), if p1_pays { 17 } else { 20 });
    }
}
