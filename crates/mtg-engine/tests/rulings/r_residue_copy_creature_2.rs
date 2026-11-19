//! Rulings residue of batches P020, P021 and P023 — a permanent that enters as a copy of
//! a creature, or a token copy of one, has the copied creature's "enters" triggered
//! abilities and its "as this enters" and "enters with" abilities, which apply as it
//! enters (CR 707.2, 603.6a, 614.1c, 614.12); a token copy that enters attacking was never
//! declared as an attacker (CR 508.4, 508.3a).
//!
//! The copied creature is Aven Riftwatcher ("Vanishing 3" — it enters with three time
//! counters — and "When this creature enters or leaves the battlefield, you gain 2
//! life.").

use crate::r_p020_common::RIFTWATCHER;
use crate::r_residue_copy_creature::*;
use crate::r_s01_common::{supported, tokens};
use crate::r_s18_common::lands_for;
use crate::r_s26_common::new_tokens;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` activates the ability of `source` whose text contains `needle` with `targets`
/// (answered in order), resolves everything and returns the tokens `p` got.
fn activate_for_tokens(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
    targets: &[Entity],
) -> Vec<ObjectId> {
    let before = t.g.battlefield.clone();
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    crate::r_s06_common::activate_containing(t, p, source, needle).expect("activation failed");
    t.resolve_all();
    new_tokens(t, p, &before)
}

/// One case of [`riftwatcher_cases`]: the copier and how it makes its copies of Aven
/// Riftwatcher (given a fresh game; returns the copies and the life P0 gained otherwise,
/// from the Riftwatcher leaving the battlefield).
struct EtbCase {
    card: &'static str,
    make: fn(&mut TestGame) -> (Vec<ObjectId>, i32),
}

/// Runs each case: every copy entered with three time counters (the copied "enters with"
/// ability applied) and its "enters" ability triggered and resolved (2 life each).
fn riftwatcher_cases(cases: &[EtbCase]) {
    for c in cases {
        supported(c.card);
        let mut t = TestGame::new(2);
        let life = t.life(P0);
        let (copies, other) = (c.make)(&mut t);
        assert!(
            !copies.is_empty(),
            "{}: no copies: {}",
            c.card,
            t.dump_log()
        );
        for id in &copies {
            let o = t.obj_now(*id);
            assert_eq!(o.chars.name, RIFTWATCHER, "{}", c.card);
            assert_eq!(
                t.counters(*id, counters::TIME),
                3,
                "{}: the copied \"enters with\" ability didn't apply",
                c.card
            );
        }
        assert_eq!(
            t.life(P0),
            life + other + 2 * copies.len() as i32,
            "{}: the copied \"enters\" triggers: {}",
            c.card,
            t.dump_log()
        );
    }
}

/// P0's Aven Riftwatcher on the battlefield (placed, not entered: no trigger).
fn rift(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, RIFTWATCHER)
}

#[test]
fn copies_have_the_copied_creatures_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c", "614.12", "707.9b");
    ruling!(
        "Altered Ego",
        "Any \"enters\" abilities of the copied creature will trigger when Altered Ego enters the battlefield. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the chosen creature will also work."
    );
    ruling!(
        "Pirated Copy",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Pirated Copy enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    ruling!(
        "Sakashima of a Thousand Faces",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Sakashima enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    ruling!(
        "Wall of Stolen Identity",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Wall of Stolen Identity enters the battlefield. Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the chosen creature will also work."
    );
    ruling!(
        "Hulking Metamorph",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when Hulking Metamorph enters the battlefield. Any \"as [this] enters the battlefield\" or \"[this] enters the battlefield with\" abilities of the chosen permanent will also work."
    );
    ruling!(
        "Mockingbird",
        "Any “enters” abilities of the copied creature will trigger when Mockingbird enters. Any “as [this creature] enters” or “[this creature] enters with” abilities of the copied creature will also work."
    );
    ruling!(
        "Machine God's Effigy",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Machine God’s Effigy enters the battlefield. Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the chosen creature will also work. Note that some of these effects may function correctly only if it is a creature, and will not do anything productive since the copy is an artifact, not a creature."
    );
    ruling!(
        "Mirrormind Crown",
        "Any enters abilities of the copied creature will trigger when the tokens enter. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the copied creature will also work."
    );
    ruling!(
        "Rionya, Fire Dancer",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the tokens enter the battlefield. Any \"As [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the copied creature will also work."
    );
    ruling!(
        "Mystic Reflection",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the permanents enter the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the target creature will also work."
    );
    ruling!(
        "Hofri Ghostforge",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"As [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the creature will also work."
    );
    ruling!(
        "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied creature card will also work."
    );
    ruling!(
        "Saw in Half",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the tokens enter the battlefield. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the copied creature will also work."
    );
    riftwatcher_cases(&[
        EtbCase {
            card: "Altered Ego",
            make: |t| {
                let r = rift(t);
                (copy_with(t, "Altered Ego", r), 0)
            },
        },
        EtbCase {
            card: "Pirated Copy",
            make: |t| {
                let r = rift(t);
                (copy_with(t, "Pirated Copy", r), 0)
            },
        },
        EtbCase {
            card: "Sakashima of a Thousand Faces",
            make: |t| {
                let r = rift(t);
                (copy_with(t, "Sakashima of a Thousand Faces", r), 0)
            },
        },
        EtbCase {
            card: "Wall of Stolen Identity",
            make: |t| {
                let r = rift(t);
                (copy_with(t, "Wall of Stolen Identity", r), 0)
            },
        },
        EtbCase {
            card: "Hulking Metamorph",
            make: |t| {
                let r = rift(t);
                (copy_with(t, "Hulking Metamorph", r), 0)
            },
        },
        EtbCase {
            card: "Mockingbird",
            make: |t| {
                let r = rift(t);
                (copy_with(t, "Mockingbird", r), 0)
            },
        },
        // An artifact, not a creature: the time counters and the life still come.
        EtbCase {
            card: "Machine God's Effigy",
            make: |t| {
                let r = rift(t);
                let e = crate::r_p023_common::enter_copying(t, P0, "Machine God's Effigy", r);
                t.resolve_all();
                assert!(!t.obj_now(e).is(CardType::Creature));
                (vec![e], 0)
            },
        },
        EtbCase {
            card: "Mirrormind Crown",
            make: |t| {
                let r = rift(t);
                let c = copy_with(t, "Mirrormind Crown", r);
                assert_eq!(c.len(), 2);
                (c, 0)
            },
        },
        EtbCase {
            card: "Rionya, Fire Dancer",
            make: |t| {
                let r = rift(t);
                (copy_with(t, "Rionya, Fire Dancer", r), 0)
            },
        },
        // "Choose target nonlegendary creature. The next time one or more creatures or
        // planeswalkers enter this turn, they enter as copies of the chosen creature." A
        // Grizzly Bears enters.
        EtbCase {
            card: "Mystic Reflection",
            make: |t| {
                let r = rift(t);
                lands_for(t, P0, "{1}{U}");
                let m = t.hand(P0, "Mystic Reflection");
                t.cast(P0, m).target(obj(r)).go();
                t.resolve_all();
                let b = t.enter(P0, "Grizzly Bears");
                t.settle();
                t.resolve_all();
                (vec![t.g.current(b)], 0)
            },
        },
        // "Whenever another nontoken creature you control dies, exile it. If you do,
        // create a token that's a copy of that creature, except it's a Spirit ..." The
        // Riftwatcher dying gains 2 life too.
        EtbCase {
            card: "Hofri Ghostforge",
            make: |t| {
                t.battlefield(P0, "Hofri Ghostforge");
                let r = rift(t);
                let before = t.g.battlefield.clone();
                crate::r_s02_common::destroy(t, r);
                t.resolve_all();
                (new_tokens(t, P0, &before), 2)
            },
        },
        // Reflection of Kiki-Jiki: "{1}, {T}: Create a token that's a copy of another
        // target nonlegendary creature you control, except it has haste."
        EtbCase {
            card: "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki",
            make: |t| {
                let kiki = crate::r_p026_common::reflection(t);
                let r = rift(t);
                lands_for(t, P0, "{1}");
                (
                    activate_for_tokens(t, P0, kiki, "Create a token", &[obj(r)]),
                    0,
                )
            },
        },
        // Two tokens; the Riftwatcher dying gains 2 life too.
        EtbCase {
            card: "Saw in Half",
            make: |t| {
                let r = rift(t);
                let c = copy_with(t, "Saw in Half", r);
                assert_eq!(c.len(), 2);
                (c, 2)
            },
        },
    ]);
}

#[test]
fn token_copies_of_cards_have_the_copied_cards_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c", "707.9b");
    ruling!(
        "Foggy Swamp Visions",
        "Any enters abilities of the copied card will trigger when the token enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the chosen creature will also work."
    );
    ruling!(
        "Dino DNA",
        "Any enters-the-battlefield abilities of the copied card will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the copied card will also work."
    );
    ruling!(
        "Soul Separator",
        "Any “enters” abilities of the copied creature card will trigger when the Spirit token enters. Any “as [this creature] enters” or “[this creature] enters with” abilities of the creature card will also work."
    );
    riftwatcher_cases(&[
        // "As an additional cost to cast this spell, waterbend {X}. Exile X target
        // creature cards from graveyards. For each creature card exiled this way, create a
        // token that's a copy of it." X = 1.
        EtbCase {
            card: "Foggy Swamp Visions",
            make: |t| {
                let card = t.graveyard(P0, RIFTWATCHER);
                lands_for(t, P0, "{2}{B}{B}");
                let s = t.hand(P0, "Foggy Swamp Visions");
                let before = t.g.battlefield.clone();
                t.cast(P0, s).x(1).target(obj(card)).go();
                t.resolve_all();
                (new_tokens(t, P0, &before), 0)
            },
        },
        // "Imprint — {1}, {T}: Exile target creature card from a graveyard. ... {6}:
        // Create a token that's a copy of target creature card exiled with this artifact,
        // except it's a 6/6 green Dinosaur creature with trample."
        EtbCase {
            card: "Dino DNA",
            make: |t| {
                let dna = t.battlefield(P0, "Dino DNA");
                let card = t.graveyard(P1, RIFTWATCHER);
                lands_for(t, P0, "{1}");
                t.activate(P0, dna, 0, &[obj(card)]).expect("imprint");
                t.resolve_all();
                let exiled = t.g.current(card);
                t.g.untap(dna);
                lands_for(t, P0, "{6}");
                let toks = activate_for_tokens(t, P0, dna, "Create a token", &[obj(exiled)]);
                assert_eq!(t.pt(toks[0]), (6, 6));
                (toks, 0)
            },
        },
        // "{5}, {T}, Sacrifice this artifact: Exile target creature card from your
        // graveyard. Create a token that's a copy of that card, except it's 1/1, it's a
        // Spirit in addition to its other types, and it has flying. Create a black Zombie
        // ..." (Only the Spirit is a copy.)
        EtbCase {
            card: "Soul Separator",
            make: |t| {
                let sep = t.battlefield(P0, "Soul Separator");
                let card = t.graveyard(P0, RIFTWATCHER);
                lands_for(t, P0, "{5}");
                let toks = activate_for_tokens(t, P0, sep, "Exile target", &[obj(card)]);
                let spirits: Vec<ObjectId> = toks
                    .into_iter()
                    .filter(|id| has_subtype(t, *id, "Spirit"))
                    .collect();
                (spirits, 0)
            },
        },
    ]);
}

#[test]
fn march_of_progress_token_has_the_copied_artifact_creatures_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c", "702.43a");
    ruling!(
        "March of Progress",
        "Any enters-the-battlefield abilities of the copied artifact creature will trigger when the token enters the battlefield. Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the chosen creature will also work."
    );
    // "Choose target artifact creature you control. For each creature chosen this way,
    // create a token that's a copy of it." Arcbound Shikari: "When this creature enters,
    // put a +1/+1 counter on each other artifact creature you control. Modular 2 (This
    // creature enters with two +1/+1 counters on it. ...)"
    supported("March of Progress");
    supported("Arcbound Shikari");
    let mut t = TestGame::new(2);
    let shikari = t.battlefield(P0, "Arcbound Shikari");
    t.g.add_counters(obj(shikari), counters::PLUS1, 2, None);
    t.g.recompute();
    let toks = cast_for_tokens(&mut t, P0, "March of Progress", "{2}{U}", &[obj(shikari)]);
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Arcbound Shikari");
    // Modular: it entered with two counters. Its trigger put one on the original.
    assert_eq!(t.counters(toks[0], counters::PLUS1), 2);
    assert_eq!(t.counters(shikari, counters::PLUS1), 3);
}

#[test]
fn siegebreaker_and_gyrus_tokens_have_the_copied_cards_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c", "508.4");
    ruling!(
        "Mardu Siegebreaker",
        "Any “enters” abilities of the copied card will trigger when the tokens enter. Any “as this creature enters” or “this creature enters with” abilities of the copied card will also work."
    );
    ruling!(
        "Gyrus, Waker of Corpses",
        "Any enters-the-battlefield abilities of the copied creature card will trigger when the token enters the battlefield. Any \"as [this creature card] enters the battlefield\" or \"[this creature card] enters the battlefield with\" abilities of the chosen creature card will also work."
    );
    riftwatcher_cases(&[
        // "When this creature enters, exile up to one other target creature you control
        // until this creature leaves the battlefield. / Whenever this creature attacks,
        // for each opponent, create a tapped token that's a copy of the exiled card
        // attacking that opponent." The Riftwatcher leaving gains 2 life.
        EtbCase {
            card: "Mardu Siegebreaker",
            make: |t| {
                let r = rift(t);
                t.answer_targets(P0, &[obj(r)]);
                let sb = t.enter(P0, "Mardu Siegebreaker");
                t.settle();
                t.resolve_all();
                assert!(t.in_exile(RIFTWATCHER));
                let sb = t.g.current(sb);
                let before = t.g.battlefield.clone();
                attack_p1(t, &[sb]);
                let toks = new_tokens(t, P0, &before);
                for id in &toks {
                    assert!(t.g.is_attacking(*id) && t.obj_now(*id).tapped);
                }
                (toks, 2)
            },
        },
        // "Whenever Gyrus attacks, you may exile target creature card with lesser power
        // from your graveyard. If you do, create a token that's a copy of that card and
        // that's tapped and attacking."
        EtbCase {
            card: "Gyrus, Waker of Corpses",
            make: |t| {
                let gyrus = t.battlefield(P0, "Gyrus, Waker of Corpses");
                t.g.add_counters(obj(gyrus), counters::PLUS1, 5, None);
                t.g.recompute();
                let card = t.graveyard(P0, RIFTWATCHER);
                t.answer_targets(P0, &[obj(card)]);
                t.answer_yes(P0, true);
                let before = t.g.battlefield.clone();
                attack_p1(t, &[gyrus]);
                let toks = new_tokens(t, P0, &before);
                for id in &toks {
                    assert!(t.g.is_attacking(*id));
                }
                (toks, 0)
            },
        },
    ]);
}

// --- Tokens that enter attacking weren't declared as attackers ----------------------------

/// Gleam of Battle ("Whenever a creature you control attacks, put a +1/+1 counter on
/// it.") for P0.
fn gleam(t: &mut TestGame) {
    supported("Gleam of Battle");
    t.battlefield(P0, "Gleam of Battle");
}

/// The new tokens are attacking but Gleam of Battle didn't trigger for them (they have no
/// +1/+1 counters), and Brazen Wolves ("Whenever this creature attacks, it gets +2/+0
/// until end of turn.") among them didn't get +2/+0: none was declared as an attacker.
fn none_declared(t: &TestGame, toks: &[ObjectId], wolves_pt: (i32, i32)) {
    assert!(!toks.is_empty());
    for id in toks {
        assert!(t.g.is_attacking(*id), "{id:?} isn't attacking");
        assert_eq!(t.counters(*id, counters::PLUS1), 0, "{id:?} 'attacked'");
        assert_eq!(name_of(t, *id), "Brazen Wolves");
        assert_eq!(t.pt(*id), wolves_pt);
    }
}

#[test]
fn tokens_entering_attacking_werent_declared_as_attackers() {
    cr!("508.4", "508.3a", "603.2");
    ruling!(
        "Echoing Assault",
        "Although the token enters attacking, it was never declared as an attacking creature. Abilities that trigger whenever a creature attacks won't trigger when that creature enters attacking."
    );
    ruling!(
        "Mardu Siegebreaker",
        "Although the tokens enter attacking, they were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won’t trigger when those tokens enter attacking."
    );
    ruling!(
        "Mirror-Style Master",
        "Although the tokens enter the battlefield attacking, they were never declared as attacking creatures (for the purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    ruling!(
        "Altaïr Ibn-La'Ahad",
        "Although the tokens enter the battlefield attacking, they were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won't trigger when they enter the battlefield attacking."
    );
    supported("Brazen Wolves");
    // Echoing Assault: "Whenever you attack a player, choose target nontoken creature
    // that's attacking that player. Create a token that's a copy of that creature, except
    // it's 1/1. The token enters tapped and attacking that player."
    supported("Echoing Assault");
    let mut t = TestGame::new(2);
    gleam(&mut t);
    t.battlefield(P0, "Echoing Assault");
    let wolves = t.battlefield(P0, "Brazen Wolves");
    t.answer_targets(P0, &[obj(wolves)]);
    attack_p1(&mut t, &[wolves]);
    // The declared Wolves triggered both.
    assert_eq!(t.pt(wolves), (2 + 2 + 1, 3 + 1));
    let toks = tokens(&t, P0);
    none_declared(&t, &toks, (1, 1));
    // Mardu Siegebreaker exiled a Brazen Wolves as it entered.
    supported("Mardu Siegebreaker");
    let mut t = TestGame::new(2);
    gleam(&mut t);
    let wolves = t.battlefield(P0, "Brazen Wolves");
    t.answer_targets(P0, &[obj(wolves)]);
    let sb = t.enter(P0, "Mardu Siegebreaker");
    t.settle();
    t.resolve_all();
    let sb = t.g.current(sb);
    unsick(&mut t, sb);
    attack_p1(&mut t, &[sb]);
    assert_eq!(t.counters(sb, counters::PLUS1), 1);
    let toks = tokens(&t, P0);
    none_declared(&t, &toks, (2, 3));
    // Mirror-Style Master: "Whenever this creature attacks, for each attacking modified
    // creature you control, create a tapped and attacking token that's a copy of that
    // creature." The Wolves are modified: they have a +1/+1 counter.
    supported("Mirror-Style Master");
    let mut t = TestGame::new(2);
    gleam(&mut t);
    let msm = t.battlefield(P0, "Mirror-Style Master");
    let wolves = t.battlefield(P0, "Brazen Wolves");
    t.g.add_counters(obj(wolves), counters::PLUS1, 1, None);
    attack_p1(&mut t, &[msm, wolves]);
    let toks: Vec<ObjectId> = tokens(&t, P0);
    none_declared(&t, &toks, (2, 3));
    // Altaïr: "Whenever Altaïr attacks, exile up to one target Assassin creature card
    // from your graveyard with a memory counter on it. Then for each creature card you own
    // in exile with a memory counter on it, create a tapped and attacking token that's a
    // copy of it."
    supported("Altaïr Ibn-La'Ahad");
    let mut t = TestGame::new(2);
    gleam(&mut t);
    let altair = t.battlefield(P0, "Altaïr Ibn-La'Ahad");
    let e = t.exile(P0, "Brazen Wolves");
    t.g.add_counters(obj(e), "memory", 1, None);
    t.answer_targets(P0, &[]);
    attack_p1(&mut t, &[altair]);
    let toks = tokens(&t, P0);
    none_declared(&t, &toks, (2, 3));
}

fn unsick(t: &mut TestGame, id: ObjectId) {
    crate::r_p023_common::unsick(t, id);
}
