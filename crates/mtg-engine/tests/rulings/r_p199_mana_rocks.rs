//! Rulings batch P199 — utility artifacts and mana rocks: The Golden Throne's
//! loss-replacement (CR 104.3, 614), The Great Henge's cost reduction (CR 601.2f), Static
//! Net's "until" exile (CR 610.3), Powerstone tokens (CR 111.10), Karn's "pay any amount of
//! mana", Locus of Enlightenment's ability copies (CR 707.10), Visions of Phyrexia, and the
//! animated artifacts Guardian Idol and Rabid Wombat's Auras (CR 613).

use crate::r_p079_common::back_face;
use crate::r_p194_common::{assert_total_cost, mana};
use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Whether P0 could play the card `id` (followed across zone changes) now.
fn playable(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let c = t.g.current(id);
    t.g.legal_actions(P0).iter().any(|a| match a {
        Action::PlayLand { card } => *card == c,
        Action::Cast { card, .. } => *card == c,
        _ => false,
    })
}

#[test]
fn rabid_wombat_shrinks_as_soon_as_an_aura_leaves() {
    cr!("613.4c", "611.3a");
    ruling!(
        "Rabid Wombat",
        "If an Aura is removed from the Wombat, its power and toughness change immediately."
    );
    let mut t = TestGame::new(2);
    let wombat = t.battlefield(P0, "Rabid Wombat");
    let aura = t.battlefield(P0, "Pacifism");
    assert!(t.g.attach(aura, obj(wombat)));
    t.g.recompute();
    assert_eq!(t.pt(wombat), (2, 3));
    move_to(&mut t, aura, Zone::Graveyard(P0));
    assert_eq!(t.pt(wombat), (0, 1));
}

#[test]
fn twinning_glass_casts_one_card_as_it_resolves() {
    cr!("608.2g", "601.2b");
    ruling!(
        "Twinning Glass",
        "Activating Twinning Glass's ability allows you to cast a single card from your hand as it resolves. It doesn't create a continuous effect, and it doesn't let you cast multiple cards for free."
    );
    let mut t = TestGame::new(2);
    let glass = t.battlefield(P0, "Twinning Glass");
    t.lands(P0, "Wastes", 1);
    // A Shock was cast this turn.
    let first = t.hand(P1, "Shock");
    mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, first).target(Entity::Player(P0)).go();
    t.resolve_all();
    let a = t.hand(P0, "Shock");
    let b = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, glass, 0, &[]).unwrap();
    t.resolve();
    // One Shock is on the stack, cast for free; the other stays in hand, not castable
    // without mana.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert!(!playable(&mut t, b));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn twinning_glass_allows_additional_costs_but_not_alternative_ones() {
    cr!("118.9a", "118.8", "601.2b");
    ruling!(
        "Twinning Glass",
        "You can't pay any alternative costs (such as those from evoke or morph) when casting the card. On the other hand, if the card has additional costs (such as those from kicker or buyback), you may pay those."
    );
    supported("Burst Lightning");
    // Kicker: Burst Lightning cast for free with its kicker paid deals 4 damage.
    let mut t = TestGame::new(2);
    let glass = t.battlefield(P0, "Twinning Glass");
    t.lands(P0, "Wastes", 5);
    let first = t.hand(P1, "Burst Lightning");
    mana(&mut t, P1, ManaType::R, 1);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P1, first).target(Entity::Player(P0)).go();
    t.resolve_all();
    let burst = t.hand(P0, "Burst Lightning");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(burst)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, glass, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(untapped_lands(&t, P0), 0);
    // Evoke: Mulldrifter cast this way is cast normally (it stays on the battlefield). No way
    // to cast it with evoke is offered, though P0 would take one.
    let mut t = TestGame::new(2);
    let glass = t.battlefield(P0, "Twinning Glass");
    t.lands(P0, "Wastes", 1);
    t.set_step(P1, Step::PrecombatMain);
    let first = t.hand(P1, "Mulldrifter");
    mana(&mut t, P1, ManaType::U, 1);
    mana(&mut t, P1, ManaType::C, 4);
    t.cast(P1, first).go();
    t.resolve_all();
    t.set_step(P1, Step::End);
    let drifter = t.hand(P0, "Mulldrifter");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(drifter)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, glass, 0, &[]).unwrap();
    let from = t.asked().len();
    t.resolve_all();
    let offered: Vec<String> = t.asked()[from..]
        .iter()
        .flat_map(|(_, d)| match d {
            Decision::ChooseCastingMethod { options, .. }
            | Decision::ChooseOption { options, .. } => options.clone(),
            _ => vec![],
        })
        .collect();
    assert!(offered.iter().all(|m| !m.contains("Evoke")), "{offered:?}");
    assert!(t.on_battlefield(drifter));
}

