/**
 * This script checks for any Biome updates and then automatically
 * publishes a new version of the plugin if so.
 */
import { $, CargoToml, semver } from "automation";
import { Octokit } from "octokit";

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

// run the tests
$.logStep("Running tests...");
await $`cargo test`;

if (Deno.args.includes("--skip-publish")) {
  Deno.exit(0);
}

$.logStep(`Committing Biome version bump commit...`);
await $`git add .`;
const message = `${isPatchBump ? "fix" : "feat"}: update to Biome ${latestTag.tag}`;
await $`git commit -m ${message}`;

$.logStep("Bumping version in Cargo.toml...");
cargoToml.bumpCargoTomlVersion(isPatchBump ? "patch" : "minor");

// release
const newVersion = cargoToml.version();
$.logStep(`Committing and publishing ${newVersion}...`);
await $`git add .`;
await $`git commit -m ${newVersion}`;
await $`git push origin main`;
await $`git tag ${newVersion}`;
await $`git push origin ${newVersion}`;

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
