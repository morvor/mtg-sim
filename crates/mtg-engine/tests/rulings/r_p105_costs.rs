//! Rulings batch P105 — costs and casting: cost determination (Emry, Lurker of the Loch;
//! Advanced Reconstruction), casting and playing from graveyards and exile (Gisa and Geralf,
//! Zask, Glimpse the Impossible, Titania, Nature's Force, Icetill Explorer), costs that mill
//! (Rot Farm Skeleton), kicker "and/or" (Urborg Lhurgoyf), mana with riders (Pyromancer's
//! Goggles, Carnelian Orb of Dragonkind, Ancient Cornucopia) and activation rights and costs
//! (Stay Hidden, Stay Silent; Disturbing Mirth; Tayam; Perpetual Timepiece; Titania, Gaea
//! Incarnate).

use crate::r_p104_common::obj;
use crate::r_p105_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_cast, can_play_land, create_token, destroy};
use crate::r_s08_common::mana_value;
use crate::r_s19_common::gain_level;
use crate::r_s33_common::shuffled_since;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

const NORMAL: CastMethod = CastMethod::Normal;

// ---------------------------------------------------------------------------
// Emry, Lurker of the Loch ({2}{U}, affinity for artifacts)
// ---------------------------------------------------------------------------

#[test]
fn emry_affinity_reduces_only_generic_mana() {
    cr!("702.41a", "601.2f", "118.7d");
    ruling!(
        "Emry, Lurker of the Loch",
        "The cost reduction ability reduces only the generic mana in Emry's cost. The colored mana must still be paid."
    );
    supported("Emry, Lurker of the Loch");
    // Five artifacts: Emry costs {U}, never less.
    for (blue, castable) in [(false, false), (true, true)] {
        let mut t = TestGame::new(2);
        for _ in 0..5 {
            t.battlefield(P0, "Ornithopter");
        }
        t.lands(P0, if blue { "Island" } else { "Wastes" }, 1);
        let emry = t.hand(P0, "Emry, Lurker of the Loch");
        assert_eq!(can_cast(&mut t, P0, emry, NORMAL), castable, "blue: {blue}");
        if castable {
            t.cast(P0, emry).go();
            assert_eq!(tapped_lands(&t, P0), 1);
        }
    }
}

#[test]
fn emry_cost_increases_apply_before_its_reduction() {
    cr!("601.2f", "202.3", "702.41a");
    ruling!(
        "Emry, Lurker of the Loch",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Emry). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Sphere of Resistance");
    // Sphere of Resistance ("Spells cost {1} more to cast") and two Ornithopters: three
    // artifacts. {2}{U} + {1} - {3} = {U}. (Reducing first would leave {1}{U}.)
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere of Resistance");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Island", 1);
    let emry = t.hand(P0, "Emry, Lurker of the Loch");
    let spell = t.cast(P0, emry).go();
    assert_eq!(mana_value(&t, spell), 3);
}

#[test]
fn emry_cost_is_locked_in_before_mana_abilities_are_activated() {
    cr!("601.2f", "601.2g", "601.2h", "702.41a");
    ruling!(
        "Emry, Lurker of the Loch",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't try to change by how much Emry's cost is reduced."
    );
    // Three Treasures: Emry costs {U}. One of them is sacrificed for that mana while
    // paying, after the total cost was locked in.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        create_token(&mut t, P0, "Treasure");
    }
    let emry = t.hand(P0, "Emry, Lurker of the Loch");
    let from = t.asked().len();
    t.cast(P0, emry).go();
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Treasure").len(), 2);
    // No player received priority while it was being cast.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Emry, Lurker of the Loch").len(), 1);
}

// ---------------------------------------------------------------------------
// Advanced Reconstruction, level 3: "Spells you cast from anywhere other than your hand
// cost {2} less to cast."
// ---------------------------------------------------------------------------

