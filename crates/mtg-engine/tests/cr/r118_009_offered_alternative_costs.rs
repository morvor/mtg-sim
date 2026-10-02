//! CR 118.9, 118.9a, 118.9c, 118.9d, 601.2b, 601.2f: alternative costs that another
//! object's static ability offers for the spells a player casts ("You may pay
//! {W}{U}{B}{R}{G} rather than pay the mana cost for spells you cast."), including
//! once-each-turn ones (see `kw/offered_costs.rs`).

use mtg_engine::card::card;
use mtg_engine::casting::CastOption;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::kw::offered_costs::OFFERED_ALT_COST;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
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

/// {W}{U}{B}{R}{G} in `p`'s mana pool.
fn wubrg(t: &mut TestGame, p: PlayerId) {
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        add_mana(t, p, ty, 1);
    }
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

/// The ways `p` could cast `card` for an alternative cost another object offers.
fn offered(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
    t.g.recompute();
    t.g.cast_options(p, card)
        .into_iter()
        .filter(|o| o.alt_source.is_some())
        .collect()
}

fn time_counters(t: &mut TestGame, o: ObjectId, n: u32) {
    t.g.add_counters(Entity::Object(o), "time", n, None);
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Cast { card, method })
}

#[test]
fn an_offered_alternative_cost_is_paid_rather_than_the_mana_cost() {
    cr!("118.9", "118.9c", "601.2b", "601.2f");
    supported("Fist of Suns");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fist of Suns");
    // Grizzly Bears {1}{G}: paid with {W}{U}{B}{R}{G} instead, all of it.
    let bears = t.hand(P0, "Grizzly Bears");
    wubrg(&mut t, P0);
    assert!(can_cast(&mut t, P0, bears, OFFERED));
    let spell = t.cast(P0, bears).method(OFFERED).go();
    assert_eq!(pool_total(&t, P0), 0);
    // The spell's mana cost and mana value don't change (CR 118.9c).
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Without Fist of Suns there's no such way to cast it.
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    wubrg(&mut t, P0);
    assert!(offered(&mut t, P0, bears).is_empty());
    assert!(t.cast(P0, bears).method(OFFERED).try_go().is_err());
}

#[test]
fn an_offered_cost_cant_be_combined_with_another_alternative_cost() {
    cr!("118.9a", "601.2b");
    ruling!(
        "Fist of Suns",
        "You can't combine this with other alternative costs, such as flashback."
    );
    ruling!(
        "Jodah, Archmage Eternal",
        "You can't combine this with other alternative costs, such as flashback."
    );
    supported("Jodah, Archmage Eternal");
    for source in ["Fist of Suns", "Jodah, Archmage Eternal"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        // Think Twice {1}{U}, flashback {2}{U}: from the graveyard it's cast with
        // flashback, an alternative cost of its own; the offered cost can't replace it.
        let tt = t.graveyard(P0, "Think Twice");
        let opts = t.g.cast_options(P0, tt);
        assert!(opts
            .iter()
            .any(|o| o.method == CastMethod::Keyword(keywords::KeywordKind::Flashback)));
        assert!(offered(&mut t, P0, tt).is_empty(), "{source}");
        wubrg(&mut t, P0);
        assert!(!can_cast(&mut t, P0, tt, OFFERED));
        // From the hand it may be cast for the offered cost.
        let tt2 = t.hand(P0, "Think Twice");
        assert_eq!(offered(&mut t, P0, tt2).len(), 1);
    }
}

