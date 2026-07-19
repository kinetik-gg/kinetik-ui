import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const SPEC_SHA256 = "f1d489f6f28b613c0bcfa4490b7855da341457ee20c66c892dc37ebff2d024ed";
const COMPONENTS = [
  "button", "text-field", "dropdown", "selection-controls", "value-controls",
  "progress-feedback", "overlay-system", "virtual-list", "editor-frame",
  "workspace-chrome", "dock", "inspector-collections", "node-graph", "timeline",
  "viewport", "color-picker", "gradient-editor", "content-structure-components",
  "icon-shortcut-components", "toolbar-components", "menu-components",
  "command-palette-components", "advanced-editor-fields", "choice-value-components",
  "feedback-status-components", "overlay-components", "navigation-surface-components",
  "collection-components", "inspector-components", "editor-chrome-components",
  "color-components", "timeline-components", "node-components", "viewport-components",
];
const JOURNEYS = [
  ["workspace-boot-and-traversal", "edit-workspace"],
  ["shared-action-projection", "edit-workspace"],
  ["collection-to-inspector-edit", "edit-workspace"],
  ["timeline-and-viewport-edit", "edit-workspace"],
  ["color-and-gradient-edit", "edit-workspace"],
  ["graph-connection-edit", "graph-workspace"],
  ["overlay-and-failure-recovery", "edit-workspace"],
];
const GATES = [
  "public-consumer-boundary", "canonical-component-composition",
  "complete-component-coverage", "deterministic-user-journeys", "semantic-structure",
  "renderer-and-scale-quality", "platform-integration", "honest-evidence",
];