#[test]
fn thrumming_stone_ripple_may_cast_either_half_of_a_split_card() {
    cr!("702.60a", "709.3");
    ruling!(
        "Thrumming Stone",
        "If you cast half of a split card (for example, Hit) and reveal another one of those split cards when the ripple ability resolves (for example, Hit // Run), you may cast either half of that split card (Hit or Run) without paying its mana cost."
    );
    for (half, sacrificed) in [(0usize, 2usize), (1, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Thrumming Stone");
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Grizzly Bears");
        let revealed = stack_library(&mut t, P0, &["Hit // Run"])[0];
        let hit = t.hand(P0, "Hit // Run");
        mana(&mut t, P0, ManaType::B, 1);
        mana(&mut t, P0, ManaType::R, 1);
        mana(&mut t, P0, ManaType::C, 1);
        // Cast Hit; ripple reveals Hit // Run: either half may be cast.
        t.answer_yes(P0, true);
        t.cast(P0, hit)
            .method(CastMethod::Half(0))
            .target(Entity::Player(P1))
            .go();
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Option, Answer::Index(half));
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let from = t.asked().len();
        t.resolve();
        let offered: Vec<Vec<String>> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseOption { options, .. } => Some(options.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            offered,
            vec![vec!["Cast Hit".to_string(), "Cast Run".to_string()]]
        );
        assert_eq!(t.zone(revealed), Zone::Stack);
        assert_eq!(t.g.player(P0).mana_pool.total(), 0);
        t.resolve_all();
        assert_eq!(t.zone(revealed), Zone::Graveyard(P0));
        // Hit twice: both Bears; Hit and Run: one.
        assert_eq!(
            2 - t.named_on_battlefield("Grizzly Bears").len(),
            sacrificed
        );
        assert_eq!(t.life(P1), 20 - 2 * sacrificed as i32);
    }
}

#[test]
fn shivan_gorge_damage_is_colorless() {
    cr!("105.2c", "609.7c");
    ruling!("Shivan Gorge", "The damage has no color. It is not red.");
    supported("Sphere of Law");
    // Sphere of Law ("If a red source would deal damage to you, prevent 2 of that damage")
    // doesn't prevent the damage the Gorge deals, though {R} was spent on the ability.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Law");
    let gorge = t.battlefield(P0, "Shivan Gorge");
    t.lands(P0, "Mountain", 3);
    t.activate(P0, gorge, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // A red source's damage is prevented.
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

/// Activates Guardian Idol's "becomes a 2/2 Golem artifact creature" with mana in pool.
fn animate_idol(t: &mut TestGame, idol: ObjectId) {
    mana(t, P0, ManaType::C, 2);
    t.activate(P0, idol, 1, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn guardian_idol_reactivated_overrides_setting_effects_but_not_modifications() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Guardian Idol",
        "Activating the last ability of Guardian Idol while it's already a creature will override any effects that set its power or toughness to a specific value. Effects that modify power or toughness without setting them to a specific value will continue to apply."
    );
    supported("Diminish");
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Guardian Idol");
    animate_idol(&mut t, idol);
    assert_eq!(t.pt(idol), (2, 2));
    // Giant Growth (+3/+3), then Diminish (base 1/1): 4/4.
    let growth = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, growth).target(idol).go();
    t.resolve_all();
    let dim = t.hand(P0, "Diminish");
    mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, dim).target(idol).go();
    t.resolve_all();
    assert_eq!(t.pt(idol), (4, 4));
    // Again: base 2/2 overrides Diminish; Giant Growth still applies.
    animate_idol(&mut t, idol);
    assert_eq!(t.pt(idol), (5, 5));
}

