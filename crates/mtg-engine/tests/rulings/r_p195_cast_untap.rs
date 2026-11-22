//! Rulings batch P195 — permanents that untap when you cast a spell (Blistercoil Weird,
//! Thermo-Alchemist, Unruly Catapult, Lobber Crew, Lightning-Rig Crew, Dwarven Patrol,
//! Saint Traft and Rem Karolus) or when something else happens (Hardbristle Bandit,
//! Dormant Gomazoa, Brimstone Trebuchet, Tiger-Seal, Anje Falkenrath): the trigger goes
//! on the stack above the spell and resolves first, even if the spell is countered;
//! copies aren't cast; "each opponent" in Two-Headed Giant.

use crate::r_p195_common::*;
use mtg_engine::testing::*;
use mtg_engine::decision::Answer;
use mtg_engine::*;

/// P0 controls `name` (tapped) and casts the instant `spell` at `target`: the untap
/// trigger is on top of the spell. Returns (permanent, spell).
fn tapped_then_cast(
    t: &mut TestGame,
    name: &str,
    spell: &str,
    target: Entity,
) -> (ObjectId, ObjectId) {
    let p = t.battlefield(P0, name);
    tap(t, p);
    let s = cast_at(t, P0, spell, &[target]);
    t.settle();
    assert!(top_is_trigger_of(t, p), "{name}: trigger not on top");
    assert!(on_stack(t, s));
    (p, s)
}

#[test]
fn untap_triggers_resolve_before_the_spell_even_if_it_is_countered() {
    cr!("603.3", "405.5", "608.1", "701.6a");
    ruling!(
        "Blistercoil Weird",
        "Blistercoil Weird will get +1/+1 and untap before that instant or sorcery spell resolves."
    );
    ruling!(
        "Blistercoil Weird",
        "Blistercoil Weird will get +1/+1 even if the instant or sorcery spell is countered or if Blistercoil Weird is untapped when its ability resolves."
    );
    ruling!(
        "Thermo-Alchemist",
        "Thermo-Alchemist's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Unruly Catapult",
        "Unruly Catapult's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Lobber Crew",
        "You'll untap Lobber Crew before that spell resolves."
    );
    // (permanent, spell that triggers it)
    for (name, spell) in [
        ("Blistercoil Weird", "Shock"),
        ("Thermo-Alchemist", "Shock"),
        ("Unruly Catapult", "Shock"),
        ("Lobber Crew", "Nivix Guildmage"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let p = t.battlefield(P0, name);
        tap(&mut t, p);
        let s = if spell == "Shock" {
            cast_at(&mut t, P0, spell, &[obj(bears)])
        } else {
            cast_at(&mut t, P0, spell, &[])
        };
        t.settle();
        assert!(top_is_trigger_of(&t, p), "{name}: trigger not on top");
        // Counter the spell under the trigger: the trigger still resolves.
        let cs = cast_at(&mut t, P1, "Counterspell", &[obj(s)]);
        t.resolve(); // Counterspell
        assert!(!on_stack(&t, cs));
        assert!(!on_stack(&t, s), "{name}: spell not countered");
        assert!(tapped(&t, p));
        t.resolve();
        assert!(!tapped(&t, p), "{name}: not untapped");
    }
    // Blistercoil Weird: the +1/+1 applies before the spell resolves, and also when it
    // was already untapped.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let (w, s) = tapped_then_cast(&mut t, "Blistercoil Weird", "Shock", obj(bears));
    t.resolve();
    assert!(on_stack(&t, s));
    assert_eq!(t.pt(w), (2, 2));
    assert!(!tapped(&t, w));
    let w2 = t.battlefield(P0, "Blistercoil Weird");
    cast_at(&mut t, P0, "Shock", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(w2), (2, 2));
}

#[test]
fn players_can_activate_the_untapped_permanent_before_the_spell_resolves() {
    cr!("603.3", "117.3b", "602.2");
    ruling!(
        "Unruly Catapult",
        "Players can cast spells and activate abilities (such as Unruly Catapult's own activated ability) after the triggered ability resolves but before the spell that caused it to trigger does."
    );
    ruling!(
        "Thermo-Alchemist",
        "Players can cast spells and activate abilities (such as Thermo-Alchemist's own activated ability) after the triggered ability resolves but before the spell that caused it to trigger does."
    );
    for name in ["Unruly Catapult", "Thermo-Alchemist"] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, name);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.activate(P0, p, 0, &[]).unwrap();
        t.resolve();
        assert_eq!(t.life(P1), 19);
        let (_, s) = tapped_then_cast(&mut t, name, "Shock", obj(bears));
        let _ = s;
        // The trigger of the first permanent is on top (both untap): resolve both
        // triggers, then activate again above the Shock.
        while !top_is_spell(&t) {
            t.resolve();
        }
        assert!(!tapped(&t, p));
        t.activate(P0, p, 0, &[]).unwrap();
        t.resolve();
        assert_eq!(t.life(P1), 18, "{name}");
        assert!(on_stack(&t, s));
    }
}

