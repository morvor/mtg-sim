//! Rulings batch S01 — aftermath (CR 702.127) and the split card rules its rulings restate
//! (CR 709).

use crate::r_s01_common::*;
use mtg_engine::ability::{Cmp, Duration, Filter, Value};
use mtg_engine::casting::grant_play_permission;
use mtg_engine::decision::{Action, Agent, Answer, Decision};
use mtg_engine::eval::Ctx;
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const AFTERMATH: CastMethod = CastMethod::Keyword(KeywordKind::Aftermath);
const FIRST_HALF: CastMethod = CastMethod::Half(0);
const SECOND_HALF: CastMethod = CastMethod::Half(1);

/// The ways `p` could begin to cast `card` now.
fn cast_methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    t.g.cast_options(p, card)
        .into_iter()
        .filter(|o| t.g.can_begin_cast(p, card, o))
        .map(|o| o.method)
        .collect()
}

/// Whether a split card is in any graveyard.
fn split_card_in_a_graveyard(g: &Game) -> bool {
    g.players.iter().any(|p| {
        p.graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name.contains(" // "))
    })
}

fn colors(cs: &[Color]) -> ColorSet {
    cs.iter()
        .fold(ColorSet::default(), |s, c| s.union(ColorSet::single(*c)))
}

/// Player `p`'s answer to "choose a color" (Iona, Shield of Emeria).
fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn a_split_card_is_one_card() {
    cr!("709.1");
    ruling!(
        "Consign // Oblivion",
        "Each split card is a single card. For example, if you discard one, you've discarded one card, not two. If an effect counts the number of instant and sorcery cards in your graveyard, Destined // Lead counts once, not twice."
    );
    ruling!(
        "Claim // Fame",
        "Each split card is a single card. For example, if you discard one, you’ve discarded one card, not two."
    );
    supported("Enigma Drake");
    let mut t = TestGame::new(2);
    // "Enigma Drake's power is equal to the number of instant and sorcery cards in your
    // graveyard." Consign // Oblivion is an instant card and a sorcery card: it counts
    // once.
    let drake = t.battlefield(P0, "Enigma Drake");
    let co = t.hand(P0, "Consign // Oblivion");
    t.hand(P0, "Island");
    t.g.discard(P0, co, None);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 1);
    t.graveyard(P0, "Claim // Fame");
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 2);
}

#[test]
fn off_the_stack_a_split_card_combines_both_halves() {
    cr!("709.4");
    ruling!(
        "Consign // Oblivion",
        "While not on the stack, the characteristics of a split card are the combination of its two halves. For example, Destined // Lead is a green and black card, it is both an instant card and a sorcery card, and its mana value is 6."
    );
    ruling!(
        "Claim // Fame",
        "While not on the stack, the characteristics of a split card are the combination of its two halves. For example, Destined // Lead is a green and black card"
    );
    let mut t = TestGame::new(2);
    let co = t.hand(P0, "Consign // Oblivion");
    let cf = t.hand(P0, "Claim // Fame");
    t.g.recompute();
    let c = t.obj(co).chars.clone();
    assert_eq!(c.colors, colors(&[Color::Blue, Color::Black]));
    assert!(c.is(CardType::Instant) && c.is(CardType::Sorcery));
    assert_eq!(t.g.mana_value_of(co), 7);
    let c = t.obj(cf).chars.clone();
    assert_eq!(c.colors, colors(&[Color::Black, Color::Red]));
    assert_eq!(t.g.mana_value_of(cf), 3);
    let ctx = Ctx::new(None, P0);
    assert!(!t
        .g
        .matches(co, &Filter::ManaValue(Cmp::Le, Box::new(Value::c(2))), &ctx));
}

