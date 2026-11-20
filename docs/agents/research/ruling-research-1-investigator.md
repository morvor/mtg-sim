# ruling-research-1: investigator's findings

Investigator (independent), 2026-11-19. Item: `docs/agents/items/ruling-research-1.md`; register entry
`control-lost-before-a-for-as-long-as-you-control-effect-begins` (docs/interpretations/triggers-durations.md, provisional).
Research only: no engine code edited, nothing built, nothing committed.

How the web was reached: every non-GitHub host is blocked by this session's proxy (CONNECT 403; WebFetch fails with
ENOTFOUND), so web pages were read only through the WebSearch tool's result summaries. Where I quote a web page,
I say whether an exact-phrase search confirmed the wording. Hosts I could read directly: github.com (git clone),
raw.githubusercontent.com, and api.github.com (search endpoints only). Through those I read the CR archive
(github.com/pit142857/mtg-cr, 130 CR texts 1999 to 2026) and the Wizards article mirror
(github.com/maxmakesmagic/ormos: Saturday School, update bulletins, release notes and FAQs, up to 2022). The list of
blocked hosts is in section 9.

---------------------------------------------------------------------------------------------------------------------

## 0. Verdict

**(A) Sources decide it.** If control of the source is lost while the ability is on the stack and regained before it
resolves, the "for as long as you control ~" effect does **nothing**. The provisional entry's reading is correct.

The decisive source is the WotC Gatherer ruling on **Seasinger**, dated **2007-09-16** (in data/rulings.jsonl.gz,
source "wotc"; the current Gatherer page gatherer.wizards.com/MED/en-us/49/seasinger still shows it, per search
results). It is about exactly this case:

> "Whether you control Seasinger is checked continually, starting when the ability is activated. If, before the
> ability resolves, there’s any point at which you don’t control Seasinger, the ability has no effect — even if you
> control Seasinger again by the time the ability resolves. The same is true regarding whether Seasinger remains
> tapped."

Seasinger's Oracle ability: "{T}: Gain control of target creature whose controller controls an Island for as long as you
control this creature and this creature remains tapped."

Five other sources back it up, and none says otherwise:
1. **CR 611.2b's current wording (since the CR effective 2024-11-08).** The effect does nothing if the duration "ends
   before the moment the effect would first be applied and doesn’t begin again during that spell or ability’s
   resolution". The clause asks whether the duration begins again *during the resolution*, not whether it holds at
   resolution. The **Foundations Update Bulletin** (magic.wizards.com, 2024-11-14, Jess Dunks and Eric Levine, read
   through search results) explains the clause: "The only change here is that this rule now explicitly handles the
   rare case where the duration for a continuous effect starts during the resolution of the spell or ability which
   created that effect." That case is Xolatoyac's own flood counter. The clause is not a check of the state at
   resolution.
2. **The July 2009 (Magic 2010) Update Bulletin** (Mark L. Gottlieb, rules manager, 2009-07-08) states the general
   principle. A "for as long as" duration, unlike an "as long as" on/off switch, "is true from the moment its ability
   goes on the stack or starts applying until the 'as long as' condition stops being true."
3. **The same model, every time it comes up.** Rulings on the untap-and-retap twin say "even if [it] becomes tapped
   again": Preacher 2007-09-16, Willow Satyr, Rubinia Soulsinger and Old Man of the Sea 2009-10-01, Entrancing Lyre
   2020-01-24. Wizards' Saturday School column said the same in 2003 and 2004 ("needs the condition to be true all
   the time, even if the ability hasn't resolved yet"), and so did the WotC Rules Team in 1997 (D'Angelo's Official
   Rules Summary, T.2.7 and E.12).
4. **Once a "for as long as you control ~" effect has begun, losing control ends it for good**, "even if you
   immediately regain control" (Time of Ice, 2018-04-27). Ten other rulings say the same: Tidebinder Mage, Dungeon
   Geists, Icefall Regent, Master Thief, Dragonlord Silumgar, Willbreaker, Shipbreaker Kraken, Merchant Raiders, Wall
   of Stolen Identity, and Ty Lee 2025-10-02.
5. **CR 702.26f and the phasing rulings.** A "for as long as" duration that tracks a permanent ends when it phases out,
   and "they will no longer affect that permanent once it’s phased in". Spectral Adversary's ruling (2021-09-24) gives
   the example "for as long as you control Mind Flayer".

**The standard resolution.** It has held since at least 1997 and is restated by the 2009 and 2024 bulletins:
- A "for as long as" duration created by a resolving spell or ability is a **period**, not an on/off condition.
- The period starts when the ability is activated or triggered and put on the stack. If the duration is set up by the
  ability's own instructions (it taps the target, or puts a counter on it), it starts as the resolution sets it up.
- The period ends at the first moment its condition is false. An ended duration never restarts, whether the effect has
  begun or not.
- If it ends before the effect would first be applied, the effect does nothing, unless the resolution itself makes the
  condition true again (611.2b's 2024 clause).

| Sub-case | Sources | Conclusion | Confidence |
|---|---|---|---|
| Control lost, not regained | CR 611.2b example (Master Thief); rulings for about 20 cards | Nothing (one-shot parts such as the tap still happen) | Certain |
| Control lost and regained before resolution | Seasinger ruling (explicit); 611.2b plus the 2024 bulletin; 2009 bulletin; Willow Satyr and Rubinia "The same is true"; Time of Ice (after it began) | Nothing | High (a direct WotC ruling, still current; nothing contrary) |
| "and ~ remains tapped": untapped and tapped again | Preacher, Seasinger, Willow Satyr, Rubinia, Old Man, Entrancing Lyre rulings; Saturday School #37 and #73; the 1997 Rules Team ruling | Nothing | Certain |
| Source leaves and returns (CR 400.7) | 400.7; rulings for Mind Flayer, Sower, Dungeon Geists, Wall of Stolen Identity, Kotose and Lightning ("since it's a new object") | Nothing; both readings agree | Certain |
| Source phases out and back in before resolution | 702.26d and f; the phasing rulings ("the duration will expire"); D'Angelo E.12.Ruling.4 ("will not restart") | Nothing | High-moderate (no ruling on this exact stack timing) |
| Duration about another object | 611.2b plus the 2024 bulletin (Xolatoyac); Old Man of the Sea (power) and Eriette rulings | False at resolution: nothing. Re-established by the resolution itself: the effect applies. False and then true again before resolution: nothing by the literal rule and Old Man's "The same is true"; open for durations the ability itself sets up | High for the first two; low-moderate for the last |

**Engine.** The current implementation gets these cases right: lost and not regained, lost and regained, untapped and
retapped, and left and returned. It has three gaps, all rare. Details are in section 8.
- (i) A source that phases out and back in while its ability is on the stack still lets the effect begin, both for
  "you control ~" and for "~ remains tapped". `changed_while_on_stack` looks only at `Untapped` and `ControlChanged`
  events.
- (ii) The window opens when the ability is put on the stack. The Tidebinder Mage and Shipbreaker Kraken release notes
  open it when the ability triggers.
- (iii) Durations about another object are checked only at resolution.

**Register.** The entry's decision stands. Its Rules field should quote the Seasinger ruling (decisive, and so far
OPEN in rulings coverage, batch P177), plus Preacher, Entrancing Lyre, Time of Ice, the July 2009 bulletin and the
Foundations bulletin. Old Man of the Sea's "even if it becomes tapped again right away" ruling (P117) is also still
OPEN. Because a ruling decides the case, the entry can become **decided**; that is the coordinator's and the user's
call.

