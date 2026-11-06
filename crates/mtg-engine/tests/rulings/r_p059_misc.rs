//! Rulings batch P059 — assorted rulings: Baleful Mastery's alternative cost (CR 118.9,
//! 601.2f), Ertai Resurrected's targets (CR 113.3b, 614.1), "whenever players finish
//! voting" triggers (CR 701.38, 603.3), Alania's copy of a countered spell (CR 707.10,
//! 113.7a), Cut a Deal's count of opponents who drew (CR 121.6, 614.6), the Duos' two
//! color triggers (CR 603.3b), a "for as long as" effect tracking a phased-out permanent
//! (CR 702.26f), and Day's Undoing skipping the end step (CR 724.1).

use crate::r_p058_common::mana;
use crate::r_p059_common::*;
use crate::r_s01_common::{attack_with, supported, tapped_lands};
use crate::r_s25_common::abilities_from;
use mtg_engine::casting::CastOption;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Baleful Mastery: "You may pay {1}{B} rather than pay this spell's mana cost. If the
// {1}{B} cost was paid, an opponent draws a card. Exile target creature or planeswalker."
// ---------------------------------------------------------------------------------------

/// The option to cast `card` for its own alternative cost ("{1}{B}").
fn own_alternative(t: &TestGame, card: ObjectId) -> CastOption {
    t.g.cast_options(P0, card)
        .into_iter()
        .find(|o| {
            matches!(o.method, CastMethod::Alternative(_))
                && o.alt_cost.as_ref().is_some_and(|c| c.mana.is_some())
        })
        .expect("its alternative cost")
}

#[test]
fn baleful_masterys_alternative_cost_with_cost_changes_still_counts_as_paid() {
    cr!("601.2f", "118.9");
    ruling!(
        "Baleful Mastery",
        "If an effect increases or decreases the cost of spells you cast, that cost increase or decrease is applied to the alternative cost you chose to pay. In that case, the cost was still paid for the purposes of the effect, even if you paid more or less for it when it was cast."
    );
    supported("Baleful Mastery");
    // Thalia, Guardian of Thraben ("Noncreature spells cost {1} more to cast."): {2}{B};
    // Goblin Electromancer ("Instant and sorcery spells you cast cost {1} less to cast."):
    // {B}. Either way, P1 draws a card.
    for (other, who, lands) in [
        ("Thalia, Guardian of Thraben", P1, 3),
        ("Goblin Electromancer", P0, 1),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(who, other);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 4);
        let m = t.hand(P0, "Baleful Mastery");
        let alt = own_alternative(&t, m);
        let hand = t.hand_size(P1);
        t.cast(P0, m).method(alt.method).target(bears).go();
        assert_eq!(tapped_lands(&t, P0), lands, "{other}");
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        assert_eq!(t.hand_size(P1), hand + 1, "{other}: the cost was paid");
    }
}

#[test]
fn baleful_mastery_cast_for_free_hasnt_paid_its_alternative_cost() {
    cr!("118.9a", "118.9");
    ruling!(
        "Baleful Mastery",
        "If you choose to pay one alternative cost, you can't pay any other alternative costs. For example, if an effect lets you cast a \"Mastery\" spell \"without paying its mana cost,\" you can't also choose to pay its given alternative cost."
    );
    // With Omniscience: cast without paying its mana cost, P1 draws nothing; for {1}{B},
    // P0 pays it and P1 draws.
    for free in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Omniscience");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 2);
        let m = t.hand(P0, "Baleful Mastery");
        let option = if free {
            t.g.cast_options(P0, m)
                .into_iter()
                .find(|o| o.alt_cost.as_ref().is_some_and(|c| c.mana.is_none()))
                .expect("Omniscience's option")
        } else {
            own_alternative(&t, m)
        };
        let hand = t.hand_size(P1);
        t.cast(P0, m).method(option.method).target(bears).go();
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        assert_eq!(tapped_lands(&t, P0), if free { 0 } else { 2 });
        assert_eq!(t.hand_size(P1), hand + usize::from(!free), "free: {free}");
    }
}