#[test]
fn guardian_idol_keeps_its_counters_while_not_a_creature() {
    cr!("122.1a", "613.4c");
    ruling!(
        "Guardian Idol",
        "If Guardian Idol has any +1/+1 counters on it, those counters will remain on it after it stops being a creature. Those counters will have no effect as long as Guardian Idol isn't a creature, but they will apply again if it later becomes a creature."
    );
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Guardian Idol");
    animate_idol(&mut t, idol);
    t.g.add_counters(obj(idol), counters::PLUS1, 1, None);
    t.g.recompute();
    assert_eq!(t.pt(idol), (3, 3));
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(idol).is(CardType::Creature));
    assert_eq!(t.counters(idol, counters::PLUS1), 1);
    t.advance_to(P0, Step::PrecombatMain);
    animate_idol(&mut t, idol);
    assert_eq!(t.pt(idol), (3, 3));
}

#[test]
fn gauntlet_of_power_leaving_can_make_marked_damage_lethal() {
    cr!("704.5g", "120.6");
    ruling!(
        "Gauntlet of Power",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a creature of the chosen color may become lethal if Gauntlet of Power leaves the battlefield during that turn."
    );
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let gauntlet = t.enter(P0, "Gauntlet of Power");
    t.settle();
    assert_eq!(t.obj_now(gauntlet).choices.color, Some(Color::Red));
    let goblin = t.battlefield(P1, "Raging Goblin");
    assert_eq!(t.pt(goblin), (2, 2));
    crate::r_p194_common::damage(&mut t, gauntlet, obj(goblin), 1);
    assert!(t.on_battlefield(goblin));
    move_to(&mut t, gauntlet, Zone::Graveyard(P0));
    assert!(!t.on_battlefield(goblin));
}

#[test]
fn gauntlet_of_power_without_a_chosen_color_does_nothing() {
    cr!("607.2d");
    ruling!(
        "Gauntlet of Power",
        "If Gauntlet of Power is somehow on the battlefield with no color chosen for its first ability, no creatures get +1/+1 and its last ability never triggers. It still won't affect colorless creatures or basic lands tapped for colorless mana."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gauntlet of Power");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thopter = t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(thopter), (0, 2));
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, forest, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    // With a chosen color (green), a Forest adds an extra {G}; Wastes don't.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.enter(P0, "Gauntlet of Power");
    t.settle();
    let wastes = t.battlefield(P0, "Wastes");
    t.activate(P0, wastes, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, forest, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 2);
}

fn karn(t: &mut TestGame) -> ObjectId {
    let k = t.battlefield(P0, "Karn, Living Legacy");
    t.g.add_counters(obj(k), counters::LOYALTY, 10, None);
    k
}

#[test]
fn karn_chooses_how_much_mana_to_pay_as_the_ability_resolves() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Karn, Living Legacy",
        "For Karn’s second loyalty ability, you don’t choose how much mana to pay until the ability resolves."
    );
    ruling!(
        "Karn, Living Legacy",
        "If you look at one or more cards this way, you must put a card into your hand. You can’t choose not to put one into your hand."
    );
    let mut t = TestGame::new(2);
    let k = karn(&mut t);
    t.lands(P0, "Wastes", 2);
    let top = stack_library(&mut t, P0, &["Grizzly Bears", "Shock"]);
    let from = t.asked().len();
    t.activate(P0, k, 1, &[]).unwrap();
    // Nothing asked about the amount yet.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseX { .. })));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseX { .. })));
    assert_eq!(untapped_lands(&t, P0), 0);
    // A card went to the hand even though the answer chose none.
    assert_eq!(t.hand_size(P0), hand + 1);
    let in_hand = top
        .iter()
        .filter(|c| matches!(t.zone(**c), Zone::Hand(_)))
        .count();
    assert_eq!(in_hand, 1);
}