---------------------------------------------------------------------------------------------------------------------

## 1. The Comprehensive Rules (data/comprehensive-rules.txt, "These rules are effective as of September 25, 2026.")

Quoted verbatim from the repo file.

- **611.2** "A continuous effect may be generated by the resolution of a spell or ability."
- **611.2a** "A continuous effect generated by the resolution of a spell or ability lasts as long as stated by the spell
  or ability creating it (such as “until end of turn”). If no duration is stated, it lasts until the end of the game."
- **611.2b** "Some continuous effects generated by the resolution of a spell or ability have durations worded “for as
  long as . . . .” If the “for as long as” duration never starts, the effect does nothing. Similarly, if that duration
  ends before the moment the effect would first be applied and doesn’t begin again during that spell or ability’s
  resolution, the effect does nothing. It doesn’t start and immediately stop again, and it doesn’t last forever."
  Example: "Master Thief has the ability “When this creature enters, gain control of target artifact for as long as you
  control this creature.” If you lose control of Master Thief before the ability resolves, it does nothing, because
  its duration—as long as you control Master Thief—was over before the effect began."
- **611.2c** "If a continuous effect generated by the resolution of a spell or ability modifies the characteristics or
  changes the controller of any objects, the set of objects it affects is determined when that continuous effect
  begins. After that point, the set won’t change. (Note that this works differently than a continuous effect from a
  static ability.) A continuous effect generated by the resolution of a spell or ability that doesn’t modify the
  characteristics or change the controller of any objects modifies the rules of the game, so it can affect objects that
  weren’t affected when that continuous effect began. If a single continuous effect has parts that modify the
  characteristics or changes the controller of any objects and other parts that don’t, the set of objects each part
  applies to is determined independently."
- **611.3a** (contrast: static abilities) "A continuous effect generated by a static ability isn’t “locked in”; it applies
  at any given moment to whatever its text indicates."
- **108.4** "A card doesn’t have a controller unless that card represents a permanent or spell; in those cases, its
  controller is determined by the rules for permanents or spells. See rules 110.2 and 112.2."
- **109.4** "Only objects on the stack or on the battlefield have a controller. Objects that are neither on the stack
  nor on the battlefield aren’t controlled by any player. See rule 108.4. There are six exceptions to this rule:"
  (109.4a–g follow.)
- **109.5** "The words “you” and “your” on an object refer to the object’s controller, its would-be controller (if a
  player is attempting to play, cast, or activate it), or its owner (if it has no controller). For a static ability,
  this is the current controller of the object it’s on. For an activated ability, this is the player who activated
  the ability. For a triggered ability, this is the controller of the object when the ability triggered, unless it’s a
  delayed triggered ability. To determine the controller of a delayed triggered ability, see rules 603.7d–f."
- **110.2** "A permanent’s owner is the same as the owner of the card that represents it (unless it’s a token; see rule
  111.2). A permanent’s controller is, by default, the player under whose control it entered the battlefield. Every
  permanent has a controller."
- **113.7a** "Once activated or triggered, an ability exists on the stack independently of its source. Destruction or
  removal of the source after that time won’t affect the ability. ..."
- **400.7** "An object that moves from one zone to another becomes a new object with no memory of, or relation to, its
  previous existence. This rule has the following exceptions." (400.7a–m: none covers a "for as long as" duration of
  an ability already on the stack.)
- **603.3a** "A triggered ability is controlled by the player who controlled its source at the time it triggered, unless
  it’s a delayed triggered ability. ..."
- **603.10** (look-back triggers; included for completeness): "Normally, objects that exist immediately after an event
  are checked to see if the event matched any trigger conditions, ..." and **603.10d** "Abilities that trigger when a
  player loses control of an object or when a player’s opponent gains control of an object from that player look back
  in time." (Nothing in 603.10 bears on durations.)
- **608.2h** "If an effect requires information from the game (such as the number of creatures on the battlefield), the
  answer is determined only once, when the effect is applied. If the effect requires information from a specific
  object, including the source of the ability itself, the effect uses the current information of that object if it’s
  in the public zone it was expected to be in; if it’s no longer in that zone, or if the effect has moved it from a
  public zone to a hidden zone, the effect uses the object’s last known information. ..."
- **613.1b** "Layer 2: Control-changing effects are applied."
- **702.26b** "If a permanent phases out, its status changes to “phased out.” Except for rules and effects that
  specifically mention phased-out permanents, a phased-out permanent is treated as though it does not exist. It can’t
  affect or be affected by anything else in the game. ..."
- **702.26d** "The phasing event doesn’t actually cause a permanent to change zones or control, even though it’s treated
  as though it’s not on the battlefield and not under its controller’s control while it’s phased out. ... Effects that
  check a phased-in permanent’s history won’t treat the phasing event as having caused the permanent to leave or enter
  the battlefield or its controller’s control."
- **702.26e** "If a continuous effect generated by the resolution of a spell or ability modifies the characteristics or
  changes the controller of any objects, a phased-out permanent won’t be included in the set of affected objects. This
  includes continuous effects that reference the permanent specifically, unless they also specifically refer to the
  permanent as phased out."
- **702.26f** "Continuous effects that affect a phased-out permanent may expire while that permanent is phased out. If
  so, they will no longer affect that permanent once it’s phased in. In particular, effects with “for as long as”
  durations that track that permanent (see rule 611.2b) end when that permanent phases out because they can no longer
  see it."
- **800.4a** "When a player leaves the game, all objects (see rule 109) owned by that player leave the game and any
  effects which give that player control of any objects or players end. ..." (In multiplayer, this is one way
  control of a source can come back to you mid-stack.)