#[test]
fn additional_costs_apply_to_an_offered_alternative_cost() {
    cr!("118.9d", "118.8", "601.2b", "601.2f");
    ruling!(
        "Fist of Suns",
        "You can pay additional costs, such as kicker, in addition to this alternative cost."
    );
    // Burst Lightning {R}, kicker {4}: kicked for {W}{U}{B}{R}{G} plus {4}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fist of Suns");
    let bolt = t.hand(P0, "Burst Lightning");
    wubrg(&mut t, P0);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, bolt)
        .method(OFFERED)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // A cost increase applies to it too: Thalia makes noncreature spells cost {1} more,
    // so {W}{U}{B}{R}{G} alone isn't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fist of Suns");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bolt = t.hand(P0, "Burst Lightning");
    wubrg(&mut t, P0);
    assert!(!can_cast(&mut t, P0, bolt, OFFERED));
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(can_cast(&mut t, P0, bolt, OFFERED));
    t.cast(P0, bolt)
        .method(OFFERED)
        .kicked(false)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn a_mandatory_additional_cost_is_still_paid() {
    cr!("118.8", "118.9", "118.9c", "601.2f");
    ruling!(
        "Rooftop Storm",
        "You must still pay any mandatory additional costs, such as exiling a creature card from your graveyard for Makeshift Mauler."
    );
    ruling!(
        "Rooftop Storm",
        "The mana cost and mana value of the spell are unchanged. Rooftop Storm only changes what you pay."
    );
    supported("Rooftop Storm");
    supported("Makeshift Mauler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rooftop Storm");
    // Makeshift Mauler {3}{U} (a Zombie Horror): "As an additional cost to cast this
    // spell, exile a creature card from your graveyard."
    let mauler = t.hand(P0, "Makeshift Mauler");
    assert_eq!(offered(&mut t, P0, mauler).len(), 1);
    // No creature card to exile: it can't be cast for {0}.
    assert!(t.cast(P0, mauler).method(OFFERED).try_go().is_err());
    let bears = t.graveyard(P0, "Grizzly Bears");
    let spell = t.cast(P0, mauler).method(OFFERED).go();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
    assert_eq!(t.g.mana_value_of(spell), 4);
    // A creature spell that isn't a Zombie gets no such cost.
    let ogre = t.hand(P0, "Gray Ogre");
    assert!(offered(&mut t, P0, ogre).is_empty());
}

#[test]
fn x_is_zero_for_an_offered_cost_without_x() {
    cr!("107.3", "118.9", "601.2b");
    ruling!(
        "Fist of Suns",
        "If you apply Fist of Suns's alternative cost to a spell with {X} in its mana cost, X is 0."
    );
    ruling!(
        "Fist of Suns",
        "If you pay {W}{U}{B}{R}{G} rather than pay the mana cost of a spell with {X} in its mana cost, the only legal choice for X is 0."
    );
    ruling!(
        "Jodah, Archmage Eternal",
        "If you apply Jodah's alternative cost to a spell with {X} in its mana cost, X is 0."
    );
    for source in ["Fist of Suns", "Jodah, Archmage Eternal"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        // Endless One {X}: "This creature enters with X +1/+1 counters on it." X can't be
        // chosen: a 0/0 that dies.
        let one = t.hand(P0, "Endless One");
        wubrg(&mut t, P0);
        t.cast(P0, one).method(OFFERED).x(5).go();
        assert!(!t
            .asked()
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseX { .. })));
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Endless One"), "{source}");
    }
}