#[test]
fn karn_paying_no_mana_looks_at_nothing() {
    cr!("608.2c", "107.1c");
    ruling!(
        "Karn, Living Legacy",
        "If you don’t pay any mana, you won’t look at any cards or put any into your hand."
    );
    let mut t = TestGame::new(2);
    let k = karn(&mut t);
    t.lands(P0, "Wastes", 2);
    let lib = t.library_size(P0);
    t.activate(P0, k, 1, &[]).unwrap();
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(untapped_lands(&t, P0), 2);
}

#[test]
fn karns_emblem_can_tap_summoning_sick_artifact_creatures() {
    cr!("302.6", "114.4");
    ruling!(
        "Karn, Living Legacy",
        "Karn’s emblem has an activated ability that allows you to tap untapped artifacts to pay its cost. Notably, you may tap artifact creatures that entered the battlefield this turn to pay this cost, even if they don’t have haste."
    );
    let mut t = TestGame::new(2);
    let k = karn(&mut t);
    t.activate(P0, k, 2, &[]).unwrap();
    t.resolve_all();
    let emblem =
        t.g.command
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).controller == P0)
            .expect("emblem");
    let thopter = t.battlefield_sick(P0, "Ornithopter");
    t.answer_choose(P0, &[obj(thopter)]);
    t.activate(P0, emblem, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(thopter).tapped);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn static_net_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3b", "603.3");
    ruling!(
        "Static Net",
        "If Static Net leaves the battlefield before its first ability resolves, the target permanent won't be exiled."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let net = t.enter(P0, "Static Net");
    t.settle();
    move_to(&mut t, net, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn static_net_returns_the_card_immediately_after_it_leaves() {
    cr!("610.3", "610.3c");
    ruling!(
        "Static Net",
        "Static Net's first ability creates two one-shot effects: one that exiles the permanent when the ability resolves, and another that returns the exiled card to the battlefield immediately after Static Net leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let net = t.enter(P0, "Static Net");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // The card comes back (under its owner's control) as the Net leaves, with nothing on
    // the stack.
    move_to(&mut t, net, Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn static_net_returns_the_card_when_its_owner_leaves_the_game() {
    cr!("800.4a", "610.3");
    ruling!(
        "Static Net",
        "In a multiplayer game, if Static Net's owner leaves the game, the exiled card will return to the battlefield."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Static Net");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn the_golden_throne_is_gone_before_the_life_loss_is_checked() {
    cr!("704.3", "704.5a", "614.1");
    ruling!(
        "The Golden Throne",
        "If a spell or ability both destroys The Golden Throne and deals enough damage to you to reduce your life total to zero or less, you will still lose the game."
    );
    let spell = custom_card(
        "Throne Breaker",
        "Sorcery",
        "{0}",
        None,
        "Destroy target artifact. ~ deals 20 damage to target player.",
    );
    // Only the damage: the Throne is exiled and P0's life becomes 1.
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, "The Golden Throne");
    crate::r_p194_common::damage(&mut t, throne, Entity::Player(P0), 20);
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.zone(throne), Zone::Exile);
    // Both at once: P0 loses.
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, "The Golden Throne");
    t.set_step(P1, Step::PrecombatMain);
    let s = t.custom(P1, spell, Zone::Hand(P1));
    t.cast(P1, s).target(throne).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert!(t.has_lost(P0));
}

#[test]
fn the_golden_throne_doesnt_apply_when_you_cant_lose() {
    cr!("104.3b", "614.1");
    ruling!(
        "The Golden Throne",
        "If an effect says that you can't lose the game, The Golden Throne's first ability doesn't apply."
    );
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, "The Golden Throne");
    t.battlefield(P0, "Platinum Angel");
    t.g.players[P0.idx()].life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
    assert!(t.on_battlefield(throne));
    assert_eq!(t.life(P0), 0);
}

#[test]
fn the_golden_throne_doesnt_apply_when_an_opponent_wins() {
    cr!("104.2b", "104.1");
    ruling!(
        "The Golden Throne",
        "If an effect states that an opponent wins the game, The Golden Throne's effect doesn't apply."
    );
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, "The Golden Throne");
    t.battlefield(P1, "Felidar Sovereign");
    t.g.players[P1.idx()].life = 40;
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert!(t.on_battlefield(throne));
}

#[test]
fn the_aetherspark_attached_while_attacked_is_still_attacked() {
    cr!("506.4", "508.1b");
    ruling!(
        "The Aetherspark",
        "If an effect causes The Aetherspark to become attached to a creature while it is being attacked, it will continue to be attacked."
    );
    let mut t = TestGame::new(2);
    let spark = t.battlefield(P0, "The Aetherspark");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let attacker = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, obj(spark))]);
    assert!(t.g.attach(spark, obj(mine)));
    t.g.recompute();
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(spark, counters::LOYALTY), 1);
}

