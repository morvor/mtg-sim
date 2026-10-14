//! Rulings batch P032 — cost increases and reductions (CR 601.2f, 118.7, 602.2b): the
//! total cost is the mana cost or alternative cost plus increases, then minus reductions;
//! generic reductions can't reduce colored requirements; the mana value doesn't change
//! (CR 202.3); and abilities that stop activated abilities other than mana abilities (CR
//! 605.1a).

use crate::r_p032_common::*;
use crate::r_s01_common::{supported, tapped_lands};
use crate::r_s02_common::{can_activate, can_cast};
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::mana_value;
use crate::r_s27_common::can_activate_containing;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn main_phase(n: usize) -> TestGame {
    let mut t = TestGame::new(n);
    t.set_step(P0, Step::PrecombatMain);
    t
}

/// P0 casts `name` from hand with `method` (lands already given); returns the spell and
/// the number of lands tapped for it.
fn cast_paying(t: &mut TestGame, name: &str, method: CastMethod, targets: &[Entity]) -> (ObjectId, usize) {
    let before = tapped_lands(t, P0);
    let card = t.hand(P0, name);
    let mut b = t.cast(P0, card).method(method);
    for e in targets {
        b = b.target(*e);
    }
    let spell = b.go();
    (spell, tapped_lands(t, P0) - before)
}

#[test]
fn cost_reductions_apply_after_increases() {
    cr!("601.2f", "118.7", "202.3");
    ruling!(
        "Geist-Fueled Scarecrow",
        "Abilities that reduce the cost of a spell apply after abilities that raise them. For example, Bedlam Reveler will cost {R}{R} to cast if you have seven instant and sorcery cards in your graveyard while you control Geist-Fueled Scarecrow."
    );
    ruling!(
        "God-Pharaoh's Statue",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as that of God-Pharaoh's Statue's first ability), then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "Vryn Wingmare",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as that of Vryn Wingmare), then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Geist-Fueled Scarecrow");
    supported("Bedlam Reveler");
    // Bedlam Reveler ({6}{R}{R}, "costs {1} less to cast for each instant and sorcery
    // card in your graveyard") with seven: {7}{R}{R} − 7 = {R}{R} (reducing first would
    // leave {1}{R}{R}).
    let mut t = main_phase(2);
    t.battlefield(P0, "Geist-Fueled Scarecrow");
    for _ in 0..7 {
        t.graveyard(P0, "Shock");
    }
    t.lands(P0, "Mountain", 3);
    let (spell, paid) = cast_paying(&mut t, "Bedlam Reveler", CastMethod::Normal, &[]);
    assert_eq!(paid, 2);
    assert_eq!(mana_value(&t, spell), 8);
    // The Great Henge ({7}{G}{G}, "costs {X} less, where X is the greatest power among
    // creatures you control") with Ghalta (12 power), and P1's God-Pharaoh's Statue
    // ({2} more) or Vryn Wingmare ({1} more, noncreature): {G}{G}; mana value 9.
    for increaser in ["God-Pharaoh's Statue", "Vryn Wingmare"] {
        supported(increaser);
        let mut t = main_phase(2);
        t.battlefield(P1, increaser);
        t.battlefield(P0, "Ghalta, Primal Hunger");
        t.lands(P0, "Forest", 4);
        let (spell, paid) = cast_paying(&mut t, "The Great Henge", CastMethod::Normal, &[]);
        assert_eq!(paid, 2, "{increaser}");
        assert_eq!(mana_value(&t, spell), 9);
    }
}

