import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { HttpAgent, Actor } from "@icp-sdk/core/agent";
import { Ed25519KeyIdentity } from "@icp-sdk/core/identity";
import { IDL } from "@icp-sdk/core/candid";

const root = resolve(import.meta.dirname, "../../..");
const network = JSON.parse(execFileSync("icp", ["network", "status", "-e", "local", "--json"], { cwd: root }));
const ids = JSON.parse(readFileSync(resolve(root, ".icp/cache/mappings/local.ids.json"), "utf8"));
const rootKey = Uint8Array.from(Buffer.from(network.root_key, "hex"));

const identity = Ed25519KeyIdentity.generate();
const agent = await HttpAgent.create({ host: network.api_url, identity, rootKey });
const versionIdl = ({ IDL }) => IDL.Service({ version: IDL.Func([], [IDL.Text], ["query"]) });

console.log(`bot agent ${identity.getPrincipal().toText()} → ${network.api_url}`);
let ok = 0;
for (const name of ["platform", "payments", "treasury"]) {
  const actor = Actor.createActor(versionIdl, { agent, canisterId: ids[name] });
  const v = await actor.version();
  if (v !== `${name} 0.1.0`) throw new Error(`${name} answered ${v}`);
  console.log(`  ✓ ${name} (${ids[name]}) → ${v}`);
  ok += 1;
}
console.log(`BOT_OK ${ok}`);