fn top_is_spell(t: &TestGame) -> bool {
    t.g.stack.last().is_some_and(|&id| {
        matches!(
            t.g.obj(id).stack.as_ref().map(|si| &si.kind),
            Some(mtg_engine::object::StackKind::Spell)
        )
    })
}

#[test]
fn a_copy_of_a_spell_is_not_cast() {
    cr!("707.10", "601.2i", "603.2");
    ruling!(
        "Blistercoil Weird",
        "If you copy an instant or sorcery spell on the stack (using Nivix Guildmage’s second ability, for example), the copy isn’t cast. Blistercoil Weird’s ability won’t trigger."
    );
    supported("Nivix Guildmage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let nivix = t.battlefield(P0, "Nivix Guildmage");
    let w = t.battlefield(P0, "Blistercoil Weird");
    let s = cast_at(&mut t, P0, "Shock", &[obj(bears)]);
    t.resolve(); // Blistercoil's trigger for the cast Shock
    assert_eq!(t.pt(w), (2, 2));
    lots(&mut t, P0);
    t.activate(P0, nivix, 1, &[obj(s)]).unwrap();
    t.answer_yes(P0, false);
    t.resolve(); // Nivix's ability: copies the Shock
    t.settle();
    assert!(
        !top_is_trigger_of(&t, w),
        "the copy triggered Blistercoil Weird"
    );
    t.resolve_all();
    assert_eq!(t.pt(w), (2, 2));
}

#[test]
fn each_opponent_in_two_headed_giant() {
    cr!("810.7", "810.8a", "120.3a");
    ruling!(
        "Thermo-Alchemist",
        "In a Two-Headed Giant game, Thermo-Alchemist's activated ability deals 1 damage to each of your opponents, resulting in the opposing team losing 2 life."
    );
    ruling!(
        "Unruly Catapult",
        "In a Two-Headed Giant game, Unruly Catapult's activated ability causes the opposing team to lose 2 life."
    );
    ruling!(
        "Lightning-Rig Crew",
        "In a Two-Headed Giant game, Lightning-Rig Crew's activated ability causes the opposing team to lose 2 life."
    );
    ruling!(
        "Brimstone Trebuchet",
        "In a Two-Headed Giant game, Brimstone Trebuchet's activated ability causes it to deal 1 damage twice."
    );
    for name in [
        "Thermo-Alchemist",
        "Unruly Catapult",
        "Lightning-Rig Crew",
        "Brimstone Trebuchet",
    ] {
        supported(name);
        let mut t = two_headed_giant();
        let before = t.life(P2);
        let p = t.battlefield(P0, name);
        let from = t.g.turn_events.len();
        t.activate(P0, p, 0, &[]).unwrap();
        t.resolve();
        assert_eq!(t.life(P2), before - 2, "{name}");
        assert_eq!(t.life(P3), before - 2, "{name}");
        assert_eq!(t.life(P0), 30, "{name}");
        let hits = t.g.turn_events[from..]
            .iter()
            .filter(|e| matches!(e, events::Event::Damage { source, amount: 1, .. } if *source == p))
            .count();
        assert_eq!(hits, 2, "{name}: dealt 1 damage twice");
    }
}

#[test]
fn lands_are_not_spells() {
    cr!("305.1", "305.9");
    ruling!("Dwarven Patrol", "Remember that lands are not spells.");
    supported("Dwarven Patrol");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dwarven Patrol");
    tap(&mut t, d);
    let land = t.hand(P0, "Island");
    t.play_land(P0, land).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(tapped(&t, d));
    // A nonred spell untaps it.
    lots(&mut t, P0);
    let c = t.hand(P0, "Divination");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!tapped(&t, d));
}

#[test]
fn hardbristle_bandit_resolves_before_the_crime_even_if_countered() {
    cr!("700.13", "603.3");
    ruling!(
        "Hardbristle Bandit",
        "Hardbristle Bandit’s last ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered."
    );
    supported("Hardbristle Bandit");
    // A crime by a spell, which is then countered.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Hardbristle Bandit");
    tap(&mut t, b);
    let s = cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.settle();
    assert!(top_is_trigger_of(&t, b));
    cast_at(&mut t, P1, "Counterspell", &[obj(s)]);
    t.resolve();
    assert!(!on_stack(&t, s));
    t.resolve();
    assert!(!tapped(&t, b));
    assert_eq!(t.life(P1), 20);
    // A crime by an activated ability.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Hardbristle Bandit");
    tap(&mut t, b);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[pl(P1)]).unwrap();
    t.settle();
    assert!(top_is_trigger_of(&t, b));
    t.resolve();
    assert!(!tapped(&t, b));
    assert_eq!(t.life(P1), 20);
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

