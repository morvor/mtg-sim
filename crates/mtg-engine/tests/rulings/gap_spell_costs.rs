//! Rulings about costs other objects offer for spells (gap-spell-costs): alternative costs
//! offered for the spells a player casts (Fist of Suns, Jodah, Leyline of Mutation,
//! Runeforge Champion, As Foretold, Warped Space, Access Maze, Demon of Fate's Design),
//! optional additional costs offered for them (the Defilers, Chorus of the Conclave), see
//! `kw/offered_costs.rs`.

use mtg_engine::ability::{Duration, PlayTerms};
use mtg_engine::card::card;
use mtg_engine::casting::{CastOption, PlayGrant};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::kw::offered_costs::OFFERED_ALT_COST;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const OFFERED: CastMethod = CastMethod::Alternative(OFFERED_ALT_COST);

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ]
    .iter()
    .map(|ty| t.player(p).mana_pool.count(*ty))
    .sum()
}

fn offered(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
    t.g.recompute();
    t.g.cast_options(p, card)
        .into_iter()
        .filter(|o| o.alt_source.is_some())
        .collect()
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Cast { card, method })
}

fn optional_costs_asked(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
        .count()
}

// --- Alternative costs offered for spells ----------------------------------------------

#[test]
fn runeforge_champion_offers_a_cost_for_rune_spells_only() {
    ruling!(
        "Runeforge Champion",
        "Runeforge Champion's last ability provides an alternative cost to cast Rune spells. You can't combine this with other alternative costs that may apply."
    );
    ruling!(
        "Runeforge Champion",
        "Casting a Rune spell for an alternative cost doesn't change its mana cost or mana value."
    );
    ruling!(
        "Runeforge Champion",
        "Runeforge Champion's last ability doesn't change when you can cast a Rune spell."
    );
    supported("Runeforge Champion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Runeforge Champion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Rune of Flight {1}{U}, an Aura: {1} instead.
    let rune = t.hand(P0, "Rune of Flight");
    let ogre = t.hand(P0, "Gray Ogre");
    assert_eq!(offered(&mut t, P0, rune).len(), 1);
    assert!(offered(&mut t, P0, ogre).is_empty());
    add_mana(&mut t, P0, ManaType::C, 1);
    // Not at instant speed: on the opponent's turn it can't be cast.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, rune, OFFERED));
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, rune, OFFERED));
    let spell = t
        .cast(P0, rune)
        .method(OFFERED)
        .target(Entity::Object(bears))
        .go();
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 2);
    // A Rune cast with an effect's permission to cast it without paying its mana cost (an
    // alternative cost the permission requires) can't also use the Champion's cost.
    let rune2 = t.exile(P0, "Rune of Flight");
    let turn = t.g.turn.number;
    t.g.play_grants.push(PlayGrant {
        player: P0,
        object: rune2,
        duration: Duration::EndOfTurn,
        free: true,
        source: None,
        turn,
        terms: PlayTerms::default(),
    });
    t.g.recompute();
    assert!(!t.g.cast_options(P0, rune2).is_empty());
    assert!(offered(&mut t, P0, rune2).is_empty());
}

#[test]
fn an_offered_cost_still_lets_mana_be_spent_as_though_any_color() {
    ruling!(
        "Fist of Suns",
        "If you cast a spell for which mana of any color can be spent to cast it, you may cast it for Fist of Suns's alternative cost and still spend any color of mana to cover that cost."
    );
    ruling!(
        "Jodah, Archmage Eternal",
        "If you cast a spell for which mana may be spent as though it were mana of any color, you may cast it for Jodah's alternative cost and still spend mana as though it were mana of any color."
    );
    ruling!(
        "Leyline of Mutation",
        "If you cast a spell for which mana may be spent as though it were mana of any color, you may cast it for Leyline of Mutation's second ability's alternative cost and still spend mana as though it were mana of any color."
    );
    supported("Leyline of Mutation");
    for source in [
        "Fist of Suns",
        "Jodah, Archmage Eternal",
        "Leyline of Mutation",
    ] {
        // A card in exile that an effect lets P0 cast, spending mana as though it were mana
        // of any color: {W}{U}{B}{R}{G} paid with five blue mana.
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        let bears = t.exile(P0, "Grizzly Bears");
        let turn = t.g.turn.number;
        t.g.play_grants.push(PlayGrant {
            player: P0,
            object: bears,
            duration: Duration::EndOfTurn,
            free: false,
            source: None,
            turn,
            terms: PlayTerms {
                spells_only: true,
                spend_as_any_color: true,
                ..Default::default()
            },
        });
        add_mana(&mut t, P0, ManaType::U, 5);
        assert!(can_cast(&mut t, P0, bears, OFFERED), "{source}");
        t.cast(P0, bears).method(OFFERED).go();
        assert_eq!(pool_total(&t, P0), 0);
        // Without that permission's terms, five blue mana can't pay it.
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        let bears = t.hand(P0, "Grizzly Bears");
        add_mana(&mut t, P0, ManaType::U, 5);
        assert!(!can_cast(&mut t, P0, bears, OFFERED), "{source}");
    }
}