// ---------------------------------------------------------------------------------------
// Ertai Resurrected: "When Ertai Resurrected enters, choose up to one — • Counter target
// spell, activated ability, or triggered ability. Its controller draws a card. • ..."
// ---------------------------------------------------------------------------------------

/// The candidates of the first target choice asked of P0 since decision `from` whose
/// text contains `text`.
fn target_candidates_of(t: &TestGame, from: usize, text: &str) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseTargets {
                text: x,
                candidates,
                ..
            } if *p == P0 && x.contains(text) => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn ertai_can_counter_equip_but_not_replacement_effects() {
    cr!("113.3b", "614.1c", "614.12", "702.6a");
    ruling!(
        "Ertai Resurrected",
        "Abilities that create replacement effects, such as a permanent entering the battlefield tapped or with counters on it, can’t be targeted. Abilities that apply “as [this creature] enters the battlefield” are also replacement effects and can’t be targeted. Similarly, the replacement effect that prevents a creature with a stun counter on it from untapping can’t be countered."
    );
    ruling!(
        "Ertai Resurrected",
        "Activated abilities are written in the form “Cost: Effect.” Some keyword abilities, such as equip and crew, are activated abilities and will have colons in their reminder texts."
    );
    supported("Ertai Resurrected");
    // P1 activates Bonesplitter's equip ability; in response, a Guildgate enters tapped
    // (a replacement effect, never on the stack), and a creature with a stun counter is
    // tapped. Ertai's only candidate is the equip ability, which it counters.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let splitter = t.battlefield(P1, "Bonesplitter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 1);
    let stunned = t.battlefield(P1, "Hill Giant");
    t.g.tap(stunned);
    put(&mut t, stunned, "stun", 1);
    let equip = t.activate(P1, splitter, 0, &[o(bears)]).unwrap().unwrap();
    let gate = t.enter(P1, "Azorius Guildgate");
    assert!(t.obj_now(gate).tapped);
    assert_eq!(t.stack_len(), 1, "only the equip ability is on the stack");
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![0]),
    );
    t.answer_targets(P0, &[o(equip)]);
    let mark = t.asked().len();
    t.enter(P0, "Ertai Resurrected");
    t.settle();
    let candidates = target_candidates_of(&t, mark, "target spell, activated ability");
    assert_eq!(candidates, vec![o(equip)], "{}", t.dump_log());
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(crate::r_s06_common::attached_to(&t, splitter), None, "countered");
    assert_eq!(t.hand_size(P1), hand + 1, "its controller drew");
    // The stun counter's replacement applies in P1's untap step: nothing to counter.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(stunned).tapped);
    assert_eq!(t.counters(stunned, "stun"), 0);
}

// ---------------------------------------------------------------------------------------
// "Whenever players finish voting" (Grudge Keeper: "Whenever players finish voting, each
// opponent who voted for a choice you didn't vote for loses 2 life.")
// ---------------------------------------------------------------------------------------

/// With Grudge Keeper on the battlefield, `start` puts a voting spell or ability on top
/// of the stack; as it resolves the players vote, and Grudge Keeper's trigger goes on the
/// stack only after it has finished resolving.
fn vote_trigger_waits(name: &str, start: impl FnOnce(&mut TestGame)) {
    supported(name);
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Grudge Keeper");
    start(&mut t);
    let top = *t.g.stack.last().expect("the voting spell or ability");
    let below = t.stack_len() - 1;
    t.g.resolve_top();
    t.g.flush_events();
    assert!(!t.g.stack.contains(&top), "{name}: it finished resolving");
    assert_eq!(t.stack_len(), below, "{name}: no trigger on the stack yet");
    assert!(abilities_from(&t, keeper).is_empty(), "{name}");
    t.settle();
    assert_eq!(abilities_from(&t, keeper).len(), 1, "{name}: one trigger");
    t.resolve_all();
}