#[test]
fn advanced_reconstruction_reduction_applies_after_increases() {
    cr!("601.2f", "202.3", "716.2a");
    ruling!(
        "Advanced Reconstruction",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Advanced Reconstruction's level 3 ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Advanced Reconstruction");
    supported("Emry, Lurker of the Loch");
    // Emry lets P0 cast artifact cards from the graveyard. With Sphere of Resistance:
    // Ornithopter ({0}) costs {0} + {1} - {2} = {0}; Mind Stone ({2}) costs {1}.
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Advanced Reconstruction");
    t.lands(P0, "Mountain", 4);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    gain_level(&mut t, P0, class, 3).unwrap();
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.battlefield(P0, "Sphere of Resistance");
    let emry = t.battlefield(P0, "Emry, Lurker of the Loch");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.activate(P0, emry, 0, &[obj(thopter)]).unwrap();
    t.resolve_all();
    let spell = t.cast(P0, thopter).go();
    assert_eq!(mana_value(&t, spell), 0);
    t.resolve_all();
    // Mind Stone: one land for {1}; its mana value is still 2.
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Advanced Reconstruction");
    t.lands(P0, "Mountain", 4);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    gain_level(&mut t, P0, class, 3).unwrap();
    t.resolve_all();
    t.battlefield(P0, "Sphere of Resistance");
    t.lands(P0, "Wastes", 1);
    let emry = t.battlefield(P0, "Emry, Lurker of the Loch");
    let stone = t.graveyard(P0, "Mind Stone");
    t.activate(P0, emry, 0, &[obj(stone)]).unwrap();
    t.resolve_all();
    let spell = t.cast(P0, stone).go();
    assert_eq!(mana_value(&t, spell), 2);
    assert_eq!(tapped_lands(&t, P0), 5);
}

// ---------------------------------------------------------------------------
// Gisa and Geralf: "Once during each of your turns, you may cast a Zombie creature spell
// from your graveyard."
// ---------------------------------------------------------------------------

#[test]
fn gisa_and_geralf_leaving_after_the_cast_began_doesnt_affect_the_spell() {
    cr!("601.2", "601.3");
    ruling!(
        "Gisa and Geralf",
        "Once you begin to cast the Zombie spell, losing control of Gisa and Geralf won’t affect the spell."
    );
    supported("Gisa and Geralf");
    supported("Walking Corpse");
    let mut t = TestGame::new(2);
    let gisa = t.battlefield(P0, "Gisa and Geralf");
    t.lands(P0, "Swamp", 2);
    let corpse = t.graveyard(P0, "Walking Corpse");
    let spell = t.cast(P0, corpse).go();
    destroy(&mut t, gisa);
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn gisa_and_geralf_zombie_spells_follow_timing_rules() {
    cr!("601.3", "307.1", "117.1a");
    ruling!(
        "Gisa and Geralf",
        "You must follow the normal timing permissions and restrictions of the Zombie spell you cast from your graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gisa and Geralf");
    t.lands(P0, "Swamp", 2);
    t.lands(P1, "Mountain", 1);
    let corpse = t.graveyard(P0, "Walking Corpse");
    assert!(can_cast(&mut t, P0, corpse, NORMAL));
    // Not with a spell on the stack, not in combat, not on another player's turn.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(pl(P0)).go();
    assert!(!can_cast(&mut t, P0, corpse, NORMAL));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, corpse, NORMAL));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, corpse, NORMAL));
}

#[test]
fn gisa_and_geralf_zombie_spells_cost_their_costs_or_an_alternative_cost() {
    cr!("601.2f", "118.9", "601.2b");
    ruling!(
        "Gisa and Geralf",
        "You must pay the costs to cast that Zombie spell. If it has an alternative cost, you may cast it for that cost instead."
    );
    supported("Rooftop Storm");
    // Without mana it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gisa and Geralf");
    let corpse = t.graveyard(P0, "Walking Corpse");
    assert!(!can_cast(&mut t, P0, corpse, NORMAL));
    // Rooftop Storm: "You may pay {0} rather than pay the mana cost for Zombie creature
    // spells you cast."
    t.battlefield(P0, "Rooftop Storm");
    t.g.recompute();
    let methods = crate::r_s08_common::legal_cast_methods(&mut t, P0, corpse);
    let alt = methods
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("Rooftop Storm's alternative cost");
    let spell = t.cast(P0, corpse).method(alt).go();
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

// ---------------------------------------------------------------------------
// Playing cards from a graveyard or exile follows the normal rules
// ---------------------------------------------------------------------------

/// The ways a land card `land` (in some zone P0 may play it from) obeys the timing and
/// land-count rules: only in P0's main phase with an empty stack, and only if P0 hasn't
/// played a land this turn.
fn land_timing_is_normal(t: &mut TestGame, land: ObjectId, other: ObjectId) {
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(pl(P0)).go();
    assert!(!can_play_land(t, P0, land));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(t, P0, land));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(t, P0, land));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(t, P0, land));
    t.play_land(P0, other).unwrap();
    assert!(!can_play_land(t, P0, land));
}