#[test]
fn leyline_of_mutation_x_is_zero_and_no_other_alternative_cost() {
    ruling!(
        "Leyline of Mutation",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    ruling!(
        "Leyline of Mutation",
        "If you cast a spell for another cost \"rather than pay\" its mana cost, you can't choose to cast it for any other alternative cost."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Mutation");
    // Endless One {X}: a 0/0 for {W}{U}{B}{R}{G}.
    let one = t.hand(P0, "Endless One");
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        add_mana(&mut t, P0, ty, 1);
    }
    t.cast(P0, one).method(OFFERED).x(3).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
    // Think Twice with flashback from the graveyard: not with the Leyline's cost too.
    let tt = t.graveyard(P0, "Think Twice");
    assert!(offered(&mut t, P0, tt).is_empty());
}

#[test]
fn as_foretold_mandatory_additional_costs_are_still_paid() {
    ruling!(
        "As Foretold",
        "If you cast a spell for an alternative cost of {0}, you can't pay any other alternative costs, such as emerge costs. You can, however, pay additional costs, such as bargain. If the spell has any mandatory additional costs, such as that of Tormenting Voice, you must pay those to cast the spell."
    );
    ruling!(
        "Charred Foyer // Warped Space",
        "If you cast a spell for an alternative cost of {0}, you can't pay any other alternative costs. You can, however, pay additional costs, such as kicker. If the card has any mandatory additional costs, you must pay those."
    );
    // Tormenting Voice {1}{R}: "As an additional cost to cast this spell, discard a card."
    let mut t = TestGame::new(2);
    let af = t.battlefield(P0, "As Foretold");
    t.g.add_counters(Entity::Object(af), "time", 2, None);
    let voice = t.hand(P0, "Tormenting Voice");
    // No card to discard: it can't be cast, even for {0}.
    assert!(t.cast(P0, voice).method(OFFERED).try_go().is_err());
    let fodder = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.cast(P0, voice).method(OFFERED).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Warped Space (the unlocked door of Charred Foyer // Warped Space): {0} for a spell
    // cast from exile; Burst Lightning kicked for {4}.
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Charred Foyer // Warped Space");
    rooms::unlock(&mut t.g, room, 1, P0);
    let bolt = t.exile(P0, "Burst Lightning");
    let turn = t.g.turn.number;
    t.g.play_grants.push(PlayGrant {
        player: P0,
        object: bolt,
        duration: Duration::EndOfTurn,
        free: false,
        source: None,
        turn,
        terms: PlayTerms::default(),
    });
    add_mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, bolt)
        .method(OFFERED)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn as_foretold_counts_time_counters_from_any_source() {
    ruling!(
        "As Foretold",
        "All counters with the same name are indistinguishable from other counters with that name. Cards from the Time Spiral block that interact with time counters will interact with As Foretold."
    );
    supported("Timecrafting");
    let mut t = TestGame::new(2);
    let af = t.battlefield(P0, "As Foretold");
    t.g.add_counters(Entity::Object(af), "time", 1, None);
    // Gray Ogre (mana value 3) isn't offered {0} with one time counter.
    let ogre = t.hand(P0, "Gray Ogre");
    assert!(offered(&mut t, P0, ogre).is_empty());
    // Timecrafting {X}{R}, X = 2: "Put X time counters on target permanent with a time
    // counter on it or suspended card."
    let tc = t.hand(P0, "Timecrafting");
    add_mana(&mut t, P0, ManaType::R, 3);
    t.cast(P0, tc)
        .modes(&[1])
        .x(2)
        .target(Entity::Object(af))
        .go();
    t.resolve_all();
    assert_eq!(t.counters(af, "time"), 3);
    assert_eq!(offered(&mut t, P0, ogre).len(), 1);
    t.cast(P0, ogre).method(OFFERED).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gray Ogre").len(), 1);
}

#[test]
fn trinisphere_leaving_as_a_cost_doesnt_change_the_locked_in_total() {
    ruling!(
        "Trinisphere",
        "If Trinisphere leaves the battlefield or becomes tapped or untapped as a cost to cast a spell, this cost is paid after you've locked in the total cost."
    );
    cr!("601.2f", "601.2h");
    supported("Shrapnel Blast");
    // Shrapnel Blast {1}{R}: "As an additional cost to cast this spell, sacrifice an
    // artifact." Sacrificing Trinisphere for it, it still costs three mana.
    let mut t = TestGame::new(2);
    let tri = t.battlefield(P0, "Trinisphere");
    let blast = t.hand(P0, "Shrapnel Blast");
    add_mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose(P0, &[Entity::Object(tri)]);
    assert!(t
        .cast(P0, blast)
        .target(Entity::Player(P1))
        .try_go()
        .is_err());
    assert!(t.on_battlefield(tri));
    t.clear_answers();
    add_mana(&mut t, P0, ManaType::R, 1);
    t.answer_choose(P0, &[Entity::Object(tri)]);
    t.cast(P0, blast).target(Entity::Player(P1)).go();
    assert!(!t.on_battlefield(tri));
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn an_adventure_may_be_cast_for_an_offered_cost() {
    ruling!(
        "Tlincalli Hunter // Retrieve Prey",
        "Casting a card as an Adventure isn’t casting it for an alternative cost. Effects that allow you to cast a spell for an alternative cost or without paying its mana cost may allow you to apply those to the Adventure."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fist of Suns");
    // Bonecrusher Giant // Stomp: either the creature or its Adventure for
    // {W}{U}{B}{R}{G}.
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    let ways = offered(&mut t, P0, giant);
    assert_eq!(ways.len(), 2);
    assert!(ways.iter().any(|o| o.face == FaceState::Half(1)));
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        add_mana(&mut t, P0, ty, 1);
    }
    // The player chooses the Adventure.
    let i = ways
        .iter()
        .position(|o| o.face == FaceState::Half(1))
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.cast(P0, giant)
        .method(OFFERED)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // On an adventure: exiled, and castable as the creature later.
    assert_eq!(t.zone(t.g.current(giant)), Zone::Exile);
}

#[test]
fn access_maze_cost_is_life_equal_to_mana_value_from_the_hand() {
    ruling!(
        "Cramped Vents // Access Maze",
        "The spells you cast using Access Maze's ability have an alternative cost that's paying life equal to their mana value. You can't choose to pay their mana cost and may not pay any other alternative costs."
    );
    ruling!(
        "Cramped Vents // Access Maze",
        "You follow all timing rules for spells cast with Access Maze's ability."
    );
    ruling!(
        "Demon of Fate's Design",
        "If you cast a spell for another cost \"rather than paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, those must be paid to cast the card."
    );
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Cramped Vents // Access Maze");
    rooms::unlock(&mut t.g, room, 1, P0);
    // Hill Giant {3}{R}: 4 life.
    let giant = t.hand(P0, "Hill Giant");
    assert_eq!(offered(&mut t, P0, giant).len(), 1);
    // Not at instant speed.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, giant, OFFERED));
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, giant).method(OFFERED).go();
    assert_eq!(t.life(P0), 16);
    // Only from the hand: a card in the graveyard with flashback isn't offered it.
    let tt = t.graveyard(P0, "Think Twice");
    assert!(offered(&mut t, P0, tt).is_empty());
    // Demon of Fate's Design: an Aura with escape from the graveyard isn't offered its cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demon of Fate's Design");
    let eyes = t.graveyard(P0, "Sentinel's Eyes");
    assert!(offered(&mut t, P0, eyes).is_empty());
    let eyes2 = t.hand(P0, "Sentinel's Eyes");
    assert_eq!(offered(&mut t, P0, eyes2).len(), 1);
}

#[test]
fn kentaro_makes_samurai_castable_for_generic_mana() {
    cr!("118.9", "118.9c");
    ruling!(
        "Kentaro, the Smiling Cat",
        "Kentaro doesn’t change when you can cast Samurai. It just makes Samurai castable for generic mana."
    );
    ruling!(
        "Kentaro, the Smiling Cat",
        "Kentaro’s ability only applies while Kentaro is on the battlefield. You have to pay for Kentaro normally. It still costs {1}{W}."
    );
    supported("Kentaro, the Smiling Cat");
    // In the hand, Kentaro (a Samurai) isn't offered its own cost.
    let mut t = TestGame::new(2);
    let k = t.hand(P0, "Kentaro, the Smiling Cat");
    assert!(offered(&mut t, P0, k).is_empty());
    // On the battlefield: Kitsune Blademaster {2}{W} for {3} of any mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kentaro, the Smiling Cat");
    let fox = t.hand(P0, "Kitsune Blademaster");
    let ogre = t.hand(P0, "Gray Ogre");
    assert!(offered(&mut t, P0, ogre).is_empty());
    add_mana(&mut t, P0, ManaType::R, 2);
    // Two mana isn't enough; and not on the opponent's turn.
    assert!(!can_cast(&mut t, P0, fox, OFFERED));
    add_mana(&mut t, P0, ManaType::R, 1);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, fox, OFFERED));
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, fox, OFFERED));
    let spell = t.cast(P0, fox).method(OFFERED).go();
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 3);
}

