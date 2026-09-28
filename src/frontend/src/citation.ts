import { Certificate, Cbor, lookup_path, reconstruct, LookupPathStatus, type HashTree, type VerifyFunc } from '@icp-sdk/core/agent';
import { IDL } from '@icp-sdk/core/candid';
import { Principal } from '@icp-sdk/core/principal';
import { idlFactory } from './bindings/declarations/platform.did';
import type { CertifiedCitation, Citation } from './bindings/platform';

function citationType(): IDL.Type {
  const service = idlFactory({ IDL }) as IDL.ServiceClass;
  const getCitation = service._fields.find(([name]) => name === 'get_citation')![1] as IDL.FuncClass;
  const certified = (getCitation.retTypes[0] as IDL.OptClass<unknown>)._type as IDL.RecordClass;
  return certified._fields.find(([name]) => name === 'citation')![1];
}

const CITATION = citationType();

const variantTag = (v: object) => Object.keys(v)[0];

type RawCitation = Omit<Citation, 'outcome' | 'reviewers'> & {
  outcome: object;
  reviewers: Array<{ vote: object; credit: Citation['discoverer'] }>;
};

export function decodeCitation(bytes: Uint8Array): Citation {
  const [raw] = IDL.decode([CITATION], bytes) as unknown as [RawCitation];
  return {
    ...raw,
    outcome: variantTag(raw.outcome) as Citation['outcome'],
    reviewers: raw.reviewers.map((r) => ({ credit: r.credit, vote: variantTag(r.vote) as Citation['reviewers'][number]['vote'] })),
  };
}

async function sha256(bytes: Uint8Array): Promise<Uint8Array> {
  return new Uint8Array(await crypto.subtle.digest('SHA-256', new Uint8Array(bytes)));
}

function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  return a.length === b.length && a.every((v, i) => v === b[i]);
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

    const citation = decodeCitation(cc.citation_candid);
    const leaf = lookup_path([citation.public_id], witnessTree);
    if (leaf.status !== LookupPathStatus.Found) return false;

    return bytesEqual(leaf.value, await sha256(cc.citation_candid));
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

export function displayedCitation(cc: CertifiedCitation): Citation | null {
  try {
    return decodeCitation(cc.citation_candid);
  } catch {
    return null;
  }
}