#[test]
fn cards_played_from_a_graveyard_or_exile_follow_the_land_rules() {
    cr!("305.2", "305.1", "116.2a", "601.2f");
    ruling!(
        "Zask, Skittering Swarmlord",
        "You must pay all costs and follow all timing rules for cards played from your graveyard this way. For example, you may play a land this way only while the stack is empty during one of your own main phases, and only if you haven't played a land yet this turn."
    );
    ruling!(
        "Glimpse the Impossible",
        "You pay all costs and follow all normal timing rules for a card played this way. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty."
    );
    ruling!(
        "Titania, Nature's Force",
        "Titania, Nature's Force doesn't change the times when you can play those Forests. You can still play only one land per turn, and only during your main phase when you have priority and the stack is empty."
    );
    supported("Zask, Skittering Swarmlord");
    supported("Glimpse the Impossible");
    supported("Titania, Nature's Force");
    // Zask: a land in the graveyard; an Insect spell (Giant Spider isn't one; Bloatfly
    // Swarm is) costs its mana cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zask, Skittering Swarmlord");
    let land = t.graveyard(P0, "Island");
    let other = t.hand(P0, "Forest");
    let swarm = t.graveyard(P0, "Bloatfly Swarm");
    assert!(!can_cast(&mut t, P0, swarm, NORMAL));
    let lands = t.lands(P0, "Swamp", 4);
    assert!(can_cast(&mut t, P0, swarm, NORMAL));
    for l in lands {
        move_to_exile(&mut t, l);
    }
    land_timing_is_normal(&mut t, land, other);
    // Titania, Nature's Force: Forests in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Titania, Nature's Force");
    let land = t.graveyard(P0, "Forest");
    let other = t.hand(P0, "Island");
    land_timing_is_normal(&mut t, land, other);
    // Glimpse the Impossible: the top three cards are exiled and may be played this turn;
    // a spell among them still costs its mana cost.
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Island", "Grizzly Bears", "Wastes"]);
    t.lands(P0, "Mountain", 3);
    let glimpse = t.hand(P0, "Glimpse the Impossible");
    t.cast(P0, glimpse).go();
    t.resolve_all();
    let bears = t.g.current(lib[1]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!can_cast(&mut t, P0, bears, NORMAL));
    t.lands(P0, "Forest", 2);
    assert!(can_cast(&mut t, P0, bears, NORMAL));
    let land = t.g.current(lib[0]);
    let other = t.hand(P0, "Forest");
    land_timing_is_normal(&mut t, land, other);
}

fn move_to_exile(t: &mut TestGame, id: ObjectId) {
    crate::r_s05_common::move_to(t, id, Zone::Exile);
}

#[test]
fn glimpse_the_impossible_moves_only_the_cards_still_exiled() {
    cr!("400.7", "603.7", "608.2c");
    ruling!(
        "Glimpse the Impossible",
        "Only the cards that remain in exile are put into your graveyard. For example, a permanent spell cast this way will remain on the battlefield."
    );
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Island", "Grizzly Bears", "Lightning Bolt"]);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 2);
    let glimpse = t.hand(P0, "Glimpse the Impossible");
    t.cast(P0, glimpse).go();
    t.resolve_all();
    // P0 plays the Island and casts the Grizzly Bears; the Lightning Bolt stays exiled.
    t.play_land(P0, t.g.current(lib[0])).unwrap();
    let bears = t.cast(P0, t.g.current(lib[1])).go();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.named_on_battlefield("Island").is_empty());
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Spawn").len(), 1);
}

#[test]
fn icetill_explorer_land_drops_add_up() {
    cr!("305.2", "305.2a");
    ruling!(
        "Icetill Explorer",
        "The effect of Icetill Explorer’s first ability is cumulative with similar effects. For example, if you control both Icetill Explorer and Exploration (an enchantment with “You may play an additional land on each of your turns”), you’ll be able to play three lands during each of your turns."
    );
    supported("Icetill Explorer");
    supported("Exploration");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Icetill Explorer");
    t.battlefield(P0, "Exploration");
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        t.play_land(P0, *l).unwrap();
        t.resolve_all();
    }
    assert!(!can_play_land(&mut t, P0, lands[3]));
    assert_eq!(t.named_on_battlefield("Forest").len(), 3);
}