- **The analogous "until" rule, 610.3a / 610.3b** (the item calls it "610.3c"; in this CR, 610.3c is "An object returned
  to the battlefield this way returns under its owner’s control unless otherwise specified."):
  - 610.3a "If a resolving spell or activated ability creates the initial one-shot effect that causes the object to
    change zones, and the specified event has already occurred before that one-shot effect would occur but after that
    spell or ability was put onto the stack, the object doesn’t move."
  - 610.3b is the same for a triggered ability, "but after that ability triggered".
  - 610.4b and 610.4c say the same for phasing "until".
  - These rules are event-based too, and their window also opens when the ability is put on the stack (for a
    triggered ability, when it triggered).

**Every "for as long as" in the CR** (grep): 116.2c, 611.2b and its example, 701.20a (revealed), 701.40a and 701.58a
(manifested and cloaked "for as long as it remains face down"), 701.60c (suspected), 701.65a (airbend), 702.26f,
702.95a (soulbond "for as long as both remain creatures on the battlefield under your control"), 702.99a (cipher),
702.185a (warp), 715.3d (adventure), 722.3c (prepared). Apart from 611.2b's example, none decides the timing question.
**"remains tapped"** appears nowhere in the CR. The CR's only example on this rule is Master Thief, which covers
control lost and not regained.

---------------------------------------------------------------------------------------------------------------------

## 2. History of the rule (archived CRs: github.com/pit142857/mtg-cr, "Comprehensive Rules/CR-*.txt"; quotes normalized to straight quotes)

| CR (file date) | Rule text (changes only) |
|---|---|
| D'Angelo, *Official Magic: The Gathering Rules Summary*, Fifth Edition rules, final release, 04/21/99 (file CR-1999-04-21.txt; "Stephen D'Angelo ... Official Magic: The Gathering Rules Summary, Network Representative for Wizards of the Coast, Inc.") | **T.2.7** "If the duration of a duration effect is only until a certain condition happens and that condition is met before the ability generating the effect resolves, the effect fails to happen in the first place. [WotC Rules Team 08/05/97] [Duelist Magazine #20, Page 35] For example, if you take control of a creature with Seasinger, and the Seasinger is untapped before the ability resolves, the ability fails to do anything and you never get control of the creature." / **E.12.2** "These abilities are duration effects. The effect lasts until the card is untapped. This is similar to a normal time duration effect such as "until end of turn", but is "until the card which generated the effect stops being tapped". [D'Angelo 09/12/95]" / **E.12.Ruling.2** "If the card untaps before the tap and hold ability actually resolves, the tap and hold effect never takes effect at all. [WotC Rules Team 08/05/97]" / **E.12.Ruling.4** "If one of these cards or its target leaves play temporarily by phasing out (see Rule G.30) or otherwise leaving play, the effect will end and will not restart when it re-enters play. [D'Angelo 10/15/96]" |
| CR current as of 2001-09-24 | **418.3d** "If an effect's duration is over before it would begin, the effect does nothing-it doesn't start and immediately stop again, and it doesn't last forever." Example: "Endoskeleton is an artifact with an activated ability that reads "2, T: Target creature gets +0/+3 as long as Endoskeleton remains tapped." If you play this ability and then Endoskeleton becomes untapped before the ability resolves, it does nothing, because its duration-remaining tapped-was over before the effect began." |
| 2002-02-20 / 2002-10-07 ... 2007-10-01 | **418.3d** "Some effects from activated or triggered abilities have durations worded "as long as . . . ." If the "as long as" duration ends between the end of announcing [2003-10: "playing"] the activated ability or putting the triggered ability onto the stack and the moment when the effect would first be applied, the effect does nothing. It doesn't start and immediately stop again, and it doesn't last forever." (Endoskeleton example.) |
| 2008-02-01 (Morningtide) | **418.3d** "... If the "as long as" duration ends before the moment the effect would first be applied, the effect does nothing. ..." Explained by the **Morningtide Update Bulletin** (Mark L. Gottlieb, 2008-01-16, magic.wizards.com/en/articles/archive/feature/morningtide-update-bulletin-2008-01-16; ormos path archive/en/articles/archive/feature/2008/01/morningtide-update-bulletin-2008-01-16.md): "**418.3d** This rule discusses what happens if an "as long as" duration ends before the effect that cares about it even starts. However, it was overly specific about the window of time, and there were still cases in which an "as long as" duration could end before the stated window began. So the rule's being broadened." |
| 2009-07-08 file ("effective as of July 11, 2009", Magic 2010) | **610.2b** "Some continuous effects generated by the resolution of a spell or ability have durations worded "for as long as . . . ." If the "for as long as" duration never starts, or it ends before the moment the effect would first be applied, the effect does nothing. It doesn't start and immediately stop again, and it doesn't last forever." (Endoskeleton example, now "for as long as".) The phasing rule **702.23e** (today's 702.26f) first appears in this CR with its "for as long as" sentence. Explained by the **July 2009 Update Bulletin** (below). |
| 2009-10-05 (Zendikar) | Renumbered to **611.2b**, same text. |
| 2011-09-30 (Innistrad) | Example changed to Master Thief ("If you lose control of Master Thief before the ability resolves, it does nothing, because its duration -- as long as you control Master Thief -- was over before the effect began.") |
| **2024-11-08 (Foundations)** | **611.2b** "... If the "for as long as" duration never starts, the effect does nothing. Similarly, if that duration ends before the moment the effect would first be applied and doesn't begin again during that spell or ability's resolution, the effect does nothing. ..." (The CR effective 2024-09-20 still had the 2009 text. This update also removed damage assignment order; nothing else in it touches durations.) |
| 2025-02-07 | Example re-templated ("When this creature enters, ... for as long as you control this creature."). Unchanged through the repo's CR effective 2026-09-25. |

**July 2009 Update Bulletin** (Mark L. Gottlieb, 2009-07-08,
magic.wizards.com/en/articles/archive/feature/july-2009-update-bulletin-2009-07-08; ormos path
archive/en/articles/archive/feature/2009/07/july-2009-update-bulletin-2009-07-08.md), verbatim:

> "### Effects that last "as long as" a condition is met now last "for as long as" that condition is met.
> There are two kinds of "as long as" abilities. One was an independent on/off switch (basically, the "as long as"
> could be replaced by "if"). For example, "Ashenmoor Cohort gets +1/+1 as long as you control another black
> creature." These weren't changed, though in some cases, the "as long as" clause was moved to the front of the
> ability for clarity.
> In the other kind, "as long as" represents a duration that's set up by an earlier part of the same ability or by the
> ability's activation cost. That kind of "as long as" effect is true from the moment its ability goes on the stack or
> starts applying until the "as long as" condition stops being true. For example, Aven Mimeomancer's "At the beginning
> of your upkeep, you may put a feather counter on target creature. If you do, that creature is 3/1 and has flying as
> long as it has a feather counter on it." All 62 of these changed to say "for as long as" (except for Orcish Squatters,
> which said "for as long as" all along). Some got sentences rejiggered in the process so they'd sound better.
> Nothing functionally changed here. It's a small addition, but it should help make these kinds of abilities clearer."

Its CR section says: "**610.2b** This rule is about effects with durations worded "for as long as ...". The important
change is that these durations used to just be worded "as long as ...", which made them easy to confuse with "as long
as" effects from static abilities."

**Foundations Update Bulletin**
(https://magic.wizards.com/en/news/announcements/foundations-update-bulletin; search results date it 2024-11-14 and
name Jess Dunks and Eric Levine as authors). The host is blocked, so I couldn't read the page itself. These sentences
come from WebSearch results; I ran an exact-phrase search on part of each, and every one matched this page:
- "Normally, these cases are pretty straightforward—if the duration never starts, or if it ends before the effect
  would be applied, the effect isn't applied." (exact-phrase search on "Normally, these cases are pretty
  straightforward" matched this page)
- "The only change here is that this rule now explicitly handles the rare case where the duration for a continuous
  effect starts during the resolution of the spell or ability which created that effect." (exact-phrase search on
  "explicitly handles the rare case where the duration" matched this page; the tool returned the whole sentence)
- The Xolatoyac example: you put Xolatoyac, the Smiling Flood's first ability on the stack "targeting a land that
  already has a flood counter" (exact phrase matched). If another effect removes the existing flood counter before the
  ability resolves, "the previous wording of 611.2b could have been read to mean that Xolatoyac's ability doesn't make
  that land an Island on resolution even though the ability itself puts a flood counter on the land" (exact phrase
  "could have been read to mean that Xolatoyac's ability" matched). The new wording makes it clear that the land does
  become an Island.