/// A Locus of Enlightenment on P0's battlefield.
fn locus(t: &mut TestGame) -> ObjectId {
    back_face(t, P0, "The Enigma Jewel // Locus of Enlightenment")
}

#[test]
fn locus_copies_have_the_same_x_and_mode() {
    cr!("707.10", "700.2");
    ruling!(
        "The Enigma Jewel // Locus of Enlightenment",
        "If the ability that's copied has an X whose value was determined as it was cast, the copy has the same value of X."
    );
    ruling!(
        "The Enigma Jewel // Locus of Enlightenment",
        "If the ability that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode or modes. You can't choose different ones."
    );
    supported("Atalya, Samite Master");
    let mut t = TestGame::new(2);
    locus(&mut t);
    let atalya = t.battlefield(P0, "Atalya, Samite Master");
    mana(&mut t, P0, ManaType::W, 3);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let from = t.asked().len();
    t.activate(P0, atalya, 0, &[]).unwrap();
    t.resolve_all();
    // Gain 3 life, twice.
    assert_eq!(t.life(P0), 26);
    // The mode and X were chosen once, for the original: none for the copy.
    let asked = t.asked();
    let modes = asked[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .count();
    let xs = asked[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseX { .. }))
        .count();
    assert_eq!((modes, xs), (1, 1));
}