#[test]
fn geist_fueled_scarecrow_taxes_alternative_and_free_casts() {
    cr!("601.2f", "118.9d");
    ruling!(
        "Geist-Fueled Scarecrow",
        "Geist-Fueled Scarecrow’s ability applies to creature spells even if you’re casting them for an alternative cost (such as that of emerge). If you cast a creature spell without paying its mana cost, you’ll still have to pay {1}."
    );
    supported("Omniscience");
    // Omniscience: "You may cast spells from your hand without paying their mana costs."
    let mut t = main_phase(2);
    t.battlefield(P0, "Geist-Fueled Scarecrow");
    t.battlefield(P0, "Omniscience");
    let card = t.hand(P0, "Craw Wurm");
    let free = crate::r_s08_common::legal_cast_methods(&mut t, P0, card);
    assert!(free.iter().all(|m| *m == CastMethod::Normal), "{free:?}");
    t.lands(P0, "Wastes", 1);
    let methods = crate::r_s08_common::legal_cast_methods(&mut t, P0, card);
    let free = methods
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("free cast with {1}");
    t.cast(P0, card).method(free).go();
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn eidolon_of_obstruction_adds_one_mana_to_loyalty_costs() {
    cr!("118.7", "606.4", "602.2b");
    ruling!(
        "Eidolon of Obstruction",
        "Eidolon of Obstruction doesn’t replace the loyalty cost of activating a loyalty ability or cause it to cost more loyalty to activate. It just adds a mana component to the ability’s activation cost."
    );
    ruling!(
        "Eidolon of Obstruction",
        "Loyalty abilities cost just {1} more to activate, no matter how much loyalty they cost to activate."
    );
    supported("Eidolon of Obstruction");
    // P1's Eidolon; P0's Jace Beleren (3 loyalty): "+2: Each player draws a card.",
    // "−1: Target player draws a card.", "−10: ...".
    let mut t = main_phase(2);
    t.battlefield(P1, "Eidolon of Obstruction");
    let jace = t.battlefield(P0, "Jace Beleren");
    assert_eq!(t.counters(jace, counters::LOYALTY), 3);
    assert!(!can_activate(&mut t, P0, jace));
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    activate_containing(&mut t, P0, jace, "Target player draws").expect("−1 with {1}");
    assert_eq!(tapped_lands(&t, P0), 1);
    assert_eq!(t.counters(jace, counters::LOYALTY), 2);
    // A +2 also costs just {1} more and still adds two loyalty.
    let mut t = main_phase(2);
    t.battlefield(P1, "Eidolon of Obstruction");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, jace, "Each player draws").expect("+2 with {1}");
    assert_eq!(tapped_lands(&t, P0), 1);
    assert_eq!(t.counters(jace, counters::LOYALTY), 5);
}

/// `p`'s permanent `name` that names `named` as it enters.
fn naming(t: &mut TestGame, p: PlayerId, name: &str, named: &str) -> ObjectId {
    t.answer(p, DecisionKind::Name, Answer::Text(named.into()));
    t.enter(p, name)
}

#[test]
fn naming_a_card_with_a_mana_ability_stops_only_its_other_abilities() {
    cr!("605.1a", "602.5");
    ruling!(
        "Pithing Needle",
        "If you name a card that has both a mana ability and another activated ability, the mana ability can be activated but the other ability can't be activated."
    );
    ruling!(
        "Disruptor Flute",
        "If you name a card that has both a mana ability and another activated ability, the mana ability can be activated but the other ability can't be activated."
    );
    ruling!(
        "Petrified Hamlet",
        "If you name a card that has both a mana ability and another activated ability, the mana ability can be activated but the other ability can't be activated."
    );
    // Mind Stone: "{T}: Add {C}." and "{1}, {T}, Sacrifice this artifact: Draw a card."
    for stopper in ["Pithing Needle", "Disruptor Flute"] {
        supported(stopper);
        let mut t = main_phase(2);
        naming(&mut t, P1, stopper, "Mind Stone");
        let stone = t.battlefield(P0, "Mind Stone");
        t.lands(P0, "Wastes", 1);
        assert!(!can_activate_containing(&mut t, P0, stone, "Draw a card"), "{stopper}");
        assert!(can_activate_containing(&mut t, P0, stone, "Add {C}"), "{stopper}");
    }
    // Petrified Hamlet names a land card: Mutavault ("{T}: Add {C}." and "{1}: Until end
    // of turn, this land becomes a 2/2 ...").
    supported("Petrified Hamlet");
    let mut t = main_phase(2);
    naming(&mut t, P1, "Petrified Hamlet", "Mutavault");
    t.resolve_all();
    let vault = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Wastes", 1);
    assert!(!can_activate_containing(&mut t, P0, vault, "becomes"));
    assert!(can_activate_containing(&mut t, P0, vault, "Add {C}"));
}