- Sower of Temptation example (paraphrased by the search tool, no exact-phrase check except "gets destroyed in response
  to its triggered ability"): if you cast Sower of Temptation and it gets destroyed in response to its triggered
  ability, you won't gain control of the target creature as that ability resolves.

**What the 2024 clause is for.** It covers an ability whose own resolution makes the condition true again before it
creates the effect: it puts a counter on the target (Xolatoyac, Aquitect's Will, Eluge, Quicksilver Fountain, Aven
Mimeomancer, Bant, Cyclopean Tomb, Mathas, Minas Morgul, Obsidian Fireheart, Sauron Dino Devotee, Shield Broker,
Ultima) or taps it (Braided Net, Zygon Infiltrator). For about 15 cards, the duration's condition could be broken
while the ability waits and then restored by the resolution. **No card with a source-based duration ("for as long as
you control ~" or "~ remains tapped") regains control of, or retaps, its own source during that ability's
resolution**; I scanned the Oracle texts in data/oracle_cards.jsonl.gz. The parun-mtg engine's issue #6027
(github.com/parun-mtg/parun/issues/6027, 2026-11-03) reaches the same conclusion from MTGJSON. So for the cards in this
item, the clause never applies, and a duration broken while the ability is on the stack means the effect does nothing.

**Can a duration "end" before it has started?** Yes; the CR has always said so. The 2001 rule: "If an effect's duration
is over before it would begin"; the 2002–2007 rule measured "between the end of announcing the activated ability or
putting the triggered ability onto the stack and the moment when the effect would first be applied"; and the 2009
bulletin: "true from the moment its ability goes on the stack". The duration is running while the ability waits; the
effect is what has not begun. The 2024 bulletin calls Xolatoyac's case the rare one where the duration "starts during
the resolution", which implies that normally the duration starts earlier.

---------------------------------------------------------------------------------------------------------------------

## 3. The rulings data (data/rulings.jsonl.gz; all quoted rulings have source "wotc")

### 3a. Rulings that decide or bear directly on the question (verbatim)

**Control lost and regained before resolution**
- **Seasinger (2007-09-16):** "Whether you control Seasinger is checked continually, starting when the ability is
  activated. If, before the ability resolves, there’s any point at which you don’t control Seasinger, the ability has
  no effect — even if you control Seasinger again by the time the ability resolves. The same is true regarding whether
  Seasinger remains tapped."
- Seasinger (2007-09-16), the contrast with a targeting condition: "If a creature’s controller doesn’t control an
  Island, that creature is an illegal target. Whether that player controls an Island is checked both when Seasinger’s
  ability is activated and when it resolves. However, targeting restrictions aren’t checked continually while a spell
  or ability is on the stack."
- Seasinger (2004-10-04): "You do not lose control of the stolen creature if it stops being a creature. You only lose
  control if Seasinger leaves your control or becomes untapped."
- **Willow Satyr (2009-10-01)** and **Rubinia Soulsinger (2009-10-01)**, same text: "If Willow Satyr stops being tapped
  before its ability resolves — even if it becomes tapped again right away — you won’t gain control of the targeted
  creature at all. The same is true if you lose control of Willow Satyr before the ability resolves."

**Untapped and tapped again before resolution**
- **Preacher (2007-09-16):** "Whether Preacher is tapped is checked continually, starting when the ability is activated.
  If, before the ability resolves, there’s any point at which Preacher is untapped, the ability has no effect — even if
  Preacher is tapped again by the time the ability resolves."
- **Entrancing Lyre (2020-01-24):** "If Entrancing Lyre becomes untapped before its activated ability resolves, the
  target creature is tapped, but it can untap as usual. This is true even if Entrancing Lyre becomes tapped again before
  the activated ability resolves." (Also in the Theros Beyond Death Release Notes, 2020-01-10; ormos
  archive/en/articles/archive/feature/2020/01/theros-beyond-death-release-notes-2020-01-10.md line 1478.)
- **Old Man of the Sea (2009-10-01):** "If Old Man of the Sea stops being tapped before its ability resolves — even if it
  becomes tapped again right away — you won’t gain control of the targeted creature at all. The same is true if the
  targeted creature’s power becomes greater than Old Man of the Sea’s power before the ability resolves."
- **Vedalken Shackles (2004-12-01):** "If Vedalken Shackles becomes untapped before its ability resolves, you won’t gain
  control of the creature." Also: "The creature’s power and the number of Islands you control matter only when the
  ability is activated and when it resolves. After that, only the fact that Vedalken Shackles remains tapped matters.
  If the creature stops being a creature, you’ll keep control of it."
- Grand Marshal Macie (2022-10-07): "If Macie becomes untapped before her last ability resolves, you won’t choose any
  effect to not end."

**Control lost and not regained** (the effect does nothing; one-shot parts such as the tap still happen)
- Master Thief (2011-09-22): "If Master Thief ceases to be under your control before its ability resolves, you won’t
  gain control of the targeted artifact at all."
- Dragonlord Silumgar (2015-02-25): "If Dragonlord Silumgar ceases to be under your control before its ability resolves,
  you won’t gain control of the creature or planeswalker at all."
- Willbreaker (2015-06-22): "If you lose control of Willbreaker before its ability resolves, you won’t gain control of
  the creature at all."
- Olivia Voldaren (2017-03-14): "If you activate Olivia Voldaren’s last ability, and before that ability resolves you
  lose control of Olivia Voldaren, the ability will resolve with no effect. You won’t gain control of the targeted
  Vampire."
- Tidebinder Mage (2013-07-01): "If you lose control of Tidebinder Mage between when it entered the battlefield and when
  the ability resolves, the target creature will become tapped. However, it won’t be affected by the other effect and
  will untap as normal during its controller’s untap step."
- Shipbreaker Kraken, in the Theros Release Notes (2013-09-16; ormos .../2013/09/theros-release-notes-2013-09-16.md:2427):
  "If you lose control of Shipbreaker Kraken between when its last ability triggers and when that ability resolves, the
  target creatures will become tapped. However, they won't be affected by the other effect and will untap as normal
  during their controllers' untap steps."
- Orcish Squatters (2008-10-01): "If you lose control of Orcish Squatters before its ability resolves, you won’t gain
  control of the targeted land at all."
- The Wretched (2009-10-01): "If you lose control of The Wretched before its ability resolves, you won’t gain control of
  the creatures blocking it at all."
- Roil Elemental (2009-10-01): "If Roil Elemental ceases to be under your control before its ability resolves, you
  won't gain control of the targeted creature at all."
- Kellogg, Dangerous Mind (2024-03-08): "If you lose control of Kellogg before its last ability resolves, you won't gain
  control of the target creature at all."
- Mind Flayer (2021-07-23): "If Mind Flayer leaves the battlefield or you lose control of it before its Dominate Monster
  ability resolves, control of the target creature doesn't change at all."
- Scroll of Isildur (2023-06-16): "If Scroll of Isildur leaves the battlefield or you lose control of it before its first
  chapter ability resolves, control of the target artifact doesn't change at all."
- Thancred Waters (2025-06-06): "If Thancred Waters leaves the battlefield or you lose control of it before its triggered
  ability resolves, the target legendary permanent won't gain indestructible at all."
- Yuffie, Materia Hunter (2025-06-06): "If you lose control of Yuffie or it leaves the battlefield before its last
  ability resolves, you won't gain control of the target noncreature artifact."
- Ty Lee, Chi Blocker (2025-10-02): "If Ty Lee, Chi Blocker ceases to be under your control before its ability resolves,
  you will still tap the targeted creature, but it will untap normally during its controller's next untap step."

**Once begun, losing control ends the effect for good, even if control is regained**
- **Time of Ice (2018-04-27):** "The effects of Time of Ice's first two chapter abilities expire if you lose control of
  it, even if you immediately regain control of it or cast another Time of Ice." (Already cited in
  tests/rulings/r_p133_cards.rs.)