/// P0 controls a tapped Dormant Gomazoa.
fn gomazoa(t: &mut TestGame) -> ObjectId {
    let g = t.battlefield(P0, "Dormant Gomazoa");
    tap(t, g);
    g
}

#[test]
fn dormant_gomazoa_triggers_on_spells_targeting_you() {
    cr!("603.2", "115.1", "603.3");
    ruling!(
        "Dormant Gomazoa",
        "Dormant Gomazoa's last ability triggers when you become the target of any spell, including one you control. It doesn't matter who else or what else the spell targets."
    );
    ruling!(
        "Dormant Gomazoa",
        "If you become the target of a spell, Dormant Gomazoa's last ability triggers and is put on the stack on top of that spell. Dormant Gomazoa's last ability will resolve (allowing you to untap it) first."
    );
    ruling!(
        "Dormant Gomazoa",
        "Dormant Gomazoa's last ability won't trigger when you become the target of an ability."
    );
    ruling!(
        "Dormant Gomazoa",
        "A spell can target you only if it specifically says the word \"target.\" One that says \"You draw two cards,\" for example, doesn't target you."
    );
    supported("Dormant Gomazoa");
    // Your own spell, targeting you: the trigger is on top and resolves first.
    for caster in [P0, P1] {
        let mut t = TestGame::new(2);
        let g = gomazoa(&mut t);
        let s = cast_at(&mut t, caster, "Shock", &[pl(P0)]);
        t.settle();
        assert!(top_is_trigger_of(&t, g), "caster {caster:?}");
        t.answer_yes(P0, true);
        t.resolve();
        assert!(!tapped(&t, g));
        assert!(on_stack(&t, s));
        assert_eq!(t.life(P0), 20);
    }
    // A spell with another target as well (Arc Lightning divided between you and a
    // creature).
    let mut t = TestGame::new(2);
    let g = gomazoa(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let c = t.hand(P0, "Arc Lightning");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, c).targets(&[pl(P0), obj(bears)]).go();
    t.settle();
    assert!(top_is_trigger_of(&t, g));
    // An ability targeting you doesn't trigger it.
    let mut t = TestGame::new(2);
    let g = gomazoa(&mut t);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.activate(P1, pyro, 0, &[pl(P0)]).unwrap();
    t.settle();
    assert!(!top_is_trigger_of(&t, g));
    assert_eq!(t.stack_len(), 1);
    // "You draw two cards" doesn't target you.
    let mut t = TestGame::new(2);
    let g = gomazoa(&mut t);
    cast_at(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(tapped(&t, g));
}

#[test]
fn dormant_gomazoa_triggers_when_a_target_is_changed_to_you_or_copied() {
    cr!("115.7", "707.10c", "603.2");
    ruling!(
        "Dormant Gomazoa",
        "You become the target of a spell when a spell is cast targeting you, when a spell has one of its targets changed to you due to a spell or ability like Deflection, when a spell that's targeting you is copied and that target isn't changed, or when a spell is copied and one of its targets is changed to you."
    );
    supported("Deflection");
    supported("Twincast");
    // Deflection changes Shock's target to you.
    let mut t = TestGame::new(2);
    let g = gomazoa(&mut t);
    let s = cast_at(&mut t, P1, "Shock", &[pl(P1)]);
    cast_at(&mut t, P1, "Deflection", &[obj(s)]);
    t.answer_targets(P1, &[pl(P0)]);
    t.resolve();
    t.settle();
    assert_eq!(stack_targets(&t, s), vec![pl(P0)]);
    assert!(top_is_trigger_of(&t, g), "target changed to you");
    // Twincast copies a Shock targeting you, the target unchanged.
    let mut t = TestGame::new(2);
    let g = gomazoa(&mut t);
    let s = cast_at(&mut t, P1, "Shock", &[pl(P0)]);
    t.answer_yes(P0, false);
    t.resolve(); // the trigger for the Shock (declined)
    assert_eq!(top_of(&t), s);
    cast_at(&mut t, P1, "Twincast", &[obj(s)]);
    t.answer_yes(P1, false); // don't choose new targets
    t.resolve(); // Twincast
    t.settle();
    assert!(top_is_trigger_of(&t, g), "copy targeting you");
    // Twincast copies a Shock at P1, its target changed to you.
    let mut t = TestGame::new(2);
    let g = gomazoa(&mut t);
    let s = cast_at(&mut t, P1, "Shock", &[pl(P1)]);
    cast_at(&mut t, P1, "Twincast", &[obj(s)]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[pl(P0)]);
    t.resolve(); // Twincast
    t.settle();
    assert!(top_is_trigger_of(&t, g), "copy retargeted to you");
}

fn top_of(t: &TestGame) -> ObjectId {
    *t.g.stack.last().unwrap()
}

#[test]
fn brimstone_trebuchet_triggers_for_each_knight() {
    cr!("603.2c", "603.3b", "117.3c");
    ruling!(
        "Brimstone Trebuchet",
        "If more than one Knight enters the battlefield under your control at once, Brimstone Trebuchet's last ability triggers for each of them. Players may take actions, such as activating its activated ability, in between each of those triggered abilities resolving."
    );
    supported("Brimstone Trebuchet");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brimstone Trebuchet");
    tap(&mut t, b);
    // Two Knight tokens at once.
    lots(&mut t, P0);
    let c = t.hand(P0, "Knight Watch");
    t.cast(P0, c).go();
    t.resolve(); // Knight Watch: two 2/2 Knights
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(!tapped(&t, b));
    t.activate(P0, b, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert!(tapped(&t, b));
    t.resolve();
    assert!(!tapped(&t, b));
    t.activate(P0, b, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn saint_traft_triggers_on_convoke_spells_and_counts_resolutions() {
    cr!("702.51a", "603.2");
    ruling!(
        "Saint Traft and Rem Karolus",
        "The last ability of Saint Traft and Rem Karolus will trigger whether you tapped any creatures to pay for the spell or not, as long as it has convoke."
    );
    ruling!(
        "Saint Traft and Rem Karolus",
        "If it’s the fourth or subsequent time the first ability has resolved in a turn, nothing happens."
    );
    supported("Saint Traft and Rem Karolus");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Saint Traft and Rem Karolus");
    // The first tap: a Human token.
    tap(&mut t, s);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // A convoke spell paid entirely with mana: still untaps it.
    lots(&mut t, P0);
    let c = t.hand(P0, "Stoke the Flames");
    t.cast(P0, c).target(pl(P1)).go();
    t.settle();
    assert!(top_is_trigger_of(&t, s));
    t.resolve_all();
    assert!(!tapped(&t, s));
    assert_eq!(t.life(P1), 16);
    // Taps two, three, four: Spirit, Angel, then nothing.
    for expected in [2, 3, 3] {
        tap(&mut t, s);
        t.resolve_all();
        t.g.untap(s);
        assert_eq!(tokens(&t, P0).len(), expected);
    }
}

#[test]
fn tiger_seal_second_card_each_turn() {
    cr!("603.2", "121.2");
    ruling!(
        "Tiger-Seal",
        "The second triggered ability can trigger only once each turn. It doesn't matter whether Tiger-Seal was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    supported("Tiger-Seal");
    // Enters after the first draw: the second draw still triggers it.
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    let s = t.battlefield(P0, "Tiger-Seal");
    tap(&mut t, s);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(top_is_trigger_of(&t, s));
    t.resolve_all();
    assert!(!tapped(&t, s));
    // Third and fourth draws: no trigger.
    tap(&mut t, s);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(tapped(&t, s));
    // Enters after the second draw: no trigger that turn.
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    let s = t.battlefield(P0, "Tiger-Seal");
    tap(&mut t, s);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn anje_madness_discard_draws_last() {
    cr!("702.35a", "603.3", "602.2b");
    ruling!(
        "Anje Falkenrath",
        "If you discard a card with madness to pay the cost of Anje’s activated ability, you’ll draw a card last. You may untap Anje before or after you choose whether to cast the discarded card, but not after you’ve chosen to cast it but before that spell resolves."
    );
    supported("Anje Falkenrath");
    supported("Fiery Temper");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Anje Falkenrath");
    let ft = t.hand(P0, "Fiery Temper");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(ft)]);
    t.activate(P0, a, 0, &[]).unwrap();
    t.settle();
    // Anje's ability is at the bottom with the madness and Anje triggers above it.
    assert_eq!(t.stack_len(), 3);
    let bottom = t.g.stack[0];
    assert!(matches!(
        t.g.obj(bottom).stack.as_ref().map(|si| &si.kind),
        Some(mtg_engine::object::StackKind::Activated { .. })
    ));
    // Resolve the triggers (declining madness), then the draw.
    t.answer_yes(P0, false);
    t.resolve();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert!(!tapped(&t, a));
    assert_eq!(t.hand_size(P0), hand - 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand);
}