#[test]
fn locus_copy_resolves_first_isnt_activated_and_may_change_targets() {
    cr!("707.10", "707.10c", "602.2");
    ruling!(
        "The Enigma Jewel // Locus of Enlightenment",
        "The copy is created on the stack, so it's not \"activated.\""
    );
    ruling!(
        "The Enigma Jewel // Locus of Enlightenment",
        "The copy of the activated ability created by Locus of Enlightenment's triggered ability resolves before the original ability."
    );
    ruling!(
        "The Enigma Jewel // Locus of Enlightenment",
        "The copy will have the same targets as the ability it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them."
    );
    let mut t = TestGame::new(2);
    locus(&mut t);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let a = t.battlefield(P1, "Raging Goblin");
    t.activate(P0, pyro, 0, &[obj(a)]).unwrap();
    t.settle();
    // The ability and Locus's trigger.
    assert_eq!(t.stack_len(), 2);
    // The copy: new target P1.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    // Only one copy (the copy didn't trigger Locus again); it's on top.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(a));
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    // Keeping the targets: both hit the same one.
    let mut t = TestGame::new(2);
    locus(&mut t);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn visions_of_phyrexia_card_is_playable_after_the_end_step_trigger() {
    cr!("611.2a", "603.4");
    ruling!(
        "Visions of Phyrexia",
        "If the card you exiled with the first ability is an instant, you may play it during your end step after Visions of Phyrexia's last ability has already resolved and created a Powerstone token."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Visions of Phyrexia");
    let bolt = t.library_top(P0, "Lightning Bolt");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    let stones = with_subtype(&t, P0, "Powerstone").len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Powerstone").len(), stones + 1);
    t.lands(P0, "Mountain", 1);
    assert!(playable(&mut t, bolt));
    let b = t.g.current(bolt);
    t.cast(P0, b).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn visions_of_phyrexia_doesnt_trigger_after_any_card_is_played_from_exile() {
    cr!("603.4");
    ruling!(
        "Visions of Phyrexia",
        "If you played a card from exile for any reason, the ability won't trigger, even if it wasn't the card you exiled with the first ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Visions of Phyrexia");
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.lands(P0, "Mountain", 5);
    t.cast(P0, giant)
        .method(CastMethod::Half(1))
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    let g = t.g.current(giant);
    assert_eq!(t.zone(g), Zone::Exile);
    t.cast(P0, g).go();
    t.resolve_all();
    assert!(t.on_battlefield(g));
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Powerstone").len(), 0);
}

#[test]
fn visions_of_phyrexia_card_follows_timing_rules_and_costs() {
    cr!("601.2", "302.1");
    ruling!(
        "Visions of Phyrexia",
        "You must follow all normal timing rules for a card you play using Visions of Phyrexia's first ability and, if it's a spell, you must pay its costs to cast it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Visions of Phyrexia");
    let bears = t.library_top(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    t.lands(P0, "Forest", 2);
    // Not during the upkeep (a creature spell).
    assert!(!playable(&mut t, bears));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(playable(&mut t, bears));
    let b = t.g.current(bears);
    t.cast(P0, b).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn the_great_henge_draws_even_if_the_creature_left() {
    cr!("609.3", "608.2c");
    ruling!(
        "The Great Henge",
        "Once The Great Henge's last ability has triggered, you'll draw a card even if you can't put a +1/+1 counter on the creature for some reason (most likely because it has left the battlefield)."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Great Henge");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, bears, Zone::Hand(P0));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn the_great_henge_is_cast_without_priority_in_between() {
    cr!("601.2", "601.2f", "117.1");
    ruling!(
        "The Great Henge",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for."
    );
    // P0 casts it with their priority, through the game loop: the one action proposes it,
    // determines its cost ({1}{G}{G} with Colossal Dreadmaw) and pays it, and no player
    // (P1 holds a Shock and a Mountain) is asked anything in between.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colossal Dreadmaw");
    let henge = t.hand(P0, "The Great Henge");
    t.lands(P0, "Forest", 3);
    t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.g.turn.passes = 0;
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: henge,
            method: CastMethod::Normal,
        }),
    );
    let from = t.asked().len();
    t.g.advance();
    let asked = t.asked();
    assert_eq!(t.zone(henge), Zone::Stack);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(
        asked[from..].iter().all(|(p, _)| *p == P0),
        "{:?}",
        &asked[from..]
    );
    let priority = asked[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
        .count();
    assert_eq!(priority, 1);
    // P0 keeps priority; once they pass, P1 may respond to the spell, already paid for.
    t.g.advance();
    assert_eq!(t.g.turn.priority, Some(P1));
    assert_eq!(t.zone(henge), Zone::Stack);
}

#[test]
fn the_great_henge_reduces_only_generic_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "The Great Henge",
        "The cost reduction ability reduces only the generic mana in The Great Henge's cost. The colored mana must still be paid."
    );
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Colossal Dreadmaw");
    };
    assert_total_cost(&setup, P0, "The Great Henge", &[(ManaType::G, 2)], 1);
    // A 9-power creature: still {G}{G}; colorless mana can't pay it.
    let big = |t: &mut TestGame| {
        t.battlefield(P0, "Colossal Dreadmaw");
        t.battlefield(P0, "Colossal Dreadmaw");
        let d = t.named_on_battlefield("Colossal Dreadmaw")[0];
        t.g.add_counters(Entity::Object(d), counters::PLUS1, 3, None);
    };
    assert_total_cost(&big, P0, "The Great Henge", &[(ManaType::G, 2)], 0);
    assert!(!crate::r_p194_common::casts_for(
        &big,
        P0,
        "The Great Henge",
        &[],
        9
    ));
}

#[test]
fn the_great_henge_uses_the_power_after_it_moves_to_the_stack() {
    cr!("601.2a", "601.2f");
    ruling!(
        "The Great Henge",
        "The first step of casting a spell is to move it to the stack. If this causes the greatest power among creatures you control to change, that new power will be used to determine the cost reduction."
    );
    // Maro's power is the number of cards in P0's hand: five with The Great Henge, four
    // once it's on the stack: {3}{G}{G}.
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Maro");
        for _ in 0..4 {
            t.hand(P0, "Wastes");
        }
    };
    assert_total_cost(&setup, P0, "The Great Henge", &[(ManaType::G, 2)], 3);
}