#[test]
fn an_offered_cost_applies_to_a_spell_cast_with_jump_start() {
    cr!("702.133a", "118.9", "601.2b");
    ruling!(
        "Radical Idea",
        "If an effect allows you to pay an alternative cost rather than a spell's mana cost, you may pay that alternative cost when you jump-start a spell. You'll still discard a card as an additional cost to cast it."
    );
    supported("Radical Idea");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fist of Suns");
    // Radical Idea {1}{U}, jump-start: with {W}{U}{B}{R}{G} and a card discarded.
    let idea = t.graveyard(P0, "Radical Idea");
    let discard = t.hand(P0, "Grizzly Bears");
    wubrg(&mut t, P0);
    let jump = CastMethod::Keyword(keywords::KeywordKind::JumpStart);
    let ways: Vec<CastOption> = t
        .g
        .cast_options(P0, idea)
        .into_iter()
        .filter(|o| o.method == jump)
        .collect();
    assert_eq!(ways.len(), 2);
    assert!(ways[0].alt_cost.is_none() && ways[1].alt_cost.is_some());
    // The player chooses the way with Fist of Suns's cost.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[Entity::Object(discard)]);
    t.cast(P0, idea).method(jump).go();
    assert_eq!(pool_total(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    // Exiled as jump-start says.
    assert!(t.in_exile("Radical Idea"));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn a_once_each_turn_alternative_cost_is_used_up() {
    cr!("118.9", "601.2b");
    ruling!(
        "Darksteel Monolith",
        "If you cast a card for an alternative cost of {0}, you can't pay any other alternative costs. You can, however, pay additional costs like kicker."
    );
    ruling!(
        "Darksteel Monolith",
        "You must still follow any relevant timing rules for the colorless spell you cast from your hand."
    );
    supported("Darksteel Monolith");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Darksteel Monolith");
    let wurm = t.hand(P0, "Juggernaut");
    let golem = t.hand(P0, "Juggernaut");
    // A colored spell gets no such cost.
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(offered(&mut t, P0, bears).is_empty());
    assert!(can_cast(&mut t, P0, wurm, OFFERED));
    t.cast(P0, wurm).method(OFFERED).go();
    // Timing rules still apply: not while it's on the stack.
    assert!(!can_cast(&mut t, P0, golem, OFFERED));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Juggernaut").len(), 1);
    // Used for this turn.
    assert!(offered(&mut t, P0, golem).is_empty());
    assert!(t.cast(P0, golem).method(OFFERED).try_go().is_err());
    // Not during an opponent's turn either (a creature spell needs sorcery timing).
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, golem, OFFERED));
    // Available again in the next turn of its controller.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, golem, OFFERED));
    t.cast(P0, golem).method(OFFERED).go();
    // Only from the hand.
    let in_gy = t.graveyard(P0, "Juggernaut");
    assert!(offered(&mut t, P0, in_gy).is_empty());
}

#[test]
fn several_objects_each_offer_their_once_each_turn_cost() {
    cr!("118.9", "601.2b", "202.3a");
    ruling!(
        "As Foretold",
        "If you control multiple As Foretolds, you may cast one spell for each of them paying {0}."
    );
    ruling!(
        "As Foretold",
        "If a spell has no mana cost, its mana value is 0. You can cast it with As Foretold's alternative cost."
    );
    supported("As Foretold");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "As Foretold");
    let b = t.battlefield(P0, "As Foretold");
    // No time counters: spells with mana value 0 or less.
    let vision = t.hand(P0, "Ancestral Vision");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(offered(&mut t, P0, bears).is_empty());
    let ways = offered(&mut t, P0, vision);
    assert_eq!(ways.len(), 2);
    // Ancestral Vision has no mana cost: it can be cast for {0} (CR 118.6 otherwise).
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, vision)
        .method(OFFERED)
        .target(Entity::Player(P0))
        .go();
    t.resolve_all();
    // Each As Foretold's cost is its own: two time counters on the other, and Grizzly
    // Bears (mana value 2) for {0} with it.
    let other = if t.g.history.once_permissions_used.iter().any(|(o, _)| *o == a) {
        b
    } else {
        a
    };
    time_counters(&mut t, other, 2);
    let ways = offered(&mut t, P0, bears);
    assert_eq!(ways.len(), 1);
    assert_eq!(ways[0].alt_source.as_ref().unwrap().source, other);
    t.cast(P0, bears).method(OFFERED).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn the_spell_cast_is_judged_by_the_half_or_face_being_cast() {
    cr!("118.9", "601.3e", "709.3a", "202.3d");
    ruling!(
        "As Foretold",
        "If you're casting a split card, only the mana value of the half you're casting is compared to the number of time counters on As Foretold."
    );
    ruling!(
        "As Foretold",
        "If the card has X in its mana cost, you must choose 0 as the value of X when casting it for another cost that doesn't include X."
    );
    let mut t = TestGame::new(2);
    let af = t.battlefield(P0, "As Foretold");
    time_counters(&mut t, af, 2);
    // Fire // Ice: each half has mana value 2 (the card has 4).
    let fi = t.hand(P0, "Fire // Ice");
    let ways = offered(&mut t, P0, fi);
    assert_eq!(ways.len(), 2);
    assert!(ways
        .iter()
        .all(|o| matches!(o.face, FaceState::Half(_)) && o.method == OFFERED));
    // Pull from Tomorrow {X}{U}{U} ("Draw X cards, then discard a card."): mana value 2
    // with X = 0, cast with X = 0.
    let mut t = TestGame::new(2);
    let af = t.battlefield(P0, "As Foretold");
    time_counters(&mut t, af, 2);
    let pull = t.hand(P0, "Pull from Tomorrow");
    let library = t.library_size(P0);
    t.cast(P0, pull).method(OFFERED).x(3).go();
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseX { .. })));
    t.resolve_all();
    assert_eq!(t.library_size(P0), library);
}