#[test]
fn disruptor_flute_names_one_of_a_cards_names() {
    cr!("201.4", "715.4", "601.2f");
    ruling!(
        "Disruptor Flute",
        "Some cards can have alternative names, such as split cards, adventurer cards, double-faced cards, and so on. You may choose one of the two names the card can have; Disruptor Flute's abilities won't affect anything with the other name. For example, if you choose Ondu Knotmaster, Throw a Line will still cost only {W}{B} to cast."
    );
    const KNOT: &str = "Ondu Knotmaster // Throw a Line";
    supported(KNOT);
    // Named Ondu Knotmaster: the creature costs {3} more; Throw a Line doesn't.
    let mut t = main_phase(2);
    naming(&mut t, P1, "Disruptor Flute", "Ondu Knotmaster");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let (_, paid) = cast_paying(&mut t, KNOT, CastMethod::Half(1), &[obj(bears)]);
    assert_eq!(paid, 2);
    t.resolve_all();
    let mut t = main_phase(2);
    naming(&mut t, P1, "Disruptor Flute", "Ondu Knotmaster");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, KNOT);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
    // Named Throw a Line: the Adventure costs {3} more.
    let mut t = main_phase(2);
    naming(&mut t, P1, "Disruptor Flute", "Throw a Line");
    let card = t.hand(P0, KNOT);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Half(1)));
}

#[test]
fn disruptor_flute_cant_name_a_token_only_name() {
    cr!("201.3", "111.4");
    ruling!(
        "Disruptor Flute",
        "You can choose the name of any card. You can't choose the name of a token unless it's also the name of a card."
    );
    // "Food" is only a token's name: the Flute names nothing, and Food's ability still
    // works. "Ornithopter" is a card name.
    let mut t = main_phase(2);
    let flute = naming(&mut t, P1, "Disruptor Flute", "Food");
    assert_ne!(
        t.obj_now(flute).choices.card_name.as_deref(),
        Some("Food")
    );
    let food = crate::r_s02_common::create_token(&mut t, P0, "Food");
    t.lands(P0, "Wastes", 2);
    assert!(can_activate(&mut t, P0, food));
    let mut t = main_phase(2);
    let flute = naming(&mut t, P1, "Disruptor Flute", "Ornithopter");
    assert_eq!(
        t.obj_now(flute).choices.card_name.as_deref(),
        Some("Ornithopter")
    );
    let card = t.hand(P0, "Ornithopter");
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
}

#[test]
fn god_pharaohs_statue_drains_the_opposing_team_twice_in_two_headed_giant() {
    cr!("810.8a", "810.9");
    ruling!(
        "God-Pharaoh's Statue",
        "In a Two-Headed Giant game, the last ability of God-Pharaoh's Statue causes the opposing team to lose 2 life."
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.set_step(P0, Step::PostcombatMain);
    t.battlefield(P0, "God-Pharaoh's Statue");
    let life = t.life(P2);
    let ours = t.life(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P2), life - 2);
    assert_eq!(t.life(P3), life - 2);
    assert_eq!(t.life(P0), ours);
}

#[test]
fn noncreature_and_nonartifact_taxes_apply_to_your_own_spells() {
    cr!("601.2f", "118.7");
    ruling!(
        "Thalia, Guardian of Thraben",
        "Thalia's ability affects each spell that's not a creature spell, including your own."
    );
    ruling!(
        "Vryn Wingmare",
        "The ability affects each spell that's not a creature spell, including your own. Creature spells with another type, such as artifact creature spells, aren't affected."
    );
    ruling!(
        "Lodestone Golem",
        "The ability affects each spell that's not an artifact spell, including your own."
    );
    // (taxer, spell taxed, its cost, spell not taxed)
    for (taxer, taxed, cost, untaxed) in [
        ("Thalia, Guardian of Thraben", "Shock", 2, "Grizzly Bears"),
        ("Vryn Wingmare", "Shock", 2, "Ornithopter"),
        ("Lodestone Golem", "Shock", 2, "Mind Stone"),
    ] {
        supported(taxer);
        let mut t = main_phase(2);
        t.battlefield(P0, taxer);
        t.lands(P0, "Mountain", 1);
        let card = t.hand(P0, taxed);
        assert!(!can_cast(&mut t, P0, card, CastMethod::Normal), "{taxer}");
        t.lands(P0, "Wastes", cost - 1);
        let (_, paid) = cast_paying(&mut t, taxed, CastMethod::Normal, &[Entity::Player(P1)]);
        assert_eq!(paid, cost, "{taxer}");
        t.resolve_all();
        // The untaxed spell can be cast for its mana cost.
        let mut t = main_phase(2);
        t.battlefield(P0, taxer);
        crate::r_s01_common::give_mana_for(&mut t, P0, untaxed);
        let card = t.hand(P0, untaxed);
        assert!(can_cast(&mut t, P0, card, CastMethod::Normal), "{taxer}: {untaxed}");
    }
}