const options = parseArgs(process.argv.slice(2));
const evidencePath = resolve(options.evidence ?? fail("--evidence is required"));
const sourceRef = options.sourceRef ?? "HEAD";
const root = resolve(new URL("../../..", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"));
const evidence = JSON.parse(readFileSync(evidencePath, "utf8"));

assertExact(Object.keys(evidence).sort(), [
  "focusRestorationTraces", "formatVersion", "gates", "knownGaps", "logs",
  "primitiveContentSurfaceAllowlist", "publicConsumerAudit", "runtime",
  "semanticSnapshots", "source", "specificationSha256", "status", "sternVersion",
  "traversalTraces",
].sort(), "top-level keys");
assert(evidence.formatVersion === 1, "formatVersion must be 1");
assert(evidence.sternVersion === "1.0.0-rc.2.dev", "unexpected Stern version");
assert(evidence.specificationSha256 === SPEC_SHA256, "specification hash mismatch");
assert(["incomplete", "final"].includes(evidence.status), "invalid status");

const wantedCommit = git("rev-parse", `${sourceRef}^{commit}`);
const wantedTree = git("rev-parse", `${sourceRef}^{tree}`);
assert(evidence.source.commit === wantedCommit, "source commit is stale or mismatched");
assert(evidence.source.tree === wantedTree, "source tree is stale or mismatched");
assert(git("rev-parse", `${evidence.source.commit}^{tree}`) === evidence.source.tree,
  "recorded source commit does not own recorded tree");
assert(typeof evidence.source.generatedFromCleanWorktree === "boolean",
  "source cleanliness must be explicit");

assertExact(evidence.runtime.components.map(({ id }) => id), COMPONENTS, "component IDs");
for (const component of evidence.runtime.components) {
  assertRecord(component, "component");
  assertStringArray(component.workspaceIds, `${component.id}.workspaceIds`);
  assert(component.workspaceIds.every((id) => ["edit-workspace", "graph-workspace"].includes(id)),
    `${component.id} references unknown workspace`);
}
assertExact(evidence.runtime.workspaces.map(({ id }) => id),
  ["edit-workspace", "graph-workspace"], "workspace IDs");
assertExact(evidence.runtime.journeys.map(({ id, workspaceId }) => [id, workspaceId]),
  JOURNEYS, "journey contracts");
for (const journey of evidence.runtime.journeys) assertRecord(journey, "journey");

assert(evidence.semanticSnapshots.length === 2, "expected two semantic snapshots");
for (const [index, snapshot] of evidence.semanticSnapshots.entries()) {
  assert(snapshot.workspaceId === evidence.runtime.workspaces[index].id,
    "semantic snapshot workspace order mismatch");
  assert(Array.isArray(snapshot.nodes) && snapshot.nodes.length > 0, "empty semantic snapshot");
  const ids = new Set(snapshot.nodes.map(({ id }) => id));
  assert(ids.size === snapshot.nodes.length, "duplicate semantic node ID");
  assert(ids.has(snapshot.root), "semantic root missing from node set");
  assert(snapshot.focusOrder.every((id) => ids.has(id)), "focus order references missing node");
  for (const node of snapshot.nodes) {
    assert(node.parent === null || ids.has(node.parent), "semantic parent missing from node set");
    assert(node.children.every((id) => ids.has(id)), "semantic child missing from node set");
  }
}
assert(evidence.traversalTraces.some(({ input }) => input === "Tab"), "missing Tab traversal trace");
assert(evidence.focusRestorationTraces.length >= 2, "missing focus restoration traces");

const logs = [
  ...evidence.logs.actions,
  ...evidence.logs.stateTransitions,
  ...evidence.logs.failurePaths,
];
assert(logs.some(({ input }) => input === "pointer"), "missing pointer log");
assert(logs.some(({ input }) => input === "keyboard"), "missing keyboard log");
assert(evidence.logs.failurePaths.length >= 3, "missing failure-path logs");
assert(evidence.logs.failurePaths.every(({ optimisticMutation }) => optimisticMutation === false),
  "failure paths must reject optimistic mutation explicitly");

assert(evidence.publicConsumerAudit.passed === true, "public-consumer audit failed");
assert(evidence.publicConsumerAudit.publicFacadeDependency === true, "public facade missing");
assertExact(evidence.publicConsumerAudit.privateSternDependencies, [], "private dependencies");
assertExact(evidence.publicConsumerAudit.forbiddenSourceMatches, [], "forbidden source matches");
assertExact(evidence.primitiveContentSurfaceAllowlist.map(({ id }) => id), [
  "frame-output-consumption", "viewport-content-surface", "native-render-attachment",
], "primitive/content-surface allowlist");
for (const entry of evidence.primitiveContentSurfaceAllowlist) {
  assertStringArray(entry.allowedPatterns, `${entry.id}.allowedPatterns`);
  assertStringArray(entry.matchedSourcePaths, `${entry.id}.matchedSourcePaths`);
  assert(entry.matchedSourcePaths.length > 0, `${entry.id} has no audited match`);
}

assertExact(evidence.gates.map(({ id }) => id), GATES, "gate IDs");
for (const gate of evidence.gates) assertRecord(gate, "gate");
for (const gap of evidence.knownGaps) {
  assert(Number.isInteger(gap.issue), "known gap needs issue number");
  assertStringArray(gap.blocksGateIds, `${gap.id}.blocksGateIds`);
  for (const gateId of gap.blocksGateIds) {
    assert(GATES.includes(gateId), `${gap.id} blocks unknown gate`);
    assert(gate(gateId).status !== "passed", `${gateId} passed while ${gap.id} remains open`);
  }
}

const allComponents = evidence.runtime.components.every(({ status }) => status === "passed");
const allJourneys = evidence.runtime.journeys.every(({ status }) => status === "passed");
const allGates = evidence.gates.every(({ status }) => status === "passed");
if (evidence.status === "final") {
  assert(evidence.source.generatedFromCleanWorktree, "final evidence needs a clean source worktree");
  assert(allComponents && allJourneys && allGates, "final evidence cannot retain incomplete claims");
  assert(evidence.knownGaps.length === 0, "final evidence cannot retain known gaps");
} else {
  assert(!(allComponents && allJourneys && allGates), "complete evidence must use final status");
}

console.log(`runtime semantic evidence: PASS (${evidence.runtime.components.length} components, ${evidence.runtime.journeys.length} journeys, ${evidence.semanticSnapshots.length} snapshots; ${evidence.status})`);

function gate(id) {
  return evidence.gates.find((candidate) => candidate.id === id) ?? fail(`missing gate: ${id}`);
}

function assertRecord(record, label) {
  assert(record && typeof record === "object" && !Array.isArray(record), `${label} must be object`);
  assert(["passed", "failed", "notExecuted", "pending"].includes(record.status),
    `${record.id ?? label} has invalid status`);
  assertStringArray(record.evidenceRefs, `${record.id ?? label}.evidenceRefs`);
  assert(record.evidenceRefs.every((ref) => ref.startsWith("#/")),
    `${record.id ?? label} has invalid evidence link`);
}

function assertStringArray(value, label) {
  assert(Array.isArray(value) && value.every((item) => typeof item === "string"),
    `${label} must be a string array`);
  assert(new Set(value).size === value.length, `${label} must be unique`);
}

function assertExact(actual, expected, label) {
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${label} mismatch`);
}

function assert(condition, message) {
  if (!condition) fail(message);
}

function fail(message) {
  throw new Error(message);
}

function git(...args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
}

function parseArgs(args) {
  const parsed = {};
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === "--evidence") parsed.evidence = args[++index];
    else if (args[index] === "--source-ref") parsed.sourceRef = args[++index];
    else fail(`unknown argument: ${args[index]}`);
  }
  return parsed;
}