// ---------------------------------------------------------------------------
// Rot Farm Skeleton: "{2}{B}{G}, Mill four cards: Return this card from your graveyard to
// the battlefield. Activate only as a sorcery."
// ---------------------------------------------------------------------------

#[test]
fn rot_farm_skeleton_mills_as_a_cost_from_the_graveyard_only() {
    cr!("602.2b", "601.2h", "113.6k");
    ruling!(
        "Rot Farm Skeleton",
        "Putting four cards from your library into your graveyard is part of the activation cost of Rot Farm Skeleton's ability. The four cards will be in the graveyard before any player can respond."
    );
    ruling!(
        "Rot Farm Skeleton",
        "You can activate the last ability only if Rot Farm Skeleton is in your graveyard."
    );
    supported("Rot Farm Skeleton");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    let on_bf = t.battlefield(P0, "Rot Farm Skeleton");
    assert!(!can_activate(&mut t, P0, on_bf));
    move_to_exile(&mut t, on_bf);
    let sk = t.graveyard(P0, "Rot Farm Skeleton");
    assert!(can_activate(&mut t, P0, sk));
    let gy = t.graveyard_size(P0);
    t.activate(P0, sk, 0, &[]).unwrap();
    // The ability is on the stack; the four cards are already in the graveyard.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.graveyard_size(P0), gy + 4);
    t.resolve_all();
    assert!(t.on_battlefield(sk));
}

// ---------------------------------------------------------------------------
// Urborg Lhurgoyf: "Kicker {U} and/or {B}"
// ---------------------------------------------------------------------------

#[test]
fn urborg_lhurgoyf_kicks_once_for_each_kicker_cost() {
    cr!("702.33a", "702.33b", "702.33c");
    ruling!(
        "Urborg Lhurgoyf",
        "You may kick Urborg Lhurgoyf only once for its {U} cost and only once for its {B} cost. You can kick it twice by paying {U}{B}, but you can't kick it twice by paying {U}{U} or {B}{B}. You can't kick it more than twice."
    );
    supported("Urborg Lhurgoyf");
    // (extra lands, times kicked, cards milled). Asked to kick both ways each time.
    let cases: [(&[&str], u32, usize); 2] = [
        // {U}{B}: kicked twice, six cards milled.
        (&["Island", "Swamp"], 2, 6),
        // {U}{U}: the {U} kicker is paid once; the {B} one can't be paid with {U}.
        (&["Island", "Island"], 1, 3),
    ];
    for (extra, kicks, milled) in cases {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 1);
        for l in extra {
            t.lands(P0, l, 1);
        }
        let goyf = t.hand(P0, "Urborg Lhurgoyf");
        for _ in 0..3 {
            t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        }
        let from = t.asked().len();
        let spell = t.cast(P0, goyf).go();
        // At most one offer per kicker cost.
        let offered = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
            .count();
        assert!(offered <= 2 && offered as u32 >= kicks, "{extra:?}");
        let cast = &t.obj(spell).stack.as_ref().unwrap().cast;
        assert_eq!(cast.times_kicked, kicks, "{extra:?}");
        t.resolve_all();
        assert_eq!(t.graveyard_size(P0), milled, "{extra:?}");
    }
}

// ---------------------------------------------------------------------------
// Pyromancer's Goggles: "{T}: Add {R}. When that mana is spent to cast a red instant or
// sorcery spell, copy that spell and you may choose new targets for the copy."
// ---------------------------------------------------------------------------

#[test]
fn pyromancers_goggles_copies_any_red_instant_or_sorcery() {
    cr!("707.10", "707.12", "603.2");
    ruling!(
        "Pyromancer's Goggles",
        "Any red instant or sorcery spell you spend the mana on will be copied, not just one that requires targets."
    );
    supported("Pyromancer's Goggles");
    supported("Pyroclasm");
    // Pyroclasm: "Pyroclasm deals 2 damage to each creature." Twice kills a 2/4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyromancer's Goggles");
    t.lands(P0, "Mountain", 1);
    let spider = t.battlefield(P1, "Giant Spider");
    let c = t.hand(P0, "Pyroclasm");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!t.on_battlefield(spider));
}