#[test]
fn cascade_skips_a_split_card_whose_combined_mana_value_is_too_high() {
    cr!("709.4", "702.85a");
    ruling!(
        "Consign // Oblivion",
        "This means that if an effect allows you to cast a card with mana value 2 from your hand, you can't cast Destined."
    );
    let mut t = TestGame::new(2);
    // Consign alone has mana value 2, but the card's mana value is 7: not less than 4.
    stack_library(&mut t, P0, &["Consign // Oblivion", "Grizzly Bears"]);
    give_mana_for(&mut t, P0, "Bloodbraid Elf");
    let elf = t.hand(P0, "Bloodbraid Elf");
    t.cast(P0, elf).go();
    t.settle();
    t.resolve();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Grizzly Bears");
    t.resolve_all();
    let bottom = t.g.player(P0).library[0];
    assert_eq!(t.g.obj(bottom).chars.name, "Consign // Oblivion");
}

#[test]
fn on_the_stack_only_the_cast_half_counts() {
    cr!("709.3b", "709.4");
    ruling!(
        "Consign // Oblivion",
        "All split cards have two card faces on a single card, and you put a split card onto the stack with only the half you're casting. The characteristics of the half of the card you didn't cast are ignored while the spell is on the stack."
    );
    ruling!(
        "Claim // Fame",
        "All split cards have two card faces on a single card, and you put a split card onto the stack with only the half you’re casting."
    );
    supported("Iona, Shield of Emeria");
    let mut t = TestGame::new(2);
    // "Your opponents can't cast spells of the chosen color": black. Consign // Oblivion is
    // a blue and black card, but Consign is a blue spell.
    choose_color(&mut t, P1, Color::Black);
    t.enter(P1, "Iona, Shield of Emeria");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let co = t.hand(P0, "Consign // Oblivion");
    let spell = t.cast(P0, co).method(CastMethod::Half(0)).target(bears).go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Consign");
    assert_eq!(c.colors, ColorSet::single(Color::Blue));
    assert!(c.is(CardType::Instant) && !c.is(CardType::Sorcery));
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Claim // Fame is black and red; with red chosen, Claim (black) can be cast.
    let mut t = TestGame::new(2);
    choose_color(&mut t, P1, Color::Red);
    t.enter(P1, "Iona, Shield of Emeria");
    t.graveyard(P0, "Grizzly Bears");
    let bears = t.g.player(P0).graveyard[0];
    t.lands(P0, "Swamp", 1);
    let cf = t.hand(P0, "Claim // Fame");
    t.cast(P0, cf).method(CastMethod::Half(0)).target(bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_split_card_name_is_one_of_its_two_names() {
    cr!("709.4a");
    ruling!(
        "Dusk // Dawn",
        "Each split card has two names. If an effect instructs you to choose a card name, you may choose one, but not both."
    );
    supported("Meddling Mage");
    let mut t = TestGame::new(2);
    // "As Meddling Mage enters, choose a nonland card name. Spells with the chosen name
    // can't be cast." Both names at once isn't a card name.
    t.answer(P0, DecisionKind::Name, Answer::Text("Dusk // Dawn".into()));
    let mage = t.enter(P0, "Meddling Mage");
    assert_eq!(t.obj_now(mage).choices.card_name.as_deref(), Some(""));
    // Naming Dawn: Dawn can't be cast from the graveyard, but Dusk can be cast.
    t.answer(P0, DecisionKind::Name, Answer::Text("Dawn".into()));
    let mage = t.enter(P0, "Meddling Mage");
    assert_eq!(t.obj_now(mage).choices.card_name.as_deref(), Some("Dawn"));
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 5);
    let gy = t.graveyard(P1, "Dusk // Dawn");
    assert!(!cast_methods(&mut t, P1, gy).contains(&AFTERMATH));
    let hand = t.hand(P1, "Dusk // Dawn");
    assert!(cast_methods(&mut t, P1, hand).contains(&FIRST_HALF));
    // Without the Mage, Dawn could be cast from the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let gy = t.graveyard(P0, "Dusk // Dawn");
    assert!(cast_methods(&mut t, P0, gy).contains(&AFTERMATH));
}

/// Casts the aftermath half of the card named `name` from `p`'s graveyard the first time
/// `p` has priority with it there; otherwise answers as the scripted agent does.
struct CastAftermathFromGraveyard {
    inner: Box<dyn Agent>,
    name: &'static str,
    done: bool,
}

impl Agent for CastAftermathFromGraveyard {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        let a = self.inner.decide(g, p, d);
        if !self.done && matches!(d, Decision::Priority { .. }) {
            if let Some(card) = g
                .player(p)
                .graveyard
                .iter()
                .find(|c| g.obj(**c).chars.name == self.name)
            {
                self.done = true;
                return Answer::Action(Action::Cast {
                    card: *card,
                    method: AFTERMATH,
                });
            }
        }
        a
    }
}