#[test]
fn lodestone_golem_changes_the_total_cost_only() {
    cr!("601.2f", "202.3", "107.3a");
    ruling!(
        "Lodestone Golem",
        "The ability affects the total cost of each nonartifact spell, but it doesn't change that spell's mana cost or mana value. The extra mana also doesn't change how many times you kicked the spell or what the value of X is (if applicable)."
    );
    // Endless One ({X}, "enters with X +1/+1 counters") with X = 2 costs {3}: it's still a
    // 2/2 with mana value 2 on the stack.
    let mut t = main_phase(2);
    t.battlefield(P1, "Lodestone Golem");
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Endless One");
    let spell = t.cast(P0, card).x(2).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(mana_value(&t, spell), 2);
    t.resolve_all();
    assert_eq!(t.pt(card), (2, 2));
    // Burst Lightning ({R}, kicker {4}) kicked costs {5}{R}: kicked once, 4 damage.
    let mut t = main_phase(2);
    t.battlefield(P1, "Lodestone Golem");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 5);
    let life = t.life(P1);
    let card = t.hand(P0, "Burst Lightning");
    let spell = t.cast(P0, card).kicked(true).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 6);
    assert_eq!(mana_value(&t, spell), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), life - 4);
}

#[test]
fn generic_reductions_dont_reduce_colored_requirements() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Geyser Drake",
        "Geyser Drake’s last ability can’t reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that spell’s cost."
    );
    ruling!(
        "Umori, the Collector",
        "Effects that reduce the generic mana cost of a spell (such as that of Umori) can't reduce that spell's colored mana requirements."
    );
    ruling!(
        "Cheering Fanatic",
        "Effects that reduce the generic mana cost of a spell can’t reduce that spell’s colored mana requirements."
    );
    ruling!(
        "Cunning Nightbonder",
        "Cunning Nightbonder's cost reduction applies only to generic mana in a spell's cost. It can't reduce a spell's colored mana requirements. For example, your first Cunning Nightbonder won't normally reduce the cost of your second since it has no generic mana in its cost."
    );
    ruling!(
        "Elderwood Scion",
        "Elderwood Scion’s cost-reduction ability can’t reduce the colored mana requirement of a spell."
    );
    // Geyser Drake: during P1's turn, Counterspell ({U}{U}) still costs {U}{U}.
    supported("Geyser Drake");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Geyser Drake");
    t.lands(P0, "Island", 2);
    let bolt = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    let shock = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    let (_, paid) = cast_paying(&mut t, "Counterspell", CastMethod::Normal, &[obj(shock)]);
    assert_eq!(paid, 2);
    // Umori, choosing creature: Llanowar Elves ({G}) still costs {G}.
    supported("Umori, the Collector");
    let mut t = main_phase(2);
    let idx = CardType::ALL
        .iter()
        .position(|c| *c == CardType::Creature)
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(idx));
    t.enter(P0, "Umori, the Collector");
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(!can_cast(&mut t, P0, elves, CastMethod::Normal));
    t.lands(P0, "Forest", 1);
    assert!(can_cast(&mut t, P0, elves, CastMethod::Normal));
    // Cunning Nightbonder: a second Nightbonder (flash, {U/B}{U/B}) still costs two.
    supported("Cunning Nightbonder");
    let mut t = main_phase(2);
    t.battlefield(P0, "Cunning Nightbonder");
    t.lands(P0, "Island", 2);
    let (_, paid) = cast_paying(&mut t, "Cunning Nightbonder", CastMethod::Normal, &[]);
    assert_eq!(paid, 2);
    // Elderwood Scion: Giant Growth ({G}) targeting it still costs {G}.
    supported("Elderwood Scion");
    let mut t = main_phase(2);
    let scion = t.battlefield(P0, "Elderwood Scion");
    let growth = t.hand(P0, "Giant Growth");
    assert!(!can_cast(&mut t, P0, growth, CastMethod::Normal));
    t.lands(P0, "Forest", 1);
    let (_, paid) = cast_paying(&mut t, "Giant Growth", CastMethod::Normal, &[obj(scion)]);
    assert_eq!(paid, 1);
    // Cheering Fanatic: attacking, it names Llanowar Elves: still {G}.
    supported("Cheering Fanatic");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let fan = t.battlefield(P0, "Cheering Fanatic");
    t.answer(P0, DecisionKind::Name, Answer::Text("Llanowar Elves".into()));
    crate::r_s01_common::attack_with(&mut t, &[(fan, Entity::Player(P1))]);
    t.resolve_all();
    t.set_step(P0, Step::PostcombatMain);
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(!can_cast(&mut t, P0, elves, CastMethod::Normal));
    // Grizzly Bears named instead would cost {G}: checked with a second Fanatic game.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let fan = t.battlefield(P0, "Cheering Fanatic");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    crate::r_s01_common::attack_with(&mut t, &[(fan, Entity::Player(P1))]);
    t.resolve_all();
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn doc_aurlock_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d", "702.34a");
    ruling!(
        "Doc Aurlock, Grizzled Genius",
        "Doc Aurlock’s abilities can’t reduce the amount of colored mana you pay for a spell or ability. It reduces only the generic mana component of that spell or ability’s cost."
    );
    supported("Doc Aurlock, Grizzled Genius");
    // Think Twice's flashback {2}{U} from the graveyard costs {U}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Doc Aurlock, Grizzled Genius");
    let card = t.graveyard(P0, "Think Twice");
    let fb = CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback);
    assert!(!can_cast(&mut t, P0, card, fb.clone()));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, card, fb.clone()));
    t.cast(P0, card).method(fb).go();
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn because_i_have_willed_it_reduces_only_generic_mana() {
    cr!("601.2f", "314.4");
    ruling!(
        "Because I Have Willed It",
        "Effects that reduce the generic mana cost of a spell can’t reduce that spell’s colored mana requirements."
    );
    // (Its abandon trigger isn't supported; its cost reduction is.)
    // The ongoing scheme in P0's command zone: "Spells you cast cost {1} less to cast."
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Archenemy,
            ..Default::default()
        },
    );
    t.set_step(P0, Step::PrecombatMain);
    t.command(P0, "Because I Have Willed It");
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(!can_cast(&mut t, P0, elves, CastMethod::Normal));
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    assert!(can_cast(&mut t, P0, elves, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn elderwood_scion_reduces_additional_costs_too() {
    cr!("601.2f", "118.8", "702.33a");
    ruling!(
        "Elderwood Scion",
        "Elderwood Scion’s cost-reduction ability can apply to any cost of a spell, including additional or alternative costs."
    );
    // Burst Lightning ({R}, kicker {4}) kicked, targeting P0's Elderwood Scion: {4}{R} −
    // {2} = {2}{R}.
    let mut t = main_phase(2);
    let scion = t.battlefield(P0, "Elderwood Scion");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Burst Lightning");
    t.cast(P0, card).kicked(true).target(obj(scion)).go();
    assert_eq!(tapped_lands(&t, P0), 3);
}

#[test]
fn cunning_nightbonder_doesnt_apply_to_itself_as_a_spell() {
    cr!("601.2f", "113.6");
    ruling!(
        "Cunning Nightbonder",
        "Cunning Nightbonder's last ability doesn't apply to itself while it's a spell."
    );
    // It has flash; with one land it can't be cast (it would need its own reduction,
    // which couldn't apply to colored mana anyway — and it can be countered).
    let mut t = main_phase(2);
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Cunning Nightbonder");
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    t.lands(P0, "Island", 1);
    let spell = t.cast(P0, card).go();
    // "can't be countered" doesn't apply to it either.
    crate::r_p146_common::counter(&mut t, spell);
    assert!(t.in_graveyard(P0, "Cunning Nightbonder"));
}

#[test]
fn thryx_spells_cant_be_countered_but_counterspells_other_effects_happen() {
    cr!("101.2", "608.2b", "701.6b");
    ruling!(
        "Thryx, the Sudden Storm",
        "A spell or ability that counters spells can still target spells you control with mana value 5 or greater. When that spell or ability resolves, your spell won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    supported("Thryx, the Sudden Storm");
    supported("Exclude");
    // P1's Exclude ("Counter target creature spell. Draw a card.") targets P0's Craw
    // Wurm (mana value 6).
    let mut t = main_phase(2);
    t.battlefield(P0, "Thryx, the Sudden Storm");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 3);
    let wurm = t.hand(P0, "Craw Wurm");
    let spell = t.cast(P0, wurm).go();
    t.lands(P1, "Island", 1);
    t.lands(P1, "Wastes", 2);
    let ex = t.hand(P1, "Exclude");
    let hand = t.hand_size(P1);
    t.cast(P1, ex).target(obj(spell)).go();
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.hand_size(P1), hand - 1 + 1);
    assert!(t.in_graveyard(P1, "Exclude"));
}

#[test]
fn semblance_anvil_and_umori_use_card_types_only() {
    cr!("205.2a", "205.4a", "205.3a");
    ruling!(
        "Semblance Anvil",
        "Artifact, battle, creature, enchantment, instant, kindred, planeswalker, and sorcery are card types."
    );
    ruling!(
        "Umori, the Collector",
        "Artifact, creature, enchantment, instant, planeswalker, and sorcery are card types. Supertypes (such as legendary) and subtypes (such as Ooze) are not. "
    );
    supported("Semblance Anvil");
    // Semblance Anvil imprinting Ornithopter (an artifact creature card): creature spells
    // and artifact spells each cost {2} less; instants don't.
    let mut t = main_phase(2);
    let thopter = t.hand(P0, "Ornithopter");
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Semblance Anvil");
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    let stone = t.hand(P0, "Mind Stone");
    assert!(can_cast(&mut t, P0, stone, CastMethod::Normal));
    t.lands(P0, "Forest", 1);
    let wurm = t.hand(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, wurm, CastMethod::Normal));
    let shock = t.hand(P0, "Lava Spike");
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal));
    // Umori offers card types only — no supertypes or subtypes.
    let mut t = main_phase(2);
    let from = t.asked().len();
    t.enter(P0, "Umori, the Collector");
    let options: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1);
    for w in ["Artifact", "Creature", "Enchantment", "Instant", "Planeswalker", "Sorcery"] {
        assert!(options[0].iter().any(|o| o.eq_ignore_ascii_case(w)), "{w}: {options:?}");
    }
    for w in ["Legendary", "Ooze", "Basic", "Snow"] {
        assert!(!options[0].iter().any(|o| o.eq_ignore_ascii_case(w)), "{w}");
    }
}

