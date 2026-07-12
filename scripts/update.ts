/**
 * This script checks for any Biome updates and then automatically
 * publishes a new version of the plugin if so.
 */
import { $, CargoToml, semver } from "automation";
import { Octokit } from "octokit";
import { aiFixBiomeUpdate } from "./ai_fix.ts";

const rootDirPath = $.path(import.meta.dirname!).parentOrThrow();
const cargoToml = new CargoToml(rootDirPath.join("Cargo.toml"));
const cargoTomlVersion = getBiomeCargoTomlTag(cargoToml.text());

$.logStep("Getting latest version...");
const latestTag = await getLatestBiomeTag();
if (cargoTomlVersion.tag === latestTag.tag) {
  $.log("No new update found. Exiting.");
  Deno.exit(0);
}

$.log("Found new version.");
$.logStep("Updating rust-toolchain.toml...");
await updateRustToolchain(latestTag.tag);
$.logStep("Updating Cargo.toml...");
const isPatchBump = cargoTomlVersion.version.major === latestTag.version.major
  && cargoTomlVersion.version.minor === latestTag.version.minor;
cargoToml.replaceAll(cargoTomlVersion.tag, latestTag.tag);

// Verify the update. A clean patch bump publishes exactly as before. A minor
// bump always gets an AI review (Biome may have added options without breaking
// the build), and a patch bump that fails the checks gets an AI fix attempt.
$.logStep("Running checks (test + clippy + wasm build)...");
const checks = await runChecks();

if (!isPatchBump || !checks.passed) {
  if (checks.passed) {
    $.logStep("Minor Biome update — running AI review for new/changed options...");
  } else {
    $.logStep("Patch update failed the checks — running AI fix...");
  }
  await aiFixBiomeUpdate({
    isPatchBump,
    fromVersion: cargoTomlVersion.tag,
    toVersion: latestTag.tag,
    checksPassed: checks.passed,
    // hand the failing output to the AI so it can go straight to fixing
    // instead of re-running the checks just to rediscover the errors.
    checkOutput: checks.output,
  });

  // the AI must leave the project in a passing state, otherwise fail the
  // workflow (nothing gets published and the maintainer is notified).
  $.logStep("Re-running checks after AI changes...");
  await assertChecks();
}

if (Deno.args.includes("--skip-publish")) {
  Deno.exit(0);
}

$.logStep(`Committing Biome version bump commit...`);
await $`git add .`;
const message = `${isPatchBump ? "fix" : "feat"}: update to Biome ${latestTag.tag}`;
await $`git commit -m ${message}`;

$.logStep("Bumping version in Cargo.toml...");
// reload from disk before bumping: the AI may have edited Cargo.toml (e.g.
// adding a new biome crate dependency), and the in-memory copy loaded at
// startup is stale. Writing the stale copy here would clobber those edits.
const releaseCargoToml = new CargoToml(rootDirPath.join("Cargo.toml"));
releaseCargoToml.bumpCargoTomlVersion(isPatchBump ? "patch" : "minor");

// release
const newVersion = releaseCargoToml.version();
$.logStep(`Committing and publishing ${newVersion}...`);
await $`git add .`;
await $`git commit -m ${newVersion}`;
await $`git push origin main`;
await $`git tag ${newVersion}`;
await $`git push origin ${newVersion}`;

// the checks that must pass before publishing. clippy is included because
// Codex runs it with warnings denied, so a clippy warning is as breaking as a
// test failure, and the wasm release build is included because that is what
// actually ships. `inheritPiped` + `captureCombined` streams the output to the
// CI log live while also capturing it so a failure can be handed to the AI.
async function runChecks(): Promise<{ passed: boolean; output: string }> {
  const results = [];
  for (const command of checkCommands()) {
    results.push(await capture(command()));
  }
  const failures = results.filter((r) => r.code !== 0);
  return {
    passed: failures.length === 0,
    output: failures.map((r) => r.combined).join("\n\n"),
  };
}

function capture(command: ReturnType<typeof $>) {
  return command
    .stdout("inheritPiped")
    .stderr("inheritPiped")
    .captureCombined()
    .noThrow();
}

// same checks as `runChecks`, but throws on the first failure so the workflow
// aborts before anything is committed, tagged, or published.
async function assertChecks(): Promise<void> {
  for (const command of checkCommands()) {
    await command();
  }
}

// a function (not a top-level const) so it is hoisted -- `runChecks` is called
// from top-level code above before a const declared here would be initialized.
function checkCommands() {
  return [
    () => $`cargo test`,
    () => $`cargo clippy --all-targets --all-features -- -D warnings`,
    () => $`cargo build --target wasm32-unknown-unknown --features wasm --release`,
  ];
}

function getBiomeCargoTomlTag(text: string) {
  const match = text.match(/git = \"https:\/\/github.com\/biomejs\/biome\", tag = \"([^\"]+)\"/);
  const tag = match?.[1];
  if (tag == null) {
    throw new Error("Could not find tag in Cargo.toml.");
  }
  $.logLight("Found tag in Cargo.toml:", tag);
  return {
    tag,
    version: tagToVersion(tag),
  };
}

async function getLatestBiomeTag() {
  const tags = await getGitTags();
  $.logLight("Found tags:\n" + tags.map(v => ` * ${v}`).join("\n"));
  const versionWithTag = tags
    .filter(tag => /^@biomejs\/biome@[0-9]+\.[0-9]+\.[0-9]+$/.test(tag))
    .map(tag => ({ tag, version: tagToVersion(tag) }));
  versionWithTag.sort((a, b) => semver.compare(a.version, b.version));
  const latestTag = versionWithTag.at(-1);
  if (latestTag == null) {
    throw new Error("Could not find tag.");
  }
  $.logLight("Latest tag:", latestTag.tag);
  return latestTag;
}

function tagToVersion(tag: string) {
  return semver.parse(tag.replace(/^@biomejs\/biome@/, ""));
}

async function updateRustToolchain(tag: string) {
  const content = await $.request(
    `https://raw.githubusercontent.com/biomejs/biome/${tag}/rust-toolchain.toml`,
  ).text();
  const match = content.match(/channel\s*=\s*"([^"]+)"/);
  if (match == null) {
    throw new Error("Could not find channel in biome's rust-toolchain.toml.");
  }
  const biomeRustVersion = match[1];
  const toolchainPath = rootDirPath.join("rust-toolchain.toml");
  const localContent = toolchainPath.readTextSync();
  const localMatch = localContent.match(/channel\s*=\s*"([^"]+)"/);
  if (localMatch == null) {
    throw new Error("Could not find channel in local rust-toolchain.toml.");
  }
  if (localMatch[1] !== biomeRustVersion) {
    $.log(`Updating Rust toolchain: ${localMatch[1]} -> ${biomeRustVersion}`);
    toolchainPath.writeTextSync(localContent.replace(localMatch[0], `channel = "${biomeRustVersion}"`));
  } else {
    $.log(`Rust toolchain already at ${biomeRustVersion}.`);
  }
}

async function getGitTags(): Promise<string[]> {
  const client = new Octokit();
  const tags = await client.paginate("GET /repos/{owner}/{repo}/tags", {
    owner: "biomejs",
    repo: "biome",
    per_page: 100,
  });
  return tags.map(tag => tag.name);
}