/// P0 casts the first half of `name` from hand (with `targets`); the game then runs with
/// both players passing, except that P0 casts the aftermath half as soon as they can.
/// Returns whether P1 was ever given priority while the card was in the graveyard.
fn cast_both_halves(t: &mut TestGame, name: &'static str, targets: &[Entity]) -> bool {
    let card = t.hand(P0, name);
    t.cast(P0, card).method(FIRST_HALF).targets(targets).go();
    let seen = watch(
        t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        split_card_in_a_graveyard,
    );
    {
        let mut agents = t.g.agents.0.lock().unwrap();
        let inner = std::mem::replace(
            &mut agents[P0.idx()],
            Box::new(mtg_engine::decision::PassiveAgent),
        );
        agents[P0.idx()] = Box::new(CastAftermathFromGraveyard {
            inner,
            name,
            done: false,
        });
    }
    let ok = t
        .g
        .run_until(500, |g| g.exile.iter().any(|c| g.obj(*c).chars.name == name));
    assert!(ok, "{}", t.dump_log());
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    seen.iter().any(|in_gy| *in_gy)
}

#[test]
fn the_aftermath_half_can_be_cast_right_after_the_first_half_resolves() {
    cr!("117.3b", "702.127a");
    ruling!(
        "Cut // Ribbons",
        "If you cast the first half of a split card with aftermath during your turn, you'll have priority immediately after it resolves. You can cast the half with aftermath from your graveyard before any player can take any other action if it's legal for you to do so."
    );
    ruling!(
        "Claim // Fame",
        "If you cast the first half of a split card with aftermath during your turn, you’ll have priority immediately after it resolves."
    );
    // Cut deals 4 damage to the Giant; then Ribbons (X = 1).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    assert!(!cast_both_halves(&mut t, "Cut // Ribbons", &[Entity::Object(giant)]));
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 19);
    // Claim returns the Bears; Fame gives it +2/+0 and haste.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 2);
    assert!(!cast_both_halves(&mut t, "Claim // Fame", &[Entity::Object(bears)]));
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn an_aftermath_card_moves_to_the_stack_as_soon_as_casting_begins() {
    cr!("601.2a", "702.127a");
    ruling!(
        "Cut // Ribbons",
        "Once you've started to cast a spell with aftermath from your graveyard, the card is immediately moved to the stack."
    );
    ruling!(
        "Claim // Fame",
        "Once you’ve started to cast a spell with aftermath from your graveyard, the card is immediately moved to the stack."
    );
    ruling!(
        "Road // Ruin",
        "Once you've started to cast a spell with aftermath from your graveyard, the card is immediately moved to the stack. Opponents can't try to stop the ability by exiling the card with another effect."
    );
    for (name, land, n) in [
        ("Cut // Ribbons", "Swamp", 3),
        ("Claim // Fame", "Mountain", 2),
        ("Road // Ruin", "Mountain", 3),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, land, n);
        let card = t.graveyard(P0, name);
        // Choosing X or targets (CR 601.2b–c) happens after the card is on the stack.
        let seen = watch(
            &mut t,
            P0,
            |d| matches!(d, Decision::ChooseTargets { .. } | Decision::ChooseX { .. }),
            |g| {
                let on_stack = g.stack.last().is_some_and(|s| g.obj(*s).is_spell());
                (on_stack, split_card_in_a_graveyard(g))
            },
        );
        if name.starts_with("Cut") {
            t.answer(P0, DecisionKind::X, Answer::Number(1));
        } else {
            t.answer_targets(P0, &[Entity::Object(bears)]);
        }
        t.cast(P0, card).method(AFTERMATH).go();
        let seen = seen.lock().unwrap().clone();
        assert!(!seen.is_empty(), "{name}");
        assert!(
            seen.iter().all(|(stack, gy)| *stack && !*gy),
            "{name}: {seen:?}"
        );
    }
}