#[test]
fn it_that_heralds_the_end_uses_x_for_mana_value() {
    cr!("202.3e", "601.2f");
    ruling!(
        "It That Heralds the End",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell's mana value."
    );
    supported("It That Heralds the End");
    // "Colorless spells you cast with mana value 7 or greater cost {1} less." Endless One
    // ({X}, colorless) with X = 7 costs 6; with X = 6, 6.
    for (x, paid) in [(7, 6), (6, 6)] {
        let mut t = main_phase(2);
        t.battlefield(P0, "It That Heralds the End");
        t.lands(P0, "Wastes", 7);
        let card = t.hand(P0, "Endless One");
        t.cast(P0, card).x(x).go();
        assert_eq!(tapped_lands(&t, P0), paid, "X = {x}");
    }
}

#[test]
fn gavi_counts_cycling_earlier_in_the_turn() {
    cr!("702.29a", "118.9");
    ruling!(
        "Gavi, Nest Warden",
        "Gavi’s first ability considers the entire turn. If you cycle a card then have Gavi come under your control, you can’t pay {0} to cycle your second card."
    );
    supported("Gavi, Nest Warden");
    use crate::r_s04_common::{can_cycle, cycle};
    // Desert Cerodon: cycling {R}.
    let mut t = main_phase(2);
    let first = t.hand(P0, "Desert Cerodon");
    let second = t.hand(P0, "Desert Cerodon");
    t.lands(P0, "Mountain", 1);
    cycle(&mut t, P0, first, 0).expect("cycle");
    t.resolve_all();
    t.battlefield(P0, "Gavi, Nest Warden");
    assert!(!can_cycle(&mut t, P0, second));
    // Without the earlier cycle, Gavi lets the first one cycle for {0}.
    let mut t = main_phase(2);
    let first = t.hand(P0, "Desert Cerodon");
    t.battlefield(P0, "Gavi, Nest Warden");
    assert!(can_cycle(&mut t, P0, first));
}