- Tidebinder Mage (2013-07-01): "If another player gains control of Tidebinder Mage, the creature will no longer be
  affected by the ability preventing it from untapping, even if you later regain control of Tidebinder Mage."
- The same for Dungeon Geists (2012 FAQ text; 2019-07-12: "If another player gains control of Dungeon Geists, its effect
  expires. It won’t keep the creature from untapping anymore, even if you later regain control of Dungeon Geists."),
  Icefall Regent (2015-02-25), Shipbreaker Kraken (2013-09-15), Merchant Raiders (2020-11-10), Wall of Stolen Identity
  (2019-08-23), Aegis Angel (2013-07-01), Quicksmith Rebel and Quicksmith Spy (2017-02-09), Kotose (2022-02-18) and
  Lightning, Security Sergeant (2025-06-06).
- Master Thief (2011-09-22): "If another player gains control of Master Thief, its control-change effect ends.
  Regaining control of Master Thief won’t cause you to regain control of the artifact." The same for Dragonlord
  Silumgar (2015-02-25) and Willbreaker (2015-06-22).
- Ty Lee, Chi Blocker (2025-10-02): "If another player gains control of Ty Lee, Chi Blocker, her effect ends. Regaining
  control of Ty Lee, Chi Blocker won't cause the effect to resume."

**Source leaves (and returns)**
- Sower of Temptation (2007-10-01): "If Sower of Temptation leaves the battlefield before the ability resolves, the
  ability will have no effect."
- Dungeon Geists (2019-07-12): "If Dungeon Geists leaves the battlefield before its triggered ability has resolved, the
  target creature will be tapped, but it will be able to untap as normal."
- Wall of Stolen Identity (2019-08-23): "If Wall of Stolen Identity leaves the battlefield before its triggered ability
  has resolved, the copied creature will be tapped, but it will untap as normal."
- Kotose, the Silent Spider (2022-02-18): "... Similarly, if Kotose leaves the battlefield and later returns, you won't
  be able to play the cards that were exiled the first time it was on the battlefield."
- Lightning, Security Sergeant (2025-06-06): "... Similarly, if Lightning leaves the battlefield and later returns, you
  won't be able to play the cards that were exiled the first time it was on the battlefield since it's a new object."
- Rulings of the same shape ("leaves the battlefield before its ... ability resolves ... at all"): Giant's Grasp
  (2021-02-05), The Akroan War (2020-01-24), Vault 87 (2024-03-08), Pyreswipe Hawk (2024-07-26), Hedge Whisperer
  (2024-02-02), Tide Shaper (2021-06-18), Awakener Druid (2009-10-01), Cytoplast Manipulator (2006-05-01),
  Opportunistic Dragon (2019-10-04), Unctus's Retrofitter (2023-02-04), Kitesail Larcenist (2023-11-10), Welcome to . . .
  (2023-11-10), Nihiloor (2021-07-23), Extraction Specialist (2022-04-29), Assimilation Aegis (2024-04-12), Exchange of
  Words (2022-10-07).

**Phasing** (17 texts on about 40 cards; earliest Teferi's Protection 2017-08-25)
- Spectral Adversary (2021-09-24): "Any continuous effects with a "for as long as" duration ignore phased-out objects. If
  ignoring those objects causes the effect's conditions to no longer be met, the duration will expire. For example,
  phasing out Mind Flayer will cause the duration of its continuous effect ("for as long as you control Mind Flayer")
  to expire."
- Teferi's Protection (2017-08-25): "Any continuous effects with a "for as long as" duration such as that of Mathas, Fiend
  Seeker ignore phased-out objects. Any such effects will expire if their conditions are no longer met after ignoring
  the phased-out objects."
- The same appears on The Phasing of Zhalfir (2022-09-09), The Pandorica (2023-10-13), Guardian of Faith, Blink Dog and
  Clever Concealment, and others.

**Durations about another object**
- Old Man of the Sea's power clause (2009-10-01; quoted above): "The same is true if the targeted creature’s power
  becomes greater than Old Man of the Sea’s power before the ability resolves."
- Eriette, the Beguiler (2024-04-12): "If the Aura you control becomes unattached from a nonland permanent (most likely
  because the Aura left the battlefield) before Eriette’s triggered ability resolves, you won’t gain control of that
  nonland permanent at all."
- Rootwater Matriarch (2004-10-04): "Can target an unenchanted creature but has no effect on that creature."
  (2009-02-01): "The control effect applies so long as at least one Aura is attached to the creature, and will continue
  to apply even if Rootwater Matriarch has left the battlefield. The effect ends if at any time the creature has no
  Auras on it."
- Xolatoyac, the Smiling Flood (2023-11-10): "The land stays an Island until the flood counter is removed, even if
  Xolatoyac, the Smiling Flood leaves the battlefield." (The 2024 bulletin's example; see section 2.)
- Contrast, a targeting restriction rather than a duration: Wellgabber Apothecary (2007-10-01): "The targeted creature
  must be tapped when the ability is activated and when it resolves. It doesn’t need to remain tapped in between."

I found no ruling saying a "for as long as" duration is judged only by the state at resolution, and none saying
regaining control, or retapping, before resolution lets the effect begin.

### 3b. Cards with these durations (data/oracle_cards.jsonl.gz; 259 cards contain "for as long as")
Classified by what the duration tracks (script output in the scratch file faly_classified.txt):

- **A1. "for as long as you control ~" (the source): 47 cards.**
  - Control changes: Aladdin, Dragonlord Silumgar, Kellogg Dangerous Mind, Master Thief, Merieke Ri Berit, Mind Flayer,
    Nihiloor, Olivia Voldaren, Orcish Squatters, Possession Engine, Pyreswipe Hawk, Rangers of Ithilien, Roil Elemental,
    Scroll of Isildur, The Wretched, Thrull Champion, Vault 87: Forced Evolution, Vodalian Mindsinger, Willbreaker,
    Yuffie Materia Hunter.
  - Untap locks: Dungeon Geists, Icefall Regent, Lukamina Crocodile Form, Merchant Raiders, Shipbreaker Kraken,
    Spider-Woman Secret Agent ("can't become untapped"), Tidebinder Mage, Time of Ice, Ty Lee Chi Blocker, Wall of
    Stolen Identity.
  - Ability or characteristic modifications: Aegis Angel, Kyodai Soul of Kamigawa, Thancred Waters and Tale of Tinúviel
    (indestructible); Quicksmith Rebel and Quicksmith Spy (granted abilities); Welcome to . . . // Jurassic Park
    (becomes a 0/4 Wall).
  - Restrictions and rules: Extraction Specialist and Possession Engine ("can't attack or block"), There and Back Again
    ("can't block"), The Phasing of Zhalfir ("can't phase in"), Jin-Gitaxias // The Great Synthesis (no maximum hand
    size).
  - Play permissions: Gwen Stacy, Hama the Bloodbender, Kotose, Lightning Security Sergeant, Taster of Wares, Victor
    Mancha.
- **A2. "for as long as you control ~ and ~ remains tapped": 5 cards.** Helm of Possession, Hivis of the Scale,
  Rubinia Soulsinger, Seasinger, Willow Satyr. All are control changes.
