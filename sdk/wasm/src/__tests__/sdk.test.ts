import {
  sha256, toHex, fromHex, concat,
  keypairFromSeed, sign, verify, testAddress,
  InferenceRequestBuilder, verifySignedRequest,
  ProviderDiscovery, ProvaClient, batchSubmit,
  SdkError,
} from '../index.js';
import type { ProviderInfo, ModelId, InferenceResult } from '../index.js';

const testModel = (): ModelId => new Uint8Array(32).fill(0xAA);
const testKp = () => keypairFromSeed(new Uint8Array(32).fill(1));
const testKp2 = () => keypairFromSeed(new Uint8Array(32).fill(2));

// ── Crypto ───────────────────────────────────────────────────

describe('crypto', () => {
  test('sha256 of empty input', () => {
    const h = sha256(new Uint8Array(0));
    expect(toHex(h)).toBe('e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
  });

  test('sha256 of "abc"', () => {
    const h = sha256(new TextEncoder().encode('abc'));
    expect(toHex(h)).toBe('ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
  });

  test('hex roundtrip', () => {
    const bytes = new Uint8Array([0xde, 0xad, 0xbe, 0xef]);
    expect(fromHex(toHex(bytes))).toEqual(bytes);
  });

  test('concat', () => {
    const a = new Uint8Array([1, 2]);
    const b = new Uint8Array([3, 4, 5]);
    expect(concat(a, b)).toEqual(new Uint8Array([1, 2, 3, 4, 5]));
  });
});

// ── Keypair ──────────────────────────────────────────────────

describe('keypair', () => {
  test('deterministic from seed', () => {
    const kp1 = keypairFromSeed(new Uint8Array(32).fill(42));
    const kp2 = keypairFromSeed(new Uint8Array(32).fill(42));
    expect(kp1.address).toEqual(kp2.address);
    expect(kp1.secret).toEqual(kp2.secret);
  });

  test('different seeds → different addresses', () => {
    expect(toHex(testKp().address)).not.toBe(toHex(testKp2().address));
  });

  test('rejects non-32-byte seed', () => {
    expect(() => keypairFromSeed(new Uint8Array(16))).toThrow();
  });

  test('sign and verify', () => {
    const kp = testKp();
    const msg = new TextEncoder().encode('hello prova');
    const sig = sign(kp, msg);
    expect(verify(kp, msg, sig)).toBe(true);
  });

  test('verify wrong message fails', () => {
    const kp = testKp();
    const sig = sign(kp, new TextEncoder().encode('hello'));
    expect(verify(kp, new TextEncoder().encode('world'), sig)).toBe(false);
  });

  test('verify wrong key fails', () => {
    const sig = sign(testKp(), new TextEncoder().encode('test'));
    expect(verify(testKp2(), new TextEncoder().encode('test'), sig)).toBe(false);
  });

  test('testAddress creates 20-byte address', () => {
    const addr = testAddress(99);
    expect(addr.length).toBe(20);
    expect(addr[19]).toBe(99);
  });
});

// ── Request Builder ──────────────────────────────────────────

describe('InferenceRequestBuilder', () => {
  test('builds and signs valid request', () => {
    const kp = testKp();
    const signed = new InferenceRequestBuilder()
      .model(testModel())
      .input(new TextEncoder().encode('test input'))
      .maxPrice(1000n)
      .deadline(50)
      .build(kp, 10);

    expect(signed.request.deadline).toBe(60);
    expect(signed.request.maxPrice).toBe(1000n);
    expect(verifySignedRequest(signed, kp)).toBe(true);
  });

  test('missing model throws', () => {
    const kp = testKp();
    expect(() =>
      new InferenceRequestBuilder()
        .input(new Uint8Array([1]))
        .build(kp, 0)
    ).toThrow(SdkError);
  });

  test('missing input throws', () => {
    const kp = testKp();
    expect(() =>
      new InferenceRequestBuilder()
        .model(testModel())
        .build(kp, 0)
    ).toThrow(SdkError);
  });

  test('tampered request fails verification', () => {
    const kp = testKp();
    const signed = new InferenceRequestBuilder()
      .model(testModel())
      .input(new Uint8Array([1, 2, 3]))
      .maxPrice(100n)
      .build(kp, 0);

    signed.request.maxPrice = 999999n;
    expect(verifySignedRequest(signed, kp)).toBe(false);
  });
});

// ── Provider Discovery ───────────────────────────────────────

describe('ProviderDiscovery', () => {
  const mkProvider = (id: number, model: ModelId, price: bigint, rep: number, stake: bigint): ProviderInfo => ({
    address: testAddress(id),
    models: [model],
    price,
    reputation: rep,
    stake,
  });

  test('find providers sorted by score', () => {
    const disc = new ProviderDiscovery();
    disc.addProvider(mkProvider(1, testModel(), 100n, 0.9, 10000n));
    disc.addProvider(mkProvider(2, testModel(), 200n, 0.95, 20000n));
    disc.addProvider(mkProvider(3, new Uint8Array(32).fill(0xBB), 50n, 1.0, 50000n));

    const found = disc.findProviders(testModel(), 200n);
    expect(found.length).toBe(2);
    expect(found[0].address[19]).toBe(2); // higher score
  });

  test('max price filter', () => {
    const disc = new ProviderDiscovery();
    disc.addProvider(mkProvider(1, testModel(), 500n, 0.9, 5000n));
    expect(disc.findProviders(testModel(), 100n).length).toBe(0);
  });

  test('no matching model', () => {
    const disc = new ProviderDiscovery();
    disc.addProvider(mkProvider(1, new Uint8Array(32).fill(0xBB), 100n, 0.9, 5000n));
    expect(disc.findProviders(testModel(), 1000n).length).toBe(0);
  });

  test('cheapest', () => {
    const disc = new ProviderDiscovery();
    disc.addProvider(mkProvider(1, testModel(), 300n, 0.5, 1000n));
    disc.addProvider(mkProvider(2, testModel(), 100n, 0.9, 5000n));
    expect(disc.cheapest(testModel())!.address[19]).toBe(2);
  });

  test('best reputation', () => {
    const disc = new ProviderDiscovery();
    disc.addProvider(mkProvider(1, testModel(), 100n, 0.7, 1000n));
    disc.addProvider(mkProvider(2, testModel(), 500n, 0.99, 1000n));
    expect(disc.bestReputation(testModel())!.address[19]).toBe(2);
  });
});

// ── Client ───────────────────────────────────────────────────

describe('ProvaClient', () => {
  const mkSigned = (kp = testKp()) =>
    new InferenceRequestBuilder()
      .model(testModel())
      .input(new Uint8Array([1, 2, 3]))
      .maxPrice(100n)
      .build(kp, 0);

  test('submit and check pending', () => {
    const client = new ProvaClient(testKp());
    const id = client.submit(mkSigned());
    expect(client.isPending(id)).toBe(true);
    expect(client.pendingCount).toBe(1);
  });

  test('cancel pending', () => {
    const client = new ProvaClient(testKp());
    const id = client.submit(mkSigned());
    expect(client.cancel(id)).toBe(true);
    expect(client.isPending(id)).toBe(false);
  });

  test('record result moves from pending to completed', () => {
    const client = new ProvaClient(testKp());
    const id = client.submit(mkSigned());
    const result: InferenceResult = {
      jobId: id,
      provider: testAddress(99),
      activationRoot: new Uint8Array(32).fill(0xBB),
      outputHash: new Uint8Array(32).fill(0xCC),
      epochCompleted: 5,
    };
    client.recordResult(result);
    expect(client.isPending(id)).toBe(false);
    expect(client.completedCount).toBe(1);
    expect(client.getResult(id)).toEqual(result);
  });

  test('batch submit', () => {
    const client = new ProvaClient(testKp());
    const requests = Array.from({ length: 5 }, () => mkSigned());
    const ids = batchSubmit(client, requests);
    expect(ids.length).toBe(5);
    expect(client.pendingCount).toBe(5);
  });

  test('client address matches keypair', () => {
    const kp = testKp();
    const client = new ProvaClient(kp);
    expect(client.address).toEqual(kp.address);
  });

  test('cancel non-existent returns false', () => {
    const client = new ProvaClient(testKp());
    expect(client.cancel(999)).toBe(false);
  });

  test('getResult non-existent returns undefined', () => {
    const client = new ProvaClient(testKp());
    expect(client.getResult(999)).toBeUndefined();
  });
});
