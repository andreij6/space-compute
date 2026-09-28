import { Certificate, Cbor, lookup_path, reconstruct, LookupPathStatus, type HashTree, type VerifyFunc } from '@icp-sdk/core/agent';
import { IDL } from '@icp-sdk/core/candid';
import { Principal } from '@icp-sdk/core/principal';
import type { CertifiedCitation, Citation, Credit } from './bindings/platform';

const CreditIdl = IDL.Record({
  at: IDL.Nat64,
  aaa: IDL.Principal,
  aaa_name_at_time: IDL.Text,
  owner: IDL.Principal,
  cycles_contributed: IDL.Nat,
});
const VoteIdl = IDL.Variant({ Disagree: IDL.Null, Agree: IDL.Null });
const ReviewerCreditIdl = IDL.Record({ vote: VoteIdl, credit: CreditIdl });
const DiscoveryStatusIdl = IDL.Variant({ UnderReview: IDL.Null, Confirmed: IDL.Null, Rejected: IDL.Null });
const SubjectRefIdl = IDL.Record({
  field: IDL.Text,
  image_url: IDL.Text,
  data_version: IDL.Nat16,
  dossier_url: IDL.Text,
  subject_id: IDL.Nat32,
  ra_deg: IDL.Float64,
  image_sha256: IDL.Vec(IDL.Nat8),
  dossier_sha256: IDL.Vec(IDL.Nat8),
  dec_deg: IDL.Float64,
});
const CitationIdl = IDL.Record({
  v: IDL.Nat8,
  reviewers: IDL.Vec(ReviewerCreditIdl),
  subject: SubjectRefIdl,
  discovery_seq: IDL.Nat64,
  public_id: IDL.Text,
  text: IDL.Text,
  created_at: IDL.Nat64,
  corroborators: IDL.Vec(CreditIdl),
  rationale: IDL.Text,
  category: IDL.Text,
  outcome: DiscoveryStatusIdl,
  total_cycles_contributed: IDL.Nat,
  protocol_version: IDL.Nat16,
  resolved_at: IDL.Nat64,
  discoverer: CreditIdl,
});

const toVariant = (tag: string) => ({ [tag]: null });
const candidCredit = (c: Credit) => c;
const candidCitation = (c: Citation) => ({
  ...c,
  outcome: toVariant(c.outcome),
  reviewers: c.reviewers.map((r) => ({ credit: candidCredit(r.credit), vote: toVariant(r.vote) })),
});

async function sha256(bytes: Uint8Array): Promise<Uint8Array> {
  return new Uint8Array(await crypto.subtle.digest('SHA-256', new Uint8Array(bytes)));
}

function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  return a.length === b.length && a.every((v, i) => v === b[i]);
}

export function hashCitation(citation: Citation): Promise<Uint8Array> {
  return sha256(new Uint8Array(IDL.encode([CitationIdl], [candidCitation(citation)])));
}

export type VerifyOptions = { blsVerify?: VerifyFunc };

export async function verifyCitation(
  cc: CertifiedCitation,
  canisterId: string,
  rootKey: Uint8Array,
  opts: VerifyOptions = {},
): Promise<boolean> {
  try {
    if (cc.certificate.length === 0 || cc.witness.length === 0) return false;
    const certificate = await Certificate.create({
      certificate: cc.certificate,
      rootKey,
      principal: { canisterId: Principal.fromText(canisterId) },
      blsVerify: opts.blsVerify,
    });
    const certifiedData = certificate.lookup_path(['canister', Principal.fromText(canisterId).toUint8Array(), 'certified_data']);
    if (certifiedData.status !== LookupPathStatus.Found) return false;

    const witnessTree = Cbor.decode<HashTree>(cc.witness);
    const witnessRoot = await reconstruct(witnessTree);
    if (!bytesEqual(witnessRoot, certifiedData.value)) return false;

    const leaf = lookup_path([cc.citation.public_id], witnessTree);
    if (leaf.status !== LookupPathStatus.Found) return false;

    const expected = await hashCitation(cc.citation);
    return bytesEqual(leaf.value, expected);
  } catch {
    return false;
  }
}

export type CitationEnv = { canisterId: string; rootKey: Uint8Array | undefined };

export async function verifyCitationFromEnv(
  cc: CertifiedCitation,
  resolve: () => CitationEnv,
  opts: VerifyOptions = {},
): Promise<boolean> {
  try {
    const { canisterId, rootKey } = resolve();
    if (!rootKey) return false;
    return await verifyCitation(cc, canisterId, rootKey, opts);
  } catch {
    return false;
  }
}