#[test]
fn vote_triggers_wait_for_the_voting_spell_or_ability_to_finish() {
    cr!("701.38a", "603.3", "701.38d");
    ruling!(
        "Trap the Trespassers",
        "Abilities that trigger \"whenever players finish voting\" trigger once all players have voted or once all secret votes are revealed, but they won't go on the stack until the current spell or ability finishes resolving."
    );
    vote_trigger_waits("Trap the Trespassers", |t| {
        t.battlefield(P1, "Grizzly Bears");
        cast(t, P0, "Trap the Trespassers", &[]);
    });
    ruling!(
        "Sail into the West",
        "Abilities that trigger \"whenever players finish voting\" trigger once all players have voted or once all secret votes are revealed, but they won't go on the stack until the current spell or ability finishes resolving."
    );
    vote_trigger_waits("Sail into the West", |t| {
        cast(t, P0, "Sail into the West", &[]);
    });
    ruling!(
        "Fateful Tempest",
        "Abilities that trigger \"whenever players finish voting\" trigger once all players have voted or once all secret votes are revealed, but they won't go on the stack until the current spell or ability finishes resolving."
    );
    vote_trigger_waits("Fateful Tempest", |t| {
        cast(t, P0, "Fateful Tempest", &[]);
    });
    ruling!(
        "Travel Through Caradhras",
        "Abilities that trigger \"whenever players finish voting\" trigger once all players have voted or once all secret votes are revealed, but they won't go on the stack until the current spell or ability finishes resolving."
    );
    vote_trigger_waits("Travel Through Caradhras", |t| {
        cast(t, P0, "Travel Through Caradhras", &[]);
    });
    ruling!(
        "Emissary Green",
        "Abilities that trigger \"whenever players finish voting\" trigger once all players have voted or once all secret votes are revealed, but they won't go on the stack until the current spell or ability finishes resolving."
    );
    vote_trigger_waits("Emissary Green", |t| {
        let green = t.battlefield(P0, "Emissary Green");
        attack_with(t, &[(green, pl(P1))]);
    });
    ruling!(
        "Círdan the Shipwright",
        "Abilities that trigger \"whenever players finish voting\" trigger once all players have voted or once all secret votes are revealed, but they won't go on the stack until the current spell or ability finishes resolving."
    );
    vote_trigger_waits("Círdan the Shipwright", |t| {
        t.enter(P0, "Círdan the Shipwright");
        t.settle();
    });
    ruling!(
        "Elrond of the White Council",
        "Abilities that trigger \"whenever players finish voting\" trigger once all players have voted or once all secret votes are revealed, but they won't go on the stack until the current spell or ability finishes resolving."
    );
    vote_trigger_waits("Elrond of the White Council", |t| {
        t.enter(P0, "Elrond of the White Council");
        t.settle();
    });
}

// ---------------------------------------------------------------------------------------
// Alania, Divergent Storm: "Whenever you cast a spell, if it's the first instant spell,
// the first sorcery spell, or the first Otter spell other than Alania you've cast this
// turn, you may have target opponent draw a card. If you do, copy that spell. You may
// choose new targets for the copy."
// ---------------------------------------------------------------------------------------

#[test]
fn alanias_copy_resolves_first_even_if_the_spell_was_countered() {
    cr!("707.10", "707.10c", "113.7a");
    ruling!(
        "Alania, Divergent Storm",
        "Alania’s ability and the copy it creates both resolve before the spell that caused the ability to trigger. They resolve even if the spell is countered before the copy is created."
    );
    supported("Alania, Divergent Storm");
    for counter_it in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Alania, Divergent Storm");
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_yes(P0, true);
        let shock = cast(&mut t, P0, "Shock", &[pl(P1)]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "Alania's trigger on top");
        if counter_it {
            cast(&mut t, P1, "Counterspell", &[o(shock)]);
            t.resolve();
            assert!(t.in_graveyard(P0, "Shock"), "countered");
        }
        // Alania's ability resolves: the copy is created on top and resolves first.
        t.resolve();
        assert_eq!(t.life(P1), 20);
        t.resolve();
        assert_eq!(t.life(P1), 18, "the copy: {}", t.dump_log());
        if !counter_it {
            assert!(t.g.stack.contains(&shock), "the original is still waiting");
        }
        t.resolve_all();
        assert_eq!(t.life(P1), if counter_it { 18 } else { 16 });
    }
}

