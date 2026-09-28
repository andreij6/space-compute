import { describe, expect, it } from 'vitest';
import { Cbor, NodeType, type HashTree, type VerifyFunc, reconstruct } from '@icp-sdk/core/agent';
import { lebEncode } from '@icp-sdk/core/candid';
import { Principal } from '@icp-sdk/core/principal';
import { decodeCitation, displayedCitation, verifyCitation, verifyCitationFromEnv } from './citation';
import type { CertifiedCitation } from './bindings/platform';
import vector from './__fixtures__/citation-vector.json';
import { renderToStaticMarkup } from 'react-dom/server';
import { MemoryRouter } from 'react-router-dom';
import { CertifiedCitationBlock } from './components/CertifiedCitationBlock';

const DER_PREFIX_HEX =
  '308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100';
const GOOD_ROOT_KEY_HEX = DER_PREFIX_HEX + 'ab'.repeat(96);
const WRONG_ROOT_KEY_HEX = DER_PREFIX_HEX + 'cd'.repeat(96);
const hexToBytes = (hex: string) => new Uint8Array(hex.match(/../g)!.map((b) => parseInt(b, 16)));
const toHex = (b: Uint8Array) => Array.from(b, (x) => x.toString(16).padStart(2, '0')).join('');

const CANISTER = Principal.fromUint8Array(new Uint8Array([7, 7, 7]));
const utf8 = (s: string) => new TextEncoder().encode(s);

const RUST_CITATION_CANDID = hexToBytes(vector.citation_candid);
const RUST_LEAF = hexToBytes(vector.leaf_sha256);

async function rustVectorCertifiedCitation(): Promise<CertifiedCitation> {
  const witnessTree: HashTree = [NodeType.Labeled, utf8(vector.public_id) as never, [NodeType.Leaf, RUST_LEAF as never]];
  const certifiedData = await reconstruct(witnessTree);
  const nowNs = BigInt(Date.now()) * 1_000_000n;
  const certTree: HashTree = [
    NodeType.Fork,
    [NodeType.Labeled, utf8('time') as never, [NodeType.Leaf, lebEncode(nowNs) as never]],
    [
      NodeType.Labeled,
      utf8('canister') as never,
      [
        NodeType.Labeled,
        CANISTER.toUint8Array() as never,
        [NodeType.Labeled, utf8('certified_data') as never, [NodeType.Leaf, certifiedData as never]],
      ],
    ],
  ];
  return {
    citation: decodeCitation(RUST_CITATION_CANDID),
    citation_candid: RUST_CITATION_CANDID,
    certificate: new Uint8Array(Cbor.encode({ tree: certTree, signature: new Uint8Array(96) })),
    witness: new Uint8Array(Cbor.encode(witnessTree)),
  };
}

const goodKeyBytes = hexToBytes(GOOD_ROOT_KEY_HEX).slice(DER_PREFIX_HEX.length / 2);
const mockBlsVerify: VerifyFunc = (pk) =>
  pk.length === goodKeyBytes.length && pk.every((v, i) => v === goodKeyBytes[i]);
const verify = (cc: CertifiedCitation, rootKeyHex = GOOD_ROOT_KEY_HEX) =>
  verifyCitation(cc, CANISTER.toText(), hexToBytes(rootKeyHex), { blsVerify: mockBlsVerify });

describe('verifyCitation against the Rust-generated vector (L-034)', () => {
  it('the vector is internally consistent: sha256(citation_candid) is the Rust leaf', async () => {
    const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new Uint8Array(RUST_CITATION_CANDID)));
    expect(toHex(digest)).toBe(vector.leaf_sha256);
  });

  it('decodes the displayed citation from the exact certified bytes', () => {
    const c = decodeCitation(RUST_CITATION_CANDID);
    expect(c.public_id).toBe(vector.public_id);
    expect(typeof c.outcome).toBe('string');
    expect(c.discoverer.aaa_name_at_time.length).toBeGreaterThan(0);
    expect(c.reviewers.every((r) => typeof r.vote === 'string')).toBe(true);
  });

  it('verifies by hashing citation_candid (never re-encoding)', async () => {
    await expect(verify(await rustVectorCertifiedCitation())).resolves.toBe(true);
  });

  it('ignores the convenience citation record: only the certified bytes count', async () => {
    const cc = await rustVectorCertifiedCitation();
    await expect(verify({ ...cc, citation: { ...cc.citation, text: 'forged' } })).resolves.toBe(true);
  });

  it('fails closed when one byte of citation_candid is tampered', async () => {
    const cc = await rustVectorCertifiedCitation();
    const tampered = new Uint8Array(cc.citation_candid);
    tampered[tampered.length - 1] ^= 0x01;
    await expect(verify({ ...cc, citation_candid: tampered })).resolves.toBe(false);
  });

  it('fails closed when citation_candid is not a Citation', async () => {
    const cc = await rustVectorCertifiedCitation();
    await expect(verify({ ...cc, citation_candid: new Uint8Array([1, 2, 3]) })).resolves.toBe(false);
    expect(displayedCitation({ ...cc, citation_candid: new Uint8Array([1, 2, 3]) })).toBeNull();
  });

  it('fails closed when the witness is tampered', async () => {
    const cc = await rustVectorCertifiedCitation();
    await expect(verify({ ...cc, witness: new Uint8Array([0xff]) })).resolves.toBe(false);
  });

  it('fails closed against the wrong root key', async () => {
    await expect(verify(await rustVectorCertifiedCitation(), WRONG_ROOT_KEY_HEX)).resolves.toBe(false);
  });

  it('fails closed when the certificate is missing', async () => {
    const cc = await rustVectorCertifiedCitation();
    await expect(verify({ ...cc, certificate: new Uint8Array(0) })).resolves.toBe(false);
  });

  it('fails closed (false, never a thrown error) when the canister env cannot be resolved', async () => {
    const cc = await rustVectorCertifiedCitation();
    await expect(
      verifyCitationFromEnv(cc, () => {
        throw new Error('ic_env cookie has no canister id for platform');
      }),
    ).resolves.toBe(false);
    await expect(verifyCitationFromEnv(cc, () => ({ canisterId: CANISTER.toText(), rootKey: undefined }))).resolves.toBe(false);
  });
});

describe('CertifiedCitationBlock with the Rust vector and a real-shape certificate', () => {
  it('shows Verified and the citation decoded from the certified bytes', async () => {
    const cc = await rustVectorCertifiedCitation();
    const forged = { ...cc, citation: { ...cc.citation, text: 'forged text' } };
    const verified = await verify(forged);
    const html = renderToStaticMarkup(
      <MemoryRouter>
        <CertifiedCitationBlock citation={displayedCitation(forged)!} verified={verified} />
      </MemoryRouter>,
    );
    expect(html).toContain('Verified');
    expect(html).not.toContain('Unverified');
    expect(html).toContain(decodeCitation(RUST_CITATION_CANDID).text);
    expect(html).not.toContain('forged text');
  });

  it('shows Unverified for a tampered certificate', async () => {
    const cc = await rustVectorCertifiedCitation();
    const verified = await verify(cc, WRONG_ROOT_KEY_HEX);
    const html = renderToStaticMarkup(
      <MemoryRouter>
        <CertifiedCitationBlock citation={displayedCitation(cc)!} verified={verified} />
      </MemoryRouter>,
    );
    expect(html).toMatch(/role="status"><span[^>]*>.*Unverified<\/span><\/span>/);
  });
});
