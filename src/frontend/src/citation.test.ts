import { describe, expect, it } from 'vitest';
import { Cbor, NodeType, type HashTree, type VerifyFunc, reconstruct } from '@icp-sdk/core/agent';
import { lebEncode } from '@icp-sdk/core/candid';
import { Principal } from '@icp-sdk/core/principal';
import { hashCitation, verifyCitation } from './citation';
import { DiscoveryStatus, Vote, type Citation, type CertifiedCitation } from './bindings/platform';

const DER_PREFIX_HEX =
  '308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100';
const GOOD_ROOT_KEY_HEX = DER_PREFIX_HEX + 'ab'.repeat(96);
const WRONG_ROOT_KEY_HEX = DER_PREFIX_HEX + 'cd'.repeat(96);
const hexToBytes = (hex: string) => new Uint8Array(hex.match(/../g)!.map((b) => parseInt(b, 16)));

const CANISTER = Principal.fromUint8Array(new Uint8Array([7, 7, 7]));
const utf8 = (s: string) => new TextEncoder().encode(s);

const citation: Citation = {
  v: 1,
  reviewers: [
    {
      vote: Vote.Agree,
      credit: {
        at: 2n,
        aaa: Principal.fromUint8Array(new Uint8Array([2])),
        aaa_name_at_time: 'Beta',
        owner: Principal.fromUint8Array(new Uint8Array([102])),
        cycles_contributed: 5n,
      },
    },
  ],
  subject: {
    field: 'ceers',
    image_url: 'https://data.example.com/7/rgb.png',
    data_version: 1,
    dossier_url: 'https://data.example.com/7/dossier.json',
    subject_id: 7,
    ra_deg: 214.9,
    image_sha256: new Uint8Array(32).fill(1),
    dossier_sha256: new Uint8Array(32).fill(2),
    dec_deg: 52.8,
  },
  discovery_seq: 1n,
  public_id: 'SC-2026-000001',
  text: 'SC-2026-000001 — Gravitational Lens. Discovered by Alpha; reviewed by Beta. Space Compute, Confirmed 2026-09-07.',
  created_at: 1n,
  corroborators: [],
  rationale: 'possible arc near the galaxy core',
  category: 'lens',
  outcome: DiscoveryStatus.Confirmed,
  total_cycles_contributed: 15n,
  protocol_version: 1,
  resolved_at: 2n,
  discoverer: {
    at: 1n,
    aaa: Principal.fromUint8Array(new Uint8Array([1])),
    aaa_name_at_time: 'Alpha',
    owner: Principal.fromUint8Array(new Uint8Array([101])),
    cycles_contributed: 10n,
  },
};

async function buildFixture(): Promise<CertifiedCitation> {
  const citationHash = await hashCitation(citation);
  const witnessTree: HashTree = [NodeType.Labeled, utf8(citation.public_id) as never, [NodeType.Leaf, citationHash as never]];
  const citationsRoot = await reconstruct(witnessTree);
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
        [NodeType.Labeled, utf8('certified_data') as never, [NodeType.Leaf, citationsRoot as never]],
      ],
    ],
  ];
  const certificate = new Uint8Array(Cbor.encode({ tree: certTree, signature: new Uint8Array(96) }));
  const witness = new Uint8Array(Cbor.encode(witnessTree));
  return { citation, certificate, witness };
}

const goodKeyBytes = hexToBytes(GOOD_ROOT_KEY_HEX).slice(DER_PREFIX_HEX.length / 2);
const mockBlsVerify: VerifyFunc = (pk) =>
  pk.length === goodKeyBytes.length && pk.every((v, i) => v === goodKeyBytes[i]);

describe('verifyCitation', () => {
  it('verifies a real (synthetic-certificate) fixture', async () => {
    const cc = await buildFixture();
    await expect(
      verifyCitation(cc, CANISTER.toText(), hexToBytes(GOOD_ROOT_KEY_HEX), { blsVerify: mockBlsVerify }),
    ).resolves.toBe(true);
  });

  it('fails closed when the citation is tampered', async () => {
    const cc = await buildFixture();
    const tampered = { ...cc, citation: { ...cc.citation, text: 'forged' } };
    await expect(
      verifyCitation(tampered, CANISTER.toText(), hexToBytes(GOOD_ROOT_KEY_HEX), { blsVerify: mockBlsVerify }),
    ).resolves.toBe(false);
  });

  it('fails closed when the witness is tampered', async () => {
    const cc = await buildFixture();
    const tampered = { ...cc, witness: new Uint8Array([0xff]) };
    await expect(
      verifyCitation(tampered, CANISTER.toText(), hexToBytes(GOOD_ROOT_KEY_HEX), { blsVerify: mockBlsVerify }),
    ).resolves.toBe(false);
  });

  it('fails closed against the wrong root key', async () => {
    const cc = await buildFixture();
    await expect(
      verifyCitation(cc, CANISTER.toText(), hexToBytes(WRONG_ROOT_KEY_HEX), { blsVerify: mockBlsVerify }),
    ).resolves.toBe(false);
  });

  it('fails closed when the certificate is missing', async () => {
    const cc = await buildFixture();
    const tampered = { ...cc, certificate: new Uint8Array(0) };
    await expect(
      verifyCitation(tampered, CANISTER.toText(), hexToBytes(GOOD_ROOT_KEY_HEX), { blsVerify: mockBlsVerify }),
    ).resolves.toBe(false);
  });
});