- **B. "for as long as ~ remains tapped" (the source): 34 cards.**
  - Control changes: Callous Oppressor, Old Man of the Sea (with a power clause about the target), Preacher, Vedalken
    Shackles.
  - Untap locks: Amber Prison, Deserter's Quarters, Entrancing Lyre, Giant Oyster, Ice Floe, Kill Switch, Mana Leech,
    Mole Worms, Phyrexian Gremlins, Rust Tick, Sand Squid, Thalakos Dreamsower, Whip Vine.
  - P/T or ability modifications: Ashnod's Battle Gear, Endoskeleton, Everglove, Flamestick, Frightshroud, Ghosthelm and
    Pearlspear Couriers, Flowstone Armor, Hisoka's Guard, Spirit Shield, Tawnos's Weaponry, Thran Weaponry, Zelyon
    Sword.
  - Other: Immovable Rod, Hedge Whisperer and The Blackstaff of Waterdeep (animate), The Pandorica (phasing).
- **Durations about another object.**
  - "it/that creature remains tapped": Braided Net, Zygon Infiltrator.
  - "has a [kind] counter on it": 20 cards, among them Xolatoyac, Aquitect's Will, Eluge, Quicksilver Fountain,
    Immortal Obligation, Mathas, Shield Broker.
  - "enchanted" or "attached": Rootwater Matriarch, Eriette, Assimilation Aegis, Blade of Shared Souls.
  - Player state: Garland ("they're the monarch").
- **Other durations.** "remains on the battlefield" (35 cards, e.g. Sower of Temptation, Opportunistic Dragon); "card
  remains exiled" play permissions (82 cards); soulbond and battlebond reminder text (25 cards); and a few others.

---------------------------------------------------------------------------------------------------------------------

## 4. Wizards articles (ormos mirror: github.com/maxmakesmagic/ormos, archive/en/...)

- **Saturday School #37** (Rune Horvik, 2003-08-02; magic.wizards.com/en/articles/archive/feature/saturday-school-37-2003-08-02).
  - Q: Sand Squid with Pemmin's Aura: tap an opponent's creature, untap the Squid in response, do this multiple times,
    "tapping the Squid at the end".
  - A: "This is not a combo. If you untap the Squid in response to its ability, the effect's duration is considered to
    have ended before it began, so it will only tap the creature. Rule 418.3d in the Comprehensive Rulebook deals with
    effects that have a duration “as long as” some condition is true. If the condition ends between the ability having
    been activated and it would be applied, it never starts. It won't last forever."
- **Saturday School #73** (Rune Horvik, 2004-04-10; .../saturday-school-73-2004-04-10).
  - Q: Callous Oppressor with Pemmin's Aura: tap it to take one Exalted Angel; without letting the ability resolve,
    untap it with the Aura and tap it to take the second.
  - A: "You can only control one creature at a time with Callous Oppressor. So-called "tap and hold" abilities, which
    have a duration "as long as…" some condition is true, needs the condition to be true all the time, even if the
    ability hasn't resolved yet. If the Oppressor is untapped before the ability resolves, the effect never happens.
    When you untap the Oppressor after having targeted the first Angel, the "tapped" condition isn't true anymore, and
    the ability will be unable to gain control over that target. You will only gain control over the second Angel. This
    is explained in rule 418.3d in the Comprehensive Rulebook."
  - This is the untap-and-retap case: the Oppressor is tapped again when the first ability resolves.
- **Saturday School #4** (Rune Horvik, 2002-11-16), Callous Oppressor killed in response: "... when the Oppressor goes
  away before the ability resolves, you will never get control over the other creature. This ability is a so-called
  “Tap and hold” ability, and if the “as long as” part stops being true before the effect resolves, the effect never
  happens. ..."
- **Saturday School #79** (Rune Horvik, 2004-05-22), Vedalken Shackles: "Like other cards with the same wording, it's not
  possible to use Vedalken Shackles to gain control of more than one creature – once the Shackles are untapped, the
  effect ends immediately, and if the Shackles are untapped before the effect starts, it does nothing. This is
  explained in rule 418.3d in the Comprehensive Rulebook."
- **"May Days"** (John Carter, 2005-04-30), Vedalken Shackles Oxidized in response: "The creature stays where it is. “As
  long as” sets a duration, and if a duration ends before the effect begins, the effect does nothing [CR 418.3d]."
- The **Morningtide** and **July 2009** update bulletins are in section 2.
- **Release notes and FAQs** (M12 FAQ 2011-07-06; Innistrad FAQ 2011-09-16; Dark Ascension FAQ 2012-01-19; M14 FAQ
  2013-07-09; Theros 2013-09-16; Dragons of Tarkir 2015-03-18; Magic Origins 2015-07-08; Aether Revolt 2017-01-06; Modern
  Masters 2017 2017-03-03; M20 2019-06-25; Commander 2019 2019-08-09; Theros Beyond Death 2020-01-10; Commander Legends
  2020-11-06; Adventures in the Forgotten Realms 2021-07-09; Kamigawa Neon Dynasty 2022-02-09; Double Masters 2022
  2022-06-24) carry the same texts as the Gatherer rulings in section 3a. None addresses regaining control before
  resolution; the Gatherer ruling on Seasinger does.
- The 2007-09-16 Seasinger and Preacher rulings date from Masters Edition (released on MTGO 2007-09-10; both cards'
  representative printing in oracle_cards is Masters Edition). The 2009-10-01 Willow Satyr, Rubinia and Old Man rulings
  date from Masters Edition III (2009-09-07). (Seasinger is a paper card: the ormos articles list it in Fallen Empires
  and Fifth Edition; see section 8.)

---------------------------------------------------------------------------------------------------------------------

## 5. Other web sources (read through WebSearch; the hosts are blocked)

- **Magic Judges Forum, Rules Q&A topic 9831**, "Continuous effects with "for as long as ~ remains tapped""
  (https://apps.magicjudges.org/forum/topic/9831/; opened 2014-05-01 by Mario Haßler; the opening post is also
  apps.magicjudges.org/forum/post/61148).
  - The question argued the **state reading**. Amber Prison is tapped to target creature A, untapped, then tapped again
    to target creature B. The poster's guess was that neither creature untaps, because each ability sees a tapped
    Prison when it resolves. The search tool's paraphrase: "The continuous effect shouldn't care about what happened
    before, it just should check at the very moment it is created" (no exact-phrase match), and he suggested 611.2b say
    this more precisely.
  - Two searches reported an answer in the thread: "It does not matter that the Prison became tapped again before the
    ability resolves, what matters is that it became untapped before the effect first applied". A later search found no
    reply. **Unverified.** It is the only judge forum thread I found on the exact question, and its reported answer is
    the period reading.
- **MTG Salvation rules threads** (www.mtgsalvation.com; content reported by the search tool):
  - "Remains Tapped vs Responce untapping" (thread 265243): Rubinia untapped in response; "the duration ... has already
    ended", and so the first steal fails.
  - "Rubinia Soulsinger + untaps" (514230).
  - "Vedalken Shackles and Voltaic Key" (283544): "what matters is that it was not tapped the entire time".
  - "Amber Prison..." (288155): "even if you re-tap it before the next untap phase, the entire continuous effect came to
    an end when its stated duration ended".
  - All are community answers giving the period reading.
- **Cranial Insertion** (www.cranial-insertion.com; article number not confirmed), on Dungeon Geists bounced in response:
  "that duration is going to end before it even begins, and rule 611.2b says that when that happens, the effect doesn't
  even try to start!" (exact phrase matched a CI article).
- **MTG wiki** (mtg.wiki/page/Duration, redirecting to "Continuous effect"; also mtg.fandom.com/wiki/Continuous_effect):
  reproduces 611.2b and the Master Thief example. Nothing on the regain case.
