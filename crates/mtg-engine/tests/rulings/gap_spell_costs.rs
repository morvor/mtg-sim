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
