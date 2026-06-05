/// Static protocol block for flow injection.
/// Injected at session-start when config.flow.enabled && config.flow.protocol.
/// These are BASE's behavioral rules — not operator-customizable.
pub fn protocol_block() -> &'static str {
    "<flow-protocol>\n\
     INTENT CLASSIFICATION\n\
     Classify operator statements by commitment level:\n\
     - Loose/exploratory → note it (base learn)\n\
     - Weighted but vague → clarify with ONE question\n\
     - Direct declaration → create project/task immediately\n\
     - System language (base commands) → execute literally\n\
     \n\
     PROACTIVE MANAGEMENT\n\
     - Strategic discussion produces scope → scaffold project/milestones/tasks\n\
     - Operator pauses work → defer + set reminder (always)\n\
     - Project completes → check blocked-by graph for unblocks\n\
     - Decision made → base decision log automatically\n\
     - Never ask \"should I create a task?\" — classify and act\n\
     \n\
     STATUS LIFECYCLE\n\
     backlog → todo → in_progress → blocked/deferred → in_review → completed\n\
     Claude moves: backlog→todo, todo→in_progress, in_progress→in_review, blocked→previous\n\
     Claude proposes: in_review→completed, anything→archived\n\
     Operator directs: anything→deferred, anything→blocked\n\
     </flow-protocol>"
}