#[test]
fn pyromancers_goggles_copy_resolves_first_even_if_the_spell_is_countered() {
    cr!("707.10", "707.10c", "608.2b", "405.5");
    ruling!(
        "Pyromancer's Goggles",
        "A copy is created even if the spell cast with the red mana produced by Pyromancer's Goggles has been countered or otherwise left the stack without resolving by the time that ability resolves. The copy resolves before the original spell."
    );
    // The copy resolves first, with the original still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyromancer's Goggles");
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(pl(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve(); // the trigger: the copy goes on the stack above the original
    t.resolve(); // the copy
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // The original is countered in response to the trigger: the copy is still created.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyromancer's Goggles");
    t.lands(P1, "Island", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(pl(P1)).go();
    t.settle();
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(spell)).go();
    t.resolve(); // Counterspell
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn carnelian_orb_mana_on_a_dragon_making_sorcery_gives_no_haste() {
    cr!("205.2a", "205.3", "111.1");
    ruling!(
        "Carnelian Orb of Dragonkind",
        "An instant or sorcery spell is not a creature spell, even if that spell creates Dragon creature tokens."
    );
    supported("Carnelian Orb of Dragonkind");
    supported("Death by Dragons");
    // Death by Dragons ({4}{R}{R}): "Each player other than target player creates a 5/5
    // red Dragon creature token with flying."
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Carnelian Orb of Dragonkind");
    t.lands(P0, "Mountain", 5);
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, orb, "Add {R}"));
    let c = t.hand(P0, "Death by Dragons");
    t.cast(P0, c).target(pl(P1)).go();
    t.resolve_all();
    let dragons = crate::r_s05_common::tokens_with_subtype(&t, P0, "Dragon");
    assert_eq!(dragons.len(), 1);
    assert!(!t.obj_now(dragons[0]).has_keyword(KeywordKind::Haste));
}

// ---------------------------------------------------------------------------
// Ancient Cornucopia: "Whenever you cast a spell that's one or more colors, you may gain
// 1 life for each of that spell's colors. Do this only once each turn."
// ---------------------------------------------------------------------------

#[test]
fn ancient_cornucopia_counts_colors_not_mana_symbols() {
    cr!("105.2", "202.2", "603.2");
    ruling!(
        "Ancient Cornucopia",
        "Ancient Cornucopia’s triggered ability counts the number of colors the spell has (from one to five), not how many colored mana symbols there are in its mana cost or how many colors of mana you spent."
    );
    supported("Ancient Cornucopia");
    // (spell, lands, life gained)
    let cases: [(&str, &[&str], i32); 2] = [
        ("Ball Lightning", &["Mountain", "Mountain", "Mountain"], 1),
        ("Lightning Helix", &["Mountain", "Plains"], 2),
    ];
    for (name, lands, gain) in cases {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ancient Cornucopia");
        for l in lands {
            t.lands(P0, l, 1);
        }
        let c = t.hand(P0, name);
        t.answer_yes(P0, true);
        if name == "Lightning Helix" {
            t.cast(P0, c).target(pl(P1)).go();
        } else {
            t.cast(P0, c).go();
        }
        t.resolve();
        assert_eq!(t.life(P0), 20 + gain, "{name}");
    }
}

#[test]
fn ancient_cornucopia_resolves_first_and_uses_last_known_colors() {
    cr!("603.3", "405.5", "608.2h", "113.7a");
    ruling!(
        "Ancient Cornucopia",
        "Ancient Cornucopia’s triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered. If that spell is countered, use its last known information to determine what colors it was."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Cornucopia");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Island", 2);
    let helix = t.hand(P0, "Lightning Helix");
    let spell = t.cast(P0, helix).target(pl(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(spell)).go();
    t.answer_yes(P0, true);
    t.resolve(); // Counterspell
    assert!(t.in_graveyard(P0, "Lightning Helix"));
    t.resolve(); // Cornucopia's trigger
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// Who may activate, and what costs mean
// ---------------------------------------------------------------------------

#[test]
fn stay_hidden_stay_silent_only_its_controller_activates_it() {
    cr!("602.2", "303.4e");
    ruling!(
        "Stay Hidden, Stay Silent",
        "Only the controller of Stay Hidden, Stay Silent may activate its last ability."
    );
    supported("Stay Hidden, Stay Silent");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Stay Hidden, Stay Silent", obj(bears));
    t.lands(P0, "Island", 6);
    t.lands(P1, "Island", 6);
    // P1 controls the enchanted creature, but not the Aura.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P1, aura));
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_activate(&mut t, P0, aura));
}

#[test]
fn disturbing_mirth_is_sacrificed_only_by_an_effect_that_allows_it() {
    cr!("701.21a", "603.2");
    ruling!(
        "Disturbing Mirth",
        "You can't sacrifice Disturbing Mirth unless another effect allows you to."
    );
    supported("Disturbing Mirth");
    let mut t = TestGame::new(2);
    let mirth = t.battlefield(P0, "Disturbing Mirth");
    // No action of P0's sacrifices it.
    t.g.recompute();
    let actions = t.g.legal_actions(P0);
    assert!(!actions.iter().any(|a| format!("{a:?}").contains(&format!("{:?}", mirth))));
    // Another Disturbing Mirth's enters ability lets P0 sacrifice it: it manifests dread.
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let other = t.hand(P0, "Disturbing Mirth");
    t.cast(P0, other).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(mirth)]);
    t.resolve_all();
    assert!(!t.on_battlefield(mirth));
    let face_down = t.g.permanents().filter(|o| o.face_down).count();
    assert_eq!(face_down, 1);
}