- **Reddit r/mtgrules**: no thread on the regain case surfaced. The search tool listed a HuggingFace dataset of
  r/mtgrules Q&A, but huggingface.co is blocked.
- **Other engines** (GitHub, readable):
  - parun-mtg/parun issue #6027 (2026-11-03), "A "for as long as" duration can't begin again during its resolution",
    reads the 2024 clause as covering a duration restored by the resolution itself and says no printed card does that.
    PR #6029 ends such durations after each instruction.
  - XMage and Forge trackers: nothing on this case.
- **Nothing found on the web for the state reading**, other than the 2014 forum question.

---------------------------------------------------------------------------------------------------------------------

## 6. The general principle, with analogies

1. **A period, not a switch.** The 2009 bulletin sets "for as long as" durations, which are "set up by an earlier part
   of the same ability or by the ability's activation cost" and "true from the moment its ability goes on the stack or
   starts applying until the "as long as" condition stops being true", against static "as long as" on/off switches.
   CR 611.3a (static effects aren't locked in) against 611.2c (resolution effects are) says the same thing for affected
   objects.
2. **An ended duration never restarts.**
   - Before the effect begins: Seasinger, Preacher, Willow Satyr, Rubinia, Old Man, Entrancing Lyre; Saturday School;
     the WotC Rules Team ruling of 08/05/97.
   - After it begins: Time of Ice ("even if you immediately regain control of it"); Tidebinder, Dungeon Geists, Icefall
     Regent, Master Thief, Silumgar, Willbreaker, Ty Lee; 702.26f ("they will no longer affect that permanent once it’s
     phased in"); D'Angelo E.12.Ruling.4 ("will not restart when it re-enters play").
3. **When the period starts.** When the ability is activated, or put on the stack, or (for a triggered ability) when it
   triggered: the 2002–2007 418.3d window; Seasinger's and Preacher's rulings "starting when the ability is activated";
   Tidebinder's "between when it entered the battlefield and when the ability resolves"; Kraken's "between when its
   last ability triggers and when that ability resolves". The 2008 broadening (Morningtide bulletin) removed the
   stated window so that a duration ending even earlier, such as between triggering and being put on the stack, still
   counts. For a duration set up by the resolution itself, the period starts as the resolution sets it up (the 2009
   bulletin's "or starts applying"; the 2024 clause).
4. **Analogy: "until" effects (610.3a, 610.3b, 610.4b, 610.4c).** If the "until" event happened after the ability was put
   on the stack (or triggered), the object doesn't move; whether the event could happen again doesn't matter. This is
   the same history-based test with the same window.
5. **Analogy: targeting conditions are checked at two moments only**, as Seasinger's targeting ruling, Vedalken Shackles
   and Wellgabber Apothecary say. Seasinger's two rulings put the two side by side: the targeting condition at
   activation and at resolution, "However, targeting restrictions aren’t checked continually"; the duration "checked
   continually, starting when the ability is activated".

---------------------------------------------------------------------------------------------------------------------

## 7. The sub-cases, weighed

1. **Control lost, not regained.** Both readings agree: the effect does nothing (CR 611.2b's own example; about 20
   rulings). One-shot parts of the ability still happen (Tidebinder: "the target creature will become tapped"; Ty Lee;
   Dungeon Geists). **Certain.**

2. **Control lost and regained before resolution.**
   - For "does nothing": the Seasinger ruling (WotC, explicit, still on Gatherer); the literal 611.2b ("doesn’t begin
     again during that spell or ability’s resolution"); the Foundations bulletin, which confines that clause to a
     duration that "starts during the resolution"; the 2009 bulletin's principle; Willow Satyr and Rubinia's "The same
     is true if you lose control ... before the ability resolves", which parallels "even if it becomes tapped again
     right away"; Time of Ice's "even if you immediately regain control of it" (same family, effect already begun).
   - For "begins": the plain-English reading of "for as long as you control ~", which holds at resolution; the 2014
     forum question's "check at the very moment it is created"; and a possible distinction between "remains tapped"
     ("remains" sounds historical) and "you control" (sounds like a state).
   - Assessment: the plain reading isn't what the CR does with these durations (2001 on), and the possible distinction
     is ruled out by Seasinger's ruling. That ruling treats the control half of its duration separately from the tapped
     half ("Whether you control Seasinger is checked continually ... The same is true regarding whether Seasinger
     remains tapped").
   - Caveat: the ruling is about Seasinger's conjunctive duration and dates from 2007. The CR changes since then
     broadened the rule (2008), renamed and reworded it (2009), and added only the during-resolution clause (2024,
     "the only change here"). None of them is a reason to read a plain "for as long as you control ~" differently.
   - **Does nothing. High confidence.**

3. **"and ~ remains tapped": untapped and tapped again.** The rulings are unanimous (Preacher, Seasinger, Willow Satyr,
   Rubinia, Old Man of the Sea, Entrancing Lyre), and so are the CR's own Endoskeleton example (2001–2011), Saturday
   School #37 and #73, and the WotC Rules Team ruling of 1997. **Does nothing. Certain.**

4. **The source leaves and returns.** Under CR 400.7 the returned permanent is a new object; "this creature" in the
   ability is the old one, which is gone for good, so the condition is false at resolution too. Rulings: Mind Flayer,
   Sower, Dungeon Geists, Wall of Stolen Identity, and Kotose and Lightning ("since it's a new object"); the 1999
   summary E.12.Ruling.4. Both readings agree. The ability's one-shot parts still happen. **Does nothing. Certain.**

5. **The source phases out and back in before resolution.** While it's phased out, the source "is treated as though it
   does not exist" and "not under its controller’s control" (702.26b, 702.26d). An existing effect's "for as long as"
   duration "end[s] when that permanent phases out", and the effect doesn't come back when it phases in (702.26f; the
   phasing rulings: "the duration will expire"; E.12.Ruling.4 "will not restart"). By 611.2b, a duration that expired
   while the ability was on the stack means the effect does nothing.
   - Against: 702.26d says "Effects that check a phased-in permanent’s history won’t treat the phasing event as having
     caused the permanent to leave ... its controller’s control". That concerns history checks ("controlled
     continuously", "entered this turn"), and 702.26f deals with "for as long as" durations specifically.
   - No ruling covers phasing while the ability is on the stack. It's rare: phasing back in mid-turn needs an effect
     like The Pandorica or an "until" phase-out (610.4).
   - **Does nothing. High-moderate confidence.**

6. **A duration about another object** ("for as long as that creature remains tapped / is enchanted / has a counter on
   it", Old Man's power clause, Eriette's "that Aura is attached to it").
   - (a) False at resolution: nothing (Old Man power ruling, Eriette, Rootwater Matriarch). **Certain.**
   - (b) Made true again by the resolution itself (Xolatoyac's counter, Braided Net's tap): the effect applies (611.2b
     since 2024; the Foundations bulletin). **Certain.**
   - (c) False and then true again before resolution, not by the resolution. The literal 611.2b says nothing happens
     (it ended and didn't begin again *during* the resolution). Old Man of the Sea's "The same is true" sentence points
     the same way for a condition about the target that the ability doesn't set up.
   - For durations the ability itself sets up (Xolatoyac, Braided Net, Zygon Infiltrator), the 2009 bulletin's "or
     starts applying" suggests the period only starts at resolution, so earlier history wouldn't matter. Consider
     Xolatoyac targeting a land whose counter is removed and then replaced before resolution: the literal 611.2b says
     nothing happens, the 2009 bulletin says it applies.
   - **(c): low-moderate confidence**; no source addresses it.

---------------------------------------------------------------------------------------------------------------------

## 8. How the engine implements it now (read, not built or run)

- `crates/mtg-engine/src/untap_choice.rs`:
  - `ended_before_it_began(g, d, ctx)`:
    - For `Duration::WhileCondition(Condition::Custom(SOURCE_REMAINS_TAPPED | CONTROL_AND_SOURCE_REMAINS_TAPPED))`, the
      duration ended if `custom_condition` is false. That function checks: source live, on the battlefield, not phased
      out, tapped, (controlled by the ability's controller), and no `changed_while_on_stack` event.
    - For `Duration::WhileYouControlSource`, it calls `changed_while_on_stack(g, s, a, false, Some(ctx.controller))`.
  - `changed_while_on_stack` scans `turn_events` and `events` for the ability's `AbilityActivated` or
    `AbilityTriggeredOnStack` event, then for an `Untapped` event of the source (when `untapped`) or a `ControlChanged`
    of the source from the ability's controller. It applies only to an ability whose source is `src`.
- Callers, each after `effect_expired` (the state at resolution: source live, on the battlefield, not phased out,
  controlled by the ability's controller):
  - `resolve.rs` `Effect::Modify` (line about 1056) and `Effect::AddRestriction` (about 1169);
  - `control_change.rs` `plan_gain_control` (about 50).
  - Durations about the affected objects (`about_affected`, i.e. `Duration::WhileCondition(_)` that isn't the custom
    source-tapped condition) are checked for those objects at resolution only.
- Tests:
  - `tests/rulings/r_p086_untap_locks.rs`: `tidebinder_mage_control_lost_and_regained_before_its_trigger_resolves_only_taps`,
    `willow_satyr_untapped_or_stolen_before_resolution_gains_nothing` (also untap-and-retap) and
    `olivia_voldaren_control_lost_and_regained_before_resolution_gains_nothing`, all `interp!` of the provisional entry.
  - `tests/cards/permanent_status_control.rs`: `preacher_untapped_before_the_ability_resolves_does_nothing` (untap, then
    tap; cites the Preacher ruling).
  - `tests/rulings/r_p133_cards.rs` cites Time of Ice's ruling.
- **Matches the sources:** lost and not regained; lost and regained; untapped and retapped; left and returned (`is_live`
  is false).
- **Gaps against the sources (all rare):**
  - (i) Phasing out and back in while the ability is on the stack lets the effect begin, both for `WhileYouControlSource`
    and for the remains-tapped conditions. `Event::PhasedOut` exists in events.rs, but `changed_while_on_stack` doesn't
    look for it, and `effect_expired` sees the phased-in source.
  - (ii) The window opens at `AbilityActivated` or `AbilityTriggeredOnStack`. A control change and change back between
    the trigger event and the trigger being put on the stack is missed (the Tidebinder and Kraken texts open the window
    at the trigger). This can happen only within a single resolution.
  - (iii) Durations about another object use the state at resolution: case 6(c) applies the effect where the literal
    611.2b says nothing happens.
- **Rulings coverage** (`python3 scripts/rulings_batches.py card ...`):
  - OPEN: Seasinger's "Whether you control Seasinger is checked continually ..." (P177), its targeting ruling (P176)
    and its 2004 ruling "You only lose control if Seasinger leaves your control or becomes untapped" (P177); Old Man of
    the Sea's "even if it becomes tapped again right away ... The same is true if the targeted creature’s power becomes
    greater" (P117).
  - CITED: Preacher's "checked continually" (P177); Entrancing Lyre's "even if ... tapped again" (P096); Willow Satyr's
    (P086) and Rubinia's (P176) "even if it becomes tapped again right away"; Time of Ice's "even if you immediately
    regain control" (P133); Spectral Adversary's phasing ruling (P128).
  - Seasinger's tier in docs/verified-cards.tsv is "compiled" (failed: round-trip; structure 1 of 3 unexercised).
    TestGame doesn't check tiers, so a Seasinger test is possible. A test citing its ruling (control lost and regained,
    and untapped and retapped) would be the natural cr!/ruling! anchor for a decided entry. Old Man's P117 ruling could
    anchor the about-another-object power clause.
- **Seasinger is a paper card.** The ormos articles list it in Fallen Empires ("Under the Sea", 2002-02-06) and Fifth
  Edition (Pro Tour Houston 2002 artist table), and it is legal in Legacy, Vintage and Commander.
  docs/agents/items/cards-digital-alchemy.texts.tsv line 877 calls it "digital", probably because the oracle_cards
  printing is Masters Edition (MTGO). That looks like a misclassification; worth telling the coordinator.

---------------------------------------------------------------------------------------------------------------------

## 9. Blocked hosts and access

- **Blocked by the proxy** (CONNECT 403 with curl; WebFetch reports ENOTFOUND for magic.wizards.com and yawgatog.com):
  - Wizards: magic.wizards.com, media.wizards.com, gatherer.wizards.com.
  - Rules sites and wikis: yawgatog.com, www.cranial-insertion.com, cranial-insertion.com, www.mtgsalvation.com,
    mtg.fandom.com, mtg.wiki, wiki.mtg-wiki.org, mtg-rules.vercel.app, cardboard.readthedocs.io, mtgpl.readthedocs.io.
  - Judges and forums: blogs.magicjudges.org, apps.magicjudges.org, boards.magicjudges.org, judgeacademy.com,
    www.judgeacademy.com, forum.judgeacademy.com, www.reddit.com, old.reddit.com, tappedout.net, www.mtg-forum.de,
    www.mtgnexus.com, feddit.it.
  - Card databases and stores: scryfall.com, api.scryfall.com, mtg.wtf, mtgassist.com, www.mtgassist.com,
    articles.starcitygames.com, mtgrocks.com, www.mtggoldfish.com, draftsim.com, www.tcgplayer.com,
    infinite.tcgplayer.com, www.magiclibrarities.net, aetherhub.com.
  - Archives and other: web.archive.org, archive.org, huggingface.co, datasets-server.huggingface.co,
    cdn-lfs.huggingface.co, www.duckduckgo.com, html.duckduckgo.com, www.bing.com, www.google.com.
- I used no archive or mirror of a blocked page. The ormos repository is an independent GitHub archive of old Wizards
  articles, not a proxy of the blocked site.
- **Reachable:** github.com (git clone of public repos), raw.githubusercontent.com, and api.github.com search endpoints.
  `repos/...` endpoints of repos not attached to this session answer "GitHub access to this repository is not enabled
  for this session" (HTTP 403).

## 10. Local artifacts (scratchpad; may be cleared)

/tmp/claude-0/-home-user-mtg-sim/1c53f6e8-8b42-5b35-8a6f-8e8bc17948ea/scratchpad/:
- crarchive/mtg-cr: the CR archive clone.
- crnorm: CR texts normalized to UTF-8.
- faly_all.tsv and faly_classified.txt: the "for as long as" cards.
- duration_rulings.txt: those cards' rulings.
- ormos: the article mirror clone (8.4 GB), deleted after use. Re-clone it from
  https://github.com/maxmakesmagic/ormos to check the quoted paths.

## 11. Status

Complete. The investigation survived one container restart: the work done before it was redone from the saved
scratch files, and this file was written afterwards.