#[test]
fn powerstone_tokens_are_predefined_and_their_mana_pays_for_abilities() {
    cr!("111.10", "106.6");
    ruling!(
        "Excavation Explosion",
        "Powerstone tokens are a kind of predefined token. Each one has the artifact subtype \"Powerstone\" and the ability \"{T}: Add {C}. This mana can't be spent to cast a nonartifact spell.\""
    );
    ruling!(
        "Excavation Explosion",
        "You can use the {C} added by a Powerstone token on anything that isn't a nonartifact spell. This includes paying costs to activate abilities of both artifact and nonartifact permanents, paying ward costs, and so on."
    );
    let mut t = TestGame::new(2);
    let x = t.hand(P0, "Excavation Explosion");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    t.cast(P0, x).target(Entity::Player(P1)).go();
    t.resolve_all();
    let stones = with_subtype(&t, P0, "Powerstone");
    assert_eq!(stones.len(), 1);
    let stone = t.obj_now(stones[0]).clone();
    assert!(stone.is(CardType::Artifact) && stone.is_token() && stone.tapped);
    assert_eq!(stone.chars.abilities.len(), 1);
    // Untapped: its {C} can't cast Grizzly Bears, but activates Mishra's Factory.
    t.g.objects[stones[0].0 as usize].tapped = false;
    let bears = t.hand(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    assert!(t.cast(P0, bears).try_go().is_err());
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    t.g.objects[forest.0 as usize].tapped = true;
    let factory = t.battlefield(P0, "Mishra's Factory");
    t.g.objects[factory.0 as usize].tapped = true;
    t.activate(P0, factory, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(factory).is(CardType::Creature));
    assert!(t.obj_now(stones[0]).tapped);
    // With only Powerstones: an artifact spell (Mind Stone), then an artifact's ability.
    let mut t = TestGame::new(2);
    let stones: Vec<ObjectId> = (0..3)
        .map(|_| crate::r_s02_common::create_token(&mut t, P0, "Powerstone"))
        .collect();
    let rock = t.hand(P0, "Mind Stone");
    t.cast(P0, rock).go();
    t.resolve_all();
    let rock = t.named_on_battlefield("Mind Stone")[0];
    let hand = t.hand_size(P0);
    t.activate(P0, rock, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(rock), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(stones.iter().all(|s| t.obj_now(*s).tapped));
}

#[test]
fn powerstone_mana_pays_a_ward_cost() {
    cr!("702.21a", "106.6");
    // (The ruling is cited by the test above; this covers its ward cost.)
    let mut t = TestGame::new(2);
    let stone = crate::r_s02_common::create_token(&mut t, P0, "Powerstone");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let armadillo = t.battlefield(P1, "Armored Armadillo");
    t.activate(P0, pyro, 0, &[obj(armadillo)]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.obj_now(stone).tapped);
    assert_eq!(t.obj_now(armadillo).damage, 1);
}

#[test]
fn the_great_henge_cost_is_locked_before_mana_abilities_are_activated() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "The Great Henge",
        "Once you determine the cost to cast The Great Henge, you may activate mana abilities to pay that cost. If the greatest power among creatures you control changes while activating mana abilities, the cost to cast The Great Henge remains what you previously determined."
    );
    // {1}{G}{G} with Colossal Dreadmaw; sacrificing the Dreadmaw to Ashnod's Altar for mana
    // while paying doesn't raise the cost.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Forest", 2);
    let henge = t.hand(P0, "The Great Henge");
    t.answer_choose(P0, &[obj(maw)]);
    t.cast(P0, henge).go();
    assert_eq!(t.zone(maw), Zone::Graveyard(P0));
    assert_eq!(t.zone(henge), Zone::Stack);
    assert_eq!(untapped_lands(&t, P0), 0);
}