#[test]
fn tayam_removes_any_kind_of_counters() {
    cr!("118.3", "602.2b");
    ruling!(
        "Tayam, Luminous Enigma",
        "You can remove any three counters to activate Tayam’s last ability, not just vigilance counters."
    );
    supported("Tayam, Luminous Enigma");
    let mut t = TestGame::new(2);
    let tayam = t.battlefield(P0, "Tayam, Luminous Enigma");
    let bears = t.battlefield(P0, "Grizzly Bears");
    set_counters(&mut t, bears, counters::PLUS1, 3);
    t.lands(P0, "Wastes", 3);
    assert!(can_activate(&mut t, P0, tayam));
    t.activate(P0, tayam, 0, &[]).unwrap();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn perpetual_timepiece_with_no_targets_just_shuffles() {
    cr!("115.1", "608.2b", "601.2c");
    ruling!(
        "Perpetual Timepiece",
        "You can activate Perpetual Timepiece's second ability with no targets just to shuffle your library. If you choose any targets and they all become illegal targets, the ability won't resolve and you won't shuffle your library."
    );
    supported("Perpetual Timepiece");
    // No targets: P0's library is shuffled.
    let mut t = TestGame::new(2);
    let tp = t.battlefield(P0, "Perpetual Timepiece");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, tp, 1, &[]).unwrap();
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(shuffled_since(&t, from, P0));
    // One target, exiled in response: no shuffle.
    let mut t = TestGame::new(2);
    let tp = t.battlefield(P0, "Perpetual Timepiece");
    t.lands(P0, "Wastes", 2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.activate(P0, tp, 1, &[obj(bears)]).unwrap();
    move_to_exile(&mut t, bears);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(!shuffled_since(&t, from, P0));
}

// ---------------------------------------------------------------------------
// Titania, Gaea Incarnate (the melded Argoth, Sanctum of Nature): "{3}{G}: Put four +1/+1
// counters on target land you control. It becomes a 0/0 Elemental creature with haste.
// It's still a land."
// ---------------------------------------------------------------------------

#[test]
fn titania_gaea_incarnate_animates_lands_indefinitely_and_repeatedly() {
    cr!("611.2a", "613.1d", "122.1a");
    ruling!(
        "Argoth, Sanctum of Nature",
        "Titania, Gaea Incarnate's last ability doesn't have a duration, which means that the land remains a creature indefinitely."
    );
    ruling!(
        "Argoth, Sanctum of Nature",
        "You can activate the ability multiple times targeting the same land to add more counters."
    );
    supported("Titania, Gaea Incarnate");
    let mut t = TestGame::new(2);
    let titania = t.battlefield(P0, "Titania, Gaea Incarnate");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Forest", 8);
    for _ in 0..2 {
        t.answer_targets(P0, &[obj(forest)]);
        crate::r_s06_common::activate_containing(&mut t, P0, titania, "four +1/+1 counters")
            .unwrap();
        t.resolve_all();
    }
    assert_eq!(t.counters(forest, counters::PLUS1), 8);
    assert_eq!(t.pt(forest), (8, 8));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let o = t.obj_now(forest);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert!(o.chars.has_subtype("Elemental"));
}