#[test]
fn dream_halls_replaces_only_the_mana_cost() {
    cr!("118.9", "118.9d", "118.8");
    ruling!(
        "Dream Halls",
        "This only replaces the mana cost (the mana in the upper right hand corner of the card). It will not pay additional costs from the card text (such as Buyback) or from other effects."
    );
    supported("Dream Halls");
    supported("Whispers of the Muse");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Dream Halls");
    // Every player's spells: P0 casts Lightning Bolt by discarding Shock (red), not by
    // discarding Counterspell (blue).
    let bolt = t.hand(P0, "Lightning Bolt");
    let counter = t.hand(P0, "Counterspell");
    assert!(!can_cast(&mut t, P0, bolt, OFFERED));
    let shock = t.hand(P0, "Shock");
    assert!(can_cast(&mut t, P0, bolt, OFFERED));
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.cast(P0, bolt)
        .method(OFFERED)
        .target(Entity::Player(P1))
        .go();
    assert!(t.in_graveyard(P0, "Shock"));
    assert!(t.in_hand(P0, "Counterspell"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Whispers of the Muse {U}, buyback {5}: for a blue card, with buyback paid in mana.
    let whispers = t.hand(P0, "Whispers of the Muse");
    add_mana(&mut t, P0, ManaType::C, 5);
    t.answer_choose(P0, &[Entity::Object(counter)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, whispers).method(OFFERED).go();
    assert_eq!(pool_total(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Counterspell"));
    t.resolve_all();
    assert!(t.in_hand(P0, "Whispers of the Muse"));
}

#[test]
fn conspiracy_unraveler_cost_keeps_mandatory_additional_costs_and_no_other_alternative() {
    cr!("118.9a", "118.9d", "601.2b");
    ruling!(
        "Conspiracy Unraveler",
        "If you cast a spell for another cost \"rather than pay its mana cost\", you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, such as that of Demand Answers, those must be paid to cast the card."
    );
    supported("Conspiracy Unraveler");
    supported("Demand Answers");
    // Conspiracy Unraveler: "You may collect evidence 10 rather than pay the mana cost for
    // spells you cast." Demand Answers {1}{R}: "As an additional cost to cast this spell,
    // sacrifice an artifact or discard a card. Draw two cards."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conspiracy Unraveler");
    let dragons = [
        t.graveyard(P0, "Shivan Dragon"),
        t.graveyard(P0, "Shivan Dragon"),
    ];
    let demand = t.hand(P0, "Demand Answers");
    // No artifact to sacrifice and no other card to discard: it can't be cast, even with
    // enough evidence.
    assert_eq!(offered(&mut t, P0, demand).len(), 1);
    assert!(t.cast(P0, demand).method(OFFERED).try_go().is_err());
    assert!(dragons.iter().all(|d| t.zone(*d) == Zone::Graveyard(P0)));
    // With a card to discard: the evidence is collected and the card discarded, no mana.
    let fodder = t.hand(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, demand, OFFERED));
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.cast(P0, demand).method(OFFERED).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(dragons.iter().all(|d| t.zone(t.g.current(*d)) == Zone::Exile));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // Think Twice in the graveyard is cast with flashback (an alternative cost of its
    // own): not for the Unraveler's cost too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conspiracy Unraveler");
    t.graveyard(P0, "Shivan Dragon");
    t.graveyard(P0, "Shivan Dragon");
    let tt = t.graveyard(P0, "Think Twice");
    t.g.recompute();
    assert!(t
        .g
        .cast_options(P0, tt)
        .iter()
        .any(|o| o.method == CastMethod::Keyword(keywords::KeywordKind::Flashback)));
    assert!(offered(&mut t, P0, tt).is_empty());
}

#[test]
fn conspiracy_unraveler_evidence_cant_include_the_card_being_cast() {
    cr!("601.2a", "601.2h", "701.59a");
    ruling!(
        "Conspiracy Unraveler",
        "If you are casting a spell from your graveyard (for example, a spell with flashback) you can't also exile that card to pay the alternative collect evidence cost offered by Conspiracy Unraveler."
    );
    // Radical Idea {1}{U}, jump-start, cast from the graveyard for collect evidence 10: it
    // is on the stack as the cost is paid, so its own mana value (2) can't count.
    let setup = |t: &mut TestGame| -> (ObjectId, ObjectId) {
        t.battlefield(P0, "Conspiracy Unraveler");
        let idea = t.graveyard(P0, "Radical Idea");
        t.graveyard(P0, "Shivan Dragon");
        t.graveyard(P0, "Gray Ogre");
        let fodder = t.hand(P0, "Grizzly Bears");
        (idea, fodder)
    };
    let jump = CastMethod::Keyword(keywords::KeywordKind::JumpStart);
    // 6 + 3 other than Radical Idea: not enough, and not offered as a legal action.
    let mut t = TestGame::new(2);
    let (idea, fodder) = setup(&mut t);
    assert_eq!(
        offered(&mut t, P0, idea)
            .iter()
            .filter(|o| o.method == jump)
            .count(),
        1
    );
    assert!(!can_cast(&mut t, P0, idea, jump.clone()));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    assert!(t.cast(P0, idea).method(jump.clone()).try_go().is_err());
    assert_eq!(t.zone(t.g.current(idea)), Zone::Graveyard(P0));
    // With Llanowar Elves (1) too, the other cards total 10: it's cast, and Radical Idea
    // is exiled by jump-start as it resolves, not as evidence.
    let mut t = TestGame::new(2);
    let (idea, fodder) = setup(&mut t);
    t.graveyard(P0, "Llanowar Elves");
    assert!(can_cast(&mut t, P0, idea, jump.clone()));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    let spell = t.cast(P0, idea).method(jump).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_exile("Radical Idea"));
}

#[test]
fn aluren_casts_cheap_creatures_free_with_flash_for_any_player() {
    cr!("118.9", "118.9a", "118.9c", "601.3c", "107.3b");
    ruling!(
        "Aluren",
        "You can't choose to cast a creature as though it had flash via Aluren and still pay the mana cost. You either cast the creature normally, or via Aluren without paying the mana cost."
    );
    ruling!(
        "Aluren",
        "The mana cost of the creatures being cast is still the stated cost on the card, even though you did not pay the cost."
    );
    ruling!("Aluren", "If creature with X in its cost is cast this way, X can only be 0.");
    ruling!(
        "Aluren",
        "Aluren checks the actual printed cost on the creature card, and is not affected by things which allow you to cast the spell for less."
    );
    ruling!(
        "Aluren",
        "You can't use Aluren when casting a creature using another alternate means, such as the Morph ability."
    );
    supported("Aluren");
    // "Any player may cast creature spells with mana value 3 or less without paying their
    // mana costs and as though they had flash." P0 controls it; P1 uses it on P0's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aluren");
    let bears = t.hand(P1, "Grizzly Bears");
    add_mana(&mut t, P1, ManaType::G, 2);
    // During P0's turn: for nothing, as though it had flash — but not for its mana cost.
    assert!(can_cast(&mut t, P1, bears, CastMethod::Free));
    assert!(!can_cast(&mut t, P1, bears, CastMethod::Normal));
    let spell = t.cast(P1, bears).method(CastMethod::Free).go();
    assert_eq!(t.g.mana_value_of(spell), 2);
    assert_eq!(t.player(P1).mana_pool.count(ManaType::G), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Mana value 4 isn't 3 or less, even when a reduction makes it cost {3}: Juggernaut
    // with Etherium Sculptor ("Artifact spells you cast cost {1} less to cast.").
    t.battlefield(P0, "Etherium Sculptor");
    let jug = t.hand(P0, "Juggernaut");
    assert!(offered(&mut t, P0, jug).is_empty());
    // Endless One {X}: X is 0, a 0/0 that dies.
    let one = t.hand(P0, "Endless One");
    t.cast(P0, one).method(CastMethod::Free).x(3).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
    // Fathom Seer (morph): cast face down for {3} (an alternative way of its own), not
    // also for nothing.
    let seer = t.hand(P0, "Fathom Seer");
    let ways = offered(&mut t, P0, seer);
    assert_eq!(ways.len(), 1);
    assert_eq!(ways[0].method, CastMethod::Free);
    assert_eq!(ways[0].face, FaceState::Front);
}

fn energy(t: &TestGame) -> u32 {
    t.player(P0).counters.get("energy").copied().unwrap_or(0)
}

#[test]
fn primal_prayers_cost_comes_with_flash() {
    cr!("601.3c", "118.9", "118.9a");
    ruling!(
        "Primal Prayers",
        "If a spell you cast this way has {X} in its mana cost, you must choose 0 as the value of X when casting it."
    );
    ruling!(
        "Primal Prayers",
        "If you cast a spell for another cost \"rather than paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, those must be paid to cast it."
    );
    supported("Primal Prayers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primal Prayers");
    t.g.players[P0.idx()].counters.insert("energy".into(), 3);
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    // Mana value 4: not one of those spells.
    assert!(offered(&mut t, P0, giant).is_empty());
    // During the opponent's turn: Grizzly Bears for {E}, as though it had flash (but not
    // for its mana cost).
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(can_cast(&mut t, P0, bears, OFFERED));
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.cast(P0, bears).method(OFFERED).go();
    assert_eq!(energy(&t), 2);
    assert_eq!(pool_total(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Endless One {X}: X is 0.
    let one = t.hand(P0, "Endless One");
    t.cast(P0, one).method(OFFERED).x(2).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
    // Goblin Bushwhacker {R}, kicker {R}: kicked for {E} plus {R}.
    t.set_step(P0, Step::PrecombatMain);
    let bw = t.hand(P0, "Goblin Bushwhacker");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bw).method(OFFERED).kicked(true).go();
    assert_eq!(energy(&t), 0);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.pt(bears).0, 3);
    // No energy left: it can't be cast that way.
    let bears2 = t.hand(P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears2, OFFERED));
}

// --- Optional additional costs offered for spells -----------------------------------------

#[test]
fn a_defiler_cost_is_paid_once_per_spell_and_adds_no_phyrexian_symbols() {
    ruling!(
        "Defiler of Vigor",
        "You may only pay the additional cost once per permanent spell."
    );
    ruling!(
        "Defiler of Faith",
        "You may only pay the additional cost once per permanent spell."
    );
    ruling!(
        "Defiler of Dreams",
        "You may only pay the additional cost once per permanent spell."
    );
    ruling!(
        "Defiler of Flesh",
        "You may only pay the additional cost once per permanent spell."
    );
    ruling!(
        "Defiler of Instinct",
        "You may only pay the additional cost once per permanent spell."
    );
    ruling!(
        "Defiler of Vigor",
        "this ability does not cause the spells to have Phyrexian mana symbols in their costs"
    );
    ruling!(
        "Defiler of Faith",
        "this ability does not cause the spells to have Phyrexian mana symbols in their costs"
    );
    ruling!(
        "Defiler of Dreams",
        "this ability does not cause the spells to have Phyrexian mana symbols in their costs"
    );
    ruling!(
        "Defiler of Flesh",
        "this ability does not cause the spells to have Phyrexian mana symbols in their costs"
    );
    ruling!(
        "Defiler of Instinct",
        "this ability does not cause the spells to have Phyrexian mana symbols in their costs"
    );
    supported("Rage Extractor");
    for (defiler, spell, ty) in [
        ("Defiler of Vigor", "Llanowar Elves", ManaType::G),
        ("Defiler of Faith", "Savannah Lions", ManaType::W),
        ("Defiler of Dreams", "Faerie Miscreant", ManaType::U),
        ("Defiler of Flesh", "Vampire Lacerator", ManaType::B),
        ("Defiler of Instinct", "Goblin Guide", ManaType::R),
    ] {
        supported(defiler);
        supported(spell);
        let mut t = TestGame::new(2);
        t.battlefield(P0, defiler);
        // Rage Extractor: "Whenever you cast a spell with {P} in its mana cost, ..."
        let extractor = t.battlefield(P0, "Rage Extractor");
        let c = t.hand(P0, spell);
        add_mana(&mut t, P0, ty, 1);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.cast(P0, c).go();
        t.settle();
        // Asked once, 2 life paid, the one colored symbol reduced: the mana is left over.
        assert_eq!(optional_costs_asked(&t), 1, "{defiler}");
        assert_eq!(t.life(P0), 18, "{defiler}");
        assert_eq!(pool_total(&t, P0), 1, "{defiler}");
        // No Phyrexian symbol: Rage Extractor doesn't trigger.
        assert_eq!(triggered_from(&t, extractor), 0, "{defiler}");
        t.resolve_all();
        // (It does for a spell with one: Mutagenic Growth {G/P}.)
        if defiler == "Defiler of Vigor" {
            let growth = t.hand(P0, "Mutagenic Growth");
            let elves = t.named_on_battlefield("Llanowar Elves")[0];
            t.cast(P0, growth).target(Entity::Object(elves)).go();
            t.settle();
            assert_eq!(triggered_from(&t, extractor), 1);
        }
    }
}

/// The triggered abilities of `src` on the stack.
fn triggered_from(t: &TestGame, src: ObjectId) -> usize {
    t.g.stack
        .iter()
        .filter(|s| {
            matches!(
                t.g.obj(**s).stack.as_deref().map(|si| &si.kind),
                Some(StackKind::Triggered { source, .. }) if *source == src
            )
        })
        .count()
}

#[test]
fn chorus_of_the_conclave_applies_only_to_spells_being_cast() {
    ruling!(
        "Chorus of the Conclave",
        "Chorus of the Conclave's ability works only while it's on the battlefield, so you can't use it to put +1/+1 counters on itself."
    );
    ruling!(
        "Chorus of the Conclave",
        "Chorus of the Conclave's ability applies to creature spells only as they're being cast. You can't pay mana to put counters on creatures being put onto the battlefield by an effect."
    );
    // Cast from the hand, Chorus itself isn't offered its cost.
    let mut t = TestGame::new(2);
    let chorus = t.hand(P0, "Chorus of the Conclave");
    for ty in [ManaType::G, ManaType::W] {
        add_mana(&mut t, P0, ty, 4);
    }
    t.cast(P0, chorus).go();
    assert_eq!(optional_costs_asked(&t), 0);
    t.resolve_all();
    let chorus = t.named_on_battlefield("Chorus of the Conclave")[0];
    assert_eq!(t.counters(chorus, "+1/+1"), 0);
    // Reanimate returns a creature card: it isn't cast, so no cost and no counters.
    let ogre = t.graveyard(P0, "Gray Ogre");
    let re = t.hand(P0, "Reanimate");
    add_mana(&mut t, P0, ManaType::B, 1);
    t.cast(P0, re).target(Entity::Object(ogre)).go();
    t.resolve_all();
    assert_eq!(optional_costs_asked(&t), 0);
    let ogre = t.named_on_battlefield("Gray Ogre")[0];
    assert_eq!(t.counters(ogre, "+1/+1"), 0);
}

#[test]
fn chorus_of_the_conclave_mana_can_be_paid_with_convoke() {
    ruling!(
        "Chorus of the Conclave",
        "Chorus of the Conclave's ability combines well with the convoke mechanic, effectively letting you tap creatures to put +1/+1 counters on the creature with convoke that you're casting, if you choose to do so."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chorus of the Conclave");
    // Siege Wurm {5}{G}{G}, convoke, with {2} more: nine creatures tapped for it.
    let mut helpers = Vec::new();
    for _ in 0..9 {
        helpers.push(t.battlefield(P0, "Llanowar Elves"));
    }
    let wurm = t.hand(P0, "Siege Wurm");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.cast(P0, wurm).go();
    assert!(helpers.iter().all(|h| t.obj_now(*h).tapped));
    t.resolve_all();
    let wurm = t.named_on_battlefield("Siege Wurm")[0];
    assert_eq!(t.counters(wurm, "+1/+1"), 2);
}
