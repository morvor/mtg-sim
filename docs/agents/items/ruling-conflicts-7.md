# ruling-conflicts-7: "for each player, choose target permanent that player controls" when a player controls none

Run at effort max, with an independent investigator subagent and a separate adversarial skeptic subagent, as in ruling-conflicts-1. The user's standing rule: the CR and rulings should not contradict, and apparent conflicts are investigated before the user decides.

**Raised by:** rulings-P138 (landed b6fb0f3c). The Vaevictis ruling is left OPEN.

**The conflict.**
- Vaevictis Asmadi, the Dire (Oracle: "Whenever Vaevictis Asmadi attacks, for each player, choose target permanent that player controls. Those players sacrifice those permanents. ..."). Ruling (2018-07-13): "If any player controls no permanents, Vaevictis's ability won't have sufficient legal targets, so it's removed from the stack without any effect."
- The engine (src/per_player_targets.rs) gives a player with no legal choice no target and still puts the ability on the stack. That policy cites the Sylvan Primordial ruling (2014-02-01): "If an opponent has any legal permanents to target, you must target one of them."
- Mass Mutiny ("up to one target"), ruling (2012-06-01): "You can cast Mass Mutiny even if an opponent doesn't control any creatures. You simply won't choose a target for that opponent."
- CR 601.2c: "... In some cases, the number of targets will be defined by the spell's text. Once the number of targets the spell has is determined, that number doesn't change, even if the information used to determine the number of targets does." CR 603.3d: "... If a choice is required when the triggered ability goes on the stack but no legal choices can be made for it, or if a rule or a continuous effect otherwise makes the ability illegal, the ability is simply removed from the stack." Also check 115.1, 115.6, 601.2c in full, and 115.10-ish rules on "for each" targets.

**Survey.** Find every card with "for each [player/opponent], [choose] target ... that player controls" without "up to" (Sylvan Primordial, Blatant Thievery, Chaos Is My Plaything, Turf War, Guff Rewrites History, Frenzied Gorespawn, Vaevictis, and others). Collect every ruling on what happens when one player has no legal target: removed from the stack, fewer targets, or can't be cast. Group them by date and by spell versus triggered ability. For a spell, 601.2c's "can't be cast" may apply instead of 603.3d's removal.

**Outcome.**
- **The rulings and CR agree** on a rule (likely: a required target for each player means a spell can't be cast and a triggered ability is removed from the stack when any player has no legal target):
  - fix per_player_targets.rs generally;
  - cite the rulings;
  - add a register entry if a judgment remains;
  - report.
- **They contradict:** change nothing, and report both readings with every source quoted verbatim with dates for the user to decide.
- Either way, give the per-player required-target policy a register entry (the P138 reviewer found it has none).
- Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