// ---------------------------------------------------------------------------------------
// Cut a Deal: "Each opponent draws a card, then you draw a card for each opponent who
// drew a card this way."
// ---------------------------------------------------------------------------------------

#[test]
fn cut_a_deal_doesnt_count_an_opponent_whose_draw_was_replaced() {
    cr!("121.6", "614.6", "614.11");
    ruling!(
        "Cut a Deal",
        "Although the draws are not optional, if an opponent doesn't draw a card, perhaps because a replacement effect replaced the draw with another effect, that player is not counted when determining how many cards you draw."
    );
    supported("Cut a Deal");
    // Three players: P1 dredges Darkblast instead of drawing; P2 draws.
    let mut t = TestGame::new(3);
    t.graveyard(P1, "Darkblast");
    t.answer_yes(P1, true);
    let hands = (t.hand_size(P1), t.hand_size(P2));
    let mine = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Cut a Deal", &[]);
    assert!(t.in_hand(P1, "Darkblast"), "dredged");
    assert_eq!(t.hand_size(P1), hands.0 + 1);
    assert_eq!(t.hand_size(P2), hands.1 + 1);
    // P0's hand: the Cut a Deal left it, and one card was drawn (for P2).
    assert_eq!(t.hand_size(P0), mine + 1, "{}", t.dump_log());
    // Nobody dredges: two cards.
    let mut t = TestGame::new(3);
    let mine = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Cut a Deal", &[]);
    assert_eq!(t.hand_size(P0), mine + 2);
}

// ---------------------------------------------------------------------------------------
// The Duos and Dire Undercurrents: two abilities triggering on two colors.
// ---------------------------------------------------------------------------------------

#[test]
fn an_object_of_both_colors_triggers_both_abilities() {
    cr!("603.3b", "603.2c");
    use mtg_engine::keywords::KeywordKind;
    ruling!(
        "Emberstrike Duo",
        "An object that’s both of the listed colors will cause both abilities to trigger. You can put them on the stack in either order."
    );
    ruling!(
        "Safehold Duo",
        "An object that’s both of the listed colors will cause both abilities to trigger. You can put them on the stack in either order."
    );
    ruling!(
        "Tattermunge Duo",
        "An object that’s both of the listed colors will cause both abilities to trigger. You can put them on the stack in either order."
    );
    ruling!(
        "Thistledown Duo",
        "An object that’s both of the listed colors will cause both abilities to trigger. You can put them on the stack in either order."
    );
    ruling!(
        "Dire Undercurrents",
        "An object that’s both of the listed colors will cause both abilities to trigger. You can put them on the stack in either order."
    );
    // (Duo, a spell of both its colors, the keyword it gains, its P/T afterwards.)
    for (duo, spell, kw, pt) in [
        ("Emberstrike Duo", "Terminate", KeywordKind::FirstStrike, (2, 2)),
        ("Safehold Duo", "Selesnya Charm", KeywordKind::Vigilance, (3, 5)),
        ("Tattermunge Duo", "Ghor-Clan Rampager", KeywordKind::Landwalk, (3, 4)),
        ("Thistledown Duo", "Azorius Charm", KeywordKind::Flying, (3, 3)),
    ] {
        supported(duo);
        for first in [0usize, 1] {
            let mut t = TestGame::new(2);
            let d = t.battlefield(P0, duo);
            let target = t.battlefield(P1, "Grizzly Bears");
            t.answer(
                P0,
                DecisionKind::Order,
                mtg_engine::decision::Answer::Indices(if first == 0 {
                    vec![0, 1]
                } else {
                    vec![1, 0]
                }),
            );
            let mark = t.asked().len();
            let s = t.hand(P0, spell);
            mana(&mut t, P0, 3);
            match spell {
                "Ghor-Clan Rampager" => t.cast(P0, s).go(),
                "Azorius Charm" => t.cast(P0, s).modes(&[1]).go(),
                "Selesnya Charm" => t.cast(P0, s).modes(&[0]).target(target).go(),
                _ => t.cast(P0, s).target(target).go(),
            };
            t.settle();
            assert_eq!(abilities_from(&t, d).len(), 2, "{duo}");
            assert!(
                t.asked()[mark..]
                    .iter()
                    .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })),
                "{duo}: the order is P0's choice"
            );
            while !abilities_from(&t, d).is_empty() {
                t.resolve();
            }
            assert_eq!(t.pt(d), pt, "{duo}");
            assert!(crate::r_s06_common::has_kw(&t, d, kw), "{duo}");
        }
    }
    // Dire Undercurrents: "Whenever a blue creature you control enters, you may have
    // target player draw a card. Whenever a black creature you control enters, you may
    // have target player discard a card." A blue-black creature (Dimir Guildmage).
    supported("Dire Undercurrents");
    let mut t = TestGame::new(2);
    let du = t.battlefield(P0, "Dire Undercurrents");
    t.hand(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P0)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.enter(P0, "Dimir Guildmage");
    t.settle();
    assert_eq!(abilities_from(&t, du).len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "drew");
    assert_eq!(t.hand_size(P1), 0, "discarded");
}

