//! Rulings batch P100 — triggered abilities that compare mana values: intervening "if"
//! clauses checked twice (CR 603.4), cast triggers resolving before the spell and
//! surviving its countering (603.2, 608.2b), last known information for a spell that left
//! the stack (608.2h), several instances of cascade (702.85c), a creature destroyed before
//! its own damage trigger resolves (704.3), simultaneous entering (614.1d, 614.12), and a
//! team's attackers in Two-Headed Giant (810.7).

use crate::r_p100_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Shock at P1 from a pool with {R} in it; the spell stays on the stack.
fn shock(t: &mut TestGame) -> ObjectId {
    let s = t.hand(P0, "Shock");
    mana(t, P0, "R");
    t.g.turn.priority = Some(P0);
    let spell = t.cast(P0, s).target(pl(P1)).go();
    t.settle();
    spell
}

#[test]
fn obscura_ascendancy_triggers_at_one_plus_its_counters_and_checks_again_on_resolution() {
    cr!("603.4");
    ruling!(
        "Obscura Ascendancy",
        "If there are no soul counters on Obscura Ascendancy, its ability triggers if you cast a spell with mana value 1. If there is one counter, its ability triggers if you cast a spell with mana value 2, and so on."
    );
    ruling!(
        "Obscura Ascendancy",
        "Obscura Ascendancy's triggered ability has an “intervening ‘if'” clause. This means that it checks the number of counters both when the ability triggers and when it resolves. Notably, this means that casting two instants in a row with mana value 1 will never cause a player to create two tokens or put two counters on Obscura Ascendancy."
    );
    supported("Obscura Ascendancy");
    // No counters: Grizzly Bears (2) doesn't trigger it; Shock (1) does: a counter and a
    // Spirit. Then Lightning Strike (2) does, Shock doesn't.
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Obscura Ascendancy");
    let bears = t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, "1G");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "soul counter"), 0);
    t.resolve_all();
    shock(&mut t);
    assert_eq!(triggers_on_stack(&t, "soul counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(asc, "soul"), 1);
    assert_eq!(tokens(&t, P0).len(), 1);
    shock(&mut t);
    assert_eq!(triggers_on_stack(&t, "soul counter"), 0);
    t.resolve_all();
    let strike = t.hand(P0, "Lightning Strike");
    mana(&mut t, P0, "1R");
    t.cast(P0, strike).target(pl(P1)).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "soul counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(asc, "soul"), 2);
    assert_eq!(tokens(&t, P0).len(), 2);
    // Two Shocks in a row with no counters: both trigger; the first resolves (a counter),
    // and the second finds two counters needed: nothing.
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Obscura Ascendancy");
    shock(&mut t);
    shock(&mut t);
    assert_eq!(triggers_on_stack(&t, "soul counter"), 2);
    t.resolve_all();
    assert_eq!(t.counters(asc, "soul"), 1);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn imminent_doom_deals_the_counters_it_had_as_it_triggered_and_resolves_before_the_spell() {
    cr!("603.2", "603.3", "608.2b", "117.3c");
    ruling!(
        "Imminent Doom",
        "The amount of damage Imminent Doom's triggered ability deals is the number of counters it had on it as the ability triggered. For example, if you cast Shock and then respond to Imminent Doom's triggered ability with a second Shock, both abilities cause Imminent Doom to deal 1 damage and Imminent Doom will end up with three doom counters on it."
    );
    ruling!(
        "Imminent Doom",
        "Imminent Doom's triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    supported("Imminent Doom");
    // Shock, and a second Shock in response to the trigger: 1 + 1 damage from the
    // triggers, 2 + 2 from the Shocks; three doom counters.
    let mut t = TestGame::new(2);
    let doom = t.enter(P0, "Imminent Doom");
    t.settle();
    assert_eq!(t.counters(doom, "doom"), 1);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    shock(&mut t);
    assert_eq!(triggers_on_stack(&t, "doom counter"), 1);
    shock(&mut t);
    assert_eq!(triggers_on_stack(&t, "doom counter"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 1 - 1 - 2 - 2, "{}", t.dump_log());
    assert_eq!(t.counters(doom, "doom"), 3);
    // The trigger resolves before the Shock, and even though the Shock is countered.
    let mut t = TestGame::new(2);
    let doom = t.enter(P0, "Imminent Doom");
    t.settle();
    t.answer_targets(P0, &[pl(P1)]);
    let spell = shock(&mut t);
    crate::r_p153_common::counter(&mut t, P1, spell);
    assert_eq!(triggers_on_stack(&t, "doom counter"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "the trigger's damage, not the Shock's");
    assert_eq!(t.counters(doom, "doom"), 2);
}

#[test]
fn ancient_cellarspawn_uses_the_last_known_mana_value_of_a_countered_spell() {
    cr!("608.2h", "603.4", "601.2f");
    ruling!(
        "Ancient Cellarspawn",
        "If the spell is no longer on the stack when Ancient Cellarspawn's last ability resolves, use its mana value as it last existed on the stack."
    );
    supported("Ancient Cellarspawn");
    // A second Ancient Cellarspawn ({1}{B}{B}, a Horror) costs {B}{B}: 2 spent for mana
    // value 3. P1 counters it in response to the trigger: P1 still loses 1 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Cellarspawn");
    let second = t.hand(P0, "Ancient Cellarspawn");
    mana(&mut t, P0, "BB");
    t.answer_targets(P0, &[pl(P1)]);
    let spell = t.cast(P0, second).go();
    t.settle();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(triggers_on_stack(&t, "loses life"), 1);
    crate::r_p153_common::counter(&mut t, P1, spell);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn kozileks_cast_trigger_checks_the_hand_as_it_is_cast_and_as_it_resolves() {
    cr!("603.4", "601.2a");
    ruling!(
        "Kozilek, the Great Distortion",
        "Kozilek's triggered ability checks to see if you have fewer than seven cards in hand when you cast it. Kozilek is on the stack at this time and not in your hand. If you don't, the ability won't trigger at all. If the ability does trigger, it will check again as it tries to resolve. If you have seven or more cards in hand at that time, the ability won't do anything."
    );
    supported("Kozilek, the Great Distortion");
    // Six other cards in hand: with Kozilek on the stack rather than in hand, six: it
    // triggers and draws one.
    let kozilek_with = |others: usize| {
        let mut t = TestGame::new(2);
        t.g.players[P0.idx()].hand.clear();
        for _ in 0..others {
            t.hand(P0, "Forest");
        }
        let k = t.hand(P0, "Kozilek, the Great Distortion");
        mana(&mut t, P0, "8CC");
        t.cast(P0, k).go();
        t.settle();
        t
    };
    let mut t = kozilek_with(6);
    assert_eq!(t.hand_size(P0), 6);
    assert_eq!(triggers_on_stack(&t, "draw"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
    // Eight other cards: eight in hand as it's cast: no trigger.
    let mut t = kozilek_with(8);
    assert_eq!(t.hand_size(P0), 8);
    assert_eq!(triggers_on_stack(&t, "draw"), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 8);
    // Six in hand as it's cast, then two cards drawn in response: the ability does nothing.
    let mut t = kozilek_with(6);
    assert_eq!(triggers_on_stack(&t, "draw"), 1);
    t.g.draw_cards(P0, 2);
    t.settle();
    assert_eq!(t.hand_size(P0), 8);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 8);
}

#[test]
fn padeem_needs_the_greatest_artifact_as_the_upkeep_begins_and_as_the_trigger_resolves() {
    cr!("603.4");
    ruling!(
        "Padeem, Consul of Innovation",
        "If you don't control an artifact with the highest mana value as your upkeep begins, Padeem's second ability won't trigger. You can't cast an instant to destroy an opponent's artifact during your turn before your upkeep begins."
    );
    ruling!(
        "Padeem, Consul of Innovation",
        "If you no longer control an artifact with the highest mana value as Padeem's triggered ability resolves, you don't draw a card."
    );
    supported("Padeem, Consul of Innovation");
    // Sol Ring (1) against an opponent's Mind Stone (2): no trigger at all (and no player
    // gets priority in the untap step to change that, CR 502.4).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Padeem, Consul of Innovation");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P1, "Mind Stone");
    t.set_step(P1, Step::End);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Mind Stone (2) against Sol Ring (1): it triggers; the Mind Stone is destroyed in
    // response: no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Padeem, Consul of Innovation");
    let stone = t.battlefield(P0, "Mind Stone");
    t.battlefield(P1, "Sol Ring");
    t.set_step(P1, Step::End);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1, "{:?}", stack_items(&t));
    destroy(&mut t, stone);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn two_instances_of_cascade_trigger_separately() {
    cr!("702.85c", "702.85a");
    ruling!(
        "Bigger on the Inside",
        "If a spell has multiple instances of cascade, each one triggers separately."
    );
    ruling!("TARDIS Bay", "If a spell has multiple instances of cascade, each one triggers separately.");
    supported("Bigger on the Inside");
    supported("TARDIS Bay");
    supported("Bloodbraid Elf");
    // Bigger on the Inside's mana ability activated twice: "The next spell they cast this
    // turn has cascade" twice over, so Hill Giant has two instances.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let aura = t.battlefield(P0, "Bigger on the Inside");
    assert!(t.g.attach(aura, obj(thopter)));
    t.settle();
    for _ in 0..2 {
        t.g.objects[thopter.0 as usize].tapped = false;
        act(&mut t, P0, thopter, "Target player adds", &[pl(P0)]).expect("Bigger on the Inside");
        t.resolve_all();
    }
    t.g.players[P0.idx()].mana_pool = Default::default();
    stack_library(&mut t, P0, &["Grizzly Bears", "Memnite", "Forest"]);
    let giant = t.hand(P0, "Hill Giant");
    mana(&mut t, P0, "3R");
    let spell = t.cast(P0, giant).go();
    t.g.recompute();
    assert_eq!(
        t.g.obj(spell).chars.keywords_of(mtg_engine::keywords::KeywordKind::Cascade).count(),
        2,
        "two instances"
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 2, "{:?}", stack_items(&t));
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(count_named(&t, "Grizzly Bears"), 1, "{:?}", battlefield_names(&t));
    assert_eq!(count_named(&t, "Memnite"), 1, "{:?}", battlefield_names(&t));
    // TARDIS Bay (a plane): "The first spell you cast during each of your turns with mana
    // value 2 or greater has cascade." Bloodbraid Elf ({2}{R}{G}) has cascade printed on
    // it as well: two instances, two triggers.
    let mut t = crate::r_s19_common::planechase_game(2);
    crate::r_s19_common::start_planar_deck(&mut t, P0, &["TARDIS Bay"]);
    t.set_step(P0, Step::PrecombatMain);
    stack_library(&mut t, P0, &["Grizzly Bears", "Memnite", "Forest"]);
    let elf = t.hand(P0, "Bloodbraid Elf");
    mana(&mut t, P0, "2RG");
    let spell = t.cast(P0, elf).go();
    t.g.recompute();
    assert_eq!(
        t.g.obj(spell).chars.keywords_of(mtg_engine::keywords::KeywordKind::Cascade).count(),
        2,
        "its own and the plane's"
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 2, "{:?}", stack_items(&t));
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(count_named(&t, "Grizzly Bears"), 1, "{:?}", battlefield_names(&t));
    assert_eq!(count_named(&t, "Memnite"), 1, "{:?}", battlefield_names(&t));
}

#[test]
fn epistolary_librarian_counts_a_teammates_attackers_in_two_headed_giant() {
    cr!("810.7", "805.10", "608.2h");
    ruling!(
        "Epistolary Librarian",
        "In Two-Headed giant and other formats where multiple players can attack at the same time, Epistolary Librarian's triggered ability counts all attacking creatures, even those controlled by other players."
    );
    supported("Epistolary Librarian");
    // P0's Librarian and P1's Grizzly Bears attack together: X is 2, so a Grizzly Bears
    // card (2) may be cast, a Gray Ogre card (3) not.
    let mut t = two_headed_giant();
    let lib = t.battlefield(P0, "Epistolary Librarian");
    let mate = t.battlefield(P1, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    let ogre = t.hand(P0, "Gray Ogre");
    t.set_step(P0, Step::PrecombatMain);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears)]);
    attack_with(&mut t, &[(lib, pl(P2)), (mate, pl(P2))]);
    t.resolve_all();
    assert!(was_offered(&t, P0, from, bears), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, ogre));
    assert!(t.on_battlefield(bears));
    // Alone in a two-player game: X is 1, so the Bears card isn't offered.
    let mut t = TestGame::new(2);
    let lib = t.battlefield(P0, "Epistolary Librarian");
    let bears = t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    attack_with(&mut t, &[(lib, pl(P1))]);
    t.resolve_all();
    assert!(!was_offered(&t, P0, from, bears), "{:?}", asked_of(&t, P0, from));
}

#[test]
fn vitu_ghazi_inspectors_trigger_with_an_illegal_target_gains_no_life() {
    cr!("608.2b", "701.59c");
    ruling!(
        "Vitu-Ghazi Inspector",
        "If the target creature is an illegal target as Vitu-Ghazi Inspector's triggered ability tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    supported("Vitu-Ghazi Inspector");
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let inspector = t.hand(P0, "Vitu-Ghazi Inspector");
    mana(&mut t, P0, "1G");
    t.answer_choose(P0, &[obj(wurm)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, inspector).kicked(true).go();
    assert_eq!(t.zone(wurm), Zone::Exile, "evidence collected");
    t.resolve();
    assert!(t.on_battlefield(inspector));
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1, "{:?}", stack_items(&t));
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // With the target still there: a counter and 2 life.
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let inspector = t.hand(P0, "Vitu-Ghazi Inspector");
    mana(&mut t, P0, "1G");
    t.answer_choose(P0, &[obj(wurm)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, inspector).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn vessel_of_the_all_consuming_dies_before_its_counter_arrives() {
    cr!("704.3", "704.5g", "510.2");
    ruling!(
        "Hidetsugu Consumes All // Vessel of the All-Consuming",
        "If Vessel of the All-Consuming is dealt lethal damage at the same time it deals damage, it will be destroyed before the first triggered ability resolves, so the +1/+1 counter won't be put on it in time to potentially save it."
    );
    supported("Hidetsugu Consumes All // Vessel of the All-Consuming");
    let mut t = TestGame::new(2);
    let vessel = t.battlefield(P0, "Hidetsugu Consumes All // Vessel of the All-Consuming");
    assert!(mtg_engine::dfc::transform(&mut t.g, vessel));
    t.settle();
    assert_eq!(t.obj_now(vessel).chars.name, "Vessel of the All-Consuming");
    assert_eq!(t.pt(vessel), (3, 3));
    let armodon = t.battlefield(P1, "Trained Armodon");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(vessel, pl(P1))], &[(armodon, vessel)]);
    t.resolve_all();
    assert_eq!(t.zone(vessel), Zone::Graveyard(P0), "{}", t.dump_log());
    assert_eq!(t.zone(armodon), Zone::Graveyard(P1));
}

#[test]
fn dauntless_dismantler_doesnt_tap_an_artifact_entering_with_it() {
    cr!("614.1d", "614.12");
    ruling!(
        "Dauntless Dismantler",
        "If an artifact an opponent controls enters the battlefield at the same time that Dauntless Dismantler enters the battlefield, Dauntless Dismantler's effect won't apply to that artifact."
    );
    supported("Dauntless Dismantler");
    supported("Show and Tell");
    // Show and Tell: P0 puts the Dismantler and P1 a Mind Stone onto the battlefield at
    // the same time: the Mind Stone is untapped. P1's next artifact enters tapped.
    let mut t = TestGame::new(2);
    let dismantler = t.hand(P0, "Dauntless Dismantler");
    let stone = t.hand(P1, "Mind Stone");
    t.answer_yes(P0, true).answer_yes(P1, true);
    t.answer_choose(P0, &[obj(dismantler)]);
    t.answer_choose(P1, &[obj(stone)]);
    cast_resolve(&mut t, P0, "Show and Tell", "2U", &[]);
    assert!(t.on_battlefield(dismantler) && t.on_battlefield(stone));
    assert!(!t.obj_now(stone).tapped);
    let ring = t.enter(P1, "Sol Ring");
    assert!(t.obj_now(ring).tapped);
}

#[test]
fn simulacrum_synthesizers_construct_counts_itself() {
    cr!("613.4c", "604.2");
    ruling!(
        "Simulacrum Synthesizer",
        "The ability of the Construct token created by Simulacrum Synthesizer's last ability does count the Construct itself, so it'll be at least 1/1."
    );
    supported("Simulacrum Synthesizer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Simulacrum Synthesizer");
    t.enter(P0, "Juggernaut");
    t.resolve_all();
    let construct = tokens(&t, P0)[0];
    assert_eq!(t.pt(construct), (3, 3), "Synthesizer, Juggernaut and itself");
    // Alone with the Synthesizer, it's 2/2; with nothing else at all, 1/1.
    for o in t.g.battlefield.clone() {
        if o != construct && t.g.obj(o).chars.name != "Simulacrum Synthesizer" {
            destroy(&mut t, o);
        }
    }
    assert_eq!(t.pt(construct), (2, 2));
    for o in t.g.battlefield.clone() {
        if o != construct {
            destroy(&mut t, o);
        }
    }
    assert_eq!(t.pt(construct), (1, 1));
}