#[test]
fn an_offered_cost_doesnt_give_a_permission_to_cast_from_anywhere() {
    cr!("118.9", "601.3");
    ruling!(
        "Tlincalli Hunter // Retrieve Prey",
        "Tlincalli Hunter's ability doesn't give you permission to cast any spells from exile."
    );
    supported("Tlincalli Hunter // Retrieve Prey");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tlincalli Hunter // Retrieve Prey");
    let ogre = t.exile(P0, "Gray Ogre");
    t.g.recompute();
    assert!(t.g.cast_options(P0, ogre).is_empty());
    assert!(!can_cast(&mut t, P0, ogre, OFFERED));
    // With a permission to cast it (Retrieve Prey's), it may be cast for {0} once each
    // turn: one creature spell from exile.
    let bears = t.graveyard(P0, "Grizzly Bears");
    let prey = t.hand(P0, "Tlincalli Hunter // Retrieve Prey");
    add_mana(&mut t, P0, ManaType::G, 2);
    t.cast(P0, prey)
        .method(CastMethod::Half(1))
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(can_cast(&mut t, P0, bears, OFFERED));
    t.cast(P0, bears).method(OFFERED).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // A noncreature spell from exile gets no such cost.
    let bolt = t.exile(P0, "Lightning Bolt");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![bolt],
        ability::Duration::EndOfTurn,
        false,
        None,
    );
    t.g.recompute();
    assert!(!t.g.cast_options(P0, bolt).is_empty());
    assert!(offered(&mut t, P0, bolt).is_empty());
}

#[test]
fn an_offered_cost_can_be_life_equal_to_the_spells_mana_value() {
    cr!("118.9", "119.4", "601.2f");
    ruling!(
        "Demon of Fate's Design",
        "If you cast a spell with {X} in its mana cost this way, the only legal choice for X is 0."
    );
    supported("Demon of Fate's Design");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demon of Fate's Design");
    // Pacifism {1}{W}: 2 life.
    let ogre = t.battlefield(P1, "Gray Ogre");
    let pac = t.hand(P0, "Pacifism");
    let ways = offered(&mut t, P0, pac);
    assert_eq!(ways.len(), 1);
    t.cast(P0, pac)
        .method(OFFERED)
        .target(Entity::Object(ogre))
        .go();
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    // Once during each of its controller's turns.
    let pac2 = t.hand(P0, "Pacifism");
    assert!(offered(&mut t, P0, pac2).is_empty());
    t.advance_to(P1, Step::PrecombatMain);
    assert!(offered(&mut t, P0, pac2).is_empty());
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(offered(&mut t, P0, pac2).len(), 1);
    // A creature spell isn't an enchantment spell.
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(offered(&mut t, P0, bears).is_empty());
    // Mana Bloom {X}{G} ("This enchantment enters with X charge counters on it."): X is 0,
    // so it costs 1 life and enters with no counters.
    let life = t.life(P0);
    let bloom = t.hand(P0, "Mana Bloom");
    t.cast(P0, bloom).method(OFFERED).x(4).go();
    assert_eq!(t.life(P0), life - 1);
    t.resolve_all();
    let bloom = t.named_on_battlefield("Mana Bloom")[0];
    assert_eq!(t.counters(bloom, "charge"), 0);
}