// ---------------------------------------------------------------------------------------
// Unite the Coalition: "Choose five. You may choose the same mode more than once. •
// Target permanent phases out. • Target player draws a card. • ..."
// ---------------------------------------------------------------------------------------

#[test]
fn a_for_as_long_as_effect_ends_when_its_permanent_phases_out() {
    cr!("702.26f", "611.2b", "702.26b");
    ruling!(
        "Unite the Coalition",
        "Any continuous effects with a “for as long as” duration ignore phased-out objects. If ignoring those objects causes the effect’s conditions to no longer be met, the duration will expire."
    );
    supported("Unite the Coalition");
    // P0's Sower of Temptation ("When this creature enters, gain control of target
    // creature for as long as this creature remains on the battlefield.") took P1's
    // Grizzly Bears; P1 phases the Sower out: the Bears are P1's again, also after the
    // Sower phases in.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    let sower = t.enter(P0, "Sower of Temptation");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P1, 2);
    let spell = t.hand(P1, "Unite the Coalition");
    t.cast(P1, spell)
        .modes(&[0, 1, 1, 1, 1])
        .target(sower)
        .target(P1)
        .target(P1)
        .target(P1)
        .target(P1)
        .go();
    t.resolve_all();
    assert!(t.obj_now(sower).phased_out, "{}", t.dump_log());
    assert_eq!(t.obj_now(bears).controller, P1, "the effect ended");
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(sower).phased_out);
    assert_eq!(t.obj_now(bears).controller, P1, "it doesn't come back");
}

// ---------------------------------------------------------------------------------------
// Day's Undoing: "... If it's your turn, end the turn."
// ---------------------------------------------------------------------------------------

#[test]
fn days_undoing_delays_next_end_step_triggers_a_turn() {
    cr!("724.1d", "603.7a");
    ruling!(
        "Day's Undoing",
        "Any “at the beginning of the next end step” triggered abilities won’t get the chance to trigger that turn because the end step is skipped. Those abilities will trigger at the beginning of the end step of the next turn. The same is true of abilities that trigger at the beginning of other phases or steps."
    );
    supported("Day's Undoing");
    // Flickerwisp exiles P1's Gray Ogre; Day's Undoing ends P0's turn; the Ogre returns at
    // P1's end step.
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[o(ogre)]);
    t.enter(P0, "Flickerwisp");
    t.resolve_all();
    assert!(t.in_exile("Gray Ogre"));
    cast_resolve(&mut t, P0, "Day's Undoing", &[]);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_exile("Gray Ogre"), "the end step was skipped");
    t.advance_to(P1, Step::PostcombatMain);
    assert!(t.in_exile("Gray Ogre"));
    end_step_of(&mut t, P1);
    assert!(t.on_battlefield(ogre));
}