#[test]
fn claim_fame_aftermath_half_only_from_a_graveyard() {
    cr!("702.127a");
    ruling!(
        "Claim // Fame",
        "If another effect allows you to cast a split card with aftermath from any zone other than a graveyard, you can’t cast the half with aftermath."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 2);
    t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    let card = t.exile(P0, "Claim // Fame");
    grant_play_permission(&mut t.g, P0, vec![card], Duration::EndOfTurn, false, None);
    let methods = cast_methods(&mut t, P0, card);
    assert!(methods.contains(&FIRST_HALF));
    assert!(!methods.contains(&SECOND_HALF) && !methods.contains(&AFTERMATH));
}

#[test]
fn claim_fame_either_half_from_a_graveyard_with_another_permission() {
    cr!("702.127a");
    ruling!(
        "Claim // Fame",
        "If another effect allows you to cast a split card with aftermath from a graveyard, you may cast either half. If you cast the half that has aftermath, you’ll exile the card if it would leave the stack."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Claim // Fame");
    grant_play_permission(&mut t.g, P0, vec![card], Duration::EndOfTurn, false, None);
    let methods = cast_methods(&mut t, P0, card);
    assert!(methods.contains(&FIRST_HALF));
    // Claim cast from the graveyard this way goes back to the graveyard.
    t.cast(P0, card)
        .method(FIRST_HALF)
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Claim // Fame"));
    // Fame cast with such a permission is exiled.
    let card = t.g.player(P0).graveyard[0];
    grant_play_permission(&mut t.g, P0, vec![card], Duration::EndOfTurn, false, None);
    t.lands(P0, "Mountain", 2);
    assert!(cast_methods(&mut t, P0, card).contains(&SECOND_HALF));
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.cast(P0, card)
        .method(SECOND_HALF)
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    assert!(t.in_exile("Claim // Fame"));
}

#[test]
fn an_aftermath_spell_is_exiled_however_it_leaves_the_stack() {
    cr!("702.127a", "608.2b");
    ruling!(
        "Claim // Fame",
        "A spell with aftermath cast from a graveyard will always be exiled afterward, whether it resolves, it’s countered, or it leaves the stack in some other way."
    );
    // Countered.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let card = t.graveyard(P0, "Claim // Fame");
    let spell = t.cast(P0, card).method(AFTERMATH).target(bears).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_exile("Claim // Fame"));
    assert!(!t.in_graveyard(P0, "Claim // Fame"));
    // Its only target became illegal: it doesn't resolve, and is exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let card = t.graveyard(P0, "Claim // Fame");
    t.cast(P0, card).method(AFTERMATH).target(bears).go();
    t.lands(P1, "Island", 1);
    let unsummon = t.hand(P1, "Unsummon");
    t.cast(P1, unsummon).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_exile("Claim // Fame"));
    // Resolved.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let card = t.graveyard(P0, "Claim // Fame");
    t.cast(P0, card).method(AFTERMATH).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t.in_exile("Claim // Fame"));
}
